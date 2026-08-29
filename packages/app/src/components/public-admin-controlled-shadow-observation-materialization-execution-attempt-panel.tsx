import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  executeControlledShadowObservationMaterializationAttemptOnce,
  getControlledShadowObservationMaterializationExecutionAttempts,
} from "@/lib/api";
import type { ControlledShadowObservationMaterializationExecutionAttemptRegistry } from "@/lib/types";

const EXECUTION_CHECKS = [
  "精确绑定当前 Stage 51–111 完整责任链",
  "执行人独立于声明人、Stage 110 复核者、工件构建者和全部上游角色",
  "先写 create-once start marker，再读取工件或 exact Stage 104 输入",
  "无论失败、中断或超时，原 claim 都永久消费且不可重试",
  "runner.artifact 只作为严格声明式绑定，不作为命令、脚本或二进制启动",
  "只读重开并重哈希 exact Stage 104 admitted Stage 102 output，不接受路径、标的或日期替换",
  "只做 session、三价格口径、显式 gap、公司行动、初始分配与可得时间的确定性投影",
  "不重新抓取或解析，不补值、插值、替代、回填或原地修正",
  "成功输出 create-once、内容寻址且仍为非可信，必须另做 Stage 113 独立验证",
  "执行中没有网络、环境变量、secret、工具、子进程或生产读写权限",
  "没有账本、持仓、绩效、模型/指标、训练、reward、订单、券商或交易权限",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationMaterializationExecutionAttemptPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowObservationMaterializationExecutionAttemptRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [checks, setChecks] = createSignal(EXECUTION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationMaterializationExecutionAttempts();
      setRegistry(next);
      if (!next.pending_claims.some((item) => item.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(next.pending_claims[0]?.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 112 观察物化执行表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.pending_claims.find(
    (item) => item.attempt_id === selectedAttemptId(),
  ));
  const disabled = createMemo(() => busy()
    || !selected()
    || reason().trim().length === 0
    || !checks().every(Boolean));

  const submit = async () => {
    const claim = selected();
    if (!claim || disabled()) return;
    const authorization = claim.authorization;
    const implementation = authorization.runner.implementation;
    const contract = implementation.implementation_contract;
    const specification = contract.exact_observation_materialization_specification;
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await executeControlledShadowObservationMaterializationAttemptOnce(
        claim.attempt_id,
        {
          expected_claim_sha256: claim.claim_sha256,
          expected_authorization_review_sha256: authorization.review_sha256,
          expected_runner_artifact_sha256: authorization.server_computed_artifact_sha256,
          expected_artifact_manifest_sha256: authorization.artifact_manifest.manifest_sha256,
          expected_implementation_contract_sha256: contract.contract_sha256,
          expected_observation_materialization_specification_sha256: specification.specification_sha256,
          expected_stage_104_admission_review_sha256: specification.stage_104_review_sha256,
          expected_stage_102_output_sha256: specification.stage_102_output_sha256,
          expected_stage_101_input_manifest_sha256: specification.stage_101_input_manifest_sha256,
          expected_cycle_claim_sha256: specification.cycle_claim_sha256,
          execution_reason: reason().trim(),
          exact_stage_51_through_stage_111_binding_confirmed: checks()[0] as boolean,
          executor_independent_from_complete_prior_chain_and_claimant_confirmed: checks()[1] as boolean,
          start_marker_consumes_claim_before_artifact_or_input_read_confirmed: checks()[2] as boolean,
          one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: checks()[3] as boolean,
          artifact_is_declarative_not_spawned_or_executed_confirmed: checks()[4] as boolean,
          only_exact_stage_104_admitted_output_is_read_only_opened_and_rehashed_confirmed: checks()[5] as boolean,
          deterministic_session_price_gap_action_allocation_and_availability_projection_confirmed: checks()[6] as boolean,
          no_refetch_reparse_fill_interpolation_substitution_backfill_or_correction_confirmed: checks()[7] as boolean,
          output_create_once_content_addressed_untrusted_and_requires_independent_validation_confirmed: checks()[8] as boolean,
          no_network_environment_secret_tool_subprocess_or_production_io_confirmed: checks()[9] as boolean,
          no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: checks()[10] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[11] as boolean,
        },
      );
      setRegistry(next); setReason(""); setChecks(EXECUTION_CHECKS.map(() => false));
      const result = next.results.find((item) => item.stage_111_attempt_id === claim.attempt_id);
      setNotice(result?.status === "completed_with_untrusted_observation_envelope"
        ? "观察 envelope 已 create-once 生成；仍为非可信，等待 Stage 113 独立验证。"
        : `观察物化失败且 claim 已永久消费：${result?.bounded_error_code ?? "unknown_failure"}`);
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 112 观察物化单次执行失败");
      await load();
    } finally { setBusy(false); }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="观察物化单次受限执行">
        <header><strong>第 112 阶段 · 自然前瞻观察单次物化</strong><span>声明式工件 · 失败不可重试</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>待执行 claim</span><strong>{current().pending_claim_count}</strong></div>
          <div><span>终态结果</span><strong>{current().terminal_result_count}</strong></div>
          <div><span>非可信观察</span><strong>{current().successful_untrusted_observation_count}</strong></div>
          <div><span>失败已消费</span><strong>{current().failed_consumed_claim_count}</strong></div>
        </div>
        <Show when={current().pending_claims.length > 0} fallback={<p>当前没有可执行的 Stage 111 claim。</p>}>
          <label><span>Stage 111 claim</span><select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
            <For each={current().pending_claims}>{(claim) => {
              const specification = claim.authorization.runner.implementation.implementation_contract.exact_observation_materialization_specification;
              return <option value={claim.attempt_id}>{specification.subject_symbols.join(", ")} · {specification.window_start_date} 至 {specification.window_end_date}</option>;
            }}</For>
          </select></label>
          <Show when={selected()}>{(claim) => {
            const specification = claim().authorization.runner.implementation.implementation_contract.exact_observation_materialization_specification;
            return <article class="public-admin-reward-governance">
              <header><strong>不可变输入与预定输出</strong><span>{specification.official_market_session_count} 个正式交易日</span></header>
              <p>claim {claim().claim_sha256.slice(0, 16)}… · artifact {claim().authorization.server_computed_artifact_sha256.slice(0, 16)}…</p>
              <p>input {specification.stage_102_output_sha256.slice(0, 16)}… · spec {specification.specification_sha256.slice(0, 16)}…</p>
              <p class="public-admin-anchor-boundary">点击执行会先永久消费 claim；任何工件、输入、矩阵、摘要或写入失败都不能重试。</p>
            </article>;
          }}</Show>
          <label><span>执行原因</span><textarea value={reason()} onInput={(event) => setReason(event.currentTarget.value)} /></label>
          <div class="public-admin-decision-checks"><For each={EXECUTION_CHECKS}>{(label, index) => (
            <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
          )}</For></div>
          <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在物化并写入终态…" : "执行一次 Stage 112 观察物化"}</button>
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().results}>{(result) => <article class="public-admin-reward-governance">
          <header><strong>{result.status === "completed_with_untrusted_observation_envelope" ? "观察已物化 · 非可信" : "物化失败 · claim 已消费"}</strong><span>{result.completed_at}</span></header>
          <p>result {result.result_sha256.slice(0, 16)}… · {result.duration_millis} ms</p>
          <p class="public-admin-anchor-boundary">{result.output_sha256 ? `output ${result.output_sha256.slice(0, 16)}… · 等待 Stage 113` : `错误码 ${result.bounded_error_code ?? "unknown_failure"} · 不允许重试`}</p>
        </article>}</For>
      </section>
    )}</Show>
  );
}
