import { For, Show, createMemo, createSignal, onMount } from "solid-js";
import {
  getHistoricalOutcomeGovernance,
  getHistoricalOutcomeDryRunAuthorizations,
  getHistoricalOutcomeDryRunImplementations,
  getHistoricalOutcomeDryRunFirstExecutionAuthorizations,
  getHistoricalOutcomeDryRunExecutionAttempts,
  getHistoricalOutcomeDryRunOutputValidations,
  getHistoricalOutcomeLabelAdmissionReviews,
  getHistoricalOutcomeLabelMaterializationImplementations,
  getHistoricalOutcomeLabelMaterializationRunAuthorizations,
  getHistoricalOutcomeLabelMaterializationIsolatedRunners,
  getHistoricalOutcomeLabelMaterializationFirstExecutionAuthorizations,
  getHistoricalOutcomeLabelMaterializationExecutionAttempts,
  getHistoricalOutcomeLabelMaterializationOutputValidations,
  getHistoricalOutcomeLabelWriteAuthorizations,
  getHistoricalOutcomeFormalLabelWrites,
  getHistoricalOutcomeFormalLabelValidations,
  getHistoricalOutcomeOfflineDatasets,
  getHistoricalOutcomeOfflineDatasetGovernance,
  getHistoricalOutcomeDryRunIsolatedRunners,
  getHistoricalOutcomeDryRunRunAuthorizations,
  getHistoricalOutcomeLabelers,
  getHistoricalOutcomePriceSnapshots,
  ingestHistoricalOutcomePriceSnapshot,
  registerHistoricalOutcomeLabeler,
  registerHistoricalOutcomeDryRunImplementation,
  registerHistoricalOutcomeDryRunIsolatedRunner,
  reviewHistoricalOutcomeDryRunAuthorization,
  reviewHistoricalOutcomeDryRunRunAuthorization,
  reviewHistoricalOutcomeDryRunFirstExecutionAuthorization,
  invokeHistoricalOutcomeDryRunOnce,
  validateHistoricalOutcomeDryRunOutput,
  reviewHistoricalOutcomeLabelAdmission,
  registerHistoricalOutcomeLabelMaterializationImplementation,
  reviewHistoricalOutcomeLabelMaterializationRunAuthorization,
  registerHistoricalOutcomeLabelMaterializationIsolatedRunner,
  reviewHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization,
  invokeHistoricalOutcomeLabelMaterializationOnce,
  validateHistoricalOutcomeLabelMaterializationOutput,
  reviewHistoricalOutcomeLabelWriteAuthorization,
  writeHistoricalOutcomeFormalLabelOnce,
  validateHistoricalOutcomeFormalLabel,
  assembleHistoricalOutcomeOfflineDataset,
  reviewHistoricalOutcomeOfflineDatasetGovernance,
  reviewHistoricalOutcomeGovernance,
  reviewHistoricalOutcomeLabeler,
} from "@/lib/api";
import type {
  HistoricalOutcomeGovernanceRegistry,
  HistoricalOutcomeGovernanceVerdict,
  HistoricalOutcomeDryRunAuthorizationRegistry,
  HistoricalOutcomeDryRunAuthorizationVerdict,
  HistoricalOutcomeDryRunImplementationRegistry,
  HistoricalOutcomeDryRunFirstExecutionAuthorizationRegistry,
  HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict,
  HistoricalOutcomeDryRunExecutionAttemptRegistry,
  HistoricalOutcomeDryRunOutputValidationRegistry,
  HistoricalOutcomeLabelAdmissionRegistry,
  HistoricalOutcomeLabelAdmissionVerdict,
  HistoricalOutcomeLabelMaterializationImplementationRegistry,
  HistoricalOutcomeLabelMaterializationRunAuthorizationRegistry,
  HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict,
  HistoricalOutcomeLabelMaterializationIsolatedRunnerRegistry,
  HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationRegistry,
  HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationVerdict,
  HistoricalOutcomeLabelMaterializationExecutionAttemptRegistry,
  HistoricalOutcomeLabelMaterializationOutputValidationRegistry,
  HistoricalOutcomeLabelWriteAuthorizationRegistry,
  HistoricalOutcomeLabelWriteAuthorizationVerdict,
  HistoricalOutcomeFormalLabelWriteRegistry,
  HistoricalOutcomeFormalLabelValidationRegistry,
  HistoricalOutcomeOfflineDatasetRegistry,
  HistoricalOutcomeOfflineDatasetGovernanceRegistry,
  HistoricalOutcomeOfflineDatasetGovernanceVerdict,
  HistoricalOutcomeDryRunIsolatedRunnerRegistry,
  HistoricalOutcomeDryRunRunAuthorizationRegistry,
  HistoricalOutcomeDryRunRunAuthorizationVerdict,
  HistoricalOutcomeLabelerRegistry,
  HistoricalOutcomeLabelerReviewVerdict,
  HistoricalOutcomePriceSnapshotRegistry,
} from "@/lib/types";
import { PublicAdminHistoricalOutcomeTransformationSpecPanel } from "./public-admin-historical-outcome-transformation-spec-panel";
import { PublicAdminHistoricalOutcomeTransformationSpecReviewPanel } from "./public-admin-historical-outcome-transformation-spec-review-panel";
import { PublicAdminHistoricalOutcomeTransformationImplementationPanel } from "./public-admin-historical-outcome-transformation-implementation-panel";
import { PublicAdminHistoricalOutcomeTransformationImplementationReviewPanel } from "./public-admin-historical-outcome-transformation-implementation-review-panel";
import { PublicAdminHistoricalOutcomeTransformationIsolatedRunnerPanel } from "./public-admin-historical-outcome-transformation-isolated-runner-panel";
import { PublicAdminHistoricalOutcomeTransformationFirstExecutionAuthorizationPanel } from "./public-admin-historical-outcome-transformation-first-execution-authorization-panel";
import { PublicAdminHistoricalOutcomeTransformationExecutionAttemptPanel } from "./public-admin-historical-outcome-transformation-execution-attempt-panel";
import { PublicAdminHistoricalOutcomeTransformationOutputValidationPanel } from "./public-admin-historical-outcome-transformation-output-validation-panel";
import { PublicAdminHistoricalOutcomeTransformationCandidateAdmissionPanel } from "./public-admin-historical-outcome-transformation-candidate-admission-panel";
import { PublicAdminHistoricalOutcomeTransformationOfficialArtifactMaterializationPanel } from "./public-admin-historical-outcome-transformation-official-artifact-materialization-panel";
import { PublicAdminHistoricalOutcomeTransformationOfficialArtifactOutputValidationPanel } from "./public-admin-historical-outcome-transformation-official-artifact-output-validation-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetSpecPanel } from "./public-admin-historical-outcome-feature-label-join-target-spec-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetSpecReviewPanel } from "./public-admin-historical-outcome-feature-label-join-target-spec-review-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetImplementationPanel } from "./public-admin-historical-outcome-feature-label-join-target-implementation-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetImplementationReviewPanel } from "./public-admin-historical-outcome-feature-label-join-target-implementation-review-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerPanel } from "./public-admin-historical-outcome-feature-label-join-target-isolated-runner-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationPanel } from "./public-admin-historical-outcome-feature-label-join-target-first-execution-authorization-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptPanel } from "./public-admin-historical-outcome-feature-label-join-target-execution-attempt-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetOutputValidationPanel } from "./public-admin-historical-outcome-feature-label-join-target-output-validation-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionPanel } from "./public-admin-historical-outcome-feature-label-join-target-candidate-admission-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationPanel } from "./public-admin-historical-outcome-feature-label-join-target-official-dataset-materialization-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationPanel } from "./public-admin-historical-outcome-feature-label-join-target-official-dataset-output-validation-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionPanel } from "./public-admin-historical-outcome-feature-label-join-target-training-store-copy-admission-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyPanel } from "./public-admin-historical-outcome-feature-label-join-target-training-store-copy-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationPanel } from "./public-admin-historical-outcome-feature-label-join-target-training-store-copy-output-validation-panel";
import { PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionPanel } from "./public-admin-historical-outcome-feature-label-join-target-training-registration-admission-panel";
import { PublicAdminHistoricalOutcomeTrainingExperimentRegistrationPanel } from "./public-admin-historical-outcome-training-experiment-registration-panel";
import { PublicAdminHistoricalOutcomeTrainingExperimentRegistrationReviewPanel } from "./public-admin-historical-outcome-training-experiment-registration-review-panel";
import { PublicAdminHistoricalOutcomeTrainingImplementationPanel } from "./public-admin-historical-outcome-training-implementation-panel";
import { PublicAdminHistoricalOutcomeTrainingImplementationReviewPanel } from "./public-admin-historical-outcome-training-implementation-review-panel";
import { PublicAdminHistoricalOutcomeTrainingIsolatedRunnerPanel } from "./public-admin-historical-outcome-training-isolated-runner-panel";
import { PublicAdminHistoricalOutcomeTrainingFirstExecutionAuthorizationPanel } from "./public-admin-historical-outcome-training-first-execution-authorization-panel";
import { PublicAdminHistoricalOutcomeTrainingExecutionAttemptPanel } from "./public-admin-historical-outcome-training-execution-attempt-panel";
import { PublicAdminHistoricalOutcomeTrainingOutputValidationPanel } from "./public-admin-historical-outcome-training-output-validation-panel";
import { PublicAdminHistoricalOutcomeValidationEvaluationImplementationPanel } from "./public-admin-historical-outcome-validation-evaluation-implementation-panel";
import { PublicAdminHistoricalOutcomeValidationEvaluationImplementationReviewPanel } from "./public-admin-historical-outcome-validation-evaluation-implementation-review-panel";
import { PublicAdminHistoricalOutcomeValidationEvaluationIsolatedRunnerPanel } from "./public-admin-historical-outcome-validation-evaluation-isolated-runner-panel";
import { PublicAdminHistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationPanel } from "./public-admin-historical-outcome-validation-evaluation-first-execution-authorization-panel";
import { PublicAdminHistoricalOutcomeValidationEvaluationExecutionAttemptPanel } from "./public-admin-historical-outcome-validation-evaluation-execution-attempt-panel";
import { PublicAdminHistoricalOutcomeValidationEvaluationOutputValidationPanel } from "./public-admin-historical-outcome-validation-evaluation-output-validation-panel";
import { PublicAdminHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionPanel } from "./public-admin-historical-outcome-validation-evaluation-per-target-candidate-admission-panel";
import { PublicAdminHistoricalOutcomeSealedHoldoutEvaluationProtocolReviewPanel } from "./public-admin-historical-outcome-sealed-holdout-evaluation-protocol-review-panel";
import { PublicAdminHistoricalOutcomeSealedHoldoutEvaluationImplementationPanel } from "./public-admin-historical-outcome-sealed-holdout-evaluation-implementation-panel";
import { PublicAdminHistoricalOutcomeSealedHoldoutEvaluationImplementationReviewPanel } from "./public-admin-historical-outcome-sealed-holdout-evaluation-implementation-review-panel";
import { PublicAdminHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerPanel } from "./public-admin-historical-outcome-sealed-holdout-evaluation-isolated-runner-panel";
import { PublicAdminHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationPanel } from "./public-admin-historical-outcome-sealed-holdout-evaluation-first-execution-authorization-panel";
import { PublicAdminHistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptPanel } from "./public-admin-historical-outcome-sealed-holdout-evaluation-execution-attempt-panel";
import { PublicAdminHistoricalOutcomeSealedHoldoutEvaluationOutputValidationPanel } from "./public-admin-historical-outcome-sealed-holdout-evaluation-output-validation-panel";
import { PublicAdminHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationPanel } from "./public-admin-historical-outcome-sealed-holdout-confirmatory-result-adjudication-panel";
import { PublicAdminControlledShadowExperimentDesignRegistrationPanel } from "./public-admin-controlled-shadow-experiment-design-registration-panel";
import { PublicAdminControlledShadowExperimentDesignRegistrationReviewPanel } from "./public-admin-controlled-shadow-experiment-design-registration-review-panel";
import { PublicAdminControlledShadowExperimentImplementationPanel } from "./public-admin-controlled-shadow-experiment-implementation-panel";
import { PublicAdminControlledShadowExperimentImplementationReviewPanel } from "./public-admin-controlled-shadow-experiment-implementation-review-panel";
import { PublicAdminControlledShadowExperimentIsolatedRunnerPanel } from "./public-admin-controlled-shadow-experiment-isolated-runner-panel";
import { PublicAdminControlledShadowExperimentFirstExecutionAuthorizationPanel } from "./public-admin-controlled-shadow-experiment-first-execution-authorization-panel";
import { PublicAdminControlledShadowExperimentExecutionAttemptPanel } from "./public-admin-controlled-shadow-experiment-execution-attempt-panel";
import { PublicAdminControlledShadowExperimentOutputValidationPanel } from "./public-admin-controlled-shadow-experiment-output-validation-panel";
import { PublicAdminControlledShadowForwardObservationProtocolRegistrationPanel } from "./public-admin-controlled-shadow-forward-observation-protocol-registration-panel";
import { PublicAdminControlledShadowForwardObservationProtocolRegistrationReviewPanel } from "./public-admin-controlled-shadow-forward-observation-protocol-registration-review-panel";
import { PublicAdminControlledShadowForwardObservationImplementationPanel } from "./public-admin-controlled-shadow-forward-observation-implementation-panel";
import { PublicAdminControlledShadowForwardObservationImplementationReviewPanel } from "./public-admin-controlled-shadow-forward-observation-implementation-review-panel";
import { PublicAdminControlledShadowForwardObservationIsolatedRunnerPanel } from "./public-admin-controlled-shadow-forward-observation-isolated-runner-panel";
import { PublicAdminControlledShadowForwardObservationFirstExecutionAuthorizationPanel } from "./public-admin-controlled-shadow-forward-observation-first-execution-authorization-panel";
import { PublicAdminControlledShadowForwardObservationExecutionAttemptPanel } from "./public-admin-controlled-shadow-forward-observation-execution-attempt-panel";
import { PublicAdminControlledShadowForwardObservationOutputValidationPanel } from "./public-admin-controlled-shadow-forward-observation-output-validation-panel";
import { PublicAdminControlledShadowFirstNaturalForwardCycleAuthorizationPanel } from "./public-admin-controlled-shadow-first-natural-forward-cycle-authorization-panel";
import { PublicAdminControlledShadowFirstNaturalForwardCycleClaimPanel } from "./public-admin-controlled-shadow-first-natural-forward-cycle-claim-panel";
import { PublicAdminControlledShadowMarketDataAdapterAuthorizationPanel } from "./public-admin-controlled-shadow-market-data-adapter-authorization-panel";
import { PublicAdminControlledShadowMarketDataReceiptAttemptPanel } from "./public-admin-controlled-shadow-market-data-receipt-attempt-panel";
import { PublicAdminControlledShadowMarketDataReceiptValidationPanel } from "./public-admin-controlled-shadow-market-data-receipt-validation-panel";
import { PublicAdminControlledShadowMarketDataParserSpecificationPanel } from "./public-admin-controlled-shadow-market-data-parser-specification-panel";
import { PublicAdminControlledShadowMarketDataParserSpecificationReviewPanel } from "./public-admin-controlled-shadow-market-data-parser-specification-review-panel";
import { PublicAdminControlledShadowMarketDataParserImplementationPanel } from "./public-admin-controlled-shadow-market-data-parser-implementation-panel";
import { PublicAdminControlledShadowMarketDataParserImplementationReviewPanel } from "./public-admin-controlled-shadow-market-data-parser-implementation-review-panel";
import { PublicAdminControlledShadowMarketDataParserIsolatedRunnerPanel } from "./public-admin-controlled-shadow-market-data-parser-isolated-runner-panel";
import { PublicAdminControlledShadowMarketDataParserFirstExecutionAuthorizationPanel } from "./public-admin-controlled-shadow-market-data-parser-first-execution-authorization-panel";
import { PublicAdminControlledShadowMarketDataParserExecutionAttemptClaimPanel } from "./public-admin-controlled-shadow-market-data-parser-execution-attempt-claim-panel";
import { PublicAdminControlledShadowMarketDataParserExecutionAttemptPanel } from "./public-admin-controlled-shadow-market-data-parser-execution-attempt-panel";
import { PublicAdminControlledShadowMarketDataParserOutputValidationPanel } from "./public-admin-controlled-shadow-market-data-parser-output-validation-panel";
import { PublicAdminControlledShadowObservationInputAdmissionPanel } from "./public-admin-controlled-shadow-observation-input-admission-panel";
import { PublicAdminControlledShadowObservationMaterializationSpecificationPanel } from "./public-admin-controlled-shadow-observation-materialization-specification-panel";
import { PublicAdminControlledShadowObservationMaterializationSpecificationReviewPanel } from "./public-admin-controlled-shadow-observation-materialization-specification-review-panel";
import { PublicAdminControlledShadowObservationMaterializationImplementationPanel } from "./public-admin-controlled-shadow-observation-materialization-implementation-panel";
import { PublicAdminControlledShadowObservationMaterializationImplementationReviewPanel } from "./public-admin-controlled-shadow-observation-materialization-implementation-review-panel";
import { PublicAdminControlledShadowObservationMaterializationIsolatedRunnerPanel } from "./public-admin-controlled-shadow-observation-materialization-isolated-runner-panel";
import { PublicAdminControlledShadowObservationMaterializationFirstExecutionAuthorizationPanel } from "./public-admin-controlled-shadow-observation-materialization-first-execution-authorization-panel";
import { PublicAdminControlledShadowObservationMaterializationExecutionAttemptClaimPanel } from "./public-admin-controlled-shadow-observation-materialization-execution-attempt-claim-panel";
import { PublicAdminControlledShadowObservationMaterializationExecutionAttemptPanel } from "./public-admin-controlled-shadow-observation-materialization-execution-attempt-panel";
import { PublicAdminControlledShadowObservationMaterializationOutputValidationPanel } from "./public-admin-controlled-shadow-observation-materialization-output-validation-panel";
import { PublicAdminControlledShadowObservationEvidenceAdmissionPanel } from "./public-admin-controlled-shadow-observation-evidence-admission-panel";
import { PublicAdminControlledShadowObservationLedgerTransitionSpecificationPanel } from "./public-admin-controlled-shadow-observation-ledger-transition-specification-panel";
import { PublicAdminControlledShadowObservationLedgerTransitionSpecificationReviewPanel } from "./public-admin-controlled-shadow-observation-ledger-transition-specification-review-panel";
import { PublicAdminControlledShadowObservationLedgerTransitionImplementationPanel } from "./public-admin-controlled-shadow-observation-ledger-transition-implementation-panel";
import { PublicAdminControlledShadowObservationLedgerTransitionImplementationReviewPanel } from "./public-admin-controlled-shadow-observation-ledger-transition-implementation-review-panel";
import { PublicAdminControlledShadowObservationLedgerTransitionIsolatedRunnerPanel } from "./public-admin-controlled-shadow-observation-ledger-transition-isolated-runner-panel";
import { PublicAdminControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationPanel } from "./public-admin-controlled-shadow-observation-ledger-transition-first-execution-authorization-panel";
import { PublicAdminControlledShadowObservationLedgerTransitionExecutionAttemptClaimPanel } from "./public-admin-controlled-shadow-observation-ledger-transition-execution-attempt-claim-panel";
import { PublicAdminControlledShadowObservationLedgerTransitionExecutionAttemptPanel } from "./public-admin-controlled-shadow-observation-ledger-transition-execution-attempt-panel";
import { PublicAdminControlledShadowObservationLedgerTransitionOutputValidationPanel } from "./public-admin-controlled-shadow-observation-ledger-transition-output-validation-panel";
import { PublicAdminControlledShadowObservationLedgerTransitionCandidateAdmissionPanel } from "./public-admin-controlled-shadow-observation-ledger-transition-candidate-admission-panel";
import { PublicAdminOpeningPortfolioSnapshotGovernanceSpecificationPanel } from "./public-admin-opening-portfolio-snapshot-governance-specification-panel";
import { PublicAdminOpeningPortfolioSnapshotGovernanceSpecificationReviewPanel } from "./public-admin-opening-portfolio-snapshot-governance-specification-review-panel";
import { PublicAdminOpeningPortfolioSourceArtifactReceiptImplementationPanel } from "./public-admin-opening-portfolio-source-artifact-receipt-implementation-panel";
import { PublicAdminOpeningPortfolioSourceArtifactReceiptImplementationReviewPanel } from "./public-admin-opening-portfolio-source-artifact-receipt-implementation-review-panel";
import { PublicAdminOpeningPortfolioSourceArtifactReceiptIsolatedReceiverPanel } from "./public-admin-opening-portfolio-source-artifact-receipt-isolated-receiver-panel";
import { PublicAdminOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationPanel } from "./public-admin-opening-portfolio-source-artifact-receipt-first-execution-authorization-panel";
import { PublicAdminOpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimPanel } from "./public-admin-opening-portfolio-source-artifact-receipt-execution-attempt-claim-panel";
import { PublicAdminOpeningPortfolioSourceArtifactReceiptExecutionAttemptPanel } from "./public-admin-opening-portfolio-source-artifact-receipt-execution-attempt-panel";
import { PublicAdminOpeningPortfolioSourceArtifactReceiptValidationPanel } from "./public-admin-opening-portfolio-source-artifact-receipt-validation-panel";
import { PublicAdminOpeningPortfolioSnapshotMaterializationImplementationPanel } from "./public-admin-opening-portfolio-snapshot-materialization-implementation-panel";
import { PublicAdminOpeningPortfolioSnapshotMaterializationImplementationReviewPanel } from "./public-admin-opening-portfolio-snapshot-materialization-implementation-review-panel";
import { PublicAdminOpeningPortfolioSnapshotMaterializationIsolatedMaterializerPanel } from "./public-admin-opening-portfolio-snapshot-materialization-isolated-materializer-panel";

const APPROVAL_CHECKS = [
  "协议在查看任何未来收益前已经冻结",
  "个股和 SPY 都使用同一数据源的复权收盘价",
  "20 / 60 / 250 日都按双方共同存在的交易日计算",
  "超额收益只使用同期 SPY，并保留个股绝对收益",
  "标签器不能读取决策时点之后的基本面或研究资料",
  "缺少任一必要价格时必须明确失败，不插值、不猜测",
] as const;

