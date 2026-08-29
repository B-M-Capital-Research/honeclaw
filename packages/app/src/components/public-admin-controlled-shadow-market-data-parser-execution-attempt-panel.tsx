import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  executeControlledShadowMarketDataParserAttemptOnce,
  getControlledShadowMarketDataParserExecutionAttempts,
} from "@/lib/api";
import type { ControlledShadowMarketDataParserExecutionAttemptRegistry } from "@/lib/types";

const EXECUTION_CHECKS = [
  "精确绑定当前 Stage 51–101 完整责任链",
  "执行人独立于声明人、Stage 100 复核者和全部上游角色",
  "这是单次尝试；无论失败原因如何，原 claim 都会永久消费且不可重试",
  "runner.artifact 只作为严格声明式程序解释，不作为命令、脚本或二进制启动",
  "只读打开 Stage 101 冻结的 Stage 94 载荷，并在解析前逐个重算长度与 SHA-256",
  "UTF-8、JSON/HTML、日期、数值、重复行、窗口和跨源覆盖均严格失败关闭",
  "成功输出 create-once 且仍为非可信，必须另做 Stage 103 独立校验",
  "执行中没有网络、环境变量、secret、工具、子进程或生产读写权限",
  "没有观察、账本、持仓、绩效、模型/指标、训练、reward、订单、券商或交易权限",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowMarketDataParserExecutionAttemptPanel() {
  const [registry, setRegistry] = createSignal<ControlledShadowMarketDataParserExecutionAttemptRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [checks, setChecks] = createSignal(EXECUTION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowMarketDataParserExecutionAttempts();
      setRegistry(next);
      if (!next.pending_claims.some((item) => item.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(next.pending_claims[0]?.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 102 parser 执行表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.pending_claims.find(
    (item) => item.attempt_id === selectedAttemptId(),
  ));
  const disabled = createMemo(() => busy() || !selected() || reason().trim().length === 0 || !checks().every(Boolean));

  const submit = async () => {
    const claim = selected();
    if (!claim || disabled()) return;
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await executeControlledShadowMarketDataParserAttemptOnce(claim.attempt_id, {
        expected_claim_sha256: claim.claim_sha256,
        expected_authorization_review_sha256: claim.authorization.review_sha256,
        expected_runner_artifact_sha256: claim.authorization.server_computed_artifact_sha256,
        expected_input_manifest_sha256: claim.fixed_input_manifest.input_manifest_sha256,
        execution_reason: reason().trim(),
        exact_stage_51_through_stage_101_binding_confirmed: checks()[0] as boolean,
        executor_independent_from_complete_prior_chain_confirmed: checks()[1] as boolean,
        one_shot_failure_consumes_claim_and_no_retry_confirmed: checks()[2] as boolean,
        artifact_is_declarative_not_spawned_or_executed_confirmed: checks()[3] as boolean,
        only_fixed_stage_94_payloads_are_read_only_opened_and_rehashed_confirmed: checks()[4] as boolean,
        strict_parser_and_cross_source_reconciliation_fail_closed_confirmed: checks()[5] as boolean,
        output_create_once_untrusted_and_requires_independent_validation_confirmed: checks()[6] as boolean,
        no_network_environment_secret_tool_subprocess_or_production_io_confirmed: checks()[7] as boolean,
        no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: checks()[8] as boolean,
        no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[9] as boolean,
      });
      setRegistry(next); setReason(""); setChecks(EXECUTION_CHECKS.map(() => false));
      const result = next.results.find((item) => item.stage_101_attempt_id === claim.attempt_id);
      setNotice(result?.status === "completed_with_untrusted_output"
        ? "单次解析完成；输出仍为非可信，等待 Stage 103 独立校验。"
        : `单次解析失败且 claim 已永久消费：${result?.bounded_error_code ?? "unknown_failure"}`);
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 102 parser 单次执行失败");
      await load();
    } finally { setBusy(false); }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="行情解析器单次受限执行">
        <header><strong>第 102 阶段 · 行情解析器单次受限执行</strong><span>声明式工件 · 失败不可重试</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>待执行 claim</span><strong>{current().pending_claim_count}</strong></div>
          <div><span>终态结果</span><strong>{current().terminal_result_count}</strong></div>
          <div><span>非可信输出</span><strong>{current().successful_untrusted_output_count}</strong></div>
          <div><span>失败已消费</span><strong>{current().failed_consumed_claim_count}</strong></div>
        </div>
        <Show when={current().pending_claims.length > 0} fallback={<p>当前没有可执行的 Stage 101 claim。</p>}>
          <label><span>Stage 101 claim</span><select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
            <For each={current().pending_claims}>{(claim) => <option value={claim.attempt_id}>{claim.fixed_input_manifest.subject_symbols.join(", ")} · {claim.fixed_input_manifest.window_start_date} 至 {claim.fixed_input_manifest.window_end_date}</option>}</For>
          </select></label>
          <Show when={selected()}>{(claim) => <article class="public-admin-reward-governance">
            <header><strong>不可变执行输入</strong><span>{claim().fixed_input_manifest.raw_payload_count} 个载荷</span></header>
            <p>claim {claim().claim_sha256.slice(0, 16)}… · artifact {claim().authorization.server_computed_artifact_sha256.slice(0, 16)}…</p>
            <p>input {claim().fixed_input_manifest.input_manifest_sha256.slice(0, 16)}… · {claim().fixed_input_manifest.total_response_bytes} bytes</p>
            <p class="public-admin-anchor-boundary">点击执行即永久消费这条 claim；任何解析、格式、覆盖或写入失败都不能重试。</p>
          </article>}</Show>
          <label><span>执行原因</span><textarea value={reason()} onInput={(event) => setReason(event.currentTarget.value)} /></label>
          <div class="public-admin-decision-checks"><For each={EXECUTION_CHECKS}>{(label, index) => (
            <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
          )}</For></div>
          <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在执行并写入终态…" : "执行一次 Stage 102 parser"}</button>
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().results}>{(result) => <article class="public-admin-reward-governance">
          <header><strong>{result.status === "completed_with_untrusted_output" ? "解析完成 · 非可信" : "解析失败 · claim 已消费"}</strong><span>{result.completed_at}</span></header>
          <p>result {result.result_sha256.slice(0, 16)}… · {result.duration_millis} ms</p>
          <p class="public-admin-anchor-boundary">{result.output_sha256 ? `output ${result.output_sha256.slice(0, 16)}… · 等待 Stage 103` : `错误码 ${result.bounded_error_code ?? "unknown_failure"} · 不允许重试`}</p>
        </article>}</For>
      </section>
    )}</Show>
  );
}
