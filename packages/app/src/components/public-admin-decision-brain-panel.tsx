import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { PublicAdminFinancialEvidenceReview } from "./public-admin-financial-evidence-review";
import { PublicAdminValuationInputReview } from "./public-admin-valuation-input-review";
import { PublicAdminHistoricalAnchorPanel } from "./public-admin-historical-anchor-panel";
import { PublicAdminHistoricalStateReconstructionPanel } from "./public-admin-historical-state-reconstruction-panel";
import { PublicAdminHistoricalOutcomeGovernancePanel } from "./public-admin-historical-outcome-governance-panel";
import {
  getInvestmentEvidenceReviewQueue,
  getInvestmentCausalDatasetGovernance,
  getInvestmentCausalTrainingExperiments,
  getInvestmentDecisionEvaluation,
  getInvestmentDecisionReplay,
  getInvestmentRewardGovernance,
  getInvestmentShadowProtocolGovernance,
  getInvestmentShadowImplementations,
  registerInvestmentShadowImplementation,
  reviewInvestmentDecision,
  reviewInvestmentCausalEvidence,
  reviewInvestmentCausalSource,
  reviewInvestmentCausalDatasetGovernance,
  reviewInvestmentRewardGovernance,
  reviewInvestmentShadowProtocolGovernance,
} from "@/lib/api";
import type {
  InvestmentDecisionErrorKind,
  InvestmentEvidenceReviewQueue,
  InvestmentEvidenceReviewQueueItem,
  InvestmentCausalDatasetGovernance,
  InvestmentCausalDatasetGovernanceVerdict,
  InvestmentCausalTrainingExperimentRegistry,
  InvestmentCausalObservation,
  InvestmentDecisionEvaluation,
  InvestmentDecisionReplay,
  InvestmentDecisionReviewRequest,
  InvestmentDecisionTrainingSample,
  InvestmentExposureAction,
  InvestmentRewardGovernance,
  InvestmentRewardGovernanceVerdict,
  InvestmentShadowProtocolGovernance,
  InvestmentShadowProtocolGovernanceVerdict,
  InvestmentShadowImplementationRegistry,
  InvestmentReviewStatus,
  InvestmentThesisVerdict,
} from "@/lib/types";

const ACTION_LABELS: Record<InvestmentExposureAction, string> = {
  increase_candidate: "加仓候选",
  maintain: "维持",
  reduce_candidate: "减仓候选",
  research_only: "仅研究",
};

const REVIEW_LABELS: Record<InvestmentReviewStatus, string> = {
  pending: "待复核",
  accepted: "接受",
  corrected: "已修正",
  rejected: "已否决",
};

const ERROR_LABELS: Record<InvestmentDecisionErrorKind, string> = {
  industry_thesis: "产业逻辑",
  company_value_capture: "公司价值捕获",
  financial_transmission: "财务传导",
  valuation: "估值",
  timing_crowding: "时机与拥挤度",
  data_quality: "数据质量",
  policy_mapping: "判断到动作映射",
  other: "其他",
};

const CAUSAL_RELATIONSHIP_LABELS = {
  direct_metric: "直接指标",
  proxy: "代理指标",
  confirmed_context: "一手背景",
  structured_source_claim: "待核验财报主张",
  computed_comparison: "两期确定性计算",
  computed_ratio: "同期确定性比率",
  computed_ratio_trend: "利润率趋势",
  operating_kpi_claim: "公司经营指标",
} as const;

const CAUSAL_PROMOTION_LABELS = {
  training_only: "仅训练",
  pending_repeat_evidence: "等待跨期证据",
  pending_human_review: "等待人工复核",
  blocked_conflict: "冲突冻结",
  blocked_human_rejection: "人工否决冻结",
  blocked_falsification: "证伪冻结",
  promoted_confidence_only: "已晋级（仅置信度）",
} as const;

const CLAIM_LIFECYCLE_LABELS = {
  active: "有效",
  superseded: "已被更正",
  conflicted: "相互冲突",
  withdrawn: "已撤回",
} as const;

const QUEUE_STATUS_LABELS = {
  pending: "待复核",
  accepted: "已接受",
  rejected: "已拒绝",
} as const;

const QUEUE_KIND_LABELS = {
  source_claim: "一手事实",
  operating_kpi: "公司经营指标",
  computed_comparison: "同比/环比",
  computed_ratio: "利润率",
} as const;

const MARKET_REGIME_LABELS: Record<string, string> = {
  supportive: "环境偏支持",
  balanced: "环境均衡",
  defensive: "环境偏防御",
  stress: "压力环境",
};

const CROWDING_STATUS_LABELS = {
  unmeasured: "尚未测量",
  partially_measured: "部分测量",
  measured: "完整测量",
} as const;

const KPI_COMPARABILITY_LABELS = {
  standardized_metric: "标准化口径",
  within_issuer_only: "仅限本公司跨期",
  contract_milestone: "合同/里程碑口径",
} as const;

const REWARD_GOVERNANCE_LABELS: Record<InvestmentRewardGovernanceVerdict, string> = {
  changes_requested: "已要求修改",
  approved_for_offline_research: "已批准离线研究",
  rejected: "已否决",
};

const SHADOW_PROTOCOL_GOVERNANCE_LABELS: Record<
  InvestmentShadowProtocolGovernanceVerdict,
  string
> = {
  changes_requested: "已要求修改",
  approved_for_future_shadow_implementation: "已批准未来实现登记",
  rejected: "已否决",
};

const DATASET_GOVERNANCE_LABELS: Record<InvestmentCausalDatasetGovernanceVerdict, string> = {
  changes_requested: "已要求修改",
  approved_for_offline_experiment: "已批准登记离线实验",
  rejected: "已否决",
};

const CAUSAL_DATASET_TARGET_LABELS: Record<string, string> = {
  supports: "支持",
  falsifies: "证伪",
  mixed: "正反混合",
  context_only: "仅作背景",
  relationship_rejected: "关系不成立",
};

const METHODOLOGY_STATUS_LABELS: Record<string, string> = {
  passed: "已通过",
  blocked: "证据不足",
  delegated_to_portfolio: "组合层审查",
};

const FIRST_PRINCIPLES_MODEL_LABELS: Record<string, string> = {
  "ai-storage-demand-supply": "AI 存储",
  "ai-compute-effective-capacity": "AI 算力",
  "ai-optical-interconnect-bandwidth": "光互连",
  "ai-data-center-power-delivery": "数据中心电力",
  "ai-platform-token-economics": "AI 平台",
  "ai-application-workflow-value": "AI 应用",
};

const FIRST_PRINCIPLES_STATE_LABELS: Record<string, string> = {
  falsification_blocked: "已有证伪，冻结",
  contested: "证据冲突",
  partially_human_supported: "部分人工支持",
  reality_measured_unreviewed: "已有量化，待复核",
  measurement_partial: "部分进入量化层",
  traceable_evidence_unmeasured: "证据可追溯，待量化",
  traceable_evidence_partial: "可追溯证据不完整",
  structure_only: "仅有结构",
};

const MEASUREMENT_BACKLOG_STATUS_LABELS: Record<string, string> = {
  ready_for_measurement_review: "可复核量化",
  review_rejected_needs_new_evidence: "需新证据",
  source_claims_need_metricization: "文字待指标化",
  context_needs_operating_kpi: "待补经营指标",
  no_traceable_evidence: "待采集一手证据",
};

const DRIVER_FAMILY_LABELS: Record<string, string> = {
  demand: "需求",
  supply: "供给",
  value_capture: "价值捕获",
};

export type DecisionReviewDraft = {
  mode: "accepted" | "corrected" | "rejected";
  verdict: Exclude<InvestmentThesisVerdict, "pending">;
  note: string;
  correctedAction: InvestmentExposureAction;
  errorKind: InvestmentDecisionErrorKind;
  errorSeverity: "minor" | "material" | "critical";
  errorExplanation: string;
};

type CausalEvidenceEffect = "unclassified" | "supports" | "falsifies" | "mixed" | "context_only";
type CausalSpeakerConfirmation = "" | "old_wang_confirmed";
type CausalSourceVerification = "" | "verified_against_source" | "evidence_mismatch" | "insufficient_source_context";
type CausalReviewStage = "source" | "verbatim" | "relationship" | "boundary" | "falsifier" | "confirmation";

type CausalReviewDraft = {
  verdict: "" | "accepted" | "rejected";
  effect: CausalEvidenceEffect;
  explanation: string;
  verbatimJudgment: string;
  applicabilityBoundary: string;
  falsifier: string;
  speakerConfirmation: CausalSpeakerConfirmation;
  sourceVerification: CausalSourceVerification;
  sourceVerificationNote: string;
  oldWangConfirmationAttested: boolean;
  stage: CausalReviewStage;
  reviewId?: string;
  sourceReviewId?: string;
};

export function buildDecisionReviewRequest(
  sample: InvestmentDecisionTrainingSample,
  draft: DecisionReviewDraft,
): InvestmentDecisionReviewRequest {
  const note = draft.note.trim();
  const explanation = draft.errorExplanation.trim();
  if (draft.mode === "accepted") {
    return {
      expected_review_id: sample.human_review.review_id,
      status: "accepted",
      thesis_verdict: draft.verdict,
      error_attributions: [],
    };
  }
  return {
    expected_review_id: sample.human_review.review_id,
    status: draft.mode,
    thesis_verdict: draft.mode === "rejected" ? "invalidated" : draft.verdict,
    correction_note: note || undefined,
    corrected_action:
      draft.mode === "corrected" ? draft.correctedAction : undefined,
    error_attributions: explanation
      ? [
          {
            kind: draft.errorKind,
            severity: draft.errorSeverity,
            explanation,
            evidence_ids: [],
          },
        ]
      : [],
  };
}

export function decisionReviewDraftIsValid(draft: DecisionReviewDraft) {
  if (draft.mode === "accepted") return draft.verdict !== "invalidated";
  if (!draft.note.trim()) return false;
  if (draft.mode === "rejected" && !draft.errorExplanation.trim()) return false;
  return true;
}

export function causalObservationCanBeAccepted(
  observation: InvestmentCausalObservation,
) {
  const provenanceCount = Number(Boolean(observation.claim)) + Number(Boolean(observation.computed)) + Number(Boolean(observation.ratio)) + Number(Boolean(observation.ratio_trend)) + Number(Boolean(observation.operating_kpi));
  if (provenanceCount > 1) return false;
  if (observation.operating_kpi) {
    const claim = observation.operating_kpi;
    const normalize = (value: string) => value.trim().toLocaleLowerCase().replace(/\s+/g, " ");
    const sourceText = normalize(`${claim.value_text} ${claim.evidence_quote}`);
    return claim.schema_version === "hone-operating-kpi-claim-v1"
      && Boolean(claim.kpi_id.trim())
      && Boolean(claim.issuer_metric_name.trim())
      && Boolean(claim.issuer_definition.trim())
      && sourceText.includes(normalize(claim.issuer_definition))
      && Boolean(claim.definition_key.trim())
      && Boolean(claim.period.trim())
      && Boolean(claim.measurement_scope.trim())
      && Boolean(claim.source_event_id.trim())
      && Boolean(claim.source_locator.trim())
      && Boolean(claim.evidence_quote.trim())
      && (claim.numeric_value == null || (Number.isFinite(claim.numeric_value) && Boolean(claim.unit.trim())))
      && claim.lifecycle_status === "active"
      && claim.disposition === "active"
      && !claim.superseded_by
      && claim.conflicting_claim_ids.length === 0
      && Boolean(observation.source_url?.startsWith("https://"));
  }
  if (observation.ratio_trend) {
    const trend = observation.ratio_trend;
    return trend.formula_version === "hone-sec-margin-trend-v1"
      && trend.current.metric_id === trend.prior.metric_id
      && trend.current.ratio_kind === trend.prior.ratio_kind
      && trend.current.numerator_metric_basis === trend.prior.numerator_metric_basis
      && trend.current.denominator_metric_basis === trend.prior.denominator_metric_basis
      && trend.current.source_url !== trend.prior.source_url
      && Number.isFinite(trend.current.result_percent)
      && Number.isFinite(trend.prior.result_percent)
      && Number.isFinite(trend.change_percentage_points)
      && Math.abs(trend.change_percentage_points - (trend.current.result_percent - trend.prior.result_percent)) <= 0.000001
      && trend.current.formula_version === "hone-sec-same-filing-ratio-v1"
      && trend.prior.formula_version === "hone-sec-same-filing-ratio-v1"
      && trend.current.source_url.startsWith("https://")
      && trend.prior.source_url.startsWith("https://");
  }
  if (observation.ratio) {
    return observation.ratio.formula_version === "hone-sec-same-filing-ratio-v1"
      && observation.ratio.denominator_metric_id === "revenue"
      && observation.ratio.numerator_claim_id !== observation.ratio.denominator_claim_id
      && Boolean(observation.ratio.numerator_metric_basis)
      && Boolean(observation.ratio.denominator_metric_basis)
      && Number.isFinite(observation.ratio.numerator_numeric_value)
      && Number.isFinite(observation.ratio.denominator_numeric_value)
      && observation.ratio.denominator_numeric_value !== 0
      && Number.isFinite(observation.ratio.result_percent)
      && observation.ratio.source_url.startsWith("https://");
  }
  if (observation.computed) {
    return observation.computed.formula_version === "hone-sec-period-comparison-v1"
      && Boolean(observation.computed.metric_basis)
      && observation.computed.current_claim_id !== observation.computed.prior_claim_id
      && Number.isFinite(observation.computed.current_numeric_value)
      && Number.isFinite(observation.computed.prior_numeric_value)
      && observation.computed.prior_numeric_value !== 0
      && Number.isFinite(observation.computed.change_percent)
      && observation.computed.current_source_url.startsWith("https://")
      && observation.computed.prior_source_url.startsWith("https://");
  }
  return !observation.claim || (
    observation.claim.lifecycle_status === "active"
    && Boolean(observation.claim.source_event_id)
    && Boolean(observation.claim.metric_basis)
    && observation.claim.metric_basis !== "unspecified_legacy"
  );
}

export function causalSourceVerificationPrompt(
  observation: InvestmentCausalObservation,
) {
  const qualitative = Boolean(
    observation.claim && observation.claim.numeric_value == null,
  ) || Boolean(
    observation.operating_kpi && observation.operating_kpi.numeric_value == null,
  );
  return qualitative
    ? "打开原始来源后，这段原话的主体、时间和上下文是否一致？"
    : "打开原始来源后，这条数值、期间、单位和上下文是否一致？";
}

function percent(value?: number) {
  return value === undefined || value === null ? "—" : `${value.toFixed(1)}%`;
}