const LABELER_REVIEW_CHECKS = [
  "已核对实现指纹与登记代码版本",
  "实现仍绑定当前结果协议和人工审批",
  "只使用 FMP 复权收盘价和标的/SPY共同交易日",
  "相同封存输入必须得到相同结果",
  "决策时点状态与未来价格严格隔离",
  "缺价、缺共同交易日或来源失效必须失败关闭",
  "实现自身不联网、不调用外部工具、不写生产数据",
] as const;

const DRY_RUN_AUTHORIZATION_CHECKS = [
  "历史基准、结果协议、标签器实现和复核绑定仍然有效",
  "已重新核对封存快照与行情序列 SHA-256",
  "FMP 来源、复权收盘价口径和截止日期可追溯",
  "标的与 SPY 共同交易日完整覆盖 20 / 60 / 250 日",
  "确定性固定样例已经独立复算",
  "未来试运行只能写入隔离输出，不能改写历史状态",
  "当前不写收益标签、训练样本、奖励、影子持仓或交易数据",
] as const;

const DRY_RUN_RUN_AUTHORIZATION_CHECKS = [
  "已逐字核对试运行实现 ID、规范 SHA-256 与不可变代码版本",
  "上游授权、封存行情、七层状态、标签器和协议绑定仍然有效",
  "代码版本可从受控源码重建，且不会下载或替换运行代码",
  "执行器未来也只能读取当前封存输入，不能联网补数",
  "共同交易日、20 / 60 / 250 日与四项指标可确定性复现",
  "未来输出必须写入一次性隔离区，不能直接成为结果标签",
  "并发、内存、时限与缺失失败边界已经明确",
  "联网和外部工具保持关闭",
  "生产、标签、训练、奖励和影子写入保持关闭",
  "订单、券商访问和交易权限保持关闭",
] as const;

const FIRST_EXECUTION_AUTHORIZATION_CHECKS = [
  "已核对隔离执行器 ID、规范 SHA-256 与固定运行边界",
  "运行复核、实现、封存行情、七层状态、标签器与协议绑定仍有效",
  "已从独立渠道复算并核对执行器制品 SHA-256",
  "制品能够从受控源码重建，并已确认当前实际可用",
  "输入挂载与根文件系统保持只读",
  "执行身份非特权且启用 no-new-privileges",
  "输出只进入一次性临时区，并必须独立校验",
  "300 秒、512 MiB、1 核、单进程和 1 MiB 输出上限保持固定",
  "不继承宿主环境变量，也不提供任何密钥",
  "联网与外部工具保持关闭",
  "生产、历史、标签、训练、奖励与影子写入保持关闭",
  "订单、券商访问与交易权限保持关闭",
  "授权只允许 24 小时内一次首次执行，逾期或消费后自动失效",
] as const;

const MATERIALIZATION_FIRST_EXECUTION_AUTHORIZATION_CHECKS = [
  "已逐字核对物化隔离 runner ID、规范 SHA-256 与固定运行边界",
  "物化运行复核、实现、准入、校验、输出、快照和协议绑定仍然有效",
  "已从独立渠道复算并核对 runner 制品 SHA-256",
  "制品能够从受控源码重建，并已确认当前实际可用",
  "输入挂载与根文件系统保持只读",
  "执行身份非特权且启用 no-new-privileges",
  "未来输出只允许 create-once 写入一次性隔离区，并必须独立校验",
  "300 秒、512 MiB、1 核、单进程和 1 MiB 输出上限保持固定",
  "不继承宿主环境变量，也不提供任何密钥",
  "联网、外部工具与子进程保持关闭",
  "只允许逐位原始结果信封，不推断方向、评级、动作、仓位或奖励",
  "生产、历史、标签、训练、奖励与影子写入保持关闭",
  "订单、券商访问与交易权限保持关闭",
  "授权只允许 24 小时内一次未来首次执行，逾期或消费后自动失效",
] as const;

const LABEL_ADMISSION_CHECKS = [
  "已核对当前独立校验记录与精确 claim、result、output、快照和协议绑定",
  "冻结结果协议适用于这条历史判断与标的",
  "20 / 60 / 250 日窗口、共同交易日起点和每个终点均已核对",
  "复权收盘价口径和公司行动处理边界已核对",
  "SPY 与标的在相同共同交易日上具有可比性",
  "判断可用时间、未来信息隔离和结果观察时间均已核对",
  "缺失数据、样本选择与幸存者偏差已经审阅并写入局限",
  "没有人工覆盖、四舍五入或改写任何重算指标",
  "本次不从收益数字自动推断方向、评级、动作或奖励语义",
  "标签物化、训练、奖励、影子、订单、券商和交易权限保持关闭",
] as const;

const MATERIALIZATION_RUN_AUTHORIZATION_CHECKS = [
  "已逐字核对物化实现 ID、规范 SHA-256 与不可变代码版本",
  "准入复核、独立校验、原始输出、封存行情与协议绑定仍然有效",
  "代码版本可从受控源码独立复现，且不会下载或替换运行代码",
  "未来实现只能逐位封装已验证的原始结果信封",
  "收益、SPY 收益、超额收益与最大回撤必须逐位保留，不重算、不舍入、不覆盖",
  "完整来源绑定和已知局限必须原样保留",
  "未来输出只能 create-once 写入隔离区",
  "任一输入缺失、冲突或绑定失效必须失败关闭",
  "联网、外部工具、生产读取、生产写入与历史修改保持关闭",
  "不得推断方向、评级、动作、仓位或奖励语义",
  "标签写入、训练、奖励、影子、订单、券商和交易权限保持关闭",
] as const;

const FORMAL_LABEL_WRITE_AUTHORIZATION_CHECKS = [
  "已核对第十九阶段校验、claim、result、output 与完整上游绑定",
  "复核人不属于物化、准入、校验或执行链中的任何既有角色",
  "正式标签 schema 只承载已验证的原始绝对与相对市场结果",
  "不从结果自动推断方向、评级、动作、仓位或奖励语义",
  "20 / 60 / 250 日指标位模式和来源已经逐项复核",
  "已知局限必须原样进入正式标签，不能删改或淡化",
  "未来 writer 必须 create-once，禁止覆盖和原地修订",
  "授权最多使用一次，并在提交后 24 小时整失效",
  "正式标签存储必须与训练数据和奖励证据物理隔离",
  "本阶段不赋予任何语义推断或奖励生成权限",
  "本阶段不开放联网、外部工具或无关生产能力",
  "训练、影子、订单、券商和真实交易权限全部关闭",
] as const;

const OFFLINE_DATASET_ASSEMBLY_CHECKS = [
  "装配当前完整候选集，不允许人工挑选或遗漏通过项",
  "新版本严格保留上一版本全部条目，只追加新候选",
  "每条记录的点时来源、正式标签与独立校验血缘完整保留",
  "本阶段不推断语义目标、不划分训练/验证/测试集",
  "本阶段不授权训练、奖励、影子、订单、券商或交易",
] as const;

const OFFLINE_DATASET_GOVERNANCE_CHECKS = [
  "已核对当前完整候选集、数据集内容 SHA 与 manifest 的精确绑定",
  "复核人未参与数据集装配、标签写入、独立校验或此前上游链路",
  "完整候选集、不可变版本和单调追加血缘均已复核",
  "公司、历史事件和来源身份形成不可拆分连通分量，不能跨切分泄漏",
  "未来切分采用确定性 70 / 15 / 15，并对封存留出集隐藏标签",
  "时间顺序和最长 250 个交易日的 purge / embargo 已冻结",
  "所有未来特征必须在历史判断时点已经可获得",
  "特征制品 SHA、来源、版本和 available_at 必须完整保留",
  "结果、标签、校验、准入、数据集和未来行情字段不得成为特征",
  "缺少或无法判定 available_at 时失败关闭，不回填、不插值",
  "本阶段不切分、不连接特征、不生成目标，也不授权训练、奖励、影子、订单、券商或交易",
] as const;

const formatRate = (value: number) => `${(value * 100).toFixed(1)}%`;

