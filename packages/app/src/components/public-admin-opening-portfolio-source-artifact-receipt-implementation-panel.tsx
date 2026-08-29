import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getOpeningPortfolioSourceArtifactReceiptImplementations,
  registerOpeningPortfolioSourceArtifactReceiptImplementation,
} from "@/lib/api";
import type { OpeningPortfolioSourceArtifactReceiptImplementationRegistry } from "@/lib/types";

const IMPLEMENTATION_CHECKS = [
  "精确绑定当前 Stage 51–126 完整责任链",
  "登记人独立于 Stage 126 复核者和全部既有责任人",
  "已重新计算复核、登记、规格与独立审计摘要",
  "完整保留 Stage 125 来源合同与三类原始工件格式",
  "未来原始字节只流式接收一次，原子提交前同时计算 SHA-256 与长度",
  "只检查 content type、魔数、安全结构与提供方元数据，不解析财务行",
  "拒绝压缩包、主动内容、密码保护、符号链接与路径穿越",
  "账号先匿名化，真实账号和凭据永不进入持久化元数据、路径或日志",
  "使用私有隔离区、静态加密、create-new 写入，并在失败时清理残留",
  "接收时间由服务端生成，并保存提供方身份和内容寻址 manifest",
  "相同内容幂等且不可覆盖；修正必须提交新的外部工件",
  "未来 receipt 仍为未受信结果，必须独立校验",
  "接收、快照物化、输出校验和快照准入继续保持分离",
  "本阶段只有合同，没有上传入口、工件、入口程序、runtime、网络、secret 或 parser",
  "不创建快照、金融白名单、账本、持仓、现金、净值/绩效、训练/RL、订单、券商或交易权限",
  "登记后必须进入 Stage 128 责任链外独立实现复核",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminOpeningPortfolioSourceArtifactReceiptImplementationPanel() {
  const [registry, setRegistry] =
    createSignal<OpeningPortfolioSourceArtifactReceiptImplementationRegistry>();
  const [selectedReviewId, setSelectedReviewId] = createSignal("");
  const [implementationName, setImplementationName] = createSignal(
    "期初组合私密来源工件接收器",
  );
  const [revision, setRevision] = createSignal("");
  const [description, setDescription] = createSignal(
    "只登记未来接收器的不可执行、零能力合同；当前不创建上传入口或来源工件。",
  );
  const [transport, setTransport] = createSignal(
    "仅服务端权威管理员鉴权的流式传输；禁止远程 URL 抓取和客户端指定存储路径。",
  );
  const [streaming, setStreaming] = createSignal(
    "进入私有临时隔离区时同步计算 SHA-256 与字节长度，全部校验通过后按内容寻址原子 create-new。",
  );
  const [format, setFormat] = createSignal(
    "核对声明格式、MIME、魔数和安全结构；拒绝主动内容、压缩包与密码保护，不解析账户财务行。",
  );
  const [privacy, setPrivacy] = createSignal(
    "真实账号先转换为不可逆别名；账号、凭据与原始敏感字段不得进入路径、元数据、错误信息或日志。",
  );
  const [quarantine, setQuarantine] = createSignal(
    "私有隔离区静态加密，失败或中断清理部分文件；重复摘要幂等，禁止覆盖既有工件。",
  );
  const [audit, setAudit] = createSignal(
    "只生成脱敏、append-only、自哈希且未受信的 receipt manifest；原始字节保持不可变。",
  );
  const [limitations, setLimitations] = createSignal(
    "当前没有上传入口、来源字节、parser、期初组合、持仓、现金或净值。",
  );
  const [constraints, setConstraints] = createSignal(
    "Stage 128 必须由责任链外人员独立复核实现合同；复核通过前不得建立接收器或读取任何来源字节。",
  );
  const [checks, setChecks] = createSignal(IMPLEMENTATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const eligible = createMemo(() => registry()?.items.filter((item) => item.registration_eligible) ?? []);
  const selected = createMemo(() => eligible().find(
    (item) => item.specification_review.review_id === selectedReviewId(),
  ));

  const load = async () => {
    try {
      const next = await getOpeningPortfolioSourceArtifactReceiptImplementations();
      setRegistry(next);
      const nextEligible = next.items.filter((item) => item.registration_eligible);
      if (!nextEligible.some((item) => item.specification_review.review_id === selectedReviewId())) {
        setSelectedReviewId(nextEligible[0]?.specification_review.review_id ?? "");
      }
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 127 来源工件接收实现登记表读取失败");
    }
  };
  onMount(() => void load());

  const disabled = createMemo(() => busy()
    || !selected()
    || implementationName().trim().length === 0
    || revision().trim().length === 0
    || [description(), transport(), streaming(), format(), privacy(), quarantine(), audit(), limitations(), constraints()]
      .some((value) => value.trim().length === 0)
    || !checks().every(Boolean));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const review = item.specification_review;
    const values = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await registerOpeningPortfolioSourceArtifactReceiptImplementation(
        review.review_id,
        {
          expected_stage_126_review_id: review.review_id,
          expected_stage_126_review_sha256: review.review_sha256,
          expected_stage_126_independent_audit_sha256: review.independent_audit.audit_sha256,
          expected_stage_125_registration_id: review.registration.registration_id,
          expected_stage_125_registration_sha256: review.registration.registration_sha256,
          expected_stage_125_specification_sha256: review.registration.specification.specification_sha256,
          implementation_name: implementationName().trim(),
          immutable_code_revision: revision().trim(),
          implementation_description: description().trim(),
          transport_and_authentication_semantics: transport().trim(),
          streaming_hash_length_and_atomic_commit_semantics: streaming().trim(),
          format_magic_and_active_content_rejection_semantics: format().trim(),
          pseudonymization_and_secret_redaction_semantics: privacy().trim(),
          quarantine_cleanup_and_idempotency_semantics: quarantine().trim(),
          audit_and_retention_semantics: audit().trim(),
          known_limitations: limitations().trim(),
          future_review_constraints: constraints().trim(),
          exact_current_stage_51_through_stage_126_binding_confirmed: values[0] as boolean,
          registrar_independent_from_stage_126_reviewer_and_complete_prior_chain_confirmed: values[1] as boolean,
          review_registration_specification_and_audit_hashes_recomputed_confirmed: values[2] as boolean,
          exact_stage_125_source_contract_and_accepted_formats_preserved_confirmed: values[3] as boolean,
          original_bytes_streamed_once_with_sha256_and_length_before_atomic_commit_confirmed: values[4] as boolean,
          content_type_magic_utf8_structure_and_provider_metadata_checked_without_financial_parsing_confirmed: values[5] as boolean,
          archives_active_content_password_protection_symlinks_and_path_traversal_rejected_confirmed: values[6] as boolean,
          source_account_identifiers_pseudonymized_and_raw_accounts_credentials_never_persisted_or_logged_confirmed: values[7] as boolean,
          private_quarantine_encryption_at_rest_create_new_and_failure_cleanup_required_confirmed: values[8] as boolean,
          server_owned_received_time_provider_identity_and_content_addressed_manifest_required_confirmed: values[9] as boolean,
          duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed: values[10] as boolean,
          receipt_output_untrusted_and_independent_receipt_validation_required_confirmed: values[11] as boolean,
          receipt_snapshot_materialization_output_validation_and_snapshot_admission_remain_separate_confirmed: values[12] as boolean,
          contract_only_no_upload_endpoint_artifact_entrypoint_runtime_network_secret_or_parser_confirmed: values[13] as boolean,
          no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: values[14] as boolean,
          future_stage_128_independent_implementation_review_required_confirmed: values[15] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: values[16] as boolean,
        },
      );
      setRegistry(next);
      setChecks(IMPLEMENTATION_CHECKS.map(() => false));
      setNotice("Stage 127 零能力接收实现合同已登记；没有上传或读取任何来源文件。下一步仅为 Stage 128 独立实现复核。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 127 来源工件接收实现登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="期初组合来源工件接收实现登记">
      <header><strong>第 127 阶段 · 来源工件接收实现登记</strong><span>零能力合同 · 无上传入口</span></header>
      <p>{current().scope}</p>
      <p class="public-admin-anchor-boundary">当前明确为空：上传入口、来源字节、parser、期初组合、金融事件白名单、账本、持仓、现金、净值与绩效。</p>
      <div class="public-admin-decision-metrics">
        <div><span>Stage 126 独立批准</span><strong>{current().independently_approved_specification_count}</strong></div>
        <div><span>可登记</span><strong>{current().registration_eligible_count}</strong></div>
        <div><span>已登记合同</span><strong>{current().implementation_contract_count}</strong></div>
        <div><span>Stage 128 候选</span><strong>{current().future_stage_128_independent_implementation_review_eligible_count}</strong></div>
      </div>
      <Show when={eligible().length > 0} fallback={<p>当前没有可登记的 Stage 126 独立批准规格。</p>}>
        <label><span>Stage 126 独立复核</span><select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
          <For each={eligible()}>{(item) => <option value={item.specification_review.review_id}>{item.specification_review.review_id.slice(0, 12)}… · {item.specification_review.registration.specification.source_contract.portfolio_scope_alias}</option>}</For>
        </select></label>
        <label><span>实现名称</span><input value={implementationName()} onInput={(event) => setImplementationName(event.currentTarget.value)} /></label>
        <label><span>不可变代码版本</span><input value={revision()} onInput={(event) => setRevision(event.currentTarget.value)} placeholder="精确 commit / revision；不可填 latest" /></label>
        <label><span>实现说明</span><textarea value={description()} onInput={(event) => setDescription(event.currentTarget.value)} /></label>
        <label><span>传输与鉴权</span><textarea value={transport()} onInput={(event) => setTransport(event.currentTarget.value)} /></label>
        <label><span>流式哈希与原子提交</span><textarea value={streaming()} onInput={(event) => setStreaming(event.currentTarget.value)} /></label>
        <label><span>格式与主动内容拒绝</span><textarea value={format()} onInput={(event) => setFormat(event.currentTarget.value)} /></label>
        <label><span>匿名化与日志脱敏</span><textarea value={privacy()} onInput={(event) => setPrivacy(event.currentTarget.value)} /></label>
        <label><span>隔离、失败清理与幂等</span><textarea value={quarantine()} onInput={(event) => setQuarantine(event.currentTarget.value)} /></label>
        <label><span>审计与保留</span><textarea value={audit()} onInput={(event) => setAudit(event.currentTarget.value)} /></label>
        <label><span>已知限制</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
        <label><span>后续复核约束</span><textarea value={constraints()} onInput={(event) => setConstraints(event.currentTarget.value)} /></label>
        <div class="public-admin-decision-checks"><For each={IMPLEMENTATION_CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((items) => items.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在登记零能力合同…" : "登记 Stage 127 接收实现合同"}</button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().items.filter((item) => item.implementation)}>{(item) => <article class="public-admin-reward-governance">
        <header><strong>{item.implementation?.implementation_name}</strong><span>{item.implementation?.registered_at}</span></header>
        <p>{item.implementation?.implementation_contract.immutable_code_revision} · 单工件上限 {Math.round((item.implementation?.implementation_contract.future_maximum_artifact_bytes ?? 0) / 1024 / 1024)} MiB</p>
        <p class="public-admin-anchor-boundary">合同已登记，等待 Stage 128 独立复核；没有上传入口、来源工件、期初组合或财务写入权限。</p>
      </article>}</For>
    </section>
  )}</Show>;
}
