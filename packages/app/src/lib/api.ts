import type {
  PublicSubscription,
  ChannelStatusInfo,
  CompanyProfile,
  CompanyProfileImportApplyRequest,
  CompanyProfileImportApplyResult,
  CompanyProfileImportPreview,
  CompanyProfileSpaceSummary,
  CompanyProfileSummary,
  HistoryMsg,
  PublicChatBootstrapResponse,
  PublicHistoryPageResponse,
  PublicPushListResponse,
  PublicPushOpenResponse,
  PublicAuthUserInfo,
  PublicBillingConfig,
  PublicBillingStatus,
  PublicAdminInviteList,
  PublicAdminInviteMutation,
  PublicAdminUsageReport,
  InvestmentDecisionEvaluation,
  InvestmentEvidenceReviewQueue,
  InvestmentDecisionReplay,
  InvestmentDecisionReviewRecord,
  InvestmentDecisionReviewRequest,
  InvestmentCausalEvidenceReviewRecord,
  InvestmentCausalEvidenceReviewRequest,
  InvestmentCausalSourceReviewRecord,
  InvestmentCausalSourceReviewRequest,
  InvestmentFinancialEvidenceReviewRequest,
  InvestmentValuationInputReviewRequest,
  InvestmentValuationInputReviewResponse,
  InvestmentFinancialEvidenceReviewResponse,
  InvestmentCausalDatasetGovernance,
  InvestmentCausalDatasetGovernanceRequest,
  InvestmentCausalTrainingExperimentRegistry,
  InvestmentCausalTrainingExperimentRequest,
  InvestmentRewardGovernance,
  InvestmentRewardGovernanceRequest,
  InvestmentShadowProtocolGovernance,
  InvestmentShadowProtocolGovernanceRequest,
  InvestmentShadowImplementationRegistry,
  InvestmentShadowImplementationRegistrationRequest,
  HistoricalDecisionAnchorCandidate,
  HistoricalAnchorDiscoveryResponse,
  HistoricalAnchorDiscoveryScreeningRecord,
  HistoricalDecisionAnchorRegistry,
  HistoricalDecisionAnchorReview,
  CreateHistoricalDecisionAnchorCandidateRequest,
  ReviewHistoricalDecisionAnchorRequest,
  ScreenHistoricalAnchorDiscoveryRequest,
  HistoricalStateReconstructionCandidate,
  HistoricalStateReconstructionRegistry,
  HistoricalStateReconstructionReview,
  CreateHistoricalStateReconstructionRequest,
  ReviewHistoricalStateReconstructionRequest,
  HistoricalOutcomeGovernanceRegistry,
  HistoricalOutcomeGovernanceReview,
  ReviewHistoricalOutcomeGovernanceRequest,
  HistoricalOutcomeLabelerRegistry,
  HistoricalOutcomeLabelerReview,
  RegisterHistoricalOutcomeLabelerRequest,
  ReviewHistoricalOutcomeLabelerRequest,
  HistoricalOutcomePriceSnapshot,
  HistoricalOutcomePriceSnapshotRegistry,
  IngestHistoricalOutcomePriceSnapshotRequest,
  HistoricalOutcomeDryRunAuthorizationRegistry,
  HistoricalOutcomeDryRunAuthorizationReview,
  ReviewHistoricalOutcomeDryRunAuthorizationRequest,
  HistoricalOutcomeDryRunImplementationRegistry,
  RegisterHistoricalOutcomeDryRunImplementationRequest,
  HistoricalOutcomeDryRunRunAuthorizationRegistry,
  ReviewHistoricalOutcomeDryRunRunAuthorizationRequest,
  HistoricalOutcomeDryRunIsolatedRunnerRegistry,
  RegisterHistoricalOutcomeDryRunIsolatedRunnerRequest,
  HistoricalOutcomeDryRunFirstExecutionAuthorizationRegistry,
  ReviewHistoricalOutcomeDryRunFirstExecutionAuthorizationRequest,
  HistoricalOutcomeDryRunExecutionAttemptRegistry,
  InvokeHistoricalOutcomeDryRunRequest,
  HistoricalOutcomeDryRunOutputValidationRegistry,
  ValidateHistoricalOutcomeDryRunOutputRequest,
  HistoricalOutcomeLabelAdmissionRegistry,
  ReviewHistoricalOutcomeLabelAdmissionRequest,
  HistoricalOutcomeLabelMaterializationImplementationRegistry,
  RegisterHistoricalOutcomeLabelMaterializationImplementationRequest,
  HistoricalOutcomeLabelMaterializationRunAuthorizationRegistry,
  ReviewHistoricalOutcomeLabelMaterializationRunAuthorizationRequest,
  HistoricalOutcomeLabelMaterializationIsolatedRunnerRegistry,
  RegisterHistoricalOutcomeLabelMaterializationIsolatedRunnerRequest,
  HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationRegistry,
  ReviewHistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationRequest,
  HistoricalOutcomeLabelMaterializationExecutionAttemptRegistry,
  InvokeHistoricalOutcomeLabelMaterializationOnceRequest,
  HistoricalOutcomeLabelMaterializationOutputValidationRegistry,
  ValidateHistoricalOutcomeLabelMaterializationOutputRequest,
  HistoricalOutcomeLabelWriteAuthorizationRegistry,
  ReviewHistoricalOutcomeLabelWriteAuthorizationRequest,
  HistoricalOutcomeFormalLabelWriteRegistry,
  WriteHistoricalOutcomeFormalLabelOnceRequest,
  HistoricalOutcomeFormalLabelValidationRegistry,
  ValidateHistoricalOutcomeFormalLabelRequest,
  HistoricalOutcomeOfflineDatasetRegistry,
  AssembleHistoricalOutcomeOfflineDatasetRequest,
  HistoricalOutcomeOfflineDatasetGovernanceRegistry,
  ReviewHistoricalOutcomeOfflineDatasetGovernanceRequest,
  HistoricalOutcomeOfflineDatasetTransformationSpecRegistry,
  RegisterHistoricalOutcomeOfflineDatasetTransformationSpecRequest,
  HistoricalOutcomeOfflineDatasetTransformationSpecReviewRegistry,
  ReviewHistoricalOutcomeOfflineDatasetTransformationSpecRequest,
  HistoricalOutcomeOfflineDatasetTransformationImplementationRegistry,
  RegisterHistoricalOutcomeOfflineDatasetTransformationImplementationRequest,
  HistoricalOutcomeOfflineDatasetTransformationImplementationReviewRegistry,
  ReviewHistoricalOutcomeOfflineDatasetTransformationImplementationRequest,
  HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerRegistry,
  RegisterHistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerRequest,
  HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationRegistry,
  ReviewHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationRequest,
  HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptRegistry,
  InvokeHistoricalOutcomeOfflineDatasetTransformationOnceRequest,
  HistoricalOutcomeOfflineDatasetTransformationOutputValidationRegistry,
  ValidateHistoricalOutcomeOfflineDatasetTransformationOutputRequest,
  HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRegistry,
  ReviewHistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRequest,
  HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationRegistry,
  MaterializeHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest,
  HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRegistry,
  ValidateHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest,
  HistoricalOutcomeFeatureLabelJoinTargetSpecRegistry,
  RegisterHistoricalOutcomeFeatureLabelJoinTargetSpecRequest,
  HistoricalOutcomeFeatureLabelJoinTargetSpecReviewRegistry,
  ReviewHistoricalOutcomeFeatureLabelJoinTargetSpecRequest,
  HistoricalOutcomeFeatureLabelJoinTargetImplementationRegistry,
  RegisterHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest,
  HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewRegistry,
  ReviewHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest,
  HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerRegistry,
  RegisterHistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerRequest,
  HistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationRegistry,
  ReviewHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationRequest,
  HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptRegistry,
  InvokeHistoricalOutcomeFeatureLabelJoinTargetOnceRequest,
  HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRegistry,
  ValidateHistoricalOutcomeFeatureLabelJoinTargetOutputRequest,
  HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRegistry,
  ReviewHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRequest,
  HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationRegistry,
  MaterializeHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest,
  HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRegistry,
  ValidateHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest,
  HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRegistry,
  ReviewHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRequest,
  HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRegistry,
  CopyHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreRequest,
  HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRegistry,
  ValidateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRequest,
  HistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionRegistry,
  ReviewHistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionRequest,
  HistoricalOutcomeTrainingExperimentRegistrationRegistry,
  RegisterHistoricalOutcomeTrainingExperimentSuiteRequest,
  HistoricalOutcomeTrainingExperimentRegistrationReviewRegistry,
  ReviewHistoricalOutcomeTrainingExperimentRegistrationRequest,
  HistoricalOutcomeTrainingImplementationRegistry,
  RegisterHistoricalOutcomeTrainingImplementationRequest,
  HistoricalOutcomeTrainingImplementationReviewRegistry,
  ReviewHistoricalOutcomeTrainingImplementationRequest,
  HistoricalOutcomeTrainingIsolatedRunnerRegistry,
  RegisterHistoricalOutcomeTrainingIsolatedRunnerRequest,
  HistoricalOutcomeTrainingFirstExecutionAuthorizationRegistry,
  ReviewHistoricalOutcomeTrainingFirstExecutionAuthorizationRequest,
  HistoricalOutcomeTrainingExecutionAttemptRegistry,
  InvokeHistoricalOutcomeTrainingOnceRequest,
  HistoricalOutcomeTrainingOutputValidationRegistry,
  ValidateHistoricalOutcomeTrainingOutputRequest,
  HistoricalOutcomeValidationEvaluationImplementationRegistry,
  RegisterHistoricalOutcomeValidationEvaluationImplementationRequest,
  HistoricalOutcomeValidationEvaluationImplementationReviewRegistry,
  ReviewHistoricalOutcomeValidationEvaluationImplementationRequest,
  HistoricalOutcomeValidationEvaluationIsolatedRunnerRegistry,
  RegisterHistoricalOutcomeValidationEvaluationIsolatedRunnerRequest,
  HistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationRegistry,
  ReviewHistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationRequest,
  HistoricalOutcomeValidationEvaluationExecutionAttemptRegistry,
  InvokeHistoricalOutcomeValidationEvaluationOnceRequest,
  HistoricalOutcomeValidationEvaluationOutputValidationRegistry,
  ValidateHistoricalOutcomeValidationEvaluationOutputRequest,
  HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRegistry,
  ReviewHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRequest,
  HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewRegistry,
  ReviewHistoricalOutcomeSealedHoldoutEvaluationProtocolRequest,
  HistoricalOutcomeSealedHoldoutEvaluationImplementationRegistry,
  RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest,
  HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRegistry,
  ReviewHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest,
  HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRegistry,
  RegisterHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRequest,
  HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRegistry,
  ReviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRequest,
  HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry,
  InvokeHistoricalOutcomeSealedHoldoutEvaluationOnceRequest,
  HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRegistry,
  ValidateHistoricalOutcomeSealedHoldoutEvaluationOutputRequest,
  HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRegistry,
  ReviewHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRequest,
  ControlledShadowExperimentDesignRegistrationRegistry,
  RegisterControlledShadowExperimentDesignRequest,
  ControlledShadowExperimentDesignRegistrationReviewRegistry,
  ReviewControlledShadowExperimentDesignRegistrationRequest,
  ControlledShadowExperimentImplementationRegistry,
  RegisterControlledShadowExperimentImplementationRequest,
  ControlledShadowExperimentImplementationReviewRegistry,
  ReviewControlledShadowExperimentImplementationRequest,
  ControlledShadowExperimentIsolatedRunnerRegistry,
  RegisterControlledShadowExperimentIsolatedRunnerRequest,
  ControlledShadowExperimentFirstExecutionAuthorizationRegistry,
  ReviewControlledShadowExperimentFirstExecutionAuthorizationRequest,
  ControlledShadowExperimentExecutionAttemptRegistry,
  InvokeControlledShadowExperimentOnceRequest,
  ControlledShadowExperimentOutputValidationRegistry,
  ValidateControlledShadowExperimentOutputRequest,
  ControlledShadowForwardObservationProtocolRegistrationRegistry,
  RegisterControlledShadowForwardObservationProtocolRequest,
  ControlledShadowForwardObservationProtocolRegistrationReviewRegistry,
  ReviewControlledShadowForwardObservationProtocolRegistrationRequest,
  ControlledShadowForwardObservationImplementationRegistry,
  RegisterControlledShadowForwardObservationImplementationRequest,
  ControlledShadowForwardObservationImplementationReviewRegistry,
  ReviewControlledShadowForwardObservationImplementationRequest,
  ControlledShadowForwardObservationIsolatedRunnerRegistry,
  RegisterControlledShadowForwardObservationIsolatedRunnerRequest,
  ControlledShadowForwardObservationFirstExecutionAuthorizationRegistry,
  ReviewControlledShadowForwardObservationFirstExecutionAuthorizationRequest,
  ControlledShadowForwardObservationExecutionAttemptRegistry,
  InvokeControlledShadowForwardObservationOnceRequest,
  ControlledShadowForwardObservationOutputValidationRegistry,
  ValidateControlledShadowForwardObservationOutputRequest,
  ControlledShadowFirstNaturalForwardCycleAuthorizationRegistry,
  ReviewControlledShadowFirstNaturalForwardCycleAuthorizationRequest,
  ControlledShadowFirstNaturalForwardCycleClaimRegistry,
  ClaimControlledShadowFirstNaturalForwardCycleRequest,
  ControlledShadowMarketDataAdapterAuthorizationRegistry,
  ReviewControlledShadowMarketDataAdapterAuthorizationRequest,
  ControlledShadowMarketDataReceiptAttemptRegistry,
  ClaimAndReadControlledShadowMarketDataReceiptRequest,
  ControlledShadowMarketDataReceiptValidationRegistry,
  ValidateControlledShadowMarketDataReceiptRequest,
  ControlledShadowMarketDataParserSpecificationRegistry,
  RegisterControlledShadowMarketDataParserSpecificationRequest,
  ControlledShadowMarketDataParserSpecificationReviewRegistry,
  ReviewControlledShadowMarketDataParserSpecificationRequest,
  ControlledShadowMarketDataParserImplementationRegistry,
  RegisterControlledShadowMarketDataParserImplementationRequest,
  ControlledShadowMarketDataParserImplementationReviewRegistry,
  ReviewControlledShadowMarketDataParserImplementationRequest,
  ControlledShadowMarketDataParserIsolatedRunnerRegistry,
  RegisterControlledShadowMarketDataParserIsolatedRunnerRequest,
  ControlledShadowMarketDataParserFirstExecutionAuthorizationRegistry,
  ReviewControlledShadowMarketDataParserFirstExecutionAuthorizationRequest,
  ControlledShadowMarketDataParserExecutionAttemptClaimRegistry,
  ClaimControlledShadowMarketDataParserExecutionAttemptRequest,
  ControlledShadowMarketDataParserExecutionAttemptRegistry,
  ExecuteControlledShadowMarketDataParserAttemptRequest,
  ControlledShadowMarketDataParserOutputValidationRegistry,
  ValidateControlledShadowMarketDataParserOutputRequest,
  ControlledShadowObservationInputAdmissionRegistry,
  ReviewControlledShadowObservationInputAdmissionRequest,
  ControlledShadowObservationMaterializationSpecificationRegistry,
  RegisterControlledShadowObservationMaterializationSpecificationRequest,
  ControlledShadowObservationMaterializationSpecificationReviewRegistry,
  ReviewControlledShadowObservationMaterializationSpecificationRequest,
  ControlledShadowObservationMaterializationImplementationRegistry,
  RegisterControlledShadowObservationMaterializationImplementationRequest,
  ControlledShadowObservationMaterializationImplementationReviewRegistry,
  ReviewControlledShadowObservationMaterializationImplementationRequest,
  ControlledShadowObservationMaterializationIsolatedRunnerRegistry,
  RegisterControlledShadowObservationMaterializationIsolatedRunnerRequest,
  ControlledShadowObservationMaterializationFirstExecutionAuthorizationRegistry,
  ReviewControlledShadowObservationMaterializationFirstExecutionAuthorizationRequest,
  ControlledShadowObservationMaterializationExecutionAttemptClaimRegistry,
  ClaimControlledShadowObservationMaterializationExecutionAttemptRequest,
  ControlledShadowObservationMaterializationExecutionAttemptRegistry,
  ExecuteControlledShadowObservationMaterializationAttemptRequest,
  ControlledShadowObservationMaterializationOutputValidationRegistry,
  ValidateControlledShadowObservationMaterializationOutputRequest,
  ControlledShadowObservationEvidenceAdmissionRegistry,
  ReviewControlledShadowObservationEvidenceAdmissionRequest,
  ControlledShadowObservationLedgerTransitionSpecificationRegistry,
  RegisterControlledShadowObservationLedgerTransitionSpecificationRequest,
  ControlledShadowObservationLedgerTransitionSpecificationReviewRegistry,
  ReviewControlledShadowObservationLedgerTransitionSpecificationRequest,
  ControlledShadowObservationLedgerTransitionImplementationRegistry,
  RegisterControlledShadowObservationLedgerTransitionImplementationRequest,
  ControlledShadowObservationLedgerTransitionImplementationReviewRegistry,
  ReviewControlledShadowObservationLedgerTransitionImplementationRequest,
  ControlledShadowObservationLedgerTransitionIsolatedRunnerRegistry,
  RegisterControlledShadowObservationLedgerTransitionIsolatedRunnerRequest,
  ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationRegistry,
  ReviewControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationRequest,
  ControlledShadowObservationLedgerTransitionExecutionAttemptClaimRegistry,
  ClaimControlledShadowObservationLedgerTransitionExecutionAttemptRequest,
  ControlledShadowObservationLedgerTransitionExecutionAttemptRegistry,
  ExecuteControlledShadowObservationLedgerTransitionAttemptRequest,
  ControlledShadowObservationLedgerTransitionOutputValidationRegistry,
  ValidateControlledShadowObservationLedgerTransitionOutputRequest,
  ControlledShadowObservationLedgerTransitionCandidateAdmissionRegistry,
  ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest,
  OpeningPortfolioSnapshotGovernanceSpecificationRegistry,
  RegisterOpeningPortfolioSnapshotGovernanceSpecificationRequest,
  OpeningPortfolioSnapshotGovernanceSpecificationReviewRegistry,
  ReviewOpeningPortfolioSnapshotGovernanceSpecificationRequest,
  OpeningPortfolioSourceArtifactReceiptImplementationRegistry,
  RegisterOpeningPortfolioSourceArtifactReceiptImplementationRequest,
  OpeningPortfolioSourceArtifactReceiptImplementationReviewRegistry,
  ReviewOpeningPortfolioSourceArtifactReceiptImplementationRequest,
  OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRegistry,
  RegisterOpeningPortfolioSourceArtifactReceiptIsolatedReceiverRequest,
  OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRegistry,
  ReviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRequest,
  OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimRegistry,
  ClaimOpeningPortfolioSourceArtifactReceiptExecutionAttemptRequest,
  OpeningPortfolioSourceArtifactReceiptExecutionAttemptRegistry,
  ReceiveOpeningPortfolioSourceArtifactReceiptAttemptRequest,
  OpeningPortfolioSourceArtifactReceiptValidationRegistry,
  ValidateOpeningPortfolioSourceArtifactReceiptRequest,
  OpeningPortfolioSnapshotMaterializationImplementationRegistry,
  RegisterOpeningPortfolioSnapshotMaterializationImplementationRequest,
  OpeningPortfolioSnapshotMaterializationImplementationReviewRegistry,
  ReviewOpeningPortfolioSnapshotMaterializationImplementationRequest,
  OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRegistry,
  RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest,
  MetaInfo,
  SkillDetailInfo,
  SkillInfo,
  UserInfo,
  CronJobInfo,
  CronJobDetailInfo,
  CronJobUpsertInput,
  PortfolioInfo,
  PortfolioSummary,
  HoldingUpsertInput,
  LogEntry,
  DesktopChannelSettings,
  DesktopChannelSettingsInput,
  DesktopChannelSettingsUpdateResult,
  WebInviteActionResult,
  WebInviteInfo,
  FinanceCalendarPayload,
  PublicCommunityPage,
  PublicCommunityResource,
  CommunityForumPage,
  CommunityForumPost,
  CompanyRatingSnapshot,
  ValuationLabSnapshot,
  PortfolioNewsSnapshot,
  PositionManagementSnapshot,
  InfluencerDigestSnapshot,
  KeyEventChainSnapshot,
  WeeklyBriefPayload,
  ResearchLibraryBundle,
  ResearchLibraryItem,
  DailySignalHistoryItem,
  DailySignalKind,
  DailySignalReport,
} from "./types";
import type { ActorRef } from "./actors";
import {
  apiFetch,
  buildApiUrl,
  createEventSource,
  friendlyBackendErrorMessage,
} from "./backend";
import { useLocale } from "./i18n";
import {
  setCachedCommunityFeed,
  setCachedPublicUser,
} from "./public-session-cache";