export function PublicAdminHistoricalOutcomeGovernancePanel() {
  const [registry, setRegistry] = createSignal<HistoricalOutcomeGovernanceRegistry>();
  const [labelers, setLabelers] = createSignal<HistoricalOutcomeLabelerRegistry>();
  const [priceSnapshots, setPriceSnapshots] = createSignal<HistoricalOutcomePriceSnapshotRegistry>();
  const [authorizations, setAuthorizations] = createSignal<HistoricalOutcomeDryRunAuthorizationRegistry>();
  const [dryRunImplementations, setDryRunImplementations] = createSignal<HistoricalOutcomeDryRunImplementationRegistry>();
  const [runAuthorizations, setRunAuthorizations] = createSignal<HistoricalOutcomeDryRunRunAuthorizationRegistry>();
  const [isolatedRunners, setIsolatedRunners] = createSignal<HistoricalOutcomeDryRunIsolatedRunnerRegistry>();
  const [firstExecutionAuthorizations, setFirstExecutionAuthorizations] = createSignal<HistoricalOutcomeDryRunFirstExecutionAuthorizationRegistry>();
  const [executionAttempts, setExecutionAttempts] = createSignal<HistoricalOutcomeDryRunExecutionAttemptRegistry>();
  const [outputValidations, setOutputValidations] = createSignal<HistoricalOutcomeDryRunOutputValidationRegistry>();
  const [labelAdmissions, setLabelAdmissions] = createSignal<HistoricalOutcomeLabelAdmissionRegistry>();
  const [labelMaterializationImplementations, setLabelMaterializationImplementations] = createSignal<HistoricalOutcomeLabelMaterializationImplementationRegistry>();
  const [materializationRunAuthorizations, setMaterializationRunAuthorizations] = createSignal<HistoricalOutcomeLabelMaterializationRunAuthorizationRegistry>();
  const [materializationIsolatedRunners, setMaterializationIsolatedRunners] = createSignal<HistoricalOutcomeLabelMaterializationIsolatedRunnerRegistry>();
  const [materializationFirstExecutionAuthorizations, setMaterializationFirstExecutionAuthorizations] = createSignal<HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationRegistry>();
  const [materializationExecutionAttempts, setMaterializationExecutionAttempts] = createSignal<HistoricalOutcomeLabelMaterializationExecutionAttemptRegistry>();
  const [verdict, setVerdict] = createSignal<HistoricalOutcomeGovernanceVerdict>("approved_for_implementation_review");
  const [rationale, setRationale] = createSignal("");
  const [checks, setChecks] = createSignal(APPROVAL_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");
  const [implementationName, setImplementationName] = createSignal("共同交易日复权收盘价确定性标签器");
  const [codeRevision, setCodeRevision] = createSignal("");
  const [selectedImplementationId, setSelectedImplementationId] = createSignal("");
  const [labelerVerdict, setLabelerVerdict] = createSignal<HistoricalOutcomeLabelerReviewVerdict>("approved_for_offline_dry_run_authorization_review");
  const [labelerRationale, setLabelerRationale] = createSignal("");
  const [labelerChecks, setLabelerChecks] = createSignal(LABELER_REVIEW_CHECKS.map(() => false));
  const [selectedBenchmarkStateId, setSelectedBenchmarkStateId] = createSignal("");
  const [selectedSnapshotLabelerId, setSelectedSnapshotLabelerId] = createSignal("");
  const [selectedSnapshotId, setSelectedSnapshotId] = createSignal("");
  const [authorizationVerdict, setAuthorizationVerdict] = createSignal<HistoricalOutcomeDryRunAuthorizationVerdict>("approved_for_dry_run_implementation_registration");
  const [authorizationRationale, setAuthorizationRationale] = createSignal("");
  const [authorizationChecks, setAuthorizationChecks] = createSignal(DRY_RUN_AUTHORIZATION_CHECKS.map(() => false));
  const [selectedDryRunAuthorizationReviewId, setSelectedDryRunAuthorizationReviewId] = createSignal("");
  const [dryRunImplementationName, setDryRunImplementationName] = createSignal("共同交易日隔离试运行实现");
  const [dryRunCodeRevision, setDryRunCodeRevision] = createSignal("");
  const [selectedRunImplementationId, setSelectedRunImplementationId] = createSignal("");
  const [runAuthorizationVerdict, setRunAuthorizationVerdict] = createSignal<HistoricalOutcomeDryRunRunAuthorizationVerdict>("approved_for_isolated_runner_registration");
  const [runAuthorizationRationale, setRunAuthorizationRationale] = createSignal("");
  const [runAuthorizationChecks, setRunAuthorizationChecks] = createSignal(DRY_RUN_RUN_AUTHORIZATION_CHECKS.map(() => false));
  const [selectedRunnerAuthorizationReviewId, setSelectedRunnerAuthorizationReviewId] = createSignal("");
  const [runnerName, setRunnerName] = createSignal("一次性确定性历史结果执行器");
  const [runnerCodeRevision, setRunnerCodeRevision] = createSignal("");
  const [runnerArtifactSha256, setRunnerArtifactSha256] = createSignal("");
  const [selectedFirstExecutionRunnerId, setSelectedFirstExecutionRunnerId] = createSignal("");
  const [firstExecutionVerdict, setFirstExecutionVerdict] = createSignal<HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict>("approved_for_one_shot_first_execution");
  const [firstExecutionRationale, setFirstExecutionRationale] = createSignal("");
  const [firstExecutionChecks, setFirstExecutionChecks] = createSignal(FIRST_EXECUTION_AUTHORIZATION_CHECKS.map(() => false));
  const [selectedLabelAdmissionAttemptId, setSelectedLabelAdmissionAttemptId] = createSignal("");
  const [labelAdmissionVerdict, setLabelAdmissionVerdict] = createSignal<HistoricalOutcomeLabelAdmissionVerdict>("approved_for_future_label_materialization");
  const [labelAdmissionRationale, setLabelAdmissionRationale] = createSignal("");
  const [labelAdmissionLimitations, setLabelAdmissionLimitations] = createSignal("");
  const [labelAdmissionChecks, setLabelAdmissionChecks] = createSignal(LABEL_ADMISSION_CHECKS.map(() => false));
  const [selectedMaterializationAdmissionAttemptId, setSelectedMaterializationAdmissionAttemptId] = createSignal("");
  const [materializationImplementationName, setMaterializationImplementationName] = createSignal("原始已验证结果信封物化器");
  const [materializationCodeRevision, setMaterializationCodeRevision] = createSignal("");
  const [selectedMaterializationRunImplementationId, setSelectedMaterializationRunImplementationId] = createSignal("");
  const [materializationRunAuthorizationVerdict, setMaterializationRunAuthorizationVerdict] = createSignal<HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict>("approved_for_materialization_runner_registration");
  const [materializationRunAuthorizationRationale, setMaterializationRunAuthorizationRationale] = createSignal("");
  const [materializationRunAuthorizationChecks, setMaterializationRunAuthorizationChecks] = createSignal(MATERIALIZATION_RUN_AUTHORIZATION_CHECKS.map(() => false));
  const [selectedMaterializationRunnerAuthorizationReviewId, setSelectedMaterializationRunnerAuthorizationReviewId] = createSignal("");
  const [materializationRunnerName, setMaterializationRunnerName] = createSignal("一次性确定性标签物化 runner");
  const [materializationRunnerCodeRevision, setMaterializationRunnerCodeRevision] = createSignal("");
  const [materializationRunnerArtifactSha256, setMaterializationRunnerArtifactSha256] = createSignal("");
  const [selectedMaterializationFirstExecutionRunnerId, setSelectedMaterializationFirstExecutionRunnerId] = createSignal("");
  const [materializationFirstExecutionVerdict, setMaterializationFirstExecutionVerdict] = createSignal<HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationVerdict>("approved_for_one_shot_first_execution");
  const [materializationFirstExecutionRationale, setMaterializationFirstExecutionRationale] = createSignal("");
  const [materializationFirstExecutionChecks, setMaterializationFirstExecutionChecks] = createSignal(MATERIALIZATION_FIRST_EXECUTION_AUTHORIZATION_CHECKS.map(() => false));
  const [materializationOutputValidations, setMaterializationOutputValidations] = createSignal<HistoricalOutcomeLabelMaterializationOutputValidationRegistry>();
  const [labelWriteAuthorizations, setLabelWriteAuthorizations] = createSignal<HistoricalOutcomeLabelWriteAuthorizationRegistry>();
  const [selectedLabelWriteValidationId, setSelectedLabelWriteValidationId] = createSignal("");
  const [labelWriteAuthorizationVerdict, setLabelWriteAuthorizationVerdict] = createSignal<HistoricalOutcomeLabelWriteAuthorizationVerdict>("approved_for_one_shot_formal_label_write");
  const [labelWriteAuthorizationRationale, setLabelWriteAuthorizationRationale] = createSignal("");
  const [labelWriteAuthorizationChecks, setLabelWriteAuthorizationChecks] = createSignal(FORMAL_LABEL_WRITE_AUTHORIZATION_CHECKS.map(() => false));
  const [formalLabelWrites, setFormalLabelWrites] = createSignal<HistoricalOutcomeFormalLabelWriteRegistry>();
  const [selectedFormalLabelAuthorizationReviewId, setSelectedFormalLabelAuthorizationReviewId] = createSignal("");
  const [formalLabelValidations, setFormalLabelValidations] = createSignal<HistoricalOutcomeFormalLabelValidationRegistry>();
  const [selectedFormalLabelId, setSelectedFormalLabelId] = createSignal("");
  const [offlineDatasets, setOfflineDatasets] = createSignal<HistoricalOutcomeOfflineDatasetRegistry>();
  const [offlineDatasetAssemblyChecks, setOfflineDatasetAssemblyChecks] = createSignal(OFFLINE_DATASET_ASSEMBLY_CHECKS.map(() => false));
  const [offlineDatasetGovernance, setOfflineDatasetGovernance] = createSignal<HistoricalOutcomeOfflineDatasetGovernanceRegistry>();
  const [selectedOfflineDatasetGovernanceId, setSelectedOfflineDatasetGovernanceId] = createSignal("");
  const [offlineDatasetGovernanceVerdict, setOfflineDatasetGovernanceVerdict] = createSignal<HistoricalOutcomeOfflineDatasetGovernanceVerdict>("approved_for_split_and_point_in_time_feature_join_spec_registration");
  const [offlineDatasetGovernanceRationale, setOfflineDatasetGovernanceRationale] = createSignal("");
  const [offlineDatasetGovernanceLimitations, setOfflineDatasetGovernanceLimitations] = createSignal("");
  const [offlineDatasetGovernanceChecks, setOfflineDatasetGovernanceChecks] = createSignal(OFFLINE_DATASET_GOVERNANCE_CHECKS.map(() => false));

  const load = async () => {
    const [governance, implementations, snapshots, dryRunAuthorizations, registeredDryRuns, reviewedRuns, registeredRunners, firstExecutionReviews, attempts, validations, admissions, materializationImplementations, materializationAuthorizationReviews, materializationRunners, materializationFirstExecutionReviews, materializationAttempts, materializationValidations, formalLabelWriteAuthorizations, formalLabelWriteRegistry, formalLabelValidationRegistry, offlineDatasetRegistry, offlineDatasetGovernanceRegistry] = await Promise.all([
      getHistoricalOutcomeGovernance(),
      getHistoricalOutcomeLabelers(),
      getHistoricalOutcomePriceSnapshots(),
      getHistoricalOutcomeDryRunAuthorizations(),
      getHistoricalOutcomeDryRunImplementations(),
      getHistoricalOutcomeDryRunRunAuthorizations(),
      getHistoricalOutcomeDryRunIsolatedRunners(),
      getHistoricalOutcomeDryRunFirstExecutionAuthorizations(),
      getHistoricalOutcomeDryRunExecutionAttempts(),
      getHistoricalOutcomeDryRunOutputValidations(),
      getHistoricalOutcomeLabelAdmissionReviews(),
      getHistoricalOutcomeLabelMaterializationImplementations(),
      getHistoricalOutcomeLabelMaterializationRunAuthorizations(),
      getHistoricalOutcomeLabelMaterializationIsolatedRunners(),
      getHistoricalOutcomeLabelMaterializationFirstExecutionAuthorizations(),
      getHistoricalOutcomeLabelMaterializationExecutionAttempts(),
      getHistoricalOutcomeLabelMaterializationOutputValidations(),
      getHistoricalOutcomeLabelWriteAuthorizations(),
      getHistoricalOutcomeFormalLabelWrites(),
      getHistoricalOutcomeFormalLabelValidations(),
      getHistoricalOutcomeOfflineDatasets(),
      getHistoricalOutcomeOfflineDatasetGovernance(),
    ]);
    setRegistry(governance);
    setLabelers(implementations);
    setPriceSnapshots(snapshots);
    setAuthorizations(dryRunAuthorizations);
    setDryRunImplementations(registeredDryRuns);
    setRunAuthorizations(reviewedRuns);
    setIsolatedRunners(registeredRunners);
    setFirstExecutionAuthorizations(firstExecutionReviews);
    setExecutionAttempts(attempts);
    setOutputValidations(validations);
    setLabelAdmissions(admissions);
    setLabelMaterializationImplementations(materializationImplementations);
    setMaterializationRunAuthorizations(materializationAuthorizationReviews);
    setMaterializationIsolatedRunners(materializationRunners);
    setMaterializationFirstExecutionAuthorizations(materializationFirstExecutionReviews);
    setMaterializationExecutionAttempts(materializationAttempts);
    setMaterializationOutputValidations(materializationValidations);
    setLabelWriteAuthorizations(formalLabelWriteAuthorizations);
    setFormalLabelWrites(formalLabelWriteRegistry);
    setFormalLabelValidations(formalLabelValidationRegistry);
    setOfflineDatasets(offlineDatasetRegistry);
    setOfflineDatasetGovernance(offlineDatasetGovernanceRegistry);
  };
  onMount(() => void load().catch((cause) => setError(cause instanceof Error ? cause.message : "历史结果协议读取失败")));

  const approvalSelected = createMemo(() => verdict() === "approved_for_implementation_review");
  const approvalBlocked = createMemo(() => (registry()?.benchmark_ready_count ?? 0) === 0);
  const submitDisabled = createMemo(() => busy()
    || !rationale().trim()
    || (approvalSelected() && (approvalBlocked() || checks().some((value) => !value))));
  const registerDisabled = createMemo(() => busy()
    || !labelers()?.registration_allowed
    || !labelers()?.current_governance_review_id
    || !implementationName().trim()
    || !codeRevision().trim());
  const selectedImplementation = createMemo(() => labelers()?.implementations.find(
    (item) => item.implementation.implementation_id === selectedImplementationId(),
  ));
  const labelerApprovalSelected = createMemo(() => labelerVerdict() === "approved_for_offline_dry_run_authorization_review");
  const labelerReviewDisabled = createMemo(() => busy()
    || !selectedImplementation()
    || !labelerRationale().trim()
    || (labelerApprovalSelected() && (
      !selectedImplementation()?.governance_binding_current
      || labelerChecks().some((value) => !value)
    )));
  const selectedBenchmarkState = createMemo(() => priceSnapshots()?.eligible_benchmark_states.find(
    (item) => item.reconstruction_id === selectedBenchmarkStateId(),
  ));
  const selectedSnapshotLabeler = createMemo(() => priceSnapshots()?.eligible_labelers.find(
    (item) => item.implementation_id === selectedSnapshotLabelerId(),
  ));
  const ingestDisabled = createMemo(() => busy() || !selectedBenchmarkState() || !selectedSnapshotLabeler());
  const selectedAuthorization = createMemo(() => authorizations()?.items.find(
    (item) => item.snapshot_id === selectedSnapshotId(),
  ));
  const authorizationApprovalSelected = createMemo(() => authorizationVerdict() === "approved_for_dry_run_implementation_registration");
  const authorizationDisabled = createMemo(() => busy()
    || !selectedAuthorization()
    || !authorizationRationale().trim()
    || (authorizationApprovalSelected() && (
      !selectedAuthorization()?.current_binding
      || authorizationChecks().some((value) => !value)
    )));
  const selectedDryRunAuthorization = createMemo(() => dryRunImplementations()?.eligible_authorizations.find(
    (item) => item.authorization_review_id === selectedDryRunAuthorizationReviewId(),
  ));
  const dryRunImplementationRegisterDisabled = createMemo(() => busy()
    || !dryRunImplementations()?.registration_allowed
    || !selectedDryRunAuthorization()
    || !dryRunImplementationName().trim()
    || !dryRunCodeRevision().trim());
  const selectedRunAuthorization = createMemo(() => runAuthorizations()?.items.find(
    (item) => item.implementation.dry_run_implementation_id === selectedRunImplementationId(),
  ));
  const runAuthorizationApprovalSelected = createMemo(() => runAuthorizationVerdict() === "approved_for_isolated_runner_registration");
  const runAuthorizationDisabled = createMemo(() => busy()
    || !selectedRunAuthorization()
    || !runAuthorizationRationale().trim()
    || (runAuthorizationApprovalSelected() && (
      !selectedRunAuthorization()?.current_binding
      || runAuthorizationChecks().some((value) => !value)
    )));
  const selectedRunnerAuthorization = createMemo(() => isolatedRunners()?.eligible_authorizations.find(
    (item) => item.review.review_id === selectedRunnerAuthorizationReviewId(),
  ));
  const runnerRegisterDisabled = createMemo(() => busy()
    || !isolatedRunners()?.registration_allowed
    || !selectedRunnerAuthorization()
    || !runnerName().trim()
    || !runnerCodeRevision().trim()
    || !/^[0-9a-fA-F]{64}$/.test(runnerArtifactSha256().trim()));
  const selectedFirstExecutionAuthorization = createMemo(() => firstExecutionAuthorizations()?.items.find(
    (item) => item.runner.isolated_runner_id === selectedFirstExecutionRunnerId(),
  ));
  const firstExecutionApprovalSelected = createMemo(() => firstExecutionVerdict() === "approved_for_one_shot_first_execution");
  const firstExecutionReviewDisabled = createMemo(() => busy()
    || !selectedFirstExecutionAuthorization()
    || !firstExecutionRationale().trim()
    || (firstExecutionApprovalSelected() && (
      !selectedFirstExecutionAuthorization()?.current_binding
      || firstExecutionChecks().some((value) => !value)
    )));
  const selectedLabelAdmission = createMemo(() => labelAdmissions()?.items.find(
    (item) => item.validation.attempt_id === selectedLabelAdmissionAttemptId(),
  ));
  const labelAdmissionApprovalSelected = createMemo(() => labelAdmissionVerdict() === "approved_for_future_label_materialization");
  const labelAdmissionReviewDisabled = createMemo(() => busy()
    || !selectedLabelAdmission()
    || !labelAdmissionRationale().trim()
    || !labelAdmissionLimitations().trim()
    || (labelAdmissionApprovalSelected() && (
      !selectedLabelAdmission()?.current_binding
      || labelAdmissionChecks().some((value) => !value)
    )));
  const selectedMaterializationAdmission = createMemo(() => labelMaterializationImplementations()?.eligible_admissions.find(
    (item) => item.attempt_id === selectedMaterializationAdmissionAttemptId(),
  ));
  const materializationImplementationRegisterDisabled = createMemo(() => busy()
    || !labelMaterializationImplementations()?.registration_allowed
    || !selectedMaterializationAdmission()
    || !materializationImplementationName().trim()
    || !materializationCodeRevision().trim());
  const selectedMaterializationRunAuthorization = createMemo(() => materializationRunAuthorizations()?.items.find(
    (item) => item.implementation.materialization_implementation_id === selectedMaterializationRunImplementationId(),
  ));
  const materializationRunApprovalSelected = createMemo(() => materializationRunAuthorizationVerdict() === "approved_for_materialization_runner_registration");
  const materializationRunAuthorizationReviewDisabled = createMemo(() => busy()
    || !selectedMaterializationRunAuthorization()
    || !materializationRunAuthorizationRationale().trim()
    || (materializationRunApprovalSelected() && (
      !selectedMaterializationRunAuthorization()?.current_binding
      || materializationRunAuthorizationChecks().some((value) => !value)
    )));
  const selectedMaterializationRunnerAuthorization = createMemo(() => materializationIsolatedRunners()?.eligible_authorizations.find(
    (item) => item.review.review_id === selectedMaterializationRunnerAuthorizationReviewId(),
  ));
  const materializationRunnerRegisterDisabled = createMemo(() => busy()
    || !materializationIsolatedRunners()?.registration_allowed
    || !selectedMaterializationRunnerAuthorization()
    || !materializationRunnerName().trim()
    || !materializationRunnerCodeRevision().trim()
    || !/^[0-9a-fA-F]{64}$/.test(materializationRunnerArtifactSha256().trim()));
  const selectedMaterializationFirstExecutionAuthorization = createMemo(() => materializationFirstExecutionAuthorizations()?.items.find(
    (item) => item.runner.isolated_runner_id === selectedMaterializationFirstExecutionRunnerId(),
  ));
  const materializationFirstExecutionApprovalSelected = createMemo(() => materializationFirstExecutionVerdict() === "approved_for_one_shot_first_execution");
  const materializationFirstExecutionReviewDisabled = createMemo(() => busy()
    || !selectedMaterializationFirstExecutionAuthorization()
    || !materializationFirstExecutionRationale().trim()
    || (materializationFirstExecutionApprovalSelected() && (
      !selectedMaterializationFirstExecutionAuthorization()?.current_binding
      || materializationFirstExecutionChecks().some((value) => !value)
    )));
  const invokableMaterializationAuthorization = createMemo(() => {
    const consumedReviewIds = new Set(
      materializationExecutionAttempts()?.attempts.map((item) => item.claim.authorization_review_id) ?? [],
    );
    return materializationFirstExecutionAuthorizations()?.items.find((item) =>
      item.current_binding
      && item.authorization_unexpired
      && item.latest_review?.one_shot_first_execution_authorized
      && !consumedReviewIds.has(item.latest_review.review_id),
    );
  });
  const materializationInvocationDisabled = createMemo(() => busy()
    || !materializationExecutionAttempts()?.invocation_endpoint_available
    || (materializationExecutionAttempts()?.invocation_eligible_authorization_count ?? 0) < 1
    || !invokableMaterializationAuthorization()?.latest_review);
  const eligibleMaterializationOutputValidation = createMemo(() =>
    materializationOutputValidations()?.items.find((item) => item.validation_eligible),
  );
  const materializationOutputValidationDisabled = createMemo(() => busy()
    || !materializationOutputValidations()?.output_validation_available
    || !eligibleMaterializationOutputValidation()?.attempt.result.output_sha256);
  const materializationOutputValidationForAttempt = (attemptId: string) =>
    materializationOutputValidations()?.items.find(
      (item) => item.attempt.claim.attempt_id === attemptId,
    )?.validation;
  const selectedLabelWriteAuthorization = createMemo(() =>
    labelWriteAuthorizations()?.items.find(
      (item) => item.materialization_validation_id === selectedLabelWriteValidationId(),
    ),
  );
  const labelWriteApprovalSelected = createMemo(() =>
    labelWriteAuthorizationVerdict() === "approved_for_one_shot_formal_label_write",
  );
  const labelWriteAuthorizationDisabled = createMemo(() => busy()
    || !selectedLabelWriteAuthorization()?.current_binding
    || !labelWriteAuthorizationRationale().trim()
    || (labelWriteApprovalSelected()
      && labelWriteAuthorizationChecks().some((value) => !value)));
  const selectedFormalLabelWriteAuthorization = createMemo(() =>
    formalLabelWrites()?.eligible_authorizations.find(
      (item) => item.authorization_review_id === selectedFormalLabelAuthorizationReviewId(),
    ),
  );
  const formalLabelWriteDisabled = createMemo(() => busy()
    || !formalLabelWrites()?.writer_endpoint_available
    || !selectedFormalLabelWriteAuthorization());
  const selectedFormalLabelForValidation = createMemo(() =>
    formalLabelValidations()?.items.find(
      (item) => item.validation_eligible && item.formal_label.label.label_id === selectedFormalLabelId(),
    ),
  );
  const formalLabelValidationDisabled = createMemo(() => busy()
    || !formalLabelValidations()?.validation_available
    || !selectedFormalLabelForValidation());
  const offlineDatasetAssemblyDisabled = createMemo(() => busy()
    || !offlineDatasets()?.assembly_available
    || offlineDatasetAssemblyChecks().some((value) => !value));
  const selectedOfflineDatasetGovernance = createMemo(() =>
    offlineDatasetGovernance()?.items.find(
      (item) => item.subject.dataset_id === selectedOfflineDatasetGovernanceId(),
    ),
  );
  const offlineDatasetGovernanceApprovalSelected = createMemo(() =>
    offlineDatasetGovernanceVerdict()
      === "approved_for_split_and_point_in_time_feature_join_spec_registration",
  );
  const offlineDatasetGovernanceDisabled = createMemo(() => busy()
    || !selectedOfflineDatasetGovernance()?.review_eligible
    || !offlineDatasetGovernanceRationale().trim()
    || !offlineDatasetGovernanceLimitations().trim()
    || (offlineDatasetGovernanceApprovalSelected()
      && offlineDatasetGovernanceChecks().some((value) => !value)));

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) => current.map((value, itemIndex) => itemIndex === index ? checked : value));
  };
  const toggleLabelerCheck = (index: number, checked: boolean) => {
    setLabelerChecks((current) => current.map((value, itemIndex) => itemIndex === index ? checked : value));
  };
  const toggleAuthorizationCheck = (index: number, checked: boolean) => {
    setAuthorizationChecks((current) => current.map((value, itemIndex) => itemIndex === index ? checked : value));
  };
  const toggleRunAuthorizationCheck = (index: number, checked: boolean) => {
    setRunAuthorizationChecks((current) => current.map((value, itemIndex) => itemIndex === index ? checked : value));
  };
  const toggleFirstExecutionCheck = (index: number, checked: boolean) => {
    setFirstExecutionChecks((current) => current.map((value, itemIndex) => itemIndex === index ? checked : value));
  };
  const toggleLabelAdmissionCheck = (index: number, checked: boolean) => {
    setLabelAdmissionChecks((current) => current.map((value, itemIndex) => itemIndex === index ? checked : value));
  };
  const toggleLabelWriteAuthorizationCheck = (index: number, checked: boolean) => {
    setLabelWriteAuthorizationChecks((current) => current.map(
      (value, itemIndex) => itemIndex === index ? checked : value,
    ));
  };
  const toggleOfflineDatasetAssemblyCheck = (index: number, checked: boolean) => {
    setOfflineDatasetAssemblyChecks((current) => current.map(
      (value, itemIndex) => itemIndex === index ? checked : value,
    ));
  };
  const toggleOfflineDatasetGovernanceCheck = (index: number, checked: boolean) => {
    setOfflineDatasetGovernanceChecks((current) => current.map(
      (value, itemIndex) => itemIndex === index ? checked : value,
    ));
  };
  const toggleMaterializationRunAuthorizationCheck = (index: number, checked: boolean) => {
    setMaterializationRunAuthorizationChecks((current) => current.map((value, itemIndex) => itemIndex === index ? checked : value));
  };
  const toggleMaterializationFirstExecutionCheck = (index: number, checked: boolean) => {
    setMaterializationFirstExecutionChecks((current) => current.map((value, itemIndex) => itemIndex === index ? checked : value));
  };

  const submit = async () => {
    const current = registry();
    if (!current || submitDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = checks();
      await reviewHistoricalOutcomeGovernance({
        expected_review_id: current.latest_review?.review_id,
        verdict: verdict(),
        rationale: rationale(),
        protocol_frozen_pre_outcome_confirmed: values[0],
        adjusted_close_source_confirmed: values[1],
        common_session_rule_confirmed: values[2],
        benchmark_rule_confirmed: values[3],
        future_isolation_confirmed: values[4],
        missing_data_fail_closed_confirmed: values[5],
      });
      setRationale("");
      setChecks(APPROVAL_CHECKS.map(() => false));
      await load();
      setNotice("不可覆盖的协议复核已写入。批准也只允许未来登记标签器实现供再次审查；尚未生成任何收益标签。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存历史结果协议复核失败");
    } finally {
      setBusy(false);
    }
  };

  const registerLabeler = async () => {
    const current = labelers();
    if (!current?.current_governance_review_id || registerDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await registerHistoricalOutcomeLabeler({
        expected_governance_review_id: current.current_governance_review_id,
        protocol_version: current.protocol_version,
        protocol_sha256: current.protocol_sha256,
        implementation_name: implementationName(),
        implementation_kind: "deterministic_common_session_adjusted_close",
        code_revision: codeRevision(),
      });
      setCodeRevision("");
      await load();
      setNotice("不可覆盖的标签器实现规范已登记，但实现未运行、没有生成任何收益标签。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "登记历史结果标签器失败");
    } finally {
      setBusy(false);
    }
  };

  const reviewLabeler = async () => {
    const selected = selectedImplementation();
    if (!selected || labelerReviewDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = labelerChecks();
      await reviewHistoricalOutcomeLabeler(selected.implementation.implementation_id, {
        expected_review_id: selected.latest_review?.review_id,
        verdict: labelerVerdict(),
        rationale: labelerRationale(),
        implementation_fingerprint_confirmed: values[0],
        protocol_binding_confirmed: values[1],
        adjusted_close_and_common_sessions_confirmed: values[2],
        deterministic_replay_confirmed: values[3],
        future_isolation_confirmed: values[4],
        missing_data_fail_closed_confirmed: values[5],
        no_network_or_production_writes_confirmed: values[6],
      });
      setLabelerRationale("");
      setLabelerChecks(LABELER_REVIEW_CHECKS.map(() => false));
      await load();
      setNotice("不可覆盖的实现复核已写入。批准只允许进入离线试运行授权复核；试运行与标签生成仍关闭。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "复核历史结果标签器失败");
    } finally {
      setBusy(false);
    }
  };

  const ingestPriceSnapshot = async () => {
    const benchmarkState = selectedBenchmarkState();
    const labeler = selectedSnapshotLabeler();
    if (!benchmarkState || !labeler || ingestDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await ingestHistoricalOutcomePriceSnapshot({
        reconstruction_id: benchmarkState.reconstruction_id,
        expected_reconstruction_sha256: benchmarkState.reconstruction_sha256,
        expected_reconstruction_review_id: benchmarkState.reconstruction_review_id,
        implementation_id: labeler.implementation_id,
        expected_implementation_spec_sha256: labeler.implementation_spec_sha256,
        expected_implementation_review_id: labeler.implementation_review_id,
        expected_protocol_sha256: labeler.protocol_sha256,
      });
      await load();
      setNotice("FMP 复权行情已封存为不可覆盖快照；没有计算收益、写标签或运行训练。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "封存历史行情失败");
    } finally {
      setBusy(false);
    }
  };

  const reviewDryRunAuthorization = async () => {
    const selected = selectedAuthorization();
    if (!selected || authorizationDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = authorizationChecks();
      await reviewHistoricalOutcomeDryRunAuthorization(selected.snapshot_id, {
        expected_review_id: selected.latest_review?.review_id,
        expected_snapshot_sha256: selected.snapshot_sha256,
        expected_implementation_spec_sha256: priceSnapshots()?.snapshots.find(
          (item) => item.snapshot.snapshot_id === selected.snapshot_id,
        )?.snapshot.implementation_spec_sha256 ?? "",
        verdict: authorizationVerdict(),
        rationale: authorizationRationale(),
        current_bindings_confirmed: values[0],
        sealed_snapshot_integrity_confirmed: values[1],
        provider_provenance_confirmed: values[2],
        complete_common_session_coverage_confirmed: values[3],
        deterministic_fixture_confirmed: values[4],
        isolated_output_confirmed: values[5],
        no_label_or_production_writes_confirmed: values[6],
      });
      setAuthorizationRationale("");
      setAuthorizationChecks(DRY_RUN_AUTHORIZATION_CHECKS.map(() => false));
      await load();
      setNotice("授权复核已写入。批准只允许下一步登记离线试运行实现；试运行和收益标签仍关闭。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存离线试运行授权复核失败");
    } finally {
      setBusy(false);
    }
  };

  const registerDryRunImplementation = async () => {
    const selected = selectedDryRunAuthorization();
    if (!selected || dryRunImplementationRegisterDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await registerHistoricalOutcomeDryRunImplementation({
        snapshot_id: selected.snapshot_id,
        expected_authorization_review_id: selected.authorization_review_id,
        expected_snapshot_sha256: selected.snapshot_sha256,
        expected_implementation_spec_sha256: selected.implementation_spec_sha256,
        expected_protocol_sha256: selected.protocol_sha256,
        implementation_name: dryRunImplementationName(),
        implementation_kind: "deterministic_isolated_common_session_replay",
        code_revision: dryRunCodeRevision(),
      });
      setSelectedDryRunAuthorizationReviewId("");
      setDryRunCodeRevision("");
      await load();
      setNotice("不可覆盖的离线试运行实现已经登记为“已登记未运行”；没有计算收益、生成标签、训练、写影子组合或交易。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "登记离线试运行实现失败");
    } finally {
      setBusy(false);
    }
  };

  const reviewDryRunRunAuthorization = async () => {
    const selected = selectedRunAuthorization();
    if (!selected || runAuthorizationDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = runAuthorizationChecks();
      await reviewHistoricalOutcomeDryRunRunAuthorization(
        selected.implementation.dry_run_implementation_id,
        {
          expected_review_id: selected.latest_review?.review_id,
          expected_review_sha256: selected.latest_review?.review_sha256,
          expected_implementation_spec_sha256: selected.implementation.dry_run_implementation_spec_sha256,
          expected_authorization_review_id: selected.implementation.authorization_review_id,
          expected_snapshot_sha256: selected.implementation.snapshot_sha256,
          expected_protocol_sha256: selected.implementation.protocol_sha256,
          verdict: runAuthorizationVerdict(),
          rationale: runAuthorizationRationale(),
          implementation_fingerprint_confirmed: values[0],
          current_upstream_bindings_confirmed: values[1],
          code_revision_reproducible_confirmed: values[2],
          sealed_input_read_only_confirmed: values[3],
          deterministic_common_session_replay_confirmed: values[4],
          isolated_ephemeral_output_confirmed: values[5],
          resource_bounds_confirmed: values[6],
          no_network_or_external_tools_confirmed: values[7],
          no_production_label_training_reward_shadow_writes_confirmed: values[8],
          no_order_broker_or_trading_confirmed: values[9],
        },
      );
      setRunAuthorizationRationale("");
      setRunAuthorizationChecks(DRY_RUN_RUN_AUTHORIZATION_CHECKS.map(() => false));
      await load();
      setNotice("不可覆盖的运行授权复核已写入。批准只允许未来登记隔离执行器供再次审查；当前没有运行代码、计算收益或生成标签。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存离线试运行运行授权复核失败");
    } finally {
      setBusy(false);
    }
  };

  const registerIsolatedRunner = async () => {
    const selected = selectedRunnerAuthorization();
    if (!selected || runnerRegisterDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await registerHistoricalOutcomeDryRunIsolatedRunner({
        dry_run_implementation_id: selected.implementation.dry_run_implementation_id,
        expected_run_authorization_review_id: selected.review.review_id,
        expected_run_authorization_review_sha256: selected.review.review_sha256,
        expected_implementation_spec_sha256: selected.implementation.dry_run_implementation_spec_sha256,
        expected_snapshot_sha256: selected.implementation.snapshot_sha256,
        expected_protocol_sha256: selected.implementation.protocol_sha256,
        runner_name: runnerName(),
        runner_kind: "ephemeral_deterministic_process",
        runner_code_revision: runnerCodeRevision(),
        runner_artifact_sha256: runnerArtifactSha256().trim().toLowerCase(),
      });
      setSelectedRunnerAuthorizationReviewId("");
      setRunnerCodeRevision("");
      setRunnerArtifactSha256("");
      await load();
      setNotice("不可覆盖的隔离执行器规范已登记为“已登记未运行”。它没有可调用入口，也没有创建输出、标签、训练、影子持仓或交易权限。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "登记离线试运行隔离执行器失败");
    } finally {
      setBusy(false);
    }
  };

  const reviewFirstExecutionAuthorization = async () => {
    const selected = selectedFirstExecutionAuthorization();
    if (!selected || firstExecutionReviewDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = firstExecutionChecks();
      await reviewHistoricalOutcomeDryRunFirstExecutionAuthorization(
        selected.runner.isolated_runner_id,
        {
          expected_review_id: selected.latest_review?.review_id,
          expected_review_sha256: selected.latest_review?.review_sha256,
          expected_isolated_runner_spec_sha256: selected.runner.isolated_runner_spec_sha256,
          expected_runner_artifact_sha256: selected.runner.runner_artifact_sha256,
          expected_run_authorization_review_sha256: selected.runner.run_authorization_review_sha256,
          expected_implementation_spec_sha256: selected.runner.dry_run_implementation_spec_sha256,
          expected_snapshot_sha256: selected.runner.snapshot_sha256,
          expected_protocol_sha256: selected.runner.protocol_sha256,
          verdict: firstExecutionVerdict(),
          rationale: firstExecutionRationale(),
          runner_spec_fingerprint_confirmed: values[0],
          current_upstream_bindings_confirmed: values[1],
          artifact_digest_independently_verified: values[2],
          artifact_reproducible_and_available_confirmed: values[3],
          sealed_inputs_and_root_read_only_confirmed: values[4],
          unprivileged_no_new_privileges_confirmed: values[5],
          ephemeral_output_and_validation_confirmed: values[6],
          resource_limits_confirmed: values[7],
          no_host_environment_or_secrets_confirmed: values[8],
          no_network_or_external_tools_confirmed: values[9],
          no_production_history_label_training_reward_shadow_writes_confirmed: values[10],
          no_order_broker_or_trading_confirmed: values[11],
          single_use_and_expiry_confirmed: values[12],
        },
      );
      setFirstExecutionRationale("");
      setFirstExecutionChecks(FIRST_EXECUTION_AUTHORIZATION_CHECKS.map(() => false));
      await load();
      setNotice("不可覆盖的首次执行授权复核已写入。批准只在 24 小时内授予一次未来调用资格；授权复核本身没有启动执行，也没有生成输出或标签。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存首次执行授权复核失败");
    } finally {
      setBusy(false);
    }
  };

  const invokeOnce = async (isolatedRunnerId: string) => {
    const item = firstExecutionAuthorizations()?.items.find(
      (candidate) => candidate.runner.isolated_runner_id === isolatedRunnerId,
    );
    const review = item?.latest_review;
    const alreadyClaimed = executionAttempts()?.attempts.some(
      (attempt) => attempt.claim.isolated_runner_id === isolatedRunnerId,
    );
    if (!item?.current_binding || !item.authorization_unexpired || !review || alreadyClaimed || busy()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const registry = await invokeHistoricalOutcomeDryRunOnce(isolatedRunnerId, {
        expected_first_execution_authorization_review_id: review.review_id,
        expected_first_execution_authorization_review_sha256: review.review_sha256,
        expected_isolated_runner_spec_sha256: item.runner.isolated_runner_spec_sha256,
        expected_runner_artifact_sha256: item.runner.runner_artifact_sha256,
        expected_snapshot_sha256: item.runner.snapshot_sha256,
        expected_protocol_sha256: item.runner.protocol_sha256,
      });
      setExecutionAttempts(registry);
      await load();
      setNotice("一次性授权已经消费。输出仅作为带哈希的未验证工件保存；没有写入结果标签、训练、奖励、影子组合、订单或交易。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "能力隔离试运行失败");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const validateOutput = async (attemptId: string) => {
    const item = outputValidations()?.items.find(
      (candidate) => candidate.attempt.claim.attempt_id === attemptId,
    );
    const result = item?.attempt.result;
    if (!item?.validation_eligible || !result?.output_sha256 || busy()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const registry = await validateHistoricalOutcomeDryRunOutput(attemptId, {
        expected_claim_sha256: item.attempt.claim.claim_sha256,
        expected_result_sha256: result.result_sha256,
        expected_output_sha256: result.output_sha256,
        expected_snapshot_sha256: item.attempt.claim.snapshot_sha256,
        expected_protocol_sha256: item.attempt.claim.protocol_sha256,
      });
      setOutputValidations(registry);
      await load();
      setNotice("独立结构校验与第二套确定性重算已写入不可变记录。即使一致，结果标签准入、训练、奖励、影子与交易仍保持关闭。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "独立输出校验失败");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const reviewLabelAdmission = async () => {
    const item = selectedLabelAdmission();
    if (!item || labelAdmissionReviewDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = labelAdmissionChecks();
      const registry = await reviewHistoricalOutcomeLabelAdmission(
        item.validation.attempt_id,
        {
          expected_review_id: item.latest_review?.review_id,
          expected_review_sha256: item.latest_review?.review_sha256,
          expected_validation_id: item.validation.validation_id,
          expected_validation_sha256: item.validation.validation_sha256,
          expected_output_sha256: item.validation.output_sha256,
          expected_snapshot_sha256: item.validation.snapshot_sha256,
          expected_protocol_sha256: item.validation.protocol_sha256,
          verdict: labelAdmissionVerdict(),
          rationale: labelAdmissionRationale(),
          known_limitations: labelAdmissionLimitations(),
          exact_validation_current_binding_confirmed: values[0],
          frozen_protocol_applicability_confirmed: values[1],
          complete_horizons_and_common_session_endpoints_confirmed: values[2],
          adjusted_close_and_corporate_action_basis_confirmed: values[3],
          benchmark_comparability_confirmed: values[4],
          event_time_and_future_isolation_confirmed: values[5],
          missingness_and_survivorship_bias_reviewed: values[6],
          no_manual_metric_override_confirmed: values[7],
          label_semantics_and_direction_not_inferred_confirmed: values[8],
          downstream_authority_remains_closed_confirmed: values[9],
        },
      );
      setLabelAdmissions(registry);
      setLabelAdmissionRationale("");
      setLabelAdmissionLimitations("");
      setLabelAdmissionChecks(LABEL_ADMISSION_CHECKS.map(() => false));
      await load();
      setNotice("不可覆盖的结果标签准入复核已写入。批准只接纳精确输出作为未来标签物化输入；当前没有写标签、训练、奖励、影子、订单或交易。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存结果标签准入复核失败");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const registerLabelMaterializationImplementation = async () => {
    const admission = selectedMaterializationAdmission();
    if (!admission || materializationImplementationRegisterDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const registry = await registerHistoricalOutcomeLabelMaterializationImplementation({
        attempt_id: admission.attempt_id,
        expected_admission_review_id: admission.admission_review_id,
        expected_admission_review_sha256: admission.admission_review_sha256,
        expected_validation_sha256: admission.validation_sha256,
        expected_output_sha256: admission.output_sha256,
        expected_snapshot_sha256: admission.snapshot_sha256,
        expected_protocol_sha256: admission.protocol_sha256,
        implementation_name: materializationImplementationName(),
        implementation_kind: "deterministic_raw_validated_outcome_envelope",
        code_revision: materializationCodeRevision(),
      });
      setLabelMaterializationImplementations(registry);
      setMaterializationCodeRevision("");
      await load();
      setNotice("不可覆盖的原始结果信封物化规范已登记。登记不是运行：没有写入标签，也没有推断方向、评级、动作、仓位或奖励。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "登记结果标签物化实现失败");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const reviewLabelMaterializationRunAuthorization = async () => {
    const item = selectedMaterializationRunAuthorization();
    if (!item || materializationRunAuthorizationReviewDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = materializationRunAuthorizationChecks();
      const registry = await reviewHistoricalOutcomeLabelMaterializationRunAuthorization(
        item.implementation.materialization_implementation_id,
        {
          expected_review_id: item.latest_review?.review_id,
          expected_review_sha256: item.latest_review?.review_sha256,
          expected_implementation_spec_sha256: item.implementation.materialization_implementation_spec_sha256,
          expected_admission_review_sha256: item.implementation.admission_review_sha256,
          expected_validation_sha256: item.implementation.validation_sha256,
          expected_output_sha256: item.implementation.output_sha256,
          expected_snapshot_sha256: item.implementation.snapshot_sha256,
          expected_protocol_sha256: item.implementation.protocol_sha256,
          verdict: materializationRunAuthorizationVerdict(),
          rationale: materializationRunAuthorizationRationale(),
          implementation_fingerprint_confirmed: values[0],
          current_upstream_bindings_confirmed: values[1],
          code_revision_reproducible_confirmed: values[2],
          deterministic_raw_envelope_only_confirmed: values[3],
          exact_metric_bit_preservation_confirmed: values[4],
          provenance_and_limitations_preserved_confirmed: values[5],
          create_once_isolated_output_confirmed: values[6],
          missing_data_fail_closed_confirmed: values[7],
          no_network_tools_or_production_access_confirmed: values[8],
          no_semantic_action_position_or_reward_inference_confirmed: values[9],
          no_label_training_reward_shadow_order_broker_or_trading_authority_confirmed: values[10],
        },
      );
      setMaterializationRunAuthorizations(registry);
      setMaterializationRunAuthorizationRationale("");
      setMaterializationRunAuthorizationChecks(MATERIALIZATION_RUN_AUTHORIZATION_CHECKS.map(() => false));
      await load();
      setNotice("不可覆盖的标签物化运行授权复核已写入。批准只开放未来隔离物化 runner 规范登记资格；当前没有运行或写入标签。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存标签物化运行授权复核失败");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const registerLabelMaterializationIsolatedRunner = async () => {
    const selected = selectedMaterializationRunnerAuthorization();
    if (!selected || materializationRunnerRegisterDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const registry = await registerHistoricalOutcomeLabelMaterializationIsolatedRunner({
        materialization_implementation_id: selected.implementation.materialization_implementation_id,
        expected_run_authorization_review_id: selected.review.review_id,
        expected_run_authorization_review_sha256: selected.review.review_sha256,
        expected_implementation_spec_sha256: selected.implementation.materialization_implementation_spec_sha256,
        expected_admission_review_sha256: selected.implementation.admission_review_sha256,
        expected_validation_sha256: selected.implementation.validation_sha256,
        expected_output_sha256: selected.implementation.output_sha256,
        expected_snapshot_sha256: selected.implementation.snapshot_sha256,
        expected_protocol_sha256: selected.implementation.protocol_sha256,
        runner_name: materializationRunnerName(),
        runner_kind: "ephemeral_deterministic_process",
        runner_code_revision: materializationRunnerCodeRevision(),
        runner_artifact_sha256: materializationRunnerArtifactSha256().trim().toLowerCase(),
      });
      setMaterializationIsolatedRunners(registry);
      setSelectedMaterializationRunnerAuthorizationReviewId("");
      setMaterializationRunnerCodeRevision("");
      setMaterializationRunnerArtifactSha256("");
      await load();
      setNotice("不可覆盖的标签物化隔离 runner 规范已登记为“已登记未运行”。它没有调用入口，也没有标签、训练、奖励、影子或交易权限。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "登记标签物化隔离 runner 失败");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const reviewLabelMaterializationFirstExecutionAuthorization = async () => {
    const selected = selectedMaterializationFirstExecutionAuthorization();
    if (!selected || materializationFirstExecutionReviewDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = materializationFirstExecutionChecks();
      const registry = await reviewHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization(
        selected.runner.isolated_runner_id,
        {
          expected_review_id: selected.latest_review?.review_id,
          expected_review_sha256: selected.latest_review?.review_sha256,
          expected_isolated_runner_spec_sha256: selected.runner.isolated_runner_spec_sha256,
          expected_runner_artifact_sha256: selected.runner.runner_artifact_sha256,
          expected_materialization_run_authorization_review_sha256: selected.runner.materialization_run_authorization_review_sha256,
          expected_implementation_spec_sha256: selected.runner.materialization_implementation_spec_sha256,
          expected_admission_review_sha256: selected.runner.admission_review_sha256,
          expected_validation_sha256: selected.runner.validation_sha256,
          expected_output_sha256: selected.runner.output_sha256,
          expected_snapshot_sha256: selected.runner.snapshot_sha256,
          expected_protocol_sha256: selected.runner.protocol_sha256,
          expected_recomputed_metrics_sha256: selected.runner.recomputed_metrics_sha256,
          verdict: materializationFirstExecutionVerdict(),
          rationale: materializationFirstExecutionRationale(),
          runner_spec_fingerprint_confirmed: values[0],
          current_upstream_bindings_confirmed: values[1],
          artifact_digest_independently_verified: values[2],
          artifact_reproducible_and_available_confirmed: values[3],
          sealed_inputs_and_root_read_only_confirmed: values[4],
          unprivileged_no_new_privileges_confirmed: values[5],
          ephemeral_output_and_validation_confirmed: values[6],
          resource_limits_confirmed: values[7],
          no_host_environment_or_secrets_confirmed: values[8],
          no_network_external_tools_or_child_processes_confirmed: values[9],
          raw_envelope_only_no_semantic_inference_confirmed: values[10],
          no_production_history_label_training_reward_shadow_writes_confirmed: values[11],
          no_order_broker_or_trading_confirmed: values[12],
          single_use_and_expiry_confirmed: values[13],
        },
      );
      setMaterializationFirstExecutionAuthorizations(registry);
      setMaterializationFirstExecutionRationale("");
      setMaterializationFirstExecutionChecks(MATERIALIZATION_FIRST_EXECUTION_AUTHORIZATION_CHECKS.map(() => false));
      await load();
      setNotice("不可覆盖的标签物化首次执行授权复核已写入。批准只提供 24 小时内一次调用额度；必须在实际执行前写 create-once claim 并重验制品与全部上游，且不会直接写入标签。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存标签物化首次执行授权复核失败");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const invokeLabelMaterializationOnce = async () => {
    const selected = invokableMaterializationAuthorization();
    const review = selected?.latest_review;
    if (!selected || !review || materializationInvocationDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const registry = await invokeHistoricalOutcomeLabelMaterializationOnce(
        selected.runner.isolated_runner_id,
        {
          expected_first_execution_authorization_review_id: review.review_id,
          expected_first_execution_authorization_review_sha256: review.review_sha256,
          expected_isolated_runner_spec_sha256: selected.runner.isolated_runner_spec_sha256,
          expected_runner_artifact_sha256: selected.runner.runner_artifact_sha256,
          expected_implementation_spec_sha256: selected.runner.materialization_implementation_spec_sha256,
          expected_admission_review_sha256: selected.runner.admission_review_sha256,
          expected_validation_sha256: selected.runner.validation_sha256,
          expected_source_output_sha256: selected.runner.output_sha256,
          expected_snapshot_sha256: selected.runner.snapshot_sha256,
          expected_protocol_sha256: selected.runner.protocol_sha256,
          expected_recomputed_metrics_sha256: selected.runner.recomputed_metrics_sha256,
        },
      );
      setMaterializationExecutionAttempts(registry);
      await load();
      setNotice("一次性授权已消费，固定纯函数已生成不可覆盖的未信任原始结果包。它不是结果标签；必须先通过下一阶段的独立结构、来源与逐位一致性校验。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "执行一次性标签物化失败；授权可能已经消费，请核对不可覆盖的 claim 与结果记录");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const validateLabelMaterializationOutput = async () => {
    const selected = eligibleMaterializationOutputValidation();
    const result = selected?.attempt.result;
    if (!selected || !result?.output_sha256 || materializationOutputValidationDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const validationRegistry = await validateHistoricalOutcomeLabelMaterializationOutput(
        selected.attempt.claim.attempt_id,
        {
          expected_claim_sha256: selected.attempt.claim.claim_sha256,
          expected_result_sha256: result.result_sha256,
          expected_output_sha256: result.output_sha256,
          expected_admission_review_sha256: selected.attempt.claim.admission_review_sha256,
          expected_validation_sha256: selected.attempt.claim.validation_sha256,
          expected_source_output_sha256: selected.attempt.claim.source_output_sha256,
          expected_snapshot_sha256: selected.attempt.claim.snapshot_sha256,
          expected_protocol_sha256: selected.attempt.claim.protocol_sha256,
          expected_recomputed_metrics_sha256: selected.attempt.claim.recomputed_metrics_sha256,
        },
      );
      setMaterializationOutputValidations(validationRegistry);
      await load();
      setNotice("独立校验记录已不可覆盖写入：结构、来源绑定和 20 / 60 / 250 日指标按位核对。即使全部一致，这个结果包仍不是正式标签，也不会开放训练、奖励、影子持仓或交易。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "独立校验标签物化结果失败；请核对不可变结果记录与完整上游绑定");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const reviewFormalLabelWriteAuthorization = async () => {
    const selected = selectedLabelWriteAuthorization();
    const registry = labelWriteAuthorizations();
    if (!selected || !registry || labelWriteAuthorizationDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = labelWriteAuthorizationChecks();
      const updated = await reviewHistoricalOutcomeLabelWriteAuthorization(
        selected.materialization_validation_id,
        {
          expected_review_id: selected.latest_review?.review_id,
          expected_review_sha256: selected.latest_review?.review_sha256,
          expected_materialization_validation_sha256: selected.materialization_validation_sha256,
          expected_claim_sha256: selected.claim_sha256,
          expected_result_sha256: selected.result_sha256,
          expected_output_sha256: selected.output_sha256,
          expected_admission_review_sha256: selected.admission_review_sha256,
          expected_source_validation_sha256: selected.source_validation_sha256,
          expected_source_output_sha256: selected.source_output_sha256,
          expected_snapshot_sha256: selected.snapshot_sha256,
          expected_protocol_sha256: selected.protocol_sha256,
          expected_recomputed_metrics_sha256: selected.recomputed_metrics_sha256,
          expected_label_contract_sha256: registry.label_contract_sha256,
          verdict: labelWriteAuthorizationVerdict(),
          rationale: labelWriteAuthorizationRationale().trim(),
          exact_validated_envelope_binding_confirmed: values[0],
          reviewer_independence_confirmed: values[1],
          formal_label_schema_confirmed: values[2],
          raw_outcome_semantics_only_confirmed: values[3],
          exact_metric_bits_and_provenance_confirmed: values[4],
          known_limitations_preserved_confirmed: values[5],
          create_once_no_overwrite_writer_confirmed: values[6],
          single_use_and_expiry_confirmed: values[7],
          label_store_isolated_from_training_confirmed: values[8],
          no_semantic_inference_or_reward_confirmed: values[9],
          no_network_tools_or_unrelated_production_access_confirmed: values[10],
          no_training_shadow_order_broker_or_trading_confirmed: values[11],
        },
      );
      setLabelWriteAuthorizations(updated);
      setLabelWriteAuthorizationRationale("");
      setLabelWriteAuthorizationChecks(FORMAL_LABEL_WRITE_AUTHORIZATION_CHECKS.map(() => false));
      await load();
      setNotice("独立授权复核已不可覆盖写入。批准仅授予 24 小时内一次未来 create-once 写入资格；本阶段没有 writer，也没有写入正式标签。训练、奖励、影子、订单、券商和交易仍全部关闭。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "正式标签未来一次写入授权复核失败");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const writeFormalRawOutcomeLabelOnce = async () => {
    const selected = selectedFormalLabelWriteAuthorization();
    if (!selected || formalLabelWriteDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const updated = await writeHistoricalOutcomeFormalLabelOnce(
        selected.authorization_review_id,
        {
          expected_authorization_review_sha256: selected.authorization_review_sha256,
          expected_materialization_validation_sha256: selected.materialization_validation_sha256,
          expected_claim_sha256: selected.materialization_claim_sha256,
          expected_result_sha256: selected.materialization_result_sha256,
          expected_output_sha256: selected.materialization_output_sha256,
          expected_admission_review_sha256: selected.admission_review_sha256,
          expected_source_validation_sha256: selected.source_validation_sha256,
          expected_source_output_sha256: selected.source_output_sha256,
          expected_snapshot_sha256: selected.snapshot_sha256,
          expected_protocol_sha256: selected.protocol_sha256,
          expected_recomputed_metrics_sha256: selected.recomputed_metrics_sha256,
          expected_label_contract_sha256: selected.label_contract_sha256,
        },
      );
      setFormalLabelWrites(updated);
      setSelectedFormalLabelAuthorizationReviewId("");
      await load();
      setNotice("一次性授权已不可逆消费。若写入成功，正式标签只包含原始绝对/相对市场结果、来源、局限和完整链绑定；它尚未通过训练准入校验，不会进入训练、奖励、影子或交易。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "正式原始结果标签一次性写入失败");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const validateFormalRawOutcomeLabel = async () => {
    const selected = selectedFormalLabelForValidation();
    if (!selected || formalLabelValidationDisabled()) return;
    const { claim, label } = selected.formal_label;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const updated = await validateHistoricalOutcomeFormalLabel(label.label_id, {
        expected_label_sha256: label.label_sha256,
        expected_claim_sha256: claim.claim_sha256,
        expected_authorization_review_sha256: claim.authorization_review_sha256,
        expected_materialization_validation_sha256: claim.materialization_validation_sha256,
        expected_materialization_output_sha256: claim.materialization_output_sha256,
        expected_source_validation_sha256: claim.source_validation_sha256,
        expected_source_output_sha256: claim.source_output_sha256,
        expected_snapshot_sha256: claim.snapshot_sha256,
        expected_protocol_sha256: claim.protocol_sha256,
        expected_recomputed_metrics_sha256: claim.recomputed_metrics_sha256,
        expected_label_contract_sha256: claim.label_contract_sha256,
      });
      setFormalLabelValidations(updated);
      setSelectedFormalLabelId("");
      await load();
      setNotice("独立校验记录已不可覆盖写入。通过只代表该正式原始结果标签进入隔离的离线训练数据集候选池；尚未复制进训练存储，也未授权训练、奖励、影子、订单、券商或交易。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "正式原始结果标签独立校验失败");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const assembleOfflineHistoricalOutcomeDataset = async () => {
    const current = offlineDatasets();
    if (!current || offlineDatasetAssemblyDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const updated = await assembleHistoricalOutcomeOfflineDataset({
        expected_candidate_set_sha256: current.current_candidate_set_sha256,
        expected_candidates: current.current_candidates,
        purpose: "historical_raw_outcome_research_only",
        complete_current_candidate_set_confirmed: true,
        monotonic_version_lineage_confirmed: true,
        point_in_time_lineage_preserved_confirmed: true,
        no_semantic_target_or_split_inference_confirmed: true,
        no_training_reward_shadow_order_broker_or_trading_confirmed: true,
      });
      setOfflineDatasets(updated);
      setOfflineDatasetAssemblyChecks(OFFLINE_DATASET_ASSEMBLY_CHECKS.map(() => false));
      await load();
      setNotice("当前完整候选集已装配为不可变、内容寻址的离线原始结果数据集。数据集仍未分割、未拼接特征、未生成语义目标，也未授权训练、奖励、影子或交易。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "离线历史结果数据集装配失败");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  const reviewOfflineHistoricalOutcomeDatasetGovernance = async () => {
    const registry = offlineDatasetGovernance();
    const selected = selectedOfflineDatasetGovernance();
    if (!registry || !selected || offlineDatasetGovernanceDisabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const checks = offlineDatasetGovernanceChecks();
      const updated = await reviewHistoricalOutcomeOfflineDatasetGovernance(
        selected.subject.dataset_id,
        {
          expected_review_id: selected.latest_review?.review_id,
          expected_review_sha256: selected.latest_review?.review_sha256,
          expected_dataset_content_sha256: selected.subject.dataset_content_sha256,
          expected_manifest_sha256: selected.subject.manifest_sha256,
          expected_candidate_set_sha256: selected.subject.candidate_set_sha256,
          expected_split_policy_sha256: registry.split_policy.policy_sha256,
          expected_feature_join_policy_sha256: registry.feature_join_policy.policy_sha256,
          verdict: offlineDatasetGovernanceVerdict(),
          rationale: offlineDatasetGovernanceRationale().trim(),
          known_limitations: offlineDatasetGovernanceLimitations().trim(),
          exact_current_dataset_binding_confirmed: checks[0] ?? false,
          reviewer_independence_confirmed: checks[1] ?? false,
          complete_candidate_and_lineage_confirmed: checks[2] ?? false,
          company_event_source_component_isolation_confirmed: checks[3] ?? false,
          deterministic_split_and_sealed_holdout_confirmed: checks[4] ?? false,
          temporal_order_and_max_horizon_embargo_confirmed: checks[5] ?? false,
          point_in_time_feature_availability_confirmed: checks[6] ?? false,
          immutable_feature_provenance_confirmed: checks[7] ?? false,
          outcome_and_label_feature_exclusion_confirmed: checks[8] ?? false,
          missing_or_ambiguous_availability_fail_closed_confirmed: checks[9] ?? false,
          no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: checks[10] ?? false,
        },
      );
      setOfflineDatasetGovernance(updated);
      setSelectedOfflineDatasetGovernanceId("");
      setOfflineDatasetGovernanceRationale("");
      setOfflineDatasetGovernanceLimitations("");
      setOfflineDatasetGovernanceChecks(OFFLINE_DATASET_GOVERNANCE_CHECKS.map(() => false));
      await load();
      setNotice("数据集治理复核已写入不可变追加链。批准只允许下一阶段登记切分与点时特征转换规范；本阶段没有切分、拼接、训练、奖励、影子或交易权限。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "离线历史结果数据集治理复核失败");
      await load().catch(() => undefined);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>{(value) => (
      <section class="public-admin-historical-anchors" aria-label="历史结果协议治理">
        <header>
          <div>
            <h3>历史结果协议冻结与审批</h3>
            <p>{value().scope}</p>
          </div>
          <span>标签仍关闭</span>
        </header>
        <div class="public-admin-decision-metrics">
          <div><span>人工基准状态</span><strong>{value().benchmark_ready_count}</strong></div>
          <div><span>观察窗口</span><strong>{value().protocol.horizons_market_sessions.join(" / ")}</strong></div>
          <div><span>市场基准</span><strong>{value().protocol.benchmark_symbol}</strong></div>
          <div><span>协议状态</span><strong>{value().labeler_implementation_registration_eligible ? "可审实现" : "未批准"}</strong></div>
        </div>
        <p class="public-admin-anchor-boundary">
          协议指纹 {value().protocol_sha256}。个股与 {value().protocol.benchmark_symbol} 使用复权收盘价和共同交易日；结果标签关闭、训练关闭、奖励关闭、影子关闭、交易关闭。
        </p>
        <Show when={approvalBlocked()}>
          <p class="public-admin-decision-notice">当前没有人工批准的历史基准状态，因此不能批准任何标签器实现评审。请先完成“锚点 → 七层点时状态 → 人工批准”。</p>
        </Show>
        <Show when={value().latest_review}>
          {(review) => <p class="public-admin-anchor-boundary">最近复核：{review().verdict} · {new Date(review().submitted_at).toLocaleString("zh-CN")} · 基准状态 {review().benchmark_state_count_at_review} 条</p>}
        </Show>

        <details class="public-admin-reward-governance">
          <summary>复核结果计算协议（不生成标签）</summary>
          <label>
            <span>复核结论</span>
            <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeGovernanceVerdict)}>
              <option value="approved_for_implementation_review">批准未来标签器实现评审</option>
              <option value="changes_requested">要求修订协议</option>
              <option value="rejected">拒绝协议</option>
            </select>
          </label>
          <label><span>复核依据</span><textarea maxlength={1600} value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
          <Show when={approvalSelected()}>
            <For each={APPROVAL_CHECKS}>{(label, index) => (
              <label class="public-admin-reward-confirm"><input type="checkbox" checked={checks()[index()]} onChange={(event) => toggleCheck(index(), event.currentTarget.checked)} /> {label}</label>
            )}</For>
          </Show>
          <button type="button" class="public-admin-decision-submit" disabled={submitDisabled()} onClick={() => void submit()}>
            写入不可覆盖的协议复核
          </button>
        </details>
        <Show when={labelers()}>{(labelerRegistry) => (
          <section class="public-admin-reward-governance" aria-label="历史结果标签器实现注册表">
            <header>
              <strong>历史结果标签器实现登记与审查</strong>
              <span>{labelerRegistry().labeler_review_status}</span>
            </header>
            <p>{labelerRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>登记实现</span><strong>{labelerRegistry().implementations.length}</strong></div>
              <div><span>当前绑定</span><strong>{labelerRegistry().current_binding_implementation_count}</strong></div>
              <div><span>人工通过</span><strong>{labelerRegistry().reviewed_implementation_count}</strong></div>
              <div><span>离线试运行</span><strong>{labelerRegistry().offline_dry_run_enabled ? "开启" : "关闭"}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              标签器必须消费封存行情快照且自身不联网；实现登记、人工复核、离线试运行授权和结果标签生成是四道独立门禁。
            </p>

            <details class="public-admin-reward-governance">
              <summary>登记冻结实现规范（不运行）</summary>
              <Show when={!labelerRegistry().registration_allowed}>
                <p class="public-admin-decision-notice">当前结果协议尚未获准登记实现。请先完成历史基准状态与上方协议复核。</p>
              </Show>
              <div class="public-admin-anchor-form">
                <label><span>实现名称</span><input maxlength={120} value={implementationName()} onInput={(event) => setImplementationName(event.currentTarget.value)} /></label>
                <label><span>不可变代码版本</span><input maxlength={200} placeholder="例如 oldwang@abc123" value={codeRevision()} onInput={(event) => setCodeRevision(event.currentTarget.value)} /></label>
                <p class="public-admin-anchor-boundary">固定种类：共同交易日 + FMP 复权收盘价 + SPY 基准 + 确定性重放；联网、外部工具、生产写入、标签写入全部关闭。</p>
                <button type="button" class="public-admin-decision-submit" disabled={registerDisabled()} onClick={() => void registerLabeler()}>
                  写入不可覆盖的实现登记
                </button>
              </div>
            </details>

            <For each={labelerRegistry().implementations}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.implementation.implementation_name}</strong>
                  <span>{item.governance_binding_current ? "绑定有效" : "绑定失效"}</span>
                </header>
                <p>{item.implementation.code_revision} · {item.implementation.status} · 指纹 {item.implementation.implementation_spec_sha256.slice(0, 12)}…</p>
                <p>复核：{item.latest_review?.verdict ?? "尚未复核"}；可进入离线试运行授权复核：{item.offline_dry_run_authorization_review_eligible ? "是" : "否"}</p>
                <button type="button" class="public-admin-decision-submit" onClick={() => setSelectedImplementationId(item.implementation.implementation_id)}>
                  选择并复核此实现
                </button>
              </article>
            )}</For>

            <Show when={selectedImplementation()}>{(selected) => (
              <details class="public-admin-reward-governance" open>
                <summary>人工复核：{selected().implementation.implementation_name}</summary>
                <label>
                  <span>复核结论</span>
                  <select value={labelerVerdict()} onChange={(event) => setLabelerVerdict(event.currentTarget.value as HistoricalOutcomeLabelerReviewVerdict)}>
                    <option value="approved_for_offline_dry_run_authorization_review">批准进入离线试运行授权复核</option>
                    <option value="changes_requested">要求修改实现</option>
                    <option value="rejected">拒绝实现</option>
                  </select>
                </label>
                <label><span>复核依据</span><textarea maxlength={1600} value={labelerRationale()} onInput={(event) => setLabelerRationale(event.currentTarget.value)} /></label>
                <Show when={labelerApprovalSelected()}>
                  <For each={LABELER_REVIEW_CHECKS}>{(label, index) => (
                    <label class="public-admin-reward-confirm"><input type="checkbox" checked={labelerChecks()[index()]} onChange={(event) => toggleLabelerCheck(index(), event.currentTarget.checked)} /> {label}</label>
                  )}</For>
                </Show>
                <button type="button" class="public-admin-decision-submit" disabled={labelerReviewDisabled()} onClick={() => void reviewLabeler()}>
                  写入不可覆盖的实现复核
                </button>
              </details>
            )}</Show>
          </section>
        )}</Show>
        <Show when={priceSnapshots()}>{(snapshotRegistry) => (
          <section class="public-admin-reward-governance" aria-label="历史结果封存行情">
            <header>
              <strong>封存行情快照</strong>
              <span>{snapshotRegistry().fully_covered_snapshot_count} 条完整覆盖</span>
            </header>
            <p>{snapshotRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>可用基准状态</span><strong>{snapshotRegistry().eligible_benchmark_states.length}</strong></div>
              <div><span>可用标签器</span><strong>{snapshotRegistry().eligible_labelers.length}</strong></div>
              <div><span>当前快照</span><strong>{snapshotRegistry().current_snapshot_count}</strong></div>
              <div><span>收益标签</span><strong>关闭</strong></div>
            </div>
            <details class="public-admin-reward-governance">
              <summary>从 FMP 封存复权行情（不计算收益）</summary>
              <Show when={snapshotRegistry().eligible_benchmark_states.length === 0 || snapshotRegistry().eligible_labelers.length === 0}>
                <p class="public-admin-decision-notice">请先完成人工历史基准状态、结果协议、标签器登记和实现复核；缺少任一绑定都不能摄取未来行情。</p>
              </Show>
              <label>
                <span>历史基准状态</span>
                <select value={selectedBenchmarkStateId()} onChange={(event) => setSelectedBenchmarkStateId(event.currentTarget.value)}>
                  <option value="">选择已批准基准</option>
                  <For each={snapshotRegistry().eligible_benchmark_states}>{(item) => (
                    <option value={item.reconstruction_id}>{item.symbol} · {new Date(item.decision_available_at).toLocaleDateString("zh-CN")}</option>
                  )}</For>
                </select>
              </label>
              <label>
                <span>已复核标签器</span>
                <select value={selectedSnapshotLabelerId()} onChange={(event) => setSelectedSnapshotLabelerId(event.currentTarget.value)}>
                  <option value="">选择当前实现</option>
                  <For each={snapshotRegistry().eligible_labelers}>{(item) => (
                    <option value={item.implementation_id}>{item.code_revision} · {item.implementation_spec_sha256.slice(0, 12)}…</option>
                  )}</For>
                </select>
              </label>
              <p class="public-admin-anchor-boundary">只封存请求截止日以前的 FMP adjClose 和 SPY 同口径序列。API Key 不进入快照，返回行情的规范化载荷与序列分别计算 SHA-256。</p>
              <button type="button" class="public-admin-decision-submit" disabled={ingestDisabled()} onClick={() => void ingestPriceSnapshot()}>
                摄取并封存行情快照
              </button>
            </details>
            <For each={snapshotRegistry().snapshots}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.snapshot.asset_symbol} / {item.snapshot.benchmark_symbol}</strong>
                  <span>{item.dry_run_authorization_review_eligible ? "可送授权复核" : "未完整或绑定失效"}</span>
                </header>
                <p>{item.snapshot.requested_from} → {item.snapshot.requested_to} · 共同交易日 {item.snapshot.common_session_count} · 覆盖 {item.snapshot.covered_horizons_market_sessions.join(" / ") || "无"}</p>
                <p>快照 {item.snapshot.snapshot_sha256.slice(0, 12)}… · 行情来源 {item.snapshot.provider} · 收益未计算、标签未写入</p>
              </article>
            )}</For>
          </section>
        )}</Show>
        <Show when={authorizations()}>{(authorizationRegistry) => (
          <section class="public-admin-reward-governance" aria-label="离线试运行授权治理">
            <header>
              <strong>离线试运行授权</strong>
              <span>{authorizationRegistry().authorization_status}</span>
            </header>
            <p>{authorizationRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>可复核快照</span><strong>{authorizationRegistry().items.filter((item) => item.current_binding).length}</strong></div>
              <div><span>已经复核</span><strong>{authorizationRegistry().reviewed_snapshot_count}</strong></div>
              <div><span>可登记试运行实现</span><strong>{authorizationRegistry().registration_eligible_snapshot_count}</strong></div>
              <div><span>试运行</span><strong>{authorizationRegistry().offline_dry_run_enabled ? "开启" : "关闭"}</strong></div>
            </div>
            <For each={authorizationRegistry().items}>{(item) => (
              <article class="public-admin-reward-governance">
                <header><strong>{item.asset_symbol} · {item.common_session_count} 个共同交易日</strong><span>{item.current_binding ? "绑定有效" : "绑定失效"}</span></header>
                <p>最近复核：{item.latest_review?.verdict ?? "尚未复核"}；可登记试运行实现：{item.dry_run_implementation_registration_eligible ? "是" : "否"}</p>
                <button type="button" class="public-admin-decision-submit" onClick={() => setSelectedSnapshotId(item.snapshot_id)}>选择并复核此快照</button>
              </article>
            )}</For>
            <Show when={selectedAuthorization()}>{(selected) => (
              <details class="public-admin-reward-governance" open>
                <summary>复核 {selected().asset_symbol} 封存快照的试运行边界</summary>
                <label>
                  <span>复核结论</span>
                  <select value={authorizationVerdict()} onChange={(event) => setAuthorizationVerdict(event.currentTarget.value as HistoricalOutcomeDryRunAuthorizationVerdict)}>
                    <option value="approved_for_dry_run_implementation_registration">批准下一步登记离线试运行实现</option>
                    <option value="changes_requested">要求补齐或修订</option>
                    <option value="rejected">拒绝授权</option>
                  </select>
                </label>
                <label><span>复核依据</span><textarea maxlength={1600} value={authorizationRationale()} onInput={(event) => setAuthorizationRationale(event.currentTarget.value)} /></label>
                <Show when={authorizationApprovalSelected()}>
                  <For each={DRY_RUN_AUTHORIZATION_CHECKS}>{(label, index) => (
                    <label class="public-admin-reward-confirm"><input type="checkbox" checked={authorizationChecks()[index()]} onChange={(event) => toggleAuthorizationCheck(index(), event.currentTarget.checked)} /> {label}</label>
                  )}</For>
                </Show>
                <button type="button" class="public-admin-decision-submit" disabled={authorizationDisabled()} onClick={() => void reviewDryRunAuthorization()}>
                  写入不可覆盖的授权复核
                </button>
                <p class="public-admin-anchor-boundary">批准后仍不运行标签器；只允许后续登记一个精确绑定此快照的隔离试运行实现。</p>
              </details>
            )}</Show>
          </section>
        )}</Show>
        <Show when={dryRunImplementations()}>{(implementationRegistry) => (
          <section class="public-admin-reward-governance" aria-label="离线试运行实现注册表">
            <header>
              <strong>离线试运行实现登记</strong>
              <span>{implementationRegistry().implementation_status}</span>
            </header>
            <p>{implementationRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>已登记实现</span><strong>{implementationRegistry().implementation_count}</strong></div>
              <div><span>当前绑定</span><strong>{implementationRegistry().current_binding_implementation_count}</strong></div>
              <div><span>可送运行复核</span><strong>{implementationRegistry().run_authorization_review_eligible_count}</strong></div>
              <div><span>实际试运行</span><strong>{implementationRegistry().offline_dry_run_enabled ? "开启" : "关闭"}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              本阶段只登记实现，固定状态为“已登记未运行”。联网、外部工具、生产写入、结果标签、训练、奖励、影子组合、订单和券商访问全部关闭。
            </p>
            <details class="public-admin-reward-governance">
              <summary>登记隔离试运行实现（只登记，不运行）</summary>
              <Show when={!implementationRegistry().registration_allowed}>
                <p class="public-admin-decision-notice">请先让封存行情快照通过上方独立试运行授权复核。</p>
              </Show>
              <label>
                <span>已批准授权</span>
                <select value={selectedDryRunAuthorizationReviewId()} onChange={(event) => setSelectedDryRunAuthorizationReviewId(event.currentTarget.value)}>
                  <option value="">选择与快照精确绑定的授权</option>
                  <For each={implementationRegistry().eligible_authorizations}>{(item) => (
                    <option value={item.authorization_review_id}>{item.asset_symbol} · {item.common_session_count} 日 · {item.snapshot_sha256.slice(0, 12)}…</option>
                  )}</For>
                </select>
              </label>
              <label><span>实现名称</span><input maxlength={120} value={dryRunImplementationName()} onInput={(event) => setDryRunImplementationName(event.currentTarget.value)} /></label>
              <label><span>不可变代码版本</span><input maxlength={160} placeholder="例如 oldwang@dryrun123" value={dryRunCodeRevision()} onInput={(event) => setDryRunCodeRevision(event.currentTarget.value)} /></label>
              <button type="button" class="public-admin-decision-submit" disabled={dryRunImplementationRegisterDisabled()} onClick={() => void registerDryRunImplementation()}>
                写入不可覆盖的试运行实现登记
              </button>
            </details>
            <For each={implementationRegistry().implementations}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.implementation.asset_symbol} · {item.implementation.implementation_name}</strong>
                  <span>{item.authorization_binding_current ? "绑定有效" : "绑定失效"}</span>
                </header>
                <p>{item.implementation.status} · {item.implementation.code_revision} · 指纹 {item.implementation.dry_run_implementation_spec_sha256.slice(0, 12)}…</p>
                <p>标签器 {item.implementation.labeler_code_revision} · 共同交易日 {item.implementation.common_session_count} · 运行授权：否 · 结果标签：否 · 交易：否</p>
              </article>
            )}</For>
          </section>
        )}</Show>
        <Show when={runAuthorizations()}>{(runAuthorizationRegistry) => (
          <section class="public-admin-reward-governance" aria-label="离线试运行运行授权复核">
            <header>
              <strong>离线试运行运行授权复核</strong>
              <span>{runAuthorizationRegistry().authorization_status}</span>
            </header>
            <p>{runAuthorizationRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>可复核实现</span><strong>{runAuthorizationRegistry().review_eligible_implementation_count}</strong></div>
              <div><span>已经复核</span><strong>{runAuthorizationRegistry().reviewed_implementation_count}</strong></div>
              <div><span>可登记隔离执行器</span><strong>{runAuthorizationRegistry().isolated_runner_registration_eligible_count}</strong></div>
              <div><span>实际运行</span><strong>{runAuthorizationRegistry().offline_dry_run_enabled ? "开启" : "关闭"}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              这是第八道非运行门禁，且实现登记者不能批准自己的实现。即使批准，运行授权仍为否，输出工件仍不存在；标签、训练、奖励、影子、订单、券商和交易全部关闭。
            </p>
            <For each={runAuthorizationRegistry().items}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.implementation.asset_symbol} · {item.implementation.implementation_name}</strong>
                  <span>{item.isolated_runner_registration_eligible ? "可登记未来执行器" : item.latest_review ? "已复核未批准" : "待独立复核"}</span>
                </header>
                <p>{item.implementation.status} · {item.implementation.code_revision} · 实现 {item.implementation.dry_run_implementation_spec_sha256.slice(0, 12)}…</p>
                <p>最近复核：{item.latest_review?.verdict ?? "尚未复核"}{item.latest_review ? ` · 审计 ${item.latest_review.review_sha256.slice(0, 12)}…` : ""}</p>
                <button type="button" class="public-admin-decision-submit" onClick={() => setSelectedRunImplementationId(item.implementation.dry_run_implementation_id)}>
                  选择并复核运行边界
                </button>
              </article>
            )}</For>
            <Show when={selectedRunAuthorization()}>{(selected) => (
              <details class="public-admin-reward-governance" open>
                <summary>复核 {selected().implementation.asset_symbol} 的未来隔离运行资格</summary>
                <label>
                  <span>复核结论</span>
                  <select value={runAuthorizationVerdict()} onChange={(event) => setRunAuthorizationVerdict(event.currentTarget.value as HistoricalOutcomeDryRunRunAuthorizationVerdict)}>
                    <option value="approved_for_isolated_runner_registration">批准下一步登记隔离执行器（仍不运行）</option>
                    <option value="changes_requested">要求补齐或修订</option>
                    <option value="rejected">拒绝授权</option>
                  </select>
                </label>
                <label><span>复核依据</span><textarea maxlength={2400} value={runAuthorizationRationale()} onInput={(event) => setRunAuthorizationRationale(event.currentTarget.value)} /></label>
                <Show when={runAuthorizationApprovalSelected()}>
                  <For each={DRY_RUN_RUN_AUTHORIZATION_CHECKS}>{(label, index) => (
                    <label class="public-admin-reward-confirm"><input type="checkbox" checked={runAuthorizationChecks()[index()]} onChange={(event) => toggleRunAuthorizationCheck(index(), event.currentTarget.checked)} /> {label}</label>
                  )}</For>
                </Show>
                <button type="button" class="public-admin-decision-submit" disabled={runAuthorizationDisabled()} onClick={() => void reviewDryRunRunAuthorization()}>
                  写入不可覆盖的运行授权复核
                </button>
                <p class="public-admin-anchor-boundary">批准不执行当前实现，只允许后续登记一个仍需独立审查的隔离执行器。运行、结果工件与标签准入继续分开。</p>
              </details>
            )}</Show>
          </section>
        )}</Show>
        <Show when={isolatedRunners()}>{(runnerRegistry) => (
          <section class="public-admin-reward-governance" aria-label="离线试运行隔离执行器登记">
            <header>
              <strong>隔离执行器规范登记</strong>
              <span>{runnerRegistry().runner_status}</span>
            </header>
            <p>{runnerRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>已登记执行器</span><strong>{runnerRegistry().runner_count}</strong></div>
              <div><span>当前绑定</span><strong>{runnerRegistry().current_binding_runner_count}</strong></div>
              <div><span>可送首次执行复核</span><strong>{runnerRegistry().execution_authorization_review_eligible_count}</strong></div>
              <div><span>实际调用</span><strong>{runnerRegistry().invocation_authorized ? "开启" : "关闭"}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              这是第九道登记门禁。规范固定 300 秒、512 MiB、1 核、单进程和 1 MiB 临时输出上限；登记记录自身无入口、无环境变量、无密钥、无网络、无生产写入。第十一阶段只会在一次性授权后调用内置的有界纯函数执行后端。
            </p>
            <details class="public-admin-reward-governance">
              <summary>登记隔离执行器规范（只登记，不调用）</summary>
              <Show when={!runnerRegistry().registration_allowed}>
                <p class="public-admin-decision-notice">请先让一个试运行实现通过独立运行授权复核。</p>
              </Show>
              <label>
                <span>已批准运行复核</span>
                <select value={selectedRunnerAuthorizationReviewId()} onChange={(event) => setSelectedRunnerAuthorizationReviewId(event.currentTarget.value)}>
                  <option value="">选择精确绑定的运行授权复核</option>
                  <For each={runnerRegistry().eligible_authorizations}>{(item) => (
                    <option value={item.review.review_id}>{item.implementation.asset_symbol} · {item.implementation.code_revision} · {item.review.review_sha256.slice(0, 12)}…</option>
                  )}</For>
                </select>
              </label>
              <label><span>执行器名称</span><input maxlength={120} value={runnerName()} onInput={(event) => setRunnerName(event.currentTarget.value)} /></label>
              <label><span>不可变执行器代码版本</span><input maxlength={160} placeholder="例如 oldwang@runner123" value={runnerCodeRevision()} onInput={(event) => setRunnerCodeRevision(event.currentTarget.value)} /></label>
              <label><span>执行器制品 SHA-256</span><input maxlength={64} placeholder="64 位十六进制摘要" value={runnerArtifactSha256()} onInput={(event) => setRunnerArtifactSha256(event.currentTarget.value)} /></label>
              <button
                type="button"
                class="public-admin-decision-submit"
                disabled={!runnerRegistry().current_runtime_artifact_sha256}
                onClick={() => {
                  setRunnerArtifactSha256(runnerRegistry().current_runtime_artifact_sha256 ?? "");
                  if (!runnerCodeRevision().trim()) setRunnerCodeRevision(runnerRegistry().current_runtime_git_sha ?? `runtime:${runnerRegistry().current_runtime_build_source}`);
                }}
              >
                填入当前后端制品指纹
              </button>
              <button type="button" class="public-admin-decision-submit" disabled={runnerRegisterDisabled()} onClick={() => void registerIsolatedRunner()}>
                写入不可覆盖的执行器规范
              </button>
              <p class="public-admin-anchor-boundary">当前后端制品来源：{runnerRegistry().current_runtime_build_source}。填入只用于精确绑定，复核者仍须独立核验摘要；登记不调用制品。首次执行授权、隔离输出校验和结果标签准入仍是后续独立门禁。</p>
            </details>
            <For each={runnerRegistry().runners}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.runner.asset_symbol} · {item.runner.runner_name}</strong>
                  <span>{item.run_authorization_binding_current ? "绑定有效" : "绑定失效"}</span>
                </header>
                <p>{item.runner.status} · {item.runner.runner_code_revision} · 规范 {item.runner.isolated_runner_spec_sha256.slice(0, 12)}…</p>
                <p>制品 {item.runner.runner_artifact_sha256.slice(0, 12)}… · 登记时入口：无 · 标签：否 · 交易：否</p>
              </article>
            )}</For>
          </section>
        )}</Show>
        <Show when={firstExecutionAuthorizations()}>{(executionRegistry) => (
          <section class="public-admin-reward-governance" aria-label="离线试运行首次执行授权复核">
            <header>
              <strong>首次执行授权复核</strong>
              <span>{executionRegistry().authorization_status}</span>
            </header>
            <p>{executionRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>可复核执行器</span><strong>{executionRegistry().review_eligible_runner_count}</strong></div>
              <div><span>已经复核</span><strong>{executionRegistry().reviewed_runner_count}</strong></div>
              <div><span>未过期一次性授权</span><strong>{executionRegistry().unexpired_authorization_count}</strong></div>
              <div><span>授权模块调用端点</span><strong>{executionRegistry().invocation_endpoint_available ? "存在" : "不存在"}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              这是第十道门禁。执行器登记者不能批准自己的首次执行；批准只在 24 小时内提供一次调用额度。授权模块本身不调用；第十一阶段可消费该额度一次，且输出仍不能进入标签、训练、奖励、影子、订单、券商或交易。
            </p>
            <For each={executionRegistry().items}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.runner.asset_symbol} · {item.runner.runner_name}</strong>
                  <span>{item.one_shot_first_execution_authorized && item.authorization_unexpired ? "一次性授权有效" : item.one_shot_first_execution_authorized ? "一次性授权已过期" : item.latest_review ? "已复核未授权" : "待独立复核"}</span>
                </header>
                <p>{item.runner.runner_code_revision} · 制品 {item.runner.runner_artifact_sha256.slice(0, 12)}… · 规范 {item.runner.isolated_runner_spec_sha256.slice(0, 12)}…</p>
                <p>最近复核：{item.latest_review?.verdict ?? "尚未复核"}{item.latest_review ? ` · 有效至 ${new Date(item.latest_review.authorization_valid_until).toLocaleString("zh-CN")}` : ""}</p>
                <button type="button" class="public-admin-decision-submit" onClick={() => setSelectedFirstExecutionRunnerId(item.runner.isolated_runner_id)}>
                  选择并复核首次执行边界
                </button>
                <Show when={item.current_binding && item.authorization_unexpired && item.latest_review && !executionAttempts()?.attempts.some((attempt) => attempt.claim.isolated_runner_id === item.runner.isolated_runner_id)}>
                  <button type="button" class="public-admin-decision-submit" disabled={busy()} onClick={() => void invokeOnce(item.runner.isolated_runner_id)}>
                    消费授权并执行一次能力隔离回放
                  </button>
                </Show>
              </article>
            )}</For>
            <Show when={selectedFirstExecutionAuthorization()}>{(selected) => (
              <details class="public-admin-reward-governance" open>
                <summary>复核 {selected().runner.asset_symbol} 的一次性首次执行资格</summary>
                <label>
                  <span>复核结论</span>
                  <select value={firstExecutionVerdict()} onChange={(event) => setFirstExecutionVerdict(event.currentTarget.value as HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict)}>
                    <option value="approved_for_one_shot_first_execution">批准 24 小时内一次首次执行（当前不调用）</option>
                    <option value="changes_requested">要求补齐或修订</option>
                    <option value="rejected">拒绝授权</option>
                  </select>
                </label>
                <label><span>复核依据</span><textarea maxlength={2400} value={firstExecutionRationale()} onInput={(event) => setFirstExecutionRationale(event.currentTarget.value)} /></label>
                <Show when={firstExecutionApprovalSelected()}>
                  <For each={FIRST_EXECUTION_AUTHORIZATION_CHECKS}>{(label, index) => (
                    <label class="public-admin-reward-confirm"><input type="checkbox" checked={firstExecutionChecks()[index()]} onChange={(event) => toggleFirstExecutionCheck(index(), event.currentTarget.checked)} /> {label}</label>
                  )}</For>
                </Show>
                <button type="button" class="public-admin-decision-submit" disabled={firstExecutionReviewDisabled()} onClick={() => void reviewFirstExecutionAuthorization()}>
                  写入不可覆盖的首次执行授权复核
                </button>
                <p class="public-admin-anchor-boundary">批准不会调用执行器。实际单次运行、输出工件校验和结果标签准入仍是后续独立门禁。</p>
              </details>
            )}</Show>
          </section>
        )}</Show>
        <Show when={executionAttempts()}>{(attemptRegistry) => (
          <section class="public-admin-reward-governance" aria-label="历史结果一次性能力隔离执行">
            <header>
              <strong>一次性能力隔离执行</strong>
              <span>{attemptRegistry().execution_status}</span>
            </header>
            <p>{attemptRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>可消费授权</span><strong>{attemptRegistry().invocation_eligible_authorization_count}</strong></div>
              <div><span>执行 claim</span><strong>{attemptRegistry().attempt_count}</strong></div>
              <div><span>完成 / 失败</span><strong>{attemptRegistry().completed_attempt_count} / {attemptRegistry().failed_attempt_count}</strong></div>
              <div><span>未验证输出</span><strong>{attemptRegistry().untrusted_output_count}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              第十一阶段会先不可变地消费授权并写入 claim，再运行有静态输入上限的纯函数。输出虽有 claim、result 与内容哈希，仍必须经过下一道独立结构校验和确定性重算。
            </p>
            <For each={attemptRegistry().attempts}>{(attempt) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{attempt.result?.untrusted_output?.asset_symbol ?? attempt.claim.isolated_runner_id}</strong>
                  <span>{attempt.result?.status ?? "已 claim，失败关闭"}</span>
                </header>
                <p>claim {attempt.claim.claim_sha256.slice(0, 12)}… · 制品复核 {attempt.claim.artifact_digest_reverified ? "通过" : "失败"} · 快照复核 {attempt.claim.sealed_snapshot_revalidated ? "通过" : "失败"}</p>
                <Show when={attempt.result}>{(result) => (
                  <>
                    <p>result {result().result_sha256.slice(0, 12)}… · 用时 {result().duration_millis} ms · 临时目录清理 {result().ephemeral_directory_removed ? "确认" : "失败"}</p>
                    <Show when={result().untrusted_output}>{(output) => (
                      <div class="public-admin-decision-metrics">
                        <For each={output().metrics}>{(metric) => (
                          <div>
                            <span>{metric.horizon_market_sessions} 日 · {metric.start_date} → {metric.end_date}</span>
                            <strong>{formatRate(metric.asset_return)}</strong>
                            <small>SPY {formatRate(metric.benchmark_return)} · 超额 {formatRate(metric.excess_return)} · 最大回撤 {formatRate(metric.asset_max_drawdown)}</small>
                          </div>
                        )}</For>
                      </div>
                    )}</Show>
                  </>
                )}</Show>
              </article>
            )}</For>
          </section>
        )}</Show>
        <Show when={outputValidations()}>{(validationRegistry) => (
          <section class="public-admin-reward-governance" aria-label="历史结果独立输出校验">
            <header>
              <strong>独立输出校验与确定性重算</strong>
              <span>{validationRegistry().validation_status}</span>
            </header>
            <p>{validationRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>待独立校验</span><strong>{validationRegistry().validation_eligible_count}</strong></div>
              <div><span>校验记录</span><strong>{validationRegistry().validation_count}</strong></div>
              <div><span>重算一致</span><strong>{validationRegistry().validated_output_count}</strong></div>
              <div><span>失败关闭</span><strong>{validationRegistry().failed_validation_count}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              第十二阶段由不同管理员核对精确 claim、result、output 与当前封存快照，并使用不调用执行实现的第二套代码逐位重算。执行调用人、运行器登记者和两级授权复核人都不能担任本次校验人。
            </p>
            <For each={validationRegistry().items}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.attempt.result.untrusted_output?.asset_symbol ?? item.attempt.claim.attempt_id}</strong>
                  <span>{item.validation?.output_validated ? "重算一致" : item.validation ? "不一致，失败关闭" : "等待独立校验"}</span>
                </header>
                <p>output {item.attempt.result.output_sha256?.slice(0, 12)}… · validator {validationRegistry().validator_implementation_sha256.slice(0, 12)}…</p>
                <Show when={item.validation}>{(validation) => (
                  <p>
                    结构 {validation().output_structure_verified ? "通过" : "失败"} · 哈希 {validation().canonical_output_hash_verified ? "通过" : "失败"} · 重算 {validation().deterministic_recomputation_match ? "一致" : "不一致"}
                    <Show when={validation().mismatch_reasons.length > 0}> · {validation().mismatch_reasons.join("；")}</Show>
                  </p>
                )}</Show>
                <Show when={item.validation_eligible}>
                  <button type="button" class="public-admin-decision-submit" disabled={busy()} onClick={() => void validateOutput(item.attempt.claim.attempt_id)}>
                    以独立身份校验并确定性重算
                  </button>
                </Show>
              </article>
            )}</For>
            <p class="public-admin-anchor-boundary">校验通过只证明该输出与封存输入确定性一致；结果标签准入、训练、奖励、影子组合、订单、券商和交易仍全部关闭。</p>
          </section>
        )}</Show>
        <Show when={labelAdmissions()}>{(admissionRegistry) => (
          <section class="public-admin-reward-governance" aria-label="历史结果标签准入复核">
            <header>
              <strong>结果标签准入复核</strong>
              <span>{admissionRegistry().admission_status}</span>
            </header>
            <p>{admissionRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>重算一致输出</span><strong>{admissionRegistry().independently_validated_output_count}</strong></div>
              <div><span>已复核</span><strong>{admissionRegistry().reviewed_output_count}</strong></div>
              <div><span>准入未来物化</span><strong>{admissionRegistry().admitted_output_count}</strong></div>
              <div><span>修订 / 拒绝</span><strong>{admissionRegistry().changes_requested_or_rejected_count}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              第十三阶段审阅“这份重算一致结果能否作为未来标签输入”，而不是再次算收益。复核人必须独立于校验、执行、登记和两级授权角色，并必须明确记录缺失、样本选择和幸存者偏差。
            </p>
            <For each={admissionRegistry().items}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.asset_symbol} / {item.benchmark_symbol}</strong>
                  <span>{item.outcome_label_input_admitted ? "已准入未来物化" : item.latest_review ? "未准入" : "等待准入复核"}</span>
                </header>
                <p>validation {item.validation.validation_sha256.slice(0, 12)}… · output {item.validation.output_sha256.slice(0, 12)}… · 判断可用时间 {new Date(item.decision_available_at).toLocaleString("zh-CN")}</p>
                <Show when={item.latest_review}>{(review) => (
                  <>
                    <p>{review().verdict} · reviewer {review().reviewer_id} · review {review().review_sha256.slice(0, 12)}…</p>
                    <p><strong>已知局限：</strong>{review().known_limitations}</p>
                  </>
                )}</Show>
                <button type="button" class="public-admin-decision-submit" disabled={busy()} onClick={() => setSelectedLabelAdmissionAttemptId(item.validation.attempt_id)}>
                  {item.latest_review ? "追加准入复核" : "开始独立准入复核"}
                </button>
              </article>
            )}</For>
            <Show when={selectedLabelAdmission()}>{(selected) => (
              <details class="public-admin-reward-governance" open>
                <summary>复核 {selected().asset_symbol} 的未来结果标签输入资格</summary>
                <label>
                  <span>复核结论</span>
                  <select value={labelAdmissionVerdict()} onChange={(event) => setLabelAdmissionVerdict(event.currentTarget.value as HistoricalOutcomeLabelAdmissionVerdict)}>
                    <option value="approved_for_future_label_materialization">批准作为未来标签物化输入（当前不写标签）</option>
                    <option value="changes_requested">要求补齐或修订</option>
                    <option value="rejected">拒绝准入</option>
                  </select>
                </label>
                <label><span>复核依据</span><textarea maxlength={2400} value={labelAdmissionRationale()} onInput={(event) => setLabelAdmissionRationale(event.currentTarget.value)} /></label>
                <label><span>已知局限与偏差（必填）</span><textarea maxlength={2400} value={labelAdmissionLimitations()} onInput={(event) => setLabelAdmissionLimitations(event.currentTarget.value)} /></label>
                <Show when={labelAdmissionApprovalSelected()}>
                  <For each={LABEL_ADMISSION_CHECKS}>{(label, index) => (
                    <label class="public-admin-reward-confirm"><input type="checkbox" checked={labelAdmissionChecks()[index()]} onChange={(event) => toggleLabelAdmissionCheck(index(), event.currentTarget.checked)} /> {label}</label>
                  )}</For>
                </Show>
                <button type="button" class="public-admin-decision-submit" disabled={labelAdmissionReviewDisabled()} onClick={() => void reviewLabelAdmission()}>
                  写入不可覆盖的标签准入复核
                </button>
                <p class="public-admin-anchor-boundary">批准只开放未来标签物化实现的登记资格。标签仍未写入，训练、奖励、影子组合、订单、券商和交易全部关闭。</p>
              </details>
            )}</Show>
          </section>
        )}</Show>
        <Show when={labelMaterializationImplementations()}>{(materializationRegistry) => (
          <section class="public-admin-reward-governance" aria-label="历史结果标签物化实现登记">
            <header>
              <strong>原始结果信封物化实现登记</strong>
              <span>{materializationRegistry().implementation_status}</span>
            </header>
            <p>{materializationRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>已准入输出</span><strong>{materializationRegistry().admitted_output_count}</strong></div>
              <div><span>实现规范</span><strong>{materializationRegistry().implementation_count}</strong></div>
              <div><span>当前精确绑定</span><strong>{materializationRegistry().current_binding_implementation_count}</strong></div>
              <div><span>可送运行授权复核</span><strong>{materializationRegistry().run_authorization_review_eligible_count}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              第十四阶段只冻结“如何逐位封装已验证原始结果”的实现规范。它不得补数、重算或人工改写指标，也不得从收益推断方向、评级、买卖动作、仓位或奖励；当前没有运行或写入标签。
            </p>
            <For each={materializationRegistry().implementations}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.implementation.asset_symbol} / {item.implementation.benchmark_symbol}</strong>
                  <span>{item.admission_binding_current ? "当前绑定 · 未运行" : "上游绑定已过期"}</span>
                </header>
                <p>{item.implementation.implementation_name} · {item.implementation.code_revision}</p>
                <p>spec {item.implementation.materialization_implementation_spec_sha256.slice(0, 12)}… · admission {item.implementation.admission_review_sha256.slice(0, 12)}… · validation {item.implementation.validation_sha256.slice(0, 12)}…</p>
                <p><strong>输出边界：</strong>{item.implementation.output_fields.join(" · ")}</p>
                <p><strong>已知局限原样保留：</strong>{item.implementation.admission_known_limitations}</p>
                <p class="public-admin-anchor-boundary">状态 {item.implementation.status}；运行授权 {item.implementation.label_materialization_run_authorized ? "开启" : "关闭"}；标签写入 {item.implementation.outcome_label_written ? "已写" : "未写"}。</p>
              </article>
            )}</For>
            <Show when={materializationRegistry().registration_allowed}>
              <details class="public-admin-reward-governance">
                <summary>登记一个不可覆盖的原始结果信封物化规范</summary>
                <label>
                  <span>精确准入输出</span>
                  <select value={selectedMaterializationAdmissionAttemptId()} onChange={(event) => setSelectedMaterializationAdmissionAttemptId(event.currentTarget.value)}>
                    <option value="">请选择</option>
                    <For each={materializationRegistry().eligible_admissions}>{(admission) => (
                      <option value={admission.attempt_id}>{admission.asset_symbol} · {admission.attempt_id.slice(0, 12)}… · review {admission.admission_review_sha256.slice(0, 8)}…</option>
                    )}</For>
                  </select>
                </label>
                <Show when={selectedMaterializationAdmission()}>{(admission) => (
                  <p>validation {admission().validation_sha256.slice(0, 12)}… · output {admission().output_sha256.slice(0, 12)}… · snapshot {admission().snapshot_sha256.slice(0, 12)}…</p>
                )}</Show>
                <label><span>实现名称</span><input maxlength={120} value={materializationImplementationName()} onInput={(event) => setMaterializationImplementationName(event.currentTarget.value)} /></label>
                <label><span>不可变代码版本</span><input maxlength={160} placeholder="例如 git commit SHA 或制品版本" value={materializationCodeRevision()} onInput={(event) => setMaterializationCodeRevision(event.currentTarget.value)} /></label>
                <button type="button" class="public-admin-decision-submit" disabled={materializationImplementationRegisterDisabled()} onClick={() => void registerLabelMaterializationImplementation()}>
                  仅登记物化实现规范（不运行）
                </button>
                <p class="public-admin-anchor-boundary">登记完成后仍需独立运行授权复核。当前不写标签、不训练、不奖励、不建立影子组合、不生成订单、不访问券商、不交易。</p>
              </details>
            </Show>
          </section>
        )}</Show>
        <Show when={materializationRunAuthorizations()}>{(authorizationRegistry) => (
          <section class="public-admin-reward-governance" aria-label="历史结果标签物化运行授权复核">
            <header>
              <strong>标签物化运行授权复核</strong>
              <span>{authorizationRegistry().authorization_status}</span>
            </header>
            <p>{authorizationRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>可复核实现</span><strong>{authorizationRegistry().review_eligible_implementation_count}</strong></div>
              <div><span>已独立复核</span><strong>{authorizationRegistry().reviewed_implementation_count}</strong></div>
              <div><span>可登记隔离 runner</span><strong>{authorizationRegistry().materialization_runner_registration_eligible_count}</strong></div>
              <div><span>已写标签</span><strong>{authorizationRegistry().outcome_label_written ? "是" : "否"}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              第十五阶段只审查实现是否能被未来隔离运行。复核人必须独立于实现登记者、标签准入人、输出校验人和此前执行链全部角色；即使批准，也只允许下一阶段登记 runner 规范，当前不运行、不写标签。
            </p>
            <For each={authorizationRegistry().items}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.implementation.asset_symbol} · {item.implementation.implementation_name}</strong>
                  <span>{item.materialization_runner_registration_eligible ? "批准登记 runner · 未运行" : item.latest_review ? "已复核 · 未批准" : "等待独立复核"}</span>
                </header>
                <p>implementation {item.implementation.materialization_implementation_id} · spec {item.implementation.materialization_implementation_spec_sha256.slice(0, 12)}…</p>
                <Show when={item.latest_review}>{(review) => (
                  <>
                    <p><strong>最新结论：</strong>{review().verdict} · {review().rationale}</p>
                    <p>review {review().review_sha256.slice(0, 12)}… · reviewer {review().reviewer_id}</p>
                  </>
                )}</Show>
                <p class="public-admin-anchor-boundary">runner {item.latest_review?.materialization_runner_registered ? "已登记" : "未登记"}；运行授权 {item.latest_review?.label_materialization_run_authorized ? "开启" : "关闭"}；标签写入 {item.latest_review?.outcome_label_written ? "已写" : "未写"}。</p>
              </article>
            )}</For>
            <Show when={authorizationRegistry().review_eligible_implementation_count > 0}>
              <details class="public-admin-reward-governance">
                <summary>写入不可覆盖的标签物化运行授权复核</summary>
                <label>
                  <span>精确物化实现</span>
                  <select value={selectedMaterializationRunImplementationId()} onChange={(event) => setSelectedMaterializationRunImplementationId(event.currentTarget.value)}>
                    <option value="">请选择</option>
                    <For each={authorizationRegistry().items}>{(item) => (
                      <option value={item.implementation.materialization_implementation_id}>{item.implementation.asset_symbol} · {item.implementation.implementation_name} · {item.implementation.materialization_implementation_id.slice(0, 12)}…</option>
                    )}</For>
                  </select>
                </label>
                <label>
                  <span>复核结论</span>
                  <select value={materializationRunAuthorizationVerdict()} onChange={(event) => setMaterializationRunAuthorizationVerdict(event.currentTarget.value as HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict)}>
                    <option value="approved_for_materialization_runner_registration">批准下一步登记隔离物化 runner（当前不运行）</option>
                    <option value="changes_requested">要求修订</option>
                    <option value="rejected">拒绝</option>
                  </select>
                </label>
                <label><span>独立复核依据</span><textarea maxlength={2400} value={materializationRunAuthorizationRationale()} onInput={(event) => setMaterializationRunAuthorizationRationale(event.currentTarget.value)} /></label>
                <Show when={materializationRunApprovalSelected()}>
                  <For each={MATERIALIZATION_RUN_AUTHORIZATION_CHECKS}>{(label, index) => (
                    <label class="public-admin-reward-confirm"><input type="checkbox" checked={materializationRunAuthorizationChecks()[index()]} onChange={(event) => toggleMaterializationRunAuthorizationCheck(index(), event.currentTarget.checked)} /> {label}</label>
                  )}</For>
                </Show>
                <button type="button" class="public-admin-decision-submit" disabled={materializationRunAuthorizationReviewDisabled()} onClick={() => void reviewLabelMaterializationRunAuthorization()}>
                  写入不可覆盖的标签物化运行授权复核
                </button>
                <p class="public-admin-anchor-boundary">批准后 runner 仍未登记，代码仍未运行，结果标签仍未写入；训练、奖励、影子、订单、券商和交易全部关闭。</p>
              </details>
            </Show>
          </section>
        )}</Show>
        <Show when={materializationIsolatedRunners()}>{(runnerRegistry) => (
          <section class="public-admin-reward-governance" aria-label="历史结果标签物化隔离 runner 登记">
            <header>
              <strong>标签物化隔离 runner 规范</strong>
              <span>{runnerRegistry().runner_status}</span>
            </header>
            <p>{runnerRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>runner 规范</span><strong>{runnerRegistry().runner_count}</strong></div>
              <div><span>当前精确绑定</span><strong>{runnerRegistry().current_binding_runner_count}</strong></div>
              <div><span>可送首次执行复核</span><strong>{runnerRegistry().execution_authorization_review_eligible_count}</strong></div>
              <div><span>调用入口</span><strong>{runnerRegistry().invocation_authorized ? "存在" : "不存在"}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              第十六阶段只登记 runner 制品摘要、代码版本、只读输入、create-once 隔离输出和固定资源上限。登记后仍须由独立角色复核一次性首次执行资格；当前不运行、不创建标签工件。
            </p>
            <Show when={runnerRegistry().registration_allowed}>
              <details class="public-admin-reward-governance">
                <summary>登记不可覆盖的标签物化 runner（只登记，不调用）</summary>
                <label>
                  <span>已批准物化运行复核</span>
                  <select value={selectedMaterializationRunnerAuthorizationReviewId()} onChange={(event) => setSelectedMaterializationRunnerAuthorizationReviewId(event.currentTarget.value)}>
                    <option value="">选择精确绑定的复核记录</option>
                    <For each={runnerRegistry().eligible_authorizations}>{(item) => (
                      <option value={item.review.review_id}>{item.implementation.asset_symbol} · {item.implementation.code_revision} · {item.review.review_sha256.slice(0, 12)}…</option>
                    )}</For>
                  </select>
                </label>
                <label><span>runner 名称</span><input maxlength={120} value={materializationRunnerName()} onInput={(event) => setMaterializationRunnerName(event.currentTarget.value)} /></label>
                <label><span>不可变 runner 代码版本</span><input maxlength={160} placeholder="例如 oldwang@materialization-runner123" value={materializationRunnerCodeRevision()} onInput={(event) => setMaterializationRunnerCodeRevision(event.currentTarget.value)} /></label>
                <label><span>runner 制品 SHA-256</span><input maxlength={64} placeholder="64 位十六进制摘要" value={materializationRunnerArtifactSha256()} onInput={(event) => setMaterializationRunnerArtifactSha256(event.currentTarget.value)} /></label>
                <button
                  type="button"
                  class="public-admin-decision-submit"
                  disabled={!runnerRegistry().current_runtime_artifact_sha256}
                  onClick={() => {
                    setMaterializationRunnerArtifactSha256(runnerRegistry().current_runtime_artifact_sha256 ?? "");
                    if (!materializationRunnerCodeRevision().trim()) setMaterializationRunnerCodeRevision(runnerRegistry().current_runtime_git_sha ?? `runtime:${runnerRegistry().current_runtime_build_source}`);
                  }}
                >
                  填入当前后端制品指纹
                </button>
                <button type="button" class="public-admin-decision-submit" disabled={materializationRunnerRegisterDisabled()} onClick={() => void registerLabelMaterializationIsolatedRunner()}>
                  写入不可覆盖的 runner 规范
                </button>
                <p class="public-admin-anchor-boundary">填入当前制品只用于精确绑定，后续复核者仍须独立核验。登记不调用制品，不写标签，不开放训练或交易。</p>
              </details>
            </Show>
            <For each={runnerRegistry().runners}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.runner.asset_symbol} · {item.runner.runner_name}</strong>
                  <span>{item.run_authorization_binding_current ? "绑定有效 · 未运行" : "绑定失效"}</span>
                </header>
                <p>{item.runner.runner_code_revision} · 制品 {item.runner.runner_artifact_sha256.slice(0, 12)}… · 规范 {item.runner.isolated_runner_spec_sha256.slice(0, 12)}…</p>
                <p>上限 {item.runner.max_wall_clock_seconds}s / {item.runner.max_memory_mib}MiB / {item.runner.max_cpu_millicores}mCPU / {item.runner.max_output_bytes} bytes</p>
                <p class="public-admin-anchor-boundary">入口：无；运行：否；标签：否；训练：否；交易：否。下一步只能进入独立首次执行授权复核。</p>
              </article>
            )}</For>
          </section>
        )}</Show>
        <Show when={materializationFirstExecutionAuthorizations()}>{(authorizationRegistry) => (
          <section class="public-admin-reward-governance" aria-label="历史结果标签物化首次执行授权复核">
            <header>
              <strong>标签物化首次执行授权复核</strong>
              <span>{authorizationRegistry().authorization_status}</span>
            </header>
            <p>{authorizationRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>可独立复核 runner</span><strong>{authorizationRegistry().review_eligible_runner_count}</strong></div>
              <div><span>已复核</span><strong>{authorizationRegistry().reviewed_runner_count}</strong></div>
              <div><span>一次性批准</span><strong>{authorizationRegistry().one_shot_first_execution_authorized_count}</strong></div>
              <div><span>当前未过期</span><strong>{authorizationRegistry().unexpired_authorization_count}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              第十七阶段只建立短时、一次性的未来首次执行授权。物化 runner 登记者及物化实现、准入、校验和原历史执行链角色均不得自批；授权记录是否仍在时限内由本区展示，是否已经消费以第十八阶段不可覆盖的执行 claim 为准。
            </p>
            <For each={authorizationRegistry().items}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.runner.asset_symbol} · {item.runner.runner_name}</strong>
                  <span>{item.authorization_unexpired ? "授权仍在 24 小时时限内" : item.latest_review ? "已复核 · 当前无有效额度" : "等待独立复核"}</span>
                </header>
                <p>制品 {item.runner.runner_artifact_sha256.slice(0, 12)}… · runner 规范 {item.runner.isolated_runner_spec_sha256.slice(0, 12)}…</p>
                <Show when={item.latest_review}>{(review) => (
                  <p>复核 {review().review_sha256.slice(0, 12)}… · {review().reviewer_id} · 有效至 {new Date(review().authorization_valid_until).toLocaleString("zh-CN")}</p>
                )}</Show>
                <p class="public-admin-anchor-boundary">时间有效性不等于未消费；消费状态请以下方一次性执行记录为准。标签、训练、奖励、影子、订单、券商与交易：关闭。</p>
              </article>
            )}</For>
            <Show when={authorizationRegistry().review_eligible_runner_count > 0}>
              <details class="public-admin-reward-governance">
                <summary>独立复核一次性首次执行资格（当前不调用）</summary>
                <label>
                  <span>当前绑定 runner</span>
                  <select value={selectedMaterializationFirstExecutionRunnerId()} onChange={(event) => setSelectedMaterializationFirstExecutionRunnerId(event.currentTarget.value)}>
                    <option value="">选择精确 runner 规范</option>
                    <For each={authorizationRegistry().items}>{(item) => (
                      <option value={item.runner.isolated_runner_id}>{item.runner.asset_symbol} · {item.runner.runner_code_revision} · {item.runner.runner_artifact_sha256.slice(0, 12)}…</option>
                    )}</For>
                  </select>
                </label>
                <Show when={selectedMaterializationFirstExecutionAuthorization()}>{(_selected) => (
                  <>
                    <label>
                      <span>复核结论</span>
                      <select value={materializationFirstExecutionVerdict()} onChange={(event) => setMaterializationFirstExecutionVerdict(event.currentTarget.value as HistoricalOutcomeLabelMaterializationFirstExecutionAuthorizationVerdict)}>
                        <option value="approved_for_one_shot_first_execution">批准 24 小时内一次未来首次执行（当前不调用）</option>
                        <option value="changes_requested">要求修订</option>
                        <option value="rejected">拒绝</option>
                      </select>
                    </label>
                    <label><span>独立复核依据</span><textarea maxlength={2400} value={materializationFirstExecutionRationale()} onInput={(event) => setMaterializationFirstExecutionRationale(event.currentTarget.value)} /></label>
                    <Show when={materializationFirstExecutionApprovalSelected()}>
                      <For each={MATERIALIZATION_FIRST_EXECUTION_AUTHORIZATION_CHECKS}>{(label, index) => (
                        <label class="public-admin-reward-confirm"><input type="checkbox" checked={materializationFirstExecutionChecks()[index()]} onChange={(event) => toggleMaterializationFirstExecutionCheck(index(), event.currentTarget.checked)} /> {label}</label>
                      )}</For>
                    </Show>
                    <button type="button" class="public-admin-decision-submit" disabled={materializationFirstExecutionReviewDisabled()} onClick={() => void reviewLabelMaterializationFirstExecutionAuthorization()}>
                      写入不可覆盖的首次执行授权复核
                    </button>
                    <p class="public-admin-anchor-boundary">批准本身不调用 runner；实际调用必须先写 create-once claim，并在执行前重新核验制品与全部上游。</p>
                  </>
                )}</Show>
              </details>
            </Show>
          </section>
        )}</Show>
        <Show when={materializationExecutionAttempts()}>{(attemptRegistry) => (
          <section class="public-admin-reward-governance" aria-label="历史结果标签物化一次性执行">
            <header>
              <strong>标签物化一次性执行</strong>
              <span>{attemptRegistry().execution_status}</span>
            </header>
            <p>{attemptRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>可消费授权</span><strong>{attemptRegistry().invocation_eligible_authorization_count}</strong></div>
              <div><span>执行 claim</span><strong>{attemptRegistry().attempt_count}</strong></div>
              <div><span>未信任结果包</span><strong>{attemptRegistry().untrusted_envelope_count}</strong></div>
              <div><span>待独立校验</span><strong>{Math.max(0, attemptRegistry().independent_validation_eligible_count - (materializationOutputValidations()?.validation_count ?? 0))}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              第十八阶段先持久化 create-once claim，再运行无环境、无网络、无工具、无子进程、无生产数据能力的固定纯函数。失败也消费授权；成功只复制已独立验证的 20 / 60 / 250 日原始指标、来源和已知局限，绝不生成正式标签。
            </p>
            <Show when={invokableMaterializationAuthorization()}>{(selected) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{selected().runner.asset_symbol} · 精确一次性物化</strong>
                  <span>执行前将重验制品与全部上游</span>
                </header>
                <p>runner {selected().runner.runner_artifact_sha256.slice(0, 12)}… · 规范 {selected().runner.isolated_runner_spec_sha256.slice(0, 12)}…</p>
                <button type="button" class="public-admin-decision-submit" disabled={materializationInvocationDisabled()} onClick={() => void invokeLabelMaterializationOnce()}>
                  消费一次性授权并执行固定物化
                </button>
                <p class="public-admin-anchor-boundary">此动作不可重放。失败会留下失败结果并永久消费该授权；成功结果仍须独立校验，不能用于标签、训练、奖励、影子持仓或交易。</p>
              </article>
            )}</Show>
            <For each={attemptRegistry().attempts}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.result?.untrusted_envelope?.asset_symbol ?? "物化执行"} · {item.claim.attempt_id}</strong>
                  <span>{item.result?.status ?? "claim 已写入 · 失败关闭"}</span>
                </header>
                <p>claim {item.claim.claim_sha256.slice(0, 12)}… · 授权 {item.claim.authorization_review_sha256.slice(0, 12)}… · 调用人 {item.claim.invoked_by}</p>
                <Show when={item.result?.untrusted_envelope}>{(envelope) => (
                  <>
                    <p>未信任输出 {item.result?.output_sha256?.slice(0, 12)}… · {envelope().asset_symbol} / {envelope().benchmark_symbol} · 共同交易日 {envelope().common_session_count}</p>
                    <div class="public-admin-decision-metrics">
                      <For each={envelope().raw_validated_metrics}>{(metric) => (
                        <div>
                          <span>{metric.horizon_market_sessions} 日超额 / 回撤</span>
                          <strong>{formatRate(metric.excess_return)} / {formatRate(metric.asset_max_drawdown)}</strong>
                        </div>
                      )}</For>
                    </div>
                    <p>已知局限：{envelope().known_limitations}</p>
                    <p class="public-admin-anchor-boundary">未信任：是；独立校验：{materializationOutputValidationForAttempt(item.claim.attempt_id)?.untrusted_envelope_validated ? "结构、来源与位模式一致，但仍不是标签" : materializationOutputValidationForAttempt(item.claim.attempt_id) ? "失败关闭" : "未完成"}；方向、评级、动作、仓位：未推断；标签、训练、奖励、影子、订单、券商与交易：均未写入或授权。</p>
                  </>
                )}</Show>
                <Show when={item.result && !item.result.untrusted_envelope}>
                  <p class="public-admin-anchor-boundary">执行失败且授权已消费。失败记录不可覆盖；必须重新走独立授权链，不能重放本次授权。</p>
                </Show>
              </article>
            )}</For>
          </section>
        )}</Show>
        <Show when={materializationOutputValidations()}>{(validationRegistry) => (
          <section class="public-admin-reward-governance" aria-label="历史结果标签物化结果独立校验">
            <header>
              <strong>标签物化结果独立校验</strong>
              <span>{validationRegistry().validation_status}</span>
            </header>
            <p>{validationRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>待校验结果包</span><strong>{validationRegistry().validation_eligible_count}</strong></div>
              <div><span>校验记录</span><strong>{validationRegistry().validation_count}</strong></div>
              <div><span>逐位一致</span><strong>{validationRegistry().validated_envelope_count}</strong></div>
              <div><span>失败关闭</span><strong>{validationRegistry().failed_validation_count}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              第十九阶段由独立管理员和独立校验实现重读未信任结果包、准入记录与封存上游，核对规范结构、完整来源和 20 / 60 / 250 日指标的 IEEE-754 位模式；不复用物化投影代码。
            </p>
            <Show when={eligibleMaterializationOutputValidation()}>{(selected) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{selected().attempt.result.untrusted_envelope?.asset_symbol ?? "结果包"} · 独立逐位校验</strong>
                  <span>校验前重验全部绑定</span>
                </header>
                <p>attempt {selected().attempt.claim.attempt_id} · output {selected().attempt.result.output_sha256?.slice(0, 12)}…</p>
                <button type="button" class="public-admin-decision-submit" disabled={materializationOutputValidationDisabled()} onClick={() => void validateLabelMaterializationOutput()}>
                  独立校验结构、来源与逐位一致性
                </button>
                <p class="public-admin-anchor-boundary">校验记录 create-once 且不可覆盖。通过仍不是正式结果标签；失败则保持失败关闭，不能人工改数或绕过重跑。</p>
              </article>
            )}</Show>
            <For each={validationRegistry().items.filter((item) => item.validation)}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.attempt.result.untrusted_envelope?.asset_symbol ?? "结果包"} · {item.validation?.validation_id}</strong>
                  <span>{item.validation?.verdict}</span>
                </header>
                <p>validator {item.validation?.validator_implementation_version} · validation {item.validation?.validation_sha256.slice(0, 12)}… · output {item.validation?.output_sha256.slice(0, 12)}…</p>
                <div class="public-admin-decision-metrics">
                  <div><span>结构</span><strong>{item.validation?.output_structure_verified ? "一致" : "失败"}</strong></div>
                  <div><span>来源</span><strong>{item.validation?.provenance_match ? "一致" : "失败"}</strong></div>
                  <div><span>指标位模式</span><strong>{item.validation?.exact_metric_bits_match ? "一致" : "失败"}</strong></div>
                  <div><span>局限</span><strong>{item.validation?.known_limitations_match ? "一致" : "失败"}</strong></div>
                </div>
                <Show when={(item.validation?.mismatch_reasons.length ?? 0) > 0}>
                  <p>不一致原因：{item.validation?.mismatch_reasons.join("；")}</p>
                </Show>
                <p class="public-admin-anchor-boundary">通过仍不是标签；结果标签准入授权、标签写入、训练、奖励、影子、订单、券商与交易权限全部关闭。</p>
              </article>
            )}</For>
          </section>
        )}</Show>
        <Show when={labelWriteAuthorizations()}>{(authorizationRegistry) => (
          <section class="public-admin-reward-governance" aria-label="正式标签未来一次写入授权复核">
            <header>
              <strong>正式标签未来一次写入授权复核</strong>
              <span>{authorizationRegistry().authorization_status}</span>
            </header>
            <p>{authorizationRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>可复核</span><strong>{authorizationRegistry().review_eligible_count}</strong></div>
              <div><span>已复核</span><strong>{authorizationRegistry().reviewed_count}</strong></div>
              <div><span>一次性批准</span><strong>{authorizationRegistry().one_shot_authorized_count}</strong></div>
              <div><span>未过期额度</span><strong>{authorizationRegistry().unexpired_authorization_count}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              第二十阶段只复核一条第十九阶段通过校验的精确原始结果包。批准本身不是写入；第 21 阶段 writer 只接受当前未过期且未消费的批准，并且方向、动作、仓位和奖励均不推断。
            </p>
            <label>
              <span>通过独立校验的结果包</span>
              <select value={selectedLabelWriteValidationId()} onChange={(event) => setSelectedLabelWriteValidationId(event.currentTarget.value)}>
                <option value="">请选择待复核结果包</option>
                <For each={authorizationRegistry().items}>{(item) => (
                  <option value={item.materialization_validation_id}>
                    {item.asset_symbol} · {item.materialization_validation_id.slice(0, 12)}…
                  </option>
                )}</For>
              </select>
            </label>
            <Show when={selectedLabelWriteAuthorization()}>{(selected) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{selected().asset_symbol} / {selected().benchmark_symbol}</strong>
                  <span>{selected().latest_review?.verdict ?? "尚未复核"}</span>
                </header>
                <p>validation {selected().materialization_validation_sha256.slice(0, 12)}… · output {selected().output_sha256.slice(0, 12)}… · label contract {authorizationRegistry().label_contract_sha256.slice(0, 12)}…</p>
                <label>
                  <span>复核结论</span>
                  <select value={labelWriteAuthorizationVerdict()} onChange={(event) => setLabelWriteAuthorizationVerdict(event.currentTarget.value as HistoricalOutcomeLabelWriteAuthorizationVerdict)}>
                    <option value="approved_for_one_shot_formal_label_write">批准 24 小时内一次未来写入</option>
                    <option value="changes_requested">要求修改</option>
                    <option value="rejected">拒绝</option>
                  </select>
                </label>
                <label>
                  <span>独立复核依据</span>
                  <textarea value={labelWriteAuthorizationRationale()} onInput={(event) => setLabelWriteAuthorizationRationale(event.currentTarget.value)} placeholder="说明完整链路、标签合同、局限与权限边界的独立核对结论" />
                </label>
                <div class="public-admin-decision-checklist">
                  <For each={FORMAL_LABEL_WRITE_AUTHORIZATION_CHECKS}>{(label, index) => (
                    <label>
                      <input type="checkbox" checked={labelWriteAuthorizationChecks()[index()]} onChange={(event) => toggleLabelWriteAuthorizationCheck(index(), event.currentTarget.checked)} />
                      <span>{label}</span>
                    </label>
                  )}</For>
                </div>
                <button type="button" class="public-admin-decision-submit" disabled={labelWriteAuthorizationDisabled()} onClick={() => void reviewFormalLabelWriteAuthorization()}>
                  写入不可覆盖的独立授权复核
                </button>
                <p class="public-admin-anchor-boundary">额度只在复核提交后 24 小时内有效且最多消费一次。实际写入必须在下一块单独执行；批准不会自动写标签或进入训练。</p>
              </article>
            )}</Show>
            <For each={authorizationRegistry().items.filter((item) => item.latest_review)}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.asset_symbol} · {item.latest_review?.review_id}</strong>
                  <span>{item.authorization_consumed_by_formal_label_writer ? "额度已消费" : item.authorization_unexpired ? "一次性额度有效" : "无有效额度"}</span>
                </header>
                <p>复核人 {item.latest_review?.reviewer_id} · 有效至 {item.latest_review?.authorization_valid_until} · {item.latest_review?.rationale}</p>
                <p class="public-admin-anchor-boundary">该记录仅是一次性额度。是否已消费和是否写入，请以下方第 21 阶段不可变 claim / label 记录为准；训练、奖励、影子、订单、券商和交易仍未授权。</p>
              </article>
            )}</For>
          </section>
        )}</Show>
        <Show when={formalLabelWrites()}>{(writeRegistry) => (
          <section class="public-admin-reward-governance" aria-label="正式原始结果标签一次性写入">
            <header>
              <strong>第 21 阶段 · 正式原始结果标签一次性写入</strong>
              <span>{writeRegistry().write_status}</span>
            </header>
            <p>{writeRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>可写授权</span><strong>{writeRegistry().eligible_authorization_count}</strong></div>
              <div><span>不可变 claim</span><strong>{writeRegistry().claim_count}</strong></div>
              <div><span>正式标签</span><strong>{writeRegistry().formal_label_count}</strong></div>
              <div><span>失败 / 中断</span><strong>{writeRegistry().failed_write_count + writeRegistry().incomplete_fail_closed_claim_count}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              这是不可逆操作：系统先写 claim，立即消费一次性额度，再以 create-new 写标签；失败或中断也不能重试同一授权。正式标签仍不是训练样本，也不产生奖励或交易权限。
            </p>
            <label>
              <span>当前未过期的一次性授权</span>
              <select value={selectedFormalLabelAuthorizationReviewId()} onChange={(event) => setSelectedFormalLabelAuthorizationReviewId(event.currentTarget.value)}>
                <option value="">请选择要消费的授权</option>
                <For each={writeRegistry().eligible_authorizations}>{(item) => (
                  <option value={item.authorization_review_id}>
                    {item.asset_symbol} / {item.benchmark_symbol} · {item.authorization_review_id.slice(0, 12)}…
                  </option>
                )}</For>
              </select>
            </label>
            <Show when={selectedFormalLabelWriteAuthorization()}>{(selected) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{selected().asset_symbol} · 原始结果标签</strong>
                  <span>有效至 {selected().authorization_valid_until}</span>
                </header>
                <p>authorization {selected().authorization_review_sha256.slice(0, 12)}… · validation {selected().materialization_validation_sha256.slice(0, 12)}… · contract {selected().label_contract_sha256.slice(0, 12)}…</p>
                <button type="button" class="public-admin-decision-submit" disabled={formalLabelWriteDisabled()} onClick={() => void writeFormalRawOutcomeLabelOnce()}>
                  消费一次性授权并 create-once 写入
                </button>
              </article>
            )}</Show>
            <For each={writeRegistry().writes}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.label?.payload.asset_symbol ?? item.claim.materialization_validation_id} · claim {item.claim.claim_id}</strong>
                  <span>{item.write_status}</span>
                </header>
                <p>授权 {item.claim.authorization_review_id} · claim SHA {item.claim.claim_sha256.slice(0, 12)}… · 目标标签 {item.claim.target_label_id}</p>
                <Show when={item.label}>{(label) => (
                  <>
                    <p>label SHA {label().label_sha256} · 写入人 {label().written_by} · {label().created_at}</p>
                    <p>原始窗口：{label().payload.raw_validated_metrics.map((metric) => `${metric.horizon_market_sessions} 日`).join(" / ")}；已知局限：{label().payload.known_limitations}</p>
                  </>
                )}</Show>
                <Show when={item.failure}>{(failure) => <p>失败：{failure().error_message}</p>}</Show>
                <p class="public-admin-anchor-boundary">训练准入校验：未完成；离线训练候选：否；方向、评级、动作、仓位、奖励、影子、订单、券商和交易：全部关闭。</p>
              </article>
            )}</For>
          </section>
        )}</Show>
        <Show when={formalLabelValidations()}>{(validationRegistry) => (
          <section class="public-admin-reward-governance" aria-label="正式原始结果标签独立校验与离线数据集候选准入">
            <header>
              <strong>第 22 阶段 · 正式标签独立校验与离线数据集候选准入</strong>
              <span>{validationRegistry().validation_status}</span>
            </header>
            <p>{validationRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>待独立校验</span><strong>{validationRegistry().validation_eligible_count}</strong></div>
              <div><span>校验记录</span><strong>{validationRegistry().validation_count}</strong></div>
              <div><span>候选准入</span><strong>{validationRegistry().admitted_candidate_count}</strong></div>
              <div><span>失败关闭</span><strong>{validationRegistry().failed_validation_count}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              校验器不复用第 21 阶段 writer，并排除写入人及完整上游生产/复核参与者。它重新核对 canonical label/claim、固定八字段、来源、局限和 20 / 60 / 250 日每一位指标。候选≠训练：不复制训练存储、不建数据集版本，也不产生奖励或交易权限。
            </p>
            <label>
              <span>待校验正式标签</span>
              <select value={selectedFormalLabelId()} onChange={(event) => setSelectedFormalLabelId(event.currentTarget.value)}>
                <option value="">请选择正式标签</option>
                <For each={validationRegistry().items.filter((item) => item.validation_eligible)}>{(item) => (
                  <option value={item.formal_label.label.label_id}>
                    {item.formal_label.label.payload.asset_symbol} / {item.formal_label.label.payload.benchmark_symbol} · {item.formal_label.label.label_id.slice(0, 12)}…
                  </option>
                )}</For>
              </select>
            </label>
            <Show when={selectedFormalLabelForValidation()}>{(selected) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{selected().formal_label.label.payload.asset_symbol} · 独立准入校验</strong>
                  <span>原始窗口 {selected().formal_label.label.payload.raw_validated_metrics.map((metric) => `${metric.horizon_market_sessions} 日`).join(" / ")}</span>
                </header>
                <p>label {selected().formal_label.label.label_sha256.slice(0, 12)}… · claim {selected().formal_label.claim.claim_sha256.slice(0, 12)}… · contract {selected().formal_label.claim.label_contract_sha256.slice(0, 12)}…</p>
                <button type="button" class="public-admin-decision-submit" disabled={formalLabelValidationDisabled()} onClick={() => void validateFormalRawOutcomeLabel()}>
                  运行独立校验并写入不可变准入记录
                </button>
              </article>
            )}</Show>
            <For each={validationRegistry().items.filter((item) => item.validation)}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.formal_label.label.payload.asset_symbol} · validation {item.validation?.validation_id}</strong>
                  <span>{item.validation?.admitted_to_offline_training_dataset_candidate ? "已准入离线数据集候选" : "独立校验失败"}</span>
                </header>
                <p>校验人 {item.validation?.validated_by} · 写入人 {item.validation?.formal_label_written_by} · {item.validation?.validated_at}</p>
                <Show when={(item.validation?.mismatch_reasons.length ?? 0) > 0}>
                  <p>不一致：{item.validation?.mismatch_reasons.join("；")}</p>
                </Show>
                <p class="public-admin-anchor-boundary">训练存储复制：否；训练运行：未授权；训练目标与奖励：未写入；影子、订单、券商和交易：全部关闭。</p>
              </article>
            )}</For>
          </section>
        )}</Show>
        <Show when={offlineDatasets()}>{(datasetRegistry) => (
          <section class="public-admin-reward-governance" aria-label="版本化离线历史结果数据集装配">
            <header>
              <strong>第 23 阶段 · 版本化离线历史结果数据集装配</strong>
              <span>{datasetRegistry().assembly_status}</span>
            </header>
            <p>{datasetRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>当前候选</span><strong>{datasetRegistry().current_candidate_count}</strong></div>
              <div><span>不可变版本</span><strong>{datasetRegistry().dataset_count}</strong></div>
              <div><span>绑定当前候选集</span><strong>{datasetRegistry().current_binding_dataset_count}</strong></div>
              <div><span>最新条目</span><strong>{datasetRegistry().latest_dataset?.entry_count ?? 0}</strong></div>
            </div>
            <p class="public-admin-anchor-boundary">
              只允许一次装配当前完整通过集，并以内容哈希冻结；后续版本必须逐条保留旧版本并只追加新候选。数据集≠训练：此处不做特征拼接、语义目标、数据分割、奖励或训练运行。
            </p>
            <Show when={datasetRegistry().assembly_available}>
              <div class="public-admin-decision-checks">
                <For each={OFFLINE_DATASET_ASSEMBLY_CHECKS}>{(label, index) => (
                  <label>
                    <input type="checkbox" checked={offlineDatasetAssemblyChecks()[index()]} onChange={(event) => toggleOfflineDatasetAssemblyCheck(index(), event.currentTarget.checked)} />
                    <span>{label}</span>
                  </label>
                )}</For>
              </div>
              <button type="button" class="public-admin-decision-submit" disabled={offlineDatasetAssemblyDisabled()} onClick={() => void assembleOfflineHistoricalOutcomeDataset()}>
                装配当前完整候选集并写入不可变数据集版本
              </button>
            </Show>
            <For each={datasetRegistry().datasets}>{(dataset) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{dataset.dataset_version} · {dataset.entry_count} 条原始结果</strong>
                  <span>新增 {dataset.added_entry_count} · {dataset.distinct_symbol_count} 个标的</span>
                </header>
                <p>dataset {dataset.dataset_id} · parent {dataset.parent_dataset_id ?? "初始版本"}</p>
                <p>content SHA {dataset.dataset_content_sha256.slice(0, 16)}… · manifest {dataset.manifest_sha256.slice(0, 16)}… · candidate set {dataset.candidate_set_sha256.slice(0, 16)}…</p>
                <p>点时范围 {dataset.earliest_decision_available_at} — {dataset.latest_decision_available_at} · 装配人 {dataset.assembled_by} · {dataset.assembled_at}</p>
                <p>标的：{dataset.entries.map((entry) => entry.asset_symbol).join("、")}</p>
                <p class="public-admin-anchor-boundary">完整集冻结：是；单调只追加：是；点时血缘保留：是；训练分割：未分配；特征与目标：未生成；训练、奖励、影子、订单、券商和交易：全部关闭。</p>
              </article>
            )}</For>
          </section>
        )}</Show>
        <Show when={offlineDatasetGovernance()}>{(governanceRegistry) => (
          <section class="public-admin-reward-governance" aria-label="离线历史结果数据集独立治理复核">
            <header>
              <strong>第 24 阶段 · 离线数据集独立治理复核</strong>
              <span>{governanceRegistry().governance_status}</span>
            </header>
            <p>{governanceRegistry().scope}</p>
            <div class="public-admin-decision-metrics">
              <div><span>可复核</span><strong>{governanceRegistry().review_eligible_count}</strong></div>
              <div><span>已复核</span><strong>{governanceRegistry().reviewed_count}</strong></div>
              <div><span>历史批准</span><strong>{governanceRegistry().approved_count}</strong></div>
              <div><span>当前绑定批准</span><strong>{governanceRegistry().current_binding_approved_count}</strong></div>
            </div>
            <article class="public-admin-reward-governance">
              <header><strong>未来防泄漏切分规范</strong><span>此处不执行切分</span></header>
              <p>公司、历史事件与来源身份构成不可拆分连通分量；未来按稳定 SHA-256 确定性分配为 {governanceRegistry().split_policy.train_percent}% 训练、{governanceRegistry().split_policy.validation_percent}% 验证、{governanceRegistry().split_policy.sealed_holdout_percent}% 封存留出。</p>
              <p>严格保持时间顺序，并按最长结果窗口设置 {governanceRegistry().split_policy.purge_embargo_market_sessions} 个交易日 purge / embargo；封存留出标签对训练 worker 不可见。</p>
            </article>
            <article class="public-admin-reward-governance">
              <header><strong>未来点时特征连接规范</strong><span>此处不连接特征</span></header>
              <p>{governanceRegistry().feature_join_policy.availability_rule}；必须保留制品 SHA、来源、版本和 available_at。</p>
              <p>结果、正式标签、校验、准入、离线数据集、未来行情和切分字段均禁止进入特征。available_at 缺失或歧义时直接排除，不回填、不插值。</p>
            </article>
            <Show when={governanceRegistry().review_eligible_count > 0}>
              <label>
                <span>当前绑定数据集</span>
                <select value={selectedOfflineDatasetGovernanceId()} onChange={(event) => setSelectedOfflineDatasetGovernanceId(event.currentTarget.value)}>
                  <option value="">请选择待复核数据集</option>
                  <For each={governanceRegistry().items.filter((item) => item.review_eligible)}>{(item) => (
                    <option value={item.subject.dataset_id}>
                      {item.subject.dataset_version} · {item.subject.entry_count} 条 · {item.subject.distinct_symbol_count} 个标的
                    </option>
                  )}</For>
                </select>
              </label>
              <label>
                <span>复核结论</span>
                <select value={offlineDatasetGovernanceVerdict()} onChange={(event) => setOfflineDatasetGovernanceVerdict(event.currentTarget.value as HistoricalOutcomeOfflineDatasetGovernanceVerdict)}>
                  <option value="approved_for_split_and_point_in_time_feature_join_spec_registration">批准仅登记未来转换规范</option>
                  <option value="changes_requested">要求修订</option>
                  <option value="rejected">拒绝</option>
                </select>
              </label>
              <label>
                <span>复核理由</span>
                <textarea value={offlineDatasetGovernanceRationale()} onInput={(event) => setOfflineDatasetGovernanceRationale(event.currentTarget.value)} placeholder="说明完整性、防泄漏边界与结论依据" />
              </label>
              <label>
                <span>已知局限</span>
                <textarea value={offlineDatasetGovernanceLimitations()} onInput={(event) => setOfflineDatasetGovernanceLimitations(event.currentTarget.value)} placeholder="例如样本量、覆盖行业、来源同质性或未来转换仍待验证" />
              </label>
              <Show when={offlineDatasetGovernanceApprovalSelected()}>
                <div class="public-admin-decision-checks">
                  <For each={OFFLINE_DATASET_GOVERNANCE_CHECKS}>{(label, index) => (
                    <label>
                      <input type="checkbox" checked={offlineDatasetGovernanceChecks()[index()]} onChange={(event) => toggleOfflineDatasetGovernanceCheck(index(), event.currentTarget.checked)} />
                      <span>{label}</span>
                    </label>
                  )}</For>
                </div>
              </Show>
              <button type="button" class="public-admin-decision-submit" disabled={offlineDatasetGovernanceDisabled()} onClick={() => void reviewOfflineHistoricalOutcomeDatasetGovernance()}>
                写入不可变治理复核记录
              </button>
            </Show>
            <For each={governanceRegistry().items}>{(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.subject.dataset_version} · {item.subject.entry_count} 条</strong>
                  <span>{item.current_binding ? "当前绑定" : "历史版本"} · {item.future_transformation_spec_registration_eligible ? "仅可登记转换规范" : item.latest_review ? "已复核未批准" : "未复核"}</span>
                </header>
                <p>装配人 {item.subject.assembled_by} · 数据链参与者 {item.subject.complete_actor_ids.length} 人 · 治理复核参与者 {item.complete_review_actor_ids.length} 人 · 重建 {item.subject.distinct_reconstruction_count} · 快照 {item.subject.distinct_snapshot_count}</p>
                <Show when={item.latest_review}>{(review) => (
                  <>
                    <p>review {review().review_id} · 复核人 {review().reviewer_id} · {review().submitted_at}</p>
                    <p>{review().rationale}</p>
                    <p>局限：{review().known_limitations}</p>
                  </>
                )}</Show>
                <p class="public-admin-anchor-boundary">切分：未执行；特征连接：未执行；语义目标：未生成；训练、奖励、影子、订单、券商和交易：全部关闭。</p>
              </article>
            )}</For>
          </section>
        )}</Show>
        <PublicAdminHistoricalOutcomeTransformationSpecPanel />
        <PublicAdminHistoricalOutcomeTransformationSpecReviewPanel />
        <PublicAdminHistoricalOutcomeTransformationImplementationPanel />
        <PublicAdminHistoricalOutcomeTransformationImplementationReviewPanel />
        <PublicAdminHistoricalOutcomeTransformationIsolatedRunnerPanel />
        <PublicAdminHistoricalOutcomeTransformationFirstExecutionAuthorizationPanel />
        <PublicAdminHistoricalOutcomeTransformationExecutionAttemptPanel />
        <PublicAdminHistoricalOutcomeTransformationOutputValidationPanel />
        <PublicAdminHistoricalOutcomeTransformationCandidateAdmissionPanel />
        <PublicAdminHistoricalOutcomeTransformationOfficialArtifactMaterializationPanel />
        <PublicAdminHistoricalOutcomeTransformationOfficialArtifactOutputValidationPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetSpecPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetSpecReviewPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetImplementationPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetImplementationReviewPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetOutputValidationPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationPanel />
        <PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmissionPanel />
        <PublicAdminHistoricalOutcomeTrainingExperimentRegistrationPanel />
        <PublicAdminHistoricalOutcomeTrainingExperimentRegistrationReviewPanel />
        <PublicAdminHistoricalOutcomeTrainingImplementationPanel />
        <PublicAdminHistoricalOutcomeTrainingImplementationReviewPanel />
        <PublicAdminHistoricalOutcomeTrainingIsolatedRunnerPanel />
        <PublicAdminHistoricalOutcomeTrainingFirstExecutionAuthorizationPanel />
        <PublicAdminHistoricalOutcomeTrainingExecutionAttemptPanel />
        <PublicAdminHistoricalOutcomeTrainingOutputValidationPanel />
        <PublicAdminHistoricalOutcomeValidationEvaluationImplementationPanel />
        <PublicAdminHistoricalOutcomeValidationEvaluationImplementationReviewPanel />
        <PublicAdminHistoricalOutcomeValidationEvaluationIsolatedRunnerPanel />
        <PublicAdminHistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationPanel />
        <PublicAdminHistoricalOutcomeValidationEvaluationExecutionAttemptPanel />
        <PublicAdminHistoricalOutcomeValidationEvaluationOutputValidationPanel />
        <PublicAdminHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionPanel />
        <PublicAdminHistoricalOutcomeSealedHoldoutEvaluationProtocolReviewPanel />
        <PublicAdminHistoricalOutcomeSealedHoldoutEvaluationImplementationPanel />
        <PublicAdminHistoricalOutcomeSealedHoldoutEvaluationImplementationReviewPanel />
        <PublicAdminHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerPanel />
        <PublicAdminHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationPanel />
        <PublicAdminHistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptPanel />
        <PublicAdminHistoricalOutcomeSealedHoldoutEvaluationOutputValidationPanel />
        <PublicAdminHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationPanel />
        <PublicAdminControlledShadowExperimentDesignRegistrationPanel />
        <PublicAdminControlledShadowExperimentDesignRegistrationReviewPanel />
        <PublicAdminControlledShadowExperimentImplementationPanel />
        <PublicAdminControlledShadowExperimentImplementationReviewPanel />
        <PublicAdminControlledShadowExperimentIsolatedRunnerPanel />
        <PublicAdminControlledShadowExperimentFirstExecutionAuthorizationPanel />
        <PublicAdminControlledShadowExperimentExecutionAttemptPanel />
        <PublicAdminControlledShadowExperimentOutputValidationPanel />
        <PublicAdminControlledShadowForwardObservationProtocolRegistrationPanel />
        <PublicAdminControlledShadowForwardObservationProtocolRegistrationReviewPanel />
        <PublicAdminControlledShadowForwardObservationImplementationPanel />
        <PublicAdminControlledShadowForwardObservationImplementationReviewPanel />
        <PublicAdminControlledShadowForwardObservationIsolatedRunnerPanel />
        <PublicAdminControlledShadowForwardObservationFirstExecutionAuthorizationPanel />
        <PublicAdminControlledShadowForwardObservationExecutionAttemptPanel />
        <PublicAdminControlledShadowForwardObservationOutputValidationPanel />
        <PublicAdminControlledShadowFirstNaturalForwardCycleAuthorizationPanel />
        <PublicAdminControlledShadowFirstNaturalForwardCycleClaimPanel />
        <PublicAdminControlledShadowMarketDataAdapterAuthorizationPanel />
        <PublicAdminControlledShadowMarketDataReceiptAttemptPanel />
        <PublicAdminControlledShadowMarketDataReceiptValidationPanel />
        <PublicAdminControlledShadowMarketDataParserSpecificationPanel />
        <PublicAdminControlledShadowMarketDataParserSpecificationReviewPanel />
        <PublicAdminControlledShadowMarketDataParserImplementationPanel />
        <PublicAdminControlledShadowMarketDataParserImplementationReviewPanel />
        <PublicAdminControlledShadowMarketDataParserIsolatedRunnerPanel />
        <PublicAdminControlledShadowMarketDataParserFirstExecutionAuthorizationPanel />
        <PublicAdminControlledShadowMarketDataParserExecutionAttemptClaimPanel />
        <PublicAdminControlledShadowMarketDataParserExecutionAttemptPanel />
        <PublicAdminControlledShadowMarketDataParserOutputValidationPanel />
        <PublicAdminControlledShadowObservationInputAdmissionPanel />
        <PublicAdminControlledShadowObservationMaterializationSpecificationPanel />
        <PublicAdminControlledShadowObservationMaterializationSpecificationReviewPanel />
        <PublicAdminControlledShadowObservationMaterializationImplementationPanel />
        <PublicAdminControlledShadowObservationMaterializationImplementationReviewPanel />
        <PublicAdminControlledShadowObservationMaterializationIsolatedRunnerPanel />
        <PublicAdminControlledShadowObservationMaterializationFirstExecutionAuthorizationPanel />
        <PublicAdminControlledShadowObservationMaterializationExecutionAttemptClaimPanel />
        <PublicAdminControlledShadowObservationMaterializationExecutionAttemptPanel />
        <PublicAdminControlledShadowObservationMaterializationOutputValidationPanel />
        <PublicAdminControlledShadowObservationEvidenceAdmissionPanel />
        <PublicAdminControlledShadowObservationLedgerTransitionSpecificationPanel />
        <PublicAdminControlledShadowObservationLedgerTransitionSpecificationReviewPanel />
        <PublicAdminControlledShadowObservationLedgerTransitionImplementationPanel />
        <PublicAdminControlledShadowObservationLedgerTransitionImplementationReviewPanel />
        <PublicAdminControlledShadowObservationLedgerTransitionIsolatedRunnerPanel />
        <PublicAdminControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationPanel />
        <PublicAdminControlledShadowObservationLedgerTransitionExecutionAttemptClaimPanel />
        <PublicAdminControlledShadowObservationLedgerTransitionExecutionAttemptPanel />
        <PublicAdminControlledShadowObservationLedgerTransitionOutputValidationPanel />
        <PublicAdminControlledShadowObservationLedgerTransitionCandidateAdmissionPanel />
        <PublicAdminOpeningPortfolioSnapshotGovernanceSpecificationPanel />
        <PublicAdminOpeningPortfolioSnapshotGovernanceSpecificationReviewPanel />
        <PublicAdminOpeningPortfolioSourceArtifactReceiptImplementationPanel />
        <PublicAdminOpeningPortfolioSourceArtifactReceiptImplementationReviewPanel />
        <PublicAdminOpeningPortfolioSourceArtifactReceiptIsolatedReceiverPanel />
        <PublicAdminOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationPanel />
        <PublicAdminOpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimPanel />
        <PublicAdminOpeningPortfolioSourceArtifactReceiptExecutionAttemptPanel />
        <PublicAdminOpeningPortfolioSourceArtifactReceiptValidationPanel />
        <PublicAdminOpeningPortfolioSnapshotMaterializationImplementationPanel />
        <PublicAdminOpeningPortfolioSnapshotMaterializationImplementationReviewPanel />
        <PublicAdminOpeningPortfolioSnapshotMaterializationIsolatedMaterializerPanel />
        <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
      </section>
    )}</Show>
  );
}
