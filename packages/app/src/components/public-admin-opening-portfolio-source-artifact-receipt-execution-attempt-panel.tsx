import { For, Show, createSignal, onMount } from "solid-js";

import {
  getOpeningPortfolioSourceArtifactReceiptExecutionAttempts,
  receiveOpeningPortfolioSourceArtifactReceiptAttemptOnce,
} from "@/lib/api";
import type {
  OpeningPortfolioSourceArtifactReceiptExecutionAttemptRegistry,
  ReceiveOpeningPortfolioSourceArtifactReceiptAttemptRequest,
} from "@/lib/types";

const CHECKS = [
  "精确绑定当前 Stage 51–131 完整责任链",
  "接收人独立于完整前序责任链及 Stage 131 领取人",
  "开始标记必须先于第一个来源字节并永久消费本次尝试",
  "只接受管理员本机流式上传，不允许远程 URL 抓取",
  "文件已完成账户别名化，且不含账号、凭证、口令或 API Key",
  "按 magic、UTF-8/JSON 结构筛查，并拒绝归档、主动内容和密码保护",
  "逐件计算 SHA-256/长度，经私有隔离区后原子写入内容地址",
  "原字节仅以 AES-256-GCM 密文保存，manifest 不记录原文件名或账号",
  "重复内容幂等且禁止覆盖；更正必须上传新工件",
  "receipt 是 create-once 未受信结果，必须经过 Stage 133 独立校验",
  "本阶段不解析金融行、不物化或接纳期初组合",
  "不创建金融白名单、账本、持仓、现金、净值、模型、训练、订单、券商或交易权限",
  "失败或中断会永久消费本次 claim，不能重试",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

type DeclaredFormat = ReceiveOpeningPortfolioSourceArtifactReceiptAttemptRequest["artifacts"][number]["declared_format"];

function formatFor(file: File): DeclaredFormat | null {
  const name = file.name.toLowerCase();
  if (name.endsWith(".pdf")) return "original_provider_pdf_statement";
  if (name.endsWith(".csv")) return "original_provider_csv_export";
  if (name.endsWith(".json")) return "original_provider_json_export";
  return null;
}

function localDateTimeNow() {
  const date = new Date(Date.now() - new Date().getTimezoneOffset() * 60_000);
  return date.toISOString().slice(0, 16);
}

export function PublicAdminOpeningPortfolioSourceArtifactReceiptExecutionAttemptPanel() {
  const [registry, setRegistry] = createSignal<OpeningPortfolioSourceArtifactReceiptExecutionAttemptRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [providerIdentifier, setProviderIdentifier] = createSignal("");
  const [providerAt, setProviderAt] = createSignal(localDateTimeNow());
  const [aliases, setAliases] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [files, setFiles] = createSignal<File[]>([]);
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getOpeningPortfolioSourceArtifactReceiptExecutionAttempts();
      setRegistry(next);
      if (!next.pending_claims.some((item) => item.attempt_id === selectedId())) setSelectedId(next.pending_claims[0]?.attempt_id ?? "");
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 132 单次接收表读取失败");
    }
  };
  onMount(() => void load());

  const aliasesList = () => aliases().split(",").map((value) => value.trim()).filter(Boolean);
  const filesValid = () => files().length > 0 && files().length <= 64 && files().every((file) => formatFor(file));
  const ready = () => Boolean(
    registry()?.encryption_key_configured
      && selectedId()
      && providerIdentifier().trim()
      && providerAt()
      && aliasesList().length
      && reason().trim()
      && filesValid()
      && checks().every(Boolean),
  );

  const submit = async () => {
    const claim = registry()?.pending_claims.find((value) => value.attempt_id === selectedId());
    if (!claim || busy() || !ready()) return;
    const review = claim.authorization;
    const receiverContract = review.receiver.receiver_contract;
    const implementationContract = receiverContract.exact_approved_implementation_contract;
    const values = checks();
    const request: ReceiveOpeningPortfolioSourceArtifactReceiptAttemptRequest = {
      expected_claim_sha256: claim.claim_sha256,
      expected_authorization_review_sha256: review.review_sha256,
      expected_isolated_receiver_spec_sha256: review.receiver.isolated_receiver_spec_sha256,
      expected_receiver_contract_sha256: receiverContract.contract_sha256,
      expected_receiver_artifact_sha256: review.server_computed_artifact_sha256,
      expected_artifact_manifest_sha256: review.artifact_manifest.manifest_sha256,
      expected_implementation_contract_sha256: implementationContract.contract_sha256,
      expected_stage_125_specification_sha256: implementationContract.stage_125_specification_sha256,
      provider_statement_or_export_identifier: providerIdentifier().trim(),
      provider_generated_at_or_statement_as_of: new Date(providerAt()).toISOString(),
      artifacts: files().map((file) => ({ declared_format: formatFor(file) as DeclaredFormat, source_account_aliases: aliasesList() })),
      execution_reason: reason().trim(),
      exact_current_stage_51_through_stage_131_binding_confirmed: values[0] as boolean,
      executor_independent_from_complete_prior_chain_and_stage_131_claimant_confirmed: values[1] as boolean,
      start_marker_consumes_claim_before_first_source_byte_confirmed: values[2] as boolean,
      administrator_authenticated_stream_only_no_remote_fetch_confirmed: values[3] as boolean,
      original_artifacts_already_account_pseudonymized_and_credentials_removed_confirmed: values[4] as boolean,
      format_magic_safe_structure_archive_active_content_password_symlink_and_path_rejection_confirmed: values[5] as boolean,
      streaming_sha256_length_private_quarantine_and_atomic_content_addressed_commit_confirmed: values[6] as boolean,
      encryption_at_rest_and_redacted_manifest_confirmed: values[7] as boolean,
      duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed: values[8] as boolean,
      receipt_create_once_untrusted_and_stage_133_independent_validation_required_confirmed: values[9] as boolean,
      no_financial_row_parsing_snapshot_materialization_or_snapshot_admission_confirmed: values[10] as boolean,
      no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: values[11] as boolean,
      one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: values[12] as boolean,
      no_unconfirmed_hari_or_old_wang_logic_claimed: values[13] as boolean,
    };
    setBusy(true); setError(""); setNotice("");
    try {
      setRegistry(await receiveOpeningPortfolioSourceArtifactReceiptAttemptOnce(claim.attempt_id, request, files()));
      setProviderIdentifier(""); setAliases(""); setReason(""); setFiles([]); setChecks(CHECKS.map(() => false));
      setNotice("来源工件已加密保存并生成未受信 receipt；它还不是期初持仓，必须等待 Stage 133 独立校验。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 132 单次接收失败；若开始标记已落盘，本次 claim 已永久终止");
      await load();
    } finally { setBusy(false); }
  };

  return <Show when={registry()}>{(current) => <section class="public-admin-reward-governance" aria-label="来源工件单次加密接收">
    <header><strong>第 132 阶段 · 来源工件单次加密接收</strong><span>{current().next_gate}</span></header>
    <p>{current().scope}</p>
    <div class="public-admin-decision-metrics">
      <div><span>待接收</span><strong>{current().pending_claim_count}</strong></div>
      <div><span>未受信 receipt</span><strong>{current().successful_untrusted_receipt_count}</strong></div>
      <div><span>失败已消费</span><strong>{current().failed_consumed_claim_count}</strong></div>
      <div><span>加密密钥</span><strong>{current().encryption_key_configured ? "已就绪" : "未配置"}</strong></div>
    </div>
    <Show when={!current().encryption_key_configured}><p class="public-admin-error">未配置独立来源工件加密密钥。服务端会在开始标记之前拒绝接收，不会读取文件。</p></Show>
    <Show when={current().pending_claims.length > 0} fallback={<p>当前没有尚未开始的 Stage 131 claim；零状态或已有终态符合预期。</p>}>
      <label><span>Stage 131 claim</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={current().pending_claims}>{(claim) => <option value={claim.attempt_id}>{claim.authorization.receiver.receiver_name} · {claim.attempt_id.slice(0, 8)}</option>}</For></select></label>
      <label><span>提供方导出/对账单标识（不得填账号）</span><input value={providerIdentifier()} onInput={(event) => setProviderIdentifier(event.currentTarget.value)} placeholder="例如 statement-2026Q2" /></label>
      <label><span>提供方生成时间 / 对账单时点</span><input type="datetime-local" value={providerAt()} onInput={(event) => setProviderAt(event.currentTarget.value)} /></label>
      <label><span>脱敏账户别名（逗号分隔）</span><input value={aliases()} onInput={(event) => setAliases(event.currentTarget.value)} placeholder="例如 broker_main, retirement" /></label>
      <label><span>PDF / CSV / JSON（合计不超过 256 MiB）</span><input type="file" multiple accept=".pdf,.csv,.json,application/pdf,text/csv,application/json" onChange={(event) => setFiles(Array.from(event.currentTarget.files ?? []))} /></label>
      <Show when={files().length > 0}><p>已选 {files().length} 个文件；原文件名不会进入服务端 receipt 或存储路径。</p></Show>
      <label><span>接收原因</span><textarea value={reason()} onInput={(event) => setReason(event.currentTarget.value)} /></label>
      <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((items) => items.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
      <button type="button" class="public-admin-decision-submit" disabled={busy() || !ready()} onClick={() => void submit()}>{busy() ? "正在单次加密接收…" : "开始一次性接收（失败不可重试）"}</button>
    </Show>
    <Show when={current().results.length > 0}><div class="public-admin-review-history"><For each={current().results}>{(result) => <article><strong>{result.status === "completed_with_untrusted_receipt" ? "未受信 receipt" : "失败已消费"}</strong><span>{result.stage_131_attempt_id.slice(0, 8)} · {result.artifact_count} 件 · {result.bounded_error_code ?? "等待 Stage 133"}</span></article>}</For></div></Show>
    <Show when={error()}><p class="public-admin-error">{error()}</p></Show><Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
  </section>}</Show>;
}
