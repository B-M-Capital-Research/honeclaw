import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationLedgerTransitionSpecifications,
  registerControlledShadowObservationLedgerTransitionSpecification,
} from "@/lib/api";
import type {
  ControlledShadowObservationLedgerTransitionSpecificationRegistry,
  RegisterControlledShadowObservationLedgerTransitionSpecificationRequest,
} from "@/lib/types";

const REGISTRATION_CHECKS = [
  "精确绑定当前 Stage 51–114 完整责任链",
  "登记者独立于 Stage 114 reviewer 和全部既有责任人",
  "重新打开、重哈希并重投影 Stage 114 准入证据与完整 envelope",
  "Stage 88 绑定只是初始化来源，不能当作开仓持仓",
  "必须另行独立准入 opening portfolio snapshot",
  "不默认本金、现金、持仓、股数或目标权重",
  "未来持仓估值只用 raw close；复权价不进入会计，避免双计",
  "显式缺口阻断 NAV，不填充、不插值、不跨口径替代",
  "分红/拆股在持仓与有效条款获准前只记 notice、不入账",
  "精确十进制、append-only、幂等与 available-at 规则已冻结",
  "修正必须来自新准入证据，只追加冲销/替代事件，不改历史",
  "本阶段只有规格，没有实现、工件、入口、runtime 或输入挂载",
  "没有账本事件、持仓、现金、NAV/绩效、训练/RL、订单、券商或交易权限",
  "实现前仍需 Stage 116 责任链外独立规格复核",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationLedgerTransitionSpecificationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowObservationLedgerTransitionSpecificationRegistry>();
  const [selectedReviewId, setSelectedReviewId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [limitations, setLimitations] = createSignal(
    "当前只有观察证据，没有独立准入的 opening portfolio snapshot；不能推断本金、现金、持仓或股数，也不能计算 NAV/绩效。",
  );
  const [reviewConstraints, setReviewConstraints] = createSignal(
    "Stage 116 必须从刚刚重投影的 exact envelope 独立重建全部会计口径，并确认复权价不入会计、显式缺口失败关闭。",
  );
  const [checks, setChecks] = createSignal(REGISTRATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationLedgerTransitionSpecifications();
      setRegistry(next);
      const registered = new Set(next.registrations.map((item) => item.stage_114_review_id));
      if (!next.candidates.some((item) => item.stage_114_review_id === selectedReviewId() && !registered.has(item.stage_114_review_id))) {
        setSelectedReviewId(next.candidates.find((item) => !registered.has(item.stage_114_review_id))?.stage_114_review_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 115 账本转换规格登记表读取失败");
    }
  };
  onMount(() => void load());

  const eligibleCandidates = createMemo(() => {
    const current = registry();
    if (!current) return [];
    const registered = new Set(current.registrations.map((item) => item.stage_114_review_id));
    return current.candidates.filter((item) => !registered.has(item.stage_114_review_id));
  });
  const selected = createMemo(() => eligibleCandidates().find(
    (item) => item.stage_114_review_id === selectedReviewId(),
  ));
  const disabled = createMemo(() => busy()
    || !selected()
    || !reason().trim()
    || !limitations().trim()
    || !reviewConstraints().trim()
    || !checks().every(Boolean));

  const submit = async () => {
    const candidate = selected();
    if (!candidate || disabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const request: RegisterControlledShadowObservationLedgerTransitionSpecificationRequest = {
        expected_stage_114_review_sha256: candidate.stage_114_review_sha256,
        expected_stage_113_validation_sha256: candidate.stage_113_validation_sha256,
        expected_stage_112_result_sha256: candidate.stage_112_result_sha256,
        expected_stage_112_output_sha256: candidate.stage_112_output_sha256,
        expected_stage_111_claim_sha256: candidate.stage_111_claim_sha256,
        registration_reason: reason().trim(),
        known_limitations: limitations().trim(),
        future_review_constraints: reviewConstraints().trim(),
        exact_current_stage_51_through_stage_114_binding_confirmed: checks()[0] as boolean,
        registrar_independent_from_stage_114_and_complete_prior_chain_confirmed: checks()[1] as boolean,
        stage_114_admission_and_full_envelope_reopened_rehashed_and_reprojected_confirmed: checks()[2] as boolean,
        stage_88_binding_not_treated_as_opening_positions_confirmed: checks()[3] as boolean,
        separately_admitted_opening_portfolio_snapshot_required_confirmed: checks()[4] as boolean,
        no_default_notional_cash_positions_or_share_quantities_confirmed: checks()[5] as boolean,
        raw_close_only_for_portfolio_marks_and_adjusted_prices_not_double_counted_confirmed: checks()[6] as boolean,
        explicit_gap_blocks_nav_no_fill_interpolation_or_substitution_confirmed: checks()[7] as boolean,
        dividend_and_split_notices_require_position_and_effective_term_validation_before_posting_confirmed: checks()[8] as boolean,
        exact_decimal_append_only_idempotent_and_available_at_rules_confirmed: checks()[9] as boolean,
        corrections_require_new_admitted_evidence_and_never_mutate_history_confirmed: checks()[10] as boolean,
        specification_only_no_implementation_artifact_entrypoint_runtime_or_input_mount_confirmed: checks()[11] as boolean,
        no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: checks()[12] as boolean,
        future_chain_external_specification_review_required_before_implementation_confirmed: checks()[13] as boolean,
        no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[14] as boolean,
      };
      const next = await registerControlledShadowObservationLedgerTransitionSpecification(
        candidate.stage_114_review_id,
        request,
      );
      setRegistry(next);
      setReason("");
      setChecks(REGISTRATION_CHECKS.map(() => false));
      setNotice("Stage 115 零能力转换规格已登记；只开放 Stage 116 独立复核，尚未建账或计算绩效。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 115 转换规格登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="观察证据到账本转换规格登记">
      <header><strong>第 115 阶段 · 观察证据到账本转换规格</strong><span>规格冻结 · 零会计写入</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>正式观察证据</span><strong>{current().admitted_observation_evidence_count}</strong></div>
        <div><span>待登记</span><strong>{current().registration_eligible_count}</strong></div>
        <div><span>已登记规格</span><strong>{current().registered_specification_count}</strong></div>
        <div><span>缺 opening snapshot</span><strong>{current().opening_portfolio_snapshot_missing_count}</strong></div>
      </div>
      <p class="public-admin-anchor-boundary">
        关键纠偏：Stage 88 只是初始化来源，不包含可直接入账的本金、现金、持仓或股数。缺少独立准入的 opening portfolio snapshot 时必须失败关闭。
      </p>
      <Show when={eligibleCandidates().length > 0} fallback={<p>当前没有待登记的 Stage 114 正式观察证据。</p>}>
        <label><span>Stage 114 正式证据</span><select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
          <For each={eligibleCandidates()}>{(item) => (
            <option value={item.stage_114_review_id}>{item.stage_114_review_id.slice(0, 12)}… · {item.subject_symbols.join("、") || "无标的"}</option>
          )}</For>
        </select></label>
        <Show when={selected()}>{(item) => <article class="public-admin-reward-governance">
          <header><strong>待冻结 envelope</strong><span>{item().admitted_available_at_utc}</span></header>
          <p>{item().observed_session_count} 个交易日 · {item().observed_price_count} 个价格观察 · {item().observed_gap_count} 个显式缺口</p>
          <p class="public-admin-anchor-boundary">output {item().stage_112_output_sha256.slice(0, 16)}…；原始证据不改写。</p>
        </article>}</Show>
        <label><span>登记理由</span><textarea value={reason()} onInput={(event) => setReason(event.currentTarget.value)} /></label>
        <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
        <label><span>Stage 116 复核约束</span><textarea value={reviewConstraints()} onInput={(event) => setReviewConstraints(event.currentTarget.value)} /></label>
        <div class="public-admin-decision-checks"><For each={REGISTRATION_CHECKS}>{(label, index) => <label>
          <input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} />
          <span>{label}</span>
        </label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在重开证据并冻结规格…" : "登记 Stage 115 零能力转换规格"}</button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().registrations}>{(item) => <article class="public-admin-reward-governance">
        <header><strong>已登记 · 等待 Stage 116 独立复核</strong><span>{item.registered_at}</span></header>
        <p>{item.registration_reason}</p>
        <p>raw close：未来持仓估值口径；SPY dividend-adjusted：仅非会计总回报对照；显式 gap：阻断 NAV。</p>
        <p class="public-admin-anchor-boundary">opening snapshot 当前不可用；financial postings={String(item.specification.financial_postings_currently_eligible)}，ledger={String(item.ledger_created)}。</p>
      </article>}</For>
    </section>
  )}</Show>;
}
