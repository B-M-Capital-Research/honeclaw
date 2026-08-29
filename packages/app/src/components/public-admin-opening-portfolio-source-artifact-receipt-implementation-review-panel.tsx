import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getOpeningPortfolioSourceArtifactReceiptImplementationReviews,
  reviewOpeningPortfolioSourceArtifactReceiptImplementation,
} from "@/lib/api";
import type {
  OpeningPortfolioSourceArtifactReceiptImplementationReviewConfirmations,
  OpeningPortfolioSourceArtifactReceiptImplementationReviewRegistry,
  OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–127 完整责任链",
  "复核人独立于 Stage 127 登记人和完整前序责任链",
  "已独立重算实现、合同、Stage 126 复核/审计及 Stage 125 登记/规格摘要",
  "完整合同由第二实现重建，未调用 Stage 127 builder 自证",
  "已重新校验 Stage 127 全部 17 项登记确认",
  "提供方原始格式与 64 MiB/256 MiB/64 件资源上限保持不变",
  "未来只允许管理员鉴权流式传输，禁止远程 URL 抓取",
  "流式 SHA-256、长度、私有隔离与原子提交合同完整",
  "格式、魔数、安全结构和主动内容拒绝合同完整",
  "账户匿名化与凭据、路径、元数据和日志脱敏合同完整",
  "静态加密、内容寻址、create-new、幂等与失败清理合同完整",
  "服务端接收时间、脱敏 manifest 与未受信 receipt 合同完整",
  "接收校验、快照物化、输出校验与准入继续分离",
  "当前无上传、来源字节、存储写入、parser/runtime、网络、secret、工具或子进程",
  "当前无快照、金融白名单、账本、持仓、现金、净值/绩效、模型、训练/RL、订单、券商或交易权限",
  "批准只开放 Stage 129 隔离接收器规格登记",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

function confirmations(values: boolean[]): OpeningPortfolioSourceArtifactReceiptImplementationReviewConfirmations {
  return {
    exact_current_stage_51_through_stage_127_binding_confirmed: values[0] as boolean,
    reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: values[1] as boolean,
    implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: values[2] as boolean,
    complete_contract_rebuilt_without_stage_127_builder_confirmed: values[3] as boolean,
    all_stage_127_registration_confirmations_revalidated_confirmed: values[4] as boolean,
    original_provider_formats_and_resource_ceilings_preserved_confirmed: values[5] as boolean,
    administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: values[6] as boolean,
    streaming_sha256_length_private_quarantine_and_atomic_commit_confirmed: values[7] as boolean,
    format_magic_safe_structure_and_active_content_rejection_confirmed: values[8] as boolean,
    account_pseudonymization_and_secret_redaction_confirmed: values[9] as boolean,
    encryption_content_addressing_create_new_idempotency_and_failure_cleanup_confirmed: values[10] as boolean,
    server_received_time_redacted_manifest_and_untrusted_receipt_confirmed: values[11] as boolean,
    receipt_validation_materialization_output_validation_and_admission_remain_separate_confirmed: values[12] as boolean,
    no_upload_source_bytes_storage_write_parser_runtime_network_secret_tool_or_subprocess_confirmed: values[13] as boolean,
    no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: values[14] as boolean,
    approval_only_opens_future_stage_129_isolated_receiver_specification_registration_confirmed: values[15] as boolean,
    no_unconfirmed_hari_or_old_wang_logic_claimed: values[16] as boolean,
  };
}

export function PublicAdminOpeningPortfolioSourceArtifactReceiptImplementationReviewPanel() {
  const [registry, setRegistry] = createSignal<OpeningPortfolioSourceArtifactReceiptImplementationReviewRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [verdict, setVerdict] = createSignal<OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict>(
    "approved_for_future_isolated_source_artifact_receiver_specification_registration",
  );
  const [rationale, setRationale] = createSignal("");
  const [binding, setBinding] = createSignal("");
  const [transport, setTransport] = createSignal("");
  const [privacy, setPrivacy] = createSignal("");
  const [separation, setSeparation] = createSignal("");
  const [limitations, setLimitations] = createSignal("尚未建立上传入口，也没有接收、读取、存储或解析任何来源字节。");
  const [constraints, setConstraints] = createSignal("Stage 129 只能登记隔离接收器规格；不得据此接收文件、生成快照或写入任何财务状态。");
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const eligible = createMemo(() => registry()?.items.filter((item) => item.review_eligible) ?? []);
  const selected = createMemo(() => eligible().find((item) => item.implementation.implementation_id === selectedId()));

  const load = async () => {
    try {
      const next = await getOpeningPortfolioSourceArtifactReceiptImplementationReviews();
      setRegistry(next);
      const candidates = next.items.filter((item) => item.review_eligible);
      if (!candidates.some((item) => item.implementation.implementation_id === selectedId())) {
        setSelectedId(candidates[0]?.implementation.implementation_id ?? "");
      }
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 128 来源工件接收实现独立复核读取失败");
    }
  };
  onMount(() => void load());

  const disabled = createMemo(() => busy() || !selected() || [rationale(), binding(), transport(), privacy(), separation(), limitations(), constraints()]
    .some((value) => value.trim().length === 0)
    || (verdict().startsWith("approved") && !checks().every(Boolean)));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    setBusy(true); setError(""); setNotice("");
    try {
      const implementation = item.implementation;
      const upstream = implementation.upstream_stage_126_review;
      const next = await reviewOpeningPortfolioSourceArtifactReceiptImplementation(
        implementation.implementation_id,
        {
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_contract_sha256: implementation.implementation_contract.contract_sha256,
          expected_stage_126_review_sha256: upstream.review_sha256,
          expected_stage_126_independent_audit_sha256: upstream.independent_audit.audit_sha256,
          expected_stage_125_registration_sha256: upstream.registration.registration_sha256,
          expected_stage_125_specification_sha256: upstream.registration.specification.specification_sha256,
          expected_independent_audit_sha256: item.current_independent_audit.audit_sha256,
          verdict: verdict(),
          rationale: rationale().trim(),
          binding_and_recomputation_assessment: binding().trim(),
          transport_resource_and_format_assessment: transport().trim(),
          privacy_storage_and_manifest_assessment: privacy().trim(),
          separation_and_zero_capability_assessment: separation().trim(),
          known_limitations: limitations().trim(),
          future_receiver_constraints: constraints().trim(),
          confirmations: confirmations(checks()),
        },
      );
      setRegistry(next); setRationale(""); setChecks(REVIEW_CHECKS.map(() => false));
      setNotice(verdict().startsWith("approved")
        ? "Stage 128 已独立批准；仍没有上传入口或来源数据，只开放 Stage 129 隔离接收器规格登记。"
        : "复核结论已终结保存；原 Stage 127 实现必须重建后才能再次进入复核链。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 128 来源工件接收实现独立复核提交失败");
      await load();
    } finally { setBusy(false); }
  };

  return <Show when={registry()}>{(current) => <section class="public-admin-reward-governance" aria-label="期初组合来源工件接收实现独立复核">
    <header><strong>第 128 阶段 · 来源工件接收实现独立复核</strong><span>第二实现 · 无上传</span></header>
    <p>{current().scope}</p>
    <p class="public-admin-anchor-boundary">当前明确为空：上传入口、来源字节、parser/runtime、期初组合、账本、持仓、现金、净值/绩效、训练、订单与交易权限。</p>
    <div class="public-admin-decision-metrics">
      <div><span>待复核</span><strong>{current().review_eligible_count}</strong></div>
      <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
      <div><span>独立批准</span><strong>{current().independently_approved_count}</strong></div>
      <div><span>Stage 129 候选</span><strong>{current().future_stage_129_isolated_receiver_specification_registration_eligible_count}</strong></div>
    </div>
    <Show when={eligible().length > 0} fallback={<p>当前没有待复核的 Stage 127 零能力实现合同。</p>}>
      <label><span>Stage 127 实现合同</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}>
        <For each={eligible()}>{(item) => <option value={item.implementation.implementation_id}>{item.implementation.implementation_name} · {item.implementation.implementation_id.slice(0, 12)}…</option>}</For>
      </select></label>
      <label><span>裁决</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict)}>
        <option value="approved_for_future_isolated_source_artifact_receiver_specification_registration">批准进入 Stage 129 隔离接收器规格登记</option>
        <option value="changes_required_rebuild_source_artifact_receipt_implementation">要求重建 Stage 127 实现</option>
        <option value="rejected_source_artifact_receipt_implementation">拒绝该实现</option>
      </select></label>
      <label><span>复核理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
      <label><span>责任链与摘要重算</span><textarea value={binding()} onInput={(event) => setBinding(event.currentTarget.value)} /></label>
      <label><span>传输、资源与格式</span><textarea value={transport()} onInput={(event) => setTransport(event.currentTarget.value)} /></label>
      <label><span>隐私、存储与 manifest</span><textarea value={privacy()} onInput={(event) => setPrivacy(event.currentTarget.value)} /></label>
      <label><span>分离关卡与零能力边界</span><textarea value={separation()} onInput={(event) => setSeparation(event.currentTarget.value)} /></label>
      <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
      <label><span>未来接收器约束</span><textarea value={constraints()} onInput={(event) => setConstraints(event.currentTarget.value)} /></label>
      <div class="public-admin-decision-checks"><For each={REVIEW_CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
      <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在追加独立复核…" : "提交 Stage 128 独立复核"}</button>
    </Show>
    <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
    <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
  </section>}</Show>;
}
