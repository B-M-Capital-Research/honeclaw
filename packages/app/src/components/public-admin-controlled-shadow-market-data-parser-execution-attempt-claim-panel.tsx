import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  claimControlledShadowMarketDataParserExecutionAttemptOnce,
  getControlledShadowMarketDataParserExecutionAttemptClaims,
} from "@/lib/api";
import type { ControlledShadowMarketDataParserExecutionAttemptClaimRegistry } from "@/lib/types";

const CLAIM_CHECKS = [
  "精确绑定当前 Stage 51–100 完整责任链",
  "声明人独立于 Stage 100 复核者和全部上游角色",
  "未过期单次授权会在任何执行前被永久消费",
  "当前服务端重哈希工件与 manifest 绑定保持不变",
  "输入仅限这一条 Stage 94 已独立验证、只读且内容寻址的固定集合",
  "声明只冻结元数据与摘要，不打开任何原始载荷",
  "当前无入口、runtime、挂载、读取、parser 执行或解析行",
  "未来输出必须 create-once、非可信并单独独立验证",
  "无观察、账本、持仓、绩效、模型/指标、训练、reward、订单、券商或交易权限",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowMarketDataParserExecutionAttemptClaimPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowMarketDataParserExecutionAttemptClaimRegistry>();
  const [selectedReviewId, setSelectedReviewId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [checks, setChecks] = createSignal(CLAIM_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowMarketDataParserExecutionAttemptClaims();
      setRegistry(next);
      if (!next.eligible_authorizations.some((item) => item.authorization.review_id === selectedReviewId())) {
        setSelectedReviewId(next.eligible_authorizations[0]?.authorization.review_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 101 parser 尝试声明表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.eligible_authorizations.find(
    (item) => item.authorization.review_id === selectedReviewId(),
  ));
  const disabled = createMemo(() => busy()
    || !selected()
    || reason().trim().length === 0
    || !checks().every(Boolean));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const authorization = item.authorization;
    const runner = authorization.runner;
    const contract = runner.implementation.implementation_contract;
    const input = item.fixed_input_manifest;
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await claimControlledShadowMarketDataParserExecutionAttemptOnce(
        authorization.review_id,
        {
          expected_authorization_review_sha256: authorization.review_sha256,
          expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256,
          expected_runner_artifact_sha256: authorization.server_computed_artifact_sha256,
          expected_artifact_manifest_sha256: authorization.artifact_manifest.manifest_sha256,
          expected_stage_94_validation_sha256: contract.validation_sha256,
          expected_stage_93_claim_sha256: contract.claim_sha256,
          expected_stage_93_result_sha256: contract.result_sha256,
          expected_stage_93_receipt_sha256: contract.receipt_sha256,
          expected_canonical_request_set_sha256: input.canonical_request_set_sha256,
          expected_fixed_input_manifest_sha256: input.input_manifest_sha256,
          claim_reason: reason().trim(),
          exact_current_stage_51_through_stage_100_binding_confirmed: checks()[0] as boolean,
          claimant_independent_from_stage_100_and_complete_prior_chain_confirmed: checks()[1] as boolean,
          authorization_unexpired_single_use_and_consumed_before_execution_confirmed: checks()[2] as boolean,
          current_server_rehashed_artifact_and_manifest_binding_confirmed: checks()[3] as boolean,
          fixed_stage_94_validated_input_set_content_addressed_and_read_only_confirmed: checks()[4] as boolean,
          claim_contains_metadata_and_hashes_but_does_not_open_raw_payloads_confirmed: checks()[5] as boolean,
          no_entrypoint_runtime_mount_payload_read_parser_execution_or_parsed_rows_confirmed: checks()[6] as boolean,
          future_output_create_once_untrusted_and_independently_validated_confirmed: checks()[7] as boolean,
          no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: checks()[8] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[9] as boolean,
        },
      );
      setRegistry(next); setReason(""); setChecks(CLAIM_CHECKS.map(() => false));
      setNotice("Stage 100 授权已由 create-once 声明永久消费；本次没有读取载荷、执行 parser 或生成解析行。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 101 parser 尝试声明失败");
      await load();
    } finally { setBusy(false); }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="行情解析器单次尝试 claim-first 声明">
        <header><strong>第 101 阶段 · 行情解析器单次尝试声明</strong><span>{current().claim_status}</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>可声明授权</span><strong>{current().claim_eligible_count}</strong></div>
          <div><span>已永久消费</span><strong>{current().authorization_consumed_count}</strong></div>
          <div><span>冻结输入</span><strong>{current().claims.length}</strong></div>
          <div><span>待 Stage 102</span><strong>{current().waiting_for_stage_102_execution_count}</strong></div>
        </div>
        <Show when={current().eligible_authorizations.length > 0} fallback={<p>当前没有未过期且未消费的 Stage 100 授权。</p>}>
          <label><span>Stage 100 授权</span><select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
            <For each={current().eligible_authorizations}>{(item) => <option value={item.authorization.review_id}>{item.authorization.runner.runner_name} · {item.fixed_input_manifest.subject_symbols.join(", ")}</option>}</For>
          </select></label>
          <Show when={selected()}>{(item) => <article class="public-admin-reward-governance">
            <header><strong>固定 Stage 94 输入</strong><span>只显示元数据与摘要</span></header>
            <p>{item().fixed_input_manifest.subject_symbols.join(", ")} · {item().fixed_input_manifest.window_start_date} 至 {item().fixed_input_manifest.window_end_date}</p>
            <p>{item().fixed_input_manifest.raw_payload_count} 个载荷 · {item().fixed_input_manifest.total_response_bytes} bytes · 清单 {item().fixed_input_manifest.input_manifest_sha256.slice(0, 16)}…</p>
            <p class="public-admin-anchor-boundary">点击声明会永久消费授权；本按钮不会运行 parser，也不会打开这些载荷。</p>
          </article>}</Show>
          <label><span>声明原因</span><textarea value={reason()} onInput={(event) => setReason(event.currentTarget.value)} /></label>
          <div class="public-admin-decision-checks"><For each={CLAIM_CHECKS}>{(label, index) => (
            <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
          )}</For></div>
          <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在永久消费授权…" : "创建 Stage 101 claim-first 声明"}</button>
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().claims}>{(claim) => <article class="public-admin-reward-governance">
          <header><strong>{claim.fixed_input_manifest.subject_symbols.join(", ")}</strong><span>授权已消费 · 未执行</span></header>
          <p>claim {claim.claim_sha256.slice(0, 16)}… · input {claim.fixed_input_manifest.input_manifest_sha256.slice(0, 16)}… · {claim.claimed_at}</p>
          <p class="public-admin-anchor-boundary">{claim.task_status}</p>
        </article>}</For>
      </section>
    )}</Show>
  );
}
