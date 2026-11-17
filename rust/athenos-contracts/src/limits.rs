//! Length limits at the domain's edges, one source for the core and the web
//! (BE-16, Q-051, step 3.10).
//!
//! The core's entities check against these; `tests/ts_bindings.rs` renders
//! them into `src/generated/limits/limits.ts` for the web, so a form and the
//! server cannot disagree about a number. Lengths are Unicode scalar values
//! (`chars().count()`, what a person counts), except where a name says
//! bytes: those bound what has to fit through the host's connector.
//!
//! Limits that already lived with their own contracts stay there:
//! [`crate::docs`] (`DOC_MAX_*`), [`crate::teams`] (`TEAM_*`),
//! [`crate::roles`] (`ROLE_*`), [`crate::calendar`] (`EVENT_*`).

/// A ticket title, after trimming.
pub const TICKET_TITLE_MIN_CHARS: usize = 3;
pub const TICKET_TITLE_MAX_CHARS: usize = 160;

/// A ticket description, in UTF-8 bytes. It travels as one parameter of a
/// single connector request, whose frame the host caps at 64 KiB
/// (`fenrir_module_kit::CONNECTOR_MAX_REQUEST_FRAME`) together with the
/// statement and every other parameter; 32 KiB leaves room for JSON
/// escaping and the largest ticket write (measured in the core's
/// `ticket_description_fits_the_connector_frame`). A doc page, which holds
/// nothing else, may hold 48 KiB.
pub const TICKET_DESCRIPTION_MAX_BYTES: usize = 32 * 1024;

/// A comment, after trimming. Characters, not bytes (BE-16: "4000" used to
/// be bytes, so an umlaut counted twice).
pub const COMMENT_MIN_CHARS: usize = 2;
pub const COMMENT_MAX_CHARS: usize = 4000;

/// The note of a time booking.
pub const WORK_LOG_NOTE_MAX_CHARS: usize = 500;

/// A workspace's name.
pub const WORKSPACE_NAME_MAX_CHARS: usize = 100;

/// A sprint's name.
pub const SPRINT_NAME_MAX_CHARS: usize = 80;

/// A release's name (stored as a milestone).
pub const RELEASE_NAME_MAX_CHARS: usize = 80;

/// A label's name.
pub const LABEL_NAME_MAX_CHARS: usize = 32;

/// One attachment, in bytes (W4-B). The files live in the database, which
/// the host's connector feeds 64 KiB at a time: 10 MiB is a screenshot, a
/// log or a short screen recording, and stays a second or two of writes.
pub const ATTACHMENT_MAX_BYTES: usize = 10 * 1024 * 1024;

/// All attachments of one workspace together, in bytes: its trash and its
/// unfinished uploads included. Bounds what one workspace can put into the
/// shared database (a hundred files at the largest).
pub const ATTACHMENT_WORKSPACE_MAX_BYTES: usize = 1024 * 1024 * 1024;

/// Attachments of one ticket (its comments' included), not counting the
/// trash.
pub const ATTACHMENTS_PER_TICKET_MAX: usize = 100;

/// An attachment's name, after sanitising.
pub const ATTACHMENT_NAME_MAX_CHARS: usize = 200;

/// One part of an upload or a download between gateway and core, in
/// bytes: base64 in the JSON of the host's call envelope, which takes at
/// most 2 MiB per request. A multiple of
/// [`crate::attachments::ATTACHMENT_CHUNK_BYTES`].
pub const ATTACHMENT_PART_MAX_BYTES: usize = 1024 * 1024;

/// The JSON body of an upload part at the core: a part in base64 and the
/// fields around it.
pub const ATTACHMENT_PART_BODY_MAX_BYTES: usize = ATTACHMENT_PART_MAX_BYTES / 3 * 4 + 64 * 1024;

