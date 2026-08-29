import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationLedgerTransitionOutputValidations,
  validateControlledShadowObservationLedgerTransitionOutputOnce,
} from "@/lib/api";
import type { ControlledShadowObservationLedgerTransitionOutputValidationRegistry } from "@/lib/types";

const VALIDATION_CHECKS = [
  "精确绑定当前 Stage 51–122 完整责任链",
  "验证者独立于 Stage 122 执行人、Stage 121 声明人和全部上游角色",
  "重新打开并重哈希 Stage 122 result 与 create-once 候选文件",
  "重新打开并重哈希 exact Stage 114 admitted observation envelope",
  "使用独立第二投影，不调用 Stage 122 projector helper 自证",
  "独立重算每条 notice 的身份、精确十进制、摘要、排序和完整候选并精确比较",
  "确认期初组合快照缺失且金融事件白名单为空",
  "通过只开放未来 Stage 124 非财务候选准入复核",
  "没有账本、持仓、现金、NAV/绩效、模型/指标、训练、reward、订单、券商或交易权限",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationLedgerTransitionOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowObservationLedgerTransitionOutputValidationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [checks, setChecks] = createSignal(VALIDATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationLedgerTransitionOutputValidations();
      setRegistry(next);
      if (!next.items.some((item) => item.validation_eligible && item.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(next.items.find((item) => item.validation_eligible)?.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 123 非财务候选独立校验表读取失败");
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
    const candidateSha = item?.result.candidate_sha256;
    if (!item || !candidateSha || disabled()) return;
    const specification = item.claim.authorization.runner.implementation.implementation_contract
      .exact_observation_ledger_transition_specification;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await validateControlledShadowObservationLedgerTransitionOutputOnce(
        item.claim.attempt_id,
        {
          expected_claim_sha256: item.claim.claim_sha256,
          expected_result_sha256: item.result.result_sha256,
          expected_candidate_sha256: candidateSha,
          expected_specification_sha256: specification.specification_sha256,
          expected_stage_114_review_sha256: specification.stage_114_review_sha256,
          expected_stage_112_output_sha256: specification.stage_112_output_sha256,
          validation_reason: reason().trim(),
          exact_current_stage_51_through_stage_122_binding_confirmed: checks()[0] as boolean,
          validator_independent_from_executor_claimant_and_complete_prior_chain_confirmed: checks()[1] as boolean,
          stage_122_result_and_create_once_candidate_reopened_and_rehashed_confirmed: checks()[2] as boolean,
          exact_stage_114_admitted_observation_envelope_reopened_and_rehashed_confirmed: checks()[3] as boolean,
          second_projection_does_not_call_stage_122_projector_helpers_confirmed: checks()[4] as boolean,
          every_notice_identity_decimal_hash_sort_and_complete_candidate_exactly_compared_confirmed: checks()[5] as boolean,
          opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: checks()[6] as boolean,
          pass_only_opens_future_stage_124_non_financial_candidate_admission_review_confirmed: checks()[7] as boolean,
          no_ledger_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed: checks()[8] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[9] as boolean,
        },
      );
      setRegistry(next);
      setReason("");
      setChecks(VALIDATION_CHECKS.map(() => false));
      const validation = next.items.find(
        (value) => value.claim.attempt_id === item.claim.attempt_id,
      )?.validation;
      setNotice(validation?.non_financial_notice_candidate_independently_validated
        ? `独立第二投影精确一致：${validation.observed_notice_count} 条通知；候选仍未受信，只等待 Stage 124 准入复核。`
        : `独立校验失败并永久关闭：${validation?.mismatch_reasons.join("；") || "未知差异"}`);
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 123 独立校验失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="非财务观察通知候选责任链外独立校验">
        <header><strong>第 123 阶段 · 非财务候选独立校验</strong><span>第二投影 · 完整候选比对</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>待独立校验</span><strong>{current().validation_eligible_count}</strong></div>
          <div><span>精确一致</span><strong>{current().independently_validated_candidate_count}</strong></div>
          <div><span>失败关闭</span><strong>{current().failed_validation_count}</strong></div>
          <div><span>Stage 124 候选</span><strong>{current().future_stage_124_admission_review_eligible_count}</strong></div>
        </div>
        <Show
          when={current().items.some((item) => item.validation_eligible)}
          fallback={<p>当前没有待独立校验的 Stage 122 非财务候选。</p>}
        >
          <label><span>Stage 122 非财务候选</span><select
            value={selectedAttemptId()}
            onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}
          >
            <For each={current().items.filter((item) => item.validation_eligible)}>{(item) => (
              <option value={item.claim.attempt_id}>
                {item.claim.attempt_id.slice(0, 12)}… · {item.result.candidate_sha256?.slice(0, 12)}… · {item.result.notice_candidate_count} 条
              </option>
            )}</For>
          </select></label>
          <Show when={selected()}>{(item) => (
            <article class="public-admin-reward-governance">
              <header><strong>冻结校验输入</strong><span>create-once</span></header>
              <p>result {item().result.result_sha256.slice(0, 16)}… · candidate {item().result.candidate_sha256?.slice(0, 16)}…</p>
              <p>Stage 114 {item().claim.authorization.runner.implementation.implementation_contract.exact_observation_ledger_transition_specification.stage_114_review_sha256.slice(0, 16)}…</p>
              <p class="public-admin-anchor-boundary">失败会形成不可覆盖终态；通过也不会建立账本、现金、仓位或 NAV。</p>
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
          >{busy() ? "正在独立重建并比对…" : "执行一次 Stage 123 独立校验"}</button>
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().items.filter((item) => item.validation)}>{(item) => (
          <article class="public-admin-reward-governance">
            <header>
              <strong>{item.validation?.non_financial_notice_candidate_independently_validated
                ? "独立第二投影一致 · 仍未受信"
                : "独立校验失败 · 候选关闭"}</strong>
              <span>{item.validation?.validated_at}</span>
            </header>
            <p>validation {item.validation?.validation_sha256.slice(0, 16)}… · {item.validation?.observed_notice_count} 条通知</p>
            <p class="public-admin-anchor-boundary">{item.validation?.non_financial_notice_candidate_independently_validated
              ? "只进入 Stage 124 非财务准入候选；没有财务账本或交易权限。"
              : item.validation?.mismatch_reasons.join("；")}</p>
          </article>
        )}</For>
      </section>
    )}</Show>
  );
}