export class ApiError extends Error {
  status: number;
  statusText: string;

  constructor(message: string, response: Response) {
    super(message);
    this.name = "ApiError";
    this.status = response.status;
    this.statusText = response.statusText;
  }
}

export function isUnauthorizedApiError(error: unknown) {
  return (
    error instanceof ApiError && (error.status === 401 || error.status === 403)
  );
}

async function parseJson<T>(response: Response): Promise<T> {
  const contentType = response.headers.get("content-type") ?? "";
  if (!response.ok) {
    const friendlyMessage = friendlyBackendErrorMessage(response.status);
    if (friendlyMessage) {
      throw new ApiError(friendlyMessage, response);
    }
    const text = await response.text();
    let message = "";
    try {
      const payload = JSON.parse(text) as { error?: string; message?: string };
      message = payload.error || payload.message || "";
    } catch {
      message = "";
    }
    throw new ApiError(message || text || response.statusText, response);
  }
  if (!contentType.toLowerCase().includes("application/json")) {
    const text = await response.text();
    const snippet = text.trim().slice(0, 80);
    throw new ApiError(
      `Expected JSON response but received ${contentType || "unknown content type"}${
        snippet ? `: ${snippet}` : ""
      }`,
      response,
    );
  }
  return response.json() as Promise<T>;
}

async function apiErrorFromResponse(response: Response): Promise<ApiError> {
  const friendlyMessage = friendlyBackendErrorMessage(response.status);
  if (friendlyMessage) {
    return new ApiError(friendlyMessage, response);
  }
  const text = await response.text();
  return new ApiError(text || response.statusText, response);
}

export async function getMeta() {
  const response = await apiFetch("/api/meta");
  return parseJson<MetaInfo>(response);
}

export async function getChannels() {
  const response = await apiFetch("/api/channels");
  return parseJson<ChannelStatusInfo[]>(response);
}

export async function getChannelSettings() {
  const response = await apiFetch("/api/channel-settings");
  return parseJson<DesktopChannelSettings>(response);
}

export async function putChannelSettings(
  settings: DesktopChannelSettingsInput,
) {
  const response = await apiFetch("/api/channel-settings", {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(settings),
  });
  return parseJson<DesktopChannelSettingsUpdateResult>(response);
}

export async function getUsers() {
  const response = await apiFetch("/api/users");
  return parseJson<UserInfo[]>(response);
}

export async function getWebInvites() {
  const response = await apiFetch("/api/web-users/invites");
  const payload = await parseJson<{ invites?: WebInviteInfo[] }>(response);
  return payload.invites ?? [];
}

export async function createWebInvite(phoneNumber: string) {
  const response = await apiFetch("/api/web-users/invites", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ phone_number: phoneNumber }),
  });
  const payload = await parseJson<{ invite: WebInviteInfo }>(response);
  return payload.invite;
}

async function mutateWebInvite(
  userId: string,
  action: "disable" | "enable" | "reset" | "api-key" | "api-key/reset",
) {
  const response = await apiFetch(
    `/api/web-users/invites/${encodeURIComponent(userId)}/${action}`,
    {
      method: "POST",
    },
  );
  return parseJson<WebInviteActionResult>(response);
}

export async function disableWebInvite(userId: string) {
  return mutateWebInvite(userId, "disable");
}

export async function enableWebInvite(userId: string) {
  return mutateWebInvite(userId, "enable");
}

export async function resetWebInvite(userId: string) {
  return mutateWebInvite(userId, "reset");
}

export async function getWebInviteApiKey(userId: string) {
  return mutateWebInvite(userId, "api-key");
}

export async function resetWebInviteApiKey(userId: string) {
  return mutateWebInvite(userId, "api-key/reset");
}

function actorQuery(actor: ActorRef) {
  const params = new URLSearchParams({
    channel: actor.channel,
    user_id: actor.user_id,
  });
  if (actor.channel_scope) params.set("channel_scope", actor.channel_scope);
  return params.toString();
}

export async function getHistory(sessionId: string) {
  const response = await apiFetch(
    `/api/history?session_id=${encodeURIComponent(sessionId)}`,
  );
  const payload = await parseJson<{ messages?: HistoryMsg[] }>(response);
  return payload.messages ?? [];
}

export async function getSkills() {
  const response = await apiFetch("/api/skills");
  return parseJson<SkillInfo[]>(response);
}

export async function getSkill(skillId: string) {
  const response = await apiFetch(`/api/skills/${encodeURIComponent(skillId)}`);
  return parseJson<SkillDetailInfo>(response);
}

export async function updateSkillState(skillId: string, enabled: boolean) {
  const response = await apiFetch(
    `/api/skills/${encodeURIComponent(skillId)}/state`,
    {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ enabled }),
    },
  );
  return parseJson<SkillInfo>(response);
}

export async function resetSkillRegistry() {
  const response = await apiFetch("/api/skills/reset", {
    method: "POST",
  });
  return parseJson<SkillInfo[]>(response);
}

export async function sendChat(
  actor: ActorRef,
  message: string,
  signal?: AbortSignal,
) {
  const response = await apiFetch("/api/chat", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      channel: actor.channel,
      user_id: actor.user_id,
      channel_scope: actor.channel_scope,
      message,
    }),
    signal,
  });

  if (!response.ok) {
    throw await apiErrorFromResponse(response);
  }

  if (!response.body) {
    throw new Error("missing response body");
  }

  return response.body;
}

export async function connectEvents(actor: ActorRef) {
  return createEventSource(`/api/events?${actorQuery(actor)}`);
}

export async function getPublicCaptchaConfig() {
  const response = await apiFetch("/api/public/auth/captcha/config");
  return parseJson<{
    enabled: boolean;
    region: string;
    prefix: string;
    scene_id: string;
    script_url: string;
  }>(response);
}

export async function publicSendSmsCode(
  phoneNumber: string,
  captchaVerifyParam?: string,
) {
  const response = await apiFetch("/api/public/auth/sms/send", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      phone_number: phoneNumber,
      captcha_verify_param: captchaVerifyParam,
    }),
  });
  await parseJson<{ ok: boolean }>(response);
}

export async function publicSmsLogin(input: {
  phone_number: string;
  verify_code: string;
  remember: boolean;
  tos_version: string;
}) {
  const response = await apiFetch("/api/public/auth/sms/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{ user: PublicAuthUserInfo }>(response);
  return payload.user;
}

export async function getPublicDevLoginConfig() {
  const response = await apiFetch("/api/public/auth/dev-login/config", {
    cache: "no-store",
  });
  return parseJson<{ enabled: boolean }>(response);
}

export async function publicDevLogin() {
  const response = await apiFetch("/api/public/auth/dev-login", {
    method: "POST",
  });
  const payload = await parseJson<{ user: PublicAuthUserInfo }>(response);
  setCachedPublicUser(payload.user);
  return payload.user;
}

export async function publicSendEmailCode(
  emailAddress: string,
  intent?: "stripe_checkout",
) {
  const response = await apiFetch("/api/public/auth/email/send", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email_address: emailAddress, intent }),
  });
  return parseJson<{ ok: boolean; message: string }>(response);
}

export async function publicEmailLogin(input: {
  email_address: string;
  verify_code: string;
  remember: boolean;
  tos_version: string;
}) {
  const response = await apiFetch("/api/public/auth/email/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{ user: PublicAuthUserInfo }>(response);
  return payload.user;
}

export async function publicLogout() {
  // Clear synchronously. The chat page intentionally does not await logout,
  // and no route may paint private data from the old session while the network
  // request is still in flight.
  setCachedPublicUser(null);
  setCachedCommunityFeed(null);
  try {
    const response = await apiFetch("/api/public/auth/logout", {
      method: "POST",
    });
    await parseJson<{ ok: boolean }>(response);
  } finally {
    // A later login must always obtain a grant for that session instead of
    // reusing the in-memory edge choice from the account that just logged out.
    resetPublicCommunityEdgeState();
  }
}

export async function getPublicAuthMe(signal?: AbortSignal) {
  const response = await apiFetch("/api/public/auth/me", { signal });
  const payload = await parseJson<{ user: PublicAuthUserInfo }>(response);
  // Remember it so the next route can paint before its own round-trip.
  setCachedPublicUser(payload.user);
  return payload.user;
}

export async function getPublicBillingConfig(signal?: AbortSignal) {
  const response = await apiFetch("/api/public/billing/config", {
    signal,
    cache: "no-store",
  });
  return parseJson<PublicBillingConfig>(response);
}

export async function getPublicBillingStatus(signal?: AbortSignal) {
  const response = await apiFetch("/api/public/billing/status", {
    signal,
    cache: "no-store",
  });
  return parseJson<PublicBillingStatus>(response);
}

export type StripeCheckoutOffer = "subscription" | "fixed_term";

export async function createStripeCheckout(offer: StripeCheckoutOffer) {
  const response = await apiFetch("/api/public/billing/checkout/stripe", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ offer }),
  });
  return parseJson<{ checkout_url: string }>(response);
}

export async function createStripePortal() {
  const response = await apiFetch("/api/public/billing/portal/stripe", {
    method: "POST",
  });
  return parseJson<{ portal_url: string }>(response);
}

const PUBLIC_ADMIN_ACTION_HEADERS = {
  "X-Hone-Admin-Action": "whitelist",
};

export async function getPublicAdminInvites(signal?: AbortSignal) {
  const response = await apiFetch("/api/public/admin/invites", {
    signal,
    cache: "no-store",
  });
  return parseJson<PublicAdminInviteList>(response);
}

export type PublicAdminUsageRangeDays = 14 | 30 | 90;

export async function getPublicAdminUsage(
  days: PublicAdminUsageRangeDays = 14,
  signal?: AbortSignal,
) {
  const response = await apiFetch(`/api/public/admin/usage?days=${days}`, {
    signal,
    cache: "no-store",
  });
  const report = await parseJson<PublicAdminUsageReport>(response);
  if (Number.isInteger(report.period_days) && report.period_days > 0) {
    return report;
  }
  const start = Date.parse(`${report.period_start}T00:00:00Z`);
  const end = Date.parse(`${report.period_end}T00:00:00Z`);
  const inferredDays =
    Number.isFinite(start) && Number.isFinite(end) && end >= start
      ? Math.floor((end - start) / 86_400_000) + 1
      : days;
  return { ...report, period_days: inferredDays };
}

export async function createPublicAdminInvite(phoneNumber: string) {
  const response = await apiFetch("/api/public/admin/invites", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...PUBLIC_ADMIN_ACTION_HEADERS,
    },
    body: JSON.stringify({ phone_number: phoneNumber }),
  });
  return parseJson<PublicAdminInviteMutation>(response);
}

export async function disablePublicAdminInvite(userId: string) {
  const response = await apiFetch(
    `/api/public/admin/invites/${encodeURIComponent(userId)}/disable`,
    {
      method: "POST",
      headers: PUBLIC_ADMIN_ACTION_HEADERS,
    },
  );
  return parseJson<PublicAdminInviteMutation>(response);
}

export async function getInvestmentDecisionEvaluation(
  symbol?: string,
  signal?: AbortSignal,
) {
  const query = new URLSearchParams();
  if (symbol?.trim()) query.set("symbol", symbol.trim().toUpperCase());
  const suffix = query.size ? `?${query}` : "";
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/evaluation${suffix}`,
    { signal, cache: "no-store" },
  );
  return parseJson<InvestmentDecisionEvaluation>(response);
}

export async function getInvestmentEvidenceReviewQueue(
  options: {
    symbol?: string;
    status?: "all" | "pending" | "accepted" | "rejected";
    kind?: "all" | "source_claim" | "operating_kpi" | "computed_comparison" | "computed_ratio";
    selection?: "full_queue" | "source_batch" | "old_wang_batch" | "active_batch";
    limit?: number;
  } = {},
  signal?: AbortSignal,
) {
  const query = new URLSearchParams();
  if (options.symbol?.trim()) query.set("symbol", options.symbol.trim().toUpperCase());
  query.set("status", options.status ?? "pending");
  query.set("kind", options.kind ?? "all");
  if (options.selection) query.set("selection", options.selection);
  query.set("limit", String(Math.max(1, Math.min(500, Math.trunc(options.limit ?? 100)))));
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/review-queue?${query}`,
    { signal, cache: "no-store" },
  );
  return parseJson<InvestmentEvidenceReviewQueue>(response);
}

export async function getInvestmentDecisionReplay(
  symbol: string,
  limit = 100,
  signal?: AbortSignal,
) {
  const normalized = symbol.trim().toUpperCase();
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/replay/${encodeURIComponent(normalized)}?limit=${Math.max(1, Math.min(500, Math.trunc(limit)))}`,
    { signal, cache: "no-store" },
  );
  return parseJson<InvestmentDecisionReplay>(response);
}

export async function reviewInvestmentDecision(
  symbol: string,
  sampleId: string,
  request: InvestmentDecisionReviewRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/review/${encodeURIComponent(symbol.trim().toUpperCase())}/${encodeURIComponent(sampleId)}`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<InvestmentDecisionReviewRecord>(response);
}

