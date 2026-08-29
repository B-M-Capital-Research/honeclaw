import { For, Show, createSignal, onMount } from "solid-js";

import {
  getControlledShadowForwardObservationOutputValidations,
  validateControlledShadowForwardObservationOutput,
} from "@/lib/api";
import type { ControlledShadowForwardObservationOutputValidationRegistry } from "@/lib/types";

const VALIDATION_CHECKS = [
  "确认由独立路径重开完整责任链，并从收据重建零行情 manifest 与预期收据",
  "确认精确绑定当前 Stage 51–88，没有用展示字段或执行器 helper 自证",
  "确认验证者独立于 Stage 88 executor、Stage 87 reviewer 和完整既有责任链",
  "确认 claim 先于 manifest 打开和二进制复核，且每个 claim 只有一个终态、不可重放",
  "确认 0 行行情、只允许自然前向且禁止历史回填",
  "确认官方交易日历为 HTTPS 内容寻址来源，证券与 SPY 必须同步观察",
  "确认当前仍无 runtime、行情读取、观察、账本、持仓或绩效",
  "确认不写模型/指标、不反馈训练/reward，不生成订单、不接券商、不交易",
  "确认通过只开放未来首个自然前向周期授权复核资格，不直接开始观察",
  "确认没有把未确认的 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowForwardObservationOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowForwardObservationOutputValidationRegistry>();
  const [checks, setChecks] = createSignal(VALIDATION_CHECKS.map(() => false));
  const [busyId, setBusyId] = createSignal("");
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      setRegistry(await getControlledShadowForwardObservationOutputValidations());
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 89 独立验证登记表读取失败");
    }
  };
  onMount(() => void load());

  const submit = async (
    item: ControlledShadowForwardObservationOutputValidationRegistry["items"][number],
  ) => {
    if (!item.validation_eligible || checks().some((value) => !value)) return;
    const { claim, result } = item.attempt;
    if (!result.output_sha256) return;
    setBusyId(claim.attempt_id);
    setError("");
    setNotice("");
    try {
      setRegistry(await validateControlledShadowForwardObservationOutput(claim.attempt_id, {
        expected_claim_sha256: claim.claim_sha256,
        expected_result_sha256: result.result_sha256,
        expected_output_sha256: result.output_sha256,
        expected_authorization_review_sha256: claim.authorization_review_sha256,
        expected_isolated_runner_spec_sha256: claim.isolated_runner_spec_sha256,
        expected_runner_artifact_sha256: claim.runner_artifact_sha256,
        expected_implementation_contract_sha256: claim.implementation_contract_sha256,
        expected_protocol_specification_sha256: claim.protocol_specification_sha256,
        expected_design_specification_sha256: claim.design_specification_sha256,
        expected_initial_observation_validation_sha256: claim.initial_observation_validation_sha256,
        expected_initialization_manifest_sha256: claim.initialization_manifest_sha256,
        independent_reopen_and_manifest_receipt_reconstruction_confirmed: true,
        exact_current_stage_51_through_stage_88_binding_confirmed: true,
        validator_independent_from_executor_stage_87_and_complete_prior_chain_confirmed: true,
        claim_first_ordering_and_single_terminal_result_confirmed: true,
        zero_market_data_natural_forward_only_and_no_backfill_confirmed: true,
        official_calendar_https_content_hash_and_spy_confirmed: true,
        zero_runtime_observation_ledger_position_and_performance_confirmed: true,
        no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: true,
        validation_only_opens_future_first_natural_forward_cycle_review_confirmed: true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
      }));
      setChecks(VALIDATION_CHECKS.map(() => false));
      setNotice("Stage 89 已写入不可变验证记录；通过只表示零行情初始化收据可复算，仍未开始自然前向观察。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 89 独立验证失败");
    } finally {
      setBusyId("");
      await load();
    }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="Stage 89 前向观察零行情初始化收据独立验证">
        <header><strong>第 89 阶段 · 零行情初始化收据独立验证</strong><span>{current().validation_status}</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>待验证</span><strong>{current().validation_eligible_count}</strong></div>
          <div><span>已验证</span><strong>{current().validation_count}</strong></div>
          <div><span>独立通过</span><strong>{current().independently_validated_initialization_receipt_count}</strong></div>
          <div><span>首周期复核资格</span><strong>{current().future_first_natural_forward_cycle_authorization_review_eligible_count}</strong></div>
        </div>
        <p class="public-admin-anchor-boundary">本阶段只验证 0 行行情、0 个自然前向交易日的初始化收据；不会启动 runtime、观察、账本、持仓、绩效或任何交易链路。</p>
        <Show when={current().validation_eligible_count > 0}>
          <div class="public-admin-decision-checks"><For each={VALIDATION_CHECKS}>{(label, index) => (
            <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
          )}</For></div>
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().items}>{(item) => (
          <article class="public-admin-reward-governance">
            <header><strong>attempt {item.attempt.claim.attempt_id}</strong><span>{item.validation?.verdict ?? "等待责任链外验证"}</span></header>
            <p>claim {item.attempt.claim.claim_sha256} · result {item.attempt.result.result_sha256}</p>
            <Show when={item.validation?.mismatch_reasons.length}><p class="public-admin-error">{item.validation?.mismatch_reasons.join("；")}</p></Show>
            <Show when={item.validation_eligible}>
              <button type="button" class="public-admin-decision-submit" disabled={busyId() !== "" || checks().some((value) => !value)} onClick={() => void submit(item)}>
                {busyId() === item.attempt.claim.attempt_id ? "正在独立重建…" : "写入一次独立验证记录"}
              </button>
            </Show>
          </article>
        )}</For>
      </section>
    )}</Show>
  );
}
