//! Graceful shutdown: on `SIGTERM` or Ctrl-C a service starts draining —
//! readiness goes to 503 and every request but the probes is answered
//! `503 ATH-SYS-005` with `Retry-After`, retryable and nothing about the
//! caller (athenos's answer, now everyone's default) — while in-flight
//! requests finish.
//!
//! An event stream never finishes on its own, and the server's graceful
//! shutdown waits for every open response: with a browser tab open the
//! process would only end when the host kills it. [`EndStreamsLayer`] ends
//! each `text/event-stream` response as soon as draining begins (a clean
//! end of the body, so `EventSource` reconnects), and the process exits in
//! moments.
//!
//! ```ignore
//! let drain = Arc::new(Drain::default());
//! axum::serve(listener, app)
//!     .with_graceful_shutdown(shutdown::drain_on_signal(Arc::clone(&drain), "athenos"))
//!     .await?;
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use http::{header::CONTENT_TYPE, Request, Response};
use tokio::sync::Notify;
use tower::{Layer, Service};

use crate::service_auth::PROBE_PATHS;
use crate::util::{unavailable_response, BoxFuture, Render};

/// Whether the service is draining. Shared by the drain layer, `/ready`
/// and the shutdown future.
#[derive(Debug, Default)]
pub struct Drain {
    draining: AtomicBool,
    began: Notify,
}

impl Drain {
    pub fn begin_shutdown(&self) {
        self.draining.store(true, Ordering::SeqCst);
        self.began.notify_waiters();
    }

    /// Resolves once draining has begun (at once when it already has).
    pub fn draining(self: &Arc<Self>) -> impl Future<Output = ()> + Send + 'static {
        let drain = Arc::clone(self);
        async move {
            loop {
                let began = drain.began.notified();
                tokio::pin!(began);
                // Registered before the check: a `begin_shutdown` in
                // between still wakes this one.
                began.as_mut().enable();
                if drain.is_draining() {
                    return;
                }
                began.await;
            }
        }
    }

    pub fn is_ready(&self) -> bool {
        !self.is_draining()
    }

    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }
}

/// Resolves on Ctrl-C or `SIGTERM`.
pub async fn signal() {
    tokio::select! {
        result = tokio::signal::ctrl_c() => {
            if let Err(err) = result {
                tracing::warn!(error = %err, "failed to listen for ctrl_c shutdown signal");
                std::future::pending::<()>().await;
            }
        }
        () = terminate() => {}
    }
}

#[cfg(unix)]
async fn terminate() {
    match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
        Ok(mut signal) => {
            signal.recv().await;
        }
        Err(err) => {
            tracing::warn!(error = %err, "failed to listen for SIGTERM shutdown signal");
            std::future::pending::<()>().await;
        }
    }
}

#[cfg(not(unix))]
async fn terminate() {
    std::future::pending::<()>().await;
}

/// Waits for [`signal`], then drains: the future for
/// `with_graceful_shutdown`.
pub async fn drain_on_signal(drain: Arc<Drain>, service: &str) {
    signal().await;
    drain.begin_shutdown();
    tracing::info!("{service} entering drain mode");
}

/// What the drain layer refused.
#[derive(Debug, Clone, Copy)]
pub struct Draining;

/// `503 ATH-SYS-005` with `Retry-After: 1`, the request id and the retry
/// details — the same envelope athenos's `AppError` writes.
pub fn draining_response<B: From<String>>(_: &Draining) -> Response<B> {
    unavailable_response("ATH-SYS-005", "Service is restarting", "draining", 1)
}

/// Refuses every request but the exempt paths while draining.
pub struct DrainLayer<B> {
    drain: Arc<Drain>,
    exempt: Arc<Vec<String>>,
    reject: Render<Draining, B>,
}

impl<B> Clone for DrainLayer<B> {
    fn clone(&self) -> Self {
        Self {
            drain: Arc::clone(&self.drain),
            exempt: Arc::clone(&self.exempt),
            reject: Arc::clone(&self.reject),
        }
    }
}