export async function reviewInvestmentCausalEvidence(
  symbol: string,
  sampleId: string,
  request: InvestmentCausalEvidenceReviewRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/causal-review/${encodeURIComponent(symbol.trim().toUpperCase())}/${encodeURIComponent(sampleId)}`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<InvestmentCausalEvidenceReviewRecord>(response);
}

export async function reviewInvestmentCausalSource(
  symbol: string,
  sampleId: string,
  request: InvestmentCausalSourceReviewRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/causal-source-review/${encodeURIComponent(symbol.trim().toUpperCase())}/${encodeURIComponent(sampleId)}`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<InvestmentCausalSourceReviewRecord>(response);
}

export async function getInvestmentFinancialEvidenceReviews(
  options: {
    symbol?: string;
    selection?: "active_batch" | "full_queue";
    limit?: number;
  } | string = {},
  signal?: AbortSignal,
) {
  const normalized = typeof options === "string" ? { symbol: options } : options;
  const query = new URLSearchParams();
  if (normalized.symbol?.trim()) {
    query.set("symbol", normalized.symbol.trim().toUpperCase());
  }
  if (normalized.selection) query.set("selection", normalized.selection);
  if (normalized.limit != null) {
    query.set("limit", String(Math.max(1, Math.min(20, Math.trunc(normalized.limit)))));
  }
  const suffix = query.size ? `?${query}` : "";
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/financial-evidence-reviews${suffix}`,
    { signal, cache: "no-store" },
  );
  return parseJson<InvestmentFinancialEvidenceReviewResponse>(response);
}

export async function reviewInvestmentFinancialEvidence(
  symbol: string,
  request: InvestmentFinancialEvidenceReviewRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/financial-evidence-reviews/${encodeURIComponent(symbol.trim().toUpperCase())}`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<InvestmentFinancialEvidenceReviewResponse>(response);
}

export async function getInvestmentValuationInputReviews(
  symbol?: string,
  signal?: AbortSignal,
) {
  const normalized = symbol?.trim().toUpperCase();
  const suffix = normalized ? `?symbol=${encodeURIComponent(normalized)}` : "";
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/valuation-input-reviews${suffix}`,
    { signal, cache: "no-store" },
  );
  return parseJson<InvestmentValuationInputReviewResponse>(response);
}

export async function reviewInvestmentValuationInputs(
  symbol: string,
  request: InvestmentValuationInputReviewRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/valuation-input-reviews/${encodeURIComponent(symbol.trim().toUpperCase())}`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<InvestmentValuationInputReviewResponse>(response);
}

export async function getInvestmentRewardGovernance(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/reward-governance",
    { signal, cache: "no-store" },
  );
  return parseJson<InvestmentRewardGovernance>(response);
}

export async function reviewInvestmentRewardGovernance(
  request: InvestmentRewardGovernanceRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/reward-governance",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<InvestmentRewardGovernance>(response);
}

export async function getInvestmentShadowProtocolGovernance(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/shadow-protocol-governance",
    { signal, cache: "no-store" },
  );
  return parseJson<InvestmentShadowProtocolGovernance>(response);
}

export async function reviewInvestmentShadowProtocolGovernance(
  request: InvestmentShadowProtocolGovernanceRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/shadow-protocol-governance",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<InvestmentShadowProtocolGovernance>(response);
}

export async function getInvestmentShadowImplementations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/shadow-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<InvestmentShadowImplementationRegistry>(response);
}

export async function registerInvestmentShadowImplementation(
  request: InvestmentShadowImplementationRegistrationRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/shadow-implementations",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<InvestmentShadowImplementationRegistry>(response);
}

export async function getHistoricalDecisionAnchors(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-anchors",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalDecisionAnchorRegistry>(response);
}

export async function createHistoricalDecisionAnchorCandidate(
  request: CreateHistoricalDecisionAnchorCandidateRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-anchors",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalDecisionAnchorCandidate>(response);
}

export async function getHistoricalAnchorDiscovery(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-anchor-discovery",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalAnchorDiscoveryResponse>(response);
}

export async function screenHistoricalAnchorDiscovery(
  suggestionId: string,
  request: ScreenHistoricalAnchorDiscoveryRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-anchor-discovery/${encodeURIComponent(suggestionId)}/screening`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalAnchorDiscoveryScreeningRecord>(response);
}

export async function reviewHistoricalDecisionAnchor(
  candidateId: string,
  request: ReviewHistoricalDecisionAnchorRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-anchors/${encodeURIComponent(candidateId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalDecisionAnchorReview>(response);
}

export async function getHistoricalStateReconstructions(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-state-reconstructions",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalStateReconstructionRegistry>(response);
}

export async function createHistoricalStateReconstruction(
  request: CreateHistoricalStateReconstructionRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-state-reconstructions",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalStateReconstructionCandidate>(response);
}

export async function reviewHistoricalStateReconstruction(
  reconstructionId: string,
  request: ReviewHistoricalStateReconstructionRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-state-reconstructions/${encodeURIComponent(reconstructionId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalStateReconstructionReview>(response);
}

export async function getHistoricalOutcomeGovernance(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-governance",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeGovernanceRegistry>(response);
}

export async function reviewHistoricalOutcomeGovernance(
  request: ReviewHistoricalOutcomeGovernanceRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-governance",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeGovernanceReview>(response);
}

export async function getHistoricalOutcomeLabelers(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-labelers",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeLabelerRegistry>(response);
}

export async function registerHistoricalOutcomeLabeler(
  request: RegisterHistoricalOutcomeLabelerRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-labelers",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeLabelerRegistry>(response);
}

export async function reviewHistoricalOutcomeLabeler(
  implementationId: string,
  request: ReviewHistoricalOutcomeLabelerRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-labelers/${encodeURIComponent(implementationId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeLabelerReview>(response);
}

export async function getHistoricalOutcomePriceSnapshots(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-price-snapshots",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomePriceSnapshotRegistry>(response);
}

export async function ingestHistoricalOutcomePriceSnapshot(
  request: IngestHistoricalOutcomePriceSnapshotRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-price-snapshots",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomePriceSnapshot>(response);
}

export async function getHistoricalOutcomeDryRunAuthorizations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-dry-run-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeDryRunAuthorizationRegistry>(response);
}

export async function reviewHistoricalOutcomeDryRunAuthorization(
  snapshotId: string,
  request: ReviewHistoricalOutcomeDryRunAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-dry-run-authorizations/${encodeURIComponent(snapshotId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeDryRunAuthorizationReview>(response);
}

export async function getHistoricalOutcomeDryRunImplementations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-dry-run-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeDryRunImplementationRegistry>(response);
}

export async function registerHistoricalOutcomeDryRunImplementation(
  request: RegisterHistoricalOutcomeDryRunImplementationRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-dry-run-implementations",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeDryRunImplementationRegistry>(response);
}

export async function getHistoricalOutcomeDryRunRunAuthorizations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-dry-run-run-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeDryRunRunAuthorizationRegistry>(response);
}

export async function reviewHistoricalOutcomeDryRunRunAuthorization(
  dryRunImplementationId: string,
  request: ReviewHistoricalOutcomeDryRunRunAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-dry-run-run-authorizations/${encodeURIComponent(dryRunImplementationId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeDryRunRunAuthorizationRegistry>(response);
}

export async function getHistoricalOutcomeDryRunIsolatedRunners(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-dry-run-isolated-runners",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeDryRunIsolatedRunnerRegistry>(response);
}

export async function registerHistoricalOutcomeDryRunIsolatedRunner(
  request: RegisterHistoricalOutcomeDryRunIsolatedRunnerRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-dry-run-isolated-runners",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeDryRunIsolatedRunnerRegistry>(response);
}

export async function getHistoricalOutcomeDryRunFirstExecutionAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-dry-run-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeDryRunFirstExecutionAuthorizationRegistry>(response);
}

export async function reviewHistoricalOutcomeDryRunFirstExecutionAuthorization(
  isolatedRunnerId: string,
  request: ReviewHistoricalOutcomeDryRunFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-dry-run-first-execution-authorizations/${encodeURIComponent(isolatedRunnerId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeDryRunFirstExecutionAuthorizationRegistry>(response);
}

export async function getHistoricalOutcomeDryRunExecutionAttempts(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-dry-run-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeDryRunExecutionAttemptRegistry>(response);
}

export async function invokeHistoricalOutcomeDryRunOnce(
  isolatedRunnerId: string,
  request: InvokeHistoricalOutcomeDryRunRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-dry-run-execution-attempts/${encodeURIComponent(isolatedRunnerId)}/invoke`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeDryRunExecutionAttemptRegistry>(response);
}

export async function getHistoricalOutcomeDryRunOutputValidations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-dry-run-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeDryRunOutputValidationRegistry>(response);
}

export async function validateHistoricalOutcomeDryRunOutput(
  attemptId: string,
  request: ValidateHistoricalOutcomeDryRunOutputRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-dry-run-output-validations/${encodeURIComponent(attemptId)}/validate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeDryRunOutputValidationRegistry>(response);
}

export async function getHistoricalOutcomeLabelAdmissionReviews(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-label-admission-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeLabelAdmissionRegistry>(response);
}

export async function reviewHistoricalOutcomeLabelAdmission(
  attemptId: string,
  request: ReviewHistoricalOutcomeLabelAdmissionRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-label-admission-reviews/${encodeURIComponent(attemptId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeLabelAdmissionRegistry>(response);
}

export async function getHistoricalOutcomeLabelMaterializationImplementations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-label-materialization-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeLabelMaterializationImplementationRegistry>(response);
}

export async function registerHistoricalOutcomeLabelMaterializationImplementation(
  request: RegisterHistoricalOutcomeLabelMaterializationImplementationRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-label-materialization-implementations",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeLabelMaterializationImplementationRegistry>(response);
}

export async function getHistoricalOutcomeLabelMaterializationRunAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-label-materialization-run-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeLabelMaterializationRunAuthorizationRegistry>(response);
}

