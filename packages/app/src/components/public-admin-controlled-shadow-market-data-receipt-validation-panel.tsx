import { For, Show, createSignal, onMount } from "solid-js";

import {
  getControlledShadowMarketDataReceiptValidations,
  validateControlledShadowMarketDataReceiptOnce,
} from "@/lib/api";
import type {
  ControlledShadowMarketDataReceiptValidationCandidate,
  ControlledShadowMarketDataReceiptValidationRegistry,
  ValidateControlledShadowMarketDataReceiptRequest,
} from "@/lib/types";

const CONFIRMATIONS = [
  "用独立实现重开 Stage 51–93 完整责任链并重算指纹",
  "验证者不是 Stage 93 执行人、Stage 92 复核人或此前责任人",
  "核对先 claim、单一终态、授权不可重放",
  "独立重建固定、脱敏、内容寻址的规范请求集合",
  "重新打开每份原始载荷并重算字节数和 SHA-256",
  "核对来源身份、时间戳与内容寻址保管路径",
  "确认持久化 claim、result、receipt 中没有配置凭据",
  "这里只核验成功 HTTP 载荷外壳，不把它当作行情事实",
  "本阶段不解析交易日历或任何行情行",
  "不启动 runtime/观察，不创建账本、持仓、绩效或模型指标",
  "不训练、不反馈 reward、不生成订单、不接券商、不交易",
  "没有把未确认的 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowMarketDataReceiptValidationPanel() {
  const [registry, setRegistry] = createSignal<ControlledShadowMarketDataReceiptValidationRegistry>();
  const [checks, setChecks] = createSignal(CONFIRMATIONS.map(() => false));
  const [busyId, setBusyId] = createSignal("");
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      setRegistry(await getControlledShadowMarketDataReceiptValidations());
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 94 原始收据独立验证表读取失败");
    }
  };
  onMount(() => void load());

  const validate = async (candidate: ControlledShadowMarketDataReceiptValidationCandidate) => {
    if (!checks().every(Boolean)) return;
    const request: ValidateControlledShadowMarketDataReceiptRequest = {
      expected_claim_sha256: candidate.claim_sha256,
      expected_result_sha256: candidate.result_sha256,
      expected_receipt_sha256: candidate.receipt_sha256,
      expected_adapter_authorization_sha256: candidate.adapter_authorization_sha256,
      expected_cycle_claim_sha256: candidate.cycle_claim_sha256,
      expected_adapter_spec_sha256: candidate.adapter_spec_sha256,
      expected_subject_symbol_set_sha256: candidate.subject_symbol_set_sha256,
      expected_time_window_sha256: candidate.time_window_sha256,
      expected_canonical_request_set_sha256: candidate.canonical_request_set_sha256,
      independent_chain_reopen_and_fingerprint_recomputation_confirmed: true,
      validator_independent_from_executor_stage_92_and_complete_prior_chain_confirmed: true,
      claim_first_single_terminal_result_and_no_replay_confirmed: true,
      redacted_fixed_request_set_independently_reconstructed_confirmed: true,
      every_raw_payload_reopened_size_and_sha256_recomputed_confirmed: true,
      source_identity_timestamp_and_content_addressed_custody_confirmed: true,
      credential_absence_from_persisted_artifacts_confirmed: true,
      successful_http_envelope_only_not_market_truth_confirmed: true,
      validation_does_not_parse_calendar_or_market_rows_confirmed: true,
      no_runtime_observation_ledger_position_performance_or_model_metric_confirmed: true,
      no_training_feedback_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    setBusyId(candidate.attempt_id);
    setError("");
    setNotice("");
    try {
      setRegistry(await validateControlledShadowMarketDataReceiptOnce(candidate.attempt_id, request));
      setChecks(CONFIRMATIONS.map(() => false));
      setNotice("Stage 94 已形成不可覆盖验证终态；通过只代表原始收据完整，不代表行情语义正确。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 94 独立验证失败");
    } finally {
      setBusyId("");
      await load();
    }
  };

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="Stage 94 原始行情收据独立验证">
      <header><strong>第 94 阶段 · 独立验证原始行情收据</strong><span>{current().validation_status}</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>待验证</span><strong>{current().pending_independent_validation_count}</strong></div>
        <div><span>已通过</span><strong>{current().independently_validated_receipt_count}</strong></div>
        <div><span>失败终态</span><strong>{current().failed_independent_validation_count}</strong></div>
        <div><span>未来解析复核资格</span><strong>{current().future_market_data_parser_review_eligible_count}</strong></div>
      </div>
      <Show when={current().candidates.length > 0}>
        <div class="public-admin-decision-checks"><For each={CONFIRMATIONS}>{(label, index) => (
          <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
        )}</For></div>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().candidates}>{(candidate) => (
        <article class="public-admin-reward-governance">
          <header><strong>{candidate.subject_symbols.join("、")}</strong><span>{candidate.window_start_date} → {candidate.window_end_date}</span></header>
          <p>{candidate.raw_payload_count} 份原始载荷 · {candidate.total_response_bytes} bytes · receipt {candidate.receipt_sha256.slice(0, 16)}…</p>
          <div class="public-admin-decision-actions">
            <button type="button" class="public-admin-decision-submit" disabled={busyId() !== "" || !checks().every(Boolean)} onClick={() => void validate(candidate)}>
              {busyId() === candidate.attempt_id ? "正在独立重算…" : "独立验证并写入终态"}
            </button>
          </div>
        </article>
      )}</For>
      <For each={current().validations}>{(record) => (
        <article class="public-admin-reward-governance">
          <header><strong>validation {record.validation_id}</strong><span>{record.verdict}</span></header>
          <p>receipt {record.receipt_sha256.slice(0, 16)}… · custody {record.raw_payload_custody_manifest_sha256.slice(0, 16)}…</p>
          <Show when={record.mismatch_reasons.length > 0}>
            <p class="public-admin-error">不一致：{record.mismatch_reasons.join("；")}</p>
          </Show>
          <Show when={record.raw_market_data_receipt_independently_validated}>
            <p class="public-admin-success">原始字节与保管链已独立验证；仍未解析交易日或价格。</p>
          </Show>
        </article>
      )}</For>
    </section>
  )}</Show>;
}
