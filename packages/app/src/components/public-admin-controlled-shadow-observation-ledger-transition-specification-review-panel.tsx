import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationLedgerTransitionSpecificationReviews,
  reviewControlledShadowObservationLedgerTransitionSpecification,
} from "@/lib/api";
import type {
  ControlledShadowObservationLedgerTransitionSpecificationReviewRegistry,
  ControlledShadowObservationLedgerTransitionSpecificationReviewVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–115 完整责任链",
  "复核者独立于 Stage 115 登记者和全部既有责任人",
  "独立复算 registration 与 specification 哈希",
  "不调用 Stage 115 builder，从当前 Stage 114 正式证据完整重建规格",
  "第二实现重建结果与已登记规格逐字段一致",
  "Stage 88 绑定只是初始化来源，不是 opening positions",
  "opening portfolio snapshot 必须另行准入，且不默认或推断本金、现金、持仓、股数、目标权重",
  "证券会计只用 raw close；adjusted prices 仅作非会计分析，不重复计入",
  "显式 gap 阻断 NAV，不填充、不插值、不替代",
  "分红和拆股在持仓与有效条款准入前只记 notice",
  "精确十进制、append-only、幂等事件与双分录约束保持一致",
  "修正只来自新准入证据，并通过 superseding 或 reversal 事件追加",
  "保守 available-at 与供应商发布时间未验证限制保持不变",
  "没有实现、工件、入口、runtime、输入挂载或财务写入",
  "没有账本事件、持仓、现金、NAV/绩效、模型、训练/RL、reward、订单、券商或交易权限",
  "批准只开放未来 Stage 117 零能力实现登记",
  "未把未确认的 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationLedgerTransitionSpecificationReviewPanel() {
  const [registry, setRegistry] = createSignal<ControlledShadowObservationLedgerTransitionSpecificationReviewRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [verdict, setVerdict] = createSignal<ControlledShadowObservationLedgerTransitionSpecificationReviewVerdict>(
    "approved_for_future_zero_capability_ledger_transition_implementation_registration",
  );
  const [rationale, setRationale] = createSignal("");
  const [binding, setBinding] = createSignal("");
  const [opening, setOpening] = createSignal("");
  const [prices, setPrices] = createSignal("");
  const [actions, setActions] = createSignal("");
  const [events, setEvents] = createSignal("");
  const [zeroCapability, setZeroCapability] = createSignal("");
  const [limitations, setLimitations] = createSignal(
    "opening portfolio snapshot 尚未独立准入；供应商发布时间仍未验证；本阶段无账本或自然前向绩效。",
  );
  const [constraints, setConstraints] = createSignal(
    "Stage 117 只能登记零能力实现合同；不得携带可执行工件、入口、runtime、输入挂载或任何财务写入权限。",
  );
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationLedgerTransitionSpecificationReviews();
      setRegistry(next);
      if (!next.items.some((item) => item.registration.registration_id === selectedId() && item.review_eligible)) {
        setSelectedId(next.items.find((item) => item.review_eligible)?.registration.registration_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 116 账本转换规格独立复核表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.items.find(
    (item) => item.registration.registration_id === selectedId() && item.review_eligible,
  ));
  const isApproval = createMemo(() => verdict()
    === "approved_for_future_zero_capability_ledger_transition_implementation_registration");
  const disabled = createMemo(() => busy() || !selected() || !rationale().trim()
    || !binding().trim() || !opening().trim() || !prices().trim() || !actions().trim()
    || !events().trim() || !zeroCapability().trim() || !limitations().trim()
    || !constraints().trim() || (isApproval() && !checks().every(Boolean)));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const previous = item.latest_review;
    const values = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewControlledShadowObservationLedgerTransitionSpecification(
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
          opening_portfolio_prerequisite_assessment: opening().trim(),
          price_basis_gap_and_nav_assessment: prices().trim(),
          corporate_action_and_double_count_assessment: actions().trim(),
          decimal_idempotency_correction_and_order_assessment: events().trim(),
          zero_capability_assessment: zeroCapability().trim(),
          known_limitations: limitations().trim(),
          future_implementation_constraints: constraints().trim(),
          exact_current_stage_51_through_stage_115_binding_confirmed: values[0] as boolean,
          reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: values[1] as boolean,
          registration_and_specification_hashes_independently_reproduced_confirmed: values[2] as boolean,
          complete_specification_rebuilt_from_current_stage_114_evidence_without_stage_115_builder_confirmed: values[3] as boolean,
          rebuilt_specification_exactly_matches_registered_specification_confirmed: values[4] as boolean,
          stage_88_binding_not_opening_positions_confirmed: values[5] as boolean,
          separate_opening_portfolio_snapshot_required_and_no_defaults_or_inference_confirmed: values[6] as boolean,
          raw_close_only_for_security_accounting_and_adjusted_prices_non_accounting_confirmed: values[7] as boolean,
          explicit_gap_blocks_nav_without_fill_interpolation_or_substitution_confirmed: values[8] as boolean,
          dividends_and_splits_notice_only_until_position_and_terms_are_admitted_confirmed: values[9] as boolean,
          exact_decimal_append_only_idempotent_event_and_double_entry_rules_confirmed: values[10] as boolean,
          corrections_require_new_admitted_evidence_and_superseding_or_reversal_events_confirmed: values[11] as boolean,
          conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: values[12] as boolean,
          no_implementation_artifact_entrypoint_runtime_input_mount_or_financial_write_confirmed: values[13] as boolean,
          no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: values[14] as boolean,
          approval_only_opens_future_zero_capability_implementation_registration_confirmed: values[15] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: values[16] as boolean,
        },
      );
      setRegistry(next);
      setNotice(isApproval()
        ? "Stage 116 独立复核已批准；仅开放未来 Stage 117 零能力实现登记，尚未建账或计算绩效。"
        : "Stage 116 复核已不可变记录；后续晋级保持失败关闭。");
      setRationale("");
      setBinding("");
      setOpening("");
      setPrices("");
      setActions("");
      setEvents("");
      setZeroCapability("");
      setChecks(REVIEW_CHECKS.map(() => false));
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 116 账本转换规格独立复核提交失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="观察证据到账本转换规格独立复核">
      <header><strong>第 116 阶段 · 账本转换规格独立复核</strong><span>第二实现 · 责任链外</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>规格</span><strong>{current().specification_count}</strong></div>
        <div><span>待复核</span><strong>{current().review_eligible_count}</strong></div>
        <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
        <div><span>独立批准</span><strong>{current().independently_approved_count}</strong></div>
      </div>
      <p class="public-admin-anchor-boundary">独立审计必须从 Stage 114 正式证据重建整份规格；不能只相信 Stage 115 自报摘要。缺 opening portfolio snapshot 时仍必须失败关闭。</p>
      <Show when={current().items.some((item) => item.review_eligible)} fallback={<p>当前没有待复核的 Stage 115 规格。</p>}>
        <label><span>待复核规格</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}>
          <For each={current().items.filter((item) => item.review_eligible)}>{(item) => <option value={item.registration.registration_id}>
            {item.registration.specification.subject_symbols.join(", ") || "无标的"} · {item.registration.registration_sha256.slice(0, 12)}…
          </option>}</For>
        </select></label>
        <Show when={selected()}>{(item) => <article class="public-admin-reward-governance">
          <header><strong>第二实现审计</strong><span>{item().current_independent_audit.mismatch_reasons.length === 0 ? "结构一致" : "失败关闭"}</span></header>
          <p>registration {item().current_independent_audit.registration_hash_independently_reproduced ? "已复算" : "不一致"} · specification {item().current_independent_audit.specification_hash_independently_reproduced ? "已复算" : "不一致"} · 完整重建 {item().current_independent_audit.rebuilt_specification_exactly_matches_registration ? "逐字段一致" : "不一致"}</p>
          <p class="public-admin-anchor-boundary">opening/no-invention {item().current_independent_audit.opening_portfolio_prerequisite_and_no_invention_contract_valid ? "通过" : "异常"} · raw/adjusted/gap/NAV {item().current_independent_audit.raw_price_adjusted_price_gap_and_nav_contract_valid ? "通过" : "异常"} · 零权限 {item().current_independent_audit.all_implementation_ledger_financial_feedback_order_broker_and_trading_authority_closed ? "关闭" : "异常"}</p>
        </article>}</Show>
        <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ControlledShadowObservationLedgerTransitionSpecificationReviewVerdict)}>
          <option value="approved_for_future_zero_capability_ledger_transition_implementation_registration">批准进入 Stage 117 零能力实现登记</option>
          <option value="changes_required_rebuild_ledger_transition_specification">要求重建规格</option>
          <option value="rejected_ledger_transition_specification">拒绝规格</option>
        </select></label>
        <label><span>复核理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
        <label><span>绑定与第二实现评估</span><textarea value={binding()} onInput={(event) => setBinding(event.currentTarget.value)} /></label>
        <label><span>opening portfolio 前置条件评估</span><textarea value={opening()} onInput={(event) => setOpening(event.currentTarget.value)} /></label>
        <label><span>价格口径、缺口与 NAV 评估</span><textarea value={prices()} onInput={(event) => setPrices(event.currentTarget.value)} /></label>
        <label><span>公司行动与防重复计入评估</span><textarea value={actions()} onInput={(event) => setActions(event.currentTarget.value)} /></label>
        <label><span>十进制、幂等、修正、顺序与双分录评估</span><textarea value={events()} onInput={(event) => setEvents(event.currentTarget.value)} /></label>
        <label><span>零能力评估</span><textarea value={zeroCapability()} onInput={(event) => setZeroCapability(event.currentTarget.value)} /></label>
        <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
        <label><span>未来实现约束</span><textarea value={constraints()} onInput={(event) => setConstraints(event.currentTarget.value)} /></label>
        <div class="public-admin-decision-checks"><For each={REVIEW_CHECKS}>{(label, index) => <label><input
          type="checkbox" checked={checks()[index()]}
          onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))}
        /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
          {busy() ? "正在写入不可变复核…" : "提交 Stage 116 独立复核"}
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
