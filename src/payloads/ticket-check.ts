/**
 * Ticket check wire types (intel layer R1): quality hints, the secret
 * scanner and "Ähnliche Tickets" while a ticket is written.
 *
 * Generated from `rust/athenos-contracts/src/ticket_check.rs` by ts-rs.
 * The server sends no sentences (except the writing assistance, whose
 * questions are the model package's own texts): render `ticket_check.rule.<rule>.*`,
 * `ticket_check.reason.<code>` and `ticket_check.secret.<kind>`. A
 * switched-off feature answers 403 `ATH-FEAT-001` with
 * `FeatureDisabledDetails`.
 */
export type { AbstainReason } from '../generated/intel/AbstainReason';
export type { Abstention } from '../generated/intel/Abstention';
export type { AssistanceScope } from '../generated/intel/AssistanceScope';
export type { AssistanceState } from '../generated/intel/AssistanceState';
export type { AssistanceUnavailable } from '../generated/intel/AssistanceUnavailable';
export type { AssistImprovement } from '../generated/intel/AssistImprovement';
export type { AssistLocale } from '../generated/intel/AssistLocale';
export type { BacklogReviewCounts } from '../generated/intel/BacklogReviewCounts';
export type { BacklogReviewGroups } from '../generated/intel/BacklogReviewGroups';
export type { BacklogReviewItem } from '../generated/intel/BacklogReviewItem';
export type { BacklogReviewPage } from '../generated/intel/BacklogReviewPage';
export type { BacklogReviewQuery } from '../generated/intel/BacklogReviewQuery';
export type { AskReporterItem } from '../generated/intel/AskReporterItem';
export type { AskReporterOutcome } from '../generated/intel/AskReporterOutcome';
export type { AskReporterRequest } from '../generated/intel/AskReporterRequest';
export type { AskReporterResponse } from '../generated/intel/AskReporterResponse';
export type { AskReporterResult } from '../generated/intel/AskReporterResult';
export type { BacklogReviewLast } from '../generated/intel/BacklogReviewLast';
export type { BacklogReviewLastQuery } from '../generated/intel/BacklogReviewLastQuery';
export type { BacklogReviewRunRequest } from '../generated/intel/BacklogReviewRunRequest';
export type { BacklogReviewTrend } from '../generated/intel/BacklogReviewTrend';
export type { BetaChecklistItem } from '../generated/intel/BetaChecklistItem';
export type { BetaChecklistState } from '../generated/intel/BetaChecklistState';
export type { BetaComparisonReason } from '../generated/intel/BetaComparisonReason';
export type { BetaCriterion } from '../generated/intel/BetaCriterion';
export type { BetaCriterionChange } from '../generated/intel/BetaCriterionChange';
export type { BetaCriterionState } from '../generated/intel/BetaCriterionState';
export type { BetaEditComparison } from '../generated/intel/BetaEditComparison';
export type { BetaQuality } from '../generated/intel/BetaQuality';
export type { BetaWritingChecklist } from '../generated/intel/BetaWritingChecklist';
export type { ClarifyingQuestion } from '../generated/intel/ClarifyingQuestion';
export type { ClarifyingQuestionOrigin } from '../generated/intel/ClarifyingQuestionOrigin';
export type { ClarifyingQuestionState } from '../generated/intel/ClarifyingQuestionState';
export type { ClarifyingQuestionTarget } from '../generated/intel/ClarifyingQuestionTarget';
export type { ReviewCriterion } from '../generated/intel/ReviewCriterion';
export type { ReviewTrendMark } from '../generated/intel/ReviewTrendMark';
export type { ReviewTrendUnavailable } from '../generated/intel/ReviewTrendUnavailable';
export type { StoredReviewHead } from '../generated/intel/StoredReviewHead';
export type { CheckField } from '../generated/intel/CheckField';
export type { CompareEditsRequest } from '../generated/intel/CompareEditsRequest';
export type { CompareEditsResponse } from '../generated/intel/CompareEditsResponse';
export type { ComparisonReason } from '../generated/intel/ComparisonReason';
export type { CriterionChange } from '../generated/intel/CriterionChange';
export type { CriterionResult } from '../generated/intel/CriterionResult';
export type { CriterionSource } from '../generated/intel/CriterionSource';
export type { CriterionState } from '../generated/intel/CriterionState';
export type { DecisionInput } from '../generated/intel/DecisionInput';
export type { DeleteIntelDataResponse } from '../generated/intel/DeleteIntelDataResponse';
export type { DuplicateCandidate } from '../generated/intel/DuplicateCandidate';
export type { EditComparison } from '../generated/intel/EditComparison';
export type { EditSide } from '../generated/intel/EditSide';
export type { FeatureDisabledDetails } from '../generated/intel/FeatureDisabledDetails';
export type { FixAction } from '../generated/intel/FixAction';
export type { HintSeverity } from '../generated/intel/HintSeverity';
export type { MetaScore } from '../generated/intel/MetaScore';
export type { ModelCriterionState } from '../generated/intel/ModelCriterionState';
export type { ModelInfo } from '../generated/intel/ModelInfo';
export type { QualityCriterion } from '../generated/intel/QualityCriterion';
export type { QualityEvidence } from '../generated/intel/QualityEvidence';
export type { QualityEvidenceCode } from '../generated/intel/QualityEvidenceCode';
export type { QualityHint } from '../generated/intel/QualityHint';
export type { QualityReport } from '../generated/intel/QualityReport';
export type { QualityRule } from '../generated/intel/QualityRule';
export type { Reason } from '../generated/intel/Reason';
export type { ReasonCode } from '../generated/intel/ReasonCode';
export type { RecordDecisionsRequest } from '../generated/intel/RecordDecisionsRequest';
export type { RecordDecisionsResponse } from '../generated/intel/RecordDecisionsResponse';
export type { RelatedDoc } from '../generated/intel/RelatedDoc';
export type { ReviewEngine } from '../generated/intel/ReviewEngine';
export type { ScoreBand } from '../generated/intel/ScoreBand';
export type { SecretFinding } from '../generated/intel/SecretFinding';
export type { SecretKind } from '../generated/intel/SecretKind';
export type { Strength } from '../generated/intel/Strength';
export type { SuggestionKind } from '../generated/intel/SuggestionKind';
export type { SwitchLevel } from '../generated/intel/SwitchLevel';
export type { TicketCheckFeature } from '../generated/intel/TicketCheckFeature';
export type { TicketCheckRequest } from '../generated/intel/TicketCheckRequest';
export type { TicketCheckResponse } from '../generated/intel/TicketCheckResponse';
export type { TriageField } from '../generated/intel/TriageField';
export type { TriageReason } from '../generated/intel/TriageReason';
export type { TriageReasonCode } from '../generated/intel/TriageReasonCode';
export type { TriageSuggestion } from '../generated/intel/TriageSuggestion';
export type { TriageTicketRef } from '../generated/intel/TriageTicketRef';
export type { Verdict } from '../generated/intel/Verdict';
export type { WritingAssistance } from '../generated/intel/WritingAssistance';

