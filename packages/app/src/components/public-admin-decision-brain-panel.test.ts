import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";
import {
  buildDecisionReviewRequest,
  causalObservationCanBeAccepted,
  causalSourceVerificationPrompt,
  decisionReviewDraftIsValid,
  type DecisionReviewDraft,
} from "./public-admin-decision-brain-panel";
import type {
  InvestmentCausalObservation,
  InvestmentDecisionTrainingSample,
} from "@/lib/types";

const panelSource = readFileSync(
  new URL("./public-admin-decision-brain-panel.tsx", import.meta.url),
  "utf8",
);

function sample(): InvestmentDecisionTrainingSample {
  return {
    schema_version: "hone-investment-training-sample-v1",
    sample_id: "SNDK-1",
    observed_at: "2026-08-12T12:00:00Z",
    selected_action: "increase_candidate",
    human_review: {
      status: "corrected",
      review_id: "review-1",
      thesis_verdict: "weakened",
      error_attributions: [],
    },
    outcomes: [],
    reward: { status: "unconfigured" },
    state: {
      symbol: "SNDK",
      company_name: "SanDisk",
      theme: "AI 存储",
      source_rating_score: 82,
      source_rating_light: "green",
      data_status: "live",
      evidence_coverage: 4,
      decision: {
        zone: "opportunity",
        action: "increase_candidate",
        confidence: "medium",
        rationale: [],
        falsifiers: [],
        next_checks: [],
      },
      evidence: [],
    },
  };
}

const corrected: DecisionReviewDraft = {
  mode: "corrected",
  verdict: "weakened",
  note: " 价值捕获被高估 ",
  correctedAction: "maintain",
  errorKind: "company_value_capture",
  errorSeverity: "material",
  errorExplanation: " 企业级份额证据不足 ",
};

