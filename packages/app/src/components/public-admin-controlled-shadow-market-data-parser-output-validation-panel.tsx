import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowMarketDataParserOutputValidations,
  validateControlledShadowMarketDataParserOutputOnce,
} from "@/lib/api";
import type { ControlledShadowMarketDataParserOutputValidationRegistry } from "@/lib/types";

const VALIDATION_CHECKS = [
  "精确绑定当前 Stage 51–102 完整责任链",
  "验证者独立于 Stage 102 执行人和全部既有责任人",
  "重新打开 create-once 的 Stage 102 result 与非可信 output",
  "逐份重新打开、限界并重哈希 Stage 94 冻结原始载荷",
  "使用第二解析实现，不调用 Stage 102 parser helper 自证",
  "独立重算每一行哈希，并逐字段比较完整输出",
  "NYSE 日历、SPY 三套价格覆盖、标的缺口、分红和拆股全部失败关闭",
  "source_available_at 仍未验证，不能表述为来源时点已经确认",
  "通过只开放未来 Stage 104 观察输入准入复核，不自动开始观察",
  "没有账本、持仓、绩效、模型/指标、训练、reward、订单、券商或交易权限",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowMarketDataParserOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowMarketDataParserOutputValidationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [checks, setChecks] = createSignal(VALIDATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowMarketDataParserOutputValidations();
      setRegistry(next);
      if (!next.items.some((item) => item.validation_eligible && item.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(next.items.find((item) => item.validation_eligible)?.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 103 解析输出独立校验表读取失败");
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
    if (!item || !outputSha || disabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await validateControlledShadowMarketDataParserOutputOnce(
        item.claim.attempt_id,
        {
          expected_claim_sha256: item.claim.claim_sha256,
          expected_result_sha256: item.result.result_sha256,
          expected_output_sha256: outputSha,
          expected_input_manifest_sha256: item.claim.fixed_input_manifest.input_manifest_sha256,
          expected_stage_94_validation_sha256:
            item.claim.fixed_input_manifest.stage_94_validation.validation_sha256,
          validation_reason: reason().trim(),
          exact_current_stage_51_through_stage_102_binding_confirmed: checks()[0] as boolean,
          validator_independent_from_executor_and_complete_prior_chain_confirmed: checks()[1] as boolean,
          stage_102_result_output_and_create_once_custody_reopened_confirmed: checks()[2] as boolean,
          fixed_stage_94_raw_payloads_rehashed_and_independently_reparsed_confirmed: checks()[3] as boolean,
          second_implementation_does_not_call_stage_102_parser_helpers_confirmed: checks()[4] as boolean,
          every_canonical_row_hash_and_complete_output_exactly_compared_confirmed: checks()[5] as boolean,
          official_calendar_spy_coverage_subject_gaps_and_actions_fail_closed_confirmed: checks()[6] as boolean,
          source_available_at_remains_unverified_confirmed: checks()[7] as boolean,
          pass_only_opens_future_observation_input_admission_review_confirmed: checks()[8] as boolean,
          no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: checks()[9] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[10] as boolean,
        },
      );
      setRegistry(next);
      setReason("");
      setChecks(VALIDATION_CHECKS.map(() => false));
      const validation = next.items.find(
        (value) => value.claim.attempt_id === item.claim.attempt_id,
      )?.validation;
      setNotice(validation?.canonical_parse_output_independently_validated
        ? "独立全量重解析精确一致；仅进入 Stage 104 观察输入准入复核候选。"
        : `独立校验失败并永久关闭该输出：${validation?.mismatch_reasons.join("；") || "未知差异"}`);
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 103 独立校验失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="行情解析输出责任链外独立校验">
        <header><strong>第 103 阶段 · 行情解析输出独立校验</strong><span>第二实现 · 全量重解析</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>待独立校验</span><strong>{current().validation_eligible_count}</strong></div>
          <div><span>精确一致</span><strong>{current().independently_validated_output_count}</strong></div>
          <div><span>失败关闭</span><strong>{current().failed_validation_count}</strong></div>
          <div><span>Stage 104 候选</span><strong>{current().future_observation_input_admission_review_eligible_count}</strong></div>
        </div>
        <Show
          when={current().items.some((item) => item.validation_eligible)}
          fallback={<p>当前没有待独立校验的 Stage 102 非可信输出。</p>}
        >
          <label><span>Stage 102 非可信输出</span><select
            value={selectedAttemptId()}
            onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}
          >
            <For each={current().items.filter((item) => item.validation_eligible)}>{(item) => (
              <option value={item.claim.attempt_id}>
                {item.claim.fixed_input_manifest.subject_symbols.join(", ")} · {item.result.output_sha256?.slice(0, 12)}…
              </option>
            )}</For>
          </select></label>
          <Show when={selected()}>{(item) => (
            <article class="public-admin-reward-governance">
              <header><strong>冻结校验输入</strong><span>{item().claim.fixed_input_manifest.raw_payload_count} 个原始载荷</span></header>
              <p>result {item().result.result_sha256.slice(0, 16)}… · output {item().result.output_sha256?.slice(0, 16)}…</p>
              <p>input {item().claim.fixed_input_manifest.input_manifest_sha256.slice(0, 16)}… · {item().claim.fixed_input_manifest.total_response_bytes} bytes</p>
              <p class="public-admin-anchor-boundary">失败也会形成不可覆盖终态；不得换验证者重试或修改原始输出。</p>
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
          >{busy() ? "正在独立重解析并比对…" : "执行一次 Stage 103 独立校验"}</button>
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().items.filter((item) => item.validation)}>{(item) => (
          <article class="public-admin-reward-governance">
            <header>
              <strong>{item.validation?.canonical_parse_output_independently_validated
                ? "独立重解析一致"
                : "独立校验失败 · 输出关闭"}</strong>
              <span>{item.validation?.validated_at}</span>
            </header>
            <p>validation {item.validation?.validation_sha256.slice(0, 16)}… · {item.validation?.observed_raw_payload_count} 个载荷</p>
            <p class="public-admin-anchor-boundary">{item.validation?.canonical_parse_output_independently_validated
              ? "只进入 Stage 104 候选；仍未开始观察。"
              : item.validation?.mismatch_reasons.join("；")}</p>
          </article>
        )}</For>
      </section>
    )}</Show>
  );
}
