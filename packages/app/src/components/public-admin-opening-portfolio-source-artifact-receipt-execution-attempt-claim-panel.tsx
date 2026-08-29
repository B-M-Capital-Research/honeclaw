import { For, Show, createSignal, onMount } from "solid-js";

import {
  claimOpeningPortfolioSourceArtifactReceiptExecutionAttemptOnce,
  getOpeningPortfolioSourceArtifactReceiptExecutionAttemptClaims,
} from "@/lib/api";
import type { OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimRegistry } from "@/lib/types";

const CHECKS = [
  "精确绑定当前 Stage 51–130 完整责任链",
  "领取人独立于 Stage 130 构建者、复核者及完整前序责任链",
  "授权未过期且在任何来源字节前永久消费",
  "服务端在 claim 前重新哈希接收器工件与 manifest",
  "claim 只包含既有元数据与摘要",
  "当前无上传流、来源字节、入口、runtime、挂载、读取或 receipt",
  "未来 Stage 132 仍须单次、create-once、未受信且另行验证",
  "claim 后不得重试、释放或恢复授权",
  "无快照、金融白名单、账本、持仓、现金、净值/绩效、模型、训练/RL、reward、订单、券商或交易权限",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminOpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimPanel() {
  const [registry, setRegistry] = createSignal<OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");
  const load = async () => {
    try {
      const next = await getOpeningPortfolioSourceArtifactReceiptExecutionAttemptClaims();
      setRegistry(next);
      if (!next.eligible_authorizations.some((item) => item.authorization.review_id === selectedId())) setSelectedId(next.eligible_authorizations[0]?.authorization.review_id ?? "");
      setError("");
    } catch (cause) { setError(cause instanceof Error ? cause.message : "Stage 131 资格占用表读取失败"); }
  };
  onMount(() => void load());
  const submit = async () => {
    const item = registry()?.eligible_authorizations.find((value) => value.authorization.review_id === selectedId());
    if (!item || busy() || !reason().trim() || !checks().every(Boolean)) return;
    const review = item.authorization;
    const values = checks();
    setBusy(true); setError(""); setNotice("");
    try {
      setRegistry(await claimOpeningPortfolioSourceArtifactReceiptExecutionAttemptOnce(review.review_id, {
        expected_authorization_review_sha256: review.review_sha256,
        expected_isolated_receiver_spec_sha256: review.receiver.isolated_receiver_spec_sha256,
        expected_receiver_contract_sha256: review.receiver.receiver_contract.contract_sha256,
        expected_receiver_artifact_sha256: review.server_computed_artifact_sha256,
        expected_artifact_manifest_sha256: review.artifact_manifest.manifest_sha256,
        expected_artifact_byte_length: review.server_observed_artifact_byte_length,
        claim_reason: reason().trim(),
        exact_current_stage_51_through_stage_130_binding_confirmed: values[0] as boolean,
        claimant_independent_from_stage_130_builder_reviewer_and_complete_prior_chain_confirmed: values[1] as boolean,
        authorization_unexpired_single_use_and_permanently_consumed_before_source_byte_confirmed: values[2] as boolean,
        server_rehashed_receiver_artifact_and_manifest_before_claim_confirmed: values[3] as boolean,
        claim_contains_only_existing_metadata_and_hashes_confirmed: values[4] as boolean,
        no_upload_stream_source_byte_entrypoint_runtime_mount_input_read_or_receipt_confirmed: values[5] as boolean,
        future_stage_132_attempt_one_shot_create_once_untrusted_and_separately_validated_confirmed: values[6] as boolean,
        no_retry_release_or_authorization_restoration_after_claim_confirmed: values[7] as boolean,
        no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: values[8] as boolean,
        no_unconfirmed_hari_or_old_wang_logic_claimed: values[9] as boolean,
      }));
      setReason(""); setChecks(CHECKS.map(() => false));
      setNotice("Stage 130 授权已永久消费；本次 claim 操作未接收任何来源字节，已进入独立 Stage 132 单次接收门禁。");
      await load();
    } catch (cause) { setError(cause instanceof Error ? cause.message : "Stage 131 资格占用失败"); await load(); }
    finally { setBusy(false); }
  };
  return <Show when={registry()}>{(current) => <section class="public-admin-reward-governance" aria-label="来源工件接收尝试资格占用">
    <header><strong>第 131 阶段 · 来源工件接收尝试 claim-first</strong><span>{current().claim_status}</span></header>
    <p>{current().scope}</p>
    <div class="public-admin-decision-metrics"><div><span>可领取</span><strong>{current().claim_eligible_count}</strong></div><div><span>已领取</span><strong>{current().claim_count}</strong></div><div><span>已消费授权</span><strong>{current().authorization_consumed_count}</strong></div><div><span>待 Stage 132</span><strong>{current().waiting_for_stage_132_attempt_count}</strong></div></div>
    <Show when={current().eligible_authorizations.length > 0} fallback={<p>当前没有可领取的 Stage 130 授权；零状态或已消费状态符合预期。</p>}>
      <label><span>Stage 130 授权</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={current().eligible_authorizations}>{(item) => <option value={item.authorization.review_id}>{item.authorization.receiver.receiver_name} · {item.authorization.review_id.slice(0, 8)}</option>}</For></select></label>
      <label><span>领取原因</span><textarea value={reason()} onInput={(event) => setReason(event.currentTarget.value)} /></label>
      <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((items) => items.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
      <button type="button" class="public-admin-decision-submit" disabled={busy() || !reason().trim() || !checks().every(Boolean)} onClick={() => void submit()}>{busy() ? "正在永久占用…" : "永久占用 Stage 130 授权"}</button>
    </Show>
    <Show when={error()}><p class="public-admin-error">{error()}</p></Show><Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
  </section>}</Show>;
}