impl<B> DrainLayer<B> {
    /// Exempt: [`PROBE_PATHS`].
    pub fn new(drain: Arc<Drain>, reject: Render<Draining, B>) -> Self {
        Self {
            drain,
            exempt: Arc::new(PROBE_PATHS.iter().map(|path| (*path).to_string()).collect()),
            reject,
        }
    }

    #[must_use]
    pub fn with_exempt_paths(mut self, paths: &[&str]) -> Self {
        self.exempt = Arc::new(paths.iter().map(|path| (*path).to_string()).collect());
        self
    }
}

impl<B: From<String> + 'static> DrainLayer<B> {
    /// With [`draining_response`].
    pub fn with_default_answer(drain: Arc<Drain>) -> Self {
        Self::new(drain, Arc::new(draining_response::<B>))
    }
}

impl<S, B> Layer<S> for DrainLayer<B> {
    type Service = DrainService<S, B>;

    fn layer(&self, inner: S) -> Self::Service {
        DrainService {
            inner,
            layer: self.clone(),
        }
    }
}

pub struct DrainService<S, B> {
    inner: S,
    layer: DrainLayer<B>,
}

impl<S: Clone, B> Clone for DrainService<S, B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            layer: self.layer.clone(),
        }
    }
}

impl<S, ReqB, ResB> Service<Request<ReqB>> for DrainService<S, ResB>
where
    S: Service<Request<ReqB>, Response = Response<ResB>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    ReqB: Send + 'static,
    ResB: Send + 'static,
{
    type Response = Response<ResB>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Response<ResB>, S::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqB>) -> Self::Future {
        let exempt = self
            .layer
            .exempt
            .iter()
            .any(|path| path == request.uri().path());
        if self.layer.drain.is_draining() && !exempt {
            return Box::pin(std::future::ready(Ok((self.layer.reject)(&Draining))));
        }
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move { inner.call(request).await })
    }
}

/// Ends every `text/event-stream` response once draining begins; any other
/// response passes through untouched. Without it the server's graceful
/// shutdown waits for the streams, which only end when the client goes.
///
/// Put it inside the [`DrainLayer`] (no matter where exactly: it only
/// wraps response bodies). The body ends cleanly, not with an error, so a
/// browser's `EventSource` reconnects — to the next process, or to a
/// `503` it retries.
pub struct EndStreamsLayer {
    drain: Arc<Drain>,
}

impl EndStreamsLayer {
    pub fn new(drain: Arc<Drain>) -> Self {
        Self { drain }
    }
}

impl Clone for EndStreamsLayer {
    fn clone(&self) -> Self {
        Self {
            drain: Arc::clone(&self.drain),
        }
    }
}

impl<S> Layer<S> for EndStreamsLayer {
    type Service = EndStreamsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        EndStreamsService {
            inner,
            drain: Arc::clone(&self.drain),
        }
    }
}

pub struct EndStreamsService<S> {
    inner: S,
    drain: Arc<Drain>,
}

impl<S: Clone> Clone for EndStreamsService<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            drain: Arc::clone(&self.drain),
        }
    }
}

impl<S, ReqB, ResB> Service<Request<ReqB>> for EndStreamsService<S>
where
    S: Service<Request<ReqB>, Response = Response<ResB>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    ReqB: Send + 'static,
    ResB: Send + 'static,
{
    type Response = Response<DrainBody<ResB>>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, S::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqB>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let drain = Arc::clone(&self.drain);
        Box::pin(async move {
            let response = inner.call(request).await?;
            let stream = is_event_stream(&response);
            Ok(response.map(|body| {
                if stream {
                    DrainBody::ending_on(body, &drain)
                } else {
                    DrainBody::untouched(body)
                }
            }))
        })
    }
}

fn is_event_stream<B>(response: &Response<B>) -> bool {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("text/event-stream"))
}

/// A response body that ends when draining begins (an event stream), or a
/// body passed through as it is ([`EndStreamsLayer`]).
pub struct DrainBody<B> {
    inner: B,
    draining: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
    ended: bool,
}

