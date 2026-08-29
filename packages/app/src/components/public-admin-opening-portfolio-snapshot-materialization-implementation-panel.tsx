import { For, Show, createSignal, onMount } from "solid-js";

import {
  getOpeningPortfolioSnapshotMaterializationImplementations,
  registerOpeningPortfolioSnapshotMaterializationImplementation,
} from "@/lib/api";
import type {
  OpeningPortfolioSnapshotMaterializationImplementationRegistry,
  RegisterOpeningPortfolioSnapshotMaterializationImplementationRequest,
} from "@/lib/types";

const CHECKS = [
  "重新核对 Stage 51–133 完整绑定",
  "登记人独立于验证人、执行人、领取人和完整前序责任链",
  "重算 validation、receipt、claim、result 与 Stage 125 规格摘要",
  "完整保留 Stage 125 来源合同与 canonical snapshot schema",
  "未来输入仅限独立验证且内容寻址的 receipt",
  "未来只在隔离、临时内存环境解密，不落盘明文",
  "PDF/CSV/JSON 使用确定性适配器，禁止远程抓取",
  "逐账户覆盖现金、持仓、上市期权、负债与未结算活动",
  "十进制以字符串精确处理，有符号数量且禁止二进制浮点",
  "遵守证券身份优先级并完成公司行动对账",
  "禁止默认、手填和推断；不支持资产使整份快照失败",
  "对账单市值仅供参考，不计算 NAV 或绩效",
  "每行绑定工件摘要与来源位置，输出不含原账号或秘密",
  "未来输出 create-once、未受信，并必须独立验证",
  "本阶段没有解密、读取、解析、工件、入口、runtime、挂载或输出",
  "不开放准入、白名单、账本、持仓、现金、净值、训练、订单或交易",
  "登记后只开放 Stage 135 责任链外独立实现复核",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

const defaultTexts = {
  implementationName: "HONE 期初组合快照确定性物化器",
  immutableCodeRevision: "hone-opening-snapshot-materializer-contract-v1",
  implementationDescription: "将独立验证的券商或托管原始导出确定性转换为未受信 canonical opening snapshot candidate；当前只登记合同。",
  deterministicParser: "按 provider、格式和冻结版本选择确定性 PDF/CSV/JSON 适配器；禁止网络、工具和非确定性外部依赖。",
  completeness: "核对声明账户数量，并要求每个账户的现金、持仓、期权、负债和未结算活动完整出现；缺失即整批失败。",
  decimal: "所有金额、价格、数量、成本基础与期权乘数使用 canonical 十进制字符串和有符号数量，禁止二进制浮点。",
  identity: "按冻结的证券身份优先级归一化并对账拆股、合并、分拆、并购、代码变化等公司行动；歧义即失败。",
  provenance: "每个输出行绑定来源工件 SHA-256、页码或行号/JSON pointer；账号仅保留不可逆 alias，错误与日志不含秘密。",
  failure: "任一账户不完整、字段歧义、资产不支持或公司行动未对账时整份候选失败；更正必须创建新候选，禁止覆盖。",
  limitations: "当前未实现或运行 provider adapter，也未证明 receipt 内金融数字真实；登记不产生持仓。",
  review: "Stage 135 必须以第二实现重建合同，并确认没有输入读取、解析或财务写入能力。",
};

export function PublicAdminOpeningPortfolioSnapshotMaterializationImplementationPanel() {
  const [registry, setRegistry] = createSignal<OpeningPortfolioSnapshotMaterializationImplementationRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [values, setValues] = createSignal(defaultTexts);
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getOpeningPortfolioSnapshotMaterializationImplementations();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.registration_eligible);
      if (!eligible.some((item) => item.candidate.stage_133_validation_id === selectedId())) {
        setSelectedId(eligible[0]?.candidate.stage_133_validation_id ?? "");
      }
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 134 零能力实现登记表读取失败");
    }
  };
  onMount(() => void load());

  const ready = () => Boolean(selectedId() && Object.values(values()).every((value) => value.trim()) && checks().every(Boolean));

  const submit = async () => {
    const item = registry()?.items.find((entry) => entry.registration_eligible && entry.candidate.stage_133_validation_id === selectedId());
    if (!item || busy() || !ready()) return;
    const candidate = item.candidate;
    const text = values();
    const confirmed = checks();
    const request: RegisterOpeningPortfolioSnapshotMaterializationImplementationRequest = {
      expected_stage_133_validation_id: candidate.stage_133_validation_id,
      expected_stage_133_validation_sha256: candidate.stage_133_validation_sha256,
      expected_stage_132_result_sha256: candidate.stage_132_result_sha256,
      expected_stage_131_claim_sha256: candidate.stage_131_claim_sha256,
      expected_receipt_id: candidate.receipt_id,
      expected_receipt_manifest_sha256: candidate.receipt_manifest_sha256,
      expected_stage_125_specification_sha256: candidate.stage_125_specification_sha256,
      implementation_name: text.implementationName,
      immutable_code_revision: text.immutableCodeRevision,
      implementation_description: text.implementationDescription,
      deterministic_parser_and_adapter_semantics: text.deterministicParser,
      account_scope_and_completeness_semantics: text.completeness,
      exact_decimal_and_signed_quantity_semantics: text.decimal,
      instrument_identity_and_corporate_action_semantics: text.identity,
      row_provenance_and_redaction_semantics: text.provenance,
      whole_snapshot_failure_and_correction_semantics: text.failure,
      known_limitations: text.limitations,
      future_review_constraints: text.review,
      exact_current_stage_51_through_stage_133_binding_confirmed: confirmed[0] as boolean,
      registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain_confirmed: confirmed[1] as boolean,
      validation_receipt_claim_result_and_specification_hashes_recomputed_confirmed: confirmed[2] as boolean,
      exact_stage_125_source_contract_and_canonical_snapshot_schema_preserved_confirmed: confirmed[3] as boolean,
      future_input_only_independently_validated_content_addressed_receipt_confirmed: confirmed[4] as boolean,
      future_decryption_only_inside_isolated_ephemeral_materializer_confirmed: confirmed[5] as boolean,
      deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed: confirmed[6] as boolean,
      account_cash_position_option_liability_and_unsettled_activity_completeness_confirmed: confirmed[7] as boolean,
      exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed: confirmed[8] as boolean,
      instrument_identity_precedence_and_corporate_action_reconciliation_confirmed: confirmed[9] as boolean,
      no_default_manual_balance_or_inference_and_unsupported_asset_fails_whole_snapshot_confirmed: confirmed[10] as boolean,
      statement_market_values_informational_and_no_nav_or_performance_confirmed: confirmed[11] as boolean,
      every_output_row_bound_to_artifact_hash_and_source_locator_without_raw_account_or_secret_confirmed: confirmed[12] as boolean,
      future_output_create_once_untrusted_canonical_candidate_and_independent_validation_required_confirmed: confirmed[13] as boolean,
      contract_only_no_decrypt_read_parse_artifact_entrypoint_runtime_mount_or_output_confirmed: confirmed[14] as boolean,
      no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: confirmed[15] as boolean,
      future_stage_135_chain_external_independent_implementation_review_required_confirmed: confirmed[16] as boolean,
      no_unconfirmed_hari_or_old_wang_logic_claimed: confirmed[17] as boolean,
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setRegistry(await registerOpeningPortfolioSnapshotMaterializationImplementation(candidate.stage_133_validation_id, request));
      setChecks(CHECKS.map(() => false));
      setNotice("第 134 阶段合同已登记；没有读取或解析来源，也没有生成持仓。下一步只允许 Stage 135 独立实现复核。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 134 登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  const updateText = (key: keyof typeof defaultTexts, value: string) => setValues((current) => ({ ...current, [key]: value }));
  const eligibleItems = () => registry()?.items.filter((item) => item.registration_eligible) ?? [];

  return <Show when={registry()}>{(current) => <section class="public-admin-reward-governance" aria-label="期初组合快照物化零能力实现登记">
    <header><strong>第 134 阶段 · 期初快照物化零能力实现登记</strong><span>{current().next_gate}</span></header>
    <p>{current().scope}</p>
    <div class="public-admin-decision-metrics">
      <div><span>独立验证 receipt</span><strong>{current().independently_validated_receipt_count}</strong></div>
      <div><span>可登记</span><strong>{current().registration_eligible_count}</strong></div>
      <div><span>合同</span><strong>{current().implementation_contract_count}</strong></div>
      <div><span>Stage 135 候选</span><strong>{current().future_stage_135_independent_implementation_review_eligible_count}</strong></div>
    </div>
    <p><strong>边界：</strong>这里冻结未来 parser/materializer 的输入、输出、完整性和逐行证据合同；没有解密、金融行解析、快照候选、真实持仓或交易权限。</p>
    <Show when={eligibleItems().length > 0} fallback={<p>当前没有待登记的 Stage 133 receipt；零状态或已完成登记均符合设计。</p>}>
      <label><span>独立验证 receipt</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={eligibleItems()}>{(item) => <option value={item.candidate.stage_133_validation_id}>{item.candidate.source_provider_name} · {item.candidate.portfolio_scope_alias} · {item.candidate.artifact_count} 件</option>}</For></select></label>
      <div class="public-admin-decision-form-grid">
        <label><span>实现名称</span><input value={values().implementationName} onInput={(event) => updateText("implementationName", event.currentTarget.value)} /></label>
        <label><span>不可变代码版本</span><input value={values().immutableCodeRevision} onInput={(event) => updateText("immutableCodeRevision", event.currentTarget.value)} /></label>
      </div>
      <For each={([
        ["implementationDescription", "实现说明"], ["deterministicParser", "确定性适配器"], ["completeness", "账户与字段完整性"],
        ["decimal", "精确十进制"], ["identity", "证券身份与公司行动"], ["provenance", "逐行来源与脱敏"],
        ["failure", "整批失败与更正"], ["limitations", "已知限制"], ["review", "后续复核约束"],
      ] as Array<[keyof typeof defaultTexts, string]>)}>{([key, label]) => <label><span>{label}</span><textarea value={values()[key]} onInput={(event) => updateText(key, event.currentTarget.value)} /></label>}</For>
      <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((items) => items.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
      <button type="button" class="public-admin-decision-submit" disabled={busy() || !ready()} onClick={() => void submit()}>{busy() ? "正在登记…" : "登记零能力物化实现合同"}</button>
    </Show>
    <Show when={current().items.some((item) => item.implementation)}><div class="public-admin-review-history"><For each={current().items.filter((item) => item.implementation)}>{(item) => <article><strong>{item.implementation?.implementation_name}</strong><span>{item.implementation?.implementation_id.slice(0, 8)} · 未运行 · 等待 Stage 135</span></article>}</For></div></Show>
    <Show when={error()}><p class="public-admin-error">{error()}</p></Show><Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
  </section>}</Show>;
}
