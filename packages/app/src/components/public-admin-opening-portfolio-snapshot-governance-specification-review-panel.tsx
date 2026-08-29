import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getOpeningPortfolioSnapshotGovernanceSpecificationReviews,
  reviewOpeningPortfolioSnapshotGovernanceSpecification,
} from "@/lib/api";
import type {
  OpeningPortfolioSnapshotGovernanceSpecificationReviewRegistry,
  OpeningPortfolioSnapshotGovernanceSpecificationReviewVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–125 完整责任链",
  "复核人独立于 Stage 125 登记人和全部既有责任人",
  "已用独立路径重算登记与规格摘要",
  "完整规格由第二实现重建，未调用 Stage 125 构造器自证",
  "独立重建结果与已登记规格逐字段一致",
  "原始来源、内容摘要、来源身份和账户匿名化合同完整",
  "账户、现金、持仓、上市期权、负债和未结算活动必须完整",
  "精确十进制、有符号数量，不补值、不推断、不允许部分准入",
  "证券身份、成本基础与公司行动对账合同完整",
  "对账单市值只作参考；净值前必须有独立行情、汇率与衍生品估值",
  "来源接收、快照物化、输出验证与快照准入仍是分离关卡",
  "本阶段没有上传、读取、解析文件，没有 runtime、快照或财务状态",
  "没有账本、持仓、现金、净值/绩效、模型、训练/RL、订单、券商或交易权限",
  "批准只开放 Stage 127 零能力来源工件接收实现登记",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminOpeningPortfolioSnapshotGovernanceSpecificationReviewPanel() {
  const [registry, setRegistry] = createSignal<OpeningPortfolioSnapshotGovernanceSpecificationReviewRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [verdict, setVerdict] = createSignal<OpeningPortfolioSnapshotGovernanceSpecificationReviewVerdict>(
    "approved_for_future_zero_capability_source_artifact_receipt_implementation_registration",
  );
  const [rationale, setRationale] = createSignal("");
  const [binding, setBinding] = createSignal("");
  const [source, setSource] = createSignal("");
  const [completeness, setCompleteness] = createSignal("");
  const [valuation, setValuation] = createSignal("");
  const [zeroCapability, setZeroCapability] = createSignal("");
  const [limitations, setLimitations] = createSignal("尚未接收、读取或解析任何券商/托管来源文件，也没有期初组合快照。");
  const [constraints, setConstraints] = createSignal("Stage 127 只能登记零能力来源工件接收实现；不得直接上传文件或生成持仓、现金和净值。");
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const eligible = createMemo(() => registry()?.items.filter((item) => item.review_eligible) ?? []);
  const selected = createMemo(() => eligible().find((item) => item.registration.registration_id === selectedId()));

  const load = async () => {
    try {
      const next = await getOpeningPortfolioSnapshotGovernanceSpecificationReviews();
      setRegistry(next);
      const candidates = next.items.filter((item) => item.review_eligible);
      if (!candidates.some((item) => item.registration.registration_id === selectedId())) {
        setSelectedId(candidates[0]?.registration.registration_id ?? "");
      }
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 126 独立规格复核读取失败");
    }
  };
  onMount(() => void load());

  const disabled = createMemo(() => busy() || !selected() || !rationale().trim() || !binding().trim()
    || !source().trim() || !completeness().trim() || !valuation().trim() || !zeroCapability().trim()
    || !limitations().trim() || !constraints().trim()
    || (verdict().startsWith("approved") && !checks().every(Boolean)));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    setBusy(true); setError(""); setNotice("");
    try {
      const values = checks();
      const latest = item.latest_review;
      const next = await reviewOpeningPortfolioSnapshotGovernanceSpecification(
        item.registration.registration_id,
        {
          expected_previous_review_id: latest?.review_id,
          expected_previous_review_sha256: latest?.review_sha256,
          expected_registration_sha256: item.registration.registration_sha256,
          expected_specification_sha256: item.registration.specification.specification_sha256,
          expected_independent_audit_sha256: item.current_independent_audit.audit_sha256,
          verdict: verdict(), rationale: rationale().trim(),
          binding_and_second_implementation_assessment: binding().trim(),
          source_artifact_and_identity_assessment: source().trim(),
          account_scope_and_snapshot_completeness_assessment: completeness().trim(),
          valuation_and_nav_prerequisite_assessment: valuation().trim(),
          zero_capability_assessment: zeroCapability().trim(),
          known_limitations: limitations().trim(), future_implementation_constraints: constraints().trim(),
          exact_current_stage_51_through_stage_125_binding_confirmed: values[0] as boolean,
          reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: values[1] as boolean,
          registration_and_specification_hashes_independently_reproduced_confirmed: values[2] as boolean,
          complete_specification_rebuilt_without_stage_125_builder_confirmed: values[3] as boolean,
          rebuilt_specification_exactly_matches_registered_specification_confirmed: values[4] as boolean,
          original_external_artifact_provenance_and_pseudonymization_contract_confirmed: values[5] as boolean,
          complete_accounts_cash_positions_options_liabilities_and_unsettled_activity_confirmed: values[6] as boolean,
          exact_decimal_signed_quantity_no_default_inference_or_partial_admission_confirmed: values[7] as boolean,
          instrument_identity_cost_basis_and_corporate_action_contract_confirmed: values[8] as boolean,
          statement_values_informational_and_independent_marks_fx_derivatives_required_confirmed: values[9] as boolean,
          source_receipt_snapshot_materialization_output_validation_and_admission_remain_separate_confirmed: values[10] as boolean,
          no_artifact_upload_read_parser_runtime_snapshot_or_financial_state_confirmed: values[11] as boolean,
          no_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: values[12] as boolean,
          approval_only_opens_future_zero_capability_source_receipt_implementation_registration_confirmed: values[13] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: values[14] as boolean,
        },
      );
      setRegistry(next); setRationale(""); setChecks(REVIEW_CHECKS.map(() => false));
      setNotice(verdict().startsWith("approved")
        ? "Stage 126 已独立批准；当前仍没有来源文件或期初组合，只开放 Stage 127 零能力实现登记。"
        : "复核意见已追加保存；原规格没有取得 Stage 127 资格。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 126 独立规格复核提交失败");
      await load();
    } finally { setBusy(false); }
  };

  return <Show when={registry()}>{(current) => <section class="public-admin-reward-governance" aria-label="期初组合快照治理规格独立复核">
    <header><strong>第 126 阶段 · 期初组合治理规格独立复核</strong><span>第二实现 · 不接数据</span></header>
    <p>{current().scope}</p>
    <p class="public-admin-anchor-boundary">当前明确为空：来源文件、parser/runtime、期初组合、金融事件、账本、持仓、现金、净值、绩效与交易权限。</p>
    <div class="public-admin-decision-metrics">
      <div><span>待复核</span><strong>{current().review_eligible_count}</strong></div>
      <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
      <div><span>独立批准</span><strong>{current().independently_approved_count}</strong></div>
      <div><span>Stage 127 候选</span><strong>{current().future_stage_127_zero_capability_source_artifact_receipt_implementation_registration_eligible_count}</strong></div>
    </div>
    <Show when={eligible().length > 0} fallback={<p>当前没有待复核的 Stage 125 规格。</p>}>
      <label><span>Stage 125 规格</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}>
        <For each={eligible()}>{(item) => <option value={item.registration.registration_id}>{item.registration.specification.source_contract.source_provider_name} · {item.registration.registration_id.slice(0, 12)}…</option>}</For>
      </select></label>
      <label><span>裁决</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as OpeningPortfolioSnapshotGovernanceSpecificationReviewVerdict)}>
        <option value="approved_for_future_zero_capability_source_artifact_receipt_implementation_registration">批准进入 Stage 127 零能力实现登记</option>
        <option value="changes_required_rebuild_opening_portfolio_governance_specification">要求重建 Stage 125 规格</option>
        <option value="rejected_opening_portfolio_governance_specification">拒绝该规格</option>
      </select></label>
      <label><span>复核理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
      <label><span>责任链与第二实现</span><textarea value={binding()} onInput={(event) => setBinding(event.currentTarget.value)} /></label>
      <label><span>来源工件与证券身份</span><textarea value={source()} onInput={(event) => setSource(event.currentTarget.value)} /></label>
      <label><span>账户范围与快照完整性</span><textarea value={completeness()} onInput={(event) => setCompleteness(event.currentTarget.value)} /></label>
      <label><span>估值与净值前置条件</span><textarea value={valuation()} onInput={(event) => setValuation(event.currentTarget.value)} /></label>
      <label><span>零能力边界</span><textarea value={zeroCapability()} onInput={(event) => setZeroCapability(event.currentTarget.value)} /></label>
      <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
      <label><span>未来实现约束</span><textarea value={constraints()} onInput={(event) => setConstraints(event.currentTarget.value)} /></label>
      <div class="public-admin-decision-checks"><For each={REVIEW_CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
      <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在追加复核…" : "提交 Stage 126 独立复核"}</button>
    </Show>
    <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
    <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
  </section>}</Show>;
}