impl<B> DrainBody<B> {
    pub fn ending_on(inner: B, drain: &Arc<Drain>) -> Self {
        Self {
            inner,
            draining: Some(Box::pin(drain.draining())),
            ended: false,
        }
    }

    pub fn untouched(inner: B) -> Self {
        Self {
            inner,
            draining: None,
            ended: false,
        }
    }
}

impl<B> http_body::Body for DrainBody<B>
where
    B: http_body::Body + Unpin,
{
    type Data = B::Data;
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = self.get_mut();
        if this.ended {
            return Poll::Ready(None);
        }
        if let Some(draining) = this.draining.as_mut() {
            if draining.as_mut().poll(cx).is_ready() {
                this.draining = None;
                this.ended = true;
                return Poll::Ready(None);
            }
        }
        Pin::new(&mut this.inner).poll_frame(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.ended || self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        if self.draining.is_some() {
            // It may end before the inner body says so.
            http_body::SizeHint::default()
        } else {
            self.inner.size_hint()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use super::*;
    use http::header::RETRY_AFTER;
    use http::StatusCode;
    use tower::ServiceExt;

    async fn ok(_: Request<()>) -> Result<Response<String>, Infallible> {
        Ok(Response::new("ok".to_string()))
    }

    #[tokio::test]
    async fn while_draining_everything_but_the_probes_is_refused() {
        let drain = Arc::new(Drain::default());
        let service = DrainLayer::<String>::with_default_answer(Arc::clone(&drain))
            .layer(tower::service_fn(ok));
        let call = |path: &'static str| {
            let service = service.clone();
            async move {
                service
                    .oneshot(Request::builder().uri(path).body(()).unwrap())
                    .await
                    .unwrap()
            }
        };
        assert_eq!(call("/work/x").await.status(), StatusCode::OK);
        assert!(drain.is_ready());

        drain.begin_shutdown();
        assert!(!drain.is_ready());
        let refused = call("/work/x").await;
        assert_eq!(refused.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(refused.headers()[RETRY_AFTER], "1");
        let body: serde_json::Value = serde_json::from_str(refused.body()).unwrap();
        assert_eq!(body["error"]["code"], "ATH-SYS-005");
        assert_eq!(body["error"]["message_key"], "errors.unavailable");
        assert_eq!(body["error"]["retryable"], true);
        assert_eq!(body["error"]["details"]["reason"], "draining");
        assert_eq!(
            refused.headers()["x-request-id"],
            body["error"]["request_id"].as_str().unwrap()
        );
        for probe in PROBE_PATHS {
            assert_eq!(call(probe).await.status(), StatusCode::OK, "{probe}");
        }
    }

    /// A body that sends one frame, then nothing, forever: an event stream
    /// between two events.
    struct Quiet {
        first: Option<bytes::Bytes>,
    }

    impl Quiet {
        fn new() -> Self {
            Self {
                first: Some(bytes::Bytes::from_static(b": open\n\n")),
            }
        }
    }

    impl http_body::Body for Quiet {
        type Data = bytes::Bytes;
        type Error = Infallible;

        fn poll_frame(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
            match self.get_mut().first.take() {
                Some(data) => Poll::Ready(Some(Ok(http_body::Frame::data(data)))),
                None => Poll::Pending,
            }
        }
    }

    fn quiet_response(content_type: &'static str) -> Response<Quiet> {
        let mut response = Response::new(Quiet::new());
        response
            .headers_mut()
            .insert(CONTENT_TYPE, http::HeaderValue::from_static(content_type));
        response
    }

    async fn next_frame<B: http_body::Body + Unpin>(
        body: &mut B,
    ) -> Option<Result<http_body::Frame<B::Data>, B::Error>> {
        std::future::poll_fn(|cx| Pin::new(&mut *body).poll_frame(cx)).await
    }

    #[tokio::test]
    async fn draining_resolves_when_draining_begins_and_at_once_after() {
        let drain = Arc::new(Drain::default());
        let waiting = tokio::spawn(drain.draining());
        tokio::task::yield_now().await;
        assert!(!waiting.is_finished());
        drain.begin_shutdown();
        tokio::time::timeout(std::time::Duration::from_secs(1), waiting)
            .await
            .unwrap()
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_millis(10), drain.draining())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn an_event_stream_ends_when_draining_begins() {
        let drain = Arc::new(Drain::default());
        let service = EndStreamsLayer::new(Arc::clone(&drain)).layer(tower::service_fn(
            |_: Request<()>| async {
                Ok::<_, Infallible>(quiet_response("text/event-stream; charset=utf-8"))
            },
        ));
        let mut body = service.oneshot(Request::new(())).await.unwrap().into_body();
        let first = next_frame(&mut body).await.unwrap().unwrap();
        assert_eq!(first.into_data().unwrap(), ": open\n\n");
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), next_frame(&mut body))
                .await
                .is_err(),
            "the stream stays open while the service serves"
        );

        drain.begin_shutdown();
        let end = tokio::time::timeout(std::time::Duration::from_secs(1), next_frame(&mut body))
            .await
            .unwrap();
        assert!(end.is_none(), "a clean end, not an error");
        assert!(http_body::Body::is_end_stream(&body));
    }

    #[tokio::test]
    async fn other_responses_pass_through_while_draining() {
        let drain = Arc::new(Drain::default());
        drain.begin_shutdown();
        let service = EndStreamsLayer::new(Arc::clone(&drain)).layer(tower::service_fn(
            |_: Request<()>| async { Ok::<_, Infallible>(quiet_response("application/json")) },
        ));
        let mut body = service.oneshot(Request::new(())).await.unwrap().into_body();
        let first = next_frame(&mut body).await.unwrap().unwrap();
        assert_eq!(first.into_data().unwrap(), ": open\n\n");
    }

    /// A server with an open event stream and its graceful shutdown: with
    /// the layer the server is done at once, without it the shutdown waits
    /// for the client (the host's SIGKILL, in production).
    async fn shutdown_with_an_open_stream(end_streams: bool) -> bool {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let drain = Arc::new(Drain::default());
        let mut app = axum::Router::new().route(
            "/stream",
            axum::routing::get(|| async {
                quiet_response("text/event-stream").map(axum::body::Body::new)
            }),
        );
        if end_streams {
            app = app.layer(EndStreamsLayer::new(Arc::clone(&drain)));
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let server_drain = Arc::clone(&drain);
        let server = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = stopped.await;
                    server_drain.begin_shutdown();
                })
                .await
        });

        let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
        client
            .write_all(b"GET /stream HTTP/1.1\r\nhost: test\r\n\r\n")
            .await
            .unwrap();
        let mut seen = Vec::new();
        let mut buf = [0u8; 1024];
        while !String::from_utf8_lossy(&seen).contains(": open") {
            let n = client.read(&mut buf).await.unwrap();
            assert!(n > 0, "the stream closed before its first event");
            seen.extend_from_slice(&buf[..n]);
        }

        let _ = stop.send(());
        let finished = tokio::time::timeout(std::time::Duration::from_secs(2), server).await;
        if finished.is_ok() {
            // The client got the end of the chunked body, then the close.
            loop {
                let n = client.read(&mut buf).await.unwrap_or(0);
                if n == 0 {
                    break;
                }
                seen.extend_from_slice(&buf[..n]);
            }
            assert!(String::from_utf8_lossy(&seen).ends_with("0\r\n\r\n"));
        }
        finished.is_ok()
    }

    #[tokio::test]
    async fn a_server_with_an_open_event_stream_shuts_down_at_once() {
        assert!(shutdown_with_an_open_stream(true).await);
    }

    #[tokio::test]
    async fn without_the_layer_an_open_event_stream_holds_the_shutdown() {
        assert!(!shutdown_with_an_open_stream(false).await);
    }
}