export async function reviewHistoricalOutcomeLabelMaterializationRunAuthorization(
  materializationImplementationId: string,
  request: ReviewHistoricalOutcomeLabelMaterializationRunAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-label-materialization-run-authorizations/${encodeURIComponent(materializationImplementationId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeLabelMaterializationRunAuthorizationRegistry>(response);
}

export async function getHistoricalOutcomeLabelMaterializationIsolatedRunners(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-label-materialization-isolated-runners",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeLabelMaterializationIsolatedRunnerRegistry>(response);
}

export async function registerHistoricalOutcomeLabelMaterializationIsolatedRunner(
  request: RegisterHistoricalOutcomeLabelMaterializationIsolatedRunnerRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-label-materialization-isolated-runners",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeLabelMaterializationIsolatedRunnerRegistry>(response);
}

export async function getHistoricalOutcomeLabelMaterializationFirstExecutionAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-label-materialization-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationRegistry>(response);
}

export async function reviewHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization(
  isolatedRunnerId: string,
  request: ReviewHistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-label-materialization-first-execution-authorizations/${encodeURIComponent(isolatedRunnerId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationRegistry>(response);
}

export async function getHistoricalOutcomeLabelMaterializationExecutionAttempts(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-label-materialization-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeLabelMaterializationExecutionAttemptRegistry>(response);
}

export async function invokeHistoricalOutcomeLabelMaterializationOnce(
  isolatedRunnerId: string,
  request: InvokeHistoricalOutcomeLabelMaterializationOnceRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-label-materialization-execution-attempts/${encodeURIComponent(isolatedRunnerId)}/invoke-once`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeLabelMaterializationExecutionAttemptRegistry>(response);
}

export async function getHistoricalOutcomeLabelMaterializationOutputValidations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-label-materialization-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeLabelMaterializationOutputValidationRegistry>(response);
}

export async function validateHistoricalOutcomeLabelMaterializationOutput(
  attemptId: string,
  request: ValidateHistoricalOutcomeLabelMaterializationOutputRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-label-materialization-output-validations/${encodeURIComponent(attemptId)}/validate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeLabelMaterializationOutputValidationRegistry>(response);
}

export async function getHistoricalOutcomeLabelWriteAuthorizations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-label-write-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeLabelWriteAuthorizationRegistry>(response);
}

export async function reviewHistoricalOutcomeLabelWriteAuthorization(
  validationId: string,
  request: ReviewHistoricalOutcomeLabelWriteAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-label-write-authorizations/${encodeURIComponent(validationId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeLabelWriteAuthorizationRegistry>(response);
}

export async function getHistoricalOutcomeFormalLabelWrites(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-formal-label-writes",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFormalLabelWriteRegistry>(response);
}

export async function writeHistoricalOutcomeFormalLabelOnce(
  authorizationReviewId: string,
  request: WriteHistoricalOutcomeFormalLabelOnceRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-formal-label-writes/${encodeURIComponent(authorizationReviewId)}/write-once`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFormalLabelWriteRegistry>(response);
}

export async function getHistoricalOutcomeFormalLabelValidations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-formal-label-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFormalLabelValidationRegistry>(response);
}

export async function validateHistoricalOutcomeFormalLabel(
  labelId: string,
  request: ValidateHistoricalOutcomeFormalLabelRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-formal-label-validations/${encodeURIComponent(labelId)}/validate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFormalLabelValidationRegistry>(response);
}

export async function getHistoricalOutcomeOfflineDatasets(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-datasets",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetRegistry>(response);
}

export async function assembleHistoricalOutcomeOfflineDataset(
  request: AssembleHistoricalOutcomeOfflineDatasetRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-datasets",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetRegistry>(response);
}

export async function getHistoricalOutcomeOfflineDatasetGovernance(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-governance",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetGovernanceRegistry>(response);
}

export async function reviewHistoricalOutcomeOfflineDatasetGovernance(
  datasetId: string,
  request: ReviewHistoricalOutcomeOfflineDatasetGovernanceRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-offline-dataset-governance/${encodeURIComponent(datasetId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetGovernanceRegistry>(response);
}

export async function getHistoricalOutcomeOfflineDatasetTransformationSpecs(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-specs",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationSpecRegistry>(response);
}

export async function registerHistoricalOutcomeOfflineDatasetTransformationSpec(
  datasetId: string,
  request: RegisterHistoricalOutcomeOfflineDatasetTransformationSpecRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-specs/${encodeURIComponent(datasetId)}/register`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationSpecRegistry>(response);
}

export async function getHistoricalOutcomeOfflineDatasetTransformationSpecReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-spec-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationSpecReviewRegistry>(response);
}

export async function reviewHistoricalOutcomeOfflineDatasetTransformationSpec(
  transformationSpecId: string,
  request: ReviewHistoricalOutcomeOfflineDatasetTransformationSpecRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-spec-reviews/${encodeURIComponent(transformationSpecId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationSpecReviewRegistry>(response);
}

export async function getHistoricalOutcomeOfflineDatasetTransformationImplementations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationImplementationRegistry>(response);
}

export async function registerHistoricalOutcomeOfflineDatasetTransformationImplementation(
  request: RegisterHistoricalOutcomeOfflineDatasetTransformationImplementationRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-implementations",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationImplementationRegistry>(response);
}

export async function getHistoricalOutcomeOfflineDatasetTransformationImplementationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-implementation-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationImplementationReviewRegistry>(
    response,
  );
}

export async function reviewHistoricalOutcomeOfflineDatasetTransformationImplementation(
  implementationId: string,
  request: ReviewHistoricalOutcomeOfflineDatasetTransformationImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-implementation-reviews/${encodeURIComponent(implementationId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationImplementationReviewRegistry>(
    response,
  );
}

export async function getHistoricalOutcomeOfflineDatasetTransformationIsolatedRunners(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-isolated-runners",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerRegistry>(response);
}

export async function registerHistoricalOutcomeOfflineDatasetTransformationIsolatedRunner(
  request: RegisterHistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-isolated-runners",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationIsolatedRunnerRegistry>(response);
}

export async function getHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationRegistry>(
    response,
  );
}

export async function reviewHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization(
  isolatedRunnerId: string,
  request: ReviewHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-first-execution-authorizations/${encodeURIComponent(isolatedRunnerId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationRegistry>(
    response,
  );
}

export async function getHistoricalOutcomeOfflineDatasetTransformationExecutionAttempts(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptRegistry>(response);
}

export async function invokeHistoricalOutcomeOfflineDatasetTransformationOnce(
  isolatedRunnerId: string,
  request: InvokeHistoricalOutcomeOfflineDatasetTransformationOnceRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-execution-attempts/${encodeURIComponent(isolatedRunnerId)}/invoke-once`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptRegistry>(response);
}

export async function getHistoricalOutcomeOfflineDatasetTransformationOutputValidations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationOutputValidationRegistry>(response);
}

export async function validateHistoricalOutcomeOfflineDatasetTransformationOutput(
  attemptId: string,
  request: ValidateHistoricalOutcomeOfflineDatasetTransformationOutputRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-output-validations/${encodeURIComponent(attemptId)}/validate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationOutputValidationRegistry>(response);
}

export async function getHistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-candidate-admission-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRegistry>(response);
}

export async function reviewHistoricalOutcomeOfflineDatasetTransformationCandidateAdmission(
  attemptId: string,
  request: ReviewHistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-candidate-admission-reviews/${encodeURIComponent(attemptId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRegistry>(response);
}

export async function getHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-official-artifact-materializations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationRegistry>(response);
}

export async function materializeHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsOnce(
  attemptId: string,
  request: MaterializeHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-official-artifact-materializations/${encodeURIComponent(attemptId)}/materialize-once`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationRegistry>(response);
}

export async function getHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-official-artifact-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRegistry>(response);
}

export async function validateHistoricalOutcomeOfflineDatasetTransformationOfficialArtifacts(
  attemptId: string,
  request: ValidateHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-offline-dataset-transformation-official-artifact-output-validations/${encodeURIComponent(attemptId)}/validate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRegistry>(response);
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetSpecs(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-specs",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetSpecRegistry>(response);
}

export async function registerHistoricalOutcomeFeatureLabelJoinTargetSpec(
  attemptId: string,
  request: RegisterHistoricalOutcomeFeatureLabelJoinTargetSpecRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-specs/${encodeURIComponent(attemptId)}/register`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetSpecRegistry>(response);
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetSpecReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-spec-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetSpecReviewRegistry>(response);
}

export async function reviewHistoricalOutcomeFeatureLabelJoinTargetSpec(
  specificationId: string,
  request: ReviewHistoricalOutcomeFeatureLabelJoinTargetSpecRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-spec-reviews/${encodeURIComponent(specificationId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetSpecReviewRegistry>(response);
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetImplementations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetImplementationRegistry>(response);
}

export async function registerHistoricalOutcomeFeatureLabelJoinTargetImplementation(
  request: RegisterHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-implementations",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetImplementationRegistry>(response);
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetImplementationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-implementation-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewRegistry>(response);
}

export async function reviewHistoricalOutcomeFeatureLabelJoinTargetImplementation(
  implementationId: string,
  request: ReviewHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-implementation-reviews/${encodeURIComponent(implementationId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewRegistry>(response);
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetIsolatedRunners(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-isolated-runners",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerRegistry>(response);
}

export async function registerHistoricalOutcomeFeatureLabelJoinTargetIsolatedRunner(
  request: RegisterHistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-isolated-runners",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerRegistry>(response);
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationRegistry>(
    response,
  );
}

export async function reviewHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization(
  isolatedRunnerId: string,
  request: ReviewHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-first-execution-authorizations/${encodeURIComponent(isolatedRunnerId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationRegistry>(
    response,
  );
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempts(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptRegistry>(response);
}

export async function invokeHistoricalOutcomeFeatureLabelJoinTargetOnce(
  isolatedRunnerId: string,
  request: InvokeHistoricalOutcomeFeatureLabelJoinTargetOnceRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-execution-attempts/${encodeURIComponent(isolatedRunnerId)}/invoke-once`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptRegistry>(response);
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetOutputValidations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRegistry>(response);
}

export async function validateHistoricalOutcomeFeatureLabelJoinTargetOutput(
  attemptId: string,
  request: ValidateHistoricalOutcomeFeatureLabelJoinTargetOutputRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-output-validations/${encodeURIComponent(attemptId)}/validate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRegistry>(response);
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-candidate-admission-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRegistry>(response);
}

export async function reviewHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmission(
  attemptId: string,
  request: ReviewHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-candidate-admission-reviews/${encodeURIComponent(attemptId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRegistry>(response);
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-official-dataset-materializations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationRegistry>(
    response,
  );
}

export async function materializeHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOnce(
  attemptId: string,
  request: MaterializeHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-official-dataset-materializations/${encodeURIComponent(attemptId)}/materialize-once`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationRegistry>(
    response,
  );
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-official-dataset-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRegistry>(
    response,
  );
}

export async function validateHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset(
  attemptId: string,
  request: ValidateHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-official-dataset-output-validations/${encodeURIComponent(attemptId)}/validate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRegistry>(
    response,
  );
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-store-copy-admission-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRegistry>(
    response,
  );
}

export async function reviewHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmission(
  attemptId: string,
  request: ReviewHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-store-copy-admission-reviews/${encodeURIComponent(attemptId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRegistry>(
    response,
  );
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopies(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-store-copies",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRegistry>(response);
}

export async function copyHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreOnce(
  attemptId: string,
  request: CopyHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-store-copies/${encodeURIComponent(attemptId)}/copy-once`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRegistry>(response);
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-store-copy-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRegistry>(response);
}

export async function validateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopy(
  attemptId: string,
  request: ValidateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-store-copy-output-validations/${encodeURIComponent(attemptId)}/validate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRegistry>(response);
}

export async function getHistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-registration-admission-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionRegistry>(response);
}

export async function reviewHistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmission(
  attemptId: string,
  request: ReviewHistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-registration-admission-reviews/${encodeURIComponent(attemptId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionRegistry>(response);
}

export async function getHistoricalOutcomeTrainingExperimentRegistrations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-experiment-registrations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeTrainingExperimentRegistrationRegistry>(response);
}

export async function registerHistoricalOutcomeTrainingExperimentSuiteOnce(
  attemptId: string,
  request: RegisterHistoricalOutcomeTrainingExperimentSuiteRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-experiment-registrations/${encodeURIComponent(attemptId)}/register-once`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeTrainingExperimentRegistrationRegistry>(response);
}

export async function getHistoricalOutcomeTrainingExperimentRegistrationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-experiment-registration-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeTrainingExperimentRegistrationReviewRegistry>(response);
}

export async function reviewHistoricalOutcomeTrainingExperimentRegistration(
  attemptId: string,
  request: ReviewHistoricalOutcomeTrainingExperimentRegistrationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-experiment-registration-reviews/${encodeURIComponent(attemptId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeTrainingExperimentRegistrationReviewRegistry>(response);
}

export async function getHistoricalOutcomeTrainingImplementations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeTrainingImplementationRegistry>(response);
}

export async function registerHistoricalOutcomeTrainingImplementation(
  request: RegisterHistoricalOutcomeTrainingImplementationRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-implementations",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeTrainingImplementationRegistry>(response);
}

export async function getHistoricalOutcomeTrainingImplementationReviews(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-implementation-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeTrainingImplementationReviewRegistry>(response);
}

export async function reviewHistoricalOutcomeTrainingImplementation(
  implementationId: string,
  request: ReviewHistoricalOutcomeTrainingImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-implementation-reviews/${encodeURIComponent(implementationId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeTrainingImplementationReviewRegistry>(response);
}

export async function getHistoricalOutcomeTrainingIsolatedRunners(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-isolated-runners",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeTrainingIsolatedRunnerRegistry>(response);
}

export async function registerHistoricalOutcomeTrainingIsolatedRunner(
  request: RegisterHistoricalOutcomeTrainingIsolatedRunnerRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-isolated-runners",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeTrainingIsolatedRunnerRegistry>(response);
}

export async function getHistoricalOutcomeTrainingFirstExecutionAuthorizations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeTrainingFirstExecutionAuthorizationRegistry>(response);
}

export async function reviewHistoricalOutcomeTrainingFirstExecutionAuthorization(
  isolatedRunnerId: string,
  request: ReviewHistoricalOutcomeTrainingFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-first-execution-authorizations/${encodeURIComponent(isolatedRunnerId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeTrainingFirstExecutionAuthorizationRegistry>(response);
}

export async function getHistoricalOutcomeTrainingExecutionAttempts(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeTrainingExecutionAttemptRegistry>(response);
}

export async function invokeHistoricalOutcomeTrainingOnce(
  isolatedRunnerId: string,
  request: InvokeHistoricalOutcomeTrainingOnceRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-execution-attempts/${encodeURIComponent(isolatedRunnerId)}/invoke-once`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeTrainingExecutionAttemptRegistry>(response);
}

export async function getHistoricalOutcomeTrainingOutputValidations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeTrainingOutputValidationRegistry>(response);
}

export async function validateHistoricalOutcomeTrainingOutput(
  attemptId: string,
  request: ValidateHistoricalOutcomeTrainingOutputRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-output-validations/${encodeURIComponent(attemptId)}/validate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeTrainingOutputValidationRegistry>(response);
}

export async function getHistoricalOutcomeValidationEvaluationImplementations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationImplementationRegistry>(response);
}

export async function registerHistoricalOutcomeValidationEvaluationImplementation(
  request: RegisterHistoricalOutcomeValidationEvaluationImplementationRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-implementations",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationImplementationRegistry>(response);
}

export async function getHistoricalOutcomeValidationEvaluationImplementationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-implementation-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationImplementationReviewRegistry>(response);
}

export async function reviewHistoricalOutcomeValidationEvaluationImplementation(
  implementationId: string,
  request: ReviewHistoricalOutcomeValidationEvaluationImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-implementation-reviews/${encodeURIComponent(implementationId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationImplementationReviewRegistry>(response);
}

export async function getHistoricalOutcomeValidationEvaluationIsolatedRunners(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-isolated-runners",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationIsolatedRunnerRegistry>(response);
}

export async function registerHistoricalOutcomeValidationEvaluationIsolatedRunner(
  request: RegisterHistoricalOutcomeValidationEvaluationIsolatedRunnerRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-isolated-runners",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationIsolatedRunnerRegistry>(response);
}

export async function getHistoricalOutcomeValidationEvaluationFirstExecutionAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationRegistry>(
    response,
  );
}

export async function reviewHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization(
  isolatedRunnerId: string,
  request: ReviewHistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-first-execution-authorizations/${encodeURIComponent(isolatedRunnerId)}/reviews`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationRegistry>(
    response,
  );
}

export async function getHistoricalOutcomeValidationEvaluationExecutionAttempts(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationExecutionAttemptRegistry>(response);
}

export async function invokeHistoricalOutcomeValidationEvaluationOnce(
  isolatedRunnerId: string,
  request: InvokeHistoricalOutcomeValidationEvaluationOnceRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-execution-attempts/${encodeURIComponent(isolatedRunnerId)}/invoke-once`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationExecutionAttemptRegistry>(response);
}

export async function getHistoricalOutcomeValidationEvaluationOutputValidations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationOutputValidationRegistry>(response);
}

export async function validateHistoricalOutcomeValidationEvaluationOutput(
  attemptId: string,
  request: ValidateHistoricalOutcomeValidationEvaluationOutputRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-output-validations/${encodeURIComponent(attemptId)}/validate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationOutputValidationRegistry>(response);
}

export async function getHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-per-target-candidate-admission-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRegistry>(
    response,
  );
}

export async function reviewHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmission(
  attemptId: string,
  targetId: string,
  request: ReviewHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-validation-evaluation-per-target-candidate-admission-reviews/${encodeURIComponent(attemptId)}/targets/${encodeURIComponent(targetId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRegistry>(
    response,
  );
}

export async function getHistoricalOutcomeSealedHoldoutEvaluationProtocolReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-protocol-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewRegistry>(response);
}

export async function reviewHistoricalOutcomeSealedHoldoutEvaluationProtocol(
  attemptId: string,
  targetId: string,
  request: ReviewHistoricalOutcomeSealedHoldoutEvaluationProtocolRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-protocol-reviews/${encodeURIComponent(attemptId)}/targets/${encodeURIComponent(targetId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewRegistry>(response);
}

export async function getHistoricalOutcomeSealedHoldoutEvaluationImplementations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationImplementationRegistry>(response);
}

export async function registerHistoricalOutcomeSealedHoldoutEvaluationImplementation(
  request: RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-implementations",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationImplementationRegistry>(response);
}

export async function getHistoricalOutcomeSealedHoldoutEvaluationImplementationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-implementation-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRegistry>(response);
}

export async function reviewHistoricalOutcomeSealedHoldoutEvaluationImplementation(
  implementationId: string,
  request: ReviewHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-implementation-reviews/${encodeURIComponent(implementationId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRegistry>(response);
}

export async function getHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunners(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-isolated-runners",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRegistry>(response);
}

export async function registerHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunner(
  request: RegisterHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-isolated-runners",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRegistry>(response);
}

export async function getHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRegistry>(response);
}

export async function reviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization(
  isolatedRunnerId: string,
  request: ReviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-first-execution-authorizations/${encodeURIComponent(isolatedRunnerId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRegistry>(response);
}

export async function getHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempts(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry>(response);
}

export async function invokeHistoricalOutcomeSealedHoldoutEvaluationOnce(
  isolatedRunnerId: string,
  request: InvokeHistoricalOutcomeSealedHoldoutEvaluationOnceRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-execution-attempts/${encodeURIComponent(isolatedRunnerId)}/invoke-once`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry>(response);
}

export async function getHistoricalOutcomeSealedHoldoutEvaluationOutputValidations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRegistry>(response);
}

export async function validateHistoricalOutcomeSealedHoldoutEvaluationOutput(
  attemptId: string,
  request: ValidateHistoricalOutcomeSealedHoldoutEvaluationOutputRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-output-validations/${encodeURIComponent(attemptId)}/validate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRegistry>(response);
}

export async function getHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudications(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-confirmatory-result-adjudications",
    { signal, cache: "no-store" },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRegistry>(response);
}

export async function reviewHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudication(
  attemptId: string,
  request: ReviewHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-confirmatory-result-adjudications/${encodeURIComponent(attemptId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRegistry>(response);
}

export async function getControlledShadowExperimentDesignRegistrations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-design-registrations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowExperimentDesignRegistrationRegistry>(response);
}

export async function registerControlledShadowExperimentDesign(
  attemptId: string,
  request: RegisterControlledShadowExperimentDesignRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-design-registrations/${encodeURIComponent(attemptId)}/register`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowExperimentDesignRegistrationRegistry>(response);
}

export async function getControlledShadowExperimentDesignRegistrationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-design-registration-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowExperimentDesignRegistrationReviewRegistry>(response);
}

export async function reviewControlledShadowExperimentDesignRegistration(
  attemptId: string,
  request: ReviewControlledShadowExperimentDesignRegistrationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-design-registration-reviews/${encodeURIComponent(attemptId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowExperimentDesignRegistrationReviewRegistry>(response);
}

export async function getControlledShadowExperimentImplementations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowExperimentImplementationRegistry>(response);
}

export async function registerControlledShadowExperimentImplementation(
  attemptId: string,
  request: RegisterControlledShadowExperimentImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-implementations/${encodeURIComponent(attemptId)}/register`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowExperimentImplementationRegistry>(response);
}

export async function getControlledShadowExperimentImplementationReviews(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-implementation-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowExperimentImplementationReviewRegistry>(response);
}

export async function reviewControlledShadowExperimentImplementation(
  implementationId: string,
  request: ReviewControlledShadowExperimentImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-implementation-reviews/${encodeURIComponent(implementationId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowExperimentImplementationReviewRegistry>(response);
}

export async function getControlledShadowExperimentIsolatedRunners(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-isolated-runners",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowExperimentIsolatedRunnerRegistry>(response);
}

export async function registerControlledShadowExperimentIsolatedRunner(
  implementationId: string,
  request: RegisterControlledShadowExperimentIsolatedRunnerRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-isolated-runners/${encodeURIComponent(implementationId)}/register`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowExperimentIsolatedRunnerRegistry>(response);
}

export async function getControlledShadowExperimentFirstExecutionAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowExperimentFirstExecutionAuthorizationRegistry>(response);
}

export async function reviewControlledShadowExperimentFirstExecutionAuthorization(
  isolatedRunnerId: string,
  request: ReviewControlledShadowExperimentFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-first-execution-authorizations/${encodeURIComponent(isolatedRunnerId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowExperimentFirstExecutionAuthorizationRegistry>(response);
}

export async function getControlledShadowExperimentExecutionAttempts(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowExperimentExecutionAttemptRegistry>(response);
}

export async function invokeControlledShadowExperimentOnce(
  isolatedRunnerId: string,
  request: InvokeControlledShadowExperimentOnceRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-execution-attempts/${encodeURIComponent(isolatedRunnerId)}/invoke-once`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowExperimentExecutionAttemptRegistry>(response);
}

export async function getControlledShadowExperimentOutputValidations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowExperimentOutputValidationRegistry>(response);
}

export async function validateControlledShadowExperimentOutput(
  attemptId: string,
  request: ValidateControlledShadowExperimentOutputRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-output-validations/${encodeURIComponent(attemptId)}/validate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowExperimentOutputValidationRegistry>(response);
}

export async function getControlledShadowForwardObservationProtocolRegistrations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-protocol-registrations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowForwardObservationProtocolRegistrationRegistry>(response);
}

export async function registerControlledShadowForwardObservationProtocol(
  validationId: string,
  request: RegisterControlledShadowForwardObservationProtocolRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-protocol-registrations/${encodeURIComponent(validationId)}/register`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowForwardObservationProtocolRegistrationRegistry>(response);
}

export async function getControlledShadowForwardObservationProtocolRegistrationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-protocol-registration-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowForwardObservationProtocolRegistrationReviewRegistry>(response);
}

export async function reviewControlledShadowForwardObservationProtocolRegistration(
  protocolRegistrationId: string,
  request: ReviewControlledShadowForwardObservationProtocolRegistrationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-protocol-registration-reviews/${encodeURIComponent(protocolRegistrationId)}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowForwardObservationProtocolRegistrationReviewRegistry>(response);
}

export async function getControlledShadowForwardObservationImplementations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowForwardObservationImplementationRegistry>(response);
}

export async function registerControlledShadowForwardObservationImplementation(
  protocolReviewId: string,
  request: RegisterControlledShadowForwardObservationImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-implementations/${encodeURIComponent(protocolReviewId)}/register`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowForwardObservationImplementationRegistry>(response);
}

export async function getControlledShadowForwardObservationImplementationReviews(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-implementation-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowForwardObservationImplementationReviewRegistry>(response);
}

export async function reviewControlledShadowForwardObservationImplementation(
  implementationId: string,
  request: ReviewControlledShadowForwardObservationImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-implementation-reviews/${encodeURIComponent(implementationId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowForwardObservationImplementationReviewRegistry>(response);
}

export async function getControlledShadowForwardObservationIsolatedRunners(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-isolated-runners",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowForwardObservationIsolatedRunnerRegistry>(response);
}

export async function registerControlledShadowForwardObservationIsolatedRunner(
  implementationId: string,
  request: RegisterControlledShadowForwardObservationIsolatedRunnerRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-isolated-runners/${encodeURIComponent(implementationId)}/register`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowForwardObservationIsolatedRunnerRegistry>(response);
}

export async function getControlledShadowForwardObservationFirstExecutionAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowForwardObservationFirstExecutionAuthorizationRegistry>(response);
}

export async function reviewControlledShadowForwardObservationFirstExecutionAuthorization(
  isolatedRunnerId: string,
  request: ReviewControlledShadowForwardObservationFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-execution-authorizations/${encodeURIComponent(isolatedRunnerId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowForwardObservationFirstExecutionAuthorizationRegistry>(response);
}

export async function getControlledShadowForwardObservationExecutionAttempts(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowForwardObservationExecutionAttemptRegistry>(response);
}

export async function invokeControlledShadowForwardObservationOnce(
  isolatedRunnerId: string,
  request: InvokeControlledShadowForwardObservationOnceRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-execution-attempts/${encodeURIComponent(isolatedRunnerId)}/invoke-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowForwardObservationExecutionAttemptRegistry>(response);
}

export async function getControlledShadowForwardObservationOutputValidations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowForwardObservationOutputValidationRegistry>(response);
}

export async function validateControlledShadowForwardObservationOutput(
  attemptId: string,
  request: ValidateControlledShadowForwardObservationOutputRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-output-validations/${encodeURIComponent(attemptId)}/validate`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowForwardObservationOutputValidationRegistry>(response);
}

export async function getControlledShadowFirstNaturalForwardCycleAuthorizations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowFirstNaturalForwardCycleAuthorizationRegistry>(response);
}

export async function reviewControlledShadowFirstNaturalForwardCycleAuthorization(
  validationId: string,
  request: ReviewControlledShadowFirstNaturalForwardCycleAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-authorizations/${encodeURIComponent(validationId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowFirstNaturalForwardCycleAuthorizationRegistry>(response);
}

export async function getControlledShadowFirstNaturalForwardCycleClaims(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-claims",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowFirstNaturalForwardCycleClaimRegistry>(response);
}

export async function claimControlledShadowFirstNaturalForwardCycleOnce(
  authorizationReviewId: string,
  request: ClaimControlledShadowFirstNaturalForwardCycleRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-claims/${encodeURIComponent(authorizationReviewId)}/claim-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowFirstNaturalForwardCycleClaimRegistry>(response);
}

export async function getControlledShadowMarketDataAdapterAuthorizations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-adapter-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowMarketDataAdapterAuthorizationRegistry>(response);
}

export async function reviewControlledShadowMarketDataAdapterAuthorization(
  cycleClaimId: string,
  request: ReviewControlledShadowMarketDataAdapterAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-adapter-authorizations/${encodeURIComponent(cycleClaimId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowMarketDataAdapterAuthorizationRegistry>(response);
}

export async function getControlledShadowMarketDataReceiptAttempts(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-receipt-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowMarketDataReceiptAttemptRegistry>(response);
}

export async function claimAndReadControlledShadowMarketDataReceiptOnce(
  adapterAuthorizationId: string,
  request: ClaimAndReadControlledShadowMarketDataReceiptRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-receipt-attempts/${encodeURIComponent(adapterAuthorizationId)}/claim-and-read-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowMarketDataReceiptAttemptRegistry>(response);
}

export async function getControlledShadowMarketDataReceiptValidations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-receipt-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowMarketDataReceiptValidationRegistry>(response);
}

export async function validateControlledShadowMarketDataReceiptOnce(
  attemptId: string,
  request: ValidateControlledShadowMarketDataReceiptRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-receipt-validations/${encodeURIComponent(attemptId)}/validate-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowMarketDataReceiptValidationRegistry>(response);
}

export async function getControlledShadowMarketDataParserSpecifications(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-specifications",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowMarketDataParserSpecificationRegistry>(response);
}

export async function registerControlledShadowMarketDataParserSpecificationOnce(
  validationId: string,
  request: RegisterControlledShadowMarketDataParserSpecificationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-specifications/${encodeURIComponent(validationId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowMarketDataParserSpecificationRegistry>(response);
}

export async function getControlledShadowMarketDataParserSpecificationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-specification-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowMarketDataParserSpecificationReviewRegistry>(response);
}

export async function reviewControlledShadowMarketDataParserSpecificationOnce(
  registrationId: string,
  request: ReviewControlledShadowMarketDataParserSpecificationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-specification-reviews/${encodeURIComponent(registrationId)}/review-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowMarketDataParserSpecificationReviewRegistry>(response);
}

export async function getControlledShadowMarketDataParserImplementations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowMarketDataParserImplementationRegistry>(response);
}

export async function registerControlledShadowMarketDataParserImplementationOnce(
  specificationReviewId: string,
  request: RegisterControlledShadowMarketDataParserImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-implementations/${encodeURIComponent(specificationReviewId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowMarketDataParserImplementationRegistry>(response);
}

export async function getControlledShadowMarketDataParserImplementationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-implementation-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowMarketDataParserImplementationReviewRegistry>(response);
}

export async function reviewControlledShadowMarketDataParserImplementationOnce(
  implementationId: string,
  request: ReviewControlledShadowMarketDataParserImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-implementation-reviews/${encodeURIComponent(implementationId)}/review-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowMarketDataParserImplementationReviewRegistry>(response);
}

export async function getControlledShadowMarketDataParserIsolatedRunners(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-isolated-runners",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowMarketDataParserIsolatedRunnerRegistry>(response);
}

export async function registerControlledShadowMarketDataParserIsolatedRunnerOnce(
  implementationId: string,
  request: RegisterControlledShadowMarketDataParserIsolatedRunnerRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-isolated-runners/${encodeURIComponent(implementationId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowMarketDataParserIsolatedRunnerRegistry>(response);
}

export async function getControlledShadowMarketDataParserFirstExecutionAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowMarketDataParserFirstExecutionAuthorizationRegistry>(response);
}

export async function reviewControlledShadowMarketDataParserFirstExecutionAuthorizationOnce(
  isolatedRunnerId: string,
  request: ReviewControlledShadowMarketDataParserFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-first-execution-authorizations/${encodeURIComponent(isolatedRunnerId)}/review-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowMarketDataParserFirstExecutionAuthorizationRegistry>(response);
}

export async function getControlledShadowMarketDataParserExecutionAttemptClaims(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-execution-attempt-claims",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowMarketDataParserExecutionAttemptClaimRegistry>(response);
}

export async function claimControlledShadowMarketDataParserExecutionAttemptOnce(
  authorizationReviewId: string,
  request: ClaimControlledShadowMarketDataParserExecutionAttemptRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-execution-attempt-claims/${encodeURIComponent(authorizationReviewId)}/claim-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowMarketDataParserExecutionAttemptClaimRegistry>(response);
}

export async function getControlledShadowMarketDataParserExecutionAttempts(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowMarketDataParserExecutionAttemptRegistry>(response);
}

export async function executeControlledShadowMarketDataParserAttemptOnce(
  attemptId: string,
  request: ExecuteControlledShadowMarketDataParserAttemptRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-execution-attempts/${encodeURIComponent(attemptId)}/execute-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowMarketDataParserExecutionAttemptRegistry>(response);
}

export async function getControlledShadowMarketDataParserOutputValidations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowMarketDataParserOutputValidationRegistry>(response);
}

export async function validateControlledShadowMarketDataParserOutputOnce(
  attemptId: string,
  request: ValidateControlledShadowMarketDataParserOutputRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-market-data-parser-output-validations/${encodeURIComponent(attemptId)}/validate-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowMarketDataParserOutputValidationRegistry>(response);
}

export async function getControlledShadowObservationInputAdmissionReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-input-admission-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationInputAdmissionRegistry>(response);
}

export async function reviewControlledShadowObservationInputAdmission(
  attemptId: string,
  request: ReviewControlledShadowObservationInputAdmissionRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-input-admission-reviews/${encodeURIComponent(attemptId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationInputAdmissionRegistry>(response);
}

export async function getControlledShadowObservationMaterializationSpecifications(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-specifications",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationMaterializationSpecificationRegistry>(response);
}

export async function registerControlledShadowObservationMaterializationSpecification(
  reviewId: string,
  request: RegisterControlledShadowObservationMaterializationSpecificationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-specifications/${encodeURIComponent(reviewId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationMaterializationSpecificationRegistry>(response);
}

export async function getControlledShadowObservationMaterializationSpecificationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-specification-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationMaterializationSpecificationReviewRegistry>(response);
}

export async function reviewControlledShadowObservationMaterializationSpecification(
  registrationId: string,
  request: ReviewControlledShadowObservationMaterializationSpecificationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-specification-reviews/${encodeURIComponent(registrationId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationMaterializationSpecificationReviewRegistry>(response);
}

export async function getControlledShadowObservationMaterializationImplementations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationMaterializationImplementationRegistry>(response);
}

export async function registerControlledShadowObservationMaterializationImplementationOnce(
  specificationReviewId: string,
  request: RegisterControlledShadowObservationMaterializationImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-implementations/${encodeURIComponent(specificationReviewId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationMaterializationImplementationRegistry>(response);
}

export async function getControlledShadowObservationMaterializationImplementationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-implementation-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationMaterializationImplementationReviewRegistry>(response);
}

export async function reviewControlledShadowObservationMaterializationImplementationOnce(
  implementationId: string,
  request: ReviewControlledShadowObservationMaterializationImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-implementation-reviews/${encodeURIComponent(implementationId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationMaterializationImplementationReviewRegistry>(response);
}

export async function getControlledShadowObservationMaterializationIsolatedRunners(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-isolated-runners",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationMaterializationIsolatedRunnerRegistry>(response);
}

export async function registerControlledShadowObservationMaterializationIsolatedRunnerOnce(
  implementationId: string,
  request: RegisterControlledShadowObservationMaterializationIsolatedRunnerRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-isolated-runners/${encodeURIComponent(implementationId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationMaterializationIsolatedRunnerRegistry>(response);
}

export async function getControlledShadowObservationMaterializationFirstExecutionAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationMaterializationFirstExecutionAuthorizationRegistry>(response);
}

export async function reviewControlledShadowObservationMaterializationFirstExecutionAuthorizationOnce(
  isolatedRunnerId: string,
  request: ReviewControlledShadowObservationMaterializationFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-first-execution-authorizations/${encodeURIComponent(isolatedRunnerId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationMaterializationFirstExecutionAuthorizationRegistry>(response);
}

export async function getControlledShadowObservationMaterializationExecutionAttemptClaims(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-execution-attempt-claims",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationMaterializationExecutionAttemptClaimRegistry>(response);
}

export async function claimControlledShadowObservationMaterializationExecutionAttemptOnce(
  authorizationReviewId: string,
  request: ClaimControlledShadowObservationMaterializationExecutionAttemptRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-execution-attempt-claims/${encodeURIComponent(authorizationReviewId)}/claim-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationMaterializationExecutionAttemptClaimRegistry>(response);
}

export async function getControlledShadowObservationMaterializationExecutionAttempts(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationMaterializationExecutionAttemptRegistry>(response);
}

export async function executeControlledShadowObservationMaterializationAttemptOnce(
  attemptId: string,
  request: ExecuteControlledShadowObservationMaterializationAttemptRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-execution-attempts/${encodeURIComponent(attemptId)}/execute-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationMaterializationExecutionAttemptRegistry>(response);
}

export async function getControlledShadowObservationMaterializationOutputValidations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationMaterializationOutputValidationRegistry>(response);
}

export async function validateControlledShadowObservationMaterializationOutputOnce(
  attemptId: string,
  request: ValidateControlledShadowObservationMaterializationOutputRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-materialization-output-validations/${encodeURIComponent(attemptId)}/validate-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationMaterializationOutputValidationRegistry>(response);
}

export async function getControlledShadowObservationEvidenceAdmissionReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-evidence-admission-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationEvidenceAdmissionRegistry>(response);
}

export async function reviewControlledShadowObservationEvidenceAdmission(
  attemptId: string,
  request: ReviewControlledShadowObservationEvidenceAdmissionRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-evidence-admission-reviews/${encodeURIComponent(attemptId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationEvidenceAdmissionRegistry>(response);
}

export async function getControlledShadowObservationLedgerTransitionSpecifications(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-specifications",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionSpecificationRegistry>(response);
}

export async function registerControlledShadowObservationLedgerTransitionSpecification(
  reviewId: string,
  request: RegisterControlledShadowObservationLedgerTransitionSpecificationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-specifications/${encodeURIComponent(reviewId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionSpecificationRegistry>(response);
}

export async function getControlledShadowObservationLedgerTransitionSpecificationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-specification-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionSpecificationReviewRegistry>(response);
}

export async function reviewControlledShadowObservationLedgerTransitionSpecification(
  registrationId: string,
  request: ReviewControlledShadowObservationLedgerTransitionSpecificationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-specification-reviews/${encodeURIComponent(registrationId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionSpecificationReviewRegistry>(response);
}

export async function getControlledShadowObservationLedgerTransitionImplementations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionImplementationRegistry>(response);
}

export async function registerControlledShadowObservationLedgerTransitionImplementationOnce(
  specificationReviewId: string,
  request: RegisterControlledShadowObservationLedgerTransitionImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-implementations/${encodeURIComponent(specificationReviewId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionImplementationRegistry>(response);
}

export async function getControlledShadowObservationLedgerTransitionImplementationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-implementation-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionImplementationReviewRegistry>(
    response,
  );
}

export async function reviewControlledShadowObservationLedgerTransitionImplementationOnce(
  implementationId: string,
  request: ReviewControlledShadowObservationLedgerTransitionImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-implementation-reviews/${encodeURIComponent(implementationId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionImplementationReviewRegistry>(
    response,
  );
}

export async function getControlledShadowObservationLedgerTransitionIsolatedRunners(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-isolated-runners",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionIsolatedRunnerRegistry>(response);
}

export async function registerControlledShadowObservationLedgerTransitionIsolatedRunnerOnce(
  implementationId: string,
  request: RegisterControlledShadowObservationLedgerTransitionIsolatedRunnerRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-isolated-runners/${encodeURIComponent(implementationId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionIsolatedRunnerRegistry>(response);
}

export async function getControlledShadowObservationLedgerTransitionFirstExecutionAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationRegistry>(response);
}

export async function reviewControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationOnce(
  isolatedRunnerId: string,
  request: ReviewControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-first-execution-authorizations/${encodeURIComponent(isolatedRunnerId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationRegistry>(response);
}

export async function getControlledShadowObservationLedgerTransitionExecutionAttemptClaims(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-execution-attempt-claims",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionExecutionAttemptClaimRegistry>(response);
}

export async function claimControlledShadowObservationLedgerTransitionExecutionAttemptOnce(
  authorizationReviewId: string,
  request: ClaimControlledShadowObservationLedgerTransitionExecutionAttemptRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-execution-attempt-claims/${encodeURIComponent(authorizationReviewId)}/claim-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionExecutionAttemptClaimRegistry>(response);
}

export async function getControlledShadowObservationLedgerTransitionExecutionAttempts(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionExecutionAttemptRegistry>(response);
}

export async function executeControlledShadowObservationLedgerTransitionAttemptOnce(
  attemptId: string,
  request: ExecuteControlledShadowObservationLedgerTransitionAttemptRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-execution-attempts/${encodeURIComponent(attemptId)}/execute-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionExecutionAttemptRegistry>(response);
}

export async function getControlledShadowObservationLedgerTransitionOutputValidations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-output-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionOutputValidationRegistry>(response);
}

export async function validateControlledShadowObservationLedgerTransitionOutputOnce(
  attemptId: string,
  request: ValidateControlledShadowObservationLedgerTransitionOutputRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-output-validations/${encodeURIComponent(attemptId)}/validate-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionOutputValidationRegistry>(response);
}

export async function getControlledShadowObservationLedgerTransitionCandidateAdmissionReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-candidate-admission-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionCandidateAdmissionRegistry>(response);
}

export async function reviewControlledShadowObservationLedgerTransitionCandidateAdmission(
  attemptId: string,
  request: ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-controlled-shadow-experiment-forward-observation-first-natural-forward-cycle-observation-ledger-transition-candidate-admission-reviews/${encodeURIComponent(attemptId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<ControlledShadowObservationLedgerTransitionCandidateAdmissionRegistry>(response);
}

export async function getOpeningPortfolioSnapshotGovernanceSpecifications(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-snapshot-governance-specifications",
    { signal, cache: "no-store" },
  );
  return parseJson<OpeningPortfolioSnapshotGovernanceSpecificationRegistry>(response);
}

export async function registerOpeningPortfolioSnapshotGovernanceSpecification(
  stage124ReviewId: string,
  request: RegisterOpeningPortfolioSnapshotGovernanceSpecificationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-snapshot-governance-specifications/${encodeURIComponent(stage124ReviewId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<OpeningPortfolioSnapshotGovernanceSpecificationRegistry>(response);
}

export async function getOpeningPortfolioSnapshotGovernanceSpecificationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-snapshot-governance-specification-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<OpeningPortfolioSnapshotGovernanceSpecificationReviewRegistry>(response);
}

export async function reviewOpeningPortfolioSnapshotGovernanceSpecification(
  registrationId: string,
  request: ReviewOpeningPortfolioSnapshotGovernanceSpecificationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-snapshot-governance-specification-reviews/${encodeURIComponent(registrationId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<OpeningPortfolioSnapshotGovernanceSpecificationReviewRegistry>(response);
}

export async function getOpeningPortfolioSourceArtifactReceiptImplementations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptImplementationRegistry>(response);
}

export async function registerOpeningPortfolioSourceArtifactReceiptImplementation(
  stage126ReviewId: string,
  request: RegisterOpeningPortfolioSourceArtifactReceiptImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-implementations/${encodeURIComponent(stage126ReviewId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptImplementationRegistry>(response);
}

export async function getOpeningPortfolioSourceArtifactReceiptImplementationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-implementation-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptImplementationReviewRegistry>(response);
}

export async function reviewOpeningPortfolioSourceArtifactReceiptImplementation(
  implementationId: string,
  request: ReviewOpeningPortfolioSourceArtifactReceiptImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-implementation-reviews/${encodeURIComponent(implementationId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptImplementationReviewRegistry>(response);
}

export async function getOpeningPortfolioSourceArtifactReceiptIsolatedReceivers(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-isolated-receivers",
    { signal, cache: "no-store" },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRegistry>(response);
}

export async function registerOpeningPortfolioSourceArtifactReceiptIsolatedReceiver(
  implementationId: string,
  request: RegisterOpeningPortfolioSourceArtifactReceiptIsolatedReceiverRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-isolated-receivers/${encodeURIComponent(implementationId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRegistry>(response);
}

export async function getOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizations(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-first-execution-authorizations",
    { signal, cache: "no-store" },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRegistry>(response);
}

export async function reviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorization(
  isolatedReceiverId: string,
  request: ReviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-first-execution-authorizations/${encodeURIComponent(isolatedReceiverId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRegistry>(response);
}

export async function getOpeningPortfolioSourceArtifactReceiptExecutionAttemptClaims(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-execution-attempt-claims",
    { signal, cache: "no-store" },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimRegistry>(response);
}

export async function claimOpeningPortfolioSourceArtifactReceiptExecutionAttemptOnce(
  authorizationReviewId: string,
  request: ClaimOpeningPortfolioSourceArtifactReceiptExecutionAttemptRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-execution-attempt-claims/${encodeURIComponent(authorizationReviewId)}/claim-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimRegistry>(response);
}

export async function getOpeningPortfolioSourceArtifactReceiptExecutionAttempts(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-execution-attempts",
    { signal, cache: "no-store" },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptExecutionAttemptRegistry>(response);
}

export async function receiveOpeningPortfolioSourceArtifactReceiptAttemptOnce(
  attemptId: string,
  request: ReceiveOpeningPortfolioSourceArtifactReceiptAttemptRequest,
  artifacts: File[],
) {
  const body = new FormData();
  body.append("request", JSON.stringify(request));
  for (const artifact of artifacts) body.append("artifact", artifact, artifact.name);
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-execution-attempts/${encodeURIComponent(attemptId)}/receive-once`,
    { method: "POST", headers: PUBLIC_ADMIN_ACTION_HEADERS, body },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptExecutionAttemptRegistry>(response);
}

export async function getOpeningPortfolioSourceArtifactReceiptValidations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-validations",
    { signal, cache: "no-store" },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptValidationRegistry>(response);
}

export async function validateOpeningPortfolioSourceArtifactReceiptOnce(
  attemptId: string,
  request: ValidateOpeningPortfolioSourceArtifactReceiptRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-source-artifact-receipt-validations/${encodeURIComponent(attemptId)}/validate-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<OpeningPortfolioSourceArtifactReceiptValidationRegistry>(response);
}

export async function getOpeningPortfolioSnapshotMaterializationImplementations(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-snapshot-materialization-implementations",
    { signal, cache: "no-store" },
  );
  return parseJson<OpeningPortfolioSnapshotMaterializationImplementationRegistry>(response);
}

export async function registerOpeningPortfolioSnapshotMaterializationImplementation(
  validationId: string,
  request: RegisterOpeningPortfolioSnapshotMaterializationImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-snapshot-materialization-implementations/${encodeURIComponent(validationId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<OpeningPortfolioSnapshotMaterializationImplementationRegistry>(response);
}

export async function getOpeningPortfolioSnapshotMaterializationImplementationReviews(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-snapshot-materialization-implementation-reviews",
    { signal, cache: "no-store" },
  );
  return parseJson<OpeningPortfolioSnapshotMaterializationImplementationReviewRegistry>(response);
}

export async function reviewOpeningPortfolioSnapshotMaterializationImplementation(
  implementationId: string,
  request: ReviewOpeningPortfolioSnapshotMaterializationImplementationRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-snapshot-materialization-implementation-reviews/${encodeURIComponent(implementationId)}/review`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<OpeningPortfolioSnapshotMaterializationImplementationReviewRegistry>(response);
}

export async function getOpeningPortfolioSnapshotMaterializationIsolatedMaterializers(
  signal?: AbortSignal,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-snapshot-materialization-isolated-materializers",
    { signal, cache: "no-store" },
  );
  return parseJson<OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRegistry>(response);
}

export async function registerOpeningPortfolioSnapshotMaterializationIsolatedMaterializer(
  implementationId: string,
  request: RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest,
) {
  const response = await apiFetch(
    `/api/public/admin/investment-decisions/controlled-shadow-opening-portfolio-snapshot-materialization-isolated-materializers/${encodeURIComponent(implementationId)}/register-once`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json", ...PUBLIC_ADMIN_ACTION_HEADERS },
      body: JSON.stringify(request),
    },
  );
  return parseJson<OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRegistry>(response);
}

export async function getInvestmentCausalDatasetGovernance(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/causal-dataset-governance",
    { signal, cache: "no-store" },
  );
  return parseJson<InvestmentCausalDatasetGovernance>(response);
}

export async function reviewInvestmentCausalDatasetGovernance(
  request: InvestmentCausalDatasetGovernanceRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/causal-dataset-governance",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<InvestmentCausalDatasetGovernance>(response);
}

export async function getInvestmentCausalTrainingExperiments(signal?: AbortSignal) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/causal-training-experiments",
    { signal, cache: "no-store" },
  );
  return parseJson<InvestmentCausalTrainingExperimentRegistry>(response);
}

export async function registerInvestmentCausalTrainingExperiment(
  request: InvestmentCausalTrainingExperimentRequest,
) {
  const response = await apiFetch(
    "/api/public/admin/investment-decisions/causal-training-experiments",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...PUBLIC_ADMIN_ACTION_HEADERS,
      },
      body: JSON.stringify(request),
    },
  );
  return parseJson<InvestmentCausalTrainingExperimentRegistry>(response);
}

export async function getPublicChatBootstrap(signal?: AbortSignal) {
  try {
    const response = await apiFetch("/api/public/bootstrap", {
      signal,
      cache: "no-store",
    });
    const payload = await parseJson<PublicChatBootstrapResponse>(response);
    setCachedPublicUser(payload.user);
    return payload;
  } catch (error) {
    if (isUnauthorizedApiError(error)) setCachedPublicUser(null);
    throw error;
  }
}

export async function getPublicHistory(before?: number, signal?: AbortSignal) {
  const query = new URLSearchParams({ limit: "20" });
  if (before !== undefined) query.set("before", String(before));
  const response = await apiFetch(`/api/public/history?${query.toString()}`, {
    signal,
    cache: "no-store",
  });
  return parseJson<PublicHistoryPageResponse>(response);
}

export async function getPublicPushes(
  before?: string,
  limit = 30,
): Promise<PublicPushListResponse> {
  const query = new URLSearchParams({ limit: String(limit) });
  if (before) query.set("before", before);
  const response = await apiFetch(`/api/public/pushes?${query.toString()}`);
  return parseJson<PublicPushListResponse>(response);
}

export async function openPublicPush(
  pushId: string,
): Promise<PublicPushOpenResponse> {
  const response = await apiFetch(
    `/api/public/pushes/${encodeURIComponent(pushId)}/open`,
    { method: "POST" },
  );
  return parseJson<PublicPushOpenResponse>(response);
}

// ── Public investment context (mainline/profile reads + refresh) ──────────

export type ProfileSummary = {
  dir: string;
  ticker?: string;
  tickers?: string[];
  title?: string;
  preview?: string;
  bytes?: number;
};

export type DigestContext = {
  actor: { channel: string; user_id: string };
  mainline_style: string | null;
  mainline_by_ticker: Record<string, string>;
  global_digest_enabled?: boolean;
  global_digest_floor_macro_picks?: number;
  last_mainline_distilled_at: string | null;
  mainline_distill_skipped: string[];
  holdings: string[];
  profile_list: ProfileSummary[];
};

export async function getDigestContext(): Promise<DigestContext> {
  const response = await apiFetch("/api/public/digest-context");
  return parseJson<DigestContext>(response);
}

export type PublicQuote = {
  symbol: string;
  name?: string;
  price: number;
  change?: number;
  change_percent?: number;
};

export type PublicQuotesResponse = {
  available: boolean;
  quotes: PublicQuote[];
};

export async function getPublicQuotes(): Promise<PublicQuotesResponse> {
  const response = await apiFetch("/api/public/quotes");
  return parseJson<PublicQuotesResponse>(response);
}

// ── 我的：自选与持仓 ───────────────────────────────────────────────────────

export type PublicHolding = {
  symbol: string;
  name?: string | null;
  /** 仓位占比(%)，自选条目为 null。 */
  weight?: number | null;
  avg_cost?: number | null;
  notes?: string | null;
  tracking_only: boolean;
};

export type PublicPortfolioResponse = {
  holdings: PublicHolding[];
  limit: number;
};

export type PublicHoldingInput = {
  symbol: string;
  name?: string;
  weight?: number;
  avg_cost?: number;
};

export async function getPublicPortfolio(): Promise<PublicPortfolioResponse> {
  const response = await apiFetch("/api/public/portfolio");
  return parseJson<PublicPortfolioResponse>(response);
}

export async function createPublicHolding(
  input: PublicHoldingInput,
): Promise<PublicPortfolioResponse> {
  const response = await apiFetch("/api/public/portfolio/holdings", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  return parseJson<PublicPortfolioResponse>(response);
}

export async function updatePublicHolding(
  symbol: string,
  input: PublicHoldingInput,
): Promise<PublicPortfolioResponse> {
  const response = await apiFetch(
    `/api/public/portfolio/holdings/${encodeURIComponent(symbol)}`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(input),
    },
  );
  return parseJson<PublicPortfolioResponse>(response);
}

export async function deletePublicHolding(
  symbol: string,
): Promise<PublicPortfolioResponse> {
  const response = await apiFetch(
    `/api/public/portfolio/holdings/${encodeURIComponent(symbol)}`,
    { method: "DELETE" },
  );
  return parseJson<PublicPortfolioResponse>(response);
}

// ── 我的：设置 ────────────────────────────────────────────────────────────

export type PublicSettings = {
  style: string | null;
  distilled_style: string | null;
  user_edited: boolean;
  last_distilled_at?: string | null;
};

export async function getPublicSettings(): Promise<PublicSettings> {
  const response = await apiFetch("/api/public/settings");
  return parseJson<PublicSettings>(response);
}

export async function putPublicInvestorStyle(
  style: string,
): Promise<PublicSettings> {
  const response = await apiFetch("/api/public/settings/investor-style", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ style }),
  });
  return parseJson<PublicSettings>(response);
}

// ── Admin: mainline context for any actor ─────────────────────────────────

export type AdminMainlineContext = DigestContext & {
  actor: { channel: string; user_id: string; channel_scope?: string | null };
};

export async function getAdminMainlineContext(
  actor: ActorRef,
): Promise<AdminMainlineContext> {
  const q = actorQuery(actor);
  const response = await apiFetch(`/api/event-engine/mainline-context?${q}`);
  return parseJson<AdminMainlineContext>(response);
}

export async function getAdminCompanyProfile(
  actor: ActorRef,
  ticker: string,
): Promise<{ ticker: string; dir: string; markdown: string }> {
  const queryParams = new URLSearchParams({
    channel: actor.channel,
    user_id: actor.user_id,
    ticker,
  });
  if (actor.channel_scope)
    queryParams.set("channel_scope", actor.channel_scope);
  const response = await apiFetch(
    `/api/event-engine/company-profile?${queryParams.toString()}`,
  );
  return parseJson(response);
}

export async function adminTriggerMainlineDistill(actor: ActorRef): Promise<{
  ok: boolean;
  mainline_count: number;
  mainline_style_set: boolean;
  skipped_tickers: string[];
  last_distilled_at: string | null;
}> {
  const q = actorQuery(actor);
  const response = await apiFetch(`/api/event-engine/mainline-distill?${q}`, {
    method: "POST",
  });
  return parseJson(response);
}

export async function refreshDigestContext(): Promise<{
  ok: boolean;
  mainline_count: number;
  mainline_style_set: boolean;
  skipped_tickers: string[];
  last_distilled_at: string | null;
}> {
  const response = await apiFetch("/api/public/digest-context/refresh", {
    method: "POST",
  });
  return parseJson(response);
}

export async function getCompanyProfileMarkdown(ticker: string): Promise<{
  ticker: string;
  dir: string;
  markdown: string;
}> {
  const response = await apiFetch(
    `/api/public/company-profile?ticker=${encodeURIComponent(ticker)}`,
  );
  return parseJson(response);
}

export async function getPublicFinanceCalendar(
  month?: string,
): Promise<FinanceCalendarPayload> {
  const query = month ? `?month=${encodeURIComponent(month)}` : "";
  const response = await apiFetch(`/api/public/finance-calendar${query}`);
  return parseJson<FinanceCalendarPayload>(response);
}

export async function sendPublicFinanceCalendar(input: {
  path: string;
  mobile_path: string;
  month: string;
}): Promise<{ ok: boolean; message: string }> {
  const response = await apiFetch("/api/public/finance-calendar/send", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(input),
  });
  return parseJson<{ ok: boolean; message: string }>(response);
}

export type PublicUploadedAttachment = {
  path: string;
  name: string;
  kind: string;
  size: number;
};

export type PublicChatAttachmentInput = {
  path: string;
  name?: string;
};

export type PublicEarningsWorkflowInput = {
  kind: "preview" | "analysis";
  company: string;
};

export async function sendPublicChat(
  message: string,
  attachments: PublicChatAttachmentInput[] = [],
  signal?: AbortSignal,
  earningsWorkflow?: PublicEarningsWorkflowInput,
) {
  const response = await apiFetch("/api/public/chat", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      message,
      attachments,
      earnings_workflow: earningsWorkflow,
      // Tell the server what the user is actually reading, so the answer is
      // written in that language instead of guessed from the conversation.
      language: useLocale(),
    }),
    signal,
  });

  if (!response.ok) {
    throw await apiErrorFromResponse(response);
  }

  if (!response.body) {
    throw new Error("missing response body");
  }

  return response.body;
}

export async function uploadPublicAttachments(files: File[]) {
  if (!files.length) return [] as PublicUploadedAttachment[];
  const form = new FormData();
  for (const file of files) {
    form.append("files", file, file.name);
  }
  const response = await apiFetch("/api/public/upload", {
    method: "POST",
    body: form,
  });
  const payload = await parseJson<{ attachments: PublicUploadedAttachment[] }>(
    response,
  );
  return payload.attachments ?? [];
}

export async function getPublicGeneratedFileBlob(path: string) {
  const response = await apiFetch(
    `/api/public/file?path=${encodeURIComponent(path)}`,
  );
  if (!response.ok) throw await apiErrorFromResponse(response);
  return response.blob();
}

export async function connectPublicEvents() {
  return createEventSource("/api/public/events");
}

type PublicCommunityEdgeSession = {
  enabled: boolean;
  mode: "off" | "shadow" | "prefer" | string;
  base_path?: string | null;
  expires_at?: number | null;
};

type PublicCommunityState = {
  unread: boolean;
  latest_content_id?: number | null;
};

type ActiveCommunityEdge = {
  basePath: "/_community/v1";
  expiresAt: number;
};

const PUBLIC_COMMUNITY_EDGE_BASE_PATH = "/_community/v1" as const;
const PUBLIC_COMMUNITY_EDGE_RETRY_DELAY_MS = 30_000;
let publicCommunityEdgeDiscoveryEnabled =
  import.meta.env.VITE_HONE_APP_COMMUNITY_EDGE_DISCOVERY === "1";
let activePublicCommunityEdge: ActiveCommunityEdge | null = null;
let publicCommunityEdgeRetryAt = 0;

function resetPublicCommunityEdgeState() {
  activePublicCommunityEdge = null;
  publicCommunityEdgeRetryAt = 0;
}

/** Test-only override; production behavior remains a compile-time flag. */
export function setPublicCommunityEdgeDiscoveryForTests(enabled: boolean) {
  publicCommunityEdgeDiscoveryEnabled = enabled;
  resetPublicCommunityEdgeState();
}

export function resetPublicCommunityEdgeDiscoveryForTests() {
  publicCommunityEdgeDiscoveryEnabled =
    import.meta.env.VITE_HONE_APP_COMMUNITY_EDGE_DISCOVERY === "1";
  resetPublicCommunityEdgeState();
}

function normalizedPublicCommunityEdgeSession(
  payload: PublicCommunityEdgeSession,
): ActiveCommunityEdge | null {
  const now = Math.floor(Date.now() / 1_000);
  const expiresAt = Number(payload.expires_at);
  if (
    !payload.enabled ||
    payload.mode !== "prefer" ||
    payload.base_path !== PUBLIC_COMMUNITY_EDGE_BASE_PATH ||
    !Number.isSafeInteger(expiresAt) ||
    expiresAt <= now + 5
  ) {
    return null;
  }
  return { basePath: PUBLIC_COMMUNITY_EDGE_BASE_PATH, expiresAt };
}

async function discoverPublicCommunityEdge(signal?: AbortSignal) {
  if (!publicCommunityEdgeDiscoveryEnabled) return null;
  const now = Date.now();
  if (
    activePublicCommunityEdge &&
    activePublicCommunityEdge.expiresAt * 1_000 > now + 5_000
  ) {
    return activePublicCommunityEdge;
  }
  if (now < publicCommunityEdgeRetryAt) return null;

  try {
    const response = await apiFetch("/api/public/community/edge-session", {
      method: "POST",
      signal,
    });
    const payload = await parseJson<PublicCommunityEdgeSession>(response);
    activePublicCommunityEdge = normalizedPublicCommunityEdgeSession(payload);
    if (!activePublicCommunityEdge) {
      publicCommunityEdgeRetryAt = now + PUBLIC_COMMUNITY_EDGE_RETRY_DELAY_MS;
    }
    return activePublicCommunityEdge;
  } catch (error) {
    if (signal?.aborted) throw error;
    activePublicCommunityEdge = null;
    publicCommunityEdgeRetryAt = now + PUBLIC_COMMUNITY_EDGE_RETRY_DELAY_MS;
    return null;
  }
}

function publicCommunityEdgeFeedPath(
  edge: ActiveCommunityEdge,
  before?: number,
) {
  if (before && Number.isSafeInteger(before) && before > 0) {
    return `${edge.basePath}/feed/pages/${before}.json`;
  }
  return `${edge.basePath}/feed/latest.json`;
}

async function fetchPublicCommunityEdge(path: string, init: RequestInit = {}) {
  return fetch(buildApiUrl(path), {
    credentials: "include",
    ...init,
  });
}

function verifiedPublicCommunityDeliveryPath(
  resourceId: number,
  version?: string | null,
  deliveryPath?: string | null,
) {
  const normalizedVersion = version?.trim().toLowerCase();
  if (
    !activePublicCommunityEdge ||
    !Number.isSafeInteger(resourceId) ||
    resourceId <= 0 ||
    !normalizedVersion ||
    !/^[a-f0-9]{12}$/.test(normalizedVersion)
  ) {
    return null;
  }
  const expected = `${activePublicCommunityEdge.basePath}/resources/${resourceId}/${normalizedVersion}`;
  return deliveryPath === expected ? expected : null;
}

export async function getPublicCommunity(
  input: {
    before?: number;
    limit?: number;
    signal?: AbortSignal;
  } = {},
) {
  const edge =
    input.limit == null || input.limit === 20
      ? await discoverPublicCommunityEdge(input.signal)
      : null;
  if (edge) {
    try {
      const [feedResponse, stateResponse] = await Promise.all([
        fetchPublicCommunityEdge(
          publicCommunityEdgeFeedPath(edge, input.before),
          {
            signal: input.signal,
          },
        ),
        fetch(buildApiUrl("/api/public/community/state"), {
          credentials: "include",
          signal: input.signal,
        }),
      ]);
      const [page, state] = await Promise.all([
        parseJson<PublicCommunityPage>(feedResponse),
        parseJson<PublicCommunityState>(stateResponse),
      ]);
      return {
        ...page,
        unread: state.unread,
        latest_content_id: state.latest_content_id,
      };
    } catch (error) {
      if (input.signal?.aborted) throw error;
      activePublicCommunityEdge = null;
      publicCommunityEdgeRetryAt =
        Date.now() + PUBLIC_COMMUNITY_EDGE_RETRY_DELAY_MS;
    }
  }

  const query = new URLSearchParams();
  if (input.before) query.set("before", String(input.before));
  if (input.limit) query.set("limit", String(input.limit));
  const suffix = query.size ? `?${query}` : "";
  const response = await apiFetch(`/api/public/community${suffix}`, {
    signal: input.signal,
  });
  return parseJson<PublicCommunityPage>(response);
}

export async function getCommunityForum(signal?: AbortSignal) {
  const response = await apiFetch("/api/public/community/forum", { signal });
  return parseJson<CommunityForumPage>(response);
}

export async function createCommunityForumPost(input: {
  title: string;
  body: string;
  tickers: string;
  topics: string;
  sourceUrl: string;
  attachment?: File | null;
}) {
  const body = new FormData();
  body.set("title", input.title);
  body.set("body", input.body);
  body.set("tickers", input.tickers);
  body.set("topics", input.topics);
  body.set("source_url", input.sourceUrl);
  if (input.attachment) body.set("attachment", input.attachment);
  const response = await apiFetch("/api/public/community/forum/posts", {
    method: "POST",
    body,
  });
  return parseJson<CommunityForumPost>(response);
}

async function mutateCommunityForumPost(
  postId: string,
  suffix: string,
  init: RequestInit,
) {
  const response = await apiFetch(
    `/api/public/community/forum/posts/${encodeURIComponent(postId)}${suffix}`,
    init,
  );
  return parseJson<CommunityForumPost>(response);
}

export function toggleCommunityForumLike(postId: string) {
  return mutateCommunityForumPost(postId, "/like", { method: "POST" });
}

export function commentCommunityForumPost(postId: string, body: string) {
  return mutateCommunityForumPost(postId, "/comments", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ body }),
  });
}

export function reportCommunityForumPost(postId: string, reason: string) {
  return mutateCommunityForumPost(postId, "/report", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ reason }),
  });
}

export function moderateCommunityForumPost(postId: string, action: "hide" | "restore") {
  return mutateCommunityForumPost(postId, "/moderation", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ action }),
  });
}

export function deleteCommunityForumPost(postId: string) {
  return mutateCommunityForumPost(postId, "", { method: "DELETE" });
}

export function deleteCommunityForumComment(postId: string, commentId: string) {
  return mutateCommunityForumPost(
    postId,
    `/comments/${encodeURIComponent(commentId)}`,
    { method: "DELETE" },
  );
}

export function communityForumAttachmentUrl(postId: string, attachmentId: string) {
  return `/api/public/community/forum/posts/${encodeURIComponent(postId)}/attachments/${encodeURIComponent(attachmentId)}`;
}

export async function markPublicCommunitySeen(contentId: number) {
  const response = await apiFetch("/api/public/community/seen", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ content_id: contentId }),
  });
  return parseJson<{ ok: boolean }>(response);
}

function publicCommunityResourcePath(
  resourceId: number,
  version?: string | null,
) {
  const normalizedVersion = version?.trim();
  const suffix = normalizedVersion
    ? `?${new URLSearchParams({ v: normalizedVersion }).toString()}`
    : "";
  return `/api/public/community/resources/${resourceId}${suffix}`;
}

export function publicCommunityResourceUrl(
  resourceId: number,
  version?: string | null,
  deliveryPath?: string | null,
) {
  return buildApiUrl(
    verifiedPublicCommunityDeliveryPath(resourceId, version, deliveryPath) ??
      publicCommunityResourcePath(resourceId, version),
  );
}

export async function resolvePublicCommunityResourceUrl(
  resourceId: number,
  version?: string | null,
  deliveryPath?: string | null,
) {
  const legacyUrl = buildApiUrl(
    publicCommunityResourcePath(resourceId, version),
  );
  const edgePath = verifiedPublicCommunityDeliveryPath(
    resourceId,
    version,
    deliveryPath,
  );
  if (!edgePath) return legacyUrl;
  try {
    const response = await fetchPublicCommunityEdge(edgePath, {
      method: "HEAD",
    });
    return response.ok ? buildApiUrl(edgePath) : legacyUrl;
  } catch {
    return legacyUrl;
  }
}

export function publicCommunityResourceDownloadName(
  resource: Pick<
    PublicCommunityResource,
    "resource_id" | "display_name" | "content_type"
  >,
) {
  const fallback = `community-resource-${resource.resource_id}`;
  const displayName = resource.display_name?.trim() || fallback;
  const contentType = (resource.content_type ?? "")
    .split(";", 1)[0]!
    .trim()
    .toLowerCase();
  if (
    contentType ===
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" &&
    /\.xls$/i.test(displayName)
  ) {
    return displayName.replace(/\.xls$/i, ".xlsx");
  }
  return displayName;
}

export async function getPublicCommunityResourceBlob(
  resourceId: number,
  version?: string | null,
  deliveryPath?: string | null,
) {
  const edgePath = verifiedPublicCommunityDeliveryPath(
    resourceId,
    version,
    deliveryPath,
  );
  if (edgePath) {
    try {
      const edgeResponse = await fetchPublicCommunityEdge(edgePath);
      if (edgeResponse.ok) return edgeResponse.blob();
    } catch {
      // The legacy authenticated API remains the per-resource safety net.
    }
  }
  const response = await apiFetch(
    publicCommunityResourcePath(resourceId, version),
  );
  if (!response.ok) throw await apiErrorFromResponse(response);
  return response.blob();
}

export async function getCronJobs(actor?: ActorRef) {
  const url = actor ? `/api/cron-jobs?${actorQuery(actor)}` : "/api/cron-jobs";
  const response = await apiFetch(url);
  const payload = await parseJson<{ jobs: CronJobInfo[] }>(response);
  return payload.jobs;
}

export async function getCronJob(id: string, actor?: ActorRef) {
  const url = actor
    ? `/api/cron-jobs/${encodeURIComponent(id)}?${actorQuery(actor)}`
    : `/api/cron-jobs/${encodeURIComponent(id)}`;
  const response = await apiFetch(url);
  const payload = await parseJson<{ job: CronJobDetailInfo }>(response);
  return payload.job;
}

export async function createCronJob(input: CronJobUpsertInput) {
  const response = await apiFetch("/api/cron-jobs", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{ job: CronJobInfo }>(response);
  return payload.job;
}

export async function updateCronJob(
  id: string,
  input: CronJobUpsertInput,
  actor?: ActorRef,
) {
  const url = actor
    ? `/api/cron-jobs/${encodeURIComponent(id)}?${actorQuery(actor)}`
    : `/api/cron-jobs/${encodeURIComponent(id)}`;
  const response = await apiFetch(url, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{ job: CronJobInfo }>(response);
  return payload.job;
}

export async function toggleCronJob(id: string, actor?: ActorRef) {
  const url = actor
    ? `/api/cron-jobs/${encodeURIComponent(id)}/toggle?${actorQuery(actor)}`
    : `/api/cron-jobs/${encodeURIComponent(id)}/toggle`;
  const response = await apiFetch(url, { method: "POST" });
  const payload = await parseJson<{ job: CronJobInfo }>(response);
  return payload.job;
}

export async function deleteCronJob(id: string, actor?: ActorRef) {
  const url = actor
    ? `/api/cron-jobs/${encodeURIComponent(id)}?${actorQuery(actor)}`
    : `/api/cron-jobs/${encodeURIComponent(id)}`;
  const response = await apiFetch(url, { method: "DELETE" });
  await parseJson(response);
  return true;
}

export async function listPortfolioActors() {
  const response = await apiFetch("/api/portfolio/actors");
  const payload = await parseJson<{ actors: PortfolioSummary[] }>(response);
  return payload.actors ?? [];
}

export async function getPortfolio(actor: ActorRef) {
  const response = await apiFetch(`/api/portfolio?${actorQuery(actor)}`);
  const payload = await parseJson<{
    portfolio: PortfolioInfo;
    summary: PortfolioSummary;
  }>(response);
  return payload;
}

export async function createHolding(input: HoldingUpsertInput) {
  const response = await apiFetch(`/api/portfolio/holdings`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{
    portfolio: PortfolioInfo;
    summary: PortfolioSummary;
  }>(response);
  return payload;
}

export async function updateHolding(symbol: string, input: HoldingUpsertInput) {
  const response = await apiFetch(
    `/api/portfolio/holdings/${encodeURIComponent(symbol)}`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(input),
    },
  );
  const payload = await parseJson<{
    portfolio: PortfolioInfo;
    summary: PortfolioSummary;
  }>(response);
  return payload;
}

export async function deleteHolding(symbol: string, actor: ActorRef) {
  const response = await apiFetch(
    `/api/portfolio/holdings/${encodeURIComponent(symbol)}?${actorQuery(actor)}`,
    {
      method: "DELETE",
    },
  );
  const payload = await parseJson<{
    portfolio: PortfolioInfo;
    summary: PortfolioSummary;
  }>(response);
  return payload;
}

// ── 个股深度研究 API ──────────────────────────────────────────────────────────

export type ResearchStartResponse = {
  message: string;
  task_id: string;
  task_name: string;
};

export type ResearchStatusResponse = {
  task_id: string;
  task_name: string;
  status: string;
  progress: string;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
  info: string | null;
  answer_file_path?: string;
  answer_exists?: boolean;
  /** 任务完成且文件存在时，直接返回 Markdown 原文 */
  answer_markdown?: string;
};

/** 接口一：发起深度研究，返回 task_id */
export async function startResearch(
  companyName: string,
): Promise<ResearchStartResponse> {
  const response = await apiFetch("/api/research/start", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ companyName }),
  });
  return parseJson<ResearchStartResponse>(response);
}

/** 接口二：轮询研究进度（完成时含 answer_markdown 原文） */
export async function getResearchStatus(
  taskId: string,
): Promise<ResearchStatusResponse> {
  const response = await apiFetch(
    `/api/research/status/${encodeURIComponent(taskId)}`,
  );
  return parseJson<ResearchStatusResponse>(response);
}

// ── 日志 API ─────────────────────────────────────────────────────────────────

/** 获取历史日志（最多 500 条） */
export async function getLogs(): Promise<LogEntry[]> {
  const response = await apiFetch("/api/logs");
  const payload = await parseJson<{ logs: LogEntry[] }>(response);
  return payload.logs ?? [];
}

// ── Task runs (周期任务观测) ────────────────────────────────────────────────

export type TaskOutcome = "ok" | "skipped" | "failed";

export interface TaskRunRecord {
  task: string;
  started_at: string;
  ended_at: string;
  outcome: TaskOutcome;
  items: number;
  error?: string | null;
}

export interface TaskSummary {
  last_seen_at: string | null;
  runs_24h: number;
  ok_24h: number;
  skipped_24h: number;
  failed_24h: number;
  last_error: string | null;
  last_failure_at: string | null;
  /// 最近一次失败之后又跑了多少次(ok/skipped 都算)。
  /// null = 24h 内没失败过;0 = 最新这次就是失败;>0 = 已恢复 N 次。
  runs_since_last_failure: number | null;
}

export interface TaskRunsResponse {
  runs: TaskRunRecord[];
  summary_by_task: Record<string, TaskSummary>;
  runtime_dir: string;
}

export async function getTaskRuns(opts?: {
  days?: number;
  limit?: number;
  task?: string;
}): Promise<TaskRunsResponse> {
  const params = new URLSearchParams();
  if (opts?.days != null) params.set("days", String(opts.days));
  if (opts?.limit != null) params.set("limit", String(opts.limit));
  if (opts?.task) params.set("task", opts.task);
  const qs = params.toString();
  const path = qs ? `/api/admin/task-runs?${qs}` : "/api/admin/task-runs";
  const response = await apiFetch(path);
  return parseJson<TaskRunsResponse>(response);
}

/** 连接实时日志 SSE 流 */
export async function connectLogStream() {
  return createEventSource("/api/logs/stream");
}

// ── 推送日志 API (cron 执行记录跨任务聚合) ────────────────────────────────

export interface NotificationRecord {
  run_id: number;
  record_source: "cron_job" | "event_engine" | string;
  job_id: string;
  job_name: string;
  event_kind?: string | null;
  channel: string;
  user_id: string;
  channel_scope?: string | null;
  channel_target: string;
  heartbeat: boolean;
  executed_at: string;
  execution_status: string;
  message_send_status: string;
  should_deliver: boolean;
  delivered: boolean;
  response_preview?: string | null;
  error_message?: string | null;
  detail?: unknown;
}

export interface NotificationHistogramBucket {
  bucket_start: string;
  total: number;
  sent: number;
  failed: number;
  skipped: number;
}

export interface NotificationsSummary {
  total: number;
  sent: number;
  failed: number;
  skipped: number;
  duplicate_suppressed: number;
  distinct_users: number;
}

export interface NotificationsResponse {
  records: NotificationRecord[];
  histogram_24h: NotificationHistogramBucket[];
  summary_24h: NotificationsSummary;
}

export interface NotificationsQuery {
  since?: string;
  until?: string;
  channel?: string;
  user_id?: string;
  channel_scope?: string;
  job_id?: string;
  execution_status?: string;
  message_send_status?: string;
  heartbeat_only?: boolean;
  limit?: number;
}

export async function getNotifications(
  q: NotificationsQuery = {},
): Promise<NotificationsResponse> {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(q)) {
    if (value === undefined || value === null || value === "") continue;
    params.set(key, String(value));
  }
  const qs = params.toString();
  const path = qs
    ? `/api/admin/notifications?${qs}`
    : "/api/admin/notifications";
  const response = await apiFetch(path);
  return parseJson<NotificationsResponse>(response);
}

// ── 推送日程 API (per-actor 拍平视图) ────────────────────────────────────────

export type ScheduleSource = "digest" | "cron_job";

export interface ScheduleEntry {
  time_local: string;
  source: ScheduleSource;
  content_hint: string;
  frequency: string;
  job_id?: string | null;
  will_be_held_by_quiet: boolean;
  bypass_quiet_hours: boolean;
  edit_hint: string;
}

export interface QuietHoursView {
  from: string;
  to: string;
  exempt_kinds: string[];
}

export type PricePolicySource = "system" | "actor_common" | "actor_direction";

export interface EffectivePriceDirectionPolicy {
  configured_first_pct: number;
  configured_first_source: PricePolicySource;
  first_direct_pct: number;
  system_floor_applied: boolean;
  large_position_first_direct_pct: number;
  first_candidate_band_pct: number;
  large_position_first_candidate_band_pct: number;
}

export interface EffectivePriceAlertPolicy {
  up: EffectivePriceDirectionPolicy;
  down: EffectivePriceDirectionPolicy;
  repeat_step_pct: number;
  repeat_step_source: PricePolicySource;
  candidate_first_pct: number;
  candidate_step_pct: number;
  min_direct_pct: number;
  large_position_weight_pct: number;
  close_direct_enabled: boolean;
}

export interface ImmediateConfig {
  event_engine_enabled: boolean;
  globally_disabled_kinds: string[];
  enabled: boolean;
  min_severity: string;
  portfolio_only: boolean;
  high_severity_daily_cap: number;
  same_symbol_cooldown_minutes: number;
  price_high_pct?: number | null;
  price_high_pct_up?: number | null;
  price_high_pct_down?: number | null;
  price_realert_step_pct?: number | null;
  large_position_weight_pct?: number | null;
  effective_price_alert_policy: EffectivePriceAlertPolicy;
  price_ladder_examples: { up: number[]; down: number[] };
  allow_kinds?: string[] | null;
  blocked_kinds: string[];
  immediate_kinds?: string[] | null;
  exempt_in_quiet: string[];
}

export interface ScheduleOverview {
  actor: string;
  timezone: string;
  quiet_hours?: QuietHoursView | null;
  schedule: ScheduleEntry[];
  immediate: ImmediateConfig;
}

export async function getSchedule(actor: string): Promise<ScheduleOverview> {
  const params = new URLSearchParams();
  params.set("actor", actor);
  const path = `/api/admin/schedule?${params.toString()}`;
  const response = await apiFetch(path);
  return parseJson<ScheduleOverview>(response);
}

// ── LLM Audit API ─────────────────────────────────────────────────────────────

import type {
  AuditQueryFilter,
  AuditRecordSummary,
  LlmAuditRecord,
} from "./types";

export async function getAuditRecords(filter: AuditQueryFilter) {
  const params = new URLSearchParams();
  for (const [k, v] of Object.entries(filter)) {
    if (v !== undefined && v !== "") {
      params.set(k, String(v));
    }
  }
  const response = await apiFetch(`/api/llm-audit?${params.toString()}`);
  return parseJson<{ records: AuditRecordSummary[]; total: number }>(response);
}

export async function getAuditRecordDetail(id: string) {
  const response = await apiFetch(`/api/llm-audit/${encodeURIComponent(id)}`);
  return parseJson<LlmAuditRecord>(response);
}

export async function listCompanyProfileActors() {
  const response = await apiFetch("/api/company-profiles/actors");
  const payload = await parseJson<{ actors: CompanyProfileSpaceSummary[] }>(
    response,
  );
  return payload.actors ?? [];
}

export async function listCompanyProfiles(actor: ActorRef) {
  const response = await apiFetch(`/api/company-profiles?${actorQuery(actor)}`);
  const payload = await parseJson<{ profiles: CompanyProfileSummary[] }>(
    response,
  );
  return payload.profiles;
}

export async function getCompanyProfile(profileId: string, actor: ActorRef) {
  const response = await apiFetch(
    `/api/company-profiles/${encodeURIComponent(profileId)}?${actorQuery(actor)}`,
  );
  const payload = await parseJson<{ profile: CompanyProfile }>(response);
  return payload.profile;
}

export async function deleteCompanyProfile(profileId: string, actor: ActorRef) {
  const response = await apiFetch(
    `/api/company-profiles/${encodeURIComponent(profileId)}?${actorQuery(actor)}`,
    {
      method: "DELETE",
    },
  );
  return parseJson<{ ok: boolean }>(response);
}

function parseDownloadFilename(response: Response, fallback: string) {
  const disposition = response.headers.get("content-disposition") ?? "";
  const match = disposition.match(/filename="([^"]+)"/i);
  return match?.[1]?.trim() || fallback;
}

export async function exportCompanyProfiles(actor: ActorRef) {
  const response = await apiFetch(
    `/api/company-profiles/export?${actorQuery(actor)}`,
  );
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || response.statusText);
  }
  const blob = await response.blob();
  const fallback = `company-profiles-${actor.channel}-${actor.user_id}.zip`;
  return {
    blob,
    fileName: parseDownloadFilename(response, fallback),
  };
}

export async function previewImportCompanyProfiles(
  actor: ActorRef,
  bundle: File,
) {
  const form = new FormData();
  form.append("bundle", bundle);
  const response = await apiFetch(
    `/api/company-profiles/import/preview?${actorQuery(actor)}`,
    {
      method: "POST",
      body: form,
    },
  );
  const payload = await parseJson<{ preview: CompanyProfileImportPreview }>(
    response,
  );
  return payload.preview;
}

export async function applyImportCompanyProfiles(
  actor: ActorRef,
  bundle: File,
  request: CompanyProfileImportApplyRequest,
) {
  const form = new FormData();
  form.append("bundle", bundle);
  form.append("mode", request.mode);
  form.append("decisions", JSON.stringify(request.decisions));
  const response = await apiFetch(
    `/api/company-profiles/import/apply?${actorQuery(actor)}`,
    {
      method: "POST",
      body: form,
    },
  );
  const payload = await parseJson<{ result: CompanyProfileImportApplyResult }>(
    response,
  );
  return payload.result;
}

// ── 通知偏好 API ──────────────────────────────────────────────────────────

/** 单个 digest 槽位 —— 后端 v0.4.x 起的新 schema(替代旧 digest_windows: string[])。
 *  时刻按 prefs.timezone 解释为本地 HH:MM;label 用于渲染 header,floor_macro 控制
 *  Pass 2 personalize 至少保留几条 macro_floor。前端编辑面板只渲染/写 id+time,
 *  label/floor_macro 透传不破坏。 */
export type DigestSlot = {
  id: string;
  time: string;
  label?: string | null;
  floor_macro?: number | null;
};

/** 勿扰时段:from/to 都是 prefs.timezone 解释的本地 HH:MM。在区间内 hold immediate
 *  推送 + 跳过 digest 触发,到 to 时刻一次性 quiet_flush;exempt_kinds 命中的 kind
 *  即使在 quiet 内也立即推。 */
export type QuietHoursPrefs = {
  from: string;
  to: string;
  exempt_kinds: string[];
};

export type NotificationPrefs = {
  enabled: boolean;
  portfolio_only: boolean;
  min_severity: "low" | "medium" | "high";
  allow_kinds: string[] | null;
  blocked_kinds: string[];
  /** IANA 时区名;null = 沿用全局 digest.timezone */
  timezone: string | null;
  /** digest 触发槽位列表;null = 沿用全局 default_slots;[] = 关 digest */
  digest_slots: DigestSlot[] | null;
  /** 价格异动即时推阈值(百分点);null = 沿用全局 thresholds.price_alert_high_pct */
  price_high_pct_override: number | null;
  /** 上涨方向价格异动阈值;null = 回落到通用 actor override / 全局阈值 */
  price_high_pct_up_override: number | null;
  /** 下跌方向价格异动阈值;null = 回落到通用 actor override / 全局阈值 */
  price_high_pct_down_override: number | null;
  /** 首次命中后的重复提醒最小前进步长;null = 沿用全局价格 band 推送步长 */
  price_realert_step_pct_override: number | null;
  /** 被视为大仓位的持仓权重百分比;null = 沿用全局 router 配置 */
  large_position_weight_pct: number | null;
  /** 强制升 High 即时推的 kind tag 列表;null/[] = 不强升 */
  immediate_kinds: string[] | null;
  /** 勿扰时段,null = 不启用 */
  quiet_hours: QuietHoursPrefs | null;
};

export type NotificationPrefsBundle = {
  prefs: NotificationPrefs;
  kind_tags: string[];
};

export type NotificationPrefsBatchEntry = {
  actor: ActorRef;
  prefs: NotificationPrefs;
};

export type NotificationPrefsBatchBundle = {
  entries: NotificationPrefsBatchEntry[];
  kind_tags: string[];
};

export async function getNotificationPrefs(
  actor: ActorRef,
): Promise<NotificationPrefsBundle> {
  const response = await apiFetch(
    `/api/notification-prefs?${actorQuery(actor)}`,
  );
  return parseJson<NotificationPrefsBundle>(response);
}

export async function getNotificationPrefsBatch(
  actors: ActorRef[],
): Promise<NotificationPrefsBatchBundle> {
  const response = await apiFetch("/api/notification-prefs/batch", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ actors }),
  });
  return parseJson<NotificationPrefsBatchBundle>(response);
}

export async function putNotificationPrefs(
  actor: ActorRef,
  prefs: NotificationPrefs,
): Promise<NotificationPrefs> {
  const body = {
    channel: actor.channel,
    user_id: actor.user_id,
    channel_scope: actor.channel_scope,
    prefs,
  };
  const response = await apiFetch("/api/notification-prefs", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const payload = await parseJson<{ prefs: NotificationPrefs }>(response);
  return payload.prefs;
}

export async function putLanguage(language: "zh" | "en"): Promise<"zh" | "en"> {
  const response = await apiFetch("/api/language", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ language }),
  });
  const payload = await parseJson<{ language: "zh" | "en" }>(response);
  return payload.language;
}

export async function listPublicSubscriptions(signal?: AbortSignal) {
  const response = await apiFetch("/api/public/subscriptions", { signal });
  const payload = await parseJson<{ subscriptions: PublicSubscription[] }>(
    response,
  );
  return payload.subscriptions;
}

export async function updatePublicSubscription(
  jobId: string,
  patch: {
    name?: string;
    task_prompt?: string;
    hour?: number;
    minute?: number;
    repeat?: string;
    weekday?: number | null;
    date?: string | null;
    enabled?: boolean;
  },
) {
  const response = await apiFetch(
    `/api/public/subscriptions/${encodeURIComponent(jobId)}`,
    {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(patch),
    },
  );
  const payload = await parseJson<{ subscription: PublicSubscription }>(
    response,
  );
  return payload.subscription;
}

export async function unsubscribePublicSubscription(jobId: string) {
  const response = await apiFetch(
    `/api/public/subscriptions/${encodeURIComponent(jobId)}/unsubscribe`,
    { method: "POST" },
  );
  return parseJson<{
    subscription: PublicSubscription;
    already_unsubscribed: boolean;
  }>(response);
}

export async function getPublicCompanyRatings(
  signal?: AbortSignal,
): Promise<CompanyRatingSnapshot> {
  const response = await apiFetch("/api/public/company-ratings", { signal });
  return parseJson<CompanyRatingSnapshot>(response);
}

export async function getPublicValuationLab(
  signal?: AbortSignal,
): Promise<ValuationLabSnapshot> {
  const response = await apiFetch("/api/public/valuation-lab", { signal });
  return parseJson<ValuationLabSnapshot>(response);
}

export async function getPublicPortfolioNews(
  signal?: AbortSignal,
): Promise<PortfolioNewsSnapshot> {
  const response = await apiFetch("/api/public/portfolio-news", { signal });
  return parseJson<PortfolioNewsSnapshot>(response);
}

export async function getPublicPositionManagement(
  signal?: AbortSignal,
): Promise<PositionManagementSnapshot> {
  const response = await apiFetch("/api/public/position-management", {
    signal,
  });
  return parseJson<PositionManagementSnapshot>(response);
}

export async function getPublicInfluencerDigest(
  signal?: AbortSignal,
): Promise<InfluencerDigestSnapshot> {
  const response = await apiFetch("/api/public/influencer-digest", { signal });
  return parseJson<InfluencerDigestSnapshot>(response);
}

export async function getPublicKeyEventChains(
  signal?: AbortSignal,
): Promise<KeyEventChainSnapshot> {
  const response = await apiFetch("/api/public/key-event-chains", { signal });
  return parseJson<KeyEventChainSnapshot>(response);
}

export async function getPublicWeeklyBrief(
  signal?: AbortSignal,
): Promise<WeeklyBriefPayload> {
  const response = await apiFetch("/api/public/weekly-brief", { signal });
  return parseJson<WeeklyBriefPayload>(response);
}

export async function getPublicResearchLibrary(
  signal?: AbortSignal,
): Promise<ResearchLibraryBundle> {
  const response = await apiFetch("/api/public/research-library", { signal });
  return parseJson<ResearchLibraryBundle>(response);
}

export async function uploadPublicResearchLibrary(form: FormData): Promise<{
  item: ResearchLibraryItem;
  deduplicated: boolean;
}> {
  const response = await apiFetch("/api/public/research-library", {
    method: "POST",
    body: form,
  });
  return parseJson<{ item: ResearchLibraryItem; deduplicated: boolean }>(
    response,
  );
}

export async function updatePublicResearchLibrary(
  id: string,
  patch: Partial<
    Pick<
      ResearchLibraryItem,
      | "title"
      | "source_name"
      | "source_url"
      | "source_date"
      | "tickers"
      | "topics"
      | "uses"
    >
  >,
): Promise<ResearchLibraryItem> {
  const response = await apiFetch(
    `/api/public/research-library/${encodeURIComponent(id)}`,
    {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(patch),
    },
  );
  const payload = await parseJson<{ item: ResearchLibraryItem }>(response);
  return payload.item;
}

export async function deletePublicResearchLibrary(id: string): Promise<void> {
  const response = await apiFetch(
    `/api/public/research-library/${encodeURIComponent(id)}`,
    {
      method: "DELETE",
    },
  );
  await parseJson<{ deleted: boolean }>(response);
}

export async function submitPublicResearchLibraryCandidate(
  id: string,
): Promise<{
  item: ResearchLibraryItem;
  deduplicated: boolean;
}> {
  const response = await apiFetch(
    `/api/public/research-library/${encodeURIComponent(id)}/submit`,
    { method: "POST" },
  );
  return parseJson<{ item: ResearchLibraryItem; deduplicated: boolean }>(
    response,
  );
}

export async function reviewPublicResearchLibraryCandidate(
  id: string,
  decision: "approve" | "reject",
  note = "",
): Promise<{
  item: ResearchLibraryItem;
  promoted_item?: ResearchLibraryItem | null;
}> {
  const response = await apiFetch(
    `/api/public/research-library/${encodeURIComponent(id)}/review`,
    {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ decision, note }),
    },
  );
  return parseJson<{
    item: ResearchLibraryItem;
    promoted_item?: ResearchLibraryItem | null;
  }>(response);
}

export async function getPublicDailySignal(
  kind: DailySignalKind,
  signal?: AbortSignal,
): Promise<DailySignalReport> {
  const response = await apiFetch(`/api/public/daily-signals/${kind}`, {
    signal,
  });
  return parseJson<DailySignalReport>(response);
}

export async function getPublicDailySignalHistory(
  kind: DailySignalKind,
  limit = 14,
  signal?: AbortSignal,
): Promise<{ items: DailySignalHistoryItem[] }> {
  const response = await apiFetch(
    `/api/public/daily-signals/${kind}/history?limit=${limit}`,
    { signal },
  );
  return parseJson<{ items: DailySignalHistoryItem[] }>(response);
}
