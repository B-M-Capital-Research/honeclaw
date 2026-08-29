import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getOpeningPortfolioSourceArtifactReceiptIsolatedReceivers,
  registerOpeningPortfolioSourceArtifactReceiptIsolatedReceiver,
} from "@/lib/api";
import type { OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRegistry } from "@/lib/types";

const CHECKS = [
  "精确绑定当前 Stage 51–128 完整责任链",
  "登记人独立于 Stage 128 复核人及完整前序责任链",
  "已重算 Stage 125–128 规格、登记、实现、审计和复核摘要",
  "只绑定未来工件身份、代码版本和复现步骤，当前工件不存在",
  "完整继承 8 个接收函数及原始 PDF/CSV/JSON 格式",
  "完整继承单件 64 MiB、单 receipt 256 MiB、最多 64 件上限",
  "未来只允许管理员鉴权流式传输，禁止远程 URL 抓取",
  "未来私有隔离、流式 SHA-256/长度及原子 create-new",
  "未来拒绝格式/魔数异常、主动内容、归档、密码、符号链接和路径穿越",
  "未来要求账户匿名化、凭据脱敏、静态加密及脱敏 manifest",
  "未来输入只读且内容寻址，输出 create-once 且未受信",
  "接收校验、快照物化、输出校验与准入继续分离",
  "固定非特权身份、只读根目录、临时工作目录和资源上限",
  "当前无上传、来源字节、工件、入口、runtime、输入、环境、secret、网络、工具、子进程或生产 I/O",
  "当前无快照、金融白名单、账本、持仓、现金、净值/绩效、模型、训练/RL、订单、券商或交易权限",
  "登记只开放 Stage 130 责任链外首次执行授权复核",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminOpeningPortfolioSourceArtifactReceiptIsolatedReceiverPanel() {
  const [registry, setRegistry] = createSignal<OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [name, setName] = createSignal("期初组合来源工件隔离接收器");
  const [revision, setRevision] = createSignal("stage-129-v1");
  const [codeRevision, setCodeRevision] = createSignal("");
  const [artifactSha, setArtifactSha] = createSignal("");
  const [reproduction, setReproduction] = createSignal("");
  const [rationale, setRationale] = createSignal("");
  const [limitations, setLimitations] = createSignal("当前没有可执行工件、上传入口、来源字节、runtime 或输入访问。");
  const [input, setInput] = createSignal("未来只允许管理员鉴权流式提交原始 PDF/CSV/JSON；禁止远程抓取和客户端路径。");
  const [output, setOutput] = createSignal("未来只允许 create-once 未受信 receipt manifest；必须经过独立校验后才能进入快照物化。");
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const eligible = createMemo(() => registry()?.eligible_implementations ?? []);
  const selected = createMemo(() => eligible().find((item) => item.implementation.implementation_id === selectedId()));
  const load = async () => {
    try {
      const next = await getOpeningPortfolioSourceArtifactReceiptIsolatedReceivers();
      setRegistry(next);
      if (!next.eligible_implementations.some((item) => item.implementation.implementation_id === selectedId())) {
        setSelectedId(next.eligible_implementations[0]?.implementation.implementation_id ?? "");
      }
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 129 隔离接收器规格读取失败");
    }
  };
  onMount(() => void load());

  const disabled = createMemo(() => busy() || !selected() || artifactSha().trim().length !== 64
    || [name(), revision(), codeRevision(), reproduction(), rationale(), limitations(), input(), output()].some((value) => value.trim().length === 0)
    || !checks().every(Boolean));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const implementation = item.implementation;
    const review = item.review;
    const upstream = implementation.upstream_stage_126_review;
    setBusy(true); setError(""); setNotice("");
    try {
      const values = checks();
      const next = await registerOpeningPortfolioSourceArtifactReceiptIsolatedReceiver(implementation.implementation_id, {
        expected_stage_128_review_id: review.review_id,
        expected_stage_128_review_sha256: review.review_sha256,
        expected_stage_128_independent_audit_sha256: review.independent_audit.audit_sha256,
        expected_stage_127_implementation_id: implementation.implementation_id,
        expected_stage_127_implementation_sha256: implementation.implementation_sha256,
        expected_stage_127_implementation_contract_sha256: implementation.implementation_contract.contract_sha256,
        expected_stage_126_review_sha256: upstream.review_sha256,
        expected_stage_126_independent_audit_sha256: upstream.independent_audit.audit_sha256,
        expected_stage_125_registration_sha256: upstream.registration.registration_sha256,
        expected_stage_125_specification_sha256: upstream.registration.specification.specification_sha256,
        receiver_name: name().trim(),
        receiver_kind: "ephemeral_deterministic_stream_only_receipt_specification",
        receiver_spec_revision: revision().trim(),
        proposed_receiver_code_revision: codeRevision().trim(),
        proposed_receiver_artifact_sha256: artifactSha().trim().toLowerCase(),
        artifact_reproduction_procedure: reproduction().trim(), rationale: rationale().trim(),
        known_limitations: limitations().trim(), future_input_constraints: input().trim(), future_output_constraints: output().trim(),
        exact_current_stage_51_through_stage_128_binding_confirmed: values[0] as boolean,
        registrar_independent_from_stage_128_reviewer_and_complete_prior_chain_confirmed: values[1] as boolean,
        review_audit_implementation_contract_registration_and_specification_hashes_reproduced_confirmed: values[2] as boolean,
        proposed_artifact_identity_revision_and_reproduction_bound_but_artifact_absent_confirmed: values[3] as boolean,
        all_eight_receipt_functions_and_original_pdf_csv_json_formats_preserved_confirmed: values[4] as boolean,
        exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: values[5] as boolean,
        future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: values[6] as boolean,
        future_private_quarantine_streaming_sha256_length_and_atomic_create_new_confirmed: values[7] as boolean,
        future_magic_safe_structure_active_content_archive_password_symlink_and_path_rejection_confirmed: values[8] as boolean,
        future_account_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed: values[9] as boolean,
        future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: values[10] as boolean,
        future_receipt_validation_snapshot_materialization_output_validation_and_admission_separate_confirmed: values[11] as boolean,
        fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: values[12] as boolean,
        no_upload_source_bytes_artifact_entrypoint_runtime_input_environment_secret_network_tool_subprocess_or_production_io_confirmed: values[13] as boolean,
        no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: values[14] as boolean,
        registration_only_opens_stage_130_chain_external_first_execution_authorization_review_confirmed: values[15] as boolean,
        no_unconfirmed_hari_or_old_wang_logic_claimed: values[16] as boolean,
      });
      setRegistry(next); setChecks(CHECKS.map(() => false));
      setNotice("Stage 129 规格已登记；仍无上传或执行能力，只开放 Stage 130 首次执行授权复核。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 129 隔离接收器规格登记失败");
      await load();
    } finally { setBusy(false); }
  };

  return <Show when={registry()}>{(current) => <section class="public-admin-reward-governance" aria-label="期初组合来源工件隔离接收器规格登记">
    <header><strong>第 129 阶段 · 隔离来源工件接收器规格</strong><span>只登记 · 不执行</span></header>
    <p>{current().scope}</p>
    <p class="public-admin-anchor-boundary">当前明确为空：上传入口、来源字节、接收器工件、runtime、输入读取、receipt、期初组合、账本、持仓、现金、净值/绩效、训练、订单与交易权限。</p>
    <div class="public-admin-decision-metrics">
      <div><span>可登记</span><strong>{current().registration_eligible_count}</strong></div>
      <div><span>已登记</span><strong>{current().isolated_receiver_count}</strong></div>
      <div><span>当前绑定</span><strong>{current().current_binding_receiver_count}</strong></div>
      <div><span>Stage 130 候选</span><strong>{current().first_execution_authorization_review_eligible_count}</strong></div>
    </div>
    <Show when={eligible().length > 0} fallback={<p>当前没有已通过 Stage 128 独立复核且尚未登记的实现。</p>}>
      <label><span>Stage 128 独立批准</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={eligible()}>{(item) => <option value={item.implementation.implementation_id}>{item.implementation.implementation_name} · {item.review.review_id.slice(0, 12)}…</option>}</For></select></label>
      <label><span>接收器名称</span><input value={name()} onInput={(event) => setName(event.currentTarget.value)} /></label>
      <label><span>规格版本</span><input value={revision()} onInput={(event) => setRevision(event.currentTarget.value)} /></label>
      <label><span>不可变代码版本</span><input value={codeRevision()} onInput={(event) => setCodeRevision(event.currentTarget.value)} /></label>
      <label><span>未来工件 SHA-256（当前工件仍不存在）</span><input value={artifactSha()} onInput={(event) => setArtifactSha(event.currentTarget.value)} /></label>
      <label><span>未来工件复现步骤</span><textarea value={reproduction()} onInput={(event) => setReproduction(event.currentTarget.value)} /></label>
      <label><span>登记理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
      <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
      <label><span>未来输入约束</span><textarea value={input()} onInput={(event) => setInput(event.currentTarget.value)} /></label>
      <label><span>未来输出约束</span><textarea value={output()} onInput={(event) => setOutput(event.currentTarget.value)} /></label>
      <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
      <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在追加规格…" : "登记 Stage 129 隔离接收器规格"}</button>
    </Show>
    <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
    <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
  </section>}</Show>;
}