describe("administrator decision brain review", () => {
  it("keeps rating financial review separate from valuation-use review", () => {
    expect(panelSource).toContain("PublicAdminFinancialEvidenceReview");
    expect(panelSource).toContain("PublicAdminValuationInputReview");
  });

  it("keeps valid replay visible while explaining quarantined history", () => {
    expect(panelSource).toContain("quarantined_sample_count");
    expect(panelSource).toContain("这些记录不会进入训练或评测");
    expect(panelSource).toContain("查看隔离原因");
    expect(panelSource).toContain("现有历史样本均未通过当前完整性校验");
  });

  it("shows a latest-state first-principles map without turning coverage into a ranking", () => {
    expect(panelSource).toContain("第一性原理产业假设地图");
    expect(panelSource).toContain("first_principles_hypothesis_map");
    expect(panelSource).toContain("可追溯证据");
    expect(panelSource).toContain("严格量化证据");
    expect(panelSource).toContain("需求 {model.demand_traceable_company_count}/{model.company_count}");
    expect(panelSource).toContain("需求 {model.demand_measured_company_count}/{model.company_count}");
    expect(panelSource).toContain("结构化来源 {model.evidence_pathway.structured_source_claim_count}");
    expect(panelSource).not.toContain("demand_observed_company_count");
    expect(panelSource).toContain("机会排名</strong> 关闭");
    expect(panelSource).toContain("动作授权</strong> 未授权");
    expect(panelSource).toContain("历史重复样本不增加权重");
  });

  it("shows the synthetic Hari conformance benchmark without implying investment authority", () => {
    expect(panelSource).toContain("Hari 已确认逻辑情景基准");
    expect(panelSource).toContain("hari_logic_scenario_benchmark.passed_scenario_count");
    expect(panelSource).toContain("expected_company_increase_authorized");
    expect(panelSource).toContain("actual_blocking_logic_ids");
    expect(panelSource).toContain("训练标签</strong> 未生成");
    expect(panelSource).toContain("决策/组合/交易</strong> 均未授权");
    expect(panelSource).toContain("只代表实现与已确认逻辑一致，不代表策略有效、能够赚钱或可以操盘");
  });

  it("shows one fail-closed empirical promotion chain instead of separate optimistic counters", () => {
    expect(panelSource).toContain("实证验证晋级清单");
    expect(panelSource).toContain("empirical_validation_readiness");
    expect(panelSource).toContain("① 人工因果数据集");
    expect(panelSource).toContain("② 历史点时基准");
    expect(panelSource).toContain("③ 未来结果协议");
    expect(panelSource).toContain("blocking_reasons");
    expect(panelSource).toContain("训练</strong> 未授权");
    expect(panelSource).toContain("影子/交易</strong> 均未授权");
    expect(panelSource).toContain("不会自动运行训练、生成奖励、建立影子持仓或下单");
  });

  it("shows Stage 98 parser implementation review without implying runtime capability", () => {
    expect(panelSource).toContain("98 行情解析器实现责任链外独立复核");
    expect(panelSource).toContain("historical_outcome_market_data_parser_implementation_review_eligible_count");
    expect(panelSource).toContain("historical_outcome_market_data_parser_implementation_independently_approved_count");
    expect(panelSource).toContain("historical_outcome_future_isolated_market_data_parser_runner_specification_registration_eligible_count");
    expect(panelSource).toContain("historical_outcome_market_data_parser_isolated_runner_registration_eligible_count");
    expect(panelSource).toContain("historical_outcome_market_data_parser_first_execution_authorization_review_eligible_count");
    expect(panelSource).toContain("99 行情解析器隔离 runner 规格登记");
    expect(panelSource).toContain("仍无 runner、runtime、原始载荷读取、解析输出、观察或交易权限");
  });

  it("shows Stage 100 as server-rehashed authorization without implying execution", () => {
    expect(panelSource).toContain("100 行情解析器首次执行授权独立复核");
    expect(panelSource).toContain("historical_outcome_market_data_parser_reproduced_artifact_pending_runner_count");
    expect(panelSource).toContain("historical_outcome_market_data_parser_reproduced_artifact_verified_runner_count");
    expect(panelSource).toContain("historical_outcome_market_data_parser_future_claim_first_attempt_eligible_count");
    expect(panelSource).toContain("手填相同摘要不能通过");
    expect(panelSource).toContain("即使批准也没有入口、runtime、挂载、载荷读取、parser 执行、解析行或交易权限");
  });

  it("shows Stage 101 as a permanent claim-first consumption gate", () => {
    expect(panelSource).toContain("101 行情解析器单次尝试 claim-first 声明");
    expect(panelSource).toContain("historical_outcome_market_data_parser_execution_attempt_claim_count");
    expect(panelSource).toContain("historical_outcome_market_data_parser_execution_attempt_authorization_consumed_count");
    expect(panelSource).toContain("只冻结既有元数据与摘要，不读取载荷、不执行 parser、不生成解析行");
  });

  it("shows Stage 103 as an independent full-output validation gate", () => {
    expect(panelSource).toContain("103 行情解析器输出责任链外独立校验");
    expect(panelSource).toContain("historical_outcome_market_data_parser_output_validation_eligible_count");
    expect(panelSource).toContain("historical_outcome_market_data_parser_output_independently_validated_count");
    expect(panelSource).toContain("historical_outcome_market_data_parser_future_observation_input_admission_review_eligible_count");
    expect(panelSource).toContain("historical_outcome_observation_input_admission_candidate_count");
    expect(panelSource).toContain("historical_outcome_future_observation_materialization_specification_registration_eligible_count");
    expect(panelSource).toContain("不调用 Stage 102 解析助手的第二套实现");
    expect(panelSource).toContain("source_available_at 仍未验证");
  });

  it("turns traceable-but-unmeasured drivers into a bounded measurement admission backlog", () => {
    expect(panelSource).toContain("量化准入待办");
    expect(panelSource).toContain("measurement_backlog.ready_for_review_count");
    expect(panelSource).toContain("target_operating_kpi_ids");
    expect(panelSource).toContain("MEASUREMENT_BACKLOG_STATUS_LABELS");
    expect(panelSource).toContain("可直接复核");
    expect(panelSource).toContain("文字待指标化");
    expect(panelSource).toContain("待补经营指标");
    expect(panelSource).toContain("只有老王本人确认关系成立且明确支持或证伪后，才晋级为部分量化");
    expect(panelSource).toContain("items.slice(0, 12)");
  });

  it("shows market-positioning context as non-scored background", () => {
    expect(panelSource).toContain("期权仓位结构");
    expect(panelSource).toContain("新闻发布活跃度");
    expect(panelSource).toContain("机构 13F 聚合");
    expect(panelSource).toContain("分析师建议与目标价");
    expect(panelSource).toContain("目标价低/共识/高");
    expect(panelSource.match(/背景证据 · 不计分/g)?.length).toBeGreaterThanOrEqual(5);
    expect(panelSource).toContain("查看 Nasdaq 原始期权链");
    expect(panelSource).toContain("查看 Nasdaq 聚合发布流");
    expect(panelSource).toContain("查看 Nasdaq 13F 聚合表");
  });

  it("shows which confirmed Hari rules gated the frozen decision", () => {
    expect(panelSource).toContain("Hari 已确认逻辑门禁");
    expect(panelSource).toContain("confirmed_logic_ids");
    expect(panelSource).toContain("increase_candidate_authorized");
    expect(panelSource).toContain("本轮非增加候选");
    expect(panelSource).toContain("增加暴露被阻断");
    expect(panelSource).toContain("组合层审查");
  });

  it("separates bounded source-review and old-Wang batches", () => {
    expect(panelSource).toContain('createSignal<"source_batch" | "old_wang_batch" | "full_queue">("source_batch")');
    expect(panelSource).toContain('const isBatch = selection !== "full_queue"');
    expect(panelSource).toContain("维护者来源核验");
    expect(panelSource).toContain("维护者核来源 · 5 条");
    expect(panelSource).toContain("老王待回答 · 5 条");
    expect(panelSource).toContain("维护者待核");
    expect(panelSource).toContain("已核待老王");
    expect(panelSource).toContain("来源已排除");
    expect(panelSource).toContain("old_wang_submission_authorized");
    expect(panelSource).toContain("当前账号只读");
    expect(panelSource).toContain("selection_scope");
  });

  it("reuses only immutable source evidence across daily snapshots and fails closed on conflicts", () => {
    expect(panelSource).toContain("source_review_reused_across_snapshots");
    expect(panelSource).toContain("source_review_origin_sample_id");
    expect(panelSource).toContain("source_review_conflict");
    expect(panelSource).toContain("跨快照沿用");
    expect(panelSource).toContain("跨快照冲突");
    expect(panelSource).toContain("来源核验沿用自同一冻结证据的历史快照");
    expect(panelSource).toContain("同一冻结证据存在相互冲突的跨快照来源核验");
    expect(panelSource).toContain("openReviewQueueItem");
  });

  it("saves a causal label independently from the company thesis", () => {
    expect(panelSource).toContain("reviewInvestmentCausalEvidence");
    expect(panelSource).toContain("确认并保存");
    expect(panelSource).toContain("整份公司判断与行动状态没有改变");
    expect(panelSource).toContain("expected_review_id: value.reviewId");
  });

  it("separates maintainer source verification from old Wang causal confirmation", () => {
    expect(panelSource).toContain("核验来源");
    expect(panelSource).toContain("老王回答");
    expect(panelSource).toContain("老王单问蒸馏复核");
    expect(panelSource).toContain("打开原始来源后，这条数值、期间、单位和上下文是否一致");
    expect(panelSource).toContain("独立保存来源问题");
    expect(panelSource).toContain("reviewInvestmentCausalSource");
    expect(panelSource).toContain("尚未生成任何因果或训练标签");
    expect(panelSource).toContain("这条证据在当时为什么能或不能改变你对");
    expect(panelSource).toContain("这条判断在什么条件下才适用");
    expect(panelSource).toContain("未来出现什么可观察事实时");
    expect(panelSource).toContain("请选择，不默认确认");
    expect(panelSource).toContain("old_wang_confirmed");
    expect(panelSource).not.toContain('<option value="source_checked_not_speaker_confirmed">');
    expect(panelSource).toContain('source_verification: value.sourceVerification');
    expect(panelSource).toContain("expected_source_review_id: value.sourceReviewId");
    expect(panelSource).toContain("old_wang_confirmation_attested: value.oldWangConfirmationAttested");
    expect(panelSource).toContain("不是服务器配置的老王审阅账号");
    expect(panelSource).toContain("verbatim_judgment: value.verbatimJudgment.trim()");
    expect(panelSource).toContain("applicability_boundary: value.applicabilityBoundary.trim()");
    expect(panelSource).toContain("falsifier: value.falsifier.trim()");
  });

  it("asks source questions that match numeric and qualitative evidence", () => {
    const numeric: InvestmentCausalObservation = {
      observation_id: "numeric-source-claim",
      relationship: "structured_source_claim",
      label: "收入",
      value: "收入 100 USD millions",
      as_of: "2026-08-12",
      source: "SEC",
      source_url: "https://www.sec.gov/filing",
      source_tier: "regulatory_primary",
      policy_status: "training_only_pending_human_review",
      claim: {
        claim_kind: "reported_fact",
        metric_id: "revenue",
        metric_basis: "US-GAAP:Revenue",
        period: "FY2026 Q2",
        numeric_value: 100,
        unit: "USD_millions",
        source_event_id: "filing-q2",
        source_document: "sec_filing",
        source_locator: "XBRL Revenue",
        quote_excerpt: "Revenue was 100 USD millions",
        disposition: "active",
        lifecycle_status: "active",
        conflicting_claim_ids: [],
      },
    };
    expect(causalSourceVerificationPrompt(numeric)).toContain("数值、期间、单位");
    expect(causalSourceVerificationPrompt({
      ...numeric,
      claim: {
        ...numeric.claim!,
        numeric_value: null,
        unit: "",
        claim_kind: "management_guidance",
        speaker: "CFO",
      },
    })).toContain("原话的主体、时间和上下文");
  });

  it("keeps the causal training dataset offline and the holdout labels sealed", () => {
    expect(panelSource).toContain("离线因果数据集（尚未训练）");
    expect(panelSource).toContain("company_split_isolation_verified");
    expect(panelSource).toContain("source_group_split_isolation_verified");
    expect(panelSource).toContain("connected_component_count");
    expect(panelSource).toContain("shared_source_group_count");
    expect(panelSource).toContain("holdout_labels_withheld");
    expect(panelSource).toContain('training_authorized ? "已开启" : "关闭"');
    expect(panelSource).toContain("feature_scope");
    expect(panelSource).toContain("authorization_scope");
  });

  it("requires immutable governance before an offline experiment can be registered", () => {
    expect(panelSource).toContain("不可变数据集治理");
    expect(panelSource).toContain("reviewInvestmentCausalDatasetGovernance");
    expect(panelSource).toContain("dataset_fingerprint_sha256");
    expect(panelSource).toContain("company_split_isolation_confirmed: approval");
    expect(panelSource).toContain("source_group_split_isolation_confirmed: approval");
    expect(panelSource).toContain("holdout_seal_confirmed: approval");
    expect(panelSource).toContain("future_leakage_audit_confirmed: approval");
    expect(panelSource).toContain("批准登记离线实验");
    expect(panelSource).toContain("训练、RL 与交易仍关闭");
  });

  it("keeps shadow protocol approval separate from any ledger or execution", () => {
    expect(panelSource).toContain("影子协议冻结与审批");
    expect(panelSource).toContain("getInvestmentShadowProtocolGovernance");
    expect(panelSource).toContain("reviewInvestmentShadowProtocolGovernance");
    expect(panelSource).toContain("expected_reward_review_id");
    expect(panelSource).toContain("confirmed_requirement_ids");
    expect(panelSource).toContain("implementation_boundary_confirmed: approval");
    expect(panelSource).toContain("批准未来实现登记");
    expect(panelSource).toContain("影子账本关闭、组合未授权、券商未连接、交易未授权");
  });

  it("registers only an immutable shadow implementation specification", () => {
    expect(panelSource).toContain("影子实现规范注册表（未启动）");
    expect(panelSource).toContain("getInvestmentShadowImplementations");
    expect(panelSource).toContain("registerInvestmentShadowImplementation");
    expect(panelSource).toContain("deterministic_replay_specification");
    expect(panelSource).toContain("expected_shadow_review_id");
    expect(panelSource).toContain("expected_reward_review_id");
    expect(panelSource).toContain("登记规范（不启动）");
    expect(panelSource).toContain("账本关闭");
    expect(panelSource).toContain("订单关闭");
    expect(panelSource).toContain("券商未连接");
  });

  it("registers Stage 74 as a frozen forward design without starting a shadow ledger", () => {
    expect(panelSource).toContain("74 受控影子实验设计登记");
    expect(panelSource).toContain("historical_outcome_future_independent_shadow_design_review_eligible_count");
    const registrationSource = readFileSync(
      new URL("./public-admin-controlled-shadow-experiment-design-registration-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(registrationSource).toContain("getControlledShadowExperimentDesignRegistrations");
    expect(registrationSource).toContain("registerControlledShadowExperimentDesign");
    expect(registrationSource).toContain("单股 5%、主题 20%、总仓 60%、现金至少 40%");
    expect(registrationSource).toContain("至少观察 252 个交易日");
    expect(registrationSource).toContain("下一步只能进行独立设计复核");
    expect(registrationSource).toContain("不创建影子持仓、订单、券商访问或交易");
  });

  it("independently reviews Stage 74 before any zero-capability shadow implementation registration", () => {
    expect(panelSource).toContain("75 受控影子实验设计独立复核");
    expect(panelSource).toContain("historical_outcome_future_zero_capability_shadow_implementation_registration_eligible_count");
    const reviewSource = readFileSync(
      new URL("./public-admin-controlled-shadow-experiment-design-registration-review-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(reviewSource).toContain("getControlledShadowExperimentDesignRegistrationReviews");
    expect(reviewSource).toContain("reviewControlledShadowExperimentDesignRegistration");
    expect(reviewSource).toContain("已用独立实现复算登记和设计指纹");
    expect(reviewSource).toContain("幸存者偏差、退市和前视泄漏");
    expect(reviewSource).toContain("批准只开放未来零能力影子实现规格登记");
    expect(reviewSource).toContain("不写模型/指标库，不训练、不奖励、不建仓、不下单、不接券商或交易");
    expect(reviewSource).toContain("影子实现、运行、账本、持仓、订单、券商与交易权限仍全部关闭");
  });

  it("registers Stage 76 only as a zero-capability deterministic specification", () => {
    expect(panelSource).toContain("76 零能力影子实现规格登记");
    expect(panelSource).toContain("historical_outcome_controlled_shadow_experiment_implementation_independent_review_eligible_count");
    const implementationSource = readFileSync(
      new URL("./public-admin-controlled-shadow-experiment-implementation-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(implementationSource).toContain("getControlledShadowExperimentImplementations");
    expect(implementationSource).toContain("registerControlledShadowExperimentImplementation");
    expect(implementationSource).toContain("本次只登记零能力规格，不声称存在可执行工件");
    expect(implementationSource).toContain("无入口、runtime、环境继承、密钥、网络、工具、子进程或生产读写");
    expect(implementationSource).toContain("不运行影子盘，不建账本/持仓/订单，不接券商或交易");
    expect(implementationSource).toContain("规格，不是程序");
  });

  it("independently reviews Stage 76 before any isolated shadow runner specification", () => {
    expect(panelSource).toContain("77 零能力影子实现独立复核");
    expect(panelSource).toContain("historical_outcome_future_isolated_shadow_runner_specification_registration_eligible_count");
    const reviewSource = readFileSync(
      new URL("./public-admin-controlled-shadow-experiment-implementation-review-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(reviewSource).toContain("getControlledShadowExperimentImplementationReviews");
    expect(reviewSource).toContain("reviewControlledShadowExperimentImplementation");
    expect(reviewSource).toContain("五层指纹重算");
    expect(reviewSource).toContain("复核人独立于 Stage 76 登记人和全部上游角色");
    expect(reviewSource).toContain("批准只开放未来隔离影子 runner 规格登记");
    expect(reviewSource).toContain("独立复核，不是运行授权");
    expect(reviewSource).toContain("运行、账本、持仓、订单、券商和交易权限仍全部关闭");
  });

  it("binds the Stage 78 executable artifact without opening an execution entrypoint", () => {
    expect(panelSource).toContain("78 隔离影子 runner 规格登记");
    expect(panelSource).toContain("historical_outcome_controlled_shadow_experiment_first_execution_authorization_review_eligible_count");
    const runnerSource = readFileSync(
      new URL("./public-admin-controlled-shadow-experiment-isolated-runner-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(runnerSource).toContain("getControlledShadowExperimentIsolatedRunners");
    expect(runnerSource).toContain("registerControlledShadowExperimentIsolatedRunner");
    expect(runnerSource).toContain("runner 可执行工件、代码版本、runtime、协议和序列化均已冻结");
    expect(runnerSource).toContain("当前没有 callable entrypoint 或输入挂载");
    expect(runnerSource).toContain("工件已绑定，入口仍关闭");
    expect(runnerSource).toContain("登记只开放独立首次影子执行授权复核");
    expect(runnerSource).toContain("不运行影子盘，不建账本/持仓/订单，不接券商或交易");
  });

  it("independently reviews Stage 79 without creating a shadow execution capability", () => {
    expect(panelSource).toContain("79 首次影子执行授权独立复核");
    expect(panelSource).toContain("historical_outcome_controlled_shadow_experiment_execution_attempt_eligible_count");
    const authorizationSource = readFileSync(
      new URL("./public-admin-controlled-shadow-experiment-first-execution-authorization-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(authorizationSource).toContain("getControlledShadowExperimentFirstExecutionAuthorizations");
    expect(authorizationSource).toContain("reviewControlledShadowExperimentFirstExecutionAuthorization");
    expect(authorizationSource).toContain("已独立复现 runner 可执行工件摘要");
    expect(authorizationSource).toContain("已确认代码版本可复现且精确工件可获得");
    expect(authorizationSource).toContain("当前没有 callable entrypoint 或输入挂载");
    expect(authorizationSource).toContain("批准只开放未来 Stage 80 claim-first 单次隔离影子执行尝试");
    expect(authorizationSource).toContain("本页不能 claim 或执行");
    expect(authorizationSource).toContain("影子运行、账本、持仓、模型/指标库、奖励、订单、券商和交易全部关闭");
  });

  it("runs Stage 80 claim-first without fabricating forward performance or execution authority", () => {
    expect(panelSource).toContain("80 claim-first 单次隔离影子初始化");
    expect(panelSource).toContain("historical_outcome_controlled_shadow_experiment_independent_output_validation_eligible_count");
    const executionSource = readFileSync(
      new URL("./public-admin-controlled-shadow-experiment-execution-attempt-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(executionSource).toContain("getControlledShadowExperimentExecutionAttempts");
    expect(executionSource).toContain("invokeControlledShadowExperimentOnce");
    expect(executionSource).toContain("先 create-once 写 claim");
    expect(executionSource).toContain("失败或中断都会永久消费本次授权");
    expect(executionSource).toContain("不能生成 21/63/126/252 日收益或晋级结论");
    expect(executionSource).toContain("必须进入 Stage 81 责任链外独立复算");
    expect(executionSource).toContain("不建账本、不写持仓/模型/指标");
    expect(executionSource).toContain("不生成订单、不接券商、不交易");
  });

  it("independently recomputes Stage 81 from the same content-addressed input", () => {
    expect(panelSource).toContain("81 初始影子观察独立第二实现复算");
    expect(panelSource).toContain("historical_outcome_controlled_shadow_experiment_output_validation_eligible_count");
    expect(panelSource).toContain("historical_outcome_future_forward_observation_protocol_registration_eligible_count");
    const validationSource = readFileSync(
      new URL("./public-admin-controlled-shadow-experiment-output-validation-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(validationSource).toContain("getControlledShadowExperimentOutputValidations");
    expect(validationSource).toContain("validateControlledShadowExperimentOutput");
    expect(validationSource).toContain("完整 Stage 51–80 责任链之外的新管理员");
    expect(validationSource).toContain("重新提交与 Stage 80 claim 完全相同的内容寻址点时输入");
    expect(validationSource).toContain("不复用 Stage 80 投影、预测或权重函数");
    expect(validationSource).toContain("0 个前向交易日");
    expect(validationSource).toContain("不建账本/持仓");
    expect(validationSource).toContain("不生成订单、不接券商、不交易");
  });

  it("registers Stage 82 as a natural-forward-only protocol without starting observation", () => {
    expect(panelSource).toContain("82 受控前向观察协议登记");
    expect(panelSource).toContain("historical_outcome_forward_observation_protocol_registered_count");
    const protocolSource = readFileSync(
      new URL("./public-admin-controlled-shadow-forward-observation-protocol-registration-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(protocolSource).toContain("getControlledShadowForwardObservationProtocolRegistrations");
    expect(protocolSource).toContain("registerControlledShadowForwardObservationProtocol");
    expect(protocolSource).toContain("自然到来的未来交易日，不回填");
    expect(protocolSource).toContain("证券与 SPY 同时点观察");
    expect(protocolSource).toContain("21/63/126/252 日检查点");
    expect(protocolSource).toContain("252 日、40 信号、12 公司、4 季度最低门槛");
    expect(protocolSource).toContain("不观察、不建账、不写持仓或绩效");
  });

  it("requires a chain-external Stage 83 review before observation implementation", () => {
    expect(panelSource).toContain("83 前向观察协议责任链外独立复核");
    expect(panelSource).toContain("historical_outcome_forward_observation_protocol_review_eligible_count");
    expect(panelSource).toContain("historical_outcome_future_zero_capability_forward_observation_implementation_registration_eligible_count");
    const reviewSource = readFileSync(
      new URL("./public-admin-controlled-shadow-forward-observation-protocol-registration-review-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(reviewSource).toContain("getControlledShadowForwardObservationProtocolRegistrationReviews");
    expect(reviewSource).toContain("reviewControlledShadowForwardObservationProtocolRegistration");
    expect(reviewSource).toContain("责任链外独立复核");
    expect(reviewSource).toContain("禁止回填");
    expect(reviewSource).toContain("单边 25bp");
    expect(reviewSource).toContain("252/40/12/4 最低门槛");
    expect(reviewSource).toContain("不观察、不建账、不写持仓/绩效/模型/指标");
  });

  it("registers Stage 84 only as a zero-capability forward-observation specification", () => {
    expect(panelSource).toContain("84 前向观察零能力实现规格登记");
    expect(panelSource).toContain("historical_outcome_forward_observation_implementation_independent_review_eligible_count");
    const implementationSource = readFileSync(
      new URL("./public-admin-controlled-shadow-forward-observation-implementation-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(implementationSource).toContain("getControlledShadowForwardObservationImplementations");
    expect(implementationSource).toContain("registerControlledShadowForwardObservationImplementation");
    expect(implementationSource).toContain("本次只登记零能力规格，不声称存在可执行工件");
    expect(implementationSource).toContain("无入口、可执行工件、runtime、挂载、适配器、环境继承、密钥、网络、工具或子进程");
    expect(implementationSource).toContain("规格，不是程序");
    expect(implementationSource).toContain("不运行影子盘，不建账本/持仓/订单，不接券商或交易");
  });

  it("keeps Stage 85 as a chain-external review gate before any isolated runner", () => {
    expect(panelSource).toContain("85 前向观察实现责任链外独立复核");
    expect(panelSource).toContain("historical_outcome_forward_observation_implementation_review_eligible_count");
    expect(panelSource).toContain("historical_outcome_forward_observation_implementation_independently_approved_count");
    expect(panelSource).toContain("historical_outcome_future_isolated_forward_observation_runner_specification_registration_eligible_count");
    expect(panelSource).toContain("批准也不创建 runner、观察、账本、持仓、绩效、订单、券商或交易能力");
    const reviewSource = readFileSync(
      new URL("./public-admin-controlled-shadow-forward-observation-implementation-review-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(reviewSource).toContain("getControlledShadowForwardObservationImplementationReviews");
    expect(reviewSource).toContain("reviewControlledShadowForwardObservationImplementation");
    expect(reviewSource).toContain("独立重算实现、合同、协议复核、协议登记、协议与设计六层指纹");
    expect(reviewSource).toContain("批准只开放未来隔离 runner 规格登记");
    expect(reviewSource).toContain("没有 runner、观察、账本、持仓、绩效、订单、券商或交易能力");
  });

  it("registers Stage 86 as an artifact-bound but non-executable forward-observation runner specification", () => {
    expect(panelSource).toContain("86 前向观察隔离 runner 规格登记");
    expect(panelSource).toContain("historical_outcome_forward_observation_isolated_runner_registration_eligible_count");
    expect(panelSource).toContain("historical_outcome_forward_observation_first_execution_authorization_review_eligible_count");
    const runnerSource = readFileSync(
      new URL("./public-admin-controlled-shadow-forward-observation-isolated-runner-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(runnerSource).toContain("getControlledShadowForwardObservationIsolatedRunners");
    expect(runnerSource).toContain("registerControlledShadowForwardObservationIsolatedRunner");
    expect(runnerSource).toContain("runner 工件 SHA-256");
    expect(runnerSource).toContain("工件复现程序");
    expect(runnerSource).toContain("工件身份已绑定，runtime 仍未实例化");
    expect(runnerSource).toContain("当前无 callable entrypoint、runtime、挂载、数据访问、观察、账本、持仓、绩效、订单、券商或交易权限");
    expect(runnerSource).toContain("下一步仅为责任链外首次前向观察执行授权复核");
  });

  it("reviews Stage 87 artifact reproduction before any one-shot forward attempt", () => {
    expect(panelSource).toContain("87 前向观察首次执行授权独立复核");
    expect(panelSource).toContain("historical_outcome_forward_observation_future_attempt_eligible_count");
    const authorizationSource = readFileSync(
      new URL("./public-admin-controlled-shadow-forward-observation-first-execution-authorization-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(authorizationSource).toContain("getControlledShadowForwardObservationFirstExecutionAuthorizations");
    expect(authorizationSource).toContain("reviewControlledShadowForwardObservationFirstExecutionAuthorization");
    expect(authorizationSource).toContain("独立复现 runner 工件 SHA-256");
    expect(authorizationSource).toContain("批准不等于执行");
    expect(authorizationSource).toContain("当前无 callable entrypoint、runtime、挂载、数据访问、观察、账本、持仓、绩效、订单、券商或交易权限");
  });

  it("keeps Stage 88 as a claim-first zero-market-data initialization gate", () => {
    expect(panelSource).toContain("88 前向观察 claim-first 单次初始化");
    expect(panelSource).toContain("historical_outcome_forward_observation_execution_attempt_eligible_count");
    expect(panelSource).toContain("historical_outcome_forward_observation_execution_independent_validation_eligible_count");
    const executionSource = readFileSync(
      new URL("./public-admin-controlled-shadow-forward-observation-execution-attempt-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(executionSource).toContain("getControlledShadowForwardObservationExecutionAttempts");
    expect(executionSource).toContain("invokeControlledShadowForwardObservationOnce");
    expect(executionSource).toContain("先落盘 claim，再复核二进制与清单");
    expect(executionSource).toContain("0 行行情、0 个自然前向交易日、0 个账本/持仓/绩效");
    expect(executionSource).toContain("未来 Stage 89 独立验证前始终不可信");
  });

  it("keeps Stage 89 as chain-external zero-market receipt validation", () => {
    expect(panelSource).toContain("89 零行情初始化收据独立验证");
    expect(panelSource).toContain("historical_outcome_forward_observation_output_validation_eligible_count");
    expect(panelSource).toContain("historical_outcome_future_first_natural_forward_cycle_authorization_review_eligible_count");
    expect(panelSource).toContain("historical_outcome_first_natural_forward_cycle_future_attempt_eligible_count");
    expect(panelSource).toContain("90 首个自然前向周期一次性授权");
    const validationSource = readFileSync(
      new URL("./public-admin-controlled-shadow-forward-observation-output-validation-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(validationSource).toContain("getControlledShadowForwardObservationOutputValidations");
    expect(validationSource).toContain("validateControlledShadowForwardObservationOutput");
    expect(validationSource).toContain("重建零行情 manifest 与预期收据");
    expect(validationSource).toContain("通过只开放未来首个自然前向周期授权复核资格");
    expect(validationSource).toContain("不会启动 runtime、观察、账本、持仓、绩效或任何交易链路");
  });

  it("keeps Stage 90 as a one-shot review rather than a market-data execution", () => {
    expect(panelSource).toContain("90 首个自然前向周期一次性授权");
    expect(panelSource).toContain("historical_outcome_first_natural_forward_cycle_authorization_active_count");
    const authorizationSource = readFileSync(
      new URL("./public-admin-controlled-shadow-first-natural-forward-cycle-authorization-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(authorizationSource).toContain("getControlledShadowFirstNaturalForwardCycleAuthorizations");
    expect(authorizationSource).toContain("reviewControlledShadowFirstNaturalForwardCycleAuthorization");
    expect(authorizationSource).toContain("首个合格自然周期起算 7 天内有效且最多一次");
    expect(authorizationSource).toContain("未来行情适配器必须另行获得明确、只读、白名单授权");
    expect(authorizationSource).toContain("本次复核不读取日历或行情");
    expect(authorizationSource).toContain("Stage 91 只能另行领取不可执行任务");
  });

  it("keeps Stage 91 as a claim-first non-executable task declaration", () => {
    expect(panelSource).toContain("91 首个自然前向周期任务声明");
    expect(panelSource).toContain("historical_outcome_first_natural_forward_cycle_claim_count");
    expect(panelSource).toContain("historical_outcome_first_natural_forward_cycle_waiting_for_market_data_adapter_authorization_count");
    const claimSource = readFileSync(
      new URL("./public-admin-controlled-shadow-first-natural-forward-cycle-claim-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(claimSource).toContain("claimControlledShadowFirstNaturalForwardCycleOnce");
    expect(claimSource).toContain("领取并永久消费授权");
    expect(claimSource).toContain("先写 claim，之后才可能解析日历或接触行情");
    expect(claimSource).toContain("行情适配器必须另经明确、只读、内容寻址白名单授权");
    expect(claimSource).toContain("当前不启动 runtime/观察，不建账、不写持仓或绩效");
  });

  it("keeps Stage 92 as an independent read-only adapter contract review", () => {
    expect(panelSource).toContain("92 只读行情适配器独立授权");
    expect(panelSource).toContain("historical_outcome_market_data_adapter_authorization_review_eligible_count");
    expect(panelSource).toContain("historical_outcome_future_claim_first_read_only_market_data_receipt_eligible_count");
    const adapterSource = readFileSync(
      new URL("./public-admin-controlled-shadow-market-data-adapter-authorization-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(adapterSource).toContain("getControlledShadowMarketDataAdapterAuthorizations");
    expect(adapterSource).toContain("reviewControlledShadowMarketDataAdapterAuthorization");
    expect(adapterSource).toContain("固定 HTTPS 路径白名单与 GET 请求");
    expect(adapterSource).toContain("未来精确股票集合与时间窗口必须先内容寻址");
    expect(adapterSource).toContain("本次不解析日历、不发请求、不读行情、不启动 runtime");
    expect(adapterSource).toContain("即使批准，也没有解析日历或读取任何行情");
  });

  it("keeps Stage 93 claim-first, single-use and untrusted", () => {
    expect(panelSource).toContain("93 先声明再单次读取原始行情");
    expect(panelSource).toContain("historical_outcome_market_data_receipt_completed_untrusted_count");
    expect(panelSource).toContain("原始载荷不是交易日、观察、持仓、绩效或交易事实");
    const receiptSource = readFileSync(
      new URL("./public-admin-controlled-shadow-market-data-receipt-attempt-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(receiptSource).toContain("claimAndReadControlledShadowMarketDataReceiptOnce");
    expect(receiptSource).toContain("失败或中断也永久消耗本次授权");
    expect(receiptSource).toContain("API 凭据不会写入 claim、收据、响应或日志");
    expect(receiptSource).toContain("成功收据仍是不可信外部证据");
    expect(receiptSource).toContain("不训练、不反馈 reward、不生成订单、不接券商、不交易");
  });

  it("keeps Stage 94 chain-external, byte-verifying and non-parsing", () => {
    expect(panelSource).toContain("94 原始行情收据责任链外独立验证");
    expect(panelSource).toContain("historical_outcome_market_data_receipt_independently_validated_count");
    expect(panelSource).toContain("通过不等于行情语义、收益或模型有效");
    const validationSource = readFileSync(
      new URL("./public-admin-controlled-shadow-market-data-receipt-validation-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(validationSource).toContain("validateControlledShadowMarketDataReceiptOnce");
    expect(validationSource).toContain("重新打开每份原始载荷并重算字节数和 SHA-256");
    expect(validationSource).toContain("这里只核验成功 HTTP 载荷外壳，不把它当作行情事实");
    expect(validationSource).toContain("本阶段不解析交易日历或任何行情行");
    expect(validationSource).toContain("不训练、不反馈 reward、不生成订单、不接券商、不交易");
  });

  it("keeps Stage 95 specification-only, explicit-action and non-executable", () => {
    expect(panelSource).toContain("95 零能力行情解析器规格登记");
    expect(panelSource).toContain("historical_outcome_market_data_parser_spec_registered_count");
    expect(panelSource).toContain("没有 parser 实现、runtime、真实解析、观察或交易权限");
    const specificationSource = readFileSync(
      new URL("./public-admin-controlled-shadow-market-data-parser-specification-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(specificationSource).toContain("registerControlledShadowMarketDataParserSpecificationOnce");
    expect(specificationSource).toContain("价格、原始价、分红调整价、分红、拆股和 NYSE 官方日历均有独立来源");
    expect(specificationSource).toContain("不去重、不前填、不插值，也不回退到未调整收盘价");
    expect(specificationSource).toContain("规格，不是解析器");
    expect(specificationSource).toContain("不训练、不反馈 reward、不生成订单、不接券商、不交易");
  });

  it("keeps Stage 96 chain-external, independently reconstructed and non-executable", () => {
    expect(panelSource).toContain("96 行情解析器规格责任链外独立复核");
    expect(panelSource).toContain("historical_outcome_market_data_parser_spec_independently_approved_count");
    expect(panelSource).toContain("没有 parser、原始载荷访问、行情行、观察或交易权限");
    const reviewSource = readFileSync(
      new URL("./public-admin-controlled-shadow-market-data-parser-specification-review-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(reviewSource).toContain("reviewControlledShadowMarketDataParserSpecificationOnce");
    expect(reviewSource).toContain("已独立重建价格、原始价、分红调整价、分红、拆股和 NYSE 日历请求");
    expect(reviewSource).toContain("已独立重建八组合成向量的输入与预期输出哈希");
    expect(reviewSource).toContain("通过只开放未来零能力 parser 实现登记资格");
    expect(reviewSource).toContain("不生成行情行、观察、账本、持仓、绩效、模型、训练、奖励、订单或交易");
  });

  it("keeps historical transcript anchors human-confirmed and isolated from training", () => {
    expect(panelSource).toContain("PublicAdminHistoricalAnchorPanel");
    const anchorSource = readFileSync(
      new URL("./public-admin-historical-anchor-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(anchorSource).toContain("历史判断锚点（候选 → 老王确认 → 回测）");
    expect(anchorSource).toContain("逐字复制原话（服务器会对照完整文件）");
    expect(anchorSource).toContain("候选没有混入事后信息");
    expect(anchorSource).toContain("自动提取关闭、自动确认关闭、结果标签关闭、训练关闭、奖励关闭、影子关闭、交易关闭");
    expect(anchorSource).toContain("保存候选（不进入训练）");
    expect(anchorSource).toContain("getHistoricalAnchorDiscovery");
    expect(anchorSource).toContain("从逐字稿定位待确认原话");
    expect(anchorSource).toContain("预填到人工候选表单（不保存）");
    expect(anchorSource).toContain("系统尚未保存任何候选");
    expect(anchorSource).toContain("动作归属或方向不够明确，必须人工选择");
    expect(anchorSource).toContain('"active_batch" | "shortlist" | "full_queue"');
    expect(anchorSource).toContain("active_review_batch");
    expect(anchorSource).toContain("每批最多 5 条");
    expect(anchorSource).toContain("active_review_batch_size");
    expect(anchorSource).toContain("说话人标签");
    expect(anchorSource).toContain("身份未确认");
    expect(anchorSource).toContain("screenHistoricalAnchorDiscovery");
    expect(anchorSource).toContain("单问：这条原话是否值得继续建立历史判断候选？");
    expect(anchorSource).toContain("continue_candidate_review");
    expect(anchorSource).toContain("不确认说话人、动作或投资逻辑");
    expect(anchorSource).toContain("查看前后原文");
    expect(anchorSource).toContain("context_sha256");
    expect(anchorSource).toContain("expected_screening_id");
    expect(anchorSource).toContain("填写修正原因（必填）");
    expect(anchorSource).toContain("只追加修正记录，不覆盖旧记录");
  });

  it("reconstructs seven point-in-time layers without future labels or execution", () => {
    expect(panelSource).toContain("PublicAdminHistoricalStateReconstructionPanel");
    const reconstructionSource = readFileSync(
      new URL("./public-admin-historical-state-reconstruction-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(reconstructionSource).toContain("历史点时状态重建");
    expect(reconstructionSource).toContain("建立七层点时状态候选");
    expect(reconstructionSource).toContain("明确缺失，不补造");
    expect(reconstructionSource).toContain("未来数据隔离");
    expect(reconstructionSource).toContain("自动重建关闭、结果标签关闭、训练关闭、奖励关闭、影子关闭、交易关闭");
    expect(reconstructionSource).toContain("冻结点时状态候选（不生成收益）");
    expect(reconstructionSource).toContain("批准项仅成为历史基准状态，结果标签和训练仍关闭");
  });

  it("freezes the historical outcome protocol before any labeler can be reviewed", () => {
    expect(panelSource).toContain("PublicAdminHistoricalOutcomeGovernancePanel");
    const governanceSource = readFileSync(
      new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
      "utf8",
    );
    const apiSource = readFileSync(
      new URL("../lib/api.ts", import.meta.url),
      "utf8",
    );
    const transformationSpecSource = readFileSync(
      new URL("./public-admin-historical-outcome-transformation-spec-panel.tsx", import.meta.url),
      "utf8",
    );
    const transformationSpecReviewSource = readFileSync(
      new URL("./public-admin-historical-outcome-transformation-spec-review-panel.tsx", import.meta.url),
      "utf8",
    );
    const transformationImplementationSource = readFileSync(
      new URL("./public-admin-historical-outcome-transformation-implementation-panel.tsx", import.meta.url),
      "utf8",
    );
    const transformationImplementationReviewSource = readFileSync(
      new URL("./public-admin-historical-outcome-transformation-implementation-review-panel.tsx", import.meta.url),
      "utf8",
    );
    const transformationIsolatedRunnerSource = readFileSync(
      new URL("./public-admin-historical-outcome-transformation-isolated-runner-panel.tsx", import.meta.url),
      "utf8",
    );
    const transformationFirstExecutionAuthorizationSource = readFileSync(
      new URL("./public-admin-historical-outcome-transformation-first-execution-authorization-panel.tsx", import.meta.url),
      "utf8",
    );
    const transformationExecutionAttemptSource = readFileSync(
      new URL("./public-admin-historical-outcome-transformation-execution-attempt-panel.tsx", import.meta.url),
      "utf8",
    );
    const transformationOutputValidationSource = readFileSync(
      new URL("./public-admin-historical-outcome-transformation-output-validation-panel.tsx", import.meta.url),
      "utf8",
    );
    const transformationCandidateAdmissionSource = readFileSync(
      new URL("./public-admin-historical-outcome-transformation-candidate-admission-panel.tsx", import.meta.url),
      "utf8",
    );
    const transformationOfficialArtifactMaterializationSource = readFileSync(
      new URL("./public-admin-historical-outcome-transformation-official-artifact-materialization-panel.tsx", import.meta.url),
      "utf8",
    );
    const transformationOfficialArtifactOutputValidationSource = readFileSync(
      new URL("./public-admin-historical-outcome-transformation-official-artifact-output-validation-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetSpecSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-spec-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetSpecReviewSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-spec-review-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetImplementationSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-implementation-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetImplementationReviewSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-implementation-review-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetIsolatedRunnerSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-isolated-runner-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetFirstExecutionAuthorizationSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-first-execution-authorization-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetExecutionAttemptSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-execution-attempt-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetOutputValidationSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-output-validation-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetCandidateAdmissionSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-candidate-admission-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetOfficialDatasetMaterializationSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-official-dataset-materialization-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetOfficialDatasetOutputValidationSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-official-dataset-output-validation-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetTrainingStoreCopyAdmissionSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-training-store-copy-admission-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetTrainingStoreCopySource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-training-store-copy-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetTrainingStoreCopyOutputValidationSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-training-store-copy-output-validation-panel.tsx", import.meta.url),
      "utf8",
    );
    const featureLabelJoinTargetTrainingRegistrationAdmissionSource = readFileSync(
      new URL("./public-admin-historical-outcome-feature-label-join-target-training-registration-admission-panel.tsx", import.meta.url),
      "utf8",
    );
    const trainingExperimentRegistrationSource = readFileSync(
      new URL("./public-admin-historical-outcome-training-experiment-registration-panel.tsx", import.meta.url),
      "utf8",
    );
    const trainingExperimentRegistrationReviewSource = readFileSync(
      new URL("./public-admin-historical-outcome-training-experiment-registration-review-panel.tsx", import.meta.url),
      "utf8",
    );
    const trainingImplementationSource = readFileSync(
      new URL("./public-admin-historical-outcome-training-implementation-panel.tsx", import.meta.url),
      "utf8",
    );
    const trainingImplementationReviewSource = readFileSync(
      new URL("./public-admin-historical-outcome-training-implementation-review-panel.tsx", import.meta.url),
      "utf8",
    );
    const trainingIsolatedRunnerSource = readFileSync(
      new URL("./public-admin-historical-outcome-training-isolated-runner-panel.tsx", import.meta.url),
      "utf8",
    );
    const trainingFirstExecutionAuthorizationSource = readFileSync(
      new URL("./public-admin-historical-outcome-training-first-execution-authorization-panel.tsx", import.meta.url),
      "utf8",
    );
    const trainingExecutionAttemptSource = readFileSync(
      new URL("./public-admin-historical-outcome-training-execution-attempt-panel.tsx", import.meta.url),
      "utf8",
    );
    const trainingOutputValidationSource = readFileSync(
      new URL("./public-admin-historical-outcome-training-output-validation-panel.tsx", import.meta.url),
      "utf8",
    );
    const validationEvaluationImplementationSource = readFileSync(
      new URL("./public-admin-historical-outcome-validation-evaluation-implementation-panel.tsx", import.meta.url),
      "utf8",
    );
    const validationEvaluationImplementationReviewSource = readFileSync(
      new URL("./public-admin-historical-outcome-validation-evaluation-implementation-review-panel.tsx", import.meta.url),
      "utf8",
    );
    const validationEvaluationIsolatedRunnerSource = readFileSync(
      new URL("./public-admin-historical-outcome-validation-evaluation-isolated-runner-panel.tsx", import.meta.url),
      "utf8",
    );
    const validationEvaluationFirstExecutionAuthorizationSource = readFileSync(
      new URL("./public-admin-historical-outcome-validation-evaluation-first-execution-authorization-panel.tsx", import.meta.url),
      "utf8",
    );
    const validationEvaluationExecutionAttemptSource = readFileSync(
      new URL("./public-admin-historical-outcome-validation-evaluation-execution-attempt-panel.tsx", import.meta.url),
      "utf8",
    );
    const validationEvaluationOutputValidationSource = readFileSync(
      new URL("./public-admin-historical-outcome-validation-evaluation-output-validation-panel.tsx", import.meta.url),
      "utf8",
    );
    const validationEvaluationPerTargetCandidateAdmissionSource = readFileSync(
      new URL("./public-admin-historical-outcome-validation-evaluation-per-target-candidate-admission-panel.tsx", import.meta.url),
      "utf8",
    );
    const sealedHoldoutEvaluationProtocolReviewSource = readFileSync(
      new URL("./public-admin-historical-outcome-sealed-holdout-evaluation-protocol-review-panel.tsx", import.meta.url),
      "utf8",
    );
    const sealedHoldoutEvaluationImplementationSource = readFileSync(
      new URL("./public-admin-historical-outcome-sealed-holdout-evaluation-implementation-panel.tsx", import.meta.url),
      "utf8",
    );
    const sealedHoldoutEvaluationImplementationReviewSource = readFileSync(
      new URL("./public-admin-historical-outcome-sealed-holdout-evaluation-implementation-review-panel.tsx", import.meta.url),
      "utf8",
    );
    const sealedHoldoutEvaluationIsolatedRunnerSource = readFileSync(
      new URL("./public-admin-historical-outcome-sealed-holdout-evaluation-isolated-runner-panel.tsx", import.meta.url),
      "utf8",
    );
    const sealedHoldoutEvaluationFirstExecutionAuthorizationSource = readFileSync(
      new URL("./public-admin-historical-outcome-sealed-holdout-evaluation-first-execution-authorization-panel.tsx", import.meta.url),
      "utf8",
    );
    const sealedHoldoutEvaluationExecutionAttemptSource = readFileSync(
      new URL("./public-admin-historical-outcome-sealed-holdout-evaluation-execution-attempt-panel.tsx", import.meta.url),
      "utf8",
    );
    const sealedHoldoutEvaluationOutputValidationSource = readFileSync(
      new URL("./public-admin-historical-outcome-sealed-holdout-evaluation-output-validation-panel.tsx", import.meta.url),
      "utf8",
    );
    const sealedHoldoutConfirmatoryResultAdjudicationSource = readFileSync(
      new URL("./public-admin-historical-outcome-sealed-holdout-confirmatory-result-adjudication-panel.tsx", import.meta.url),
      "utf8",
    );
    expect(governanceSource).toContain("历史结果协议冻结与审批");
    expect(governanceSource).toContain("getHistoricalOutcomeGovernance");
    expect(governanceSource).toContain("reviewHistoricalOutcomeGovernance");
    expect(governanceSource).toContain("批准未来标签器实现评审");
    expect(governanceSource).toContain("当前没有人工批准的历史基准状态，因此不能批准任何标签器实现评审");
    expect(governanceSource).toContain("结果标签关闭、训练关闭、奖励关闭、影子关闭、交易关闭");
    expect(governanceSource).toContain("复核结果计算协议（不生成标签）");
    expect(governanceSource).toContain("历史结果标签器实现登记与审查");
    expect(governanceSource).toContain("getHistoricalOutcomeLabelers");
    expect(governanceSource).toContain("registerHistoricalOutcomeLabeler");
    expect(governanceSource).toContain("reviewHistoricalOutcomeLabeler");
    expect(governanceSource).toContain("getHistoricalOutcomePriceSnapshots");
    expect(governanceSource).toContain("ingestHistoricalOutcomePriceSnapshot");
    expect(governanceSource).toContain("getHistoricalOutcomeDryRunAuthorizations");
    expect(governanceSource).toContain("reviewHistoricalOutcomeDryRunAuthorization");
    expect(governanceSource).toContain("getHistoricalOutcomeDryRunImplementations");
    expect(governanceSource).toContain("registerHistoricalOutcomeDryRunImplementation");
    expect(governanceSource).toContain("getHistoricalOutcomeDryRunRunAuthorizations");
    expect(governanceSource).toContain("reviewHistoricalOutcomeDryRunRunAuthorization");
    expect(governanceSource).toContain("getHistoricalOutcomeDryRunIsolatedRunners");
    expect(governanceSource).toContain("registerHistoricalOutcomeDryRunIsolatedRunner");
    expect(governanceSource).toContain("getHistoricalOutcomeDryRunFirstExecutionAuthorizations");
    expect(governanceSource).toContain("reviewHistoricalOutcomeDryRunFirstExecutionAuthorization");
    expect(governanceSource).toContain("getHistoricalOutcomeDryRunExecutionAttempts");
    expect(governanceSource).toContain("invokeHistoricalOutcomeDryRunOnce");
    expect(governanceSource).toContain("getHistoricalOutcomeDryRunOutputValidations");
    expect(governanceSource).toContain("validateHistoricalOutcomeDryRunOutput");
    expect(governanceSource).toContain("登记冻结实现规范（不运行）");
    expect(governanceSource).toContain("批准进入离线试运行授权复核");
    expect(governanceSource).toContain("实现登记、人工复核、离线试运行授权和结果标签生成是四道独立门禁");
    expect(governanceSource).toContain("联网、外部工具、生产写入、标签写入全部关闭");
    expect(governanceSource).toContain("从 FMP 封存复权行情（不计算收益）");
    expect(governanceSource).toContain("收益未计算、标签未写入");
    expect(governanceSource).toContain("批准下一步登记离线试运行实现");
    expect(governanceSource).toContain("批准后仍不运行标签器");
    expect(governanceSource).toContain("登记隔离试运行实现（只登记，不运行）");
    expect(governanceSource).toContain("订单和券商访问全部关闭");
    expect(governanceSource).toContain("批准下一步登记隔离执行器（仍不运行）");
    expect(governanceSource).toContain("写入不可覆盖的运行授权复核");
    expect(governanceSource).toContain("运行授权仍为否，输出工件仍不存在");
    expect(governanceSource).toContain("隔离执行器规范登记");
    expect(governanceSource).toContain("登记隔离执行器规范（只登记，不调用）");
    expect(governanceSource).toContain("无入口、无环境变量、无密钥、无网络、无生产写入");
    expect(governanceSource).toContain("首次执行授权、隔离输出校验和结果标签准入仍是后续独立门禁");
    expect(governanceSource).toContain("首次执行授权复核");
    expect(governanceSource).toContain("批准 24 小时内一次首次执行（当前不调用）");
    expect(governanceSource).toContain("写入不可覆盖的首次执行授权复核");
    expect(governanceSource).toContain("一次性能力隔离执行");
    expect(governanceSource).toContain("未验证工件");
    expect(governanceSource).toContain("独立输出校验与确定性重算");
    expect(governanceSource).toContain("执行调用人、运行器登记者和两级授权复核人都不能担任本次校验人");
    expect(governanceSource).toContain("getHistoricalOutcomeLabelAdmissionReviews");
    expect(governanceSource).toContain("reviewHistoricalOutcomeLabelAdmission");
    expect(governanceSource).toContain("结果标签准入复核");
    expect(governanceSource).toContain("已知局限与偏差（必填）");
    expect(governanceSource).toContain("getHistoricalOutcomeLabelMaterializationImplementations");
    expect(governanceSource).toContain("registerHistoricalOutcomeLabelMaterializationImplementation");
    expect(governanceSource).toContain("原始结果信封物化实现登记");
    expect(governanceSource).toContain("deterministic_raw_validated_outcome_envelope");
    expect(governanceSource).toContain("不得从收益推断方向、评级、买卖动作、仓位或奖励");
    expect(governanceSource).toContain("仅登记物化实现规范（不运行）");
    expect(governanceSource).toContain("getHistoricalOutcomeLabelMaterializationRunAuthorizations");
    expect(governanceSource).toContain("reviewHistoricalOutcomeLabelMaterializationRunAuthorization");
    expect(governanceSource).toContain("标签物化运行授权复核");
    expect(governanceSource).toContain("批准下一步登记隔离物化 runner（当前不运行）");
    expect(governanceSource).toContain("标签写入、训练、奖励、影子、订单、券商和交易权限保持关闭");
    expect(governanceSource).toContain("getHistoricalOutcomeLabelMaterializationIsolatedRunners");
    expect(governanceSource).toContain("registerHistoricalOutcomeLabelMaterializationIsolatedRunner");
    expect(governanceSource).toContain("标签物化隔离 runner 规范");
    expect(governanceSource).toContain("写入不可覆盖的 runner 规范");
    expect(governanceSource).toContain("第十六阶段只登记 runner 制品摘要");
    expect(governanceSource).toContain("getHistoricalOutcomeLabelMaterializationFirstExecutionAuthorizations");
    expect(governanceSource).toContain("reviewHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization");
    expect(governanceSource).toContain("标签物化首次执行授权复核");
    expect(governanceSource).toContain("第十七阶段只建立短时、一次性的未来首次执行授权");
    expect(governanceSource).toContain("批准 24 小时内一次未来首次执行（当前不调用）");
    expect(governanceSource).toContain("getHistoricalOutcomeLabelMaterializationExecutionAttempts");
    expect(governanceSource).toContain("invokeHistoricalOutcomeLabelMaterializationOnce");
    expect(governanceSource).toContain("标签物化一次性执行");
    expect(governanceSource).toContain("消费一次性授权并执行固定物化");
    expect(governanceSource).toContain("成功只复制已独立验证的 20 / 60 / 250 日原始指标");
    expect(governanceSource).toContain("绝不生成正式标签");
    expect(governanceSource).toContain("getHistoricalOutcomeLabelMaterializationOutputValidations");
    expect(governanceSource).toContain("validateHistoricalOutcomeLabelMaterializationOutput");
    expect(governanceSource).toContain("标签物化结果独立校验");
    expect(governanceSource).toContain("独立校验结构、来源与逐位一致性");
    expect(governanceSource).toContain("通过仍不是正式结果标签");
    expect(apiSource).toContain("/historical-outcome-label-materialization-output-validations");
    expect(apiSource).toContain("/validate`");
    expect(governanceSource).toContain("getHistoricalOutcomeLabelWriteAuthorizations");
    expect(governanceSource).toContain("reviewHistoricalOutcomeLabelWriteAuthorization");
    expect(governanceSource).toContain("正式标签未来一次写入授权复核");
    expect(governanceSource).toContain("批准本身不是写入");
    expect(governanceSource).toContain("第 21 阶段 writer 只接受当前未过期且未消费的批准");
    expect(apiSource).toContain("/historical-outcome-label-write-authorizations");
    expect(apiSource).toContain("/review`");
    expect(governanceSource).toContain("getHistoricalOutcomeFormalLabelWrites");
    expect(governanceSource).toContain("writeHistoricalOutcomeFormalLabelOnce");
    expect(governanceSource).toContain("第 21 阶段 · 正式原始结果标签一次性写入");
    expect(governanceSource).toContain("消费一次性授权并 create-once 写入");
    expect(governanceSource).toContain("失败或中断也不能重试同一授权");
    expect(apiSource).toContain("/historical-outcome-formal-label-writes");
    expect(apiSource).toContain("/write-once`");
    expect(governanceSource).toContain("getHistoricalOutcomeFormalLabelValidations");
    expect(governanceSource).toContain("validateHistoricalOutcomeFormalLabel");
    expect(governanceSource).toContain("第 22 阶段 · 正式标签独立校验与离线数据集候选准入");
    expect(governanceSource).toContain("候选≠训练");
    expect(governanceSource).toContain("运行独立校验并写入不可变准入记录");
    expect(apiSource).toContain("/historical-outcome-formal-label-validations");
    expect(apiSource).toContain("/validate`");
    expect(governanceSource).toContain("getHistoricalOutcomeOfflineDatasets");
    expect(governanceSource).toContain("assembleHistoricalOutcomeOfflineDataset");
    expect(governanceSource).toContain("第 23 阶段 · 版本化离线历史结果数据集装配");
    expect(governanceSource).toContain("装配当前完整候选集并写入不可变数据集版本");
    expect(governanceSource).toContain("数据集≠训练");
    expect(apiSource).toContain("/historical-outcome-offline-datasets");
    expect(governanceSource).toContain("getHistoricalOutcomeOfflineDatasetGovernance");
    expect(governanceSource).toContain("reviewHistoricalOutcomeOfflineDatasetGovernance");
    expect(governanceSource).toContain("第 24 阶段 · 离线数据集独立治理复核");
    expect(governanceSource).toContain("purge / embargo");
    expect(governanceSource).toContain("写入不可变治理复核记录");
    expect(apiSource).toContain("/historical-outcome-offline-dataset-governance");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTransformationSpecPanel");
    expect(transformationSpecSource).toContain("getHistoricalOutcomeOfflineDatasetTransformationSpecs");
    expect(transformationSpecSource).toContain("registerHistoricalOutcomeOfflineDatasetTransformationSpec");
    expect(transformationSpecSource).toContain("第 25 阶段 · 不可变转换规范登记");
    expect(transformationSpecSource).toContain("登记不可变转换规范（不执行）");
    expect(transformationSpecSource).toContain("独立复核：未完成");
    expect(transformationSpecSource).toContain("个精确 feature ID");
    expect(apiSource).toContain("/historical-outcome-offline-dataset-transformation-specs");
    expect(apiSource).toContain("/register`");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTransformationSpecReviewPanel");
    expect(transformationSpecReviewSource).toContain("第 26 阶段 · 转换规范独立复核");
    expect(transformationSpecReviewSource).toContain("65 个 feature ID");
    expect(transformationSpecReviewSource).toContain("不登记实现、不执行");
    expect(transformationSpecReviewSource).toContain("getHistoricalOutcomeOfflineDatasetTransformationSpecReviews");
    expect(transformationSpecReviewSource).toContain("reviewHistoricalOutcomeOfflineDatasetTransformationSpec");
    expect(apiSource).toContain("/historical-outcome-offline-dataset-transformation-spec-reviews");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTransformationImplementationPanel");
    expect(transformationImplementationSource).toContain("第 27 阶段 · 隔离转换实现规范登记");
    expect(transformationImplementationSource).toContain("登记实现规范（无入口、不执行）");
    expect(apiSource).toContain("/historical-outcome-offline-dataset-transformation-implementations");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTransformationImplementationReviewPanel");
    expect(transformationImplementationReviewSource).toContain("第 28 阶段 · 隔离转换实现独立复核");
    expect(transformationImplementationReviewSource).toContain("独立工件与沙箱审计合同");
    expect(transformationImplementationReviewSource).toContain("不登记 runner、不执行");
    expect(transformationImplementationReviewSource).toContain("getHistoricalOutcomeOfflineDatasetTransformationImplementationReviews");
    expect(transformationImplementationReviewSource).toContain("reviewHistoricalOutcomeOfflineDatasetTransformationImplementation");
    expect(apiSource).toContain("/historical-outcome-offline-dataset-transformation-implementation-reviews");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTransformationIsolatedRunnerPanel");
    expect(transformationIsolatedRunnerSource).toContain("第 29 阶段 · 隔离转换 runner 规范登记");
    expect(transformationIsolatedRunnerSource).toContain("登记 runner 规范（无入口、不执行）");
    expect(transformationIsolatedRunnerSource).toContain("唯一下一门禁：独立首次执行授权复核");
    expect(transformationIsolatedRunnerSource).toContain("getHistoricalOutcomeOfflineDatasetTransformationIsolatedRunners");
    expect(transformationIsolatedRunnerSource).toContain("registerHistoricalOutcomeOfflineDatasetTransformationIsolatedRunner");
    expect(apiSource).toContain("/historical-outcome-offline-dataset-transformation-isolated-runners");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTransformationFirstExecutionAuthorizationPanel");
    expect(transformationFirstExecutionAuthorizationSource).toContain("第 30 阶段 · 隔离转换首次执行授权复核");
    expect(transformationFirstExecutionAuthorizationSource).toContain("授权不是调用");
    expect(transformationFirstExecutionAuthorizationSource).toContain("追加首次执行授权复核（不调用、不执行）");
    expect(transformationFirstExecutionAuthorizationSource).toContain("getHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizations");
    expect(transformationFirstExecutionAuthorizationSource).toContain("reviewHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization");
    expect(apiSource).toContain("/historical-outcome-offline-dataset-transformation-first-execution-authorizations");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTransformationExecutionAttemptPanel");
    expect(transformationExecutionAttemptSource).toContain("第 31 阶段 · 隔离转换一次性执行尝试");
    expect(transformationExecutionAttemptSource).toContain("领取授权并执行一次（失败也消费）");
    expect(transformationExecutionAttemptSource).toContain("只执行固定纯函数");
    expect(transformationExecutionAttemptSource).toContain("正式 manifest：未创建");
    expect(transformationExecutionAttemptSource).toContain("invokeHistoricalOutcomeOfflineDatasetTransformationOnce");
    expect(apiSource).toContain("/historical-outcome-offline-dataset-transformation-execution-attempts");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTransformationOutputValidationPanel");
    expect(transformationOutputValidationSource).toContain("第 32 阶段 · 离线转换输出独立重算");
    expect(transformationOutputValidationSource).toContain("独立算法 · 不复用执行代码");
    expect(transformationOutputValidationSource).toContain("独立重算并校验一次");
    expect(transformationOutputValidationSource).toContain("正式 manifest：未创建");
    expect(transformationOutputValidationSource).toContain("validateHistoricalOutcomeOfflineDatasetTransformationOutput");
    expect(apiSource).toContain("/historical-outcome-offline-dataset-transformation-output-validations");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTransformationCandidateAdmissionPanel");
    expect(transformationCandidateAdmissionSource).toContain("第 33 阶段 · 离线转换候选独立准入复核");
    expect(transformationCandidateAdmissionSource).toContain("准入不是正式物化");
    expect(transformationCandidateAdmissionSource).toContain("追加候选准入复核（不物化）");
    expect(transformationCandidateAdmissionSource).toContain("65 项点时特征白名单");
    expect(transformationCandidateAdmissionSource).toContain("reviewHistoricalOutcomeOfflineDatasetTransformationCandidateAdmission");
    expect(apiSource).toContain("/historical-outcome-offline-dataset-transformation-candidate-admission-reviews");
    expect(panelSource).toContain("㉝ 离线转换候选独立准入");
    expect(panelSource).toContain("准入不是正式物化");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTransformationOfficialArtifactMaterializationPanel");
    expect(transformationOfficialArtifactMaterializationSource).toContain("第 34 阶段 · 正式 manifest / feature bundle 一次性物化");
    expect(transformationOfficialArtifactMaterializationSource).toContain("正式物化不是训练准入");
    expect(transformationOfficialArtifactMaterializationSource).toContain("一次性物化正式工件（失败也消费）");
    expect(transformationOfficialArtifactMaterializationSource).toContain("materializeHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsOnce");
    expect(apiSource).toContain("/historical-outcome-offline-dataset-transformation-official-artifact-materializations");
    expect(panelSource).toContain("㉞ 正式 manifest / feature bundle 一次性物化");
    expect(panelSource).toContain("正式物化不是训练准入");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTransformationOfficialArtifactOutputValidationPanel");
    expect(transformationOfficialArtifactOutputValidationSource).toContain("第 35 阶段 · 正式工件物化后独立校验");
    expect(transformationOfficialArtifactOutputValidationSource).toContain("校验通过仍不是训练输入");
    expect(transformationOfficialArtifactOutputValidationSource).toContain("独立校验正式工件一次");
    expect(transformationOfficialArtifactOutputValidationSource).toContain("validateHistoricalOutcomeOfflineDatasetTransformationOfficialArtifacts");
    expect(apiSource).toContain("/historical-outcome-offline-dataset-transformation-official-artifact-output-validations");
    expect(panelSource).toContain("㉟ 正式工件物化后独立校验");
    expect(panelSource).toContain("通过只开放未来 join/target 治理规范登记");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetSpecPanel");
    expect(featureLabelJoinTargetSpecSource).toContain("第 36 阶段 · join/target 治理规范登记");
    expect(featureLabelJoinTargetSpecSource).toContain("目标不是“买/卖标签”");
    expect(featureLabelJoinTargetSpecSource).toContain("登记 join/target 治理规范");
    expect(featureLabelJoinTargetSpecSource).toContain("registerHistoricalOutcomeFeatureLabelJoinTargetSpec");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-specs");
    expect(panelSource).toContain("㊱ 特征—标签连接与连续目标规范");
    expect(panelSource).toContain("不把投资动作或奖励伪装成标签");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetSpecReviewPanel");
    expect(featureLabelJoinTargetSpecReviewSource).toContain("第 37 阶段 · join/target 规范独立复核");
    expect(featureLabelJoinTargetSpecReviewSource).toContain("只是工程候选，不是老王确认逻辑或策略真理");
    expect(featureLabelJoinTargetSpecReviewSource).toContain("reviewHistoricalOutcomeFeatureLabelJoinTargetSpec");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-spec-reviews");
    expect(panelSource).toContain("㊲ join/target 规范独立语义与指纹复核");
    expect(panelSource).toContain("批准不执行 join、不生成训练行");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetImplementationPanel");
    expect(featureLabelJoinTargetImplementationSource).toContain("第 38 阶段 · join/target 隔离实现登记");
    expect(featureLabelJoinTargetImplementationSource).toContain("九维目标只投影原始 f64 位");
    expect(featureLabelJoinTargetImplementationSource).toContain("registerHistoricalOutcomeFeatureLabelJoinTargetImplementation");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-implementations");
    expect(panelSource).toContain("㊳ join/target 隔离实现登记");
    expect(panelSource).toContain("登记没有入口、runner、标签访问、join");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetImplementationReviewPanel");
    expect(featureLabelJoinTargetImplementationReviewSource).toContain("第 39 阶段 · join/target 实现独立复核");
    expect(featureLabelJoinTargetImplementationReviewSource).toContain("九维目标仍是工程候选，不是策略真理");
    expect(featureLabelJoinTargetImplementationReviewSource).toContain("reviewHistoricalOutcomeFeatureLabelJoinTargetImplementation");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-implementation-reviews");
    expect(panelSource).toContain("㊴ join/target 实现独立复核");
    expect(panelSource).toContain("批准仍不创建 runner、不读取标签、不执行 join");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetIsolatedRunnerPanel");
    expect(featureLabelJoinTargetIsolatedRunnerSource).toContain("第 40 阶段 · join/target 隔离 runner 规范登记");
    expect(featureLabelJoinTargetIsolatedRunnerSource).toContain("没有调用入口");
    expect(featureLabelJoinTargetIsolatedRunnerSource).toContain("registerHistoricalOutcomeFeatureLabelJoinTargetIsolatedRunner");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-isolated-runners");
    expect(panelSource).toContain("㊵ join/target 隔离 runner 规格登记");
    expect(panelSource).toContain("没有可调用入口、标签/训练库读取、join");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizationPanel");
    expect(featureLabelJoinTargetFirstExecutionAuthorizationSource).toContain("第 41 阶段 · join/target 首次执行授权复核");
    expect(featureLabelJoinTargetFirstExecutionAuthorizationSource).toContain("授权不等于执行");
    expect(featureLabelJoinTargetFirstExecutionAuthorizationSource).toContain("追加首次执行授权复核（不 claim、不执行）");
    expect(featureLabelJoinTargetFirstExecutionAuthorizationSource).toContain("no_generic_label_or_training_store_access_confirmed");
    expect(featureLabelJoinTargetFirstExecutionAuthorizationSource).toContain("getHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorizations");
    expect(featureLabelJoinTargetFirstExecutionAuthorizationSource).toContain("reviewHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-first-execution-authorizations");
    expect(panelSource).toContain("㊶ join/target 首次执行授权复核");
    expect(panelSource).toContain("批准不是 claim 或执行");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptPanel");
    expect(featureLabelJoinTargetExecutionAttemptSource).toContain("第 42 阶段 · join/target 一次性执行尝试");
    expect(featureLabelJoinTargetExecutionAttemptSource).toContain("失败也消费");
    expect(featureLabelJoinTargetExecutionAttemptSource).toContain("validation 与 sealed holdout");
    expect(featureLabelJoinTargetExecutionAttemptSource).toContain("invokeHistoricalOutcomeFeatureLabelJoinTargetOnce");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-execution-attempts");
    expect(panelSource).toContain("㊷ join/target 一次性执行尝试");
    expect(panelSource).toContain("候选尚非正式 joined dataset 或训练数据");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetOutputValidationPanel");
    expect(featureLabelJoinTargetOutputValidationSource).toContain("第 43 阶段 · join/target 独立输出校验");
    expect(featureLabelJoinTargetOutputValidationSource).toContain("不复用第 42 阶段投影或信封校验算法");
    expect(featureLabelJoinTargetOutputValidationSource).toContain("validateHistoricalOutcomeFeatureLabelJoinTargetOutput");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-output-validations");
    expect(panelSource).toContain("㊸ join/target 独立输出校验");
    expect(panelSource).toContain("通过仍只是不可信候选");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionPanel");
    expect(featureLabelJoinTargetCandidateAdmissionSource).toContain("第 44 阶段 · join/target 候选独立准入复核");
    expect(featureLabelJoinTargetCandidateAdmissionSource).toContain("准入不是正式数据集");
    expect(featureLabelJoinTargetCandidateAdmissionSource).toContain("reviewHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmission");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-candidate-admission-reviews");
    expect(panelSource).toContain("㊹ join/target 候选独立准入复核");
    expect(panelSource).toContain("批准只开放下一道 create-once 物化门禁");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationPanel");
    expect(featureLabelJoinTargetOfficialDatasetMaterializationSource).toContain("第 45 阶段 · 正式 joined dataset 一次性物化");
    expect(featureLabelJoinTargetOfficialDatasetMaterializationSource).toContain("正式数据集仍不是训练数据");
    expect(featureLabelJoinTargetOfficialDatasetMaterializationSource).toContain("失败也消费");
    expect(featureLabelJoinTargetOfficialDatasetMaterializationSource).toContain("materializeHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOnce");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-official-dataset-materializations");
    expect(panelSource).toContain("㊺ 正式 joined dataset 一次性物化");
    expect(panelSource).toContain("落盘仍不是训练准入");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationPanel");
    expect(featureLabelJoinTargetOfficialDatasetOutputValidationSource).toContain("第 46 阶段 · 正式 joined dataset 独立输出校验");
    expect(featureLabelJoinTargetOfficialDatasetOutputValidationSource).toContain("不复用第 45 阶段校验辅助函数");
    expect(featureLabelJoinTargetOfficialDatasetOutputValidationSource).toContain("独立通过仍不是训练库准入");
    expect(featureLabelJoinTargetOfficialDatasetOutputValidationSource).toContain("validateHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-official-dataset-output-validations");
    expect(panelSource).toContain("㊻ 正式 joined dataset 独立输出校验");
    expect(panelSource).toContain("独立通过只开放未来训练库复制准入复核");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionPanel");
    expect(featureLabelJoinTargetTrainingStoreCopyAdmissionSource).toContain("第 47 阶段 · 训练存储复制独立准入复核");
    expect(featureLabelJoinTargetTrainingStoreCopyAdmissionSource).toContain("准入不是复制，更不是训练");
    expect(featureLabelJoinTargetTrainingStoreCopyAdmissionSource).toContain("reviewHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmission");
    expect(featureLabelJoinTargetTrainingStoreCopyAdmissionSource).toContain("复制后仍需另一实现独立校验");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-store-copy-admission-reviews");
    expect(panelSource).toContain("㊼ 训练存储复制独立准入复核");
    expect(panelSource).toContain("批准也只开放未来 create-once 复制门禁");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyPanel");
    expect(featureLabelJoinTargetTrainingStoreCopySource).toContain("第 48 阶段 · 训练存储一次性复制");
    expect(featureLabelJoinTargetTrainingStoreCopySource).toContain("复制不是训练");
    expect(featureLabelJoinTargetTrainingStoreCopySource).toContain("copyHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreOnce");
    expect(featureLabelJoinTargetTrainingStoreCopySource).toContain("复制失败同样消费资格");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-store-copies");
    expect(panelSource).toContain("㊽ 训练存储一次性复制");
    expect(panelSource).toContain("复制成功仍不是训练登记或训练授权");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationPanel");
    expect(featureLabelJoinTargetTrainingStoreCopyOutputValidationSource).toContain("第 49 阶段 · 训练存储副本独立校验");
    expect(featureLabelJoinTargetTrainingStoreCopyOutputValidationSource).toContain("复制一致 ≠ 模型有效");
    expect(featureLabelJoinTargetTrainingStoreCopyOutputValidationSource).toContain("validateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopy");
    expect(featureLabelJoinTargetTrainingStoreCopyOutputValidationSource).toContain("通过只证明复制一致");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-store-copy-output-validations");
    expect(panelSource).toContain("㊾ 训练存储副本独立校验");
    expect(panelSource).toContain("通过只证明复制一致，不证明模型有效");
    expect(featureLabelJoinTargetTrainingRegistrationAdmissionSource).toContain("第 50 阶段 · 训练登记独立准入复核");
    expect(featureLabelJoinTargetTrainingRegistrationAdmissionSource).toContain("登记准入 ≠ 训练有效");
    expect(featureLabelJoinTargetTrainingRegistrationAdmissionSource).toContain("reviewHistoricalOutcomeFeatureLabelJoinTargetTrainingRegistrationAdmission");
    expect(featureLabelJoinTargetTrainingRegistrationAdmissionSource).toContain("批准也只开放未来 create-once 登记门禁");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-registration-admission-reviews");
    expect(panelSource).toContain("㊿ 训练登记独立准入复核");
    expect(panelSource).toContain("批准也只开放未来 create-once 训练登记门禁");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTrainingExperimentRegistrationPanel");
    expect(trainingExperimentRegistrationSource).toContain("第 51 阶段 · 训练实验一次性登记");
    expect(trainingExperimentRegistrationSource).toContain("登记 ≠ 训练运行");
    expect(trainingExperimentRegistrationSource).toContain("registerHistoricalOutcomeTrainingExperimentSuiteOnce");
    expect(trainingExperimentRegistrationSource).toContain("登记成功也必须独立复核");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-experiment-registrations");
    expect(panelSource).toContain("51 训练实验一次性登记");
    expect(panelSource).toContain("登记完成仍不授权或启动训练");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTrainingExperimentRegistrationReviewPanel");
    expect(trainingExperimentRegistrationReviewSource).toContain("第 52 阶段 · 训练实验登记独立复核");
    expect(trainingExperimentRegistrationReviewSource).toContain("登记复核 ≠ 训练授权");
    expect(trainingExperimentRegistrationReviewSource).toContain("reviewHistoricalOutcomeTrainingExperimentRegistration");
    expect(trainingExperimentRegistrationReviewSource).toContain("批准仍不创建 runner、不授权或启动训练");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-experiment-registration-reviews");
    expect(panelSource).toContain("52 训练实验登记独立复核");
    expect(panelSource).toContain("已独立批准 · 等待训练实现登记");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTrainingImplementationPanel");
    expect(trainingImplementationSource).toContain("第 53 阶段 · 训练实现登记");
    expect(trainingImplementationSource).toContain("实现登记 ≠ 训练运行");
    expect(trainingImplementationSource).toContain("registerHistoricalOutcomeTrainingImplementation");
    expect(trainingImplementationSource).toContain("下一步只能独立复核实现");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-implementations");
    expect(panelSource).toContain("53 训练实现登记");
    expect(panelSource).toContain("已登记 · 等待独立实现复核");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTrainingImplementationReviewPanel");
    expect(trainingImplementationReviewSource).toContain("第 54 阶段 · 训练实现独立复核");
    expect(trainingImplementationReviewSource).toContain("实现复核 ≠ runner 或训练授权");
    expect(trainingImplementationReviewSource).toContain("reviewHistoricalOutcomeTrainingImplementation");
    expect(trainingImplementationReviewSource).toContain("只有未来 runner 规格登记资格");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-implementation-reviews");
    expect(panelSource).toContain("54 训练实现独立复核");
    expect(panelSource).toContain("已独立批准 · 仅可登记 runner 规格");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTrainingIsolatedRunnerPanel");
    expect(trainingIsolatedRunnerSource).toContain("第 55 阶段 · 训练隔离 runner 规范登记");
    expect(trainingIsolatedRunnerSource).toContain("无入口 · 不执行");
    expect(trainingIsolatedRunnerSource).toContain("registerHistoricalOutcomeTrainingIsolatedRunner");
    expect(trainingIsolatedRunnerSource).toContain("唯一下一门禁：独立首次执行授权复核");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-isolated-runners");
    expect(panelSource).toContain("55 训练隔离 runner 规格登记");
    expect(panelSource).toContain("registered_not_run · 等待首次执行授权复核");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTrainingFirstExecutionAuthorizationPanel");
    expect(trainingFirstExecutionAuthorizationSource).toContain("第 56 阶段 · 训练首次执行授权复核");
    expect(trainingFirstExecutionAuthorizationSource).toContain("授权不等于执行");
    expect(trainingFirstExecutionAuthorizationSource).toContain("reviewHistoricalOutcomeTrainingFirstExecutionAuthorization");
    expect(trainingFirstExecutionAuthorizationSource).toContain("尚未 claim、未读取数据、未训练，也未生成模型或指标");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-first-execution-authorizations");
    expect(panelSource).toContain("56 训练首次执行授权复核");
    expect(panelSource).toContain("授权只开放下一阶段一次 claim-first、train-only 拟合，不代表模型有效");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTrainingExecutionAttemptPanel");
    expect(trainingExecutionAttemptSource).toContain("第 57 阶段 · 训练一次性执行尝试");
    expect(trainingExecutionAttemptSource).toContain("真实拟合 ≠ 模型有效");
    expect(trainingExecutionAttemptSource).toContain("invokeHistoricalOutcomeTrainingOnce");
    expect(trainingExecutionAttemptSource).toContain("失败同样消耗授权，绝不自动重放");
    expect(trainingExecutionAttemptSource).toContain("validation 与 sealed holdout 标签继续隐藏");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-execution-attempts");
    expect(panelSource).toContain("57 训练一次性执行尝试");
    expect(panelSource).toContain("真实拟合 ≠ 模型有效，不做 validation 选模");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeTrainingOutputValidationPanel");
    expect(trainingOutputValidationSource).toContain("第 58 阶段 · 训练产物独立复算验证");
    expect(trainingOutputValidationSource).toContain("可重现 ≠ 有效，更不等于可交易");
    expect(trainingOutputValidationSource).toContain("validateHistoricalOutcomeTrainingOutput");
    expect(trainingOutputValidationSource).toContain("9 个模型工件和 81 项 train-only 诊断");
    expect(trainingOutputValidationSource).toContain("validation 与 sealed holdout 标签继续隐藏");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-output-validations");
    expect(panelSource).toContain("58 训练产物独立复算验证");
    expect(panelSource).toContain("可重现 ≠ 模型有效");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeValidationEvaluationImplementationPanel");
    expect(validationEvaluationImplementationSource).toContain("第 59 阶段 · validation 评估实现登记");
    expect(validationEvaluationImplementationSource).toContain("先冻结规则，再看 validation");
    expect(validationEvaluationImplementationSource).toContain("registerHistoricalOutcomeValidationEvaluationImplementation");
    expect(validationEvaluationImplementationSource).toContain("component block bootstrap");
    expect(validationEvaluationImplementationSource).toContain("禁止挑 seed");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-validation-evaluation-implementations");
    expect(panelSource).toContain("59 validation 评估实现登记");
    expect(panelSource).toContain("当前无入口、无标签访问、无评估、无选模");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeValidationEvaluationImplementationReviewPanel");
    expect(validationEvaluationImplementationReviewSource).toContain("第 60 阶段 · validation 评估实现独立复核");
    expect(validationEvaluationImplementationReviewSource).toContain("独立复算，不接受勾选替代审计");
    expect(validationEvaluationImplementationReviewSource).toContain("reviewHistoricalOutcomeValidationEvaluationImplementation");
    expect(validationEvaluationImplementationReviewSource).toContain("10,000 次 component-block bootstrap");
    expect(validationEvaluationImplementationReviewSource).toContain("只开放未来隔离 runner 规格登记");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-validation-evaluation-implementation-reviews");
    expect(panelSource).toContain("60 validation 评估实现独立复核");
    expect(panelSource).toContain("批准仍无标签访问、评估、选模");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeValidationEvaluationIsolatedRunnerPanel");
    expect(validationEvaluationIsolatedRunnerSource).toContain("第 61 阶段 · validation 评估隔离 runner");
    expect(validationEvaluationIsolatedRunnerSource).toContain("登记不是执行");
    expect(validationEvaluationIsolatedRunnerSource).toContain("registerHistoricalOutcomeValidationEvaluationIsolatedRunner");
    expect(validationEvaluationIsolatedRunnerSource).toContain("sealed holdout 始终不可见");
    expect(validationEvaluationIsolatedRunnerSource).toContain("当前连这些挂载都不存在");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-validation-evaluation-isolated-runners");
    expect(panelSource).toContain("61 validation 评估隔离 runner 登记");
    expect(panelSource).toContain("登记不是运行");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationPanel");
    expect(validationEvaluationFirstExecutionAuthorizationSource).toContain("第 62 阶段 · validation 评估首次执行授权复核");
    expect(validationEvaluationFirstExecutionAuthorizationSource).toContain("授权不等于执行");
    expect(validationEvaluationFirstExecutionAuthorizationSource).toContain("reviewHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization");
    expect(validationEvaluationFirstExecutionAuthorizationSource).toContain("24 小时内有效，最多消费一次");
    expect(validationEvaluationFirstExecutionAuthorizationSource).toContain("sealed holdout features/labels 永久隐藏");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-validation-evaluation-first-execution-authorizations");
    expect(panelSource).toContain("62 validation 评估首次执行授权");
    expect(panelSource).toContain("本阶段没有 claim、标签挂载、评估、选模、输出");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeValidationEvaluationExecutionAttemptPanel");
    expect(validationEvaluationExecutionAttemptSource).toContain("第 63 阶段 · validation 评估一次性执行");
    expect(validationEvaluationExecutionAttemptSource).toContain("先写不可逆 claim");
    expect(validationEvaluationExecutionAttemptSource).toContain("getHistoricalOutcomeValidationEvaluationExecutionAttempts");
    expect(validationEvaluationExecutionAttemptSource).toContain("invokeHistoricalOutcomeValidationEvaluationOnce");
    expect(validationEvaluationExecutionAttemptSource).toContain("当前是进程内能力隔离，不是操作系统级沙箱");
    expect(validationEvaluationExecutionAttemptSource).toContain("逐目标验证 ≠ 正式选模");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-validation-evaluation-execution-attempts");
    expect(panelSource).toContain("63 validation 评估一次性执行");
    expect(panelSource).toContain("sealed holdout、全局有效性、正式选模");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeValidationEvaluationOutputValidationPanel");
    expect(validationEvaluationOutputValidationSource).toContain("第 64 阶段 · validation 评估输出独立复算");
    expect(validationEvaluationOutputValidationSource).toContain("getHistoricalOutcomeValidationEvaluationOutputValidations");
    expect(validationEvaluationOutputValidationSource).toContain("validateHistoricalOutcomeValidationEvaluationOutput");
    expect(validationEvaluationOutputValidationSource).toContain("独立复算 81 指标、54 检验与 9 建议");
    expect(validationEvaluationOutputValidationSource).toContain("sealed holdout");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-validation-evaluation-output-validations");
    expect(panelSource).toContain("64 validation 评估输出独立复算");
    expect(panelSource).toContain("通过仍不是正式选模");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionPanel");
    expect(validationEvaluationPerTargetCandidateAdmissionSource).toContain("第 65 阶段 · 逐目标候选准入复核");
    expect(validationEvaluationPerTargetCandidateAdmissionSource).toContain("九个目标，九道独立门");
    expect(validationEvaluationPerTargetCandidateAdmissionSource).toContain("getHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReviews");
    expect(validationEvaluationPerTargetCandidateAdmissionSource).toContain("reviewHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmission");
    expect(validationEvaluationPerTargetCandidateAdmissionSource).toContain("sealed holdout");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-validation-evaluation-per-target-candidate-admission-reviews");
    expect(panelSource).toContain("65 逐目标候选准入复核");
    expect(panelSource).toContain("不做综合分");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeSealedHoldoutEvaluationProtocolReviewPanel");
    expect(sealedHoldoutEvaluationProtocolReviewSource).toContain("第 66 阶段 · 封存样本评估协议独立复核");
    expect(sealedHoldoutEvaluationProtocolReviewSource).toContain("只冻结尺子，不打开试卷");
    expect(sealedHoldoutEvaluationProtocolReviewSource).toContain("getHistoricalOutcomeSealedHoldoutEvaluationProtocolReviews");
    expect(sealedHoldoutEvaluationProtocolReviewSource).toContain("reviewHistoricalOutcomeSealedHoldoutEvaluationProtocol");
    expect(sealedHoldoutEvaluationProtocolReviewSource).toContain("10,000");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-protocol-reviews");
    expect(panelSource).toContain("66 sealed-holdout 评估协议独立复核");
    expect(panelSource).toContain("本阶段不读取、挂载、解密、投影或执行 sealed holdout");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeSealedHoldoutEvaluationImplementationPanel");
    expect(sealedHoldoutEvaluationImplementationSource).toContain("第 67 阶段 · sealed-holdout 评估实现登记");
    expect(sealedHoldoutEvaluationImplementationSource).toContain("登记不是执行");
    expect(sealedHoldoutEvaluationImplementationSource).toContain("zero capability · no entrypoint");
    expect(sealedHoldoutEvaluationImplementationSource).toContain("getHistoricalOutcomeSealedHoldoutEvaluationImplementations");
    expect(sealedHoldoutEvaluationImplementationSource).toContain("registerHistoricalOutcomeSealedHoldoutEvaluationImplementation");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-implementations");
    expect(panelSource).toContain("67 sealed-holdout 评估实现登记");
    expect(panelSource).toContain("登记不是执行，没有入口、挂载、数据 adapter、留出集访问或评估授权");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeSealedHoldoutEvaluationImplementationReviewPanel");
    expect(sealedHoldoutEvaluationImplementationReviewSource).toContain("第 68 阶段 · sealed-holdout 评估实现独立复核");
    expect(sealedHoldoutEvaluationImplementationReviewSource).toContain("独立复算，不接受勾选替代审计");
    expect(sealedHoldoutEvaluationImplementationReviewSource).toContain("Stage 51–67");
    expect(sealedHoldoutEvaluationImplementationReviewSource).toContain("10,000");
    expect(sealedHoldoutEvaluationImplementationReviewSource).toContain("getHistoricalOutcomeSealedHoldoutEvaluationImplementationReviews");
    expect(sealedHoldoutEvaluationImplementationReviewSource).toContain("reviewHistoricalOutcomeSealedHoldoutEvaluationImplementation");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-implementation-reviews");
    expect(panelSource).toContain("68 sealed-holdout 评估实现独立复核");
    expect(panelSource).toContain("批准不创建 runner，不读取 sealed holdout，不评估、不选模、不交易");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerPanel");
    expect(sealedHoldoutEvaluationIsolatedRunnerSource).toContain("第 69 阶段 · sealed-holdout 评估隔离 runner");
    expect(sealedHoldoutEvaluationIsolatedRunnerSource).toContain("登记不是访问，也不是执行");
    expect(sealedHoldoutEvaluationIsolatedRunnerSource).toContain("17/29/43");
    expect(sealedHoldoutEvaluationIsolatedRunnerSource).toContain("getHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunners");
    expect(sealedHoldoutEvaluationIsolatedRunnerSource).toContain("registerHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunner");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-isolated-runners");
    expect(panelSource).toContain("69 sealed-holdout 评估隔离 runner 登记");
    expect(panelSource).toContain("登记不提供留出集访问、挂载或执行能力");
    expect(panelSource).toContain("historical_outcome_sealed_holdout_evaluation_isolated_runner_current_binding_count");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationPanel");
    expect(sealedHoldoutEvaluationFirstExecutionAuthorizationSource).toContain("第 70 阶段 · sealed-holdout 评估首次执行授权复核");
    expect(sealedHoldoutEvaluationFirstExecutionAuthorizationSource).toContain("24 小时内最多允许一次");
    expect(sealedHoldoutEvaluationFirstExecutionAuthorizationSource).toContain("Stage 51–69");
    expect(sealedHoldoutEvaluationFirstExecutionAuthorizationSource).toContain("getHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizations");
    expect(sealedHoldoutEvaluationFirstExecutionAuthorizationSource).toContain("reviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-first-execution-authorizations");
    expect(panelSource).toContain("70 sealed-holdout 首次访问与执行授权复核");
    expect(panelSource).toContain("审批接口没有 claim、挂载或执行入口");
    expect(panelSource).toContain("historical_outcome_sealed_holdout_evaluation_execution_attempt_eligible_count");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptPanel");
    expect(sealedHoldoutEvaluationExecutionAttemptSource).toContain("第 71 阶段 · sealed-holdout 一次性确认执行");
    expect(sealedHoldoutEvaluationExecutionAttemptSource).toContain("先写不可逆 claim");
    expect(sealedHoldoutEvaluationExecutionAttemptSource).toContain("getHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempts");
    expect(sealedHoldoutEvaluationExecutionAttemptSource).toContain("invokeHistoricalOutcomeSealedHoldoutEvaluationOnce");
    expect(sealedHoldoutEvaluationExecutionAttemptSource).toContain("不得反馈调参、重训、换种子");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-execution-attempts");
    expect(panelSource).toContain("71 sealed-holdout 一次性确认执行");
    expect(panelSource).toContain("成功、失败和中断都不能重放");
    expect(panelSource).toContain("historical_outcome_sealed_holdout_evaluation_independent_output_validation_eligible_count");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeSealedHoldoutEvaluationOutputValidationPanel");
    expect(sealedHoldoutEvaluationOutputValidationSource).toContain("第 72 阶段 · sealed-holdout 输出独立复算");
    expect(sealedHoldoutEvaluationOutputValidationSource).toContain("Stage 64 的独立实现");
    expect(sealedHoldoutEvaluationOutputValidationSource).toContain("getHistoricalOutcomeSealedHoldoutEvaluationOutputValidations");
    expect(sealedHoldoutEvaluationOutputValidationSource).toContain("validateHistoricalOutcomeSealedHoldoutEvaluationOutput");
    expect(sealedHoldoutEvaluationOutputValidationSource).toContain("未来确认结果裁决复核");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-output-validations");
    expect(panelSource).toContain("72 sealed-holdout 输出独立复算");
    expect(panelSource).toContain("通过只开放未来裁决复核");
    expect(panelSource).toContain("historical_outcome_future_confirmatory_result_adjudication_review_eligible_count");
    expect(governanceSource).toContain("PublicAdminHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationPanel");
    expect(sealedHoldoutConfirmatoryResultAdjudicationSource).toContain("第 73 阶段 · 确认结果独立裁决");
    expect(sealedHoldoutConfirmatoryResultAdjudicationSource).toContain("可复现 ≠ 有经济意义");
    expect(sealedHoldoutConfirmatoryResultAdjudicationSource).toContain("定量失败不得人工覆盖");
    expect(sealedHoldoutConfirmatoryResultAdjudicationSource).toContain("no_unconfirmed_hari_or_old_wang_logic_claimed");
    expect(sealedHoldoutConfirmatoryResultAdjudicationSource).toContain("getHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudications");
    expect(sealedHoldoutConfirmatoryResultAdjudicationSource).toContain("reviewHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudication");
    expect(apiSource).toContain("/historical-outcome-feature-label-join-target-training-sealed-holdout-evaluation-confirmatory-result-adjudications");
    expect(panelSource).toContain("73 确认结果独立裁决");
    expect(panelSource).toContain("historical_outcome_future_controlled_shadow_experiment_design_registration_eligible_count");
    expect(panelSource).toContain("④ 标签器实现");
    expect(panelSource).toContain("可送试运行授权复核");
    expect(panelSource).toContain("⑤ 封存行情输入");
    expect(panelSource).toContain("⑥ 试运行授权");
    expect(panelSource).toContain("⑦ 试运行实现");
    expect(panelSource).toContain("已登记未运行");
    expect(panelSource).toContain("⑧ 运行授权复核");
    expect(panelSource).toContain("可登记未来执行器");
    expect(panelSource).toContain("⑨ 隔离执行器规范");
    expect(panelSource).toContain("可送首次执行复核");
    expect(panelSource).toContain("⑱ 物化一次性执行");
    expect(panelSource).toContain("结果不是标签");
    expect(panelSource).toContain("⑲ 物化结果独立校验");
    expect(panelSource).toContain("结构、来源与位模式一致");
    expect(panelSource).toContain("通过仍不是正式结果标签");
    expect(panelSource).toContain("⑳ 正式标签写入授权复核");
    expect(panelSource).toContain("一次性额度有效 · 尚未写入");
    expect(panelSource).toContain("㉑ 正式原始结果标签写入");
    expect(panelSource).toContain("正式原始标签已写入 · 待独立准入校验");
    expect(panelSource).toContain("historical_outcome_formal_label_written_count");
    expect(panelSource).toContain("㉒ 正式标签独立校验与离线数据集候选准入");
    expect(panelSource).toContain("historical_outcome_formal_label_admitted_training_candidate_count");
    expect(panelSource).toContain("候选≠训练");
    expect(panelSource).toContain("㉓ 版本化离线历史结果数据集装配");
    expect(panelSource).toContain("historical_outcome_offline_dataset_current_binding_count");
    expect(panelSource).toContain("数据集≠训练");
    expect(panelSource).toContain("㉔ 离线数据集独立治理复核");
    expect(panelSource).toContain("historical_outcome_offline_dataset_governance_current_binding_approved_count");
    expect(panelSource).toContain("批准也不执行切分或特征连接");
    expect(panelSource).toContain("㉕ 不可变转换规范登记");
    expect(panelSource).toContain("historical_outcome_offline_dataset_transformation_spec_current_binding_registered_count");
    expect(panelSource).toContain("登记、独立复核和执行严格分开");
    expect(panelSource).toContain("登记记录没有通用代码入口、环境密钥或外部能力");
    expect(panelSource).toContain("㉖ 转换规范独立复核");
    expect(panelSource).toContain("historical_outcome_offline_dataset_transformation_spec_current_binding_approved_count");
    expect(panelSource).toContain("仅可登记未来隔离实现");
    expect(panelSource).toContain("㉗ 隔离转换实现规范登记");
    expect(panelSource).toContain("historical_outcome_offline_dataset_transformation_implementation_current_binding_count");
    expect(panelSource).toContain("登记记录没有可调用入口、环境继承、密钥、网络、工具或子进程");
    expect(panelSource).toContain("下一步只有独立实现复核");
    expect(panelSource).toContain("㉘ 隔离转换实现独立复核");
    expect(panelSource).toContain("historical_outcome_offline_dataset_transformation_implementation_current_binding_approved_count");
    expect(panelSource).toContain("仅可登记未来 runner 规范");
    expect(panelSource).toContain("批准也不登记 runner、不执行");
    expect(panelSource).toContain("㉙ 隔离转换 runner 规范登记");
    expect(panelSource).toContain("historical_outcome_offline_dataset_transformation_runner_current_binding_count");
    expect(panelSource).toContain("已登记未运行 · 等待首次执行复核");
    expect(panelSource).toContain("唯一下一门禁是独立首次执行授权复核");
    expect(panelSource).toContain("㉚ 隔离转换首次执行授权复核");
    expect(panelSource).toContain("historical_outcome_offline_dataset_transformation_execution_attempt_eligible_count");
    expect(panelSource).toContain("必须在第 31 阶段人工领取，失败也消费");
    expect(panelSource).toContain("㉛ 隔离转换一次性执行尝试");
    expect(panelSource).toContain("㉜ 离线转换输出独立重算");
    expect(panelSource).toContain("historical_outcome_offline_dataset_transformation_untrusted_candidate_envelope_count");
    expect(panelSource).toContain("候选不是正式 manifest、feature bundle 或训练输入");
    expect(panelSource).toContain("⑩ 首次执行授权");
    expect(panelSource).toContain("授权仅在 24 小时内提供一次执行额度");
    expect(panelSource).toContain("⑪ 单次隔离执行");
    expect(panelSource).toContain("未验证输出");
    expect(panelSource).toContain("⑫ 独立输出校验");
    expect(panelSource).toContain("⑬ 结果标签准入");
    expect(panelSource).toContain("⑭ 标签物化实现");
    expect(panelSource).toContain("⑮ 物化运行授权复核");
    expect(panelSource).toContain("仅批准登记 runner");
    expect(panelSource).toContain("⑯ 物化隔离 runner");
    expect(panelSource).toContain("没有调用入口，仍需独立首次执行授权复核");
    expect(panelSource).toContain("⑰ 物化首次执行授权");
    expect(panelSource).toContain("未过期额度");
    expect(panelSource).toContain("批准仅提供 24 小时内一次调用资格");
    expect(panelSource).toContain("不推断方向、评级、动作、仓位或奖励");
    expect(panelSource).toContain("幸存者偏差");
  });

  it("shows a sandboxed experiment registry without implying training has run", () => {
    expect(panelSource).toContain("离线实验注册表");
    expect(panelSource).toContain("getInvestmentCausalTrainingExperiments");
    expect(panelSource).toContain("冻结提示词基线");
    expect(panelSource).toContain("监督式因果分类");
    expect(panelSource).toContain("偏好学习关闭");
    expect(panelSource).toContain("RL 关闭");
    expect(panelSource).toContain("已登记未运行");
    expect(panelSource).toContain("封存测试集不可访问");
    expect(panelSource).toContain("drift_monitoring_protocol");
    expect(panelSource).toContain("契约变化或未来信息泄漏会立即停用组件");
  });

  it("preserves optimistic review identity and structured error attribution", () => {
    expect(buildDecisionReviewRequest(sample(), corrected)).toEqual({
      expected_review_id: "review-1",
      status: "corrected",
      thesis_verdict: "weakened",
      correction_note: "价值捕获被高估",
      corrected_action: "maintain",
      error_attributions: [
        {
          kind: "company_value_capture",
          severity: "material",
          explanation: "企业级份额证据不足",
          evidence_ids: [],
        },
      ],
    });
  });

  it("does not turn an accepted thesis into a hidden correction", () => {
    const request = buildDecisionReviewRequest(sample(), {
      ...corrected,
      mode: "accepted",
      verdict: "supported",
    });
    expect(request).toEqual({
      expected_review_id: "review-1",
      status: "accepted",
      thesis_verdict: "supported",
      error_attributions: [],
    });
  });

  it("requires a correction note and rejection evidence", () => {
    expect(decisionReviewDraftIsValid({ ...corrected, note: "" })).toBe(false);
    expect(
      decisionReviewDraftIsValid({
        ...corrected,
        mode: "rejected",
        errorExplanation: "",
      }),
    ).toBe(false);
    expect(decisionReviewDraftIsValid(corrected)).toBe(true);
  });

  it("never tunnels causal labels through the full-thesis review endpoint", () => {
    const causal = {
      ...corrected,
      mode: "accepted" as const,
      verdict: "supported" as const,
      // A stale client may still carry this legacy field at runtime. The
      // full-thesis request builder must ignore it; causal labels only use the
      // independent distilled-review endpoint.
      causalLinkReviews: [
        {
          driver_id: "pricing_margin",
          observation_id: "financial:SNDK:2026-07-31:gross-margin",
          verdict: "accepted" as const,
          effect: "supports" as const,
          explanation: " 毛利率是直接指标，但只部分支持定价权。 ",
        },
      ],
    };
    expect(buildDecisionReviewRequest(sample(), causal as DecisionReviewDraft)).not.toHaveProperty(
      "causal_link_reviews",
    );
    expect(decisionReviewDraftIsValid(causal)).toBe(true);
  });

  it("never allows a corrected, conflicted, or withdrawn claim to be accepted", () => {
    const observation: InvestmentCausalObservation = {
      observation_id: "claim-1",
      relationship: "structured_source_claim",
      label: "管理层指引主张",
      value: "收入增长 20%",
      as_of: "2026-08-12",
      source: "Company IR",
      source_url: "https://example.com/ir",
      source_tier: "company_primary",
      policy_status: "training_only_pending_human_review",
      claim: {
        claim_kind: "management_guidance",
        metric_id: "revenue_growth",
        metric_basis: "non-GAAP organic growth",
        period: "FY2026 Q3",
        numeric_value: 20,
        unit: "%",
        speaker: "CFO",
        source_event_id: "call-q3",
        source_document: "earnings_call_transcript",
        source_locator: "prepared remarks / CFO",
        quote_excerpt: "预计收入增长 20%",
        disposition: "active",
        lifecycle_status: "active",
        conflicting_claim_ids: [],
      },
    };
    expect(causalObservationCanBeAccepted(observation)).toBe(true);
    for (const lifecycle_status of ["superseded", "conflicted", "withdrawn"] as const) {
      expect(
        causalObservationCanBeAccepted({
          ...observation,
          claim: { ...observation.claim!, lifecycle_status },
        }),
      ).toBe(false);
    }
  });

  it("accepts only complete deterministic comparisons with two traceable sources", () => {
    const observation: InvestmentCausalObservation = {
      observation_id: "computed-change:sequential:inventory-q1:inventory-q2",
      relationship: "computed_comparison",
      label: "库存环比变化",
      value: "库存环比 +20.0%（100 → 120 USD millions）",
      as_of: "2026-05-01",
      source: "HONE 确定性计算（两期 SEC XBRL）",
      source_url: "https://www.sec.gov/filing-q2",
      source_tier: "deterministic_computation",
      policy_status: "training_only_pending_human_review",
      computed: {
        formula_version: "hone-sec-period-comparison-v1",
        comparison_kind: "sequential_quarter",
        metric_id: "inventory",
        metric_basis: "sec_xbrl:inventory:quarterly",
        current_claim_id: "inventory-q2",
        prior_claim_id: "inventory-q1",
        current_period: "FY2026 Q2",
        prior_period: "FY2026 Q1",
        current_numeric_value: 120,
        prior_numeric_value: 100,
        unit: "USD millions",
        change_percent: 20,
        current_published_at: "2026-05-01T13:00:00Z",
        prior_published_at: "2026-02-01T13:00:00Z",
        current_source_url: "https://www.sec.gov/filing-q2",
        prior_source_url: "https://www.sec.gov/filing-q1",
      },
    };
    expect(causalObservationCanBeAccepted(observation)).toBe(true);
    expect(causalObservationCanBeAccepted({
      ...observation,
      computed: { ...observation.computed!, prior_source_url: "" },
    })).toBe(false);
    expect(causalObservationCanBeAccepted({
      ...observation,
      computed: {
        ...observation.computed!,
        prior_claim_id: observation.computed!.current_claim_id,
      },
    })).toBe(false);
  });

  it("accepts only a fully traceable same-filing margin ratio", () => {
    const observation: InvestmentCausalObservation = {
      observation_id: "computed-ratio:gross_margin:gross-profit-q2:revenue-q2",
      relationship: "computed_ratio",
      label: "SEC 同期毛利率",
      value: "FY2026 Q2 45.00%",
      as_of: "2026-08-01",
      source: "HONE 确定性计算（同一 SEC filing）",
      source_url: "https://www.sec.gov/filing-q2",
      source_tier: "deterministic_computation",
      policy_status: "training_only_pending_human_review",
      ratio: {
        formula_version: "hone-sec-same-filing-ratio-v1",
        ratio_kind: "gross_margin",
        metric_id: "gross_margin",
        numerator_metric_id: "gross_profit",
        numerator_metric_basis: "US-GAAP:GrossProfit",
        numerator_claim_id: "gross-profit-q2",
        numerator_numeric_value: 90,
        denominator_metric_id: "revenue",
        denominator_metric_basis: "US-GAAP:RevenueFromContractWithCustomerExcludingAssessedTax",
        denominator_claim_id: "revenue-q2",
        denominator_numeric_value: 200,
        period: "FY2026 Q2",
        result_percent: 45,
        published_at: "2026-08-01T13:00:00Z",
        source_url: "https://www.sec.gov/filing-q2",
      },
    };
    expect(causalObservationCanBeAccepted(observation)).toBe(true);
    expect(causalObservationCanBeAccepted({
      ...observation,
      ratio: { ...observation.ratio!, denominator_numeric_value: 0 },
    })).toBe(false);
    expect(causalObservationCanBeAccepted({
      ...observation,
      computed: {
        formula_version: "hone-sec-period-comparison-v1",
        comparison_kind: "year_over_year",
        metric_id: "revenue",
        metric_basis: "x",
        current_claim_id: "a",
        prior_claim_id: "b",
        current_period: "FY2026 Q2",
        prior_period: "FY2025 Q2",
        current_numeric_value: 1,
        prior_numeric_value: 1,
        unit: "USD_millions",
        change_percent: 0,
        current_published_at: "2026-08-01T13:00:00Z",
        prior_published_at: "2025-08-01T13:00:00Z",
        current_source_url: "https://www.sec.gov/a",
        prior_source_url: "https://www.sec.gov/b",
      },
    })).toBe(false);
  });

  it("recomputes a margin trend before allowing human acceptance", () => {
    const ratio = {
      formula_version: "hone-sec-same-filing-ratio-v1",
      ratio_kind: "gross_margin" as const,
      metric_id: "gross_margin",
      numerator_metric_id: "gross_profit",
      numerator_metric_basis: "US-GAAP:GrossProfit",
      numerator_claim_id: "gross-profit-q2",
      numerator_numeric_value: 90,
      denominator_metric_id: "revenue",
      denominator_metric_basis: "US-GAAP:Revenue",
      denominator_claim_id: "revenue-q2",
      denominator_numeric_value: 200,
      period: "FY2026 Q2",
      result_percent: 45,
      published_at: "2026-08-01T13:00:00Z",
      source_url: "https://www.sec.gov/2026-q2",
    };
    const observation: InvestmentCausalObservation = {
      observation_id: "computed-ratio-trend:gross_margin:yoy:prior:current",
      relationship: "computed_ratio_trend",
      label: "SEC 毛利率同比变化",
      value: "+5.00 个百分点",
      as_of: "2026-08-01",
      source: "HONE 确定性计算（可比 SEC 利润率）",
      source_url: ratio.source_url,
      source_tier: "deterministic_computation",
      policy_status: "training_only_pending_human_review",
      ratio_trend: {
        formula_version: "hone-sec-margin-trend-v1",
        comparison_kind: "year_over_year",
        metric_id: "gross_margin",
        current: ratio,
        prior: {
          ...ratio,
          numerator_claim_id: "gross-profit-prior",
          denominator_claim_id: "revenue-prior",
          period: "FY2025 Q2",
          result_percent: 40,
          published_at: "2025-08-01T13:00:00Z",
          source_url: "https://www.sec.gov/2025-q2",
        },
        change_percentage_points: 5,
      },
    };
    expect(causalObservationCanBeAccepted(observation)).toBe(true);
    expect(causalObservationCanBeAccepted({
      ...observation,
      ratio_trend: { ...observation.ratio_trend!, change_percentage_points: 6 },
    })).toBe(false);
  });

  it("accepts only an active issuer-defined operating KPI with verbatim trace", () => {
    const observation: InvestmentCausalObservation = {
      observation_id: "asp-q4",
      relationship: "operating_kpi_claim",
      label: "NAND ASP",
      value: "NAND average selling price increased 15% sequentially",
      as_of: "2026-08-06",
      source: "Sandisk investor relations",
      source_url: "https://investor.sandisk.com/q4-call",
      source_tier: "company_primary",
      policy_status: "training_only_pending_human_review",
      operating_kpi: {
        schema_version: "hone-operating-kpi-claim-v1",
        claim_kind: "reported_fact",
        kpi_id: "nand_asp_change",
        issuer_metric_name: "NAND ASP",
        issuer_definition: "NAND average selling price",
        definition_key: "nandaveragesellingprice",
        period: "FY2026 Q4",
        numeric_value: 15,
        unit: "%",
        value_text: "NAND average selling price increased 15% sequentially",
        measurement_scope: "company NAND realized price; sequential quarter",
        comparison_basis: "sequential_quarter",
        speaker: "CFO",
        source_event_id: "sndk-q4-call",
        source_document: "earnings_call_transcript",
        source_locator: "prepared remarks / CFO",
        evidence_quote: "NAND average selling price increased 15% sequentially",
        definition_changed: false,
        disposition: "active",
        lifecycle_status: "active",
        conflicting_claim_ids: [],
      },
    };
    expect(causalObservationCanBeAccepted(observation)).toBe(true);
    expect(causalObservationCanBeAccepted({
      ...observation,
      operating_kpi: {
        ...observation.operating_kpi!,
        issuer_definition: "industry spot price",
      },
    })).toBe(false);
    expect(causalObservationCanBeAccepted({
      ...observation,
      operating_kpi: {
        ...observation.operating_kpi!,
        lifecycle_status: "conflicted",
        conflicting_claim_ids: ["asp-other"],
      },
    })).toBe(false);
  });

  it("shows the immutable primary-source artifact audit trail when available", () => {
    expect(panelSource).toContain("来源时间精度");
    expect(panelSource).toContain("仅日期，按当日末保守入库");
    expect(panelSource).toContain("原文文件哈希");
    expect(panelSource).toContain("source_sha256");
    expect(panelSource).toContain("归档对象");
  });

  it("shows Stage 97 as a zero-capability contract pending independent implementation review", () => {
    expect(panelSource).toContain("97 行情解析器零能力实现契约登记");
    expect(panelSource).toContain("等待 Stage 98 独立实现复核");
    expect(panelSource).toContain("没有工件、entrypoint、runtime、原始载荷读取");
  });

  it("shows Stage 108 as a chain-external review with no observation authority", () => {
    expect(panelSource).toContain("108 观察物化实现责任链外独立复核");
    expect(panelSource).toContain("完整责任链外的新角色独立重算");
    expect(panelSource).toContain("Stage 109 候选");
    expect(panelSource).toContain("批准也不产生 runner、输入读取、观察、账本、持仓、绩效或交易权限");
  });

  it("shows Stage 109 as a proposed-artifact runner specification with no execution authority", () => {
    expect(panelSource).toContain("109 观察物化隔离 runner 规格登记");
    expect(panelSource).toContain("拟议工件摘要、不可变代码版本、固定非特权 runtime");
    expect(panelSource).toContain("Stage 110 候选");
    expect(panelSource).toContain("拟议工件不存在，没有入口、runtime、输入读取、观察、账本、持仓、绩效或交易权限");
  });

  it("shows Stage 110 as a server-rehashed one-shot authorization review without execution", () => {
    expect(panelSource).toContain("110 观察物化首次执行授权独立复核");
    expect(panelSource).toContain("内容寻址保管目录只读重算 runner 工件与自哈希 manifest");
    expect(panelSource).toContain("historical_outcome_observation_materialization_future_claim_first_attempt_eligible_count");
    expect(panelSource).toContain("本阶段不 claim、不启动 runtime、不读取 Stage 104 输入");
  });

  it("shows Stage 111 as a permanent claim-first authorization consumption gate", () => {
    expect(panelSource).toContain("111 观察物化单次尝试 claim-first 声明");
    expect(panelSource).toContain("historical_outcome_observation_materialization_execution_attempt_claim_count");
    expect(panelSource).toContain("声明失败或未来执行失败都不会返还授权");
    expect(panelSource).toContain("本阶段没有执行入口、输入读取、观察输出");
  });

  it("shows Stage 112 as one-shot deterministic materialization awaiting validation", () => {
    expect(panelSource).toContain("112 自然前瞻观察单次物化");
    expect(panelSource).toContain("historical_outcome_observation_materialization_execution_successful_untrusted_observation_count");
    expect(panelSource).toContain("重哈希声明式工件与 exact Stage 104 admitted output");
    expect(panelSource).toContain("待 Stage 113");
    expect(panelSource).toContain("没有账本、持仓、绩效、模型/训练、奖励、订单、券商或交易权限");
  });

  it("shows Stage 113 as a chain-external full reprojection before evidence admission", () => {
    expect(panelSource).toContain("113 观察物化输出责任链外独立校验");
    expect(panelSource).toContain("historical_outcome_observation_materialization_independently_validated_observation_count");
    expect(panelSource).toContain("不调用 Stage 112 materializer helper");
    expect(panelSource).toContain("Stage 114 候选");
    expect(panelSource).toContain("通过只开放 Stage 114 证据准入复核");
  });

  it("shows Stage 114 as a separate immutable evidence admission before any ledger", () => {
    expect(panelSource).toContain("114 观察证据责任链外独立准入");
    expect(panelSource).toContain("historical_outcome_observation_evidence_admitted_count");
    expect(panelSource).toContain("原 envelope 不改写，供应商发布时间仍未验证");
    expect(panelSource).toContain("批准只开放 Stage 115 账本转换规格登记");
    expect(panelSource).toContain("不建账、不算净值/绩效、不训练/RL、不交易");
  });

  it("shows Stage 115 as a zero-capability accounting specification with an explicit opening-state gap", () => {
    expect(panelSource).toContain("115 观察证据到账本转换规格登记");
    expect(panelSource).toContain("Stage 88 不是 opening positions");
    expect(panelSource).toContain("不得默认本金、现金、持仓或股数");
    expect(panelSource).toContain("historical_outcome_observation_ledger_transition_specification_registered_count");
    expect(panelSource).toContain("当前没有实现、账本事件、持仓、现金、净值/绩效");
  });

  it("shows Stage 116 as a second-implementation review that opens only Stage 117", () => {
    expect(panelSource).toContain("116 账本转换规格责任链外独立复核");
    expect(panelSource).toContain("不调用 Stage 115 builder");
    expect(panelSource).toContain("historical_outcome_observation_ledger_transition_specification_independently_approved_count");
    expect(panelSource).toContain("批准也不创建实现、账本事件、持仓、现金、NAV/绩效");
  });

  it("shows Stage 117 as a zero-capability contract that opens only Stage 118", () => {
    expect(panelSource).toContain("117 账本转换零能力实现合同");
    expect(panelSource).toContain("opening portfolio 门槛");
    expect(panelSource).toContain("exact decimal、append-only、幂等事件、双重记账");
    expect(panelSource).toContain("historical_outcome_observation_ledger_transition_implementation_future_stage_118_independent_review_eligible_count");
    expect(panelSource).toContain("没有源码、入口、runtime、输入读取、账本/事件、持仓、现金、NAV/绩效");
  });

  it("shows Stage 118 independent review and Stage 119 isolated runner registration", () => {
    expect(panelSource).toContain("118 账本转换实现责任链外独立复核");
    expect(panelSource).toContain("119 账本转换隔离 runner 规格登记");
    expect(panelSource).toContain("未来工件哈希、不可变代码版本、复现步骤");
    expect(panelSource).toContain("historical_outcome_observation_ledger_transition_isolated_runner_future_stage_120_first_execution_authorization_review_eligible_count");
    expect(panelSource).toContain("期初组合不存在，金融事件白名单为空");
  });

  it("shows Stage 120 as server-rehashed one-shot authorization without financial state", () => {
    expect(panelSource).toContain("120 账本转换首次执行授权独立复核");
    expect(panelSource).toContain("只读常规工件和自哈希 manifest");
    expect(panelSource).toContain("historical_outcome_observation_ledger_transition_first_execution_authorization_future_stage_121_claim_first_attempt_eligible_count");
    expect(panelSource).toContain("未来最多只允许非金融通知候选");
  });

  it("shows Stage 121 claim through Stage 124 non-financial evidence admission", () => {
    expect(panelSource).toContain("121 账本转换执行尝试原子认领");
    expect(panelSource).toContain("create-once 自哈希记录永久消费一次性授权");
    expect(panelSource).toContain("historical_outcome_observation_ledger_transition_execution_attempt_claim_waiting_for_stage_122_execution_count");
    expect(panelSource).toContain("122 非财务观察通知单次转换");
    expect(panelSource).toContain("historical_outcome_observation_ledger_transition_execution_successful_untrusted_candidate_count");
    expect(panelSource).toContain("不创建 ledger event、持仓、现金、NAV/绩效");
    expect(panelSource).toContain("123 非财务候选责任链外独立校验");
    expect(panelSource).toContain("historical_outcome_observation_ledger_transition_independently_validated_candidate_count");
    expect(panelSource).toContain("独立重开 Stage 122 候选和 exact Stage 114 观察证据");
    expect(panelSource).toContain("通过后仍未受信");
    expect(panelSource).toContain("124 正式非财务观察证据独立准入");
    expect(panelSource).toContain("historical_outcome_observation_ledger_transition_admitted_non_financial_observation_evidence_count");
    expect(panelSource).toContain("只治理外部来源期初组合快照");
  });

  it("shows Stage 125 external-source opening portfolio governance without financial state", () => {
    expect(panelSource).toContain("125 外部来源期初组合快照治理规格");
    expect(panelSource).toContain("historical_outcome_opening_portfolio_snapshot_governance_registered_specification_count");
    expect(panelSource).toContain("现金、持仓、负债、未结算活动");
    expect(panelSource).toContain("当前不接收来源文件、不手填余额、不生成期初快照");
    expect(panelSource).toContain("Stage 126 候选");
  });

  it("shows Stage 126 chain-external review without accepting a source artifact", () => {
    expect(panelSource).toContain("126 期初组合治理规格责任链外独立复核");
    expect(panelSource).toContain("不调用 Stage 125 构造器");
    expect(panelSource).toContain("historical_outcome_opening_portfolio_snapshot_governance_specification_independently_approved_count");
    expect(panelSource).toContain("Stage 127 候选");
    expect(panelSource).toContain("批准也不接收或读取来源文件");
  });

  it("shows Stage 127 as a zero-capability private receipt contract", () => {
    expect(panelSource).toContain("127 来源工件接收零能力实现登记");
    expect(panelSource).toContain("流式摘要与长度");
    expect(panelSource).toContain("账号匿名化、日志脱敏、内容寻址、失败清理");
    expect(panelSource).toContain("historical_outcome_opening_portfolio_source_artifact_receipt_implementation_contract_count");
    expect(panelSource).toContain("Stage 128 候选");
    expect(panelSource).toContain("没有上传入口、来源字节或 parser");
  });

  it("shows Stage 128 chain-external implementation review without source data authority", () => {
    expect(panelSource).toContain("128 来源工件接收实现责任链外独立复核");
    expect(panelSource).toContain("不调用 Stage 127 builder");
    expect(panelSource).toContain("historical_outcome_opening_portfolio_source_artifact_receipt_implementation_independently_approved_count");
    expect(panelSource).toContain("Stage 129 候选");
    expect(panelSource).toContain("仍不得上传、读取或解析来源文件");
  });

  it("shows Stage 129 as a proposed-artifact isolated receiver specification", () => {
    expect(panelSource).toContain("129 隔离来源工件接收器规格登记");
    expect(panelSource).toContain("historical_outcome_opening_portfolio_source_artifact_receipt_isolated_receiver_current_binding_count");
    expect(panelSource).toContain("Stage 130 候选");
    expect(panelSource).toContain("当前无上传、来源字节、工件、入口、runtime 或财务状态");
  });

  it("shows Stage 130 as a server-rehashed single-use authorization gate", () => {
    expect(panelSource).toContain("130 来源接收器首次执行授权");
    expect(panelSource).toContain("服务端重哈希只读工件");
    expect(panelSource).toContain("historical_outcome_opening_portfolio_source_artifact_receipt_first_execution_authorization_artifact_verified_count");
    expect(panelSource).toContain("Stage 131 候选");
    expect(panelSource).toContain("不接收来源文件、不启动 runtime");
  });

  it("shows Stage 132 as encrypted custody with an untrusted receipt", () => {
    expect(panelSource).toContain("132 来源工件单次加密接收");
    expect(panelSource).toContain("historical_outcome_opening_portfolio_source_artifact_receipt_execution_encryption_key_configured");
    expect(panelSource).toContain("原文件加密内容寻址保存");
    expect(panelSource).toContain("不是期初持仓");
  });

  it("shows Stage 133 as independent integrity validation without holdings truth", () => {
    expect(panelSource).toContain("133 加密 receipt 责任链外独立验证");
    expect(panelSource).toContain("historical_outcome_opening_portfolio_source_artifact_receipt_validation_independently_validated_receipt_count");
    expect(panelSource).toContain("不证明文件内持仓真实");
    expect(panelSource).toContain("不解析金融行");
  });

  it("shows Stage 134 as a zero-capability materializer contract, not holdings", () => {
    expect(panelSource).toContain("134 期初快照物化零能力实现登记");
    expect(panelSource).toContain("historical_outcome_opening_portfolio_snapshot_materialization_implementation_current_binding_count");
    expect(panelSource).toContain("精确十进制");
    expect(panelSource).toContain("不解密、不解析、不生成候选或真实持仓");
  });

  it("shows Stage 135 as an independent implementation review with no materialization authority", () => {
    expect(panelSource).toContain("135 物化实现责任链外独立审查");
    expect(panelSource).toContain("historical_outcome_opening_portfolio_snapshot_materialization_implementation_review_independently_approved_count");
    expect(panelSource).toContain("第二实现独立重建");
    expect(panelSource).toContain("Stage 136 候选");
    expect(panelSource).toContain("当前仍无 receipt 读取、解密、parser/runtime、期初快照、财务状态、训练、订单或交易权限");
  });
});
