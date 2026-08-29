import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  executeControlledShadowObservationLedgerTransitionAttemptOnce,
  getControlledShadowObservationLedgerTransitionExecutionAttempts,
} from "@/lib/api";
import type { ControlledShadowObservationLedgerTransitionExecutionAttemptRegistry } from "@/lib/types";

const EXECUTION_CHECKS = [
  "精确绑定当前 Stage 51–121 完整责任链",
  "执行人独立于声明人、Stage 120 复核者、工件构建者和全部上游角色",
  "先写 create-once start marker，再读取工件或 exact Stage 114 输入",
  "无论失败、中断或超时，原 claim 都永久消费且不可重试",
  "runner.artifact 只作为严格声明式绑定，不作为命令、脚本或二进制启动",
  "只读重开并重哈希 exact Stage 114 admitted Stage 112 output",
  "期初组合快照缺失，不推断默认本金、现金、仓位或股数",
  "只投影非财务通知白名单，不写 ledger event 或财务分录",
  "证券仅保留 raw close；SPY dividend-adjusted 只作非会计基准比较",
  "显式 gap 阻断 NAV；分红与拆股只保留待验证通知",
  "成功输出 create-once、内容寻址且仍为非可信，必须另做 Stage 123 独立验证",
  "执行中没有网络、环境变量、secret、工具、子进程或生产读写权限",
  "没有权威财务状态、模型/指标、训练、reward、订单、券商或交易权限",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationLedgerTransitionExecutionAttemptPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowObservationLedgerTransitionExecutionAttemptRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [checks, setChecks] = createSignal(EXECUTION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationLedgerTransitionExecutionAttempts();
      setRegistry(next);
      if (!next.pending_claims.some((item) => item.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(next.pending_claims[0]?.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 122 观察到账本转换执行表读取失败");
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
    const runnerContract = authorization.runner.runner_contract;
    const contract = authorization.runner.implementation.implementation_contract;
    const specification = contract.exact_observation_ledger_transition_specification;
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await executeControlledShadowObservationLedgerTransitionAttemptOnce(
        claim.attempt_id,
        {
          expected_claim_sha256: claim.claim_sha256,
          expected_authorization_review_sha256: authorization.review_sha256,
          expected_runner_contract_sha256: runnerContract.contract_sha256,
          expected_runner_artifact_sha256: authorization.server_computed_artifact_sha256,
          expected_artifact_manifest_sha256: authorization.artifact_manifest.manifest_sha256,
          expected_implementation_contract_sha256: contract.contract_sha256,
          expected_observation_ledger_transition_specification_sha256: specification.specification_sha256,
          expected_stage_114_admission_review_sha256: specification.stage_114_review_sha256,
          expected_stage_113_validation_sha256: specification.stage_113_validation_sha256,
          expected_stage_112_result_sha256: specification.stage_112_result_sha256,
          expected_stage_112_output_sha256: specification.stage_112_output_sha256,
          expected_stage_111_claim_sha256: specification.stage_111_claim_sha256,
          execution_reason: reason().trim(),
          exact_stage_51_through_stage_121_binding_confirmed: checks()[0] as boolean,
          executor_independent_from_complete_prior_chain_and_claimant_confirmed: checks()[1] as boolean,
          start_marker_consumes_claim_before_artifact_or_input_read_confirmed: checks()[2] as boolean,
          one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: checks()[3] as boolean,
          artifact_is_declarative_not_spawned_or_executed_confirmed: checks()[4] as boolean,
          only_exact_stage_114_admitted_output_is_read_only_reopened_and_rehashed_confirmed: checks()[5] as boolean,
          opening_portfolio_snapshot_absent_no_default_notional_cash_positions_or_shares_confirmed: checks()[6] as boolean,
          non_financial_notice_allowlist_only_and_no_ledger_event_or_financial_posting_confirmed: checks()[7] as boolean,
          raw_security_close_and_dividend_adjusted_spy_benchmark_separated_confirmed: checks()[8] as boolean,
          explicit_gap_blocks_nav_and_corporate_actions_remain_pending_validation_confirmed: checks()[9] as boolean,
          output_create_once_content_addressed_untrusted_and_requires_independent_validation_confirmed: checks()[10] as boolean,
          no_network_environment_secret_tool_subprocess_or_production_io_confirmed: checks()[11] as boolean,
          no_authoritative_financial_state_model_metric_training_reward_order_broker_or_trading_confirmed: checks()[12] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[13] as boolean,
        },
      );
      setRegistry(next); setReason(""); setChecks(EXECUTION_CHECKS.map(() => false));
      const result = next.results.find((item) => item.stage_121_attempt_id === claim.attempt_id);
      setNotice(result?.status === "completed_with_untrusted_non_financial_notice_candidate"
        ? `已生成 ${result.notice_candidate_count} 条非财务通知候选；仍为非可信，等待 Stage 123 独立验证。`
        : `转换失败且 claim 已永久消费：${result?.bounded_error_code ?? "unknown_failure"}`);
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 122 观察到账本转换单次执行失败");
      await load();
    } finally { setBusy(false); }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="观察到账本转换单次受限执行">
        <header><strong>第 122 阶段 · 非财务观察通知单次转换</strong><span>声明式工件 · 失败不可重试</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>待执行 claim</span><strong>{current().pending_claim_count}</strong></div>
          <div><span>终态结果</span><strong>{current().terminal_result_count}</strong></div>
          <div><span>非可信候选</span><strong>{current().successful_untrusted_candidate_count}</strong></div>
          <div><span>失败已消费</span><strong>{current().failed_consumed_claim_count}</strong></div>
        </div>
        <Show when={current().pending_claims.length > 0} fallback={<p>当前没有可执行的 Stage 121 claim。</p>}>
          <label><span>Stage 121 claim</span><select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
            <For each={current().pending_claims}>{(claim) => {
              const specification = claim.authorization.runner.implementation.implementation_contract.exact_observation_ledger_transition_specification;
              return <option value={claim.attempt_id}>{specification.subject_symbols.join(", ")} · {specification.earliest_market_session_date} 至 {specification.latest_market_session_date}</option>;
            }}</For>
          </select></label>
          <Show when={selected()}>{(claim) => {
            const specification = claim().authorization.runner.implementation.implementation_contract.exact_observation_ledger_transition_specification;
            return <article class="public-admin-reward-governance">
              <header><strong>不可变输入与零财务权限</strong><span>{specification.observed_session_count} 个交易日</span></header>
              <p>claim {claim().claim_sha256.slice(0, 16)}… · artifact {claim().authorization.server_computed_artifact_sha256.slice(0, 16)}…</p>
              <p>input {specification.stage_112_output_sha256.slice(0, 16)}… · spec {specification.specification_sha256.slice(0, 16)}…</p>
              <p class="public-admin-anchor-boundary">没有期初组合快照。本次最多生成非财务通知候选，不会建立账本、现金、仓位、NAV 或交易状态。</p>
            </article>;
          }}</Show>
          <label><span>执行原因</span><textarea value={reason()} onInput={(event) => setReason(event.currentTarget.value)} /></label>
          <div class="public-admin-decision-checks"><For each={EXECUTION_CHECKS}>{(label, index) => (
            <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
          )}</For></div>
          <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在投影并写入终态…" : "执行一次 Stage 122 非财务转换"}</button>
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().results}>{(result) => <article class="public-admin-reward-governance">
          <header><strong>{result.status === "completed_with_untrusted_non_financial_notice_candidate" ? "候选已生成 · 非可信" : "转换失败 · claim 已消费"}</strong><span>{result.completed_at}</span></header>
          <p>result {result.result_sha256.slice(0, 16)}… · {result.duration_millis} ms</p>
          <p class="public-admin-anchor-boundary">{result.candidate_sha256 ? `candidate ${result.candidate_sha256.slice(0, 16)}… · ${result.notice_candidate_count} 条 · 等待 Stage 123` : `错误码 ${result.bounded_error_code ?? "unknown_failure"} · 不允许重试`}</p>
        </article>}</For>
      </section>
    )}</Show>
  );
}
