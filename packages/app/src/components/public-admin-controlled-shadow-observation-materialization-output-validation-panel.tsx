import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationMaterializationOutputValidations,
  validateControlledShadowObservationMaterializationOutputOnce,
} from "@/lib/api";
import type { ControlledShadowObservationMaterializationOutputValidationRegistry } from "@/lib/types";

const VALIDATION_CHECKS = [
  "精确绑定当前 Stage 51–112 完整责任链",
  "验证者独立于 Stage 112 执行人和全部既有责任人",
  "重新打开并重哈希 Stage 112 result 与 create-once 非可信输出",
  "重新打开并重哈希 exact Stage 104 准入的 Stage 102 输入",
  "使用独立第二投影，不调用 Stage 112 materializer helper 自证",
  "独立重算 sessions、三价格口径、显式缺口、公司行动、初始分配和 available-at",
  "独立重算每一行哈希、规范排序和完整 envelope，并逐字段精确比较",
  "通过只开放未来 Stage 114 观察证据准入复核",
  "没有账本、持仓、绩效、模型/指标、训练、reward、订单、券商或交易权限",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationMaterializationOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowObservationMaterializationOutputValidationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [checks, setChecks] = createSignal(VALIDATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationMaterializationOutputValidations();
      setRegistry(next);
      if (!next.items.some((item) => item.validation_eligible && item.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(next.items.find((item) => item.validation_eligible)?.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 113 观察物化输出独立校验表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.items.find(
    (item) => item.validation_eligible && item.claim.attempt_id === selectedAttemptId(),
  ));
  const disabled = createMemo(
    () => busy() || !selected() || reason().trim().length === 0 || !checks().every(Boolean),
  );

  const submit = async () => {
    const item = selected();
    const outputSha = item?.result.output_sha256;
    const specification = item?.claim.authorization.runner.implementation
      .upstream_specification_registration.specification;
    if (!item || !outputSha || !specification || disabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await validateControlledShadowObservationMaterializationOutputOnce(
        item.claim.attempt_id,
        {
          expected_claim_sha256: item.claim.claim_sha256,
          expected_result_sha256: item.result.result_sha256,
          expected_output_sha256: outputSha,
          expected_specification_sha256: specification.specification_sha256,
          expected_stage_104_review_sha256: specification.stage_104_review_sha256,
          expected_stage_102_output_sha256: specification.stage_102_output_sha256,
          validation_reason: reason().trim(),
          exact_current_stage_51_through_stage_112_binding_confirmed: checks()[0] as boolean,
          validator_independent_from_executor_and_complete_prior_chain_confirmed: checks()[1] as boolean,
          stage_112_result_and_create_once_output_reopened_and_rehashed_confirmed: checks()[2] as boolean,
          exact_stage_104_admitted_stage_102_input_reopened_and_rehashed_confirmed: checks()[3] as boolean,
          second_projection_does_not_call_stage_112_materializer_helpers_confirmed: checks()[4] as boolean,
          sessions_prices_gaps_actions_allocation_availability_independently_recomputed_confirmed: checks()[5] as boolean,
          every_row_hash_sort_order_and_complete_envelope_exactly_compared_confirmed: checks()[6] as boolean,
          pass_only_opens_future_stage_114_observation_evidence_admission_review_confirmed: checks()[7] as boolean,
          no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: checks()[8] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[9] as boolean,
        },
      );
      setRegistry(next);
      setReason("");
      setChecks(VALIDATION_CHECKS.map(() => false));
      const validation = next.items.find(
        (value) => value.claim.attempt_id === item.claim.attempt_id,
      )?.validation;
      setNotice(validation?.observation_envelope_independently_validated
        ? "独立第二投影与 Stage 112 输出精确一致；仅进入 Stage 114 观察证据准入复核候选。"
        : `独立校验失败并永久关闭该输出：${validation?.mismatch_reasons.join("；") || "未知差异"}`);
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 113 独立校验失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="观察物化输出责任链外独立校验">
        <header><strong>第 113 阶段 · 观察物化输出独立校验</strong><span>第二投影 · 完整 envelope</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>待独立校验</span><strong>{current().validation_eligible_count}</strong></div>
          <div><span>精确一致</span><strong>{current().independently_validated_observation_count}</strong></div>
          <div><span>失败关闭</span><strong>{current().failed_validation_count}</strong></div>
          <div><span>Stage 114 候选</span><strong>{current().future_stage_114_observation_evidence_admission_review_eligible_count}</strong></div>
        </div>
        <Show
          when={current().items.some((item) => item.validation_eligible)}
          fallback={<p>当前没有待独立校验的 Stage 112 非可信观察 envelope。</p>}
        >
          <label><span>Stage 112 非可信观察</span><select
            value={selectedAttemptId()}
            onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}
          >
            <For each={current().items.filter((item) => item.validation_eligible)}>{(item) => (
              <option value={item.claim.attempt_id}>
                {item.claim.attempt_id.slice(0, 12)}… · {item.result.output_sha256?.slice(0, 12)}…
              </option>
            )}</For>
          </select></label>
          <Show when={selected()}>{(item) => (
            <article class="public-admin-reward-governance">
              <header><strong>冻结校验输入</strong><span>create-once</span></header>
              <p>result {item().result.result_sha256.slice(0, 16)}… · output {item().result.output_sha256?.slice(0, 16)}…</p>
              <p>Stage 104 {item().claim.authorization.runner.implementation.upstream_specification_registration.specification.stage_104_review_sha256.slice(0, 16)}…</p>
              <p class="public-admin-anchor-boundary">失败也会形成不可覆盖终态；不得更换验证者重试或修改 Stage 112 输出。</p>
            </article>
          )}</Show>
          <label><span>独立验证原因</span><textarea
            value={reason()}
            onInput={(event) => setReason(event.currentTarget.value)}
          /></label>
          <div class="public-admin-decision-checks"><For each={VALIDATION_CHECKS}>{(label, index) => (
            <label><input
              type="checkbox"
              checked={checks()[index()]}
              onChange={(event) => setChecks((values) => values.map(
                (value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value,
              ))}
            /><span>{label}</span></label>
          )}</For></div>
          <button
            type="button"
            class="public-admin-decision-submit"
            disabled={disabled()}
            onClick={() => void submit()}
          >{busy() ? "正在独立重投影并比对…" : "执行一次 Stage 113 独立校验"}</button>
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().items.filter((item) => item.validation)}>{(item) => (
          <article class="public-admin-reward-governance">
            <header>
              <strong>{item.validation?.observation_envelope_independently_validated
                ? "独立第二投影一致"
                : "独立校验失败 · 输出关闭"}</strong>
              <span>{item.validation?.validated_at}</span>
            </header>
            <p>validation {item.validation?.validation_sha256.slice(0, 16)}… · {item.validation?.observed_price_count} 个价格观察</p>
            <p class="public-admin-anchor-boundary">{item.validation?.observation_envelope_independently_validated
              ? "只进入 Stage 114 候选；仍未形成账本、持仓、绩效或训练反馈。"
              : item.validation?.mismatch_reasons.join("；")}</p>
          </article>
        )}</For>
      </section>
    )}</Show>
  );
}
