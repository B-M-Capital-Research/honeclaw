import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowForwardObservationProtocolRegistrations,
  registerControlledShadowForwardObservationProtocol,
} from "@/lib/api";
import type { ControlledShadowForwardObservationProtocolRegistrationRegistry } from "@/lib/types";

const CHECKS = [
  "确认精确绑定 Stage 51–81 全链路，登记人独立于 Stage 81 校验者、executor 与上游角色",
  "确认只允许协议批准后自然到来的未来交易日，不回填、不追溯改写",
  "确认每周周期必须 claim-first、create-once，并使用内容寻址的点时白名单来源",
  "确认使用官方美股交易日历，证券与 SPY 同时点观察",
  "确认保留原始收盘、复权、拆股、分红及公司行动证据，更正只能追加记录",
  "确认下一完整交易日模拟成交、单边 25bp 成本与四种反事实保持冻结",
  "确认 21/63/126/252 日检查点及 252 日、40 信号、12 公司、4 季度最低门槛，不允许提前晋级",
  "确认停止规则失败关闭且不得原地重启，观察前还需责任链外独立复核",
  "确认本阶段不观察、不建账、不写持仓或绩效，不开放模型/指标、反馈、reward、订单、券商或交易",
] as const;

export function PublicAdminControlledShadowForwardObservationProtocolRegistrationPanel() {
  const [registry, setRegistry] = createSignal<ControlledShadowForwardObservationProtocolRegistrationRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [texts, setTexts] = createSignal(["", "", "", "", ""]);
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [message, setMessage] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowForwardObservationProtocolRegistrations();
      setRegistry(next);
      const first = next.items.find((item) => item.registration_eligible);
      setSelectedId((current) => next.items.some((item) => item.registration_eligible && item.source.validation?.validation_id === current) ? current : first?.source.validation?.validation_id ?? "");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Stage 82 协议登记表读取失败");
    }
  };
  onMount(() => void load());

  const eligible = createMemo(() => registry()?.items.filter((item) => item.registration_eligible) ?? []);
  const selected = createMemo(() => eligible().find((item) => item.source.validation?.validation_id === selectedId()));
  const disabled = createMemo(() => busy() || !selected() || texts().some((value) => !value.trim()) || checks().some((value) => !value));

  const submit = async () => {
    const item = selected();
    const validation = item?.source.validation;
    const claim = item?.source.attempt.claim;
    const result = item?.source.attempt.result;
    if (!validation || !claim || !result || !result.output_sha256 || disabled()) return;
    setBusy(true); setMessage("");
    try {
      const next = await registerControlledShadowForwardObservationProtocol(validation.validation_id, {
        expected_validation_sha256: validation.validation_sha256,
        expected_claim_sha256: claim.claim_sha256,
        expected_result_sha256: result.result_sha256,
        expected_output_sha256: result.output_sha256,
        expected_input_manifest_sha256: claim.input_manifest_sha256,
        expected_authorization_review_sha256: claim.authorization_review_sha256,
        expected_isolated_runner_spec_sha256: claim.isolated_runner_spec_sha256,
        expected_runner_artifact_sha256: claim.runner_artifact_sha256,
        expected_implementation_contract_sha256: claim.implementation_contract_sha256,
        expected_design_specification_sha256: claim.design_specification_sha256,
        expected_candidate_set_sha256: claim.candidate_set_sha256,
        expected_feature_order_sha256: claim.feature_order_sha256,
        expected_preprocessing_sha256: claim.preprocessing_sha256,
        expected_target_id: claim.target_id,
        expected_frozen_candidate_algorithm_id: claim.frozen_candidate_algorithm_id,
        protocol_rationale: texts()[0], source_custody_plan: texts()[1], market_calendar_plan: texts()[2],
        corporate_action_correction_policy: texts()[3], stop_execution_plan: texts()[4],
        exact_stage_51_through_stage_81_binding_confirmed: true,
        registrar_independent_from_stage_81_and_complete_prior_chain_confirmed: true,
        natural_forward_only_no_backfill_confirmed: true,
        weekly_claim_first_content_addressed_observation_confirmed: true,
        official_us_market_calendar_and_spy_synchronization_confirmed: true,
        point_in_time_allowlisted_source_custody_confirmed: true,
        adjusted_prices_dividends_and_append_only_corrections_confirmed: true,
        next_full_session_fill_and_registered_costs_confirmed: true,
        checkpoints_minimum_samples_metrics_and_counterfactuals_preserved_confirmed: true,
        stop_rules_fail_closed_and_no_in_place_restart_confirmed: true,
        independent_protocol_review_required_before_observation_confirmed: true,
        no_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed: true,
      });
      setRegistry(next); setTexts(["", "", "", "", ""]); setChecks(CHECKS.map(() => false));
      setMessage("Stage 82 协议已不可变登记；仅进入未来独立协议复核，不会开始观察或生成绩效。");
      await load();
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Stage 82 协议登记失败");
    } finally { setBusy(false); }
  };

  const labels = ["协议依据", "点时来源与证据保管方案", "官方交易日历与半日市/停牌处理", "复权、分红、拆股与追加更正政策", "触发停止后的执行与封存方案"];
  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="Stage 82 受控前向观察协议登记">
      <header><strong>第 82 阶段 · 受控前向观察协议登记</strong><span>{current().protocol_registration_status}</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>待登记</span><strong>{current().protocol_registration_eligible_count}</strong></div>
        <div><span>已登记</span><strong>{current().protocol_registered_count}</strong></div>
        <div><span>当前绑定</span><strong>{current().current_binding_count}</strong></div>
        <div><span>待独立复核</span><strong>{current().future_independent_protocol_review_eligible_count}</strong></div>
      </div>
      <Show when={eligible().length > 0} fallback={<p>当前没有可登记的 Stage 81 独立验证结果。</p>}>
        <label><span>Stage 81 validation</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={eligible()}>{(item) => <option value={item.source.validation?.validation_id}>{item.source.validation?.validation_id.slice(0, 12)}…</option>}</For></select></label>
        <For each={labels}>{(label, index) => <label><span>{label}</span><textarea value={texts()[index()]} onInput={(event) => setTexts((values) => values.map((value, i) => i === index() ? event.currentTarget.value : value))} /></label>}</For>
        <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, i) => i === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在冻结协议…" : "登记自然前向观察协议"}</button>
      </Show>
      <Show when={message()}><p class="public-admin-anchor-boundary">{message()}</p></Show>
      <For each={current().items.filter((item) => item.registration)}>{(item) => <article class="public-admin-reward-governance"><header><strong>协议 {item.registration?.protocol_registration_id.slice(0, 12)}…</strong><span>等待独立复核</span></header><p>{item.registration?.registered_at} · {item.registration?.registered_by}</p><p>最早观察：{item.registration?.protocol_specification.observation_not_before} · {item.registration?.protocol_specification.signal_cadence}</p><p class="public-admin-anchor-boundary">观察未开始 · 账本未创建 · 绩效未计算 · 交易权限关闭</p></article>}</For>
    </section>
  )}</Show>;
}