/// A profile picture, in bytes (pp1): the browser sends a 256×256 WebP or
/// PNG (tens of KiB); the rest is room for a larger PNG from another
/// client. Sixteen pieces of [`crate::attachments::ATTACHMENT_CHUNK_BYTES`].
pub const AVATAR_MAX_BYTES: usize = 512 * 1024;
/// A profile picture's width and height at most, in pixels (read from the
/// header, never decoded).
pub const AVATAR_MAX_SIDE: u32 = 1024;
/// The colour profile a profile picture may carry at most, in bytes (an
/// sRGB or Display P3 profile is a few KiB).
pub const AVATAR_MAX_ICC_BYTES: usize = 16 * 1024;
/// The JSON body of `POST /user/avatar/upload` at the core: the picture in
/// base64 and the fields around it.
pub const AVATAR_UPLOAD_BODY_MAX_BYTES: usize = AVATAR_MAX_BYTES / 3 * 4 + 64 * 1024;

/// A ticket's "Release-Notiz" (W4-H), after trimming: the line the release
/// notes show instead of its title.
pub const TICKET_RELEASE_NOTE_MAX_CHARS: usize = 500;

/// A release-note template's title, a section heading and the heading of
/// the tickets no section takes: one line each.
pub const RELEASE_NOTE_TITLE_MAX_CHARS: usize = 160;
pub const RELEASE_NOTE_HEADING_MAX_CHARS: usize = 80;

/// A template's (or a release's) intro and outro, and its empty text.
pub const RELEASE_NOTE_TEXT_MAX_CHARS: usize = 4000;

/// The sections of a template, and the types plus labels one section
/// matches.
pub const RELEASE_NOTE_SECTIONS_MAX: usize = 12;
pub const RELEASE_NOTE_SECTION_MATCH_MAX: usize = 20;

/// A link template (`https://…/{key}`).
pub const RELEASE_NOTE_LINK_MAX_CHARS: usize = 300;

/// A stored release-notes draft, in UTF-8 bytes: one parameter of one
/// connector request (64 KiB frame), like a ticket description.
pub const RELEASE_NOTES_DRAFT_MAX_BYTES: usize = 32 * 1024;

/// Tickets one release may leave out, and ticket ids its section orders
/// name together: they travel in one connector request.
pub const RELEASE_NOTES_EXCLUDED_MAX: usize = 200;
pub const RELEASE_NOTES_ORDER_MAX: usize = 300;

/// Over the platform's workspace quota (`quota_workspaces_per_account`):
/// `409` on a workspace create (W9A-06).
pub const QUOTA_WORKSPACES_CODE: &str = "ATH-QUOTA-001";
/// Over the account's storage quota (`quota_storage_mb_per_account`, all
/// the workspaces the owner owns): `409` on an upload. The workspace's own
/// limit ([`ATTACHMENT_WORKSPACE_MAX_BYTES`]) stays.
pub const QUOTA_STORAGE_CODE: &str = "ATH-QUOTA-002";
/// Over the tickets a person may create per rolling 24 hours
/// (`quota_tickets_per_day`): `429`.
pub const QUOTA_TICKETS_PER_DAY_CODE: &str = "ATH-QUOTA-003";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachments::ATTACHMENT_CHUNK_BYTES;

    /// The host's call envelope: one request carries at most 2 MiB.
    const HOST_ENVELOPE_BYTES: usize = 2 * 1024 * 1024;

    #[test]
    fn avatar_limits_fit_the_envelope() {
        // Checked when the test compiles: the limits are constants.
        const {
            // Whole pieces in storage, and one part between gateway and core.
            assert!(AVATAR_MAX_BYTES % ATTACHMENT_CHUNK_BYTES == 0);
            assert!(AVATAR_MAX_BYTES <= ATTACHMENT_PART_MAX_BYTES);
            // The largest picture in base64 plus room for the other fields
            // fits the core's body limit, and that limit fits the envelope.
            assert!(AVATAR_MAX_BYTES.div_ceil(3) * 4 + 1024 <= AVATAR_UPLOAD_BODY_MAX_BYTES);
            assert!(AVATAR_UPLOAD_BODY_MAX_BYTES < HOST_ENVELOPE_BYTES);
            assert!(AVATAR_MAX_ICC_BYTES < AVATAR_MAX_BYTES);
        }
    }
}