function dateTime(value: string) {
  return new Date(value).toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function latestOutcome(sample: InvestmentDecisionTrainingSample, horizon: number) {
  return sample.outcomes.find(
    (outcome) => outcome.horizon_market_sessions === horizon,
  );
}

export function PublicAdminDecisionBrainPanel() {
  const [evaluation, setEvaluation] =
    createSignal<InvestmentDecisionEvaluation | null>(null);
  const [reviewQueue, setReviewQueue] =
    createSignal<InvestmentEvidenceReviewQueue | null>(null);
  const [replay, setReplay] = createSignal<InvestmentDecisionReplay | null>(null);
  const [rewardGovernance, setRewardGovernance] =
    createSignal<InvestmentRewardGovernance | null>(null);
  const [shadowGovernance, setShadowGovernance] =
    createSignal<InvestmentShadowProtocolGovernance | null>(null);
  const [shadowImplementations, setShadowImplementations] =
    createSignal<InvestmentShadowImplementationRegistry | null>(null);
  const [datasetGovernance, setDatasetGovernance] =
    createSignal<InvestmentCausalDatasetGovernance | null>(null);
  const [trainingExperiments, setTrainingExperiments] =
    createSignal<InvestmentCausalTrainingExperimentRegistry | null>(null);
  const [symbol, setSymbol] = createSignal("SNDK");
  const [selectedId, setSelectedId] = createSignal<string>();
  const [loading, setLoading] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);
  const [causalSubmitting, setCausalSubmitting] = createSignal("");
  const [governanceSubmitting, setGovernanceSubmitting] = createSignal(false);
  const [governanceRationale, setGovernanceRationale] = createSignal("");
  const [governanceConfirmed, setGovernanceConfirmed] = createSignal(false);
  const [shadowGovernanceSubmitting, setShadowGovernanceSubmitting] = createSignal(false);
  const [shadowGovernanceRationale, setShadowGovernanceRationale] = createSignal("");
  const [shadowGovernanceConfirmed, setShadowGovernanceConfirmed] = createSignal(false);
  const [shadowImplementationSubmitting, setShadowImplementationSubmitting] = createSignal(false);
  const [shadowImplementationName, setShadowImplementationName] = createSignal("");
  const [shadowImplementationRevision, setShadowImplementationRevision] = createSignal("");
  const [datasetGovernanceSubmitting, setDatasetGovernanceSubmitting] = createSignal(false);
  const [datasetGovernanceRationale, setDatasetGovernanceRationale] = createSignal("");
  const [datasetGovernanceConfirmed, setDatasetGovernanceConfirmed] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");
  const [queueStatus, setQueueStatus] =
    createSignal<"all" | "pending" | "accepted" | "rejected">("pending");
  const [queueKind, setQueueKind] =
    createSignal<"all" | "source_claim" | "operating_kpi" | "computed_comparison" | "computed_ratio">("all");
  const [queueSelection, setQueueSelection] =
    createSignal<"source_batch" | "old_wang_batch" | "full_queue">("source_batch");
  const [mode, setMode] =
    createSignal<DecisionReviewDraft["mode"]>("accepted");
  const [verdict, setVerdict] =
    createSignal<Exclude<InvestmentThesisVerdict, "pending">>("supported");
  const [note, setNote] = createSignal("");
  const [correctedAction, setCorrectedAction] =
    createSignal<InvestmentExposureAction>("maintain");
  const [errorKind, setErrorKind] =
    createSignal<InvestmentDecisionErrorKind>("valuation");
  const [errorSeverity, setErrorSeverity] =
    createSignal<"minor" | "material" | "critical">("material");
  const [errorExplanation, setErrorExplanation] = createSignal("");
  const [causalReviewDrafts, setCausalReviewDrafts] = createSignal<
    Record<string, CausalReviewDraft>
  >({});
  const [focusedCausalKey, setFocusedCausalKey] = createSignal("");
  const controller = new AbortController();

  const selectedSample = createMemo(() =>
    replay()?.samples.find((sample) => sample.sample_id === selectedId()),
  );

  const causalKey = (driverId: string, observationId: string) =>
    `${driverId}\u0000${observationId}`;

  const selectedCausalDrivers = createMemo(() => {
    const model = selectedSample()?.state.first_principles;
    const drivers = model
      ? [
          ...model.demand_drivers,
          ...model.supply_drivers,
          ...model.value_capture_drivers,
        ].filter((driver) => driver.observations.length > 0)
      : [];
    const first = drivers[0]?.observations[0];
    const focused = focusedCausalKey()
      || (drivers[0] && first ? causalKey(drivers[0].driver_id, first.observation_id) : "");
    return drivers
      .map((driver) => ({
        ...driver,
        observations: driver.observations.filter(
          (observation) => causalKey(driver.driver_id, observation.observation_id) === focused,
        ),
      }))
      .filter((driver) => driver.observations.length > 0);
  });

  createEffect(() => {
    const next: Record<string, CausalReviewDraft> = {};
    for (const review of selectedSample()?.human_review.causal_source_reviews ?? []) {
      next[causalKey(review.driver_id, review.observation_id)] = {
        verdict: "",
        effect: "unclassified",
        explanation: "",
        verbatimJudgment: "",
        applicabilityBoundary: "",
        falsifier: "",
        speakerConfirmation: "",
        sourceVerification: review.verdict,
        sourceVerificationNote: review.note,
        oldWangConfirmationAttested: false,
        stage: review.verdict === "verified_against_source" ? "verbatim" : "source",
        sourceReviewId: review.review_id,
      };
    }
    for (const review of selectedSample()?.human_review.causal_link_reviews ?? []) {
      const existing = next[causalKey(review.driver_id, review.observation_id)];
      next[causalKey(review.driver_id, review.observation_id)] = {
        verdict: review.verdict,
        effect: review.effect ?? "unclassified",
        explanation: review.explanation,
        verbatimJudgment: review.verbatim_judgment ?? "",
        applicabilityBoundary: review.applicability_boundary ?? "",
        falsifier: review.falsifier ?? "",
        speakerConfirmation: review.speaker_confirmation === "old_wang_confirmed"
          || review.speaker_confirmation === "old_wang_confirmed_after_source_check"
          ? "old_wang_confirmed"
          : "",
        sourceVerification: existing?.sourceVerification ?? (review.speaker_confirmation === "evidence_mismatch"
          ? "evidence_mismatch"
          : review.speaker_confirmation === "insufficient_source_context"
            ? "insufficient_source_context"
            : review.speaker_confirmation === "source_checked_not_speaker_confirmed"
                || review.speaker_confirmation === "old_wang_confirmed_after_source_check"
              ? "verified_against_source"
              : ""),
        sourceVerificationNote: existing?.sourceVerificationNote ?? "",
        oldWangConfirmationAttested: false,
        stage: "source",
        reviewId: review.review_id,
        sourceReviewId: existing?.sourceReviewId,
      };
    }
    setCausalReviewDrafts(next);
  });

  const updateCausalReview = (
    driverId: string,
    observationId: string,
    patch: Partial<CausalReviewDraft>,
  ) => {
    const key = causalKey(driverId, observationId);
    setCausalReviewDrafts((current) => ({
      ...current,
      [key]: {
        verdict: current[key]?.verdict ?? "",
        effect: current[key]?.effect ?? "unclassified",
        explanation: current[key]?.explanation ?? "",
        verbatimJudgment: current[key]?.verbatimJudgment ?? "",
        applicabilityBoundary: current[key]?.applicabilityBoundary ?? "",
        falsifier: current[key]?.falsifier ?? "",
        speakerConfirmation: current[key]?.speakerConfirmation ?? "",
        sourceVerification: current[key]?.sourceVerification ?? "",
        sourceVerificationNote: current[key]?.sourceVerificationNote ?? "",
        oldWangConfirmationAttested: current[key]?.oldWangConfirmationAttested ?? false,
        stage: current[key]?.stage ?? "source",
        reviewId: current[key]?.reviewId,
        sourceReviewId: current[key]?.sourceReviewId,
        ...patch,
      },
    }));
  };

  const draft = createMemo<DecisionReviewDraft>(() => ({
    mode: mode(),
    verdict: verdict(),
    note: note(),
    correctedAction: correctedAction(),
    errorKind: errorKind(),
    errorSeverity: errorSeverity(),
    errorExplanation: errorExplanation(),
  }));

  const loadEvaluation = async () => {
    setEvaluation(await getInvestmentDecisionEvaluation(undefined, controller.signal));
  };

  const loadRewardGovernance = async () => {
    setRewardGovernance(await getInvestmentRewardGovernance(controller.signal));
  };

  const loadShadowGovernance = async () => {
    setShadowGovernance(await getInvestmentShadowProtocolGovernance(controller.signal));
  };

  const loadShadowImplementations = async () => {
    setShadowImplementations(await getInvestmentShadowImplementations(controller.signal));
  };

  const loadDatasetGovernance = async () => {
    setDatasetGovernance(await getInvestmentCausalDatasetGovernance(controller.signal));
  };

  const loadTrainingExperiments = async () => {
    setTrainingExperiments(await getInvestmentCausalTrainingExperiments(controller.signal));
  };

  const loadReviewQueue = async (
    status = queueStatus(),
    kind = queueKind(),
    selection = queueSelection(),
  ) => {
    const isBatch = selection !== "full_queue";
    setReviewQueue(await getInvestmentEvidenceReviewQueue({
      status: isBatch ? "pending" : status,
      kind: isBatch ? "all" : kind,
      selection,
      limit: isBatch ? 5 : 100,
    }, controller.signal));
  };

  const loadReplay = async (requestedSymbol?: string, requestedSampleId?: string) => {
    const normalized = (requestedSymbol ?? symbol()).trim().toUpperCase();
    if (!/^[A-Z0-9.-]{1,16}$/.test(normalized)) {
      setError("请输入有效的美股代码，例如 SNDK、NVDA 或 BRK.B。");
      return false;
    }
    setLoading(true);
    setError("");
    setNotice("");
    try {
      const next = await getInvestmentDecisionReplay(
        normalized,
        requestedSampleId ? 500 : 100,
        controller.signal,
      );
      setReplay(next);
      setSelectedId(
        (requestedSampleId && next.samples.some((sample) => sample.sample_id === requestedSampleId)
          ? requestedSampleId
          : [...next.samples]
          .reverse()
          .find((sample) => sample.human_review.status === "pending")?.sample_id ??
          next.samples.at(-1)?.sample_id),
      );
      return true;
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "读取决策轨迹失败");
      return false;
    } finally {
      setLoading(false);
    }
  };

  const openReviewQueueItem = async (item: InvestmentEvidenceReviewQueueItem) => {
    setSymbol(item.symbol);
    const key = causalKey(item.driver_id, item.observation.observation_id);
    setFocusedCausalKey(key);
    if (!await loadReplay(item.symbol, item.sample_id)) return;
    updateCausalReview(item.driver_id, item.observation.observation_id, {
      sourceVerification: (item.review_source_verification ?? "") as CausalSourceVerification,
      sourceVerificationNote: item.source_review_note ?? "",
      sourceReviewId: item.source_review_id ?? undefined,
      stage: item.review_source_verification === "verified_against_source" ? "verbatim" : "source",
      oldWangConfirmationAttested: false,
    });
  };

  const submitReview = async () => {
    const sample = selectedSample();
    if (!sample || !decisionReviewDraftIsValid(draft())) return;
    setSubmitting(true);
    setError("");
    setNotice("");
    try {
      const record = await reviewInvestmentDecision(
        sample.state.symbol,
        sample.sample_id,
        buildDecisionReviewRequest(sample, draft()),
      );
      setReplay((current) =>
        current
          ? {
              ...current,
              samples: current.samples.map((item) =>
                item.sample_id === sample.sample_id
                  ? { ...item, human_review: record.review }
                  : item,
              ),
            }
          : current,
      );
      setNotice("复核已保存，并写入不可覆盖的审计记录。");
      await Promise.all([loadEvaluation(), loadReviewQueue()]);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存复核失败");
    } finally {
      setSubmitting(false);
    }
  };

  const submitCausalReview = async (
    driverId: string,
    observationId: string,
  ) => {
    if (!reviewQueue()?.old_wang_submission_authorized) {
      setError("当前账号可以核验来源，但不是服务器配置的老王审阅账号，不能提交老王因果判断。");
      return;
    }
    const sample = selectedSample();
    const key = causalKey(driverId, observationId);
    const value = causalReviewDrafts()[key];
    if (
      !sample
      || value?.sourceVerification !== "verified_against_source"
      || !value.sourceReviewId
      || !value.sourceVerificationNote.trim()
    ) return;
    if (
      !value.verdict
      || !value.explanation.trim()
      || !value.verbatimJudgment.trim()
      || !value.applicabilityBoundary.trim()
      || !value.falsifier.trim()
      || !value.speakerConfirmation
      || (value.speakerConfirmation === "old_wang_confirmed" && !value.oldWangConfirmationAttested)
      || (value.verdict === "accepted" && value.effect === "unclassified")
    ) return;
    setCausalSubmitting(key);
    setError("");
    setNotice("");
    try {
      await reviewInvestmentCausalEvidence(sample.state.symbol, sample.sample_id, {
        expected_review_id: value.reviewId,
        expected_source_review_id: value.sourceReviewId,
        driver_id: driverId,
        observation_id: observationId,
        verdict: value.verdict as "accepted" | "rejected",
        effect: value.verdict === "rejected" ? "unclassified" : value.effect,
        explanation: value.explanation.trim(),
        verbatim_judgment: value.verbatimJudgment.trim(),
        applicability_boundary: value.applicabilityBoundary.trim(),
        falsifier: value.falsifier.trim(),
        speaker_confirmation: value.speakerConfirmation as Exclude<CausalSpeakerConfirmation, "">,
        source_verification: value.sourceVerification,
        source_verification_note: value.sourceVerificationNote.trim(),
        old_wang_confirmation_attested: value.oldWangConfirmationAttested,
      });
      await Promise.all([
        loadReplay(sample.state.symbol, sample.sample_id),
        loadEvaluation(),
        loadReviewQueue(),
      ]);
      setFocusedCausalKey("");
      setNotice("单条老王因果判断已保存；它引用独立的来源核验记录，原话、结构化标签、边界和反证分别留痕，整份公司判断与行动状态没有改变。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存因果证据复核失败");
    } finally {
      setCausalSubmitting("");
    }
  };

  const submitCausalSourceReview = async (
    driverId: string,
    observationId: string,
  ) => {
    const sample = selectedSample();
    const key = causalKey(driverId, observationId);
    const value = causalReviewDrafts()[key];
    if (!sample || !value?.sourceVerification || !value.sourceVerificationNote.trim()) return;
    setCausalSubmitting(key);
    setError("");
    setNotice("");
    try {
      const record = await reviewInvestmentCausalSource(sample.state.symbol, sample.sample_id, {
        expected_review_id: value.sourceReviewId,
        driver_id: driverId,
        observation_id: observationId,
        verdict: value.sourceVerification,
        note: value.sourceVerificationNote.trim(),
      });
      updateCausalReview(driverId, observationId, {
        sourceReviewId: record.review_id,
        stage: record.verdict === "verified_against_source" ? "verbatim" : "source",
      });
      await loadReviewQueue();
      setNotice(record.verdict === "verified_against_source"
        ? "来源核验已独立保存，尚未生成任何因果或训练标签。现在可交给老王回答这一条。"
        : "来源问题已独立保存并排除出当前单问批次；没有生成因果标签，也没有改变公司判断与行动。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存来源核验失败");
    } finally {
      setCausalSubmitting("");
    }
  };

  const submitRewardGovernance = async (verdict: InvestmentRewardGovernanceVerdict) => {
    const governance = rewardGovernance();
    const design = evaluation()?.reward_design;
    const rationale = governanceRationale().trim();
    if (!governance || !design || !rationale) return;
    const approval = verdict === "approved_for_offline_research";
    if (approval && !governanceConfirmed()) return;
    setGovernanceSubmitting(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewInvestmentRewardGovernance({
        expected_review_id: governance.latest_review?.review_id,
        design_version: governance.design_version,
        proposal_sha256: governance.proposal_sha256,
        verdict,
        rationale,
        component_weights: approval
          ? design.proposed_components.map((component) => ({
              component_id: component.component_id,
              weight_percent: component.proposed_weight_percent,
            }))
          : undefined,
        confirmed_hard_gate_ids: approval
          ? design.hard_gates.map((gate) => gate.gate_id)
          : undefined,
        counterfactual_protocol_confirmed: approval,
      });
      setRewardGovernance(next);
      setGovernanceRationale("");
      setGovernanceConfirmed(false);
      setNotice("奖励目标意见已写入不可覆盖的治理记录；奖励、影子组合和交易仍保持关闭。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存奖励治理意见失败");
    } finally {
      setGovernanceSubmitting(false);
    }
  };

  const submitShadowGovernance = async (
    verdict: InvestmentShadowProtocolGovernanceVerdict,
  ) => {
    const governance = shadowGovernance();
    const rationale = shadowGovernanceRationale().trim();
    if (!governance || !rationale) return;
    const approval = verdict === "approved_for_future_shadow_implementation";
    if (approval && !shadowGovernanceConfirmed()) return;
    setShadowGovernanceSubmitting(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewInvestmentShadowProtocolGovernance({
        expected_review_id: governance.latest_review?.review_id,
        expected_reward_review_id: governance.reward_review_id,
        policy_version: governance.policy_version,
        protocol_sha256: governance.protocol_sha256,
        verdict,
        rationale,
        confirmed_requirement_ids: approval
          ? governance.review_requirements.map((item) => item.requirement_id)
          : undefined,
        implementation_boundary_confirmed: approval,
      });
      setShadowGovernance(next);
      setShadowGovernanceRationale("");
      setShadowGovernanceConfirmed(false);
      setNotice("影子协议意见已写入不可覆盖记录；账本、持仓模拟、订单、券商与交易仍全部关闭。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存影子协议治理意见失败");
    } finally {
      setShadowGovernanceSubmitting(false);
    }
  };

  const submitShadowImplementation = async () => {
    const registry = shadowImplementations();
    const implementationName = shadowImplementationName().trim();
    const codeRevision = shadowImplementationRevision().trim();
    if (
      !registry
      || !registry.registration_allowed
      || !registry.current_shadow_review_id
      || !registry.current_reward_review_id
      || !implementationName
      || !codeRevision
    ) return;
    setShadowImplementationSubmitting(true);
    setError("");
    setNotice("");
    try {
      const next = await registerInvestmentShadowImplementation({
        expected_shadow_review_id: registry.current_shadow_review_id,
        expected_reward_review_id: registry.current_reward_review_id,
        policy_version: registry.policy_version,
        protocol_sha256: registry.protocol_sha256,
        implementation_name: implementationName,
        implementation_kind: "deterministic_replay_specification",
        code_revision: codeRevision,
      });
      setShadowImplementations(next);
      setShadowImplementationName("");
      setShadowImplementationRevision("");
      setNotice("影子实现规范已登记，但没有创建账本、运行模拟、生成订单或连接券商。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "登记影子实现规范失败");
    } finally {
      setShadowImplementationSubmitting(false);
    }
  };

  const submitDatasetGovernance = async (
    verdict: InvestmentCausalDatasetGovernanceVerdict,
  ) => {
    const governance = datasetGovernance();
    const rationale = datasetGovernanceRationale().trim();
    if (!governance || !rationale) return;
    const approval = verdict === "approved_for_offline_experiment";
    if (approval && !datasetGovernanceConfirmed()) return;
    setDatasetGovernanceSubmitting(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewInvestmentCausalDatasetGovernance({
        expected_review_id: governance.latest_review?.review_id,
        dataset_policy_version: governance.dataset.policy_version,
        dataset_fingerprint_sha256: governance.dataset.dataset_fingerprint_sha256,
        verdict,
        rationale,
        company_split_isolation_confirmed: approval,
        source_group_split_isolation_confirmed: approval,
        holdout_seal_confirmed: approval,
        future_leakage_audit_confirmed: approval,
      });
      setDatasetGovernance(next);
      setDatasetGovernanceRationale("");
      setDatasetGovernanceConfirmed(false);
      setNotice("数据集意见已写入不可覆盖记录；批准也只开放离线实验登记，训练、RL 与交易仍关闭。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存数据集治理意见失败");
    } finally {
      setDatasetGovernanceSubmitting(false);
    }
  };

  onMount(() => {
    void (async () => {
      setLoading(true);
      try {
        await Promise.all([
          loadEvaluation(),
          loadRewardGovernance(),
          loadShadowGovernance(),
          loadShadowImplementations(),
          loadDatasetGovernance(),
          loadTrainingExperiments(),
          loadReviewQueue(),
          loadReplay(),
        ]);
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : "读取决策大脑数据失败");
      } finally {
        setLoading(false);
      }
    })();
  });
  onCleanup(() => controller.abort());

  return (
    <section
      class="public-workspace-panel public-admin-panel public-admin-decision-brain"
      aria-labelledby="public-admin-decision-brain-title"
    >
      <div class="public-admin-decision-heading">
        <div>
          <span class="public-admin-decision-eyebrow">仅管理员 · 决策训练</span>
          <h2 id="public-admin-decision-brain-title">HONE 决策大脑复盘</h2>
          <p>复核当时判断，等待真实结果，定位错误。这里不能下单，也没有启用 RL 奖励。</p>
        </div>
        <span class="public-admin-decision-reward">奖励关闭</span>
      </div>

      <PublicAdminFinancialEvidenceReview />
      <PublicAdminValuationInputReview />
      <PublicAdminHistoricalAnchorPanel />
      <PublicAdminHistoricalStateReconstructionPanel />
      <PublicAdminHistoricalOutcomeGovernancePanel />

      <Show when={evaluation()}>
        {(report) => (
          <>
            <div class="public-admin-decision-metrics">
              <div><span>决策样本</span><strong>{report().sample_count}</strong></div>
              <div><span>人工复核率</span><strong>{percent(report().review.review_rate_percent)}</strong></div>
              <div><span>完整长期样本</span><strong>{report().evidence_gate.observed_250_session_samples}/{report().evidence_gate.minimum_250_session_samples}</strong></div>
              <div><span>非重叠长期样本</span><strong>{report().evidence_gate.observed_non_overlapping_250_session_episodes}/{report().evidence_gate.minimum_non_overlapping_250_session_episodes}</strong></div>
              <div><span>因果链接复核</span><strong>{report().causal_review ? percent(report().causal_review?.review_rate_percent) : "尚无"}</strong></div>
              <div><span>经营指标复核</span><strong>{report().operating_kpi_review ? `${report().operating_kpi_review?.reviewed_claims ?? 0}/${report().operating_kpi_review?.available_claims ?? 0}` : "尚无"}</strong></div>
              <div><span>经营指标公司</span><strong>{report().operating_kpi_review?.distinct_symbols ?? 0}</strong></div>
              <div><span>经营指标口径</span><strong>{report().operating_kpi_review?.distinct_definitions ?? 0}</strong></div>
              <div><span>口径冲突/替代</span><strong>{report().operating_kpi_review ? `${report().operating_kpi_review?.conflicted_claims ?? 0}/${report().operating_kpi_review?.superseded_claims ?? 0}` : "0/0"}</strong></div>
              <div><span>同比/环比复核</span><strong>{report().computed_review ? `${report().computed_review?.reviewed_comparisons}/${report().computed_review?.available_comparisons}` : "尚无"}</strong></div>
              <div><span>利润率复核</span><strong>{report().computed_review ? `${report().computed_review?.reviewed_ratios}/${report().computed_review?.available_ratios}` : "尚无"}</strong></div>
              <div><span>利润率趋势复核</span><strong>{report().computed_review ? `${report().computed_review?.reviewed_ratio_trends}/${report().computed_review?.available_ratio_trends}` : "尚无"}</strong></div>
              <div><span>一手财务事实</span><strong>{report().claim_corpus?.claim_count ?? 0}</strong></div>
              <div><span>可追溯同比/环比</span><strong>{report().claim_corpus?.derived_comparison_count ?? 0}</strong></div>
              <div><span>同期利润率</span><strong>{report().claim_corpus?.derived_ratio_count ?? 0}</strong></div>
              <div><span>利润率趋势</span><strong>{report().claim_corpus?.derived_ratio_trend_count ?? 0}</strong></div>
              <div><span>跨期事实公司</span><strong>{report().claim_corpus?.symbols_with_repeated_periods ?? 0}/{report().claim_corpus?.distinct_symbols ?? 0}</strong></div>
              <div><span>事实冲突</span><strong>{report().claim_corpus?.conflicted_claims ?? 0}</strong></div>
              <div><span>覆盖公司</span><strong>{report().evidence_gate.observed_distinct_symbols}/{report().evidence_gate.minimum_distinct_symbols}</strong></div>
              <div><span>覆盖市场季度</span><strong>{report().evidence_gate.observed_decision_quarters}/{report().evidence_gate.minimum_decision_quarters}</strong></div>
              <div><span>进入奖励设计</span><strong>{report().evidence_gate.status === "eligible_for_reward_design_review" ? "可审查" : "证据不足"}</strong></div>
            </div>
            <div class="public-admin-decision-gate" classList={{ "is-ready": report().evidence_gate.status === "eligible_for_reward_design_review" }}>
              <strong>{report().evidence_gate.scope}</strong>
              <For each={report().evidence_gate.reasons}>{(reason) => <span>{reason}</span>}</For>
            </div>
            <section class="public-admin-decision-direction" aria-label="Hari 已确认逻辑情景基准">
              <h3>Hari 已确认逻辑情景基准</h3>
              <p>{report().hari_logic_scenario_benchmark.scope}</p>
              <div>
                <span><strong>一致性</strong> {report().hari_logic_scenario_benchmark.passed_scenario_count}/{report().hari_logic_scenario_benchmark.scenario_count}</span>
                <span><strong>结果</strong> {report().hari_logic_scenario_benchmark.all_passed ? "全部通过" : "存在失败"}</span>
                <span><strong>训练标签</strong> 未生成</span>
                <span><strong>决策/组合/交易</strong> 均未授权</span>
              </div>
              <div class="public-admin-decision-model-map">
                <For each={report().hari_logic_scenario_benchmark.cases}>
                  {(item) => (
                    <article>
                      <header>
                        <strong>{item.label}</strong>
                        <span>{item.passed ? "通过" : "失败"}</span>
                      </header>
                      <p>{item.covered_logic_ids.join(" · ")}</p>
                      <div>
                        <span>预期增加候选 {item.expected_company_increase_authorized ? "是" : "否"}</span>
                        <span>实际增加候选 {item.actual_company_increase_authorized ? "是" : "否"}</span>
                      </div>
                      <small>
                        预期阻断：{item.expected_blocking_logic_ids.length > 0 ? item.expected_blocking_logic_ids.join("、") : "无"}；
                        实际阻断：{item.actual_blocking_logic_ids.length > 0 ? item.actual_blocking_logic_ids.join("、") : "无"}
                      </small>
                      <small>组合层保持独立：{item.actual_portfolio_delegated ? "是" : "否"}</small>
                      <For each={item.failure_reasons}>{(reason) => <small>{reason}</small>}</For>
                    </article>
                  )}
                </For>
              </div>
              <small>这是固定合成边界测试；“全部通过”只代表实现与已确认逻辑一致，不代表策略有效、能够赚钱或可以操盘。</small>
            </section>
            <Show when={report().empirical_validation_readiness}>
              {(readiness) => (
                <section class="public-admin-decision-direction" aria-label="实证验证晋级清单">
                  <h3>实证验证晋级清单</h3>
                  <p>{readiness().scope}</p>
                  <div>
                    <span><strong>总状态</strong> {readiness().empirical_validation_ready ? "可进入实证验证" : "尚被阻断"}</span>
                    <span><strong>训练</strong> 未授权</span>
                    <span><strong>奖励</strong> 未授权</span>
                    <span><strong>影子/交易</strong> 均未授权</span>
                  </div>
                  <div class="public-admin-decision-model-map">
                    <article>
                      <header><strong>① 人工因果数据集</strong><span>{readiness().causal_dataset_governance_review_ready ? "可送治理复核" : "证据不足"}</span></header>
                      <p>{readiness().causal_dataset_governance_review_ready ? "人工标签、公司、驱动与隔离门槛已齐" : "人工标签或覆盖门槛尚未齐全"}</p>
                      <div>
                        <span>有效标签 {readiness().causal_eligible_example_count}</span>
                        <span>公司 {readiness().causal_distinct_symbols}</span>
                        <span>驱动 {readiness().causal_distinct_drivers}</span>
                      </div>
                    </article>
                    <article>
                      <header><strong>② 历史点时基准</strong><span>{readiness().benchmark_state_ready_count > 0 ? "已有可用状态" : "尚无可用状态"}</span></header>
                      <p>{readiness().benchmark_state_ready_count > 0 ? "已有人工批准且仍绑定当前锚点的七层状态" : readiness().confirmed_historical_anchor_count > 0 ? "等待七层点时重建与人工复核" : "等待老王确认至少一条历史判断锚点"}</p>
                      <div>
                        <span>确认锚点 {readiness().confirmed_historical_anchor_count}</span>
                        <span>重建候选 {readiness().reconstruction_candidate_count}</span>
                        <span>批准状态 {readiness().benchmark_state_ready_count}</span>
                        <span>失效重建 {readiness().stale_reconstruction_count}</span>
                      </div>
                    </article>
                    <article>
                      <header><strong>③ 未来结果协议</strong><span>{readiness().historical_outcome_implementation_review_ready ? "可登记实现评审" : "尚未获准"}</span></header>
                      <p>{readiness().historical_outcome_implementation_review_ready ? "冻结协议已获准进入独立标签器实现评审" : "等待历史基准状态与协议人工复核"}</p>
                      <div>
                        <span>协议 {readiness().historical_outcome_protocol_version}</span>
                        <span>标签生成 {readiness().outcome_label_generation_enabled ? "已开启" : "关闭"}</span>
                      </div>
                      <small>协议指纹：{readiness().historical_outcome_protocol_sha256 ? `${readiness().historical_outcome_protocol_sha256.slice(0, 12)}…` : "不可用"}</small>
                    </article>
                    <article>
                      <header><strong>④ 标签器实现</strong><span>{readiness().historical_outcome_offline_dry_run_authorization_review_eligible ? "可送试运行授权复核" : "尚未通过"}</span></header>
                      <p>{readiness().historical_outcome_labeler_current_binding_count > 0 ? "已登记绑定当前协议的确定性实现，仍需封存行情与独立授权" : "等待不可变实现登记与人工复核"}</p>
                      <div>
                        <span>登记 {readiness().historical_outcome_labeler_implementation_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_labeler_current_binding_count}</span>
                        <span>人工通过 {readiness().historical_outcome_labeler_reviewed_count}</span>
                      </div>
                      <small>实现状态：{readiness().historical_outcome_labeler_review_status}</small>
                    </article>
                    <article>
                      <header><strong>⑤ 封存行情输入</strong><span>{readiness().historical_outcome_fully_covered_snapshot_count > 0 ? "已有完整快照" : "尚未完整"}</span></header>
                      <p>{readiness().historical_outcome_price_snapshot_count > 0 ? "快照已绑定点时状态、标签器、FMP 来源与截止日期" : "等待摄取标的与 SPY 的复权收盘价"}</p>
                      <div>
                        <span>当前快照 {readiness().historical_outcome_price_snapshot_count}</span>
                        <span>完整覆盖 {readiness().historical_outcome_fully_covered_snapshot_count}</span>
                      </div>
                    </article>
                    <article>
                      <header><strong>⑥ 试运行授权</strong><span>{readiness().historical_outcome_dry_run_registration_eligible_count > 0 ? "可登记试运行实现" : "尚未批准"}</span></header>
                      <p>授权与运行是两道门禁；批准也不计算收益、不生成标签。</p>
                      <div>
                        <span>批准快照 {readiness().historical_outcome_dry_run_registration_eligible_count}</span>
                        <span>试运行 {readiness().historical_outcome_offline_dry_run_enabled ? "开启" : "关闭"}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_dry_run_authorization_status}</small>
                    </article>
                    <article>
                      <header><strong>⑦ 试运行实现</strong><span>{readiness().historical_outcome_dry_run_current_binding_count > 0 ? "已登记未运行" : "尚未登记"}</span></header>
                      <p>实现登记与实际运行是两道门禁；当前只冻结代码、输入、输出和权限边界。</p>
                      <div>
                        <span>登记 {readiness().historical_outcome_dry_run_implementation_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_dry_run_current_binding_count}</span>
                        <span>可送运行复核 {readiness().historical_outcome_dry_run_execution_authorization_review_eligible_count}</span>
                      </div>
                      <small>实现状态：{readiness().historical_outcome_dry_run_implementation_status}</small>
                    </article>
                    <article>
                      <header><strong>⑧ 运行授权复核</strong><span>{readiness().historical_outcome_dry_run_runner_registration_eligible_count > 0 ? "可登记未来执行器" : "尚未批准"}</span></header>
                      <p>审批仍不是执行；它只允许未来登记隔离执行器，并继续等待独立实现与运行门禁。</p>
                      <div>
                        <span>已复核 {readiness().historical_outcome_dry_run_execution_authorization_reviewed_count}</span>
                        <span>可登记执行器 {readiness().historical_outcome_dry_run_runner_registration_eligible_count}</span>
                        <span>实际运行 {readiness().historical_outcome_offline_dry_run_enabled ? "开启" : "关闭"}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_dry_run_execution_authorization_status}</small>
                    </article>
                    <article>
                      <header><strong>⑨ 隔离执行器规范</strong><span>{readiness().historical_outcome_dry_run_isolated_runner_current_binding_count > 0 ? "已登记" : "尚未登记"}</span></header>
                      <p>执行器登记与调用严格分离；登记记录没有通用代码入口、环境密钥或外部能力。</p>
                      <div>
                        <span>登记 {readiness().historical_outcome_dry_run_isolated_runner_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_dry_run_isolated_runner_current_binding_count}</span>
                        <span>可送首次执行复核 {readiness().historical_outcome_dry_run_first_execution_authorization_review_eligible_count}</span>
                        <span>实际运行 {readiness().historical_outcome_offline_dry_run_enabled ? "开启" : "关闭"}</span>
                      </div>
                      <small>执行器状态：{readiness().historical_outcome_dry_run_isolated_runner_status}</small>
                    </article>
                    <article>
                      <header><strong>⑩ 首次执行授权</strong><span>{readiness().historical_outcome_dry_run_unexpired_first_execution_authorization_count > 0 ? "一次性授权有效" : "尚未授权"}</span></header>
                      <p>授权与调用严格分离；授权仅在 24 小时内提供一次执行额度，授权复核本身不会调用。</p>
                      <div>
                        <span>已复核 {readiness().historical_outcome_dry_run_first_execution_authorization_reviewed_count}</span>
                        <span>获批 {readiness().historical_outcome_dry_run_one_shot_first_execution_authorized_count}</span>
                        <span>未过期 {readiness().historical_outcome_dry_run_unexpired_first_execution_authorization_count}</span>
                        <span>实际运行 {readiness().historical_outcome_offline_dry_run_enabled ? "开启" : "关闭"}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_dry_run_first_execution_authorization_status}</small>
                    </article>
                    <article>
                      <header><strong>⑪ 单次隔离执行</strong><span>{readiness().historical_outcome_dry_run_untrusted_output_count > 0 ? "已有未验证输出" : readiness().historical_outcome_dry_run_failed_attempt_count > 0 ? "失败且额度已消费" : "尚未执行"}</span></header>
                      <p>先写不可变 claim，再以无网络、无工具、无生产写入能力的有界纯函数回放；结果仍不可信。</p>
                      <div>
                        <span>尝试 {readiness().historical_outcome_dry_run_execution_attempt_count}</span>
                        <span>完成 {readiness().historical_outcome_dry_run_completed_attempt_count}</span>
                        <span>失败 {readiness().historical_outcome_dry_run_failed_attempt_count}</span>
                        <span>未验证输出 {readiness().historical_outcome_dry_run_untrusted_output_count}</span>
                      </div>
                      <small>执行状态：{readiness().historical_outcome_dry_run_execution_attempt_status}</small>
                    </article>
                    <article>
                      <header><strong>⑫ 独立输出校验</strong><span>{readiness().historical_outcome_dry_run_validated_output_count > 0 ? "重算一致" : readiness().historical_outcome_dry_run_failed_output_validation_count > 0 ? "不一致，失败关闭" : "等待独立校验"}</span></header>
                      <p>由不同管理员核对不可变哈希和当前封存快照，再用第二套实现逐位重算；通过也不生成结果标签。</p>
                      <div>
                        <span>待校验 {readiness().historical_outcome_dry_run_output_validation_eligible_count}</span>
                        <span>记录 {readiness().historical_outcome_dry_run_output_validation_count}</span>
                        <span>一致 {readiness().historical_outcome_dry_run_validated_output_count}</span>
                        <span>失败 {readiness().historical_outcome_dry_run_failed_output_validation_count}</span>
                      </div>
                      <small>校验状态：{readiness().historical_outcome_dry_run_output_validation_status}</small>
                    </article>
                    <article>
                      <header><strong>⑬ 结果标签准入</strong><span>{readiness().historical_outcome_label_admitted_output_count > 0 ? "准入未来物化" : readiness().historical_outcome_label_admission_rejected_or_changes_requested_count > 0 ? "要求修订 / 拒绝" : "等待独立复核"}</span></header>
                      <p>独立审阅协议适用性、共同交易日端点、复权与基准口径、未来隔离、缺失和幸存者偏差；批准也不直接写标签。</p>
                      <div>
                        <span>已复核 {readiness().historical_outcome_label_admission_reviewed_output_count}</span>
                        <span>已准入 {readiness().historical_outcome_label_admitted_output_count}</span>
                        <span>修订 / 拒绝 {readiness().historical_outcome_label_admission_rejected_or_changes_requested_count}</span>
                        <span>标签写入 {readiness().outcome_label_generation_enabled ? "开启" : "关闭"}</span>
                      </div>
                      <small>准入状态：{readiness().historical_outcome_label_admission_status}</small>
                    </article>
                    <article>
                      <header><strong>⑭ 标签物化实现</strong><span>{readiness().historical_outcome_label_materialization_current_binding_count > 0 ? "已登记 · 未运行" : "尚未登记"}</span></header>
                      <p>只冻结将精确已验证指标、来源与已知局限逐位封装为原始结果信封的实现；不推断方向、评级、动作、仓位或奖励。</p>
                      <div>
                        <span>登记 {readiness().historical_outcome_label_materialization_implementation_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_label_materialization_current_binding_count}</span>
                        <span>可送运行授权复核 {readiness().historical_outcome_label_materialization_run_authorization_review_eligible_count}</span>
                        <span>标签写入 {readiness().outcome_label_generation_enabled ? "开启" : "关闭"}</span>
                      </div>
                      <small>实现状态：{readiness().historical_outcome_label_materialization_implementation_status}</small>
                    </article>
                    <article>
                      <header><strong>⑮ 物化运行授权复核</strong><span>{readiness().historical_outcome_label_materialization_runner_registration_eligible_count > 0 ? "仅批准登记 runner" : readiness().historical_outcome_label_materialization_run_authorization_reviewed_count > 0 ? "已复核 · 未批准" : "等待独立复核"}</span></header>
                      <p>独立于实现、准入、校验和此前执行链的管理员核对逐位保留、来源局限、隔离与权限边界；批准也不运行或写标签。</p>
                      <div>
                        <span>已复核 {readiness().historical_outcome_label_materialization_run_authorization_reviewed_count}</span>
                        <span>可登记 runner {readiness().historical_outcome_label_materialization_runner_registration_eligible_count}</span>
                        <span>运行 {readiness().outcome_label_generation_enabled ? "开启" : "关闭"}</span>
                        <span>标签写入 {readiness().outcome_label_generation_enabled ? "开启" : "关闭"}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_label_materialization_run_authorization_status}</small>
                    </article>
                    <article>
                      <header><strong>⑯ 物化隔离 runner</strong><span>{readiness().historical_outcome_label_materialization_isolated_runner_current_binding_count > 0 ? "已登记 · 未运行" : "尚未登记"}</span></header>
                      <p>冻结 runner 制品摘要、代码版本、只读输入、create-once 隔离输出和资源上限；没有调用入口，仍需独立首次执行授权复核。</p>
                      <div>
                        <span>登记 {readiness().historical_outcome_label_materialization_isolated_runner_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_label_materialization_isolated_runner_current_binding_count}</span>
                        <span>可送首次执行复核 {readiness().historical_outcome_label_materialization_first_execution_authorization_review_eligible_count}</span>
                        <span>标签写入 {readiness().outcome_label_generation_enabled ? "开启" : "关闭"}</span>
                      </div>
                      <small>runner 状态：{readiness().historical_outcome_label_materialization_isolated_runner_status}</small>
                    </article>
                    <article>
                      <header><strong>⑰ 物化首次执行授权</strong><span>{readiness().historical_outcome_label_materialization_unexpired_first_execution_authorization_count > 0 ? "授权仍在时限内" : readiness().historical_outcome_label_materialization_first_execution_authorization_reviewed_count > 0 ? "已复核 · 无有效额度" : "等待独立复核"}</span></header>
                      <p>独立冻结精确 runner、制品和全部上游绑定；批准仅提供 24 小时内一次调用资格，实际是否消费以第十八阶段不可覆盖的 claim 为准。</p>
                      <div>
                        <span>已复核 {readiness().historical_outcome_label_materialization_first_execution_authorization_reviewed_count}</span>
                        <span>一次性批准 {readiness().historical_outcome_label_materialization_one_shot_first_execution_authorized_count}</span>
                        <span>未过期额度 {readiness().historical_outcome_label_materialization_unexpired_first_execution_authorization_count}</span>
                        <span>标签写入 {readiness().outcome_label_generation_enabled ? "开启" : "关闭"}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_label_materialization_first_execution_authorization_status}</small>
                    </article>
                    <article>
                      <header><strong>⑱ 物化一次性执行</strong><span>{readiness().historical_outcome_label_materialization_untrusted_envelope_count > 0 ? "未信任结果包待校验" : readiness().historical_outcome_label_materialization_failed_attempt_count > 0 ? "执行失败 · 授权已消费" : readiness().historical_outcome_label_materialization_execution_attempt_count > 0 ? "claim 已写入 · 失败关闭" : "尚未执行"}</span></header>
                      <p>先写 create-once claim，再用无环境、无网络、无工具和无生产能力的固定纯函数复制已验证原始指标、来源与局限；结果不是标签。</p>
                      <div>
                        <span>执行 {readiness().historical_outcome_label_materialization_execution_attempt_count}</span>
                        <span>成功 {readiness().historical_outcome_label_materialization_completed_attempt_count}</span>
                        <span>失败 {readiness().historical_outcome_label_materialization_failed_attempt_count}</span>
                        <span>待独立校验 {readiness().historical_outcome_label_materialization_independent_validation_eligible_count}</span>
                      </div>
                      <small>执行状态：{readiness().historical_outcome_label_materialization_execution_attempt_status}</small>
                    </article>
                    <article>
                      <header><strong>⑲ 物化结果独立校验</strong><span>{readiness().historical_outcome_label_materialization_validated_envelope_count > 0 ? "结构、来源与位模式一致" : readiness().historical_outcome_label_materialization_failed_output_validation_count > 0 ? "不一致 · 失败关闭" : "等待独立校验"}</span></header>
                      <p>独立重读未信任结果包和封存上游，核对规范结构、完整来源、已知局限及 20 / 60 / 250 日指标位模式；不复用物化投影代码。</p>
                      <div>
                        <span>待校验 {readiness().historical_outcome_label_materialization_output_validation_eligible_count}</span>
                        <span>记录 {readiness().historical_outcome_label_materialization_output_validation_count}</span>
                        <span>一致 {readiness().historical_outcome_label_materialization_validated_envelope_count}</span>
                        <span>失败 {readiness().historical_outcome_label_materialization_failed_output_validation_count}</span>
                      </div>
                      <small>校验状态：{readiness().historical_outcome_label_materialization_output_validation_status}；通过仍不是正式结果标签。</small>
                    </article>
                    <article>
                      <header><strong>⑳ 正式标签写入授权复核</strong><span>{readiness().historical_outcome_label_write_authorization_unexpired_count > 0 ? "一次性额度有效 · 尚未写入" : readiness().historical_outcome_label_write_authorization_reviewed_count > 0 ? "已复核 · 无有效额度" : "等待独立复核"}</span></header>
                      <p>独立复核一条精确通过校验的原始结果包及固定标签合同；批准最多授予 24 小时内一次未来 create-once 写入资格，批准本身不写标签。</p>
                      <div>
                        <span>可复核 {readiness().historical_outcome_label_write_authorization_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_label_write_authorization_reviewed_count}</span>
                        <span>批准 {readiness().historical_outcome_label_write_authorization_one_shot_authorized_count}</span>
                        <span>未过期 {readiness().historical_outcome_label_write_authorization_unexpired_count}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_label_write_authorization_status}；批准不是标签，训练、奖励、影子与交易仍关闭。</small>
                    </article>
                    <article>
                      <header><strong>㉑ 正式原始结果标签写入</strong><span>{readiness().historical_outcome_formal_label_written_count > 0 ? "正式原始标签已写入 · 待独立准入校验" : readiness().historical_outcome_formal_label_failed_write_count > 0 || readiness().historical_outcome_formal_label_incomplete_fail_closed_claim_count > 0 ? "写入失败或中断 · 授权已消费" : readiness().historical_outcome_formal_label_write_eligible_authorization_count > 0 ? "一次性授权可写" : "等待有效授权"}</span></header>
                      <p>先不可变写 claim 并消费授权，再以 create-new 写入仅含原始绝对/相对市场结果、来源、局限和完整链绑定的正式标签；不推断方向、动作、仓位或奖励。</p>
                      <div>
                        <span>可写 {readiness().historical_outcome_formal_label_write_eligible_authorization_count}</span>
                        <span>claim {readiness().historical_outcome_formal_label_write_claim_count}</span>
                        <span>已写 {readiness().historical_outcome_formal_label_written_count}</span>
                        <span>失败/中断 {readiness().historical_outcome_formal_label_failed_write_count + readiness().historical_outcome_formal_label_incomplete_fail_closed_claim_count}</span>
                      </div>
                      <small>写入状态：{readiness().historical_outcome_formal_label_write_status}；标签尚未独立验证或准入训练数据集，训练、奖励、影子与交易仍关闭。</small>
                    </article>
                    <article>
                      <header><strong>㉒ 正式标签独立校验与离线数据集候选准入</strong><span>{readiness().historical_outcome_formal_label_admitted_training_candidate_count > 0 ? "独立校验通过 · 仅进入候选池" : readiness().historical_outcome_formal_label_failed_validation_count > 0 ? "独立校验失败 · 关闭" : readiness().historical_outcome_formal_label_validation_eligible_count > 0 ? "等待独立校验" : "等待正式标签"}</span></header>
                      <p>排除 writer 和完整上游参与者，独立重开授权与来源链，核对 canonical 哈希、固定八字段、来源、局限及 20 / 60 / 250 日指标位模式；通过记录本身只是一条隔离候选准入凭证。</p>
                      <div>
                        <span>待校验 {readiness().historical_outcome_formal_label_validation_eligible_count}</span>
                        <span>记录 {readiness().historical_outcome_formal_label_validation_count}</span>
                        <span>候选 {readiness().historical_outcome_formal_label_admitted_training_candidate_count}</span>
                        <span>失败 {readiness().historical_outcome_formal_label_failed_validation_count}</span>
                      </div>
                      <small>校验状态：{readiness().historical_outcome_formal_label_validation_status}；候选≠训练，训练存储、数据集版本、训练运行、奖励、影子与交易仍关闭。</small>
                    </article>
                    <article>
                      <header><strong>㉓ 版本化离线历史结果数据集装配</strong><span>{readiness().historical_outcome_offline_dataset_current_binding_count > 0 ? "当前完整候选集已冻结 · 待独立数据集治理" : readiness().historical_outcome_offline_dataset_assembly_eligible_count > 0 ? "可装配当前完整候选集" : "等待通过候选"}</span></header>
                      <p>将当前全部独立校验通过的原始结果候选复制到隔离存储，生成内容寻址的不可变版本；后续版本严格保留旧条目并只追加新候选，不允许挑样本、覆写或回填。</p>
                      <div>
                        <span>可装配 {readiness().historical_outcome_offline_dataset_assembly_eligible_count}</span>
                        <span>版本 {readiness().historical_outcome_offline_dataset_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_offline_dataset_current_binding_count}</span>
                        <span>最新条目 {readiness().historical_outcome_offline_dataset_latest_entry_count}</span>
                      </div>
                      <small>装配状态：{readiness().historical_outcome_offline_dataset_assembly_status}；数据集≠训练，特征拼接、语义目标、时间/来源分组切分、训练、奖励、影子与交易仍关闭。</small>
                    </article>
                    <article>
                      <header><strong>㉔ 离线数据集独立治理复核</strong><span>{readiness().historical_outcome_offline_dataset_governance_current_binding_approved_count > 0 ? "仅可登记未来转换规范" : readiness().historical_outcome_offline_dataset_governance_review_eligible_count > 0 ? "等待独立治理复核" : "等待当前绑定数据集"}</span></header>
                      <p>冻结公司/历史事件/来源连通分量隔离、确定性 70/15/15、250 个交易日 purge/embargo、封存留出标签隔离，以及 available_at 不晚于历史判断时点的严格特征连接规则。</p>
                      <div>
                        <span>可复核 {readiness().historical_outcome_offline_dataset_governance_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_offline_dataset_governance_reviewed_count}</span>
                        <span>历史批准 {readiness().historical_outcome_offline_dataset_governance_approved_count}</span>
                        <span>当前批准 {readiness().historical_outcome_offline_dataset_governance_current_binding_approved_count}</span>
                      </div>
                      <small>治理状态：{readiness().historical_outcome_offline_dataset_governance_status}；批准也不执行切分或特征连接，不生成目标，不授权训练、奖励、影子或交易。</small>
                    </article>
                    <article>
                      <header><strong>㉕ 不可变转换规范登记</strong><span>{readiness().historical_outcome_offline_dataset_transformation_spec_current_binding_registered_count > 0 ? "已登记 · 等待独立复核" : readiness().historical_outcome_offline_dataset_transformation_spec_registration_eligible_count > 0 ? "可登记当前治理批准" : "等待治理批准"}</span></header>
                      <p>把确定性连通分量切分 manifest 与产业、公司、财务、估值、拥挤、宏观、组合七层中的 65 个精确 feature ID 固化为内容寻址合同；边界目标可重放，时间歧义和 namespace 改名绕过均失败关闭。</p>
                      <div>
                        <span>可登记 {readiness().historical_outcome_offline_dataset_transformation_spec_registration_eligible_count}</span>
                        <span>历史登记 {readiness().historical_outcome_offline_dataset_transformation_spec_registered_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_offline_dataset_transformation_spec_current_binding_registered_count}</span>
                        <span>待独立复核 {readiness().historical_outcome_offline_dataset_transformation_spec_independent_review_eligible_count}</span>
                      </div>
                      <small>规范状态：{readiness().historical_outcome_offline_dataset_transformation_spec_status}；登记、独立复核和执行严格分开，此阶段不生成 manifest 或 bundle，也不授权训练、奖励、影子或交易。</small>
                    </article>
                    <article>
                      <header><strong>㉖ 转换规范独立复核</strong><span>{readiness().historical_outcome_offline_dataset_transformation_spec_current_binding_approved_count > 0 ? "仅可登记未来隔离实现" : readiness().historical_outcome_offline_dataset_transformation_spec_review_eligible_count > 0 ? "等待独立语义审计" : "等待当前转换规范"}</span></header>
                      <p>由未参与数据集、治理或规范登记链的独立角色，使用另一套语义审计验证精确整数边界、共同交易日 purge/embargo、65 个 feature ID、历史制品版本、缺失语义和来源合同。</p>
                      <div>
                        <span>可复核 {readiness().historical_outcome_offline_dataset_transformation_spec_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_offline_dataset_transformation_spec_reviewed_count}</span>
                        <span>历史批准 {readiness().historical_outcome_offline_dataset_transformation_spec_approved_count}</span>
                        <span>实现登记资格 {readiness().historical_outcome_offline_dataset_transformation_implementation_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_offline_dataset_transformation_spec_review_status}；批准不登记实现、不执行转换、不定义目标，也不授权训练、奖励、影子、订单、券商或交易。</small>
                    </article>
                    <article>
                      <header><strong>㉗ 隔离转换实现规范登记</strong><span>{readiness().historical_outcome_offline_dataset_transformation_implementation_current_binding_count > 0 ? "已登记 · 等待独立实现复核" : readiness().historical_outcome_offline_dataset_transformation_implementation_registration_eligible_count > 0 ? "可登记批准规范" : "等待独立规范批准"}</span></header>
                      <p>冻结实现工件、代码版本、确定性切分和 65 项点时特征算法、规范化序列化、固定输入输出 schema 与资源沙箱；登记记录没有可调用入口、环境继承、密钥、网络、工具或子进程。</p>
                      <div>
                        <span>可登记 {readiness().historical_outcome_offline_dataset_transformation_implementation_registration_eligible_count}</span>
                        <span>历史实现 {readiness().historical_outcome_offline_dataset_transformation_implementation_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_offline_dataset_transformation_implementation_current_binding_count}</span>
                        <span>待独立实现复核 {readiness().historical_outcome_offline_dataset_transformation_implementation_independent_review_eligible_count}</span>
                      </div>
                      <small>实现状态：{readiness().historical_outcome_offline_dataset_transformation_implementation_status}；下一步只有独立实现复核，不运行、不生成 manifest/bundle、不定义目标，也不授权训练、奖励、影子或交易。</small>
                    </article>
                    <article>
                      <header><strong>㉘ 隔离转换实现独立复核</strong><span>{readiness().historical_outcome_offline_dataset_transformation_implementation_current_binding_approved_count > 0 ? "仅可登记未来 runner 规范" : readiness().historical_outcome_offline_dataset_transformation_implementation_review_eligible_count > 0 ? "等待独立工件与沙箱审计" : "等待当前隔离实现"}</span></header>
                      <p>由未参与完整上游和实现登记链的独立角色，重新核对工件摘要、不可变代码版本、切分/特征算法、规范化序列化与固定 schema，并确认单 subject、2048 MiB 的零能力沙箱。</p>
                      <div>
                        <span>可复核 {readiness().historical_outcome_offline_dataset_transformation_implementation_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_offline_dataset_transformation_implementation_reviewed_count}</span>
                        <span>当前批准 {readiness().historical_outcome_offline_dataset_transformation_implementation_current_binding_approved_count}</span>
                        <span>runner 规范登记资格 {readiness().historical_outcome_offline_dataset_transformation_runner_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_offline_dataset_transformation_implementation_review_status}；批准也不登记 runner、不执行、不生成 manifest/bundle、不定义目标，也不授权训练、奖励、影子、订单、券商或交易。</small>
                    </article>
                    <article>
                      <header><strong>㉙ 隔离转换 runner 规范登记</strong><span>{readiness().historical_outcome_offline_dataset_transformation_runner_current_binding_count > 0 ? "已登记未运行 · 等待首次执行复核" : readiness().historical_outcome_offline_dataset_transformation_runner_registration_eligible_count > 0 ? "可登记批准实现" : "等待独立实现批准"}</span></header>
                      <p>把当前批准实现绑定到内容寻址 runner 工件、不可变代码版本、固定零能力运行时、只读输入和 create-once 输出合同；登记记录没有调用入口。</p>
                      <div>
                        <span>登记资格 {readiness().historical_outcome_offline_dataset_transformation_runner_registration_eligible_count}</span>
                        <span>历史 runner {readiness().historical_outcome_offline_dataset_transformation_runner_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_offline_dataset_transformation_runner_current_binding_count}</span>
                        <span>可送首次执行复核 {readiness().historical_outcome_offline_dataset_transformation_runner_first_execution_authorization_review_eligible_count}</span>
                      </div>
                      <small>runner 状态：{readiness().historical_outcome_offline_dataset_transformation_runner_status}；唯一下一门禁是独立首次执行授权复核。当前不执行、不生成输出、manifest/bundle、目标或训练输入。</small>
                    </article>
                    <article>
                      <header><strong>㉚ 隔离转换首次执行授权复核</strong><span>{readiness().historical_outcome_offline_dataset_transformation_execution_attempt_eligible_count > 0 ? "24 小时单次资格有效 · 尚未执行" : readiness().historical_outcome_offline_dataset_transformation_first_execution_authorization_review_eligible_count > 0 ? "等待独立工件复现与授权复核" : "等待 registered_not_run runner"}</span></header>
                      <p>由未参与完整治理链的独立角色重算 runner 工件摘要，复核不可变代码、只读输入、零环境/网络/密钥能力、固定资源上限和 create-once 输出边界；批准只允许未来最多一次隔离调用。</p>
                      <div>
                        <span>可复核 {readiness().historical_outcome_offline_dataset_transformation_first_execution_authorization_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_offline_dataset_transformation_first_execution_authorization_reviewed_count}</span>
                        <span>历史批准 {readiness().historical_outcome_offline_dataset_transformation_first_execution_authorization_approved_count}</span>
                        <span>有效单次资格 {readiness().historical_outcome_offline_dataset_transformation_first_execution_authorization_unexpired_count}</span>
                        <span>可进入执行尝试门禁 {readiness().historical_outcome_offline_dataset_transformation_execution_attempt_eligible_count}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_offline_dataset_transformation_first_execution_authorization_status}；批准仅建立 24 小时单次资格，必须在第 31 阶段人工领取，失败也消费。</small>
                    </article>
                    <article>
                      <header><strong>㉛ 隔离转换一次性执行尝试</strong><span>{readiness().historical_outcome_offline_dataset_transformation_untrusted_candidate_envelope_count > 0 ? "候选待独立校验" : readiness().historical_outcome_offline_dataset_transformation_failed_attempt_count > 0 ? "失败且授权已消费" : readiness().historical_outcome_offline_dataset_transformation_execution_attempt_eligible_count > 0 ? "可人工领取并执行一次" : "等待当前单次授权"}</span></header>
                      <p>claim 前重开完整当前绑定并重算运行制品；claim 后只运行固定纯函数，生成确定性切分和 65 项显式缺失特征候选。成功或失败都消费授权。</p>
                      <div>
                        <span>可执行资格 {readiness().historical_outcome_offline_dataset_transformation_execution_attempt_eligible_count}</span>
                        <span>历史尝试 {readiness().historical_outcome_offline_dataset_transformation_execution_attempt_count}</span>
                        <span>完成 {readiness().historical_outcome_offline_dataset_transformation_completed_attempt_count}</span>
                        <span>失败 {readiness().historical_outcome_offline_dataset_transformation_failed_attempt_count}</span>
                        <span>待独立校验 {readiness().historical_outcome_offline_dataset_transformation_independent_validation_eligible_count}</span>
                      </div>
                      <small>执行状态：{readiness().historical_outcome_offline_dataset_transformation_execution_attempt_status}；候选不是正式 manifest、feature bundle 或训练输入，不授权训练、奖励、影子、订单、券商或交易。</small>
                    </article>
                    <article>
                      <header><strong>㉜ 离线转换输出独立重算</strong><span>{readiness().historical_outcome_offline_dataset_transformation_failed_output_validation_count > 0 ? "重算不一致 · 失败关闭" : readiness().historical_outcome_offline_dataset_transformation_validated_candidate_envelope_count > 0 ? "候选已验证 · 尚非正式工件" : readiness().historical_outcome_offline_dataset_transformation_output_validation_eligible_count > 0 ? "等待独立重算" : "等待未信任候选"}</span></header>
                      <p>由独立角色重新打开完整不可变链，用图遍历而非执行层并查集重算传递连通分量、连续时间边界、250 交易日 purge/embargo 与 65 项显式缺失值。</p>
                      <div>
                        <span>待校验 {readiness().historical_outcome_offline_dataset_transformation_output_validation_eligible_count}</span>
                        <span>校验记录 {readiness().historical_outcome_offline_dataset_transformation_output_validation_count}</span>
                        <span>通过 {readiness().historical_outcome_offline_dataset_transformation_validated_candidate_envelope_count}</span>
                        <span>失败关闭 {readiness().historical_outcome_offline_dataset_transformation_failed_output_validation_count}</span>
                      </div>
                      <small>校验状态：{readiness().historical_outcome_offline_dataset_transformation_output_validation_status}；通过仍不创建正式 manifest、feature bundle 或训练输入，下一步必须另设准入/物化门禁。</small>
                    </article>
                    <article>
                      <header><strong>㉝ 离线转换候选独立准入</strong><span>{readiness().historical_outcome_offline_dataset_transformation_candidate_admitted_count > 0 ? "已准入 · 等待独立正式物化" : readiness().historical_outcome_offline_dataset_transformation_candidate_admission_rejected_or_changes_requested_count > 0 ? "修改或拒绝 · 失败关闭" : readiness().historical_outcome_offline_dataset_transformation_validated_candidate_envelope_count > 0 ? "等待独立准入复核" : "等待独立校验候选"}</span></header>
                      <p>独立管理员复核精确候选的分量隔离、时间边界、purge/embargo、65 项点时特征、显式缺失、来源排除与 create-once 正式产物合同。</p>
                      <div>
                        <span>已复核 {readiness().historical_outcome_offline_dataset_transformation_candidate_admission_reviewed_count}</span>
                        <span>已准入 {readiness().historical_outcome_offline_dataset_transformation_candidate_admitted_count}</span>
                        <span>修改/拒绝 {readiness().historical_outcome_offline_dataset_transformation_candidate_admission_rejected_or_changes_requested_count}</span>
                      </div>
                      <small>准入状态：{readiness().historical_outcome_offline_dataset_transformation_candidate_admission_status}；准入不是正式物化，不创建 manifest/feature bundle，也不开放 join、目标、训练或交易。</small>
                    </article>
                    <article>
                      <header><strong>㉞ 正式 manifest / feature bundle 一次性物化</strong><span>{readiness().historical_outcome_offline_dataset_transformation_official_artifact_materialization_failed_or_incomplete_count > 0 ? "物化失败或中断 · 资格已消费" : readiness().historical_outcome_offline_dataset_transformation_unvalidated_official_artifact_pair_count > 0 ? "已物化 · 等待独立校验" : readiness().historical_outcome_offline_dataset_transformation_candidate_admitted_count > 0 ? "等待一次性物化" : "等待准入候选"}</span></header>
                      <p>先写入不可撤销 claim，再从精确绑定且独立校验通过的候选逐字节物化正式切分清单和正式特征包；不重算、不补数、不改写。</p>
                      <div>
                        <span>已领取 {readiness().historical_outcome_offline_dataset_transformation_official_artifact_materialization_claimed_count}</span>
                        <span>已完成 {readiness().historical_outcome_offline_dataset_transformation_official_artifact_materialization_completed_count}</span>
                        <span>失败/中断 {readiness().historical_outcome_offline_dataset_transformation_official_artifact_materialization_failed_or_incomplete_count}</span>
                        <span>待独立校验 {readiness().historical_outcome_offline_dataset_transformation_unvalidated_official_artifact_pair_count}</span>
                      </div>
                      <small>物化状态：{readiness().historical_outcome_offline_dataset_transformation_official_artifact_materialization_status}；正式物化不是训练准入，join、语义目标、训练、奖励、影子、订单、券商与交易仍全部关闭。</small>
                    </article>
                    <article>
                      <header><strong>㉟ 正式工件物化后独立校验</strong><span>{readiness().historical_outcome_offline_dataset_transformation_failed_official_artifact_output_validation_count > 0 ? "校验不一致 · 失败关闭" : readiness().historical_outcome_offline_dataset_transformation_independently_validated_official_artifact_pair_count > 0 ? "工件已验证 · 等待 join/target 治理" : readiness().historical_outcome_offline_dataset_transformation_official_artifact_output_validation_eligible_count > 0 ? "等待独立校验" : "等待正式工件"}</span></header>
                      <p>由不同角色重新读取 claim、result、正式 manifest、正式 feature bundle 和精确源候选，独立重算五类摘要并逐字段核对。</p>
                      <div>
                        <span>待校验 {readiness().historical_outcome_offline_dataset_transformation_official_artifact_output_validation_eligible_count}</span>
                        <span>校验记录 {readiness().historical_outcome_offline_dataset_transformation_official_artifact_output_validation_count}</span>
                        <span>通过 {readiness().historical_outcome_offline_dataset_transformation_independently_validated_official_artifact_pair_count}</span>
                        <span>失败关闭 {readiness().historical_outcome_offline_dataset_transformation_failed_official_artifact_output_validation_count}</span>
                      </div>
                      <small>校验状态：{readiness().historical_outcome_offline_dataset_transformation_official_artifact_output_validation_status}；通过只开放未来 join/target 治理规范登记，不连接标签、不定义目标、不训练或交易。</small>
                    </article>
                    <article>
                      <header><strong>㊱ 特征—标签连接与连续目标规范</strong><span>{readiness().historical_outcome_feature_label_join_target_stale_or_mismatched_specification_count > 0 ? "绑定漂移 · 失败关闭" : readiness().historical_outcome_feature_label_join_target_independent_review_eligible_count > 0 ? "规范已登记 · 等待独立复核" : readiness().historical_outcome_feature_label_join_target_spec_registration_eligible_count > 0 ? "等待规范登记" : "等待正式工件独立校验"}</span></header>
                      <p>冻结 entry 一对一连接、purge/embargo 排除、点时可用性、sealed holdout 标签隔离和 20/60/250 日连续结果目标向量；不把投资动作或奖励伪装成标签。</p>
                      <div>
                        <span>可登记 {readiness().historical_outcome_feature_label_join_target_spec_registration_eligible_count}</span>
                        <span>规范 {readiness().historical_outcome_feature_label_join_target_specification_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_feature_label_join_target_current_binding_specification_count}</span>
                        <span>待独立复核 {readiness().historical_outcome_feature_label_join_target_independent_review_eligible_count}</span>
                      </div>
                      <small>规范状态：{readiness().historical_outcome_feature_label_join_target_specification_status}；登记不执行 join、不分配目标、不创建训练行，也不训练、奖励、影子或交易。</small>
                    </article>
                    <article>
                      <header><strong>㊲ join/target 规范独立语义与指纹复核</strong><span>{readiness().historical_outcome_feature_label_join_target_spec_current_binding_approved_count > 0 ? "复核通过 · 仅可登记未来实现" : readiness().historical_outcome_feature_label_join_target_spec_review_eligible_count > 0 ? "等待独立复核" : "等待当前规范"}</span></header>
                      <p>由另一角色独立重算 record/body/join/target 指纹、正式工件与 65 项目录绑定、连接防泄漏语义及九维连续目标定义。</p>
                      <div>
                        <span>待复核 {readiness().historical_outcome_feature_label_join_target_spec_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_feature_label_join_target_spec_reviewed_count}</span>
                        <span>已批准 {readiness().historical_outcome_feature_label_join_target_spec_current_binding_approved_count}</span>
                        <span>可登记实现 {readiness().historical_outcome_feature_label_join_target_implementation_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_feature_label_join_target_spec_review_status}；250 日超额收益仍只是工程目标候选，批准不执行 join、不生成训练行，也不证明策略有效或授权交易。</small>
                    </article>
                    <article>
                      <header><strong>㊳ join/target 隔离实现登记</strong><span>{readiness().historical_outcome_feature_label_join_target_implementation_current_binding_count > 0 ? "实现已登记 · 等待独立复核" : readiness().historical_outcome_feature_label_join_target_implementation_registration_eligible_count > 0 ? "等待实现登记" : "等待规范独立批准"}</span></header>
                      <p>冻结不可变实现工件、代码版本、严格一对一 join、九维原始 f64 目标投影、sealed holdout 隔离和零能力沙箱合同。</p>
                      <div>
                        <span>可登记 {readiness().historical_outcome_feature_label_join_target_implementation_registration_eligible_count}</span>
                        <span>历史实现 {readiness().historical_outcome_feature_label_join_target_implementation_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_feature_label_join_target_implementation_current_binding_count}</span>
                        <span>待独立实现复核 {readiness().historical_outcome_feature_label_join_target_implementation_independent_review_eligible_count}</span>
                      </div>
                      <small>实现状态：{readiness().historical_outcome_feature_label_join_target_implementation_status}；登记没有入口、runner、标签访问、join、joined/training rows、训练、奖励、影子或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>㊴ join/target 实现独立复核</strong><span>{readiness().historical_outcome_feature_label_join_target_implementation_current_binding_approved_count > 0 ? "复核通过 · 仅可登记 runner 规格" : readiness().historical_outcome_feature_label_join_target_implementation_review_eligible_count > 0 ? "等待独立实现复核" : "等待当前实现"}</span></header>
                      <p>由另一角色独立重算实现记录与合同指纹，复核一对一 join、九维原始 f64 目标、防泄漏边界和零能力沙箱。</p>
                      <div>
                        <span>待复核 {readiness().historical_outcome_feature_label_join_target_implementation_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_feature_label_join_target_implementation_reviewed_count}</span>
                        <span>当前批准 {readiness().historical_outcome_feature_label_join_target_implementation_current_binding_approved_count}</span>
                        <span>可登记 runner 规格 {readiness().historical_outcome_feature_label_join_target_runner_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_feature_label_join_target_implementation_review_status}；批准仍不创建 runner、不读取标签、不执行 join、不校验输出、不训练或交易，九维目标仍只是工程候选。</small>
                    </article>
                    <article>
                      <header><strong>㊵ join/target 隔离 runner 规格登记</strong><span>{readiness().historical_outcome_feature_label_join_target_isolated_runner_current_binding_count > 0 ? "已登记未运行 · 等待首次执行复核" : readiness().historical_outcome_feature_label_join_target_runner_registration_eligible_count > 0 ? "可登记批准实现" : "等待实现独立批准"}</span></header>
                      <p>冻结 runner 工件、代码版本、固定运行时、精确只读输入、create-once 内容寻址输出和单数据集静态资源上限。</p>
                      <div>
                        <span>登记资格 {readiness().historical_outcome_feature_label_join_target_runner_registration_eligible_count}</span>
                        <span>历史 runner {readiness().historical_outcome_feature_label_join_target_isolated_runner_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_feature_label_join_target_isolated_runner_current_binding_count}</span>
                        <span>可送首次执行复核 {readiness().historical_outcome_feature_label_join_target_first_execution_authorization_review_eligible_count}</span>
                      </div>
                      <small>runner 状态：{readiness().historical_outcome_feature_label_join_target_isolated_runner_status}；没有可调用入口、标签/训练库读取、join、目标分配、joined rows、训练、奖励、影子、订单、券商或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>㊶ join/target 首次执行授权复核</strong><span>{readiness().historical_outcome_feature_label_join_target_execution_attempt_eligible_count > 0 ? "24 小时单次资格有效 · 未执行" : readiness().historical_outcome_feature_label_join_target_first_execution_authorization_review_eligible_count > 0 ? "等待独立首次执行复核" : "等待当前 runner"}</span></header>
                      <p>由新的独立角色复现 runner 工件与完整上游链，核对一对一 join、九项原始 f64 目标、PIT/purge/embargo/split/sealed holdout 和零能力沙箱。</p>
                      <div>
                        <span>待复核 {readiness().historical_outcome_feature_label_join_target_first_execution_authorization_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_feature_label_join_target_first_execution_authorization_reviewed_count}</span>
                        <span>未过期单次资格 {readiness().historical_outcome_feature_label_join_target_unexpired_first_execution_authorization_count}</span>
                        <span>下一门禁候选 {readiness().historical_outcome_feature_label_join_target_execution_attempt_eligible_count}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_feature_label_join_target_first_execution_authorization_status}；批准不是 claim 或执行，当前不读取通用标签库、不 join、不创建 joined/training rows，也不训练、奖励、影子或交易。</small>
                    </article>
                    <article>
                      <header><strong>㊷ join/target 一次性执行尝试</strong><span>{readiness().historical_outcome_feature_label_join_target_independent_output_validation_eligible_count > 0 ? "不可信候选已生成 · 待独立校验" : readiness().historical_outcome_feature_label_join_target_failed_execution_attempt_count > 0 ? "执行失败 · 授权已消费" : readiness().historical_outcome_feature_label_join_target_execution_invocation_eligible_authorization_count > 0 ? "等待领取单次授权" : "等待有效授权"}</span></header>
                      <p>create-once 领取授权后，只对精确 official split、65 项点时特征与当前原始结果做固定一对一投影；train 暴露九项原始 f64 位模式，validation 与 sealed holdout 目标值保持隐藏。</p>
                      <div>
                        <span>可领取 {readiness().historical_outcome_feature_label_join_target_execution_invocation_eligible_authorization_count}</span>
                        <span>尝试 {readiness().historical_outcome_feature_label_join_target_execution_attempt_count}</span>
                        <span>成功候选 {readiness().historical_outcome_feature_label_join_target_untrusted_candidate_envelope_count}</span>
                        <span>待独立校验 {readiness().historical_outcome_feature_label_join_target_independent_output_validation_eligible_count}</span>
                      </div>
                      <small>执行状态：{readiness().historical_outcome_feature_label_join_target_execution_status}；成功或失败都消费授权，候选尚非正式 joined dataset 或训练数据，不训练、奖励、影子、订单、券商或交易。</small>
                    </article>
                    <article>
                      <header><strong>㊸ join/target 独立输出校验</strong><span>{readiness().historical_outcome_feature_label_join_target_candidate_admission_review_eligible_count > 0 ? "独立通过 · 等待候选准入复核" : readiness().historical_outcome_feature_label_join_target_failed_output_validation_count > 0 ? "独立重算失败 · 已关闭" : readiness().historical_outcome_feature_label_join_target_output_validation_eligible_count > 0 ? "等待独立逐行逐位重算" : "等待不可信候选"}</span></header>
                      <p>完整上游链之外的角色重新打开精确 claim/result/output、正式工件与当前原始结果，独立重算一对一连接、65 项特征、九维原始位模式、目标承诺和分区隐藏。</p>
                      <div>
                        <span>待校验 {readiness().historical_outcome_feature_label_join_target_output_validation_eligible_count}</span>
                        <span>校验记录 {readiness().historical_outcome_feature_label_join_target_output_validation_count}</span>
                        <span>独立通过 {readiness().historical_outcome_feature_label_join_target_independently_validated_untrusted_candidate_count}</span>
                        <span>待准入复核 {readiness().historical_outcome_feature_label_join_target_candidate_admission_review_eligible_count}</span>
                      </div>
                      <small>校验状态：{readiness().historical_outcome_feature_label_join_target_output_validation_status}；通过仍只是不可信候选，不创建正式 joined dataset，不复制训练库，不训练、奖励、影子、订单、券商或交易。</small>
                    </article>
                    <article>
                      <header><strong>㊹ join/target 候选独立准入复核</strong><span>{readiness().historical_outcome_feature_label_join_target_future_official_joined_dataset_materialization_eligible_count > 0 ? "已准入 · 等待独立物化" : readiness().historical_outcome_feature_label_join_target_candidate_admission_rejected_or_changes_requested_count > 0 ? "要求修改/拒绝 · 已关闭" : readiness().historical_outcome_feature_label_join_target_candidate_admission_review_eligible_count > 0 ? "等待独立准入复核" : "等待独立校验候选"}</span></header>
                      <p>由 Stage 43 校验者、Stage 42 执行者、完整上游链和此前复核人之外的新角色，绑定精确候选哈希、65 项特征、九项目标承诺、样本边界与 create-once 正式数据集合同。</p>
                      <div>
                        <span>可复核 {readiness().historical_outcome_feature_label_join_target_candidate_admission_review_eligible_count}</span>
                        <span>复核记录 {readiness().historical_outcome_feature_label_join_target_candidate_admission_reviewed_count}</span>
                        <span>已准入 {readiness().historical_outcome_feature_label_join_target_candidate_admitted_count}</span>
                        <span>未来物化资格 {readiness().historical_outcome_feature_label_join_target_future_official_joined_dataset_materialization_eligible_count}</span>
                      </div>
                      <small>准入状态：{readiness().historical_outcome_feature_label_join_target_candidate_admission_status}；批准只开放下一道 create-once 物化门禁，不创建训练数据、不训练、奖励、影子、订单、券商或交易。</small>
                    </article>
                    <article>
                      <header><strong>㊺ 正式 joined dataset 一次性物化</strong><span>{readiness().historical_outcome_feature_label_join_target_official_dataset_pending_independent_validation_count > 0 ? "已物化 · 等待独立校验" : readiness().historical_outcome_feature_label_join_target_official_dataset_materialization_failed_count > 0 ? "物化失败 · claim 已消费" : readiness().historical_outcome_feature_label_join_target_official_dataset_materialization_eligible_count > 0 ? "等待 claim-first 物化" : "等待已准入候选"}</span></header>
                      <p>由完整上游链之外的新角色先不可逆消费 claim，再逐字节复制 Stage 44 精确准入的 rows、排除审计与目标承诺；不允许重算、修补、插补、覆盖或重放。</p>
                      <div>
                        <span>已准入 {readiness().historical_outcome_feature_label_join_target_official_dataset_admitted_candidate_count}</span>
                        <span>可物化 {readiness().historical_outcome_feature_label_join_target_official_dataset_materialization_eligible_count}</span>
                        <span>已完成 {readiness().historical_outcome_feature_label_join_target_official_dataset_materialization_completed_count}</span>
                        <span>待独立校验 {readiness().historical_outcome_feature_label_join_target_official_dataset_pending_independent_validation_count}</span>
                      </div>
                      <small>物化状态：{readiness().historical_outcome_feature_label_join_target_official_dataset_materialization_status}；落盘仍不是训练准入，训练库复制、训练、奖励、影子、订单、券商与交易全部关闭。</small>
                    </article>
                    <article>
                      <header><strong>㊻ 正式 joined dataset 独立输出校验</strong><span>{readiness().historical_outcome_feature_label_join_target_future_training_store_copy_admission_review_eligible_count > 0 ? "独立通过 · 等待复制准入复核" : readiness().historical_outcome_feature_label_join_target_official_dataset_failed_output_validation_count > 0 ? "独立复核失败 · 已关闭" : readiness().historical_outcome_feature_label_join_target_official_dataset_output_validation_eligible_count > 0 ? "等待独立逐行逐位校验" : "等待正式数据集"}</span></header>
                      <p>由物化者和完整上游链之外的新角色自行重开不可变 claim、result 与 official joined dataset，独立重算工件、rows、排除项及目标承诺并核对精确准入候选。</p>
                      <div>
                        <span>待校验 {readiness().historical_outcome_feature_label_join_target_official_dataset_output_validation_eligible_count}</span>
                        <span>校验记录 {readiness().historical_outcome_feature_label_join_target_official_dataset_output_validation_count}</span>
                        <span>独立通过 {readiness().historical_outcome_feature_label_join_target_independently_validated_official_joined_dataset_count}</span>
                        <span>待复制准入复核 {readiness().historical_outcome_feature_label_join_target_future_training_store_copy_admission_review_eligible_count}</span>
                      </div>
                      <small>校验状态：{readiness().historical_outcome_feature_label_join_target_official_dataset_output_validation_status}；独立通过只开放未来训练库复制准入复核，不复制、不训练、奖励、影子、订单、券商或交易。</small>
                    </article>
                    <article>
                      <header><strong>㊼ 训练存储复制独立准入复核</strong><span>{readiness().historical_outcome_feature_label_join_target_future_create_once_training_store_copy_eligible_count > 0 ? "已准入 · 等待独立复制门禁" : readiness().historical_outcome_feature_label_join_target_training_store_copy_admission_rejected_or_changes_requested_count > 0 ? "已退回或拒绝" : readiness().historical_outcome_feature_label_join_target_future_training_store_copy_admission_review_eligible_count > 0 ? "等待独立准入复核" : "等待 Stage 46 独立通过"}</span></header>
                      <p>由 Stage 46 校验者、Stage 45 物化者和完整上游之外的新角色，精确复核不可变正式数据集的数据合同、点时/缺失、切分隔离及九项目标承诺。</p>
                      <div>
                        <span>已复核 {readiness().historical_outcome_feature_label_join_target_training_store_copy_admission_reviewed_count}</span>
                        <span>已准入 {readiness().historical_outcome_feature_label_join_target_training_store_copy_candidate_admitted_count}</span>
                        <span>退回/拒绝 {readiness().historical_outcome_feature_label_join_target_training_store_copy_admission_rejected_or_changes_requested_count}</span>
                        <span>未来可复制 {readiness().historical_outcome_feature_label_join_target_future_create_once_training_store_copy_eligible_count}</span>
                      </div>
                      <small>准入状态：{readiness().historical_outcome_feature_label_join_target_training_store_copy_admission_status}；批准也只开放未来 create-once 复制门禁，不复制、不训练、奖励、影子、订单、券商或交易。</small>
                    </article>
                    <article>
                      <header><strong>㊽ 训练存储一次性复制</strong><span>{readiness().historical_outcome_feature_label_join_target_training_store_copy_pending_independent_validation_count > 0 ? "已复制 · 等待独立复制后校验" : readiness().historical_outcome_feature_label_join_target_training_store_copy_failed_count > 0 ? "复制失败 · claim 已消费" : readiness().historical_outcome_feature_label_join_target_training_store_copy_eligible_count > 0 ? "等待领取一次性复制 claim" : "等待 Stage 47 准入"}</span></header>
                      <p>先写不可变 claim，再把 Stage 47 精确准入的正式 joined dataset 原样复制到隔离训练存储目录；失败或中断同样消费资格，不允许重算、修补、插补、覆盖或重放。</p>
                      <div>
                        <span>已准入 {readiness().historical_outcome_feature_label_join_target_training_store_copy_admitted_dataset_count}</span>
                        <span>可复制 {readiness().historical_outcome_feature_label_join_target_training_store_copy_eligible_count}</span>
                        <span>claim {readiness().historical_outcome_feature_label_join_target_training_store_copy_claim_count}</span>
                        <span>已完成 {readiness().historical_outcome_feature_label_join_target_training_store_copy_completed_count}</span>
                        <span>失败 {readiness().historical_outcome_feature_label_join_target_training_store_copy_failed_count}</span>
                        <span>待复制后校验 {readiness().historical_outcome_feature_label_join_target_training_store_copy_pending_independent_validation_count}</span>
                      </div>
                      <small>复制状态：{readiness().historical_outcome_feature_label_join_target_training_store_copy_status}；复制成功仍不是训练登记或训练授权，奖励、影子、订单、券商和交易继续关闭。</small>
                    </article>
                    <article>
                      <header><strong>㊾ 训练存储副本独立校验</strong><span>{readiness().historical_outcome_feature_label_join_target_future_training_registration_review_eligible_count > 0 ? "独立通过 · 等待训练登记准入复核" : readiness().historical_outcome_feature_label_join_target_training_store_copy_failed_output_validation_count > 0 ? "独立复核失败 · 已关闭" : readiness().historical_outcome_feature_label_join_target_training_store_copy_output_validation_eligible_count > 0 ? "等待独立逐行逐位校验" : "等待 Stage 48 复制"}</span></header>
                      <p>由复制人和完整上游之外的新角色重新打开 Stage 48 claim、result 与副本，独立重算全部指纹，并与 Stage 47 正式数据集逐行、逐位、逐目标承诺核对。</p>
                      <div>
                        <span>待校验 {readiness().historical_outcome_feature_label_join_target_training_store_copy_output_validation_eligible_count}</span>
                        <span>已记录 {readiness().historical_outcome_feature_label_join_target_training_store_copy_output_validation_count}</span>
                        <span>独立通过 {readiness().historical_outcome_feature_label_join_target_independently_validated_training_store_copy_count}</span>
                        <span>失败 {readiness().historical_outcome_feature_label_join_target_training_store_copy_failed_output_validation_count}</span>
                        <span>待训练登记复核 {readiness().historical_outcome_feature_label_join_target_future_training_registration_review_eligible_count}</span>
                      </div>
                      <small>校验状态：{readiness().historical_outcome_feature_label_join_target_training_store_copy_output_validation_status}；通过只证明复制一致，不证明模型有效，也不登记、授权或运行训练。</small>
                    </article>
                    <article>
                      <header><strong>㊿ 训练登记独立准入复核</strong><span>{readiness().historical_outcome_feature_label_join_target_future_create_once_training_registration_eligible_count > 0 ? "已准入 · 等待一次性训练登记" : readiness().historical_outcome_feature_label_join_target_training_registration_admission_rejected_or_changes_requested_count > 0 ? "退回或拒绝 · 已关闭" : readiness().historical_outcome_feature_label_join_target_future_training_registration_review_eligible_count > 0 ? "等待独立准入复核" : "等待 Stage 49 校验"}</span></header>
                      <p>由 Stage 49 校验者、Stage 48 复制者和完整上游之外的新角色，复核副本指纹、逐行逐位一致性、65 项特征、九项目标及目标隐藏边界。</p>
                      <div>
                        <span>待复核 {readiness().historical_outcome_feature_label_join_target_future_training_registration_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_feature_label_join_target_training_registration_admission_reviewed_count}</span>
                        <span>已准入 {readiness().historical_outcome_feature_label_join_target_training_registration_candidate_admitted_count}</span>
                        <span>退回/拒绝 {readiness().historical_outcome_feature_label_join_target_training_registration_admission_rejected_or_changes_requested_count}</span>
                        <span>未来可登记 {readiness().historical_outcome_feature_label_join_target_future_create_once_training_registration_eligible_count}</span>
                      </div>
                      <small>准入状态：{readiness().historical_outcome_feature_label_join_target_training_registration_admission_status}；批准也只开放未来 create-once 训练登记门禁，不登记、不授权或运行训练。</small>
                    </article>
                    <article>
                      <header><strong>51 训练实验一次性登记</strong><span>{readiness().historical_outcome_training_experiment_pending_independent_review_count > 0 ? "registered_not_run · 等待独立复核" : readiness().historical_outcome_training_experiment_registration_failed_or_incomplete_count > 0 ? "claim 已消费 · 登记失败/不完整" : readiness().historical_outcome_feature_label_join_target_future_create_once_training_registration_eligible_count > 0 ? "等待 claim-first 一次性登记" : "等待 Stage 50 准入"}</span></header>
                      <p>先写不可变 claim，再一次性登记服务器固定的零预测基线、岭回归和梯度提升三种实验臂及 17 / 29 / 43 三组种子；明确 65 项特征、九项连续结果目标、目标隐藏边界和资源上限。</p>
                      <div>
                        <span>已准入候选 {readiness().historical_outcome_training_experiment_registration_admitted_candidate_count}</span>
                        <span>claim {readiness().historical_outcome_training_experiment_registration_claim_count}</span>
                        <span>已登记未运行 {readiness().historical_outcome_training_experiment_registered_not_run_count}</span>
                        <span>失败/不完整 {readiness().historical_outcome_training_experiment_registration_failed_or_incomplete_count}</span>
                        <span>待独立复核 {readiness().historical_outcome_training_experiment_pending_independent_review_count}</span>
                      </div>
                      <small>登记状态：{readiness().historical_outcome_training_experiment_registration_status}；登记完成仍不授权或启动训练，也不生成标量奖励、动作、仓位或排名。</small>
                    </article>
                    <article>
                      <header><strong>52 训练实验登记独立复核</strong><span>{readiness().historical_outcome_training_experiment_registration_independently_approved_count > 0 ? "已独立批准 · 等待训练实现登记" : readiness().historical_outcome_training_experiment_registration_rejected_or_changes_requested_count > 0 ? "退回或拒绝 · 已关闭" : readiness().historical_outcome_training_experiment_registration_review_eligible_count > 0 ? "等待独立复核" : "等待 Stage 51 登记"}</span></header>
                      <p>由登记人和完整上游之外的新角色，独立重算 claim、实验规范、registration 与 result，并复核三模型臂、三种子、九项目标、指标和封存集边界。</p>
                      <div>
                        <span>待复核 {readiness().historical_outcome_training_experiment_registration_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_training_experiment_registration_reviewed_count}</span>
                        <span>独立批准 {readiness().historical_outcome_training_experiment_registration_independently_approved_count}</span>
                        <span>退回/拒绝 {readiness().historical_outcome_training_experiment_registration_rejected_or_changes_requested_count}</span>
                        <span>可进入实现登记 {readiness().historical_outcome_future_training_implementation_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_training_experiment_registration_review_status}；批准仍不创建 runner、不授权或启动训练。</small>
                    </article>
                    <article>
                      <header><strong>53 训练实现登记</strong><span>{readiness().historical_outcome_training_implementation_current_binding_count > 0 ? "已登记 · 等待独立实现复核" : readiness().historical_outcome_training_implementation_registration_eligible_count > 0 ? "可登记 · 未提交" : "等待 Stage 52 独立批准"}</span></header>
                      <p>冻结不可变代码版本、实现工件哈希、三模型臂、三种子、65 项特征、九项目标、逐目标逐种子指标与资源上限，但不暴露任何可调用入口。</p>
                      <div>
                        <span>可登记 {readiness().historical_outcome_training_implementation_registration_eligible_count}</span>
                        <span>实现记录 {readiness().historical_outcome_training_implementation_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_training_implementation_current_binding_count}</span>
                        <span>待独立实现复核 {readiness().historical_outcome_training_implementation_pending_independent_review_count}</span>
                      </div>
                      <small>登记状态：{readiness().historical_outcome_training_implementation_status}；只开放下一道独立实现复核，不创建 runner、不访问训练数据、不运行训练。</small>
                    </article>
                    <article>
                      <header><strong>54 训练实现独立复核</strong><span>{readiness().historical_outcome_training_implementation_independently_approved_count > 0 ? "已独立批准 · 仅可登记 runner 规格" : readiness().historical_outcome_training_implementation_review_rejected_or_changes_requested_count > 0 ? "退回或拒绝 · 已关闭" : readiness().historical_outcome_training_implementation_review_eligible_count > 0 ? "等待独立复核" : "等待 Stage 53 实现登记"}</span></header>
                      <p>由实现登记人和完整上游之外的新角色，独立重算实现/合同摘要，复核三臂三种子、65/9、train/validation/holdout 隔离、逐目标指标和零能力边界。</p>
                      <div>
                        <span>待复核 {readiness().historical_outcome_training_implementation_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_training_implementation_reviewed_count}</span>
                        <span>独立批准 {readiness().historical_outcome_training_implementation_independently_approved_count}</span>
                        <span>退回/拒绝 {readiness().historical_outcome_training_implementation_review_rejected_or_changes_requested_count}</span>
                        <span>可登记 runner 规格 {readiness().historical_outcome_future_isolated_training_runner_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_training_implementation_review_status}；实现复核不是 runner、数据访问或训练授权。</small>
                    </article>
                    <article>
                      <header><strong>55 训练隔离 runner 规格登记</strong><span>{readiness().historical_outcome_training_isolated_runner_current_binding_count > 0 ? "registered_not_run · 等待首次执行授权复核" : readiness().historical_outcome_future_isolated_training_runner_registration_eligible_count > 0 ? "可登记 · 未提交" : "等待 Stage 54 独立批准"}</span></header>
                      <p>冻结 runner 工件、不可变代码、零环境运行时、未来精确只读 training-store 输入、train/validation/sealed-holdout 边界、create-once 候选输出和固定资源上限，但不提供调用入口。</p>
                      <div>
                        <span>runner 记录 {readiness().historical_outcome_training_isolated_runner_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_training_isolated_runner_current_binding_count}</span>
                        <span>可送首次执行复核 {readiness().historical_outcome_training_first_execution_authorization_review_eligible_count}</span>
                      </div>
                      <small>登记状态：{readiness().historical_outcome_training_isolated_runner_status}；登记不是数据访问、训练、模型工件或指标授权。</small>
                    </article>
                    <article>
                      <header><strong>56 训练首次执行授权复核</strong><span>{readiness().historical_outcome_training_execution_attempt_eligible_count > 0 ? "24 小时内单次资格 · 未 claim" : readiness().historical_outcome_training_first_execution_authorization_reviewed_count > 0 ? "已复核 · 当前未授权" : readiness().historical_outcome_training_first_execution_authorization_review_eligible_count > 0 ? "等待独立复核" : "等待 Stage 55 runner"}</span></header>
                      <p>由 runner 登记人和完整上游之外的新角色，独立复核工件、只读训练副本、三臂三种子、65/9、train/validation/holdout 隔离与零能力沙箱。</p>
                      <div>
                        <span>可复核 {readiness().historical_outcome_training_first_execution_authorization_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_training_first_execution_authorization_reviewed_count}</span>
                        <span>批准 {readiness().historical_outcome_training_first_execution_authorization_approved_count}</span>
                        <span>有效单次资格 {readiness().historical_outcome_training_one_shot_first_execution_authorized_count}</span>
                        <span>执行尝试候选 {readiness().historical_outcome_training_execution_attempt_eligible_count}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_training_first_execution_authorization_status}；授权只开放下一阶段一次 claim-first、train-only 拟合，不代表模型有效。</small>
                    </article>
                    <article>
                      <header><strong>57 训练一次性执行尝试</strong><span>{readiness().historical_outcome_training_independent_output_validation_eligible_count > 0 ? "未验证候选 · 等待独立校验" : readiness().historical_outcome_training_execution_claim_count > readiness().historical_outcome_training_completed_execution_attempt_count + readiness().historical_outcome_training_failed_execution_attempt_count ? "claim 已消费 · 结果缺失失败关闭" : readiness().historical_outcome_training_failed_execution_attempt_count > 0 ? "执行失败 · 授权已消费" : readiness().historical_outcome_training_execution_attempt_eligible_count > 0 ? "可执行一次 · 尚未 claim" : "等待 Stage 56 授权"}</span></header>
                      <p>先不可逆写入 claim，再只用精确 training-store 的 train 标签拟合固定三臂三种子；validation 和 sealed holdout 标签继续隐藏。</p>
                      <div>
                        <span>claim {readiness().historical_outcome_training_execution_claim_count}</span>
                        <span>完成 {readiness().historical_outcome_training_completed_execution_attempt_count}</span>
                        <span>失败 {readiness().historical_outcome_training_failed_execution_attempt_count}</span>
                        <span>未验证候选 {readiness().historical_outcome_training_untrusted_artifact_envelope_count}</span>
                        <span>可送独立校验 {readiness().historical_outcome_training_independent_output_validation_eligible_count}</span>
                      </div>
                      <small>执行状态：{readiness().historical_outcome_training_execution_attempt_status}；真实拟合 ≠ 模型有效，不做 validation 选模，不写 reward、影子仓位或订单。</small>
                    </article>
                    <article>
                      <header><strong>58 训练产物独立复算验证</strong><span>{readiness().historical_outcome_training_failed_output_validation_count > 0 ? "复算不一致 · 失败关闭" : readiness().historical_outcome_training_independently_validated_train_only_artifact_envelope_count > 0 ? "逐位通过 · 等待 validation 评估实现登记" : readiness().historical_outcome_training_output_validation_eligible_count > 0 ? "等待第二实现复算" : "等待 Stage 57 训练产物"}</span></header>
                      <p>执行链之外的新角色重开冻结数据和套件，第二实现独立复算 65 项预处理、9 个模型工件与 81 项 train-only 诊断；任一浮点位模式不一致即失败关闭。</p>
                      <div>
                        <span>待验证 {readiness().historical_outcome_training_output_validation_eligible_count}</span>
                        <span>验证记录 {readiness().historical_outcome_training_output_validation_count}</span>
                        <span>逐位通过 {readiness().historical_outcome_training_independently_validated_train_only_artifact_envelope_count}</span>
                        <span>失败 {readiness().historical_outcome_training_failed_output_validation_count}</span>
                        <span>可登记 validation 评估实现 {readiness().historical_outcome_future_validation_evaluation_implementation_registration_eligible_count}</span>
                      </div>
                      <small>验证状态：{readiness().historical_outcome_training_output_validation_status}；可重现 ≠ 模型有效，validation/holdout 标签、选模、模型库、reward 和交易仍关闭。</small>
                    </article>
                    <article>
                      <header><strong>59 validation 评估实现登记</strong><span>{readiness().historical_outcome_validation_evaluation_implementation_current_binding_count > 0 ? "规则已冻结 · 等待独立复核" : readiness().historical_outcome_validation_evaluation_implementation_registration_eligible_count > 0 ? "可预注册 · 尚未看标签" : "等待 Stage 58 逐位通过"}</span></header>
                      <p>在读取 validation 标签前冻结逐目标逐种子指标、零预测配对基准、component block bootstrap、Holm 修正、样本不足和禁止 seed shopping 的规则。</p>
                      <div>
                        <span>可登记 {readiness().historical_outcome_validation_evaluation_implementation_registration_eligible_count}</span>
                        <span>实现记录 {readiness().historical_outcome_validation_evaluation_implementation_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_validation_evaluation_implementation_current_binding_count}</span>
                        <span>待独立复核 {readiness().historical_outcome_validation_evaluation_implementation_independent_review_eligible_count}</span>
                      </div>
                      <small>登记状态：{readiness().historical_outcome_validation_evaluation_implementation_status}；当前无入口、无标签访问、无评估、无选模、无 sealed holdout 或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>60 validation 评估实现独立复核</strong><span>{readiness().historical_outcome_validation_evaluation_implementation_independently_approved_count > 0 ? "独立批准 · 仅可登记未来 runner" : readiness().historical_outcome_validation_evaluation_implementation_review_eligible_count > 0 ? "等待链外角色复算" : "等待 Stage 59 实现"}</span></header>
                      <p>链外角色独立重算实现、合同与候选集合指纹，并核对 3×3×9 工件矩阵、逐目标指标、bootstrap/Holm、最小效果与三种子全通过规则。</p>
                      <div>
                        <span>待复核 {readiness().historical_outcome_validation_evaluation_implementation_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_validation_evaluation_implementation_reviewed_count}</span>
                        <span>独立批准 {readiness().historical_outcome_validation_evaluation_implementation_independently_approved_count}</span>
                        <span>退回/拒绝 {readiness().historical_outcome_validation_evaluation_implementation_review_rejected_or_changes_requested_count}</span>
                        <span>可登记未来 runner {readiness().historical_outcome_future_isolated_validation_evaluation_runner_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_validation_evaluation_implementation_review_status}；批准仍无标签访问、评估、选模、sealed holdout、模型/指标库或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>61 validation 评估隔离 runner 登记</strong><span>{readiness().historical_outcome_validation_evaluation_isolated_runner_current_binding_count > 0 ? "规格已冻结 · 等待独立首次授权" : readiness().historical_outcome_future_isolated_validation_evaluation_runner_registration_eligible_count > 0 ? "可登记 · 当前无入口" : "等待 Stage 60 独立批准"}</span></header>
                      <p>冻结内容寻址 runner 工件、运行时、未来只读 validation 与九候选输入、逐目标逐种子 create-once 不可信输出、sealed holdout 隔离及资源上限。</p>
                      <div>
                        <span>runner 记录 {readiness().historical_outcome_validation_evaluation_isolated_runner_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_validation_evaluation_isolated_runner_current_binding_count}</span>
                        <span>可进入首次授权复核 {readiness().historical_outcome_validation_evaluation_first_execution_authorization_review_eligible_count}</span>
                      </div>
                      <small>登记状态：{readiness().historical_outcome_validation_evaluation_isolated_runner_status}；登记不是运行，当前无 validation 标签、评估、选模、输出或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>62 validation 评估首次执行授权</strong><span>{readiness().historical_outcome_validation_evaluation_execution_attempt_eligible_count > 0 ? "24 小时单次资格 · 尚未执行" : readiness().historical_outcome_validation_evaluation_first_execution_authorization_review_eligible_count > 0 ? "等待链外角色复核" : "等待 Stage 61 runner"}</span></header>
                      <p>链外复核者重新绑定 runner、评估实现、独立复核、九候选与完整上游；批准只产生 24 小时、最多一次的未来隔离调用资格。</p>
                      <div>
                        <span>已复核 {readiness().historical_outcome_validation_evaluation_first_execution_authorization_reviewed_count}</span>
                        <span>批准记录 {readiness().historical_outcome_validation_evaluation_first_execution_authorization_approved_count}</span>
                        <span>未过期 {readiness().historical_outcome_validation_evaluation_first_execution_authorization_unexpired_count}</span>
                        <span>单次资格 {readiness().historical_outcome_validation_evaluation_first_execution_authorization_one_shot_count}</span>
                        <span>可进入执行尝试 {readiness().historical_outcome_validation_evaluation_execution_attempt_eligible_count}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_validation_evaluation_first_execution_authorization_status}；本阶段没有 claim、标签挂载、评估、选模、输出、sealed holdout 或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>63 validation 评估一次性执行</strong><span>{readiness().historical_outcome_validation_evaluation_independent_output_validation_eligible_count > 0 ? "不可信结果 · 待独立复算" : readiness().historical_outcome_validation_evaluation_execution_claim_count > readiness().historical_outcome_validation_evaluation_completed_attempt_count + readiness().historical_outcome_validation_evaluation_failed_attempt_count ? "claim 已消费 · 结果待落盘" : readiness().historical_outcome_validation_evaluation_execution_attempt_eligible_count > 0 ? "可 claim-first 执行一次" : "等待 Stage 62 授权"}</span></header>
                      <p>先不可逆 claim，再由宿主标签代理只向固定 worker 投影 validation 行；运行冻结的 3 算法 × 3 种子 × 9 目标指标、成分块 bootstrap 与 Holm 校正。</p>
                      <div>
                        <span>claim {readiness().historical_outcome_validation_evaluation_execution_claim_count}</span>
                        <span>完成 {readiness().historical_outcome_validation_evaluation_completed_attempt_count}</span>
                        <span>失败且已消费 {readiness().historical_outcome_validation_evaluation_failed_attempt_count}</span>
                        <span>不可信 envelope {readiness().historical_outcome_validation_evaluation_untrusted_envelope_count}</span>
                        <span>待独立复算 {readiness().historical_outcome_validation_evaluation_independent_output_validation_eligible_count}</span>
                      </div>
                      <small>执行状态：{readiness().historical_outcome_validation_evaluation_execution_attempt_status}；sealed holdout、全局有效性、正式选模、模型/指标库与交易权限仍关闭。</small>
                    </article>
                    <article>
                      <header><strong>64 validation 评估输出独立复算</strong><span>{readiness().historical_outcome_validation_evaluation_failed_output_validation_count > 0 ? "复算不一致 · 失败关闭" : readiness().historical_outcome_validation_evaluation_independently_validated_untrusted_envelope_count > 0 ? "逐位通过 · 等待逐目标准入复核" : readiness().historical_outcome_validation_evaluation_output_validation_eligible_count > 0 ? "等待第二实现复算" : "等待 Stage 63 评估产物"}</span></header>
                      <p>链外验证者以第二套路径重构 validation-only 投影与九候选预测，逐位复算 81 指标、54 项 component bootstrap/Holm 检验和 9 项逐目标建议。</p>
                      <div>
                        <span>待复算 {readiness().historical_outcome_validation_evaluation_output_validation_eligible_count}</span>
                        <span>验证记录 {readiness().historical_outcome_validation_evaluation_output_validation_count}</span>
                        <span>逐位通过 {readiness().historical_outcome_validation_evaluation_independently_validated_untrusted_envelope_count}</span>
                        <span>失败关闭 {readiness().historical_outcome_validation_evaluation_failed_output_validation_count}</span>
                        <span>可进入逐目标准入复核 {readiness().historical_outcome_future_per_target_candidate_admission_review_eligible_count}</span>
                      </div>
                      <small>验证状态：{readiness().historical_outcome_validation_evaluation_output_validation_status}；通过仍不是正式选模、模型有效性、收益证明或交易授权。</small>
                    </article>
                    <article>
                      <header><strong>65 逐目标候选准入复核</strong><span>{readiness().historical_outcome_validation_evaluation_per_target_candidate_admitted_count > 0 ? "逐目标准入 · 等待留出集协议复核" : readiness().historical_outcome_validation_evaluation_per_target_candidate_count > 0 ? "逐个复核 · 不做综合分" : "等待 Stage 64 独立复算"}</span></header>
                      <p>把九个目标拆成九道独立门：每个目标单独核对三种算法 × 三个冻结种子的九项指标、证据状态、三种子门槛与建议，不能互相掩盖。</p>
                      <div>
                        <span>目标候选 {readiness().historical_outcome_validation_evaluation_per_target_candidate_count}</span>
                        <span>已复核 {readiness().historical_outcome_validation_evaluation_per_target_candidate_reviewed_count}</span>
                        <span>已准入 {readiness().historical_outcome_validation_evaluation_per_target_candidate_admitted_count}</span>
                        <span>证据不足 {readiness().historical_outcome_validation_evaluation_per_target_insufficient_evidence_count}</span>
                        <span>三种子未全过 {readiness().historical_outcome_validation_evaluation_per_target_no_candidate_passed_count}</span>
                      </div>
                      <small>准入状态：{readiness().historical_outcome_validation_evaluation_per_target_candidate_admission_status}；准入只允许未来 sealed-holdout 评估协议复核，不开放留出集或正式选模。</small>
                    </article>
                    <article>
                      <header><strong>66 sealed-holdout 评估协议独立复核</strong><span>{readiness().historical_outcome_sealed_holdout_evaluation_protocol_independently_approved_count > 0 ? "协议已批准 · 仅可登记实现" : readiness().historical_outcome_sealed_holdout_evaluation_protocol_admitted_target_count > 0 ? "逐目标复核 · 不打开试卷" : "等待 Stage 65 逐目标准入"}</span></header>
                      <p>在任何 sealed holdout 数据可见前，逐目标冻结一种算法、三个种子、固定指标与门槛、独立组件 bootstrap、三项假设 Holm 校正和一次性无反馈复用规则。</p>
                      <div>
                        <span>已准入目标 {readiness().historical_outcome_sealed_holdout_evaluation_protocol_admitted_target_count}</span>
                        <span>已复核 {readiness().historical_outcome_sealed_holdout_evaluation_protocol_reviewed_count}</span>
                        <span>独立批准 {readiness().historical_outcome_sealed_holdout_evaluation_protocol_independently_approved_count}</span>
                        <span>退回/拒绝 {readiness().historical_outcome_sealed_holdout_evaluation_protocol_rejected_or_changes_requested_count}</span>
                        <span>可登记未来评估实现 {readiness().historical_outcome_future_sealed_holdout_evaluation_implementation_registration_eligible_count}</span>
                      </div>
                      <small>协议状态：{readiness().historical_outcome_sealed_holdout_evaluation_protocol_review_status}；本阶段不读取、挂载、解密、投影或执行 sealed holdout，也不正式选模。</small>
                    </article>
                    <article>
                      <header><strong>67 sealed-holdout 评估实现登记</strong><span>{readiness().historical_outcome_sealed_holdout_evaluation_implementation_current_binding_count > 0 ? "已登记未运行 · 等待独立复核" : readiness().historical_outcome_sealed_holdout_evaluation_implementation_registration_eligible_count > 0 ? "可登记 · 当前无入口" : "等待 Stage 66 协议批准"}</span></header>
                      <p>把 Stage 66 的逐目标协议绑定成不可变、内容寻址的零能力实现合同；固定一种算法、三个种子、65/1 输入输出、统计规则和未来不可信输出格式。</p>
                      <div>
                        <span>可登记 {readiness().historical_outcome_sealed_holdout_evaluation_implementation_registration_eligible_count}</span>
                        <span>实现记录 {readiness().historical_outcome_sealed_holdout_evaluation_implementation_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_sealed_holdout_evaluation_implementation_current_binding_count}</span>
                        <span>待独立复核 {readiness().historical_outcome_sealed_holdout_evaluation_implementation_independent_review_eligible_count}</span>
                      </div>
                      <small>实现状态：{readiness().historical_outcome_sealed_holdout_evaluation_implementation_status}；登记不是执行，没有入口、挂载、数据 adapter、留出集访问或评估授权。</small>
                    </article>
                    <article>
                      <header><strong>68 sealed-holdout 评估实现独立复核</strong><span>{readiness().historical_outcome_sealed_holdout_evaluation_implementation_independently_approved_count > 0 ? "独立批准 · 仅可登记 runner" : readiness().historical_outcome_sealed_holdout_evaluation_implementation_review_eligible_count > 0 ? "等待链外复核 · 无数据访问" : "等待 Stage 67 实现登记"}</span></header>
                      <p>由 Stage 51–67 完整角色链之外的新复核者重算实现、合同和 Stage 66 协议哈希，核对单目标、单算法、三种子、固定统计门槛、one-shot 无反馈与零能力边界。</p>
                      <div>
                        <span>待复核 {readiness().historical_outcome_sealed_holdout_evaluation_implementation_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_sealed_holdout_evaluation_implementation_reviewed_count}</span>
                        <span>独立批准 {readiness().historical_outcome_sealed_holdout_evaluation_implementation_independently_approved_count}</span>
                        <span>退回/拒绝 {readiness().historical_outcome_sealed_holdout_evaluation_implementation_rejected_or_changes_requested_count}</span>
                        <span>可登记未来 runner {readiness().historical_outcome_future_isolated_sealed_holdout_evaluation_runner_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_sealed_holdout_evaluation_implementation_review_status}；批准不创建 runner，不读取 sealed holdout，不评估、不选模、不交易。</small>
                    </article>
                    <article>
                      <header><strong>69 sealed-holdout 评估隔离 runner 登记</strong><span>{readiness().historical_outcome_sealed_holdout_evaluation_isolated_runner_current_binding_count > 0 ? "已登记未运行 · 等待一次性授权复核" : readiness().historical_outcome_sealed_holdout_evaluation_isolated_runner_registration_eligible_count > 0 ? "可登记 · 当前无入口" : "等待 Stage 68 独立批准"}</span></header>
                      <p>为每条 Stage 68 批准复核只登记一个不可变、内容寻址、无入口 runner 规格；冻结单目标、单算法、17/29/43 三候选、sealed split、统计合同与静态资源边界。</p>
                      <div>
                        <span>可登记 {readiness().historical_outcome_sealed_holdout_evaluation_isolated_runner_registration_eligible_count}</span>
                        <span>runner 记录 {readiness().historical_outcome_sealed_holdout_evaluation_isolated_runner_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_sealed_holdout_evaluation_isolated_runner_current_binding_count}</span>
                        <span>待一次性授权复核 {readiness().historical_outcome_sealed_holdout_evaluation_first_execution_authorization_review_eligible_count}</span>
                      </div>
                      <small>runner 状态：{readiness().historical_outcome_sealed_holdout_evaluation_isolated_runner_status}；登记不提供留出集访问、挂载或执行能力，下一门禁仍须链外独立审批。</small>
                    </article>
                    <article>
                      <header><strong>70 sealed-holdout 首次访问与执行授权复核</strong><span>{readiness().historical_outcome_sealed_holdout_evaluation_execution_attempt_eligible_count > 0 ? "24 小时单次资格有效 · 尚未领取" : readiness().historical_outcome_sealed_holdout_evaluation_first_execution_authorization_review_eligible_count > 0 ? "等待链外独立复核 · 不打开试卷" : "等待 Stage 69 runner"}</span></header>
                      <p>由 Stage 51–69 完整责任链之外的新复核者重算 runner 工件和全链绑定，只批准未来一次、限时、精确只读的单目标 sealed-holdout 访问与评估资格。</p>
                      <div>
                        <span>可复核 {readiness().historical_outcome_sealed_holdout_evaluation_first_execution_authorization_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_sealed_holdout_evaluation_first_execution_authorization_reviewed_count}</span>
                        <span>历史批准 {readiness().historical_outcome_sealed_holdout_evaluation_first_execution_authorization_approved_count}</span>
                        <span>有效单次资格 {readiness().historical_outcome_sealed_holdout_evaluation_first_execution_authorization_unexpired_count}</span>
                        <span>可送 Stage 71 {readiness().historical_outcome_sealed_holdout_evaluation_execution_attempt_eligible_count}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_sealed_holdout_evaluation_first_execution_authorization_status}；审批接口没有 claim、挂载或执行入口，不读取 sealed holdout，不创建输出，也不选模或交易。</small>
                    </article>
                    <article>
                      <header><strong>71 sealed-holdout 一次性确认执行</strong><span>{readiness().historical_outcome_sealed_holdout_evaluation_independent_output_validation_eligible_count > 0 ? "不可信结果 · 等待独立复算" : readiness().historical_outcome_sealed_holdout_evaluation_execution_attempt_eligible_count > 0 ? "可 claim 一次 · 尚未开卷" : readiness().historical_outcome_sealed_holdout_evaluation_failed_attempt_count > 0 ? "授权已消费 · 失败关闭" : "等待 Stage 70 授权"}</span></header>
                      <p>先建立不可逆 claim，再只投影一个目标、一个冻结算法及 17/29/43 三种子到 sealed-holdout；按固定门槛、组件 bootstrap 和三项 Holm 校正输出一次确认结果。</p>
                      <div>
                        <span>claim {readiness().historical_outcome_sealed_holdout_evaluation_execution_claim_count}</span>
                        <span>完成 {readiness().historical_outcome_sealed_holdout_evaluation_completed_attempt_count}</span>
                        <span>失败 {readiness().historical_outcome_sealed_holdout_evaluation_failed_attempt_count}</span>
                        <span>不可信 envelope {readiness().historical_outcome_sealed_holdout_evaluation_untrusted_confirmation_envelope_count}</span>
                        <span>待独立验证 {readiness().historical_outcome_sealed_holdout_evaluation_independent_output_validation_eligible_count}</span>
                      </div>
                      <small>执行状态：{readiness().historical_outcome_sealed_holdout_evaluation_execution_attempt_status}；成功、失败和中断都不能重放，结果不能反馈、正式选模、写库或交易。</small>
                    </article>
                    <article>
                      <header><strong>72 sealed-holdout 输出独立复算</strong><span>{readiness().historical_outcome_future_confirmatory_result_adjudication_review_eligible_count > 0 ? "逐位通过 · 等待裁决复核" : readiness().historical_outcome_sealed_holdout_evaluation_failed_output_validation_count > 0 ? "复算不一致 · 失败关闭" : readiness().historical_outcome_sealed_holdout_evaluation_output_validation_eligible_count > 0 ? "等待责任链外验证者" : "等待 Stage 71 结果"}</span></header>
                      <p>由执行者和完整 Stage 51–71 责任链之外的新管理员，使用第二实现重构 holdout 投影，重新预测三冻结种子，并逐位复算三指标、component bootstrap、Holm 和全部门槛。</p>
                      <div>
                        <span>待复算 {readiness().historical_outcome_sealed_holdout_evaluation_output_validation_eligible_count}</span>
                        <span>验证记录 {readiness().historical_outcome_sealed_holdout_evaluation_output_validation_count}</span>
                        <span>逐位通过 {readiness().historical_outcome_sealed_holdout_evaluation_independently_validated_confirmation_count}</span>
                        <span>失败关闭 {readiness().historical_outcome_sealed_holdout_evaluation_failed_output_validation_count}</span>
                        <span>待裁决 {readiness().historical_outcome_future_confirmatory_result_adjudication_review_eligible_count}</span>
                      </div>
                      <small>验证状态：{readiness().historical_outcome_sealed_holdout_evaluation_output_validation_status}；通过只开放未来裁决复核，不代表模型有效、收益成立或允许交易。</small>
                    </article>
                    <article>
                      <header><strong>73 确认结果独立裁决</strong><span>{readiness().historical_outcome_future_controlled_shadow_experiment_design_registration_eligible_count > 0 ? "裁决通过 · 仅可登记实验设计" : readiness().historical_outcome_sealed_holdout_confirmatory_result_quantitative_fail_or_insufficient_count > 0 ? "定量失败/不足 · 不可覆盖" : readiness().historical_outcome_sealed_holdout_confirmatory_result_adjudication_candidate_count > 0 ? "等待经济意义与偏差复核" : "等待 Stage 72 结果"}</span></header>
                      <p>把统计可复现与投资可用性分开：复核样本和独立分量、效应量、多重检验、目标经济语义、覆盖偏差、失败模式、局限与证伪条件。</p>
                      <div>
                        <span>候选 {readiness().historical_outcome_sealed_holdout_confirmatory_result_adjudication_candidate_count}</span>
                        <span>定量通过 {readiness().historical_outcome_sealed_holdout_confirmatory_result_quantitative_pass_count}</span>
                        <span>失败/不足 {readiness().historical_outcome_sealed_holdout_confirmatory_result_quantitative_fail_or_insufficient_count}</span>
                        <span>已复核 {readiness().historical_outcome_sealed_holdout_confirmatory_result_adjudication_reviewed_count}</span>
                        <span>裁决通过 {readiness().historical_outcome_sealed_holdout_confirmatory_result_adjudication_approved_count}</span>
                        <span>可登记设计 {readiness().historical_outcome_future_controlled_shadow_experiment_design_registration_eligible_count}</span>
                      </div>
                      <small>裁决状态：{readiness().historical_outcome_sealed_holdout_confirmatory_result_adjudication_status}；人工不能覆盖定量失败，通过也不正式选模、不启动影子盘或交易。</small>
                    </article>
                    <article>
                      <header><strong>74 受控影子实验设计登记</strong><span>{readiness().historical_outcome_future_independent_shadow_design_review_eligible_count > 0 ? "已登记 · 等待独立设计复核" : readiness().historical_outcome_controlled_shadow_experiment_design_registration_eligible_count > 0 ? "可登记 · 尚未运行" : "等待 Stage 73 裁决"}</span></header>
                      <p>冻结实验候选、SPY/现金/等权/规则反事实、信号时点、组合上限、成本、252 日观察窗口、分项指标与停止规则。</p>
                      <div>
                        <span>已裁决候选 {readiness().historical_outcome_controlled_shadow_experiment_design_adjudicated_candidate_count}</span>
                        <span>待登记 {readiness().historical_outcome_controlled_shadow_experiment_design_registration_eligible_count}</span>
                        <span>已登记 {readiness().historical_outcome_controlled_shadow_experiment_design_registered_count}</span>
                        <span>待独立复核 {readiness().historical_outcome_future_independent_shadow_design_review_eligible_count}</span>
                      </div>
                      <small>登记状态：{readiness().historical_outcome_controlled_shadow_experiment_design_registration_status}；当前不正式选模、不写模型库，不创建影子账本、持仓或订单。</small>
                    </article>
                    <article>
                      <header><strong>75 受控影子实验设计独立复核</strong><span>{readiness().historical_outcome_future_zero_capability_shadow_implementation_registration_eligible_count > 0 ? "独立通过 · 仅可登记零能力实现" : readiness().historical_outcome_controlled_shadow_experiment_design_changes_or_rejected_count > 0 ? "要求新建设计/已拒绝" : readiness().historical_outcome_controlled_shadow_experiment_design_review_eligible_count > 0 ? "等待责任链外复核" : "等待 Stage 74 登记"}</span></header>
                      <p>责任链外复核者独立复算登记和设计指纹，并审查点时与退市偏差、全部反事实、信号与成本、组合边界、观察门槛、多重检验、停止规则及未确认投资逻辑隔离。</p>
                      <div>
                        <span>已登记设计 {readiness().historical_outcome_controlled_shadow_experiment_design_review_registered_design_count}</span>
                        <span>待复核 {readiness().historical_outcome_controlled_shadow_experiment_design_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_controlled_shadow_experiment_design_reviewed_count}</span>
                        <span>独立通过 {readiness().historical_outcome_controlled_shadow_experiment_design_independently_approved_count}</span>
                        <span>待改/拒绝 {readiness().historical_outcome_controlled_shadow_experiment_design_changes_or_rejected_count}</span>
                        <span>可登记零能力实现 {readiness().historical_outcome_future_zero_capability_shadow_implementation_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_controlled_shadow_experiment_design_review_status}；通过不代表模型有效，不实现或运行影子盘，也不创建持仓、订单、券商访问或真实交易。</small>
                    </article>
                    <article>
                      <header><strong>76 零能力影子实现规格登记</strong><span>{readiness().historical_outcome_controlled_shadow_experiment_implementation_independent_review_eligible_count > 0 ? "已登记 · 仅可独立复核" : readiness().historical_outcome_controlled_shadow_experiment_implementation_registration_eligible_count > 0 ? "可登记纯规格" : "等待 Stage 75 批准"}</span></header>
                      <p>把已独立批准的实验设计绑定为内容寻址的确定性重放规格；只有纯函数语义和未来不可信输入/输出信封，不含可执行程序。</p>
                      <div>
                        <span>待登记 {readiness().historical_outcome_controlled_shadow_experiment_implementation_registration_eligible_count}</span>
                        <span>已登记 {readiness().historical_outcome_controlled_shadow_experiment_implementation_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_controlled_shadow_experiment_implementation_current_binding_count}</span>
                        <span>可独立复核 {readiness().historical_outcome_controlled_shadow_experiment_implementation_independent_review_eligible_count}</span>
                      </div>
                      <small>登记状态：{readiness().historical_outcome_controlled_shadow_experiment_implementation_status}；无入口、runtime、网络、生产读写、影子账本、持仓、订单、券商或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>77 零能力影子实现独立复核</strong><span>{readiness().historical_outcome_future_isolated_shadow_runner_specification_registration_eligible_count > 0 ? "独立通过 · 仅可登记 runner 规格" : readiness().historical_outcome_controlled_shadow_experiment_implementation_changes_requested_or_rejected_count > 0 ? "要求新建责任链/已拒绝" : readiness().historical_outcome_controlled_shadow_experiment_implementation_review_eligible_count > 0 ? "等待责任链外复核" : "等待 Stage 76 登记"}</span></header>
                      <p>责任链外复核者重算实现、合同、设计复核、设计登记和设计规格五层指纹，并逐项复核确定性语义、点时边界与全部零权限约束。</p>
                      <div>
                        <span>已登记实现 {readiness().historical_outcome_controlled_shadow_experiment_implementation_review_implementation_count}</span>
                        <span>待复核 {readiness().historical_outcome_controlled_shadow_experiment_implementation_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_controlled_shadow_experiment_implementation_reviewed_count}</span>
                        <span>独立通过 {readiness().historical_outcome_controlled_shadow_experiment_implementation_independently_approved_count}</span>
                        <span>待改/拒绝 {readiness().historical_outcome_controlled_shadow_experiment_implementation_changes_requested_or_rejected_count}</span>
                        <span>可登记隔离 runner 规格 {readiness().historical_outcome_future_isolated_shadow_runner_specification_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_controlled_shadow_experiment_implementation_review_status}；通过不是运行授权，runner、影子账本、持仓、订单、券商和交易仍全部关闭。</small>
                    </article>
                    <article>
                      <header><strong>78 隔离影子 runner 规格登记</strong><span>{readiness().historical_outcome_controlled_shadow_experiment_first_execution_authorization_review_eligible_count > 0 ? "规格已登记 · 仅可首次授权复核" : readiness().historical_outcome_controlled_shadow_experiment_isolated_runner_registration_eligible_count > 0 ? "可登记不可执行规格" : "等待 Stage 77 批准"}</span></header>
                      <p>登记内容寻址的 runner，绑定精确可执行工件、代码版本和固定 runtime，同时冻结未来点时只读输入、一次性不可信输出、非特权身份与资源上限。</p>
                      <div>
                        <span>待登记 {readiness().historical_outcome_controlled_shadow_experiment_isolated_runner_registration_eligible_count}</span>
                        <span>已登记规格 {readiness().historical_outcome_controlled_shadow_experiment_isolated_runner_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_controlled_shadow_experiment_isolated_runner_current_binding_count}</span>
                        <span>可首次授权复核 {readiness().historical_outcome_controlled_shadow_experiment_first_execution_authorization_review_eligible_count}</span>
                      </div>
                      <small>登记状态：{readiness().historical_outcome_controlled_shadow_experiment_isolated_runner_status}；工件存在不等于入口或运行授权，当前无 callable entrypoint、挂载、数据访问、影子账本、持仓、订单、券商或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>79 首次影子执行授权独立复核</strong><span>{readiness().historical_outcome_controlled_shadow_experiment_execution_attempt_eligible_count > 0 ? "短时单次资格有效 · 未 claim" : readiness().historical_outcome_controlled_shadow_experiment_first_execution_authorization_reviewed_count > 0 ? "已复核 · 无有效资格" : readiness().historical_outcome_controlled_shadow_experiment_first_execution_authorization_review_eligible_count > 0 ? "等待责任链外复核" : "等待 Stage 78 规格"}</span></header>
                      <p>责任链外复核者重算 Stage 51–78 完整哈希链，独立复现可执行工件摘要并确认代码版本可复现；批准最多开放 24 小时内一次未来 Stage 80 claim-first 尝试。</p>
                      <div>
                        <span>已复核 {readiness().historical_outcome_controlled_shadow_experiment_first_execution_authorization_reviewed_count}</span>
                        <span>批准 {readiness().historical_outcome_controlled_shadow_experiment_first_execution_authorization_approved_count}</span>
                        <span>未过期 {readiness().historical_outcome_controlled_shadow_experiment_first_execution_authorization_unexpired_count}</span>
                        <span>单次资格 {readiness().historical_outcome_controlled_shadow_experiment_first_execution_authorization_one_shot_count}</span>
                        <span>Stage 80 候选 {readiness().historical_outcome_controlled_shadow_experiment_execution_attempt_eligible_count}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_controlled_shadow_experiment_first_execution_authorization_status}；本阶段没有 claim、输入、影子运行、账本、持仓、订单、券商或交易能力。</small>
                    </article>
                    <article>
                      <header><strong>80 claim-first 单次隔离影子初始化</strong><span>{readiness().historical_outcome_controlled_shadow_experiment_independent_output_validation_eligible_count > 0 ? "不可信输出 · 等待 Stage 81" : readiness().historical_outcome_controlled_shadow_experiment_execution_claim_count > readiness().historical_outcome_controlled_shadow_experiment_execution_completed_count + readiness().historical_outcome_controlled_shadow_experiment_execution_failed_count ? "claim 后中断 · 失败关闭" : readiness().historical_outcome_controlled_shadow_experiment_execution_failed_count > 0 ? "失败且授权已消费" : readiness().historical_outcome_controlled_shadow_experiment_execution_attempt_eligible_count > 0 ? "可提交点时输入" : "等待 Stage 79 授权"}</span></header>
                      <p>先不可变写 claim，再复核当前二进制并打开点时、只读、内容寻址、白名单输入；只执行冻结三种子信号和只做多虚拟组合初始化。</p>
                      <div>
                        <span>claim {readiness().historical_outcome_controlled_shadow_experiment_execution_claim_count}</span>
                        <span>完成 {readiness().historical_outcome_controlled_shadow_experiment_execution_completed_count}</span>
                        <span>失败 {readiness().historical_outcome_controlled_shadow_experiment_execution_failed_count}</span>
                        <span>不可信观察 {readiness().historical_outcome_controlled_shadow_experiment_untrusted_initial_observation_count}</span>
                        <span>Stage 81 候选 {readiness().historical_outcome_controlled_shadow_experiment_independent_output_validation_eligible_count}</span>
                      </div>
                      <small>执行状态：{readiness().historical_outcome_controlled_shadow_experiment_execution_status}；0 个已观察交易日不得生成未来收益，不写账本/持仓/模型/指标，不反馈 reward，不生成订单、不接券商、不交易。</small>
                    </article>
                    <article>
                      <header><strong>81 初始影子观察独立第二实现复算</strong><span>{readiness().historical_outcome_controlled_shadow_experiment_failed_output_validation_count > 0 ? "复算不一致 · 永久失败关闭" : readiness().historical_outcome_future_forward_observation_protocol_registration_eligible_count > 0 ? "逐位通过 · 仅可登记前向协议" : readiness().historical_outcome_controlled_shadow_experiment_output_validation_eligible_count > 0 ? "等待责任链外校验者" : "等待 Stage 80 输出"}</span></header>
                      <p>责任链外新角色重新提交同一内容寻址点时输入，不复用 Stage 80 投影、预测或权重 helper，逐位复算 17/29/43 三种子、排序、tie-break 和五重组合上限。</p>
                      <div>
                        <span>待复算 {readiness().historical_outcome_controlled_shadow_experiment_output_validation_eligible_count}</span>
                        <span>验证记录 {readiness().historical_outcome_controlled_shadow_experiment_output_validation_count}</span>
                        <span>逐位通过 {readiness().historical_outcome_controlled_shadow_experiment_independently_validated_initial_observation_count}</span>
                        <span>失败关闭 {readiness().historical_outcome_controlled_shadow_experiment_failed_output_validation_count}</span>
                        <span>可登记前向协议 {readiness().historical_outcome_future_forward_observation_protocol_registration_eligible_count}</span>
                      </div>
                      <small>验证状态：{readiness().historical_outcome_controlled_shadow_experiment_output_validation_status}；通过不生成前向绩效、账本或持仓，也不反馈训练/reward，不生成订单、不接券商、不交易。</small>
                    </article>
                    <article>
                      <header><strong>82 受控前向观察协议登记</strong><span>{readiness().historical_outcome_future_independent_protocol_review_eligible_count > 0 ? "协议已冻结 · 等待独立复核" : readiness().historical_outcome_forward_observation_protocol_registration_eligible_count > 0 ? "可登记" : "等待 Stage 81"}</span></header>
                      <p>只冻结自然时间前进、周度 claim-first、官方交易日历、SPY 同步基准、点时来源、复权/公司行动、更正留痕、成本、样本门槛与停止规则。</p>
                      <div>
                        <span>待登记 {readiness().historical_outcome_forward_observation_protocol_registration_eligible_count}</span>
                        <span>已登记 {readiness().historical_outcome_forward_observation_protocol_registered_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_forward_observation_protocol_current_binding_count}</span>
                        <span>待独立复核 {readiness().historical_outcome_future_independent_protocol_review_eligible_count}</span>
                      </div>
                      <small>协议状态：{readiness().historical_outcome_forward_observation_protocol_registration_status}；不得回填，当前未开始观察、未建账、未写持仓或绩效，也没有任何交易权限。</small>
                    </article>
                    <article>
                      <header><strong>83 前向观察协议责任链外独立复核</strong><span>{readiness().historical_outcome_future_zero_capability_forward_observation_implementation_registration_eligible_count > 0 ? "独立通过 · 仅可登记零能力实现" : readiness().historical_outcome_forward_observation_protocol_changes_required_or_rejected_count > 0 ? "需重建或已拒绝" : readiness().historical_outcome_forward_observation_protocol_review_eligible_count > 0 ? "等待责任链外复核" : "等待 Stage 82"}</span></header>
                      <p>独立重算 Stage 82 登记、协议和完整 Stage 74 设计，逐项复核自然前向、禁止回填、日历、来源、公司行动、成本、门槛、指标与停止规则。</p>
                      <div>
                        <span>已登记协议 {readiness().historical_outcome_forward_observation_protocol_review_registered_count}</span>
                        <span>待复核 {readiness().historical_outcome_forward_observation_protocol_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_forward_observation_protocol_reviewed_count}</span>
                        <span>独立通过 {readiness().historical_outcome_forward_observation_protocol_independently_approved_count}</span>
                        <span>待重建/拒绝 {readiness().historical_outcome_forward_observation_protocol_changes_required_or_rejected_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_forward_observation_protocol_review_status}；批准也只开放未来零能力观察实现规格登记，不开始观察、不建账、不建仓或交易。</small>
                    </article>
                    <article>
                      <header><strong>84 前向观察零能力实现规格登记</strong><span>{readiness().historical_outcome_forward_observation_implementation_independent_review_eligible_count > 0 ? "规格已冻结 · 等待独立复核" : readiness().historical_outcome_forward_observation_implementation_registration_eligible_count > 0 ? "可登记" : "等待 Stage 83"}</span></header>
                      <p>冻结周度 claim、官方交易日历、点时来源托管、公司行动更正、信号投影、组合转移、成交成本反事实、检查点指标与停止规则的纯函数标识。</p>
                      <div>
                        <span>待登记 {readiness().historical_outcome_forward_observation_implementation_registration_eligible_count}</span>
                        <span>已登记 {readiness().historical_outcome_forward_observation_implementation_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_forward_observation_implementation_current_binding_count}</span>
                        <span>待独立复核 {readiness().historical_outcome_forward_observation_implementation_independent_review_eligible_count}</span>
                      </div>
                      <small>实现状态：{readiness().historical_outcome_forward_observation_implementation_status}；规格，不是程序，无入口、runtime、挂载或生产读写，不开始观察、不建账、不写持仓/绩效，不下单、不接券商、不交易。</small>
                    </article>
                    <article>
                      <header><strong>85 前向观察实现责任链外独立复核</strong><span>{readiness().historical_outcome_forward_observation_implementation_independently_approved_count > 0 ? "独立通过 · 仅可登记隔离 runner 规格" : readiness().historical_outcome_forward_observation_implementation_changes_required_or_rejected_count > 0 ? "需重建或已拒绝" : readiness().historical_outcome_forward_observation_implementation_review_eligible_count > 0 ? "等待责任链外复核" : "等待 Stage 84"}</span></header>
                      <div>
                        <span>实现 {readiness().historical_outcome_forward_observation_implementation_review_registered_count}</span>
                        <span>待复核 {readiness().historical_outcome_forward_observation_implementation_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_forward_observation_implementation_reviewed_count}</span>
                        <span>独立批准 {readiness().historical_outcome_forward_observation_implementation_independently_approved_count}</span>
                        <span>可登记 runner 规格 {readiness().historical_outcome_future_isolated_forward_observation_runner_specification_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_forward_observation_implementation_review_status}；批准也不创建 runner、观察、账本、持仓、绩效、订单、券商或交易能力。</small>
                    </article>
                    <article>
                      <header><strong>86 前向观察隔离 runner 规格登记</strong><span>{readiness().historical_outcome_forward_observation_first_execution_authorization_review_eligible_count > 0 ? "规格已冻结 · 等待独立首跑授权" : readiness().historical_outcome_forward_observation_isolated_runner_registration_eligible_count > 0 ? "可登记" : "等待 Stage 85"}</span></header>
                      <p>绑定精确 runner 工件摘要、复现程序、不可变代码版本、固定非特权身份、未来点时输入与 create-once 非可信输出；不创建入口或 runtime。</p>
                      <div>
                        <span>待登记 {readiness().historical_outcome_forward_observation_isolated_runner_registration_eligible_count}</span>
                        <span>已登记 {readiness().historical_outcome_forward_observation_isolated_runner_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_forward_observation_isolated_runner_current_binding_count}</span>
                        <span>待首跑授权复核 {readiness().historical_outcome_forward_observation_first_execution_authorization_review_eligible_count}</span>
                      </div>
                      <small>runner 状态：{readiness().historical_outcome_forward_observation_isolated_runner_status}；工件身份已绑定，但无 callable entrypoint、runtime、挂载、观察、账本、持仓、绩效、订单、券商或交易能力。</small>
                    </article>
                    <article>
                      <header><strong>87 前向观察首次执行授权独立复核</strong><span>{readiness().historical_outcome_forward_observation_future_attempt_eligible_count > 0 ? "一次性候选已签发 · 尚未执行" : readiness().historical_outcome_forward_observation_first_execution_authorization_review_eligible_count > 0 ? "等待独立复现工件" : "等待 Stage 86"}</span></header>
                      <p>责任链外复核者必须独立复现 runner 工件 SHA-256 并重算完整绑定；批准仅签发 24 小时内最多一次的未来 Stage 88 claim-first 尝试候选。</p>
                      <div>
                        <span>已复核 {readiness().historical_outcome_forward_observation_first_execution_authorization_reviewed_count}</span>
                        <span>批准 {readiness().historical_outcome_forward_observation_first_execution_authorization_approved_count}</span>
                        <span>未过期 {readiness().historical_outcome_forward_observation_first_execution_authorization_unexpired_count}</span>
                        <span>未来尝试候选 {readiness().historical_outcome_forward_observation_future_attempt_eligible_count}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_forward_observation_first_execution_authorization_status}；没有 claim、入口、runtime、挂载、数据访问、观察、账本、持仓、绩效、订单、券商或交易能力。</small>
                    </article>
                    <article>
                      <header><strong>88 前向观察 claim-first 单次初始化</strong><span>{readiness().historical_outcome_forward_observation_execution_independent_validation_eligible_count > 0 ? "不可信初始化收据 · 等待独立验证" : readiness().historical_outcome_forward_observation_execution_attempt_eligible_count > 0 ? "可初始化一次" : readiness().historical_outcome_forward_observation_execution_claim_count > 0 ? "授权已消费" : "等待 Stage 87"}</span></header>
                      <p>先永久消费精确 Stage 87 授权，再复核二进制和零行情初始化清单；成功也只产生不可信的 day-0 初始化收据。</p>
                      <div>
                        <span>可尝试 {readiness().historical_outcome_forward_observation_execution_attempt_eligible_count}</span>
                        <span>已 claim {readiness().historical_outcome_forward_observation_execution_claim_count}</span>
                        <span>完成 {readiness().historical_outcome_forward_observation_execution_completed_count}</span>
                        <span>失败/中断 {readiness().historical_outcome_forward_observation_execution_failed_count + readiness().historical_outcome_forward_observation_execution_interrupted_count}</span>
                        <span>待独立验证 {readiness().historical_outcome_forward_observation_execution_independent_validation_eligible_count}</span>
                      </div>
                      <small>初始化状态：{readiness().historical_outcome_forward_observation_execution_status}；0 行行情、0 个前向交易日，没有持久 runtime、观察、账本、持仓、绩效、训练、奖励、订单、券商或交易能力。</small>
                    </article>
                    <article>
                      <header><strong>89 零行情初始化收据独立验证</strong><span>{readiness().historical_outcome_forward_observation_independently_validated_initialization_receipt_count > 0 ? "独立通过 · 等待首周期复核" : readiness().historical_outcome_forward_observation_output_validation_eligible_count > 0 ? "待责任链外验证" : "等待 Stage 88"}</span></header>
                      <p>责任链外新角色独立重建零行情 manifest 与预期收据，核对 claim-first、单一终态、自然前向、官方日历/SPY 和全部零权限位。</p>
                      <div>
                        <span>待验证 {readiness().historical_outcome_forward_observation_output_validation_eligible_count}</span>
                        <span>已验证 {readiness().historical_outcome_forward_observation_output_validation_count}</span>
                        <span>独立通过 {readiness().historical_outcome_forward_observation_independently_validated_initialization_receipt_count}</span>
                        <span>失败 {readiness().historical_outcome_forward_observation_failed_output_validation_count}</span>
                        <span>首周期复核资格 {readiness().historical_outcome_future_first_natural_forward_cycle_authorization_review_eligible_count}</span>
                      </div>
                      <small>验证状态：{readiness().historical_outcome_forward_observation_output_validation_status}；通过也不启动 runtime、不读取行情、不开始观察、不建账或交易。</small>
                    </article>
                    <article>
                      <header><strong>90 首个自然前向周期一次性授权</strong><span>{readiness().historical_outcome_first_natural_forward_cycle_claim_count > 0 ? "已由 Stage 91 永久消费" : readiness().historical_outcome_first_natural_forward_cycle_future_attempt_eligible_count > 0 ? "已批准 · 可领取 Stage 91 任务" : readiness().historical_outcome_first_natural_forward_cycle_authorization_review_eligible_count > 0 ? "待责任链外复核" : "等待 Stage 89"}</span></header>
                      <p>只对精确的 Stage 89 零行情初始化收据授权首个合格自然周期起算 7 天内最多一次的未来 claim-first 尝试；行情适配器仍需另行授权。</p>
                      <div>
                        <span>可复核 {readiness().historical_outcome_first_natural_forward_cycle_authorization_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_first_natural_forward_cycle_authorization_reviewed_count}</span>
                        <span>已批准 {readiness().historical_outcome_first_natural_forward_cycle_authorization_approved_count}</span>
                        <span>当前生效 {readiness().historical_outcome_first_natural_forward_cycle_authorization_active_count}</span>
                        <span>未来单次资格 {readiness().historical_outcome_first_natural_forward_cycle_future_attempt_eligible_count}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_first_natural_forward_cycle_authorization_status}；当前没有日历/行情读取、runtime、观察、账本、持仓、绩效或交易能力。</small>
                    </article>
                    <article>
                      <header><strong>91 首个自然前向周期任务声明</strong><span>{readiness().historical_outcome_first_natural_forward_cycle_claim_count > 0 ? "授权已消费 · 等待行情适配器授权" : readiness().historical_outcome_first_natural_forward_cycle_claim_eligible_count > 0 ? "可领取不可执行任务" : "等待 Stage 90"}</span></header>
                      <div class="public-admin-decision-metrics compact">
                        <span>候选 {readiness().historical_outcome_first_natural_forward_cycle_claim_authorization_candidate_count}</span>
                        <span>可领取 {readiness().historical_outcome_first_natural_forward_cycle_claim_eligible_count}</span>
                        <span>已领取 {readiness().historical_outcome_first_natural_forward_cycle_claim_count}</span>
                        <span>已消费 {readiness().historical_outcome_first_natural_forward_cycle_authorization_consumed_count}</span>
                        <span>待适配器授权 {readiness().historical_outcome_first_natural_forward_cycle_waiting_for_market_data_adapter_authorization_count}</span>
                      </div>
                      <small>任务状态：{readiness().historical_outcome_first_natural_forward_cycle_claim_status}；claim 不解析日历、不读取行情，也不启动观察或交易。</small>
                    </article>
                    <article>
                      <header><strong>92 只读行情适配器独立授权</strong><span>{readiness().historical_outcome_future_claim_first_read_only_market_data_receipt_eligible_count > 0 ? "合同已批准 · 等待另行领取数据收据" : readiness().historical_outcome_market_data_adapter_authorization_review_eligible_count > 0 ? "待责任链外复核" : readiness().historical_outcome_market_data_adapter_authorization_reviewed_count > 0 ? "已复核 · 无可用授权" : "等待 Stage 91"}</span></header>
                      <p>只复核固定 GET/HTTPS 来源白名单、数据类别、凭据隔离和内容寻址约束；批准也不会在本阶段解析日历、请求或读取行情。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>已领取任务 {readiness().historical_outcome_market_data_adapter_claimed_task_count}</span>
                        <span>可复核 {readiness().historical_outcome_market_data_adapter_authorization_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_market_data_adapter_authorization_reviewed_count}</span>
                        <span>批准/拒绝 {readiness().historical_outcome_market_data_adapter_authorization_approved_count}/{readiness().historical_outcome_market_data_adapter_authorization_rejected_count}</span>
                        <span>当前生效 {readiness().historical_outcome_market_data_adapter_active_authorization_count}</span>
                        <span>未来只读收据资格 {readiness().historical_outcome_future_claim_first_read_only_market_data_receipt_eligible_count}</span>
                      </div>
                      <small>适配器授权状态：{readiness().historical_outcome_market_data_adapter_authorization_status}；没有数据请求、runtime、观察、账本、持仓、绩效、模型/指标、订单、券商或交易能力。</small>
                    </article>
                    <article>
                      <header><strong>93 先声明再单次读取原始行情</strong><span>{readiness().historical_outcome_market_data_receipt_completed_untrusted_count > 0 ? "原始收据待独立验证" : readiness().historical_outcome_market_data_receipt_invocation_eligible_authorization_count > 0 ? "可声明并单次读取" : readiness().historical_outcome_market_data_receipt_claim_count > 0 ? "授权已消耗" : "等待 Stage 92"}</span></header>
                      <p>标的从已验证影子组合推导，时间窗从授权后的下一纽约自然日推导；先持久化 claim，再对标的、SPY 与 NYSE 日历执行固定一次 GET。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>可执行授权 {readiness().historical_outcome_market_data_receipt_invocation_eligible_authorization_count}</span>
                        <span>已声明 {readiness().historical_outcome_market_data_receipt_claim_count}</span>
                        <span>未信任收据 {readiness().historical_outcome_market_data_receipt_completed_untrusted_count}</span>
                        <span>失败 {readiness().historical_outcome_market_data_receipt_failed_consumed_count}</span>
                        <span>中断 {readiness().historical_outcome_market_data_receipt_interrupted_consumed_count}</span>
                        <span>待独立验证 {readiness().historical_outcome_market_data_receipt_independent_validation_eligible_count}</span>
                      </div>
                      <small>收据状态：{readiness().historical_outcome_market_data_receipt_status}；原始载荷不是交易日、观察、持仓、绩效或交易事实。</small>
                    </article>
                    <article>
                      <header><strong>94 原始行情收据责任链外独立验证</strong><span>{readiness().historical_outcome_market_data_receipt_independently_validated_count > 0 ? "完整性已验证 · 等待解析规格复核" : readiness().historical_outcome_market_data_receipt_validation_failed_count > 0 ? "验证失败终态" : readiness().historical_outcome_market_data_receipt_validation_pending_count > 0 ? "待责任链外验证" : "等待 Stage 93"}</span></header>
                      <p>独立重算 claim/result/receipt/规范请求和每份原始载荷的字节数、SHA-256 与保管路径；只验证完整性，不解释任何价格。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>完成收据 {readiness().historical_outcome_market_data_receipt_validation_completed_untrusted_count}</span>
                        <span>待验证 {readiness().historical_outcome_market_data_receipt_validation_pending_count}</span>
                        <span>已通过 {readiness().historical_outcome_market_data_receipt_independently_validated_count}</span>
                        <span>失败 {readiness().historical_outcome_market_data_receipt_validation_failed_count}</span>
                        <span>未来解析复核资格 {readiness().historical_outcome_future_market_data_parser_review_eligible_count}</span>
                      </div>
                      <small>验证状态：{readiness().historical_outcome_market_data_receipt_validation_status}；通过不等于行情语义、收益或模型有效。</small>
                    </article>
                    <article>
                      <header><strong>95 零能力行情解析器规格登记</strong><span>{readiness().historical_outcome_market_data_parser_spec_registered_count > 0 ? "规格已冻结 · 等待独立复核" : readiness().historical_outcome_market_data_parser_spec_registration_eligible_count > 0 ? "待登记规格" : "等待 Stage 94"}</span></header>
                      <p>冻结显式价格、原始价、分红调整价、分红、拆股和 NYSE 日历来源，以及严格 schema、失败关闭规则和合成测试向量。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>独立验证收据 {readiness().historical_outcome_market_data_parser_spec_independently_validated_receipt_count}</span>
                        <span>待登记 {readiness().historical_outcome_market_data_parser_spec_registration_eligible_count}</span>
                        <span>已登记 {readiness().historical_outcome_market_data_parser_spec_registered_count}</span>
                        <span>待规格独立复核 {readiness().historical_outcome_future_market_data_parser_spec_review_eligible_count}</span>
                      </div>
                      <small>规格状态：{readiness().historical_outcome_market_data_parser_spec_status}；没有 parser 实现、runtime、真实解析、观察或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>96 行情解析器规格责任链外独立复核</strong><span>{readiness().historical_outcome_market_data_parser_spec_independently_approved_count > 0 ? "独立通过 · 仅开放未来零能力实现登记" : readiness().historical_outcome_market_data_parser_spec_review_changes_required_or_rejected_count > 0 ? "需重建或已拒绝" : readiness().historical_outcome_market_data_parser_spec_review_eligible_count > 0 ? "待责任链外复核" : "等待 Stage 95"}</span></header>
                      <p>由第二套实现独立重建五类 FMP 请求、NYSE 交易日请求、Stage 95 规格哈希与八组合成向量，不读取原始载荷。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>已登记规格 {readiness().historical_outcome_market_data_parser_spec_review_registered_count}</span>
                        <span>待复核 {readiness().historical_outcome_market_data_parser_spec_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_market_data_parser_spec_reviewed_count}</span>
                        <span>独立通过 {readiness().historical_outcome_market_data_parser_spec_independently_approved_count}</span>
                        <span>未来零能力实现登记资格 {readiness().historical_outcome_future_zero_capability_market_data_parser_implementation_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_market_data_parser_spec_review_status}；没有 parser、原始载荷访问、行情行、观察或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>97 行情解析器零能力实现契约登记</strong><span>{readiness().historical_outcome_market_data_parser_implementation_current_binding_contract_count > 0 ? "契约已登记 · 等待 Stage 98 独立实现复核" : readiness().historical_outcome_market_data_parser_implementation_registration_eligible_count > 0 ? "待登记零能力实现契约" : "等待 Stage 96"}</span></header>
                      <p>逐哈希绑定独立批准规格、八组合成向量与纯函数标识；只冻结确定性语义，不上传源码或创建可执行入口。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>独立批准规格 {readiness().historical_outcome_market_data_parser_implementation_independently_approved_specification_count}</span>
                        <span>待登记 {readiness().historical_outcome_market_data_parser_implementation_registration_eligible_count}</span>
                        <span>契约 {readiness().historical_outcome_market_data_parser_implementation_contract_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_market_data_parser_implementation_current_binding_contract_count}</span>
                        <span>待 Stage 98 复核 {readiness().historical_outcome_market_data_parser_implementation_independent_review_eligible_count}</span>
                      </div>
                      <small>登记状态：{readiness().historical_outcome_market_data_parser_implementation_status}；没有工件、entrypoint、runtime、原始载荷读取、解析输出、观察或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>98 行情解析器实现责任链外独立复核</strong><span>{readiness().historical_outcome_market_data_parser_implementation_independently_approved_count > 0 ? "独立通过 · 仅开放 Stage 99 runner 规格登记" : readiness().historical_outcome_market_data_parser_implementation_review_changes_required_or_rejected_count > 0 ? "需重建或已拒绝" : readiness().historical_outcome_market_data_parser_implementation_review_eligible_count > 0 ? "待责任链外复核" : "等待 Stage 97"}</span></header>
                      <p>第二套实现独立重算 Stage 97 实现与契约、Stage 96 复核、Stage 95 登记和规格哈希，并复核八个纯函数与八组合成向量。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>实现契约 {readiness().historical_outcome_market_data_parser_implementation_review_implementation_count}</span>
                        <span>待复核 {readiness().historical_outcome_market_data_parser_implementation_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_market_data_parser_implementation_reviewed_count}</span>
                        <span>独立通过 {readiness().historical_outcome_market_data_parser_implementation_independently_approved_count}</span>
                        <span>Stage 99 runner 规格登记资格 {readiness().historical_outcome_future_isolated_market_data_parser_runner_specification_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_market_data_parser_implementation_review_status}；仍无 runner、runtime、原始载荷读取、解析输出、观察或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>99 行情解析器隔离 runner 规格登记</strong><span>{readiness().historical_outcome_market_data_parser_first_execution_authorization_review_eligible_count > 0 ? "规格已登记 · 仅开放 Stage 100 独立授权复核" : readiness().historical_outcome_market_data_parser_isolated_runner_registration_eligible_count > 0 ? "待登记零能力 runner 规格" : "等待 Stage 98"}</span></header>
                      <p>冻结未来工件身份、不可变代码版本、固定非特权 runtime、Stage 94 已验证只读输入和 create-once 不可信输出；不声称工件或 runtime 已存在。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>待登记 {readiness().historical_outcome_market_data_parser_isolated_runner_registration_eligible_count}</span>
                        <span>runner 规格 {readiness().historical_outcome_market_data_parser_isolated_runner_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_market_data_parser_isolated_runner_current_binding_count}</span>
                        <span>待 Stage 100 {readiness().historical_outcome_market_data_parser_first_execution_authorization_review_eligible_count}</span>
                      </div>
                      <small>登记状态：{readiness().historical_outcome_market_data_parser_isolated_runner_status}；没有源码、可执行工件、入口、runtime 实例、挂载、载荷读取、解析行或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>100 行情解析器首次执行授权独立复核</strong><span>{readiness().historical_outcome_market_data_parser_future_claim_first_attempt_eligible_count > 0 ? "已批准 · 仅开放 Stage 101 单次 claim" : readiness().historical_outcome_market_data_parser_reproduced_artifact_pending_runner_count > 0 ? "待服务端可核验工件" : readiness().historical_outcome_market_data_parser_first_execution_authorization_review_ready_count > 0 ? "待责任链外复核" : "等待 Stage 99"}</span></header>
                      <p>只有服务端在固定内容寻址目录中读取只读常规工件与自哈希 manifest，并自行重算 SHA-256、长度、代码版本和复现步骤后，才允许人工复核；手填相同摘要不能通过。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>runner {readiness().historical_outcome_market_data_parser_first_execution_authorization_runner_count}</span>
                        <span>待工件 {readiness().historical_outcome_market_data_parser_reproduced_artifact_pending_runner_count}</span>
                        <span>服务端已核验 {readiness().historical_outcome_market_data_parser_reproduced_artifact_verified_runner_count}</span>
                        <span>已复核 {readiness().historical_outcome_market_data_parser_first_execution_authorization_reviewed_count}</span>
                        <span>未过期单次授权 {readiness().historical_outcome_market_data_parser_first_execution_authorization_unexpired_count}</span>
                        <span>Stage 101 claim 候选 {readiness().historical_outcome_market_data_parser_future_claim_first_attempt_eligible_count}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_market_data_parser_first_execution_authorization_status}；即使批准也没有入口、runtime、挂载、载荷读取、parser 执行、解析行或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>101 行情解析器单次尝试 claim-first 声明</strong><span>{readiness().historical_outcome_market_data_parser_execution_attempt_claim_count > 0 ? "授权已永久消费 · 等待 Stage 102" : readiness().historical_outcome_market_data_parser_execution_attempt_claim_eligible_count > 0 ? "待先声明并消费授权" : "等待 Stage 100"}</span></header>
                      <p>在任何执行前冻结同一当前工件与精确 Stage 94 已验证输入清单，并以 create-once 记录永久消费 Stage 100 授权。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>授权候选 {readiness().historical_outcome_market_data_parser_execution_attempt_authorization_candidate_count}</span>
                        <span>可声明 {readiness().historical_outcome_market_data_parser_execution_attempt_claim_eligible_count}</span>
                        <span>已声明 {readiness().historical_outcome_market_data_parser_execution_attempt_claim_count}</span>
                        <span>已消费 {readiness().historical_outcome_market_data_parser_execution_attempt_authorization_consumed_count}</span>
                        <span>待 Stage 102 {readiness().historical_outcome_market_data_parser_execution_attempt_waiting_for_stage_102_count}</span>
                      </div>
                      <small>声明状态：{readiness().historical_outcome_market_data_parser_execution_attempt_claim_status}；只冻结既有元数据与摘要，不读取载荷、不执行 parser、不生成解析行。</small>
                    </article>
                    <article>
                      <header><strong>102 行情解析器单次受限执行</strong><span>{readiness().historical_outcome_market_data_parser_execution_successful_untrusted_output_count > 0 ? "已有非可信输出 · 等待 Stage 103" : readiness().historical_outcome_market_data_parser_execution_failed_consumed_claim_count > 0 ? "执行失败 · claim 已消费" : readiness().historical_outcome_market_data_parser_execution_pending_claim_count > 0 ? "待执行一次" : "等待 Stage 101"}</span></header>
                      <p>工件只作为声明式绑定，由 HONE 受信任进程内解析器读取并重哈希固定 Stage 94 载荷；不启动任意命令、脚本或二进制。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>待执行 {readiness().historical_outcome_market_data_parser_execution_pending_claim_count}</span>
                        <span>终态结果 {readiness().historical_outcome_market_data_parser_execution_terminal_result_count}</span>
                        <span>非可信输出 {readiness().historical_outcome_market_data_parser_execution_successful_untrusted_output_count}</span>
                        <span>失败已消费 {readiness().historical_outcome_market_data_parser_execution_failed_consumed_claim_count}</span>
                      </div>
                      <small>成功仍须 Stage 103 独立校验；失败不可重试原 claim。当前没有观察、账本、持仓、训练、订单、券商或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>103 行情解析器输出责任链外独立校验</strong><span>{readiness().historical_outcome_market_data_parser_output_independently_validated_count > 0 ? "独立重解析一致 · 等待 Stage 104" : readiness().historical_outcome_market_data_parser_output_validation_failed_count > 0 ? "校验失败 · 输出关闭" : readiness().historical_outcome_market_data_parser_output_validation_eligible_count > 0 ? "待第二实现独立校验" : "等待 Stage 102"}</span></header>
                      <p>由不同责任人使用不调用 Stage 102 解析助手的第二套实现，重新打开固定原始载荷、逐行重解析并与完整非可信输出精确比对。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>待校验 {readiness().historical_outcome_market_data_parser_output_validation_eligible_count}</span>
                        <span>终态校验 {readiness().historical_outcome_market_data_parser_output_validation_count}</span>
                        <span>独立一致 {readiness().historical_outcome_market_data_parser_output_independently_validated_count}</span>
                        <span>失败关闭 {readiness().historical_outcome_market_data_parser_output_validation_failed_count}</span>
                        <span>Stage 104 候选 {readiness().historical_outcome_market_data_parser_future_observation_input_admission_review_eligible_count}</span>
                      </div>
                      <small>校验状态：{readiness().historical_outcome_market_data_parser_output_validation_status}；source_available_at 仍未验证，尚无观察、账本、持仓、绩效、训练、订单、券商或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>104 首次自然前向周期观察输入独立准入</strong><span>{readiness().historical_outcome_observation_input_admitted_count > 0 ? "已准入 · 等待 Stage 105" : readiness().historical_outcome_observation_input_admission_changes_requested_or_rejected_count > 0 ? "要求修改或拒绝" : readiness().historical_outcome_observation_input_admission_review_eligible_count > 0 ? "待责任链外复核" : "等待 Stage 103"}</span></header>
                      <p>不声称未知的供应商发布时间；以 HONE 保管取得、解析完成、独立校验和复核提交时间的最大值作为保守 available_at。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>候选 {readiness().historical_outcome_observation_input_admission_candidate_count}</span>
                        <span>待复核 {readiness().historical_outcome_observation_input_admission_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_observation_input_admission_reviewed_count}</span>
                        <span>已准入 {readiness().historical_outcome_observation_input_admitted_count}</span>
                        <span>Stage 105 候选 {readiness().historical_outcome_future_observation_materialization_specification_registration_eligible_count}</span>
                      </div>
                      <small>准入状态：{readiness().historical_outcome_observation_input_admission_status}；批准也不生成观察、账本、持仓、绩效、训练、订单、券商或交易能力。</small>
                    </article>
                    <article>
                      <header><strong>105 首次自然前向周期观察物化规格</strong><span>{readiness().historical_outcome_observation_materialization_specification_registered_count > 0 ? "规格已登记 · 等待 Stage 106" : readiness().historical_outcome_observation_materialization_specification_registration_eligible_count > 0 ? "待登记零能力规格" : "等待 Stage 104"}</span></header>
                      <p>冻结已准入输出的逐日股票/SPY 三价格口径、显式缺口、公司行动、原始十进制和摘要规则；只引用初始影子组合，不重算组合或执行会计转换。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>已准入输入 {readiness().historical_outcome_observation_materialization_specification_admitted_input_count}</span>
                        <span>待登记 {readiness().historical_outcome_observation_materialization_specification_registration_eligible_count}</span>
                        <span>已登记 {readiness().historical_outcome_observation_materialization_specification_registered_count}</span>
                        <span>Stage 106 候选 {readiness().historical_outcome_observation_materialization_specification_future_independent_review_eligible_count}</span>
                      </div>
                      <small>规格状态：{readiness().historical_outcome_observation_materialization_specification_status}；没有实现、工件、入口、runtime、观察、账本、持仓、绩效或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>106 观察物化规格责任链外独立复核</strong><span>{readiness().historical_outcome_observation_materialization_specification_independently_approved_count > 0 ? "已独立批准 · 仅开放 Stage 107" : readiness().historical_outcome_observation_materialization_specification_review_eligible_count > 0 ? "待第二实现复核" : "等待 Stage 105"}</span></header>
                      <p>从当前 Stage 104 准入源重新构建整份 Stage 105 规格，独立核对摘要、交易日、三价格口径、显式缺口、公司行动、初始组合绑定、时间限制和零权限边界。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>规格 {readiness().historical_outcome_observation_materialization_specification_review_specification_count}</span>
                        <span>待复核 {readiness().historical_outcome_observation_materialization_specification_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_observation_materialization_specification_reviewed_count}</span>
                        <span>独立批准 {readiness().historical_outcome_observation_materialization_specification_independently_approved_count}</span>
                        <span>Stage 107 候选 {readiness().historical_outcome_future_zero_capability_observation_materialization_implementation_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_observation_materialization_specification_review_status}；即使批准，也没有实现、runtime、观察、绩效或交易能力。</small>
                    </article>
                    <article>
                      <header><strong>107 观察物化零能力实现契约</strong><span>{readiness().historical_outcome_observation_materialization_implementation_contract_count > 0 ? "契约已登记 · 等待 Stage 108" : readiness().historical_outcome_observation_materialization_implementation_registration_eligible_count > 0 ? "待 create-once 登记" : "等待 Stage 106"}</span></header>
                      <p>把独立批准的规格冻结为确定性纯函数、schema、路径、哈希与失败关闭边界；不提交源码或可执行制品，也不挂载或读取输入。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>独立批准规格 {readiness().historical_outcome_observation_materialization_implementation_approved_specification_count}</span>
                        <span>待登记 {readiness().historical_outcome_observation_materialization_implementation_registration_eligible_count}</span>
                        <span>契约 {readiness().historical_outcome_observation_materialization_implementation_contract_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_observation_materialization_implementation_current_binding_count}</span>
                        <span>Stage 108 候选 {readiness().historical_outcome_observation_materialization_implementation_independent_review_eligible_count}</span>
                      </div>
                      <small>实现状态：{readiness().historical_outcome_observation_materialization_implementation_status}；没有源代码工件、入口、runtime、观察、账本、持仓、绩效或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>108 观察物化实现责任链外独立复核</strong><span>{readiness().historical_outcome_observation_materialization_implementation_review_independently_approved_count > 0 ? "已独立批准 · 仅开放 Stage 109" : readiness().historical_outcome_observation_materialization_implementation_review_eligible_count > 0 ? "待第二实现复核" : "等待 Stage 107"}</span></header>
                      <p>由 Stage 51–107 完整责任链外的新角色独立重算实现、契约、复核、审计、登记与规格哈希，并复核八个纯函数和全部失败关闭边界。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>契约 {readiness().historical_outcome_observation_materialization_implementation_review_implementation_count}</span>
                        <span>待复核 {readiness().historical_outcome_observation_materialization_implementation_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_observation_materialization_implementation_reviewed_count}</span>
                        <span>独立批准 {readiness().historical_outcome_observation_materialization_implementation_review_independently_approved_count}</span>
                        <span>Stage 109 候选 {readiness().historical_outcome_future_isolated_observation_materialization_runner_specification_registration_eligible_count}</span>
                      </div>
                      <small>复核状态：{readiness().historical_outcome_observation_materialization_implementation_review_status}；批准也不产生 runner、输入读取、观察、账本、持仓、绩效或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>109 观察物化隔离 runner 规格登记</strong><span>{readiness().historical_outcome_observation_materialization_isolated_runner_count > 0 ? "规格已登记 · 仅开放 Stage 110" : readiness().historical_outcome_observation_materialization_isolated_runner_registration_eligible_count > 0 ? "待 create-once 登记" : "等待 Stage 108"}</span></header>
                      <p>把精确 Stage 108 独立批准绑定到拟议工件摘要、不可变代码版本、固定非特权 runtime、Stage 104 准入输入、create-once 不可信输出和静态资源上限。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>待登记 {readiness().historical_outcome_observation_materialization_isolated_runner_registration_eligible_count}</span>
                        <span>已登记 {readiness().historical_outcome_observation_materialization_isolated_runner_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_observation_materialization_isolated_runner_current_binding_count}</span>
                        <span>Stage 110 候选 {readiness().historical_outcome_observation_materialization_isolated_runner_first_execution_authorization_review_eligible_count}</span>
                      </div>
                      <small>runner 状态：{readiness().historical_outcome_observation_materialization_isolated_runner_status}；拟议工件不存在，没有入口、runtime、输入读取、观察、账本、持仓、绩效或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>110 观察物化首次执行授权独立复核</strong><span>{readiness().historical_outcome_observation_materialization_future_claim_first_attempt_eligible_count > 0 ? "24 小时单次资格 · 未 claim" : readiness().historical_outcome_observation_materialization_reproduced_artifact_verified_runner_count > 0 ? "等待链外独立复核" : "等待只读复现工件"}</span></header>
                      <p>服务端从内容寻址保管目录只读重算 runner 工件与自哈希 manifest，由 Stage 51–109 完整责任链外的新角色复核；批准最多开放一次、24 小时有效的未来 Stage 111 claim-first 候选。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>runner {readiness().historical_outcome_observation_materialization_first_execution_authorization_runner_count}</span>
                        <span>工件已核验 {readiness().historical_outcome_observation_materialization_reproduced_artifact_verified_runner_count}</span>
                        <span>待复现 {readiness().historical_outcome_observation_materialization_reproduced_artifact_pending_runner_count}</span>
                        <span>已复核 {readiness().historical_outcome_observation_materialization_first_execution_authorization_reviewed_count}</span>
                        <span>Stage 111 候选 {readiness().historical_outcome_observation_materialization_future_claim_first_attempt_eligible_count}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_observation_materialization_first_execution_authorization_status}；本阶段不 claim、不启动 runtime、不读取 Stage 104 输入，也不产生观察、账本、持仓、绩效或交易。</small>
                    </article>
                    <article>
                      <header><strong>111 观察物化单次尝试 claim-first 声明</strong><span>{readiness().historical_outcome_observation_materialization_execution_attempt_claim_count > 0 ? "授权已消费 · 等待 Stage 112" : readiness().historical_outcome_observation_materialization_execution_attempt_claim_eligible_count > 0 ? "可声明 · 尚未消费" : "等待 Stage 110"}</span></header>
                      <p>在任何入口、runtime 或 Stage 104 输入读取出现前，用 create-once 元数据记录永久消费一条 Stage 110 授权；声明失败或未来执行失败都不会返还授权。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>候选 {readiness().historical_outcome_observation_materialization_execution_attempt_authorization_candidate_count}</span>
                        <span>可声明 {readiness().historical_outcome_observation_materialization_execution_attempt_claim_eligible_count}</span>
                        <span>已声明 {readiness().historical_outcome_observation_materialization_execution_attempt_claim_count}</span>
                        <span>已消费 {readiness().historical_outcome_observation_materialization_execution_attempt_authorization_consumed_count}</span>
                        <span>待 Stage 112 {readiness().historical_outcome_observation_materialization_waiting_for_stage_112_execution_count}</span>
                      </div>
                      <small>声明状态：{readiness().historical_outcome_observation_materialization_execution_attempt_claim_status}；本阶段没有执行入口、输入读取、观察输出、账本、持仓、绩效或交易。</small>
                    </article>
                    <article>
                      <header><strong>112 自然前瞻观察单次物化</strong><span>{readiness().historical_outcome_observation_materialization_execution_successful_untrusted_observation_count > 0 ? "非可信观察已生成 · 等待 Stage 113" : readiness().historical_outcome_observation_materialization_execution_failed_consumed_claim_count > 0 ? "失败且已消费 · 不可重试" : readiness().historical_outcome_observation_materialization_execution_pending_claim_count > 0 ? "可执行一次" : "等待 Stage 111"}</span></header>
                      <p>先落 start marker 永久消费声明，再重哈希声明式工件与 exact Stage 104 admitted output，由受信任进程内函数确定性投影交易日、三价格口径、显式缺口、公司行动和初始分配绑定。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>待执行 {readiness().historical_outcome_observation_materialization_execution_pending_claim_count}</span>
                        <span>终态 {readiness().historical_outcome_observation_materialization_execution_terminal_result_count}</span>
                        <span>非可信观察 {readiness().historical_outcome_observation_materialization_execution_successful_untrusted_observation_count}</span>
                        <span>失败已消费 {readiness().historical_outcome_observation_materialization_execution_failed_consumed_claim_count}</span>
                        <span>待 Stage 113 {readiness().historical_outcome_observation_materialization_waiting_for_stage_113_validation_count}</span>
                      </div>
                      <small>执行状态：{readiness().historical_outcome_observation_materialization_execution_status}；当前没有账本、持仓、绩效、模型/训练、奖励、订单、券商或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>113 观察物化输出责任链外独立校验</strong><span>{readiness().historical_outcome_observation_materialization_independently_validated_observation_count > 0 ? "精确一致 · 等待 Stage 114" : readiness().historical_outcome_observation_materialization_output_validation_failed_count > 0 ? "校验失败 · 永久关闭" : readiness().historical_outcome_observation_materialization_output_validation_eligible_count > 0 ? "待独立第二投影" : "等待 Stage 112"}</span></header>
                      <p>责任链外验证者重新打开 exact Stage 112 create-once 输出和 Stage 104 准入输入，不调用 Stage 112 materializer helper，独立重投影 sessions、三价格口径、显式缺口、公司行动、初始分配、available-at、行哈希、排序和完整 envelope。</p>
                      <div class="public-admin-decision-metrics compact">
                        <span>待校验 {readiness().historical_outcome_observation_materialization_output_validation_eligible_count}</span>
                        <span>校验记录 {readiness().historical_outcome_observation_materialization_output_validation_count}</span>
                        <span>精确一致 {readiness().historical_outcome_observation_materialization_independently_validated_observation_count}</span>
                        <span>失败关闭 {readiness().historical_outcome_observation_materialization_output_validation_failed_count}</span>
                        <span>Stage 114 候选 {readiness().historical_outcome_observation_materialization_future_stage_114_observation_evidence_admission_review_eligible_count}</span>
                      </div>
                      <small>校验状态：{readiness().historical_outcome_observation_materialization_output_validation_status}；通过只开放 Stage 114 证据准入复核，仍没有账本、持仓、绩效、模型/训练、奖励、订单、券商或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>114 观察证据责任链外独立准入</strong><span>{readiness().historical_outcome_observation_ledger_transition_specification_registered_count > 0 ? "已进入 Stage 115" : readiness().historical_outcome_observation_evidence_admitted_count > 0 ? "正式证据 · 等待 Stage 115" : readiness().historical_outcome_observation_evidence_changes_requested_or_rejected_count > 0 ? "退回或拒绝" : readiness().historical_outcome_observation_evidence_admission_review_eligible_count > 0 ? "待独立复核" : "等待 Stage 113"}</span></header>
                      <p class="public-admin-anchor-meta">
                        <span>独立验证候选 {readiness().historical_outcome_observation_evidence_independently_validated_candidate_count}</span>
                        <span>待复核 {readiness().historical_outcome_observation_evidence_admission_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_observation_evidence_admission_reviewed_count}</span>
                        <span>正式证据 {readiness().historical_outcome_observation_evidence_admitted_count}</span>
                        <span>Stage 115 候选 {readiness().historical_outcome_observation_evidence_future_stage_115_ledger_transition_specification_registration_eligible_count}</span>
                      </p>
                      <small>准入状态：{readiness().historical_outcome_observation_evidence_admission_status}；原 envelope 不改写，供应商发布时间仍未验证。批准只开放 Stage 115 账本转换规格登记，不建账、不算净值/绩效、不训练/RL、不交易。</small>
                    </article>
                    <article>
                      <header><strong>115 观察证据到账本转换规格登记</strong><span>{readiness().historical_outcome_observation_ledger_transition_specification_reviewed_count > 0 ? "已进入 Stage 116" : readiness().historical_outcome_observation_ledger_transition_specification_registered_count > 0 ? "规格已冻结 · 等待 Stage 116" : readiness().historical_outcome_observation_ledger_transition_specification_registration_eligible_count > 0 ? "待登记" : "等待 Stage 114"}</span></header>
                      <p>只冻结 future append-only event stream 的可复算会计语义：Stage 88 不是 opening positions；不得默认本金、现金、持仓或股数。未来持仓 mark 只用 raw close，复权价不入会计，显式 gap 阻断 NAV，公司行动缺条款或持仓时只记 notice。</p>
                      <p class="public-admin-anchor-meta">
                        <span>正式证据 {readiness().historical_outcome_observation_ledger_transition_specification_admitted_evidence_count}</span>
                        <span>待登记 {readiness().historical_outcome_observation_ledger_transition_specification_registration_eligible_count}</span>
                        <span>已登记 {readiness().historical_outcome_observation_ledger_transition_specification_registered_count}</span>
                        <span>Stage 116 候选 {readiness().historical_outcome_observation_ledger_transition_specification_future_stage_116_independent_review_eligible_count}</span>
                        <span>缺 opening snapshot {readiness().historical_outcome_observation_ledger_transition_specification_opening_portfolio_snapshot_missing_count}</span>
                      </p>
                      <small>登记状态：{readiness().historical_outcome_observation_ledger_transition_specification_status}；当前没有实现、账本事件、持仓、现金、净值/绩效、训练/RL、订单、券商或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>116 账本转换规格责任链外独立复核</strong><span>{readiness().historical_outcome_observation_ledger_transition_specification_independently_approved_count > 0 ? "独立批准 · 仅开放 Stage 117" : readiness().historical_outcome_observation_ledger_transition_specification_changes_required_or_rejected_count > 0 ? "要求重建或拒绝" : readiness().historical_outcome_observation_ledger_transition_specification_review_eligible_count > 0 ? "待独立复核" : "等待 Stage 115"}</span></header>
                      <p>第二套实现不调用 Stage 115 builder，从 exact Stage 114 正式证据完整重建规格并逐字段比对；再次核对 Stage 88/opening、raw 与 adjusted 价格、显式 gap、公司行动防双计、十进制、幂等、修正与双分录约束。</p>
                      <p class="public-admin-anchor-meta">
                        <span>规格 {readiness().historical_outcome_observation_ledger_transition_specification_review_specification_count}</span>
                        <span>待复核 {readiness().historical_outcome_observation_ledger_transition_specification_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_observation_ledger_transition_specification_reviewed_count}</span>
                        <span>独立批准 {readiness().historical_outcome_observation_ledger_transition_specification_independently_approved_count}</span>
                        <span>Stage 117 候选 {readiness().historical_outcome_observation_ledger_transition_specification_future_stage_117_zero_capability_implementation_registration_eligible_count}</span>
                        <span>缺 opening snapshot {readiness().historical_outcome_observation_ledger_transition_specification_review_opening_portfolio_snapshot_missing_count}</span>
                      </p>
                      <small>复核状态：{readiness().historical_outcome_observation_ledger_transition_specification_review_status}；批准也不创建实现、账本事件、持仓、现金、NAV/绩效，不训练/RL，不生成订单，不接券商，不交易。</small>
                    </article>
                    <article>
                      <header><strong>117 账本转换零能力实现合同</strong><span>{readiness().historical_outcome_observation_ledger_transition_implementation_current_binding_count > 0 ? "合同已冻结 · 仅开放 Stage 118" : readiness().historical_outcome_observation_ledger_transition_implementation_registration_eligible_count > 0 ? "待登记" : "等待 Stage 116"}</span></header>
                      <p>create-once 自哈希合同只冻结 opening portfolio 门槛、raw/adjusted 价格隔离、gap 阻断 NAV、公司行动 notice 门禁、exact decimal、append-only、幂等事件、双重记账、可用时间和更正语义。</p>
                      <p class="public-admin-anchor-meta">
                        <span>独立批准规格 {readiness().historical_outcome_observation_ledger_transition_implementation_independently_approved_specification_count}</span>
                        <span>待登记 {readiness().historical_outcome_observation_ledger_transition_implementation_registration_eligible_count}</span>
                        <span>合同 {readiness().historical_outcome_observation_ledger_transition_implementation_contract_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_observation_ledger_transition_implementation_current_binding_count}</span>
                        <span>Stage 118 候选 {readiness().historical_outcome_observation_ledger_transition_implementation_future_stage_118_independent_review_eligible_count}</span>
                        <span>缺 opening snapshot {readiness().historical_outcome_observation_ledger_transition_implementation_opening_portfolio_snapshot_missing_count}</span>
                      </p>
                      <small>登记状态：{readiness().historical_outcome_observation_ledger_transition_implementation_status}；没有源码、入口、runtime、输入读取、账本/事件、持仓、现金、NAV/绩效、训练/RL、订单、券商或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>118 账本转换实现责任链外独立复核</strong><span>{readiness().historical_outcome_observation_ledger_transition_implementation_independently_approved_count > 0 ? "独立批准 · 仅开放 Stage 119" : readiness().historical_outcome_observation_ledger_transition_implementation_changes_required_or_rejected_count > 0 ? "要求重建或拒绝" : readiness().historical_outcome_observation_ledger_transition_implementation_review_eligible_count > 0 ? "待独立复核" : "等待 Stage 117"}</span></header>
                      <p>第二套实现不调用 Stage 117 builder，独立重建完整合同并逐字段核对 opening portfolio 前置门槛、会计价格、gap、公司行动、精确十进制、幂等、双分录、纠错、available-at 与全部零权限位。</p>
                      <p class="public-admin-anchor-meta">
                        <span>实现 {readiness().historical_outcome_observation_ledger_transition_implementation_review_implementation_count}</span>
                        <span>待复核 {readiness().historical_outcome_observation_ledger_transition_implementation_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_observation_ledger_transition_implementation_reviewed_count}</span>
                        <span>独立批准 {readiness().historical_outcome_observation_ledger_transition_implementation_independently_approved_count}</span>
                        <span>Stage 119 候选 {readiness().historical_outcome_observation_ledger_transition_implementation_future_stage_119_isolated_runner_specification_registration_eligible_count}</span>
                      </p>
                      <small>复核状态：{readiness().historical_outcome_observation_ledger_transition_implementation_review_status}；批准仍不产生工件、opening snapshot、账本事件、持仓、现金、NAV/绩效、训练/RL/reward、订单、券商或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>119 账本转换隔离 runner 规格登记</strong><span>{readiness().historical_outcome_observation_ledger_transition_isolated_runner_current_binding_count > 0 ? "规格已冻结 · 仅开放 Stage 120" : readiness().historical_outcome_observation_ledger_transition_isolated_runner_registration_eligible_count > 0 ? "待登记" : "等待 Stage 118"}</span></header>
                      <p>只冻结未来工件哈希、不可变代码版本、复现步骤、固定非特权 runtime、Stage 114 精确只读输入、create-once 不可信候选输出与资源上限。</p>
                      <p class="public-admin-anchor-meta">
                        <span>待登记 {readiness().historical_outcome_observation_ledger_transition_isolated_runner_registration_eligible_count}</span>
                        <span>已登记 {readiness().historical_outcome_observation_ledger_transition_isolated_runner_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_observation_ledger_transition_isolated_runner_current_binding_count}</span>
                        <span>Stage 120 候选 {readiness().historical_outcome_observation_ledger_transition_isolated_runner_future_stage_120_first_execution_authorization_review_eligible_count}</span>
                      </p>
                      <small>登记状态：{readiness().historical_outcome_observation_ledger_transition_isolated_runner_status}；期初组合不存在，金融事件白名单为空；无执行、账本/事件、持仓、现金、NAV/绩效、模型/训练/RL、订单、券商或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>120 账本转换首次执行授权独立复核</strong><span>{readiness().historical_outcome_observation_ledger_transition_first_execution_authorization_future_stage_121_claim_first_attempt_eligible_count > 0 ? "限时一次 · 仅开放 Stage 121 claim" : readiness().historical_outcome_observation_ledger_transition_first_execution_authorization_artifact_pending_runner_count > 0 ? "等待真实工件" : readiness().historical_outcome_observation_ledger_transition_first_execution_authorization_review_eligible_runner_count > 0 ? "待责任链外复核" : "等待 Stage 119"}</span></header>
                      <p>服务端只接受内容寻址保管区中的只读常规工件和自哈希 manifest，并自行重算摘要与长度；批准有效期 24 小时且只能使用一次。</p>
                      <p class="public-admin-anchor-meta">
                        <span>runner {readiness().historical_outcome_observation_ledger_transition_first_execution_authorization_runner_count}</span>
                        <span>待工件 {readiness().historical_outcome_observation_ledger_transition_first_execution_authorization_artifact_pending_runner_count}</span>
                        <span>已核验 {readiness().historical_outcome_observation_ledger_transition_first_execution_authorization_artifact_verified_runner_count}</span>
                        <span>已复核 {readiness().historical_outcome_observation_ledger_transition_first_execution_authorization_reviewed_runner_count}</span>
                        <span>已批准 {readiness().historical_outcome_observation_ledger_transition_first_execution_authorization_approved_runner_count}</span>
                        <span>Stage 121 候选 {readiness().historical_outcome_observation_ledger_transition_first_execution_authorization_future_stage_121_claim_first_attempt_eligible_count}</span>
                      </p>
                      <small>授权状态：{readiness().historical_outcome_observation_ledger_transition_first_execution_authorization_status}；opening snapshot 仍缺失，金融事件白名单仍为空，未来最多只允许非金融通知候选，不产生权威持仓、现金、NAV/绩效或交易状态。</small>
                    </article>
                    <article>
                      <header><strong>121 账本转换执行尝试原子认领</strong><span>{readiness().historical_outcome_observation_ledger_transition_execution_attempt_claim_waiting_for_stage_122_execution_count > 0 ? "授权已消费 · 等待 Stage 122" : readiness().historical_outcome_observation_ledger_transition_execution_attempt_claim_eligible_count > 0 ? "可认领 · 尚未执行" : "等待 Stage 120"}</span></header>
                      <p>在任何 runner 入口、runtime 或 Stage 114 已准入输出读取之前，create-once 自哈希记录永久消费一次性授权；认领不可撤销、重试、释放或恢复。</p>
                      <p class="public-admin-anchor-meta">
                        <span>候选 {readiness().historical_outcome_observation_ledger_transition_execution_attempt_claim_authorization_candidate_count}</span>
                        <span>可认领 {readiness().historical_outcome_observation_ledger_transition_execution_attempt_claim_eligible_count}</span>
                        <span>已认领 {readiness().historical_outcome_observation_ledger_transition_execution_attempt_claim_count}</span>
                        <span>已消费 {readiness().historical_outcome_observation_ledger_transition_execution_attempt_claim_authorization_consumed_count}</span>
                        <span>待 Stage 122 {readiness().historical_outcome_observation_ledger_transition_execution_attempt_claim_waiting_for_stage_122_execution_count}</span>
                      </p>
                      <small>认领状态：{readiness().historical_outcome_observation_ledger_transition_execution_attempt_claim_status}；未进入 Stage 122 前不执行工件、不读输入。认领后授权不可恢复、释放或重试。</small>
                    </article>
                    <article>
                      <header><strong>122 非财务观察通知单次转换</strong><span>{readiness().historical_outcome_observation_ledger_transition_execution_successful_untrusted_candidate_count > 0 ? "候选已生成 · 等待 Stage 123" : readiness().historical_outcome_observation_ledger_transition_execution_failed_consumed_claim_count > 0 ? "失败终态 · 不可重试" : readiness().historical_outcome_observation_ledger_transition_execution_pending_claim_count > 0 ? "待责任链外执行" : "等待 Stage 121"}</span></header>
                      <p>先落不可变 start marker，再重核声明式工件和 exact Stage 114 已准入证据；期初组合缺失时只投影非财务观察通知候选。</p>
                      <p class="public-admin-anchor-meta">
                        <span>待执行 {readiness().historical_outcome_observation_ledger_transition_execution_pending_claim_count}</span>
                        <span>终态 {readiness().historical_outcome_observation_ledger_transition_execution_terminal_result_count}</span>
                        <span>非可信候选 {readiness().historical_outcome_observation_ledger_transition_execution_successful_untrusted_candidate_count}</span>
                        <span>失败已消费 {readiness().historical_outcome_observation_ledger_transition_execution_failed_consumed_claim_count}</span>
                      </p>
                      <small>执行状态：{readiness().historical_outcome_observation_ledger_transition_execution_status}；不创建 ledger event、持仓、现金、NAV/绩效、模型/训练、订单、券商或交易状态。</small>
                    </article>
                    <article>
                      <header><strong>123 非财务候选责任链外独立校验</strong><span>{readiness().historical_outcome_observation_ledger_transition_independently_validated_candidate_count > 0 ? "精确一致 · 等待 Stage 124" : readiness().historical_outcome_observation_ledger_transition_output_validation_failed_count > 0 ? "校验失败 · 永久关闭" : readiness().historical_outcome_observation_ledger_transition_output_validation_eligible_count > 0 ? "待独立第二投影" : "等待 Stage 122"}</span></header>
                      <p>独立重开 Stage 122 候选和 exact Stage 114 观察证据，用第二实现重建每条通知、精确十进制、摘要、规范排序和完整候选。</p>
                      <p class="public-admin-anchor-meta">
                        <span>待校验 {readiness().historical_outcome_observation_ledger_transition_output_validation_eligible_count}</span>
                        <span>校验记录 {readiness().historical_outcome_observation_ledger_transition_output_validation_count}</span>
                        <span>精确一致 {readiness().historical_outcome_observation_ledger_transition_independently_validated_candidate_count}</span>
                        <span>失败关闭 {readiness().historical_outcome_observation_ledger_transition_output_validation_failed_count}</span>
                        <span>Stage 124 候选 {readiness().historical_outcome_observation_ledger_transition_future_stage_124_admission_review_eligible_count}</span>
                      </p>
                      <small>校验状态：{readiness().historical_outcome_observation_ledger_transition_output_validation_status}；通过后仍未受信，只开放非财务准入复核，不产生账本、持仓、现金、NAV/绩效或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>124 正式非财务观察证据独立准入</strong><span>{readiness().historical_outcome_observation_ledger_transition_admitted_non_financial_observation_evidence_count > 0 ? "正式证据已准入 · 等待 Stage 125" : readiness().historical_outcome_observation_ledger_transition_candidate_admission_changes_requested_or_rejected_count > 0 ? "已退回或拒绝" : readiness().historical_outcome_observation_ledger_transition_candidate_admission_review_eligible_count > 0 ? "待责任链外复核" : "等待 Stage 123"}</span></header>
                      <p>新的责任链外管理员重开 Stage 123 终态和 exact Stage 122 candidate；批准只创建分离、自哈希、追加式正式非财务观察证据。</p>
                      <p class="public-admin-anchor-meta">
                        <span>待复核 {readiness().historical_outcome_observation_ledger_transition_candidate_admission_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_observation_ledger_transition_candidate_admission_reviewed_count}</span>
                        <span>正式证据 {readiness().historical_outcome_observation_ledger_transition_admitted_non_financial_observation_evidence_count}</span>
                        <span>退回/拒绝 {readiness().historical_outcome_observation_ledger_transition_candidate_admission_changes_requested_or_rejected_count}</span>
                        <span>Stage 125 候选 {readiness().historical_outcome_observation_ledger_transition_future_stage_125_opening_portfolio_snapshot_governance_specification_eligible_count}</span>
                      </p>
                      <small>准入状态：{readiness().historical_outcome_observation_ledger_transition_candidate_admission_status}；原 candidate 继续未受信且不可变。下一步只治理外部来源期初组合快照，不补造仓位、现金、NAV/绩效或交易记录。</small>
                    </article>
                    <article>
                      <header><strong>125 外部来源期初组合快照治理规格</strong><span>{readiness().historical_outcome_opening_portfolio_snapshot_governance_registered_specification_count > 0 ? "规格已冻结 · 等待 Stage 126" : readiness().historical_outcome_opening_portfolio_snapshot_governance_registration_eligible_count > 0 ? "待责任链外登记" : "等待 Stage 124"}</span></header>
                      <p>冻结券商、托管机构或已核验组合会计系统原始导出的来源契约，以及账户、现金、持仓、负债、未结算活动、证券身份、精确十进制与未来 NAV 的完整规则。</p>
                      <p class="public-admin-anchor-meta">
                        <span>Stage 124 正式证据 {readiness().historical_outcome_opening_portfolio_snapshot_governance_stage_124_admitted_evidence_count}</span>
                        <span>可登记 {readiness().historical_outcome_opening_portfolio_snapshot_governance_registration_eligible_count}</span>
                        <span>已登记规格 {readiness().historical_outcome_opening_portfolio_snapshot_governance_registered_specification_count}</span>
                        <span>Stage 126 候选 {readiness().historical_outcome_opening_portfolio_snapshot_governance_future_stage_126_independent_specification_review_eligible_count}</span>
                      </p>
                      <small>规格状态：{readiness().historical_outcome_opening_portfolio_snapshot_governance_registration_status}；当前不接收来源文件、不手填余额、不生成期初快照，不创建持仓、现金、账本、净值/绩效、训练、订单或交易状态。</small>
                    </article>
                    <article>
                      <header><strong>126 期初组合治理规格责任链外独立复核</strong><span>{readiness().historical_outcome_opening_portfolio_snapshot_governance_specification_independently_approved_count > 0 ? "独立批准 · 仅开放 Stage 127" : readiness().historical_outcome_opening_portfolio_snapshot_governance_specification_changes_requested_or_rejected_count > 0 ? "已退回或拒绝" : readiness().historical_outcome_opening_portfolio_snapshot_governance_specification_review_eligible_count > 0 ? "待第二实现复核" : "等待 Stage 125"}</span></header>
                      <p>第二实现不调用 Stage 125 构造器，独立重建并重哈希完整账户、现金、持仓、期权、负债、未结算活动、证券身份和独立估值前置门。</p>
                      <p class="public-admin-anchor-meta">
                        <span>待复核 {readiness().historical_outcome_opening_portfolio_snapshot_governance_specification_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_opening_portfolio_snapshot_governance_specification_reviewed_count}</span>
                        <span>独立批准 {readiness().historical_outcome_opening_portfolio_snapshot_governance_specification_independently_approved_count}</span>
                        <span>退回/拒绝 {readiness().historical_outcome_opening_portfolio_snapshot_governance_specification_changes_requested_or_rejected_count}</span>
                        <span>Stage 127 候选 {readiness().historical_outcome_opening_portfolio_snapshot_governance_future_stage_127_zero_capability_source_artifact_receipt_implementation_registration_eligible_count}</span>
                      </p>
                      <small>复核状态：{readiness().historical_outcome_opening_portfolio_snapshot_governance_specification_review_status}；批准也不接收或读取来源文件，不生成期初组合、持仓、现金、净值/绩效、训练、订单或交易状态。</small>
                    </article>
                    <article>
                      <header><strong>127 来源工件接收零能力实现登记</strong><span>{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_current_binding_count > 0 ? "合同已登记 · 等待 Stage 128" : readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_registration_eligible_count > 0 ? "待责任链外登记" : "等待 Stage 126"}</span></header>
                      <p>冻结未来私密接收器的流式摘要与长度、格式/魔数、主动内容拒绝、账号匿名化、日志脱敏、内容寻址、失败清理和未受信 receipt manifest。</p>
                      <p class="public-admin-anchor-meta">
                        <span>Stage 126 独立批准 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_independently_approved_specification_count}</span>
                        <span>可登记 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_registration_eligible_count}</span>
                        <span>已登记合同 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_contract_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_current_binding_count}</span>
                        <span>Stage 128 候选 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_future_stage_128_independent_implementation_review_eligible_count}</span>
                      </p>
                      <small>实现状态：{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_status}；本阶段没有上传入口、来源字节或 parser，不生成期初组合、账本、持仓、现金、净值/绩效、训练、订单或交易状态。</small>
                    </article>
                    <article>
                      <header><strong>128 来源工件接收实现责任链外独立复核</strong><span>{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_independently_approved_count > 0 ? "独立批准 · 仅开放 Stage 129" : readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_changes_required_or_rejected_count > 0 ? "已退回或拒绝" : readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_review_eligible_count > 0 ? "待第二实现复核" : "等待 Stage 127"}</span></header>
                      <p>第二实现不调用 Stage 127 builder，独立重建接收合同并重算完整上游摘要，复核格式、资源上限、私有隔离、流式哈希、主动内容拒绝、匿名化、内容寻址、失败清理与未受信 manifest。</p>
                      <p class="public-admin-anchor-meta">
                        <span>实现合同 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_review_implementation_count}</span>
                        <span>待复核 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_review_eligible_count}</span>
                        <span>已复核 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_reviewed_count}</span>
                        <span>独立批准 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_independently_approved_count}</span>
                        <span>退回/拒绝 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_changes_required_or_rejected_count}</span>
                        <span>Stage 129 候选 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_future_stage_129_isolated_receiver_specification_registration_eligible_count}</span>
                      </p>
                      <small>复核状态：{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_implementation_review_status}；批准也只允许登记隔离接收器规格，仍不得上传、读取或解析来源文件，不生成任何财务或交易状态。</small>
                    </article>
                    <article>
                      <header><strong>129 隔离来源工件接收器规格登记</strong><span>{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_current_binding_count > 0 ? "规格已登记 · 等待 Stage 130" : readiness().historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_registration_eligible_count > 0 ? "待责任链外登记" : "等待 Stage 128"}</span></header>
                      <div class="public-admin-readiness-numbers">
                        <span>可登记 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_registration_eligible_count}</span>
                        <span>已登记 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_current_binding_count}</span>
                        <span>Stage 130 候选 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_future_stage_130_first_execution_authorization_review_eligible_count}</span>
                      </div>
                      <small>规格状态：{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_status}；只冻结未来工件身份、非特权 runtime 与流式输入/未受信输出边界，当前无上传、来源字节、工件、入口、runtime 或财务状态。</small>
                    </article>
                    <article>
                      <header><strong>130 来源接收器首次执行授权</strong><span>{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_future_stage_131_claim_first_attempt_eligible_count > 0 ? "24 小时一次性授权 · 尚未 claim" : readiness().historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_artifact_verified_count > 0 ? "工件已核验 · 待独立复核" : "等待只读内容寻址工件"}</span></header>
                      <div class="public-admin-readiness-numbers">
                        <span>接收器 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_receiver_count}</span>
                        <span>待工件 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_artifact_pending_count}</span>
                        <span>服务端核验 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_artifact_verified_count}</span>
                        <span>已复核 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_reviewed_count}</span>
                        <span>Stage 131 候选 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_future_stage_131_claim_first_attempt_eligible_count}</span>
                      </div>
                      <small>授权状态：{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_status}；服务端重哈希只读工件与自哈希 manifest，授权仍不接收来源文件、不启动 runtime、不创建 receipt、期初组合或财务状态。</small>
                    </article>
                    <article>
                      <header><strong>131 来源接收尝试 claim-first</strong><span>{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_waiting_for_stage_132_count > 0 ? "授权已永久消费 · 等待 Stage 132" : readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_eligible_count > 0 ? "可领取 · 尚未接收" : "等待 Stage 130"}</span></header>
                      <div class="public-admin-readiness-numbers">
                        <span>候选 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_authorization_candidate_count}</span>
                        <span>可领取 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_eligible_count}</span>
                        <span>已领取 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_count}</span>
                        <span>已消费 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_authorization_consumed_count}</span>
                        <span>待 Stage 132 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_waiting_for_stage_132_count}</span>
                      </div>
                      <small>领取状态：{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_attempt_claim_status}；领取发生在任何上传流和来源字节之前，且不可重试、释放或恢复授权。</small>
                    </article>
                    <article>
                      <header><strong>132 来源工件单次加密接收</strong><span>{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_successful_untrusted_receipt_count > 0 ? "未受信 receipt · 等待 Stage 133" : readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_pending_claim_count > 0 ? readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_encryption_key_configured ? "可单次接收" : "等待加密密钥" : "等待 Stage 131"}</span></header>
                      <div class="public-admin-readiness-numbers">
                        <span>待接收 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_pending_claim_count}</span>
                        <span>终态 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_terminal_result_count}</span>
                        <span>未受信 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_successful_untrusted_receipt_count}</span>
                        <span>失败已消费 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_failed_consumed_claim_count}</span>
                        <span>密钥 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_encryption_key_configured ? "就绪" : "未配置"}</span>
                      </div>
                      <small>接收状态：{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_execution_status}；开始标记先于来源字节，原文件加密内容寻址保存。receipt 仍未受信，不是期初持仓，也不能创建账本或交易权限。</small>
                    </article>
                    <article>
                      <header><strong>133 加密 receipt 责任链外独立验证</strong><span>{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_validation_independently_validated_receipt_count > 0 ? "完整性已验证 · 仅开放 Stage 134" : readiness().historical_outcome_opening_portfolio_source_artifact_receipt_validation_failed_independent_validation_count > 0 ? "终态失败" : readiness().historical_outcome_opening_portfolio_source_artifact_receipt_validation_pending_independent_validation_count > 0 ? readiness().historical_outcome_opening_portfolio_source_artifact_receipt_validation_encryption_key_configured ? "待责任链外验证" : "等待同一加密密钥" : "等待 Stage 132"}</span></header>
                      <div>
                        <span>未受信 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_validation_completed_untrusted_receipt_count}</span>
                        <span>待验证 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_validation_pending_independent_validation_count}</span>
                        <span>通过 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_validation_independently_validated_receipt_count}</span>
                        <span>失败 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_validation_failed_independent_validation_count}</span>
                        <span>Stage 134 候选 {readiness().historical_outcome_opening_portfolio_source_artifact_receipt_validation_future_stage_134_eligible_count}</span>
                      </div>
                      <small>验证状态：{readiness().historical_outcome_opening_portfolio_source_artifact_receipt_validation_status}；只证明 result、receipt、密文、认证解密、明文哈希、格式和脱敏证据完整，不证明文件内持仓真实，也不解析金融行或生成财务、训练、订单与交易权限。</small>
                    </article>
                    <article>
                      <header><strong>134 期初快照物化零能力实现登记</strong><span>{readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_current_binding_count > 0 ? "合同已登记 · 等待 Stage 135" : readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_registration_eligible_count > 0 ? "可登记合同" : "等待 Stage 133"}</span></header>
                      <div>
                        <span>已验证 receipt {readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_independently_validated_receipt_count}</span>
                        <span>可登记 {readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_registration_eligible_count}</span>
                        <span>合同 {readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_contract_count}</span>
                        <span>当前绑定 {readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_current_binding_count}</span>
                        <span>Stage 135 候选 {readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_future_stage_135_independent_review_eligible_count}</span>
                      </div>
                      <small>登记状态：{readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_status}；只冻结完整账户、精确十进制、证券身份、公司行动和逐行来源合同，不解密、不解析、不生成候选或真实持仓。</small>
                    </article>
                    <article>
                      <header><strong>135 物化实现责任链外独立审查</strong><span>{readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_independently_approved_count > 0 ? "合同独立通过 · 仅开放 Stage 136" : readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_changes_required_or_rejected_count > 0 ? "终态未通过 · 需新 Stage 134" : readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_eligible_count > 0 ? "待第二实现审查" : "等待 Stage 134"}</span></header>
                      <div>
                        <span>实现 {readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_implementation_count}</span>
                        <span>待审 {readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_eligible_count}</span>
                        <span>已审 {readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_reviewed_count}</span>
                        <span>独立通过 {readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_independently_approved_count}</span>
                        <span>未通过 {readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_changes_required_or_rejected_count}</span>
                        <span>Stage 136 候选 {readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_future_stage_136_eligible_count}</span>
                      </div>
                      <small>审查状态：{readiness().historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_status}；第二实现独立重建并复核完整账户、精确十进制、证券身份和逐行来源合同。当前仍无 receipt 读取、解密、parser/runtime、期初快照、财务状态、训练、订单或交易权限。</small>
                    </article>
                  </div>
                  <strong class="public-admin-decision-evidence-tier">当前最小阻断项</strong>
                  <For each={readiness().blocking_reasons}>{(reason) => <small>{reason}</small>}</For>
                  <small>清单通过也只允许进入下一道独立人工评审；它不会自动运行训练、生成奖励、建立影子持仓或下单。</small>
                </section>
              )}
            </Show>
            <Show when={report().first_principles_hypothesis_map}>
              {(map) => (
                <section class="public-admin-decision-direction" aria-label="第一性原理产业假设地图">
                  <h3>第一性原理产业假设地图</h3>
                  <p>{map().scope}</p>
                  <div>
                    <span><strong>产业模型</strong> {map().model_count}</span>
                    <span><strong>最新公司状态</strong> {map().company_count}</span>
                    <span><strong>机会排名</strong> 关闭</span>
                    <span><strong>动作授权</strong> 未授权</span>
                  </div>
                  <div class="public-admin-decision-model-map">
                    <For each={map().models}>
                      {(model) => (
                        <article>
                          <header>
                            <strong>{FIRST_PRINCIPLES_MODEL_LABELS[model.model_id] ?? model.model_id}</strong>
                            <span>{FIRST_PRINCIPLES_STATE_LABELS[model.state] ?? model.state}</span>
                          </header>
                          <p>{model.symbols.join(" · ")}</p>
                          <strong class="public-admin-decision-evidence-tier">可追溯证据</strong>
                          <div>
                            <span>需求 {model.demand_traceable_company_count}/{model.company_count}</span>
                            <span>供给 {model.supply_traceable_company_count}/{model.company_count}</span>
                            <span>价值捕获 {model.value_capture_traceable_company_count}/{model.company_count}</span>
                            <span>三层齐全 {model.fully_traceable_company_count}/{model.company_count}</span>
                          </div>
                          <strong class="public-admin-decision-evidence-tier">严格量化证据</strong>
                          <div>
                            <span>需求 {model.demand_measured_company_count}/{model.company_count}</span>
                            <span>供给 {model.supply_measured_company_count}/{model.company_count}</span>
                            <span>价值捕获 {model.value_capture_measured_company_count}/{model.company_count}</span>
                            <span>三层齐全 {model.fully_measured_company_count}/{model.company_count}</span>
                          </div>
                          <small>
                            证据路径：直接指标 {model.evidence_pathway.direct_metric_count}；代理指标 {model.evidence_pathway.proxy_count}；
                            一手上下文 {model.evidence_pathway.confirmed_context_count}；结构化来源 {model.evidence_pathway.structured_source_claim_count}；
                            确定性计算 {model.evidence_pathway.computed_comparison_count + model.evidence_pathway.computed_ratio_count + model.evidence_pathway.computed_ratio_trend_count}；
                            经营 KPI {model.evidence_pathway.operating_kpi_claim_count}
                          </small>
                          <small>
                            人工晋级驱动 {model.promoted_driver_count}；冲突 {model.blocked_conflict_driver_count}；
                            否决 {model.blocked_rejection_driver_count}；证伪 {model.blocked_falsification_driver_count}
                          </small>
                          <small>{model.interpretation}</small>
                          <For each={model.missing_checks.slice(0, 4)}>{(check) => <small>{check}</small>}</For>
                        </article>
                      )}
                    </For>
                  </div>
                  <section class="public-admin-measurement-backlog" aria-label="第一性原理量化准入待办">
                    <header>
                      <div>
                        <h4>量化准入待办</h4>
                        <p>{map().measurement_backlog.scope}</p>
                      </div>
                      <strong>{map().measurement_backlog.measured_driver_count}/{map().measurement_backlog.total_driver_count} 个驱动已量化</strong>
                    </header>
                    <div class="public-admin-measurement-backlog-summary">
                      <span>可直接复核 {map().measurement_backlog.ready_for_review_count}</span>
                      <span>需新证据 {map().measurement_backlog.rejected_needs_new_evidence_count}</span>
                      <span>文字待指标化 {map().measurement_backlog.metricization_required_count}</span>
                      <span>待补经营指标 {map().measurement_backlog.operating_kpi_required_count}</span>
                      <span>尚无一手证据 {map().measurement_backlog.no_traceable_evidence_count}</span>
                    </div>
                    <div class="public-admin-measurement-backlog-list">
                      <For each={map().measurement_backlog.items.slice(0, 12)}>
                        {(item) => (
                          <article>
                            <header>
                              <strong>{item.symbol} · {item.driver_label}</strong>
                              <span>{MEASUREMENT_BACKLOG_STATUS_LABELS[item.status] ?? item.status}</span>
                            </header>
                            <p>{FIRST_PRINCIPLES_MODEL_LABELS[item.model_id] ?? item.model_id} · {DRIVER_FAMILY_LABELS[item.driver_family] ?? item.driver_family}</p>
                            <div>
                              <span>可追溯 {item.traceable_observation_count}</span>
                              <span>待复核 {item.pending_review_candidate_count}</span>
                            </div>
                            <small>{item.next_check}</small>
                            <small>应观察：{item.required_observations.slice(0, 3).join("；")}</small>
                            <Show when={item.target_operating_kpi_ids.length > 0}>
                              <small>优先经营指标：{item.target_operating_kpi_ids.join("；")}</small>
                            </Show>
                          </article>
                        )}
                      </For>
                    </div>
                    <small>仅展示前 12 个待办。可复核项目沿用下方“因果证据复核”的单条不可变审计；只有老王本人确认关系成立且明确支持或证伪后，才晋级为部分量化。</small>
                  </section>
                  <small>点时指纹 {map().map_fingerprint_sha256.slice(0, 16)}… · 只取每家公司最新决策，历史重复样本不增加权重。</small>
                </section>
              )}
            </Show>
            <Show when={report().shadow_policy}>
              {(policy) => (
                <div class="public-admin-decision-direction">
                  <h3>影子组合协议（尚未启动）</h3>
                  <p>{policy().scope}</p>
                  <div>
                    <span><strong>授权</strong> 未授权</span>
                    <span><strong>虚拟基准本金</strong> ${policy().constraints.virtual_notional_usd.toLocaleString()}</span>
                    <span><strong>单一公司上限</strong> {percent(policy().constraints.maximum_single_name_weight_percent)}</span>
                    <span><strong>主题上限</strong> {percent(policy().constraints.maximum_theme_weight_percent)}</span>
                    <span><strong>最低现金</strong> {percent(policy().constraints.minimum_cash_weight_percent)}</span>
                    <span><strong>模拟摩擦</strong> 单边 {policy().constraints.slippage_bps_per_side} 基点</span>
                  </div>
                  <small>{policy().constraints.rebalance_frequency}；{policy().constraints.execution_assumption}。</small>
                  <For each={policy().readiness_reasons}>{(reason) => <small>{reason}</small>}</For>
                  <Show when={policy().candidates.length > 0} fallback={<small>尚无点时决策样本可形成候选。</small>}>
                    <div>
                      <For each={policy().candidates.slice(0, 10)}>
                        {(candidate) => (
                          <span>
                            <strong>{candidate.symbol}</strong>{" "}
                            {candidate.status === "eligible_for_protocol_review"
                              ? `可送审 ${candidate.target_weight_min_percent}%–${candidate.target_weight_max_percent}%（非授权）`
                              : `阻断：${candidate.blocking_reasons.join("；")}`}
                          </span>
                        )}
                      </For>
                    </div>
                  </Show>
                  <Show when={shadowGovernance()}>
                    {(governance) => (
                      <section class="public-admin-reward-governance" aria-label="影子组合协议治理">
                        <header>
                          <strong>影子协议冻结与审批</strong>
                          <span>
                            {governance().latest_review
                              ? SHADOW_PROTOCOL_GOVERNANCE_LABELS[
                                  governance().latest_review!.verdict
                                ]
                              : "尚未审查"}
                          </span>
                        </header>
                        <p>{governance().scope}</p>
                        <div>
                          <For each={governance().review_requirements}>
                            {(requirement) => (
                              <span title={requirement.definition}>
                                <strong>{requirement.label}</strong>
                              </span>
                            )}
                          </For>
                        </div>
                        <Show when={governance().latest_review}>
                          {(review) => (
                            <blockquote>
                              {review().rationale}
                              <small>{dateTime(review().submitted_at)} · 不可覆盖记录</small>
                            </blockquote>
                          )}
                        </Show>
                        <label>
                          <span>本次意见（必填）</span>
                          <textarea
                            maxlength={10000}
                            value={shadowGovernanceRationale()}
                            onInput={(event) =>
                              setShadowGovernanceRationale(event.currentTarget.value)
                            }
                            placeholder="说明协议哪里需要修改、为什么拒绝，或为何允许进入未来实现登记。"
                          />
                        </label>
                        <label class="public-admin-reward-confirm">
                          <input
                            type="checkbox"
                            checked={shadowGovernanceConfirmed()}
                            onChange={(event) =>
                              setShadowGovernanceConfirmed(event.currentTarget.checked)
                            }
                          />
                          <span>我已逐项确认全部冻结约束；批准只允许未来另行登记实现，当前无账本、持仓、订单、券商或交易权限。</span>
                        </label>
                        <div class="public-admin-reward-actions">
                          <button
                            type="button"
                            disabled={
                              shadowGovernanceSubmitting()
                              || !shadowGovernanceRationale().trim()
                            }
                            onClick={() => void submitShadowGovernance("changes_requested")}
                          >
                            要求修改
                          </button>
                          <button
                            type="button"
                            disabled={
                              shadowGovernanceSubmitting()
                              || !shadowGovernanceRationale().trim()
                            }
                            onClick={() => void submitShadowGovernance("rejected")}
                          >
                            否决协议
                          </button>
                          <button
                            type="button"
                            class="is-primary"
                            title={
                              governance().evidence_gate_status
                                !== "eligible_for_reward_design_review"
                                ? "长期证据门槛尚未通过"
                                : governance().reward_governance_status
                                    !== "approved_for_offline_research"
                                  ? "当前奖励目标尚未批准"
                                  : "只批准未来影子实现登记"
                            }
                            disabled={
                              shadowGovernanceSubmitting()
                              || !shadowGovernanceRationale().trim()
                              || !shadowGovernanceConfirmed()
                              || governance().evidence_gate_status
                                !== "eligible_for_reward_design_review"
                              || governance().reward_governance_status
                                !== "approved_for_offline_research"
                            }
                            onClick={() =>
                              void submitShadowGovernance(
                                "approved_for_future_shadow_implementation",
                              )
                            }
                          >
                            批准未来实现登记
                          </button>
                        </div>
                        <small>
                          当前状态：影子账本关闭、组合未授权、券商未连接、交易未授权。
                        </small>
                      </section>
                    )}
                  </Show>
                  <Show when={shadowImplementations()}>
                    {(registry) => (
                      <section
                        class="public-admin-reward-governance"
                        aria-label="影子实现规范注册表"
                      >
                        <header>
                          <strong>影子实现规范注册表（未启动）</strong>
                          <span>{registry().registration_allowed ? "可登记规范" : "登记关闭"}</span>
                        </header>
                        <p>{registry().scope}</p>
                        <div class="public-admin-reward-actions">
                          <span>确定性重放</span>
                          <span>账本关闭</span>
                          <span>订单关闭</span>
                          <span>券商未连接</span>
                        </div>
                        <Show
                          when={registry().implementations.length > 0}
                          fallback={<small>尚无实现规范登记；当前不会创建任何影子账本。</small>}
                        >
                          <For each={registry().implementations}>
                            {(implementation) => (
                              <blockquote>
                                <strong>{implementation.implementation_name}</strong>
                                <small>
                                  代码版本 {implementation.code_revision} · 规范指纹 {implementation.implementation_spec_sha256.slice(0, 12)}… · {dateTime(implementation.registered_at)}
                                </small>
                              </blockquote>
                            )}
                          </For>
                        </Show>
                        <label>
                          <span>实现规范名称</span>
                          <input
                            maxlength={120}
                            value={shadowImplementationName()}
                            onInput={(event) => setShadowImplementationName(event.currentTarget.value)}
                            placeholder="例如：确定性影子重放规范"
                          />
                        </label>
                        <label>
                          <span>不可变代码版本</span>
                          <input
                            maxlength={200}
                            value={shadowImplementationRevision()}
                            onInput={(event) => setShadowImplementationRevision(event.currentTarget.value)}
                            placeholder="例如：oldwang@提交哈希"
                          />
                        </label>
                        <div class="public-admin-reward-actions">
                          <button
                            type="button"
                            class="is-primary"
                            title={registry().registration_allowed ? "只登记规范，不启动" : "上游长期证据与治理审批尚未全部通过"}
                            disabled={
                              shadowImplementationSubmitting()
                              || !registry().registration_allowed
                              || !shadowImplementationName().trim()
                              || !shadowImplementationRevision().trim()
                            }
                            onClick={() => void submitShadowImplementation()}
                          >
                            登记规范（不启动）
                          </button>
                        </div>
                        <small>
                          运行授权：关闭；影子组合：未授权；订单生成：关闭；真实交易：未授权。
                        </small>
                      </section>
                    )}
                  </Show>
                </div>
              )}
            </Show>
            <Show when={report().reward_design}>
              {(design) => (
                <div class="public-admin-decision-direction">
                  <h3>奖励与反事实评估提案（奖励关闭）</h3>
                  <p>{design().scope}</p>
                  <div>
                    <span><strong>审批</strong> 未批准</span>
                    <span><strong>奖励计算</strong> 关闭</span>
                    <span><strong>候选权重合计</strong> {percent(design().proposed_weight_total_percent)}</span>
                    <span><strong>市场环境</strong> {design().counterfactual_protocol.minimum_market_regimes} 类分别验证</span>
                    <span><strong>反事实基线</strong> {design().counterfactual_protocol.comparators.length} 组</span>
                  </div>
                  <small>一票否决：{design().hard_gates.map((gate) => gate.label).join("、")}。</small>
                  <div>
                    <For each={design().proposed_components}>
                      {(component) => (
                        <span title={`${component.measurement}；防止取巧：${component.anti_shortcut}`}>
                          <strong>{component.label}</strong> {percent(component.proposed_weight_percent)}
                        </span>
                      )}
                    </For>
                  </div>
                  <small>对照：{design().counterfactual_protocol.comparators.map((item) => item.label).join("、")}。</small>
                  <For each={design().readiness_reasons}>{(reason) => <small>{reason}</small>}</For>
                  <Show when={rewardGovernance()}>
                    {(governance) => (
                      <section class="public-admin-reward-governance" aria-label="奖励目标治理">
                        <header>
                          <strong>老王/治理责任人确认</strong>
                          <span>
                            {governance().latest_review
                              ? REWARD_GOVERNANCE_LABELS[governance().latest_review!.verdict]
                              : "尚未审查"}
                          </span>
                        </header>
                        <p>{governance().scope}</p>
                        <Show when={governance().latest_review}>
                          {(review) => (
                            <blockquote>
                              {review().rationale}
                              <small>{dateTime(review().submitted_at)} · 不可覆盖记录</small>
                            </blockquote>
                          )}
                        </Show>
                        <label>
                          <span>本次意见（必填）</span>
                          <textarea
                            maxlength={10000}
                            value={governanceRationale()}
                            onInput={(event) => setGovernanceRationale(event.currentTarget.value)}
                            placeholder="说明为什么接受、要求怎样修改，或哪项奖励可能诱导模型取巧。"
                          />
                        </label>
                        <label class="public-admin-reward-confirm">
                          <input
                            type="checkbox"
                            checked={governanceConfirmed()}
                            onChange={(event) => setGovernanceConfirmed(event.currentTarget.checked)}
                          />
                          <span>我已逐项确认六类候选权重、四项一票否决和反事实评测；批准仅限离线研究。</span>
                        </label>
                        <div class="public-admin-reward-actions">
                          <button
                            type="button"
                            disabled={governanceSubmitting() || !governanceRationale().trim()}
                            onClick={() => void submitRewardGovernance("changes_requested")}
                          >
                            要求修改
                          </button>
                          <button
                            type="button"
                            disabled={governanceSubmitting() || !governanceRationale().trim()}
                            onClick={() => void submitRewardGovernance("rejected")}
                          >
                            否决提案
                          </button>
                          <button
                            type="button"
                            class="is-primary"
                            title={governance().evidence_gate_status === "eligible_for_reward_design_review" ? "只批准离线研究" : "长期证据门槛尚未通过"}
                            disabled={
                              governanceSubmitting()
                              || !governanceRationale().trim()
                              || !governanceConfirmed()
                              || governance().evidence_gate_status !== "eligible_for_reward_design_review"
                            }
                            onClick={() => void submitRewardGovernance("approved_for_offline_research")}
                          >
                            批准离线研究目标
                          </button>
                        </div>
                      </section>
                    )}
                  </Show>
                </div>
              )}
            </Show>
            <div class="public-admin-decision-horizons">
              <For each={report().horizons}>
                {(horizon) => (
                  <div>
                    <strong>{horizon.horizon_market_sessions} 个交易日</strong>
                    <span>{horizon.observed_count} 个完整样本</span>
                    <span>平均超额 {percent(horizon.average_excess_return_percent)}</span>
                    <span>超额胜率 {percent(horizon.positive_excess_rate_percent)}</span>
                    <span>平均最大回撤 {percent(horizon.average_max_drawdown_percent)}</span>
                  </div>
                )}
              </For>
            </div>
            <div class="public-admin-decision-direction">
              <h3>250 日动作方向验证</h3>
              <p>加仓看后续是否跑赢 SPY；减仓看后续是否跑输 SPY。维持和仅研究没有可验证的方向，不硬算命中率。</p>
              <div>
                <For each={report().action_horizons.filter((item) => item.horizon_market_sessions === 250 && item.directional_sample_count > 0)}>
                  {(item) => (
                    <span><strong>{ACTION_LABELS[item.action]}</strong> {item.directional_sample_count} 个 · 方向命中 {percent(item.directional_success_rate_percent)}</span>
                  )}
                </For>
              </div>
              <Show when={report().correction_comparisons.find((item) => item.horizon_market_sessions === 250)}>
                {(comparison) => (
                  <small>人工修正可对照 {comparison().comparable_direction_count} 个：改善 {comparison().improved_direction_count}，变差 {comparison().worsened_direction_count}，不变 {comparison().unchanged_direction_count}；另有 {comparison().not_comparable_count} 个非方向动作。</small>
                )}
              </Show>
            </div>
            <Show when={report().causal_training_dataset}>
              {(dataset) => (
                <div class="public-admin-decision-direction public-admin-causal-dataset">
                  <h3>离线因果数据集（尚未训练）</h3>
                  <p>{dataset().feature_scope}</p>
                  <div>
                    <span><strong>可用标签</strong> {dataset().eligible_example_count}</span>
                    <span><strong>训练</strong> {dataset().train_example_count}</span>
                    <span><strong>验证</strong> {dataset().validation_example_count}</span>
                    <span><strong>封存测试</strong> {dataset().holdout_test_example_count}</span>
                    <span><strong>公司</strong> {dataset().distinct_symbols}</span>
                    <span><strong>驱动</strong> {dataset().distinct_drivers}</span>
                  </div>
                  <small>{dataset().split_scope}</small>
                  <small>
                    公司隔离 {dataset().company_split_isolation_verified ? "通过" : "未通过"}；
                    原文身份隔离 {dataset().source_group_split_isolation_verified ? "通过" : "未通过"}；
                    测试标签 {dataset().holdout_labels_withheld ? "保持封存" : "已暴露"}；
                    训练授权 {dataset().training_authorized ? "已开启" : "关闭"}。
                  </small>
                  <small>
                    不可拆分连通组 {dataset().connected_component_count} 个；跨公司共享原文身份 {dataset().shared_source_group_count} 个；
                    最大连通组覆盖 {dataset().largest_component_symbol_count} 家公司。
                  </small>
                  <Show when={Object.keys(dataset().development_target_counts).length > 0}>
                    <small>
                      训练/验证标签分布：{Object.entries(dataset().development_target_counts)
                        .map(([label, count]) => `${CAUSAL_DATASET_TARGET_LABELS[label] ?? label} ${count}`)
                        .join("；")}
                    </small>
                  </Show>
                  <For each={dataset().readiness_reasons}>{(reason) => <small>{reason}</small>}</For>
                  <small>{dataset().authorization_scope}</small>
                  <Show when={datasetGovernance()}>
                    {(governance) => (
                      <section class="public-admin-reward-governance" aria-label="因果训练数据集治理">
                        <header>
                          <strong>不可变数据集治理</strong>
                          <span>
                            {governance().latest_review
                              ? DATASET_GOVERNANCE_LABELS[governance().latest_review!.verdict]
                              : "尚未审查"}
                          </span>
                        </header>
                        <p>{governance().scope}</p>
                        <small title={governance().dataset.dataset_fingerprint_sha256}>
                          当前版本：{governance().dataset.policy_version} · 指纹 {governance().dataset.dataset_fingerprint_sha256.slice(0, 12)}…
                        </small>
                        <Show when={governance().latest_review}>
                          {(review) => (
                            <blockquote>
                              {review().rationale}
                              <small>
                                {dateTime(review().submitted_at)} ·
                                {review().dataset_fingerprint_sha256 === governance().dataset.dataset_fingerprint_sha256
                                  ? "当前数据集"
                                  : "旧数据集，批准已失效"}
                              </small>
                            </blockquote>
                          )}
                        </Show>
                        <label>
                          <span>本次意见（必填）</span>
                          <textarea
                            maxlength={10000}
                            value={datasetGovernanceRationale()}
                            onInput={(event) => setDatasetGovernanceRationale(event.currentTarget.value)}
                            placeholder="说明标签质量、公司/原文身份隔离、封存测试集或未来信息泄漏方面的问题。"
                          />
                        </label>
                        <label class="public-admin-reward-confirm">
                          <input
                            type="checkbox"
                            checked={datasetGovernanceConfirmed()}
                            onChange={(event) => setDatasetGovernanceConfirmed(event.currentTarget.checked)}
                          />
                          <span>我已确认公司与共享来源连通组隔离、测试标签封存和未来信息剔除；批准仅允许登记离线实验。</span>
                        </label>
                        <div class="public-admin-reward-actions">
                          <button
                            type="button"
                            disabled={datasetGovernanceSubmitting() || !datasetGovernanceRationale().trim()}
                            onClick={() => void submitDatasetGovernance("changes_requested")}
                          >
                            要求修改
                          </button>
                          <button
                            type="button"
                            disabled={datasetGovernanceSubmitting() || !datasetGovernanceRationale().trim()}
                            onClick={() => void submitDatasetGovernance("rejected")}
                          >
                            否决数据集
                          </button>
                          <button
                            type="button"
                            class="is-primary"
                            title={governance().dataset.status === "eligible_for_dataset_governance_review" ? "只开放离线实验登记" : "真实人工标签与覆盖门槛尚未通过"}
                            disabled={
                              datasetGovernanceSubmitting()
                              || !datasetGovernanceRationale().trim()
                              || !datasetGovernanceConfirmed()
                              || governance().dataset.status !== "eligible_for_dataset_governance_review"
                            }
                            onClick={() => void submitDatasetGovernance("approved_for_offline_experiment")}
                          >
                            批准登记离线实验
                          </button>
                        </div>
                      </section>
                    )}
                  </Show>
                  <Show when={trainingExperiments()}>
                    {(registry) => (
                      <section class="public-admin-reward-governance" aria-label="离线训练实验注册表">
                        <header>
                          <strong>离线实验注册表</strong>
                          <span>{registry().registration_allowed ? "可登记，未执行" : "登记关闭"}</span>
                        </header>
                        <p>{registry().scope}</p>
                        <div class="public-admin-reward-actions">
                          <span>冻结提示词基线</span>
                          <span>监督式因果分类</span>
                          <span>偏好学习关闭</span>
                          <span>RL 关闭</span>
                        </div>
                        <small>
                          盲评：至少 {registry().blind_evaluation_protocol.minimum_distinct_seeds} 个随机种子；
                          封存分区 {registry().blind_evaluation_protocol.sealed_split}；
                          训练进程不可查看标签；独立评估器必需。
                        </small>
                        <small>{registry().blind_evaluation_protocol.thresholds_origin}</small>
                        <small>{registry().blind_evaluation_protocol.promotion_scope}</small>
                        <small>
                          漂移监控：{registry().drift_monitoring_protocol.rolling_window_days} 天滚动窗口，
                          至少 {registry().drift_monitoring_protocol.minimum_audited_examples} 条人工审计；
                          契约变化或未来信息泄漏会立即停用组件。
                        </small>
                        <Show
                          when={registry().experiments.length > 0}
                          fallback={<small>尚未登记任何实验；当前不会运行训练。</small>}
                        >
                          <For each={registry().experiments.slice(0, 5)}>
                            {(experiment) => (
                              <blockquote>
                                {experiment.experiment_name} · {experiment.algorithm === "frozen_prompt_baseline" ? "冻结基线" : "监督式因果分类"}
                                <small>{dateTime(experiment.registered_at)} · 已登记未运行 · 封存测试集不可访问</small>
                              </blockquote>
                            )}
                          </For>
                        </Show>
                      </section>
                    )}
                  </Show>
                </div>
              )}
            </Show>
            <Show when={report().causal_effects}>
              {(effects) => (
                <div class="public-admin-decision-direction">
                  <h3>因果证据校准</h3>
                  <p>{effects().note}</p>
                  <div>
                    <span><strong>已分类</strong> {effects().supporting_links + effects().falsifying_links + effects().mixed_links + effects().context_only_links}/{effects().accepted_links}</span>
                    <span><strong>支持</strong> {effects().supporting_links}</span>
                    <span><strong>证伪</strong> {effects().falsifying_links}</span>
                    <span><strong>正反混合</strong> {effects().mixed_links}</span>
                    <span><strong>仅背景</strong> {effects().context_only_links}</span>
                  </div>
                  <Show when={effects().by_driver.some((item) => item.reviewed_links > 0)} fallback={<small>尚无足够人工标签形成驱动因子校准。</small>}>
                    <small>
                      {effects().by_driver
                        .filter((item) => item.reviewed_links > 0)
                        .slice(0, 6)
                        .map((item) => `${item.label}：支持 ${item.supporting_links} / 证伪 ${item.falsifying_links}`)
                        .join("；")}
                    </small>
                  </Show>
                  <Show when={effects().by_market_regime?.some((item) => item.reviewed_links > 0)} fallback={<small>市场状态校准等待带点时宏观快照的新样本与人工标签。</small>}>
                    <small>
                      市场状态：{effects().by_market_regime
                        .filter((item) => item.reviewed_links > 0)
                        .map((item) => `${MARKET_REGIME_LABELS[item.label] ?? "未命名环境"} 支持 ${item.supporting_links} / 证伪 ${item.falsifying_links}`)
                        .join("；")}
                    </small>
                  </Show>
                </div>
              )}
            </Show>
          </>
        )}
      </Show>

      <Show when={reviewQueue()}>
        {(queue) => (
          <section class="public-admin-decision-queue" aria-labelledby="public-admin-decision-queue-title">
            <header>
              <div>
                <h3 id="public-admin-decision-queue-title">
                  {queue().selection_mode === "source_batch"
                    ? "维护者来源核验"
                    : queue().selection_mode === "old_wang_batch"
                      ? "老王待回答"
                      : "完整财务证据队列"}
                </h3>
                <p>{queue().selection_scope}</p>
              </div>
              <div class="public-admin-decision-queue-filters">
                <label>
                  <span>查看</span>
                  <select value={queueSelection()} onChange={(event) => {
                    const next = event.currentTarget.value as "source_batch" | "old_wang_batch" | "full_queue";
                    setQueueSelection(next);
                    if (next !== "full_queue") {
                      setQueueStatus("pending");
                      setQueueKind("all");
                    }
                    void loadReviewQueue(next !== "full_queue" ? "pending" : queueStatus(), next !== "full_queue" ? "all" : queueKind(), next);
                  }}>
                    <option value="source_batch">维护者核来源 · 5 条</option>
                    <option value="old_wang_batch">
                      老王待回答 · 5 条{queue().old_wang_submission_authorized ? "" : "（当前账号只读）"}
                    </option>
                    <option value="full_queue">完整队列</option>
                  </select>
                </label>
                <label>
                  <span>状态</span>
                  <select disabled={queueSelection() !== "full_queue"} value={queueStatus()} onChange={(event) => { const next = event.currentTarget.value as "all" | "pending" | "accepted" | "rejected"; setQueueStatus(next); void loadReviewQueue(next, queueKind(), queueSelection()); }}>
                    <option value="pending">待复核</option>
                    <option value="accepted">已接受</option>
                    <option value="rejected">已拒绝</option>
                    <option value="all">全部</option>
                  </select>
                </label>
                <label>
                  <span>类型</span>
                  <select disabled={queueSelection() !== "full_queue"} value={queueKind()} onChange={(event) => { const next = event.currentTarget.value as "all" | "source_claim" | "operating_kpi" | "computed_comparison" | "computed_ratio"; setQueueKind(next); void loadReviewQueue(queueStatus(), next, queueSelection()); }}>
                    <option value="all">全部</option>
                    <option value="source_claim">一手事实</option>
                    <option value="operating_kpi">公司经营指标</option>
                    <option value="computed_comparison">同比/环比</option>
                    <option value="computed_ratio">利润率</option>
                  </select>
                </label>
              </div>
            </header>
            <div class="public-admin-decision-queue-summary">
              <Show when={queue().selection_mode !== "full_queue"}>
                <strong>本轮 {queue().items.length} 条 · {queue().selected_symbols.length} 家公司 · {queue().selected_drivers.length} 个驱动</strong>
              </Show>
              <span>全部 {queue().total_candidates}</span>
              <span>待复核 {queue().pending_candidates}</span>
              <span>已接受 {queue().accepted_candidates}</span>
              <span>已拒绝 {queue().rejected_candidates}</span>
              <span>来源可核验 {queue().source_review_ready_candidates}</span>
              <span>来源待补齐 {queue().source_blocked_candidates}</span>
              <span>维护者待核 {queue().source_unreviewed_candidates}</span>
              <span>已核待老王 {queue().source_verified_waiting_causal_candidates}</span>
              <span>来源已排除 {queue().source_excluded_candidates}</span>
              <span>跨快照沿用 {queue().source_review_reused_candidates ?? 0}</span>
              <span>跨快照冲突 {queue().source_review_conflicted_candidates ?? 0}</span>
              <span>支持 {queue().supporting_candidates}</span>
              <span>证伪 {queue().falsifying_candidates}</span>
            </div>
            <Show when={!queue().old_wang_reviewer_configured}>
              <p class="public-admin-decision-queue-empty">服务器尚未配置老王 Web 审阅账号；任何管理员都不能提交老王因果判断。</p>
            </Show>
            <Show when={queue().old_wang_reviewer_configured && !queue().old_wang_submission_authorized}>
              <p class="public-admin-decision-queue-empty">当前管理员只能核验来源和只读查看老王待答题；老王因果提交已由服务器身份门禁关闭。</p>
            </Show>
            <Show when={queue().items.length > 0} fallback={<p class="public-admin-decision-queue-empty">当前筛选下没有证据。</p>}>
              <div class="public-admin-decision-queue-list">
                <For each={queue().items.slice(0, 12)}>
                  {(item) => (
                    <article class={`is-${item.priority}`}>
                      <div>
                        <span>{item.symbol} · {item.company_name} · {QUEUE_KIND_LABELS[item.kind]} · {item.observation.as_of}</span>
                        <strong>{item.driver_label}：{item.observation.label}</strong>
                        <p>{item.observation.value}</p>
                        <small>{item.priority_reasons.join(" ")}</small>
                        <Show when={!item.source_review_ready}>
                          <small>来源待补齐：{item.source_review_blockers.join("；")}。该材料保留在完整队列，但不会进入本轮复核或监督训练。</small>
                        </Show>
                        <Show when={item.source_review_reused_across_snapshots}>
                          <small>来源核验沿用自同一冻结证据的历史快照 {item.source_review_origin_sample_id}；证据身份未变化，无需重复核对。</small>
                        </Show>
                        <Show when={item.source_review_conflict}>
                          <small>同一冻结证据存在相互冲突的跨快照来源核验，已失败关闭；请先修正来源审计，不能交给老王或进入训练。</small>
                        </Show>
                        <Show when={item.review_effect && item.review_effect !== "unclassified"}>
                          <small>已标注：{item.review_effect === "supports" ? "支持" : item.review_effect === "falsifies" ? "证伪" : item.review_effect === "mixed" ? "正反混合" : "仅背景"}</small>
                        </Show>
                        <Show when={item.review_source_verification === "evidence_mismatch"}>
                          <small>来源排除：原文、数值或口径不一致；不会进入训练。</small>
                        </Show>
                        <Show when={item.review_source_verification === "insufficient_source_context"}>
                          <small>来源排除：上下文不足；不会进入训练。</small>
                        </Show>
                        <Show when={item.training_label_eligible}>
                          <small>已满足当前监督标签合同；仍需数据集治理和独立实验审批。</small>
                        </Show>
                      </div>
                      <aside>
                        <em class={`is-${item.status}`}>{QUEUE_STATUS_LABELS[item.status]}</em>
                        <button type="button" disabled={!item.source_review_ready} onClick={() => void openReviewQueueItem(item)}>
                          {!item.source_review_ready
                            ? "来源待补齐"
                            : queue().selection_mode === "source_batch"
                              ? "核验来源"
                              : queue().selection_mode === "old_wang_batch"
                                ? "老王回答"
                                : "查看复核"}
                        </button>
                      </aside>
                    </article>
                  )}
                </For>
              </div>
            </Show>
          </section>
        )}
      </Show>

      <div class="public-admin-decision-search">
        <label>
          <span>查看公司决策轨迹</span>
          <input
            value={symbol()}
            maxlength={16}
            onInput={(event) => setSymbol(event.currentTarget.value.toUpperCase())}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                setFocusedCausalKey("");
                void loadReplay();
              }
            }}
            placeholder="例如 SNDK"
          />
        </label>
        <button type="button" disabled={loading()} onClick={() => { setFocusedCausalKey(""); void loadReplay(); }}>
          {loading() ? "读取中…" : "读取轨迹"}
        </button>
      </div>

      <Show when={error()}><p class="public-admin-decision-message is-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-decision-message is-success">{notice()}</p></Show>

      <Show when={(replay()?.quarantined_sample_count ?? 0) > 0}>
        <div class="public-admin-decision-warning" role="status">
          已隔离 {replay()?.quarantined_sample_count} 条未通过当前完整性校验的历史样本；这些记录不会进入训练或评测。
          <details>
            <summary>查看隔离原因</summary>
            <For each={replay()?.quarantine_warnings ?? []}>
              {(warning) => <p>{warning.file_name}：{warning.reason}</p>}
            </For>
          </details>
        </div>
      </Show>
      <Show
        when={(replay()?.samples.length ?? 0) > 0}
        fallback={
          <p class="public-admin-decision-empty">
            {(replay()?.quarantined_sample_count ?? 0) > 0
              ? "该公司的现有历史样本均未通过当前完整性校验，请先修复来源记录后再复核。"
              : "该公司还没有训练样本。下一次每日公司评级刷新后会自动建立。"}
          </p>
        }
      >
        <div class="public-admin-decision-layout">
          <div class="public-admin-decision-sample-list" aria-label="决策样本">
            <For each={[...(replay()?.samples ?? [])].reverse()}>
              {(sample) => (
                <button
                  type="button"
                  classList={{ "is-selected": selectedId() === sample.sample_id }}
                  onClick={() => setSelectedId(sample.sample_id)}
                >
                  <span>{dateTime(sample.observed_at)}</span>
                  <strong>{sample.state.source_rating_score.toFixed(1)} 分 · {ACTION_LABELS[sample.selected_action]}</strong>
                  <em class={`is-${sample.human_review.status}`}>{REVIEW_LABELS[sample.human_review.status]}</em>
                </button>
              )}
            </For>
          </div>

          <Show when={selectedSample()}>
            {(sample) => (
              <div class="public-admin-decision-review">
                <div class="public-admin-decision-snapshot">
                  <div>
                    <span>{sample().state.symbol} · {sample().state.theme}</span>
                    <h3>{sample().state.company_name}</h3>
                  </div>
                  <strong>{sample().state.source_rating_score.toFixed(1)}</strong>
                </div>
                <p>{sample().state.decision.rationale.join(" ")}</p>
                <Show when={sample().state.decision.methodology?.policy_version}>
                  <section class="public-admin-decision-completeness" aria-label="Hari 已确认逻辑门禁">
                    <header>
                      <div>
                        <strong>Hari 已确认逻辑门禁</strong>
                        <span>{sample().state.decision.methodology!.skill_id} {sample().state.decision.methodology!.skill_version} · 只使用 {sample().state.decision.methodology!.confirmed_logic_ids.join("、")}</span>
                      </div>
                      <em>
                        {sample().state.decision.methodology!.pre_methodology_action !== "increase_candidate"
                          ? "本轮非增加候选"
                          : sample().state.decision.methodology!.increase_candidate_authorized
                            ? "公司门禁通过"
                            : "增加暴露被阻断"}
                      </em>
                    </header>
                    <div>
                      <For each={sample().state.decision.methodology!.rules}>
                        {(rule) => (
                          <details class={rule.status === "passed" ? "is-pass" : rule.status === "blocked" ? "is-missing" : "is-partial"}>
                            <summary>
                              <span>{rule.logic_id} · {rule.label}</span>
                              <em>{METHODOLOGY_STATUS_LABELS[rule.status] ?? rule.status}</em>
                            </summary>
                            <Show when={rule.evidence.length > 0}><p>{rule.evidence.join("；")}</p></Show>
                            <Show when={rule.gaps.length > 0}><small>待补：{rule.gaps.join("；")}</small></Show>
                          </details>
                        )}
                      </For>
                    </div>
                    <Show when={sample().state.decision.methodology!.blocking_reasons.length > 0}>
                      <p>当前阻断：{sample().state.decision.methodology!.blocking_reasons.join("；")}</p>
                    </Show>
                    <small>{sample().state.decision.methodology!.scope}</small>
                  </section>
                </Show>
                <Show when={sample().state.decision_completeness}>
                  {(completeness) => (
                    <section class="public-admin-decision-completeness" aria-label="决策链完整性">
                      <header>
                        <div>
                          <strong>决策链完整性 {completeness().passed_checks}/{completeness().total_checks}</strong>
                          <span>{completeness().directional_research_ready ? "可形成方向研究" : "仅可继续研究"} · {completeness().portfolio_decision_ready ? "组合输入完整" : "不可进入组合动作"}</span>
                        </div>
                        <em>{percent(completeness().completeness_percent)}</em>
                      </header>
                      <div>
                        <For each={completeness().checks}>
                          {(check) => (
                            <details class={`is-${check.status}`}>
                              <summary>
                                <span>{check.label}</span>
                                <em>{check.status === "pass" ? "完整" : check.status === "partial" ? "部分" : "缺失"}</em>
                              </summary>
                              <Show when={check.evidence.length > 0}><p>{check.evidence.join("；")}</p></Show>
                              <Show when={check.gaps.length > 0}><small>待补：{check.gaps.join("；")}</small></Show>
                            </details>
                          )}
                        </For>
                      </div>
                      <small>{completeness().scope}</small>
                    </section>
                  )}
                </Show>
                <Show when={sample().state.crowding}>
                  {(crowding) => (
                    <section class="public-admin-decision-crowding" aria-label="拥挤度证据">
                      <header>
                        <div>
                          <strong>拥挤度证据 · {CROWDING_STATUS_LABELS[crowding().status]}</strong>
                          <span>{crowding().label}</span>
                        </div>
                        <em>{crowding().score == null ? "—" : crowding().score!.toFixed(1)}</em>
                      </header>
                      <Show when={crowding().components.length > 0}>
                        <div class="public-admin-decision-crowding-grid">
                          <For each={crowding().components}>
                            {(component) => (
                              <article>
                                <span>{component.label}</span>
                                <strong>{component.pressure_score.toFixed(1)}</strong>
                                <small>原始 {component.raw_value_percent >= 0 ? "+" : ""}{component.raw_value_percent.toFixed(1)}% · 权重 {(component.weight * 100).toFixed(0)}%</small>
                                <small>{component.as_of} · {component.source}</small>
                              </article>
                            )}
                          </For>
                        </div>
                      </Show>
                      <Show when={crowding().short_interest}>
                        {(shortInterest) => (
                          <article class="public-admin-decision-short-interest">
                            <header>
                              <div>
                                <strong>空头仓位变化</strong>
                                <span>背景证据 · 不计分</span>
                              </div>
                              <em>{shortInterest().change_percent >= 0 ? "+" : ""}{shortInterest().change_percent.toFixed(1)}%</em>
                            </header>
                            <p>
                              当前空头 {Math.round(shortInterest().current_shares_short).toLocaleString("zh-CN")} 股，
                              日均成交 {Math.round(shortInterest().average_daily_share_volume).toLocaleString("zh-CN")} 股，
                              回补约 {shortInterest().days_to_cover.toFixed(1)} 天。
                            </p>
                            <small>{shortInterest().interpretation}</small>
                            <a href={shortInterest().source_url} target="_blank" rel="noreferrer">
                              {shortInterest().as_of} · 查看 Nasdaq 原始结算表
                            </a>
                          </article>
                        )}
                      </Show>
                      <Show when={crowding().options_positioning}>
                        {(options) => (
                          <article class="public-admin-decision-background-context is-options">
                            <header>
                              <div>
                                <strong>期权仓位结构</strong>
                                <span>背景证据 · 不计分</span>
                              </div>
                              <em>
                                P/C 未平仓 {options().put_call_open_interest_ratio == null
                                  ? "—"
                                  : options().put_call_open_interest_ratio!.toFixed(2)}
                              </em>
                            </header>
                            <p>
                              {options().expiration_date} 到期（余 {options().days_to_expiration} 天）；
                              看涨/看跌未平仓量 {Math.round(options().call_open_interest).toLocaleString("zh-CN")}
                              /{Math.round(options().put_open_interest).toLocaleString("zh-CN")}；
                              当日成交量比 {options().put_call_volume_ratio == null
                                ? "—"
                                : options().put_call_volume_ratio!.toFixed(2)}。
                            </p>
                            <small>{options().interpretation}</small>
                            <a href={options().source_url} target="_blank" rel="noreferrer">
                              {options().as_of} · 查看 Nasdaq 原始期权链
                            </a>
                          </article>
                        )}
                      </Show>
                      <Show when={crowding().news_attention}>
                        {(news) => (
                          <article class="public-admin-decision-background-context is-news">
                            <header>
                              <div>
                                <strong>新闻发布活跃度</strong>
                                <span>背景证据 · 不计分</span>
                              </div>
                              <em>
                                速率比 {news().activity_ratio == null ? "—" : news().activity_ratio!.toFixed(2)}
                              </em>
                            </header>
                            <p>
                              近 {news().recent_window_days} 日 {news().recent_article_count} 篇；此前
                              {news().window_days - news().recent_window_days} 日 {news().prior_article_count} 篇；
                              观察到 {news().unique_publishers} 个发布方。
                            </p>
                            <small>{news().interpretation}</small>
                            <a href={news().source_url} target="_blank" rel="noreferrer">
                              {news().as_of} · 查看 Nasdaq 聚合发布流
                            </a>
                          </article>
                        )}
                      </Show>
                      <Show when={crowding().institutional_holdings}>
                        {(institutional) => (
                          <article class="public-admin-decision-background-context is-institutional">
                            <header>
                              <div>
                                <strong>机构 13F 聚合</strong>
                                <span>背景证据 · 不计分</span>
                              </div>
                              <em>{institutional().institutional_ownership_percent.toFixed(1)}%</em>
                            </header>
                            <p>
                              {institutional().institutional_holders.toLocaleString("zh-CN")} 位持有人；
                              增持/减持分类股数 {Math.round(institutional().increased_positions_shares).toLocaleString("zh-CN")}
                              /{Math.round(institutional().decreased_positions_shares).toLocaleString("zh-CN")}；
                              前 {institutional().top_sample_rows} 条记录跨 {institutional().report_period_count} 个报告期
                              （{institutional().earliest_report_period}—{institutional().latest_report_period}）。
                            </p>
                            <small>{institutional().interpretation}</small>
                            <a href={institutional().source_url} target="_blank" rel="noreferrer">
                              {institutional().observed_on} 观察 · 查看 Nasdaq 13F 聚合表
                            </a>
                          </article>
                        )}
                      </Show>
                      <Show when={crowding().analyst_consensus}>
                        {(analyst) => (
                          <article class="public-admin-decision-background-context is-analyst">
                            <header>
                              <div>
                                <strong>分析师建议与目标价</strong>
                                <span>背景证据 · 不计分</span>
                              </div>
                              <em>{analyst().dominant_rating} {analyst().dominant_share_percent.toFixed(1)}%</em>
                            </header>
                            <p>
                              买入/持有/卖出 {analyst().buy_count}/{analyst().hold_count}/{analyst().sell_count}
                              （共 {analyst().recommendation_count} 个建议）；目标价低/共识/高
                              {" "}{analyst().low_target_price.toFixed(2)} / {analyst().consensus_target_price.toFixed(2)} / {analyst().high_target_price.toFixed(2)}，
                              区间宽度为共识值的 {analyst().target_range_width_percent.toFixed(1)}%。
                            </p>
                            <small>{analyst().interpretation}</small>
                            <a href={analyst().source_url} target="_blank" rel="noreferrer">
                              {analyst().observed_on} 观察 · 查看 Nasdaq 聚合原始数据
                            </a>
                          </article>
                        )}
                      </Show>
                      <Show when={crowding().missing_checks.length > 0}>
                        <details>
                          <summary>仍缺 {crowding().missing_checks.length} 类证据</summary>
                          <p>{crowding().missing_checks.join("；")}</p>
                        </details>
                      </Show>
                      <small>{crowding().scope}</small>
                    </section>
                  )}
                </Show>
                <Show when={sample().state.decision.causal_confidence}>
                  {(confidence) => (
                    <div class={`public-admin-decision-confidence is-${confidence().adjustment}`}>
                      <strong>因果置信度：{confidence().base_confidence} → {confidence().effective_confidence}</strong>
                      <span>已晋级 {confidence().promoted_driver_count} 个驱动；冲突冻结 {confidence().blocked_conflict_count} 个；人工否决 {confidence().blocked_human_rejection_count} 个；证伪冻结 {confidence().blocked_falsification_count ?? 0} 个。动作不受此项直接改变。</span>
                    </div>
                  )}
                </Show>
                <div class="public-admin-decision-outcomes">
                  <For each={[20, 60, 250]}>
                    {(horizon) => {
                      const outcome = () => latestOutcome(sample(), horizon);
                      return (
                        <div>
                          <span>{horizon} 日结果</span>
                          <strong>{outcome()?.status === "observed" ? `超额 ${percent(outcome()?.excess_return_percent)}` : "等待走完"}</strong>
                          <em>{outcome()?.status === "observed" ? `最大回撤 ${percent(outcome()?.max_drawdown_percent)}` : "不会提前标注"}</em>
                        </div>
                      );
                    }}
                  </For>
                </div>

                <Show when={sample().state.first_principles?.operating_kpi_registry}>
                  {(registry) => (
                    <section class="public-admin-decision-kpis" aria-labelledby="public-admin-kpi-title">
                      <div>
                        <h4 id="public-admin-kpi-title">产业专属经营指标</h4>
                        <p>{registry().version} · 每个指标先保留公司原始定义，再进入跨期验证。公司自定义口径默认禁止跨公司比较。</p>
                      </div>
                      <div class="public-admin-decision-kpi-grid">
                        <For each={registry().entries}>
                          {(entry) => {
                            const model = sample().state.first_principles;
                            const driver = model
                              ? [...model.demand_drivers, ...model.supply_drivers, ...model.value_capture_drivers]
                                .find((item) => item.driver_id === entry.driver_id)
                              : undefined;
                            return (
                              <details>
                                <summary>
                                  <span>{entry.label}</span>
                                  <em>{KPI_COMPARABILITY_LABELS[entry.comparability_policy]}</em>
                                </summary>
                                <p>{entry.definition}</p>
                                <dl>
                                  <div><dt>验证驱动</dt><dd>{driver?.label ?? entry.driver_id}</dd></div>
                                  <div><dt>单位/期间</dt><dd>{entry.unit} · {entry.period_policy}</dd></div>
                                  <div><dt>优先来源</dt><dd>{entry.source_priority.join(" → ")}</dd></div>
                                  <div><dt>接收门槛</dt><dd>{entry.acceptance_requirements.join("；")}</dd></div>
                                  <div><dt>禁止推断</dt><dd>{entry.forbidden_inference}</dd></div>
                                </dl>
                              </details>
                            );
                          }}
                        </For>
                      </div>
                    </section>
                  )}
                </Show>

                <Show when={selectedCausalDrivers().length > 0}>
                  <section class="public-admin-decision-causal" aria-labelledby="public-admin-causal-title">
                    <div>
                      <h4 id="public-admin-causal-title">老王单问蒸馏复核</h4>
                      <p>一次只处理一条证据；原话、结构化归纳、适用边界和反证分别留痕。未经老王本人确认的内容不能进入监督训练候选。</p>
                    </div>
                    <For each={selectedCausalDrivers()}>
                      {(driver) => (
                        <article>
                          <header>
                            <strong>{driver.label}</strong>
                            <span>{driver.mechanism}</span>
                            <Show when={driver.promotion?.policy_version}>
                              <em class={`is-${driver.promotion.status}`}>
                                {CAUSAL_PROMOTION_LABELS[driver.promotion.status]} · 有效 {driver.promotion.active_claim_count} · 已接受 {driver.promotion.accepted_claim_count} · {driver.promotion.distinct_periods} 个期间 / {driver.promotion.evidence_span_days} 天
                              </em>
                              <small>{driver.promotion.reasons.join(" ")}</small>
                            </Show>
                          </header>
                          <For each={driver.observations}>
                            {(observation) => {
                              const key = () => causalKey(driver.driver_id, observation.observation_id);
                              const value = (): CausalReviewDraft => causalReviewDrafts()[key()] ?? {
                                verdict: "",
                                effect: "unclassified",
                                explanation: "",
                                verbatimJudgment: "",
                                applicabilityBoundary: "",
                                falsifier: "",
                                speakerConfirmation: "",
                                sourceVerification: "",
                                sourceVerificationNote: "",
                                oldWangConfirmationAttested: false,
                                stage: "source",
                              };
                              const claimReviewable = () => causalObservationCanBeAccepted(observation);
                              return (
                                <div class="public-admin-decision-causal-observation">
                                  <div class="public-admin-decision-causal-fact">
                                    <span>{CAUSAL_RELATIONSHIP_LABELS[observation.relationship]} · {observation.as_of}</span>
                                    <strong>{observation.label}</strong>
                                    <p>{observation.value}</p>
                                    <Show when={observation.claim}>
                                      {(claim) => (
                                        <dl class="public-admin-decision-claim-trace">
                                          <div><dt>指标/期间</dt><dd>{claim().metric_id} · {claim().period}</dd></div>
                                          <div><dt>定义口径</dt><dd>{!claim().metric_basis || claim().metric_basis === "unspecified_legacy" ? "旧数据未标注（不可晋级）" : claim().metric_basis}</dd></div>
                                          <div><dt>披露值</dt><dd>{claim().numeric_value ?? "定性"}{claim().numeric_value == null ? "" : ` ${claim().unit}`}</dd></div>
                                          <div><dt>生命周期</dt><dd class={`is-${claim().lifecycle_status || "legacy"}`}>{CLAIM_LIFECYCLE_LABELS[claim().lifecycle_status] || "旧数据未标注"} · {claim().disposition || "legacy"}</dd></div>
                                          <div><dt>来源文件</dt><dd>{claim().source_document} · {claim().source_event_id}</dd></div>
                                          <div><dt>说话人/位置</dt><dd>{claim().speaker || "公司文件"} · {claim().source_locator}</dd></div>
                                          <div><dt>原文短证据</dt><dd>{claim().quote_excerpt}</dd></div>
                                        </dl>
                                      )}
                                    </Show>
                                    <Show when={observation.computed}>
                                      {(computed) => (
                                        <dl class="public-admin-decision-claim-trace">
                                          <div><dt>计算类型</dt><dd>{computed().comparison_kind === "year_over_year" ? "同比" : "环比"} · {computed().formula_version}</dd></div>
                                          <div><dt>指标口径</dt><dd>{computed().metric_id} · {computed().metric_basis}</dd></div>
                                          <div><dt>本期</dt><dd>{computed().current_period} · {computed().current_numeric_value} {computed().unit}</dd></div>
                                          <div><dt>对照期</dt><dd>{computed().prior_period} · {computed().prior_numeric_value} {computed().unit}</dd></div>
                                          <div><dt>变化</dt><dd>{computed().change_percent.toFixed(2)}%</dd></div>
                                          <div><dt>两端来源</dt><dd><a href={computed().prior_source_url} target="_blank" rel="noreferrer">对照期 SEC ↗</a> · <a href={computed().current_source_url} target="_blank" rel="noreferrer">本期 SEC ↗</a></dd></div>
                                        </dl>
                                      )}
                                    </Show>
                                    <Show when={observation.ratio}>
                                      {(ratio) => (
                                        <dl class="public-admin-decision-claim-trace">
                                          <div><dt>计算类型</dt><dd>{ratio().ratio_kind === "gross_margin" ? "毛利率" : "营业利润率"} · {ratio().formula_version}</dd></div>
                                          <div><dt>期间</dt><dd>{ratio().period}</dd></div>
                                          <div><dt>分子</dt><dd>{ratio().numerator_metric_id} · {ratio().numerator_numeric_value} · {ratio().numerator_metric_basis}</dd></div>
                                          <div><dt>分母</dt><dd>{ratio().denominator_metric_id} · {ratio().denominator_numeric_value} · {ratio().denominator_metric_basis}</dd></div>
                                          <div><dt>结果</dt><dd>{ratio().result_percent.toFixed(2)}%</dd></div>
                                          <div><dt>来源</dt><dd><a href={ratio().source_url} target="_blank" rel="noreferrer">同一 SEC filing ↗</a></dd></div>
                                        </dl>
                                      )}
                                    </Show>
                                    <Show when={observation.ratio_trend}>
                                      {(trend) => (
                                        <dl class="public-admin-decision-claim-trace">
                                          <div><dt>趋势类型</dt><dd>{trend().comparison_kind === "year_over_year" ? "同比" : "环比"} · {trend().formula_version}</dd></div>
                                          <div><dt>指标</dt><dd>{trend().metric_id === "gross_margin" ? "毛利率" : "营业利润率"}</dd></div>
                                          <div><dt>本期</dt><dd>{trend().current.period} · {trend().current.result_percent.toFixed(2)}%</dd></div>
                                          <div><dt>对照期</dt><dd>{trend().prior.period} · {trend().prior.result_percent.toFixed(2)}%</dd></div>
                                          <div><dt>变化</dt><dd>{trend().change_percentage_points.toFixed(2)} 个百分点</dd></div>
                                          <div><dt>两端来源</dt><dd><a href={trend().prior.source_url} target="_blank" rel="noreferrer">对照期 SEC ↗</a> · <a href={trend().current.source_url} target="_blank" rel="noreferrer">本期 SEC ↗</a></dd></div>
                                        </dl>
                                      )}
                                    </Show>
                                    <Show when={observation.operating_kpi}>
                                      {(claim) => (
                                        <dl class="public-admin-decision-claim-trace">
                                          <div><dt>经营指标</dt><dd>{claim().issuer_metric_name} · {claim().kpi_id}</dd></div>
                                          <div><dt>公司原始定义</dt><dd>{claim().issuer_definition}</dd></div>
                                          <div><dt>期间/范围</dt><dd>{claim().period} · {claim().measurement_scope}</dd></div>
                                          <div><dt>数值</dt><dd>{claim().numeric_value ?? "里程碑"}{claim().numeric_value == null ? "" : ` ${claim().unit}`}</dd></div>
                                          <div><dt>比较口径</dt><dd>{claim().comparison_basis} · {claim().definition_changed ? "公司声明口径已变" : "未声明口径变化"}</dd></div>
                                          <div><dt>生命周期</dt><dd class={`is-${claim().lifecycle_status}`}>{CLAIM_LIFECYCLE_LABELS[claim().lifecycle_status]} · {claim().disposition}</dd></div>
                                          <div><dt>原文短证据</dt><dd>{claim().evidence_quote}</dd></div>
                                          <div><dt>文件/位置</dt><dd>{claim().source_document} · {claim().source_locator}</dd></div>
                                          <Show when={claim().source_time_precision}>
                                            {(precision) => <div><dt>来源时间精度</dt><dd>{precision() === "exact" ? "精确时间" : "仅日期，按当日末保守入库"}</dd></div>}
                                          </Show>
                                          <Show when={claim().source_artifact}>
                                            {(artifact) => (
                                              <>
                                                <div><dt>原文文件哈希</dt><dd><code>{artifact().source_sha256}</code></dd></div>
                                                <div><dt>归档对象</dt><dd>{artifact().object_path} · {artifact().byte_length} 字节</dd></div>
                                              </>
                                            )}
                                          </Show>
                                        </dl>
                                      )}
                                    </Show>
                                    <Show when={observation.source_url} fallback={<small>{observation.source}</small>}>
                                      {(url) => <a href={url()} target="_blank" rel="noreferrer">{observation.source} ↗</a>}
                                    </Show>
                                  </div>
                                  <div class="public-admin-distillation-step">
                                    <small>两阶段单问蒸馏 · 维护者只核来源，老王只判断因果；来源记录永不充当训练标签</small>
                                    <Show when={value().stage === "source"}>
                                      <label>
                                        <span>{causalSourceVerificationPrompt(observation)}</span>
                                        <select
                                          value={value().sourceVerification}
                                          onChange={(event) => updateCausalReview(driver.driver_id, observation.observation_id, {
                                            sourceVerification: event.currentTarget.value as CausalSourceVerification,
                                            oldWangConfirmationAttested: false,
                                          })}
                                        >
                                          <option value="">尚未核对</option>
                                          <option value="verified_against_source">已逐项核对，一致</option>
                                          <option value="evidence_mismatch">原文或口径不一致</option>
                                          <option value="insufficient_source_context">上下文不足，不能判断</option>
                                        </select>
                                      </label>
                                      <label>
                                        <span>来源核验记录</span>
                                        <textarea
                                          maxlength={2000}
                                          value={value().sourceVerificationNote}
                                          onInput={(event) => updateCausalReview(driver.driver_id, observation.observation_id, { sourceVerificationNote: event.currentTarget.value })}
                                          placeholder="写明打开了哪份原文、核对了什么；若不一致，指出具体字段。"
                                        />
                                      </label>
                                      <Show
                                        when={value().sourceVerification === "verified_against_source"}
                                        fallback={
                                          <button
                                            type="button"
                                            disabled={
                                              causalSubmitting() === key()
                                              || !value().sourceVerification
                                              || !value().sourceVerificationNote.trim()
                                            }
                                            onClick={() => void submitCausalSourceReview(driver.driver_id, observation.observation_id)}
                                          >
                                            {causalSubmitting() === key() ? "保存中…" : "独立保存来源问题"}
                                          </button>
                                        }
                                      >
                                        <button
                                          type="button"
                                          disabled={causalSubmitting() === key() || !value().sourceVerificationNote.trim()}
                                          onClick={() => void submitCausalSourceReview(driver.driver_id, observation.observation_id)}
                                        >
                                          {causalSubmitting() === key() ? "保存中…" : "保存核验并交给老王"}
                                        </button>
                                      </Show>
                                    </Show>
                                    <Show when={value().stage === "verbatim"}>
                                      <label>
                                        <span>这条证据在当时为什么能或不能改变你对“{driver.label}”的判断？</span>
                                        <textarea
                                          maxlength={2000}
                                          value={value().verbatimJudgment}
                                          onInput={(event) => updateCausalReview(driver.driver_id, observation.observation_id, { verbatimJudgment: event.currentTarget.value })}
                                          placeholder="请用你的原话回答；不要只写涨跌结果。"
                                        />
                                      </label>
                                      <div class="public-admin-distillation-actions">
                                        <button type="button" onClick={() => updateCausalReview(driver.driver_id, observation.observation_id, { stage: "source" })}>上一步</button>
                                        <button type="button" disabled={!value().verbatimJudgment.trim()} onClick={() => updateCausalReview(driver.driver_id, observation.observation_id, { stage: "relationship" })}>继续</button>
                                      </div>
                                    </Show>
                                    <Show when={value().stage === "relationship"}>
                                      <label>
                                        <span>把刚才的原话归一化后，这条证据与该驱动是什么关系？</span>
                                        <select
                                          value={value().verdict}
                                          onChange={(event) => {
                                            const next = event.currentTarget.value as "" | "accepted" | "rejected";
                                            updateCausalReview(driver.driver_id, observation.observation_id, { verdict: next, ...(next === "rejected" ? { effect: "unclassified" as const } : {}) });
                                          }}
                                        >
                                          <option value="">暂不判断</option>
                                          <option value="accepted" disabled={!claimReviewable()}>关系成立</option>
                                          <option value="rejected">关系不成立</option>
                                        </select>
                                      </label>
                                      <Show when={value().verdict === "accepted"}>
                                        <label>
                                          <span>证据作用</span>
                                          <select value={value().effect} onChange={(event) => updateCausalReview(driver.driver_id, observation.observation_id, { effect: event.currentTarget.value as CausalEvidenceEffect })}>
                                            <option value="unclassified">请选择</option>
                                            <option value="supports">支持该驱动</option>
                                            <option value="falsifies">证伪该驱动</option>
                                            <option value="mixed">正反混合</option>
                                            <option value="context_only">仅背景信息</option>
                                          </select>
                                        </label>
                                      </Show>
                                      <label>
                                        <span>结构化归纳（与原话分开）</span>
                                        <textarea maxlength={2000} value={value().explanation} onInput={(event) => updateCausalReview(driver.driver_id, observation.observation_id, { explanation: event.currentTarget.value })} placeholder="只归纳因果含义，不新增原话中没有的因果。" />
                                      </label>
                                      <div class="public-admin-distillation-actions">
                                        <button type="button" onClick={() => updateCausalReview(driver.driver_id, observation.observation_id, { stage: "verbatim" })}>上一步</button>
                                        <button type="button" disabled={!value().verdict || !value().explanation.trim() || (value().verdict === "accepted" && value().effect === "unclassified")} onClick={() => updateCausalReview(driver.driver_id, observation.observation_id, { stage: "boundary" })}>继续</button>
                                      </div>
                                    </Show>
                                    <Show when={value().stage === "boundary"}>
                                      <label>
                                        <span>这条判断在什么条件下才适用？</span>
                                        <textarea maxlength={2000} value={value().applicabilityBoundary} onInput={(event) => updateCausalReview(driver.driver_id, observation.observation_id, { applicabilityBoundary: event.currentTarget.value })} placeholder="写清行业、周期、公司或数据口径边界；尚不明确也请直接写明。" />
                                      </label>
                                      <div class="public-admin-distillation-actions">
                                        <button type="button" onClick={() => updateCausalReview(driver.driver_id, observation.observation_id, { stage: "relationship" })}>上一步</button>
                                        <button type="button" disabled={!value().applicabilityBoundary.trim()} onClick={() => updateCausalReview(driver.driver_id, observation.observation_id, { stage: "falsifier" })}>继续</button>
                                      </div>
                                    </Show>
                                    <Show when={value().stage === "falsifier"}>
                                      <label>
                                        <span>未来出现什么可观察事实时，你会承认这条判断不成立？</span>
                                        <textarea maxlength={2000} value={value().falsifier} onInput={(event) => updateCausalReview(driver.driver_id, observation.observation_id, { falsifier: event.currentTarget.value })} placeholder="请写可观察的反证；尚不明确也请直接写明。" />
                                      </label>
                                      <div class="public-admin-distillation-actions">
                                        <button type="button" onClick={() => updateCausalReview(driver.driver_id, observation.observation_id, { stage: "boundary" })}>上一步</button>
                                        <button type="button" disabled={!value().falsifier.trim()} onClick={() => updateCausalReview(driver.driver_id, observation.observation_id, { stage: "confirmation" })}>继续</button>
                                      </div>
                                    </Show>
                                    <Show when={value().stage === "confirmation"}>
                                      <label>
                                        <span>这份记录的确认范围是什么？</span>
                                        <select disabled={!reviewQueue()?.old_wang_submission_authorized} value={value().speakerConfirmation} onChange={(event) => updateCausalReview(driver.driver_id, observation.observation_id, { speakerConfirmation: event.currentTarget.value as CausalSpeakerConfirmation })}>
                                          <option value="">请选择，不默认确认</option>
                                          <option value="old_wang_confirmed">老王本人直接确认</option>
                                        </select>
                                        <small>这里不能选择维护者；维护者的来源核验已经单独留痕，只有老王本人确认才能生成因果标签候选。</small>
                                      </label>
                                      <Show when={value().speakerConfirmation === "old_wang_confirmed"}>
                                        <label class="public-admin-reward-confirm">
                                          <input
                                            type="checkbox"
                                            disabled={!reviewQueue()?.old_wang_submission_authorized}
                                            checked={value().oldWangConfirmationAttested}
                                            onChange={(event) => updateCausalReview(driver.driver_id, observation.observation_id, { oldWangConfirmationAttested: event.currentTarget.checked })}
                                          />
                                          <span>我确认上面的回答是我本人针对这一条已核验原文作出的判断，不是维护者、AI 或事后涨跌替我补写。</span>
                                        </label>
                                      </Show>
                                      <div class="public-admin-distillation-summary">
                                        <span>来源核验：{value().sourceVerificationNote}</span>
                                        <span>原话：{value().verbatimJudgment}</span>
                                        <span>归纳：{value().explanation}</span>
                                        <span>边界：{value().applicabilityBoundary}</span>
                                        <span>反证：{value().falsifier}</span>
                                      </div>
                                      <div class="public-admin-distillation-actions">
                                        <button type="button" onClick={() => updateCausalReview(driver.driver_id, observation.observation_id, { stage: "falsifier" })}>上一步</button>
                                        <button
                                          type="button"
                                          class="public-admin-causal-submit"
                                          disabled={
                                            causalSubmitting() === key()
                                            || !reviewQueue()?.old_wang_submission_authorized
                                            || !value().speakerConfirmation
                                            || (value().speakerConfirmation === "old_wang_confirmed" && !value().oldWangConfirmationAttested)
                                            || (value().verdict === "accepted" && !claimReviewable())
                                          }
                                          onClick={() => void submitCausalReview(driver.driver_id, observation.observation_id)}
                                        >
                                          {causalSubmitting() === key() ? "保存中…" : value().reviewId ? "更新这条蒸馏复核" : "确认并保存"}
                                        </button>
                                      </div>
                                    </Show>
                                  </div>
                                </div>
                              );
                            }}
                          </For>
                        </article>
                      )}
                    </For>
                  </section>
                </Show>

                <div class="public-admin-decision-mode" role="group" aria-label="复核结论">
                  <button type="button" classList={{ "is-active": mode() === "accepted" }} onClick={() => { setMode("accepted"); setVerdict("supported"); }}>接受</button>
                  <button type="button" classList={{ "is-active": mode() === "corrected" }} onClick={() => { setMode("corrected"); setVerdict("weakened"); }}>修正</button>
                  <button type="button" classList={{ "is-active": mode() === "rejected" }} onClick={() => { setMode("rejected"); setVerdict("invalidated"); }}>否决</button>
                </div>

                <div class="public-admin-decision-form">
                  <label>
                    <span>基本判断</span>
                    <select value={verdict()} disabled={mode() === "rejected"} onChange={(event) => setVerdict(event.currentTarget.value as Exclude<InvestmentThesisVerdict, "pending">)}>
                      <option value="supported">得到支持</option>
                      <option value="weakened">有所削弱</option>
                      <option value="inconclusive">暂无法确认</option>
                      <option value="invalidated">已经证伪</option>
                    </select>
                  </label>
                  <Show when={mode() === "corrected"}>
                    <label>
                      <span>修正后的动作</span>
                      <select value={correctedAction()} onChange={(event) => setCorrectedAction(event.currentTarget.value as InvestmentExposureAction)}>
                        <For each={Object.entries(ACTION_LABELS)}>{([value, label]) => <option value={value}>{label}</option>}</For>
                      </select>
                    </label>
                  </Show>
                  <Show when={mode() !== "accepted"}>
                    <label>
                      <span>主要错误类型</span>
                      <select value={errorKind()} onChange={(event) => setErrorKind(event.currentTarget.value as InvestmentDecisionErrorKind)}>
                        <For each={Object.entries(ERROR_LABELS)}>{([value, label]) => <option value={value}>{label}</option>}</For>
                      </select>
                    </label>
                    <label>
                      <span>错误程度</span>
                      <select value={errorSeverity()} onChange={(event) => setErrorSeverity(event.currentTarget.value as "minor" | "material" | "critical")}>
                        <option value="minor">轻微</option>
                        <option value="material">重要</option>
                        <option value="critical">关键</option>
                      </select>
                    </label>
                    <label class="is-wide">
                      <span>修正说明（必填）</span>
                      <textarea value={note()} maxlength={4000} onInput={(event) => setNote(event.currentTarget.value)} placeholder="当时哪一项判断应该怎样改？" />
                    </label>
                    <label class="is-wide">
                      <span>{mode() === "rejected" ? "证伪依据（必填）" : "错误依据（可选）"}</span>
                      <textarea value={errorExplanation()} maxlength={2000} onInput={(event) => setErrorExplanation(event.currentTarget.value)} placeholder="写清楚被什么事实证伪，避免只根据股价倒推。" />
                    </label>
                  </Show>
                </div>

                <button
                  type="button"
                  class="public-admin-decision-submit"
                  disabled={submitting() || !decisionReviewDraftIsValid(draft())}
                  onClick={() => void submitReview()}
                >
                  {submitting() ? "保存中…" : "确认复核并写入审计"}
                </button>
              </div>
            )}
          </Show>
        </div>
      </Show>
    </section>
  );
}
