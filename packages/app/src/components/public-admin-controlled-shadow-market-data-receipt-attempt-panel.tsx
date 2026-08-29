import { For, Show, createSignal, onMount } from "solid-js";

import {
  claimAndReadControlledShadowMarketDataReceiptOnce,
  getControlledShadowMarketDataReceiptAttempts,
} from "@/lib/api";
import type {
  ClaimAndReadControlledShadowMarketDataReceiptRequest,
  ControlledShadowMarketDataReceiptAttemptRegistry,
  ControlledShadowMarketDataReceiptCandidate,
} from "@/lib/types";

const CONFIRMATIONS = [
  "先写不可覆盖 claim；失败或中断也永久消耗本次授权",
  "精确绑定当前 Stage 51–92 完整哈希链",
  "执行人与 Stage 92 复核者及完整上游责任链独立",
  "只允许固定 HTTPS 来源、路径、查询参数与 GET",
  "股票集合由服务端从已验证影子组合推导，并只额外加入 SPY",
  "只读授权后的自然前向窗口，禁止回填，窗口已内容寻址",
  "API 凭据不会写入 claim、收据、响应或日志",
  "原始载荷、请求、响应、来源、读取和可用时间均留哈希或时间戳",
  "成功收据仍是不可信外部证据，必须另行独立验证",
  "不解析交易日、不生成观察/账本/持仓/绩效/模型指标",
  "不训练、不反馈 reward、不生成订单、不接券商、不交易",
  "没有把未确认的 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowMarketDataReceiptAttemptPanel() {
  const [registry, setRegistry] = createSignal<ControlledShadowMarketDataReceiptAttemptRegistry>();
  const [checks, setChecks] = createSignal(CONFIRMATIONS.map(() => false));
  const [reason, setReason] = createSignal("");
  const [busyId, setBusyId] = createSignal("");
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      setRegistry(await getControlledShadowMarketDataReceiptAttempts());
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 93 原始行情收据表读取失败");
    }
  };
  onMount(() => void load());

  const ready = () => checks().every(Boolean) && reason().trim().length > 0;
  const invoke = async (candidate: ControlledShadowMarketDataReceiptCandidate) => {
    if (!ready() || !candidate.fmp_configured) return;
    const request: ClaimAndReadControlledShadowMarketDataReceiptRequest = {
      expected_adapter_authorization_sha256: candidate.adapter_authorization_sha256,
      expected_cycle_claim_sha256: candidate.cycle_claim_sha256,
      expected_adapter_spec_sha256: candidate.adapter_spec_sha256,
      expected_subject_symbol_set_sha256: candidate.subject_symbol_set_sha256,
      expected_time_window_sha256: candidate.time_window_sha256,
      execution_reason: reason(),
      claim_first_single_use_and_failure_consumes_authorization_confirmed: true,
      exact_stage_51_through_stage_92_binding_confirmed: true,
      executor_independent_from_stage_92_and_complete_prior_chain_confirmed: true,
      fixed_get_https_path_and_query_allowlist_confirmed: true,
      server_derived_subject_symbols_and_spy_only_confirmed: true,
      natural_forward_window_content_addressed_no_backfill_confirmed: true,
      credential_redacted_not_persisted_returned_or_logged_confirmed: true,
      raw_payload_hashes_timestamps_and_custody_retained_confirmed: true,
      receipt_untrusted_pending_independent_validation_confirmed: true,
      no_parsed_calendar_observation_ledger_position_performance_or_model_metric_confirmed: true,
      no_training_feedback_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    setBusyId(candidate.adapter_authorization_id);
    setError("");
    setNotice("");
    try {
      setRegistry(await claimAndReadControlledShadowMarketDataReceiptOnce(candidate.adapter_authorization_id, request));
      setChecks(CONFIRMATIONS.map(() => false));
      setReason("");
      setNotice("Stage 93 已到达不可覆盖终态；成功内容仍只是待独立验证的原始证据。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 93 单次只读行情收据执行失败");
    } finally {
      setBusyId("");
      await load();
    }
  };

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="Stage 93 只读行情原始收据">
      <header><strong>第 93 阶段 · 先声明再读取原始行情</strong><span>{current().receipt_status}</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>可执行授权</span><strong>{current().invocation_eligible_authorization_count}</strong></div>
        <div><span>已声明</span><strong>{current().claim_count}</strong></div>
        <div><span>未信任收据</span><strong>{current().completed_untrusted_receipt_count}</strong></div>
        <div><span>失败/中断</span><strong>{current().failed_authorization_consumed_count + current().interrupted_authorization_consumed_count}</strong></div>
      </div>
      <Show when={current().eligible_authorizations.length > 0}>
        <div class="public-admin-decision-checks"><For each={CONFIRMATIONS}>{(label, index) => (
          <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
        )}</For></div>
        <textarea class="public-admin-decision-textarea" value={reason()} onInput={(event) => setReason(event.currentTarget.value)} placeholder="为什么现在需要取得这一个自然前向窗口的原始数据（必填）" />
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().eligible_authorizations}>{(candidate) => (
        <article class="public-admin-reward-governance">
          <header><strong>{candidate.subject_symbols.join("、")} + {candidate.benchmark_symbol}</strong><span>{candidate.window_start_date} → {candidate.window_end_date}</span></header>
          <p>固定请求 {candidate.expected_request_count} 个 · 股票集合 {candidate.subject_symbol_set_sha256.slice(0, 16)}… · 时间窗 {candidate.time_window_sha256.slice(0, 16)}…</p>
          <Show when={!candidate.fmp_configured}><p class="public-admin-error">FMP Key 或固定 base_url 未配置，服务端不会写 claim，也不会发请求。</p></Show>
          <div class="public-admin-decision-actions">
            <button type="button" class="public-admin-decision-submit" disabled={busyId() !== "" || !ready() || !candidate.fmp_configured} onClick={() => void invoke(candidate)}>
              {busyId() === candidate.adapter_authorization_id ? "已写 claim，正在单次读取…" : "声明并单次读取原始数据"}
            </button>
          </div>
        </article>
      )}</For>
      <For each={current().items}>{(item) => (
        <article class="public-admin-reward-governance">
          <header><strong>attempt {item.claim.attempt_id}</strong><span>{item.result?.status ?? "进程中断，授权已消耗"}</span></header>
          <p>{item.claim.subject_symbols.join("、")} · {item.claim.window_start_date} → {item.claim.window_end_date} · {item.claim.expected_request_count} 个固定请求</p>
          <Show when={item.result?.untrusted_raw_market_data_receipt}>{(receipt) => (
            <p class="public-admin-anchor-boundary">未信任收据 {receipt().receipt_sha256.slice(0, 16)}…；{receipt().raw_payload_count} 个原始载荷，共 {receipt().total_response_bytes} bytes；独立验证尚未完成。</p>
          )}</Show>
          <Show when={item.result?.bounded_error_code}><p class="public-admin-error">失败码：{item.result?.bounded_error_code}；不得重试该授权。</p></Show>
        </article>
      )}</For>
    </section>
  )}</Show>;
}