/**
 * "Athenos AI Ticketqualität (Beta)": the outer result contract of
 * `QualityReport.beta` and the checklist and edit comparison contract of
 * the current package (0.8.4-local-beta1, v2: with clarifying questions),
 * the v1 contracts of 0.8.3-local-beta1 that are still read (no
 * questions), every contract read (newest first),
 * `BetaCriterion.decision_source` when no overlay rule decided, the
 * question policy of a v2 result and its question limit. The same values
 * as the Rust constants.
 */
export const BETA_FEEDBACK_CONTRACT = 'athenos-quality-beta-feedback-v2';
export const BETA_FEEDBACK_CONTRACT_V1 = 'athenos-quality-beta-feedback-v1';
export const BETA_FEEDBACK_CONTRACTS: readonly string[] = [
  BETA_FEEDBACK_CONTRACT,
  BETA_FEEDBACK_CONTRACT_V1,
];
export const BETA_ASSISTANCE_CONTRACT = 'athenos-quality-beta-assistance-v2';
export const BETA_ASSISTANCE_CONTRACT_V1 = 'athenos-quality-beta-assistance-v1';
export const BETA_ASSISTANCE_CONTRACTS: readonly string[] = [
  BETA_ASSISTANCE_CONTRACT,
  BETA_ASSISTANCE_CONTRACT_V1,
];
export const BETA_MODEL_DECISION = 'onnx_0.8.1';
export const BETA_QUESTION_POLICY = 'athenos-clarifying-questions-v1';
export const BETA_MAX_CLARIFYING_QUESTIONS = 2;
