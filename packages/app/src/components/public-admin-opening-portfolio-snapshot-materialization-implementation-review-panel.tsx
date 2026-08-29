import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getOpeningPortfolioSnapshotMaterializationImplementationReviews,
  reviewOpeningPortfolioSnapshotMaterializationImplementation,
} from "@/lib/api";
import type {
  OpeningPortfolioSnapshotMaterializationImplementationReviewConfirmations,
  OpeningPortfolioSnapshotMaterializationImplementationReviewRegistry,
  OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–134 完整责任链",
  "复核人独立于 registrar、validator、executor、claimant 与完整前序责任链",
  "独立重算实现、合同、validation、result、claim、receipt 与 Stage 125 specification 摘要",
  "完整合同由第二实现重建，未调用 Stage 134 builder 自证",
  "已重新校验 Stage 134 全部 18 项登记确认",
  "未来输入仅限已独立验证、内容寻址的加密 receipt",
  "未来解密只可在隔离物化器的临时内存发生",
  "PDF/CSV/JSON 适配器确定且禁止远程 URL 抓取",
  "账户、现金、持仓、上市期权、负债与未结算活动必须完整",
  "只允许精确十进制字符串和有符号数量，禁止二进制浮点",
  "证券身份优先级与公司行动对账合同完整",
  "禁止默认、手填或推断；缺失、歧义和不支持资产使整份快照失败",
  "对账单市场价值只作信息字段，不产生 NAV 或绩效",
  "每行绑定工件 SHA-256 与来源位置，并删除真实账号和凭据",
  "未来输出 create-once、untrusted，且验证与准入继续分离",
  "当前没有 key/input read、解密、parser、工件、入口、runtime、挂载或输出",
  "当前没有快照、金融白名单、账本、持仓、现金、净值/绩效、模型、训练/RL、订单、券商或交易权限",
  "批准只开放 Stage 136 隔离物化器规格登记",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

function confirmations(values: boolean[]): OpeningPortfolioSnapshotMaterializationImplementationReviewConfirmations {
  return {
    exact_current_stage_51_through_stage_134_binding_confirmed: values[0] as boolean,
    reviewer_independent_from_registrar_validator_executor_claimant_and_complete_prior_chain_confirmed: values[1] as boolean,
    implementation_contract_validation_result_claim_receipt_and_specification_hashes_independently_reproduced_confirmed: values[2] as boolean,
    complete_contract_rebuilt_without_stage_134_builder_confirmed: values[3] as boolean,
    all_stage_134_registration_confirmations_revalidated_confirmed: values[4] as boolean,
    input_only_independently_validated_content_addressed_receipt_confirmed: values[5] as boolean,
    future_decryption_only_in_isolated_ephemeral_memory_confirmed: values[6] as boolean,
    deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed: values[7] as boolean,
    complete_accounts_cash_positions_options_liabilities_and_unsettled_activity_confirmed: values[8] as boolean,
    exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed: values[9] as boolean,
    instrument_identity_precedence_and_corporate_action_reconciliation_confirmed: values[10] as boolean,
    no_default_manual_or_inferred_financial_values_and_whole_snapshot_failure_confirmed: values[11] as boolean,
    statement_market_values_informational_and_no_nav_or_performance_confirmed: values[12] as boolean,
    every_output_row_bound_to_artifact_hash_and_source_locator_with_redaction_confirmed: values[13] as boolean,
    output_create_once_untrusted_and_separate_validation_and_admission_confirmed: values[14] as boolean,
    no_key_input_read_decrypt_parse_artifact_entrypoint_runtime_mount_or_output_confirmed: values[15] as boolean,
    no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: values[16] as boolean,
    approval_only_opens_future_stage_136_isolated_materializer_specification_registration_confirmed: values[17] as boolean,
    no_unconfirmed_hari_or_old_wang_logic_claimed: values[18] as boolean,
  };
}

export function PublicAdminOpeningPortfolioSnapshotMaterializationImplementationReviewPanel() {
  const [registry, setRegistry] = createSignal<OpeningPortfolioSnapshotMaterializationImplementationReviewRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [verdict, setVerdict] = createSignal<OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict>(
    "approved_for_future_isolated_materializer_specification_registration",
  );
  const [rationale, setRationale] = createSignal("");
  const [binding, setBinding] = createSignal("");
  const [schema, setSchema] = createSignal("");
  const [provenance, setProvenance] = createSignal("");
  const [separation, setSeparation] = createSignal("");
  const [limitations, setLimitations] = createSignal("尚无可执行物化器、输入访问、解密、parser/runtime 或真实期初快照。");
  const [constraints, setConstraints] = createSignal("Stage 136 只能登记隔离物化器规格；不得读取 receipt、运行 parser 或创建财务状态。");
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const eligible = createMemo(() => registry()?.items.filter((item) => item.review_eligible) ?? []);
  const selected = createMemo(() => eligible().find((item) => item.implementation.implementation_id === selectedId()));

  const load = async () => {
    try {
      const next = await getOpeningPortfolioSnapshotMaterializationImplementationReviews();
      setRegistry(next);
      const candidates = next.items.filter((item) => item.review_eligible);
      if (!candidates.some((item) => item.implementation.implementation_id === selectedId())) {
        setSelectedId(candidates[0]?.implementation.implementation_id ?? "");
      }
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 135 期初快照物化实现独立复核读取失败");
    }
  };
  onMount(() => void load());

  const disabled = createMemo(() => busy() || !selected()
    || [rationale(), binding(), schema(), provenance(), separation(), limitations(), constraints()]
      .some((value) => value.trim().length === 0)
    || (verdict().startsWith("approved") && !checks().every(Boolean)));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    setBusy(true); setError(""); setNotice("");
    try {
      const implementation = item.implementation;
      const contract = implementation.implementation_contract;
      const upstream = implementation.upstream_stage_133_validation;
      const next = await reviewOpeningPortfolioSnapshotMaterializationImplementation(
        implementation.implementation_id,
        {
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_contract_sha256: contract.contract_sha256,
          expected_stage_133_validation_sha256: upstream.validation_sha256,
          expected_stage_132_result_sha256: upstream.stage_132_result_sha256,
          expected_stage_131_claim_sha256: upstream.stage_131_claim_sha256,
          expected_receipt_manifest_sha256: upstream.receipt_manifest_sha256,
          expected_stage_125_specification_sha256: upstream.stage_125_specification_sha256,
          expected_independent_audit_sha256: item.current_independent_audit.audit_sha256,
          verdict: verdict(),
          rationale: rationale().trim(),
          binding_and_recomputation_assessment: binding().trim(),
          parser_schema_and_completeness_assessment: schema().trim(),
          decimal_identity_and_provenance_assessment: provenance().trim(),
          failure_separation_and_zero_capability_assessment: separation().trim(),
          known_limitations: limitations().trim(),
          future_materializer_constraints: constraints().trim(),
          confirmations: confirmations(checks()),
        },
      );
      setRegistry(next); setRationale(""); setChecks(REVIEW_CHECKS.map(() => false));
      setNotice(verdict().startsWith("approved")
        ? "Stage 135 已独立批准；仍没有来源读取、解析或真实持仓，只开放 Stage 136 隔离物化器规格登记。"
        : "复核结论已终结保存；原 Stage 134 实现必须重建后才能再次进入复核链。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 135 期初快照物化实现独立复核提交失败");
      await load();
    } finally { setBusy(false); }
  };

  return <Show when={registry()}>{(current) => <section class="public-admin-reward-governance" aria-label="期初组合快照物化实现独立复核">
    <header><strong>第 135 阶段 · 期初快照物化实现独立复核</strong><span>第二实现 · 零能力</span></header>
    <p>{current().scope}</p>
    <p class="public-admin-anchor-boundary">当前明确为空：key/input read、receipt 解密、parser/runtime、候选/真实快照、账本、持仓、现金、净值/绩效、训练、订单与交易权限。</p>
    <div class="public-admin-decision-metrics">
      <div><span>待复核</span><strong>{current().review_eligible_count}</strong></div>
      <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
      <div><span>独立批准</span><strong>{current().independently_approved_count}</strong></div>
      <div><span>Stage 136 候选</span><strong>{current().future_stage_136_isolated_materializer_specification_registration_eligible_count}</strong></div>
    </div>
    <Show when={eligible().length > 0} fallback={<p>当前没有待复核的 Stage 134 零能力物化实现合同。</p>}>
      <label><span>Stage 134 实现合同</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}>
        <For each={eligible()}>{(item) => <option value={item.implementation.implementation_id}>{item.implementation.implementation_name} · {item.implementation.implementation_id.slice(0, 12)}…</option>}</For>
      </select></label>
      <label><span>裁决</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict)}>
        <option value="approved_for_future_isolated_materializer_specification_registration">批准进入 Stage 136 隔离物化器规格登记</option>
        <option value="changes_required_rebuild_materialization_implementation">要求重建 Stage 134 实现</option>
        <option value="rejected_materialization_implementation">拒绝该实现</option>
      </select></label>
      <label><span>复核理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
      <label><span>责任链与摘要重算</span><textarea value={binding()} onInput={(event) => setBinding(event.currentTarget.value)} /></label>
      <label><span>解析器、schema 与完整性</span><textarea value={schema()} onInput={(event) => setSchema(event.currentTarget.value)} /></label>
      <label><span>十进制、证券身份与逐行来源</span><textarea value={provenance()} onInput={(event) => setProvenance(event.currentTarget.value)} /></label>
      <label><span>失败关闭、关卡分离与零能力</span><textarea value={separation()} onInput={(event) => setSeparation(event.currentTarget.value)} /></label>
      <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
      <label><span>未来物化器约束</span><textarea value={constraints()} onInput={(event) => setConstraints(event.currentTarget.value)} /></label>
      <div class="public-admin-decision-checks"><For each={REVIEW_CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
      <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在追加独立复核…" : "提交 Stage 135 独立复核"}</button>
    </Show>
    <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
    <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
  </section>}</Show>;
}
