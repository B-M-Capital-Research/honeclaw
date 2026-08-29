import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationMaterializationSpecificationReviews,
  reviewControlledShadowObservationMaterializationSpecification,
} from "@/lib/api";
import type {
  ControlledShadowObservationMaterializationSpecificationReviewRegistry,
  ControlledShadowObservationMaterializationSpecificationReviewVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–105 完整责任链",
  "复核者独立于 Stage 105 登记者和全部既有责任人",
  "独立重算 registration 与 specification 摘要",
  "未调用 Stage 105 builder，从当前 Stage 104 源完整重建规格",
  "第二实现重建结果与已登记规格逐字段一致",
  "官方交易日、标的、SPY 和三价格口径矩阵保持一致",
  "标的缺口显式；SPY 缺失、重复和越界失败关闭",
  "分红、拆股和三价格口径继续分开",
  "原始十进制、排序、逐行哈希和内容寻址路径保持一致",
  "初始影子组合只绑定，不重算或执行会计转换",
  "保守 available_at 和供应商发布时间未验证限制保持一致",
  "单 envelope、create-once，禁止覆盖、回填、填充、插值或替代",
  "未来输出仍不可信且必须独立校验",
  "没有实现、工件、入口、runtime、挂载、环境、密钥、网络、工具、子进程或生产 I/O",
  "不生成观察、账本、持仓、绩效、模型、训练、reward、订单、券商或交易",
  "批准只开放未来 Stage 107 零能力实现登记",
  "未把未确认的 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationMaterializationSpecificationReviewPanel() {
  const [registry, setRegistry] = createSignal<ControlledShadowObservationMaterializationSpecificationReviewRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [verdict, setVerdict] = createSignal<ControlledShadowObservationMaterializationSpecificationReviewVerdict>(
    "approved_for_future_zero_capability_observation_materialization_implementation_registration",
  );
  const [rationale, setRationale] = createSignal("");
  const [binding, setBinding] = createSignal("");
  const [matrix, setMatrix] = createSignal("");
  const [actions, setActions] = createSignal("");
  const [allocation, setAllocation] = createSignal("");
  const [zeroCapability, setZeroCapability] = createSignal("");
  const [limitations, setLimitations] = createSignal("供应商发布时间仍未验证；本阶段没有真实观察或自然前向绩效。");
  const [constraints, setConstraints] = createSignal("Stage 107 只能登记零能力实现合同；不得携带工件、入口、runtime、输入挂载或执行权限。");
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationMaterializationSpecificationReviews();
      setRegistry(next);
      if (!next.items.some((item) => item.registration.registration_id === selectedId() && item.review_eligible)) {
        setSelectedId(next.items.find((item) => item.review_eligible)?.registration.registration_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 106 规格独立复核表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.items.find(
    (item) => item.registration.registration_id === selectedId() && item.review_eligible,
  ));
  const isApproval = createMemo(() => verdict()
    === "approved_for_future_zero_capability_observation_materialization_implementation_registration");
  const disabled = createMemo(() => busy() || !selected() || !rationale().trim()
    || !binding().trim() || !matrix().trim() || !actions().trim() || !allocation().trim()
    || !zeroCapability().trim() || !limitations().trim() || !constraints().trim()
    || (isApproval() && !checks().every(Boolean)));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const previous = item.latest_review;
    const values = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewControlledShadowObservationMaterializationSpecification(
        item.registration.registration_id,
        {
          expected_previous_review_id: previous?.review_id ?? null,
          expected_previous_review_sha256: previous?.review_sha256 ?? null,
          expected_registration_sha256: item.registration.registration_sha256,
          expected_specification_sha256: item.registration.specification.specification_sha256,
          expected_independent_audit_sha256: item.current_independent_audit.audit_sha256,
          verdict: verdict(),
          rationale: rationale().trim(),
          binding_and_second_implementation_assessment: binding().trim(),
          session_price_basis_and_gap_assessment: matrix().trim(),
          corporate_action_decimal_order_and_hash_assessment: actions().trim(),
          initial_allocation_and_availability_assessment: allocation().trim(),
          zero_capability_assessment: zeroCapability().trim(),
          known_limitations: limitations().trim(),
          future_implementation_constraints: constraints().trim(),
          exact_current_stage_51_through_stage_105_binding_confirmed: values[0] as boolean,
          reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: values[1] as boolean,
          registration_and_specification_hashes_independently_reproduced_confirmed: values[2] as boolean,
          complete_specification_rebuilt_from_current_stage_104_source_without_stage_105_builder_confirmed: values[3] as boolean,
          rebuilt_specification_exactly_matches_registered_specification_confirmed: values[4] as boolean,
          official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: values[5] as boolean,
          subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: values[6] as boolean,
          dividends_splits_and_price_bases_remain_separate_confirmed: values[7] as boolean,
          decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: values[8] as boolean,
          initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed: values[9] as boolean,
          conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: values[10] as boolean,
          one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: values[11] as boolean,
          future_output_untrusted_and_independent_validation_required_confirmed: values[12] as boolean,
          no_implementation_artifact_entrypoint_runtime_mount_environment_secret_network_tool_subprocess_or_production_io_confirmed: values[13] as boolean,
          no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: values[14] as boolean,
          approval_only_opens_future_zero_capability_implementation_registration_confirmed: values[15] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: values[16] as boolean,
        },
      );
      setRegistry(next);
      setNotice(isApproval()
        ? "Stage 106 独立复核已批准；仅开放未来 Stage 107 零能力实现登记，观察仍未生成。"
        : "Stage 106 复核已不可变记录；后续晋级保持失败关闭。");
      setRationale("");
      setBinding("");
      setMatrix("");
      setActions("");
      setAllocation("");
      setZeroCapability("");
      setChecks(REVIEW_CHECKS.map(() => false));
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 106 规格独立复核提交失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="首次自然前向周期观察物化规格独立复核">
      <header><strong>第 106 阶段 · 物化规格独立复核</strong><span>第二实现 · 责任链外</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>规格</span><strong>{current().specification_count}</strong></div>
        <div><span>待复核</span><strong>{current().review_eligible_count}</strong></div>
        <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
        <div><span>独立批准</span><strong>{current().independently_approved_count}</strong></div>
      </div>
      <p class="public-admin-anchor-boundary">独立审计必须重建整份规格；不能只比较 Stage 105 自报的布尔位或摘要。</p>
      <Show when={current().items.some((item) => item.review_eligible)} fallback={<p>当前没有待复核的 Stage 105 规格。</p>}>
        <label><span>待复核规格</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}>
          <For each={current().items.filter((item) => item.review_eligible)}>{(item) => <option value={item.registration.registration_id}>
            {item.registration.specification.subject_symbols.join(", ")} · {item.registration.registration_sha256.slice(0, 12)}…
          </option>}</For>
        </select></label>
        <Show when={selected()}>{(item) => <article class="public-admin-reward-governance">
          <header><strong>第二实现审计</strong><span>{item().current_independent_audit.mismatch_reasons.length === 0 ? "结构一致" : "失败关闭"}</span></header>
          <p>registration {item().current_independent_audit.registration_hash_independently_reproduced ? "已复算" : "不一致"} · specification {item().current_independent_audit.specification_hash_independently_reproduced ? "已复算" : "不一致"}</p>
          <p>完整重建 {item().current_independent_audit.rebuilt_specification_exactly_matches_registration ? "逐字段一致" : "不一致"} · 零权限 {item().current_independent_audit.all_implementation_runtime_observation_store_feedback_order_broker_and_trading_authority_closed ? "关闭" : "异常"}</p>
        </article>}</Show>
        <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ControlledShadowObservationMaterializationSpecificationReviewVerdict)}>
          <option value="approved_for_future_zero_capability_observation_materialization_implementation_registration">批准进入 Stage 107 零能力实现登记</option>
          <option value="changes_required_rebuild_observation_materialization_specification">要求重建规格</option>
          <option value="rejected_observation_materialization_specification">拒绝规格</option>
        </select></label>
        <label><span>复核理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
        <label><span>绑定与第二实现评估</span><textarea value={binding()} onInput={(event) => setBinding(event.currentTarget.value)} /></label>
        <label><span>交易日、价格口径与缺口评估</span><textarea value={matrix()} onInput={(event) => setMatrix(event.currentTarget.value)} /></label>
        <label><span>公司行动、十进制、排序与哈希评估</span><textarea value={actions()} onInput={(event) => setActions(event.currentTarget.value)} /></label>
        <label><span>初始组合与可用时间评估</span><textarea value={allocation()} onInput={(event) => setAllocation(event.currentTarget.value)} /></label>
        <label><span>零能力评估</span><textarea value={zeroCapability()} onInput={(event) => setZeroCapability(event.currentTarget.value)} /></label>
        <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
        <label><span>未来实现约束</span><textarea value={constraints()} onInput={(event) => setConstraints(event.currentTarget.value)} /></label>
        <div class="public-admin-decision-checks"><For each={REVIEW_CHECKS}>{(label, index) => <label><input
          type="checkbox" checked={checks()[index()]}
          onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))}
        /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
          {busy() ? "正在写入不可变复核…" : "提交 Stage 106 独立复核"}
        </button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().items.filter((item) => item.latest_review)}>{(item) => <article class="public-admin-reward-governance">
        <header><strong>{item.latest_review?.specification_independently_approved ? "规格已独立批准" : "规格复核未通过"}</strong><span>{item.latest_review?.submitted_at}</span></header>
        <p>{item.latest_review?.rationale}</p>
        <p class="public-admin-anchor-boundary">{item.latest_review?.known_limitations}</p>
      </article>}</For>
    </section>
  )}</Show>;
}
