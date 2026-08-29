import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getOpeningPortfolioSnapshotMaterializationIsolatedMaterializers,
  registerOpeningPortfolioSnapshotMaterializationIsolatedMaterializer,
} from "@/lib/api";
import type {
  OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRegistry,
  RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest,
} from "@/lib/types";

const CHECKS = [
  "精确绑定当前 Stage 51–135 完整责任链",
  "登记人独立于 Stage 135 reviewer 及完整前序责任链",
  "独立复算 review、audit、implementation、contract、validation、result、claim、receipt 与 Stage 125 specification 摘要",
  "只冻结未来工件 SHA、不可变代码版本与复现步骤；当前工件不存在",
  "完整保留十个物化函数、来源合同与 canonical snapshot schema",
  "未来输入只能是 Stage 133 已独立验证、只读、内容寻址的加密 receipt",
  "完整账户、现金、持仓、上市期权、负债、未结算活动；任何缺失或歧义使整份快照失败",
  "精确十进制、有符号数量、证券身份、公司行动与逐行来源语义完整",
  "未来解密仅可在隔离临时内存，禁止明文持久化",
  "PDF/CSV/JSON 解析确定且禁止远程抓取",
  "对账单市场价值仅供参考，不产生 NAV 或绩效",
  "未来输出仅可 create-once、untrusted，并须独立验证与准入",
  "固定非特权身份、只读根目录、临时工作目录和资源上限",
  "当前无源码/可执行工件/入口/runtime/挂载/读取/environment/secret/network/tool/subprocess/生产 IO",
  "当前无快照、金融白名单、账本、持仓/现金、NAV/绩效、模型、训练/RL、reward、订单、券商或交易权限",
  "登记仅开放 Stage 137 责任链外首次执行授权复核",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminOpeningPortfolioSnapshotMaterializationIsolatedMaterializerPanel() {
  const [registry, setRegistry] = createSignal<OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [name, setName] = createSignal("期初组合快照隔离物化器");
  const [revision, setRevision] = createSignal("v1");
  const [codeRevision, setCodeRevision] = createSignal("");
  const [artifactSha, setArtifactSha] = createSignal("");
  const [reproduction, setReproduction] = createSignal("");
  const [rationale, setRationale] = createSignal("只冻结未来物化器身份与零权限边界，不运行任何代码。");
  const [limitations, setLimitations] = createSignal("工件、入口、runtime 与输入均不存在，尚未获得执行授权。");
  const [inputConstraints, setInputConstraints] = createSignal("只接受 Stage 133 独立验证、内容寻址、只读的加密 receipt；解密仅在未来隔离临时内存发生。");
  const [outputConstraints, setOutputConstraints] = createSignal("只允许 create-once 不可信候选；必须另行独立验证与准入，不形成权威财务状态。");
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const eligible = createMemo(() => registry()?.eligible_implementations ?? []);
  const selected = createMemo(() => eligible().find((item) => item.implementation.implementation_id === selectedId()));
  const load = async () => {
    try {
      const next = await getOpeningPortfolioSnapshotMaterializationIsolatedMaterializers();
      setRegistry(next);
      if (!next.eligible_implementations.some((item) => item.implementation.implementation_id === selectedId())) {
        setSelectedId(next.eligible_implementations[0]?.implementation.implementation_id ?? "");
      }
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 136 隔离物化器规格读取失败");
    }
  };
  onMount(() => void load());

  const disabled = createMemo(() => busy() || !selected() || !/^[a-fA-F0-9]{64}$/.test(artifactSha().trim())
    || [name(), revision(), codeRevision(), reproduction(), rationale(), limitations(), inputConstraints(), outputConstraints()].some((value) => value.trim().length === 0)
    || !checks().every(Boolean));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const implementation = item.implementation;
    const review = item.review;
    const contract = implementation.implementation_contract;
    const values = checks();
    const request: RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest = {
      expected_stage_135_review_id: review.review_id,
      expected_stage_135_review_sha256: review.review_sha256,
      expected_stage_135_independent_audit_sha256: review.independent_audit.audit_sha256,
      expected_stage_134_implementation_id: implementation.implementation_id,
      expected_stage_134_implementation_sha256: implementation.implementation_sha256,
      expected_stage_134_implementation_contract_sha256: contract.contract_sha256,
      expected_stage_133_validation_sha256: contract.stage_133_validation_sha256,
      expected_stage_132_result_sha256: contract.stage_132_result_sha256,
      expected_stage_131_claim_sha256: contract.stage_131_claim_sha256,
      expected_receipt_manifest_sha256: contract.receipt_manifest_sha256,
      expected_stage_125_specification_sha256: contract.stage_125_specification_sha256,
      materializer_name: name().trim(),
      materializer_kind: "ephemeral_deterministic_pdf_csv_json_snapshot_materialization_specification",
      materializer_spec_revision: revision().trim(),
      proposed_materializer_code_revision: codeRevision().trim(),
      proposed_materializer_artifact_sha256: artifactSha().trim().toLowerCase(),
      artifact_reproduction_procedure: reproduction().trim(),
      rationale: rationale().trim(),
      known_limitations: limitations().trim(),
      future_input_constraints: inputConstraints().trim(),
      future_output_constraints: outputConstraints().trim(),
      exact_current_stage_51_through_stage_135_binding_confirmed: values[0] as boolean,
      registrar_independent_from_stage_135_and_complete_prior_chain_confirmed: values[1] as boolean,
      implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed: values[2] as boolean,
      proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: values[3] as boolean,
      all_ten_snapshot_materialization_functions_and_canonical_schemas_preserved_confirmed: values[4] as boolean,
      future_input_only_stage_133_independently_validated_read_only_content_addressed_encrypted_receipt_confirmed: values[5] as boolean,
      complete_accounts_cash_positions_options_liabilities_unsettled_and_whole_snapshot_failure_semantics_preserved_confirmed: values[6] as boolean,
      exact_decimal_signed_quantities_identity_corporate_action_and_row_provenance_semantics_preserved_confirmed: values[7] as boolean,
      future_decryption_only_in_isolated_ephemeral_memory_and_no_plaintext_persistence_confirmed: values[8] as boolean,
      deterministic_pdf_csv_json_parsing_and_no_remote_fetch_confirmed: values[9] as boolean,
      statement_market_values_informational_and_no_nav_or_performance_confirmed: values[10] as boolean,
      future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: values[11] as boolean,
      fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: values[12] as boolean,
      no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: values[13] as boolean,
      no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: values[14] as boolean,
      registration_only_opens_stage_137_chain_external_first_execution_authorization_review_confirmed: values[15] as boolean,
      no_unconfirmed_hari_or_old_wang_logic_claimed: values[16] as boolean,
    };
    setBusy(true); setError(""); setNotice("");
    try {
      setRegistry(await registerOpeningPortfolioSnapshotMaterializationIsolatedMaterializer(implementation.implementation_id, request));
      setChecks(CHECKS.map(() => false));
      setNotice("Stage 136 规格已 create-once 登记；物化器仍未创建或运行，只开放 Stage 137 首次执行授权复核。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 136 隔离物化器规格登记失败");
      await load();
    } finally { setBusy(false); }
  };

  return <Show when={registry()}>{(current) => <section class="public-admin-reward-governance" aria-label="期初组合快照隔离物化器规格登记">
    <header><strong>第 136 阶段 · 隔离物化器规格登记</strong><span>create-once · 未执行</span></header>
    <p>{current().scope}</p>
    <p class="public-admin-anchor-boundary">明确为空：工件、入口、runtime、input read/decrypt、parser、候选/真实快照、财务状态、训练、订单与交易权限。</p>
    <div class="public-admin-decision-metrics">
      <div><span>可登记</span><strong>{current().registration_eligible_count}</strong></div>
      <div><span>已登记</span><strong>{current().materializer_count}</strong></div>
      <div><span>当前绑定</span><strong>{current().current_binding_materializer_count}</strong></div>
      <div><span>Stage 137 候选</span><strong>{current().first_execution_authorization_review_eligible_count}</strong></div>
    </div>
    <Show when={eligible().length > 0} fallback={<p>当前没有通过 Stage 135 且尚未登记的物化实现。</p>}>
      <label><span>Stage 135 独立批准</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={eligible()}>{(item) => <option value={item.implementation.implementation_id}>{item.implementation.implementation_name} · {item.review.review_id.slice(0, 12)}…</option>}</For></select></label>
      <label><span>物化器名称</span><input value={name()} onInput={(event) => setName(event.currentTarget.value)} /></label>
      <label><span>规格版本</span><input value={revision()} onInput={(event) => setRevision(event.currentTarget.value)} /></label>
      <label><span>不可变代码版本</span><input value={codeRevision()} onInput={(event) => setCodeRevision(event.currentTarget.value)} placeholder="commit / tree / build revision" /></label>
      <label><span>未来工件 SHA-256</span><input value={artifactSha()} onInput={(event) => setArtifactSha(event.currentTarget.value)} /></label>
      <label><span>工件复现步骤</span><textarea value={reproduction()} onInput={(event) => setReproduction(event.currentTarget.value)} /></label>
      <label><span>登记理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
      <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
      <label><span>未来输入约束</span><textarea value={inputConstraints()} onInput={(event) => setInputConstraints(event.currentTarget.value)} /></label>
      <label><span>未来输出约束</span><textarea value={outputConstraints()} onInput={(event) => setOutputConstraints(event.currentTarget.value)} /></label>
      <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, i) => i === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
      <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在 create-once 登记…" : "登记 Stage 136 隔离物化器规格"}</button>
    </Show>
    <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
    <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
  </section>}</Show>;
}
