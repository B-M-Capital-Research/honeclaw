import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizations,
  reviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorization,
} from "@/lib/api";
import type {
  OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRegistry,
  OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationVerdict,
} from "@/lib/types";

const CHECKS = [
  "精确绑定当前 Stage 51–129 完整责任链",
  "复核者独立于 Stage 129 登记人、工件构建者及完整前序责任链",
  "服务端已重新计算只读常规工件 SHA-256，摘要和长度完全匹配",
  "自哈希 manifest、代码版本、runtime 与复现步骤摘要完全匹配",
  "接收器工件构建者与 Stage 130 复核者相互分离",
  "8 个来源接收函数及原始 PDF/CSV/JSON 格式继续由原合同绑定",
  "单件 64 MiB、单 receipt 256 MiB、最多 64 件上限保持不变",
  "未来只允许管理员鉴权流式提交，禁止远程 URL 抓取",
  "未来私有隔离、哈希/长度/魔数/结构校验及原子 create-new 保持不变",
  "未来账户匿名化、凭据脱敏、静态加密及脱敏 manifest 保持不变",
  "未来输入只读且内容寻址，输出 create-once 且未受信",
  "接收校验、快照物化、输出校验与准入严格分离",
  "固定非特权身份、只读根目录、临时工作区和资源上限保持不变",
  "授权 24 小时、仅一次，并与 Stage 131 claim 严格分离",
  "当前无上传、来源字节、runtime、挂载/读取、receipt 或快照",
  "无环境继承、secret、网络、工具、子进程或生产 I/O",
  "无金融白名单、账本、持仓、现金、净值/绩效、模型、训练/RL、reward、订单、券商或交易权限",
  "批准只开放未来 Stage 131 claim-first 单次尝试候选",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationPanel() {
  const [registry, setRegistry] = createSignal<OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [verdict, setVerdict] = createSignal<OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationVerdict>("changes_requested_rebuild_artifact");
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [artifactEvidence, setArtifactEvidence] = createSignal("");
  const [sandboxEvidence, setSandboxEvidence] = createSignal("");
  const [rationale, setRationale] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizations();
      setRegistry(next);
      if (!next.items.some((item) => item.receiver.isolated_receiver_id === selectedId())) {
        setSelectedId(next.items[0]?.receiver.isolated_receiver_id ?? "");
      }
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 130 来源接收器首次执行授权读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.items.find((item) => item.receiver.isolated_receiver_id === selectedId()));
  const approving = createMemo(() => verdict() === "approved_for_one_future_claim_first_source_artifact_receipt_attempt");
  const disabled = createMemo(() => busy() || !selected()?.artifact_inspection.artifact_verified
    || !selected()?.artifact_inspection.manifest || artifactEvidence().trim().length === 0
    || sandboxEvidence().trim().length === 0 || rationale().trim().length === 0
    || (approving() && !checks().every(Boolean)));

  const submit = async () => {
    const item = selected();
    const manifest = item?.artifact_inspection.manifest;
    if (!item || !manifest || disabled()) return;
    const receiver = item.receiver;
    const contract = receiver.receiver_contract;
    const implementation = receiver.implementation;
    const source = implementation.implementation_contract;
    const review = receiver.implementation_review;
    const values = checks();
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await reviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorization(receiver.isolated_receiver_id, {
        expected_review_id: item.latest_review?.review_id,
        expected_review_sha256: item.latest_review?.review_sha256,
        expected_isolated_receiver_id: receiver.isolated_receiver_id,
        expected_isolated_receiver_spec_sha256: receiver.isolated_receiver_spec_sha256,
        expected_receiver_contract_sha256: contract.contract_sha256,
        expected_receiver_spec_revision: contract.receiver_spec_revision,
        expected_receiver_code_revision: contract.proposed_receiver_code_revision,
        expected_receiver_artifact_sha256: contract.proposed_receiver_artifact_sha256,
        expected_stage_128_review_id: review.review_id,
        expected_stage_128_review_sha256: review.review_sha256,
        expected_stage_128_independent_audit_sha256: review.independent_audit.audit_sha256,
        expected_stage_127_implementation_sha256: implementation.implementation_sha256,
        expected_stage_127_implementation_contract_sha256: source.contract_sha256,
        expected_stage_126_review_sha256: source.stage_126_review_sha256,
        expected_stage_125_registration_sha256: source.stage_125_registration_sha256,
        expected_stage_125_specification_sha256: source.stage_125_specification_sha256,
        expected_artifact_manifest_sha256: manifest.manifest_sha256,
        artifact_reproduction_review_evidence: artifactEvidence().trim(),
        sandbox_contract_review_evidence: sandboxEvidence().trim(),
        verdict: verdict(), rationale: rationale().trim(),
        exact_current_stage_51_through_stage_129_binding_confirmed: values[0] as boolean,
        reviewer_independent_from_stage_129_registrar_builder_and_complete_prior_chain_confirmed: values[1] as boolean,
        server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: values[2] as boolean,
        self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: values[3] as boolean,
        artifact_builder_and_reviewer_separation_confirmed: values[4] as boolean,
        all_eight_receipt_functions_and_original_pdf_csv_json_formats_remain_bound_confirmed: values[5] as boolean,
        exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: values[6] as boolean,
        future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: values[7] as boolean,
        future_private_quarantine_hash_length_magic_structure_and_atomic_create_new_confirmed: values[8] as boolean,
        future_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed: values[9] as boolean,
        future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: values[10] as boolean,
        future_receipt_validation_snapshot_materialization_validation_and_admission_separate_confirmed: values[11] as boolean,
        fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: values[12] as boolean,
        authorization_single_use_24_hour_expiry_and_stage_131_claim_separation_confirmed: values[13] as boolean,
        no_upload_source_bytes_runtime_mount_input_read_receipt_or_snapshot_created_confirmed: values[14] as boolean,
        no_environment_secret_network_tool_subprocess_or_production_io_confirmed: values[15] as boolean,
        no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: values[16] as boolean,
        approval_only_opens_future_stage_131_claim_first_attempt_confirmed: values[17] as boolean,
        no_unconfirmed_hari_or_old_wang_logic_claimed: values[18] as boolean,
      });
      setRegistry(next); setChecks(CHECKS.map(() => false));
      setArtifactEvidence(""); setSandboxEvidence(""); setRationale("");
      setNotice(approving() ? "已签发 24 小时内一次性的未来 Stage 131 claim 候选；本次未接收或读取任何来源文件。" : "复核已追加保存；没有开放接收或执行能力。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 130 来源接收器首次执行授权复核失败");
      await load();
    } finally { setBusy(false); }
  };

  return <Show when={registry()}>{(current) => <section class="public-admin-reward-governance" aria-label="来源工件接收器首次执行授权独立复核">
    <header><strong>第 130 阶段 · 来源工件接收器首次执行授权</strong><span>{current().authorization_status}</span></header>
    <p>{current().scope}</p>
    <div class="public-admin-decision-metrics">
      <div><span>待真实工件</span><strong>{current().artifact_pending_receiver_count}</strong></div>
      <div><span>服务端已核验</span><strong>{current().artifact_verified_receiver_count}</strong></div>
      <div><span>已复核</span><strong>{current().reviewed_receiver_count}</strong></div>
      <div><span>Stage 131 候选</span><strong>{current().future_claim_eligible_count}</strong></div>
    </div>
    <article class="public-admin-reward-governance"><header><strong>不能手填路径或只提交 SHA</strong><span>服务端重哈希 · 24 小时 · 一次</span></header>
      <p>工件和 manifest 必须位于服务端派生的内容寻址保管位置；符号链接、可写文件、摘要或长度不一致一律失败关闭。</p>
      <p class="public-admin-anchor-boundary">本阶段没有上传端点，不读取来源文件，不启动 runtime，也不创建 receipt、快照或财务状态。</p>
    </article>
    <Show when={current().items.length > 0} fallback={<p>当前没有可进入 Stage 130 的 Stage 129 接收器；零状态符合预期。</p>}>
      <label><span>Stage 129 接收器</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={current().items}>{(item) => <option value={item.receiver.isolated_receiver_id}>{item.receiver.receiver_name} · {item.artifact_inspection.status}</option>}</For></select></label>
      <Show when={selected()}>{(item) => <article class="public-admin-reward-governance"><header><strong>服务端内容寻址工件</strong><span>{item().artifact_inspection.artifact_verified ? "核验通过" : "尚不可复核"}</span></header><p>{item().artifact_inspection.custody_locator}</p><Show when={item().artifact_inspection.manifest}>{(manifest) => <p>构建者 {manifest().reproduced_by} · 代码 {manifest().receiver_code_revision} · {manifest().artifact_byte_length} bytes</p>}</Show></article>}</Show>
      <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationVerdict)}><option value="changes_requested_rebuild_artifact">要求修改并重建工件</option><option value="rejected">拒绝</option><option value="approved_for_one_future_claim_first_source_artifact_receipt_attempt">批准未来一次 claim-first 尝试候选</option></select></label>
      <label><span>工件复现复核证据</span><textarea value={artifactEvidence()} onInput={(event) => setArtifactEvidence(event.currentTarget.value)} /></label>
      <label><span>隔离合同复核证据</span><textarea value={sandboxEvidence()} onInput={(event) => setSandboxEvidence(event.currentTarget.value)} /></label>
      <label><span>复核依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
      <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((currentChecks) => currentChecks.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
      <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在保存复核…" : "保存 Stage 130 独立复核"}</button>
    </Show>
    <Show when={error()}><p class="public-admin-error">{error()}</p></Show><Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
  </section>}</Show>;
}
