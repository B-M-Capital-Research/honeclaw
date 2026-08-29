import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowMarketDataParserImplementations,
  registerControlledShadowMarketDataParserImplementationOnce,
} from "@/lib/api";
import type {
  ControlledShadowMarketDataParserImplementationRegistry,
  RegisterControlledShadowMarketDataParserImplementationRequest,
} from "@/lib/types";

const CHECKS = [
  "精确绑定 Stage 51–96 当前不可变责任链",
  "登记者不是 Stage 96 复核者或此前完整责任链成员",
  "已独立重算 review、registration 与 specification 指纹",
  "这里只登记零能力契约，不提交源码或可执行制品",
  "保留显式价格、原始价、分红调整价、分红、拆股和官方交易日来源",
  "保留严格 UTF-8、JSON/HTML、日期及有限数值规则",
  "重复、越界、缺失与格式错误仍然失败关闭",
  "不去重、不前填、不插值、不回退、不推断公司行动",
  "保留 SPY/官方日历覆盖、标的缺口与跨来源对账约束",
  "绑定全部八组合成向量哈希",
  "source_available_at 仍未验证，留待独立证据链",
  "未来输出须内容寻址、create-once、非可信并独立验证",
  "没有入口、runtime、载荷挂载/读取、环境、秘密、网络、工具或子进程",
  "不生成行情行、观察、账本、持仓、绩效、模型、训练、奖励、订单或交易",
  "隔离 runner 前必须完成 Stage 98 责任链外独立实现复核",
  "没有把未确认的 Hari/老王观点写成系统规则",
] as const;

const emptyFields = () => ({
  implementation_name: "",
  immutable_code_revision: "",
  implementation_description: "",
  deterministic_parser_semantics: "",
  source_schema_and_numeric_semantics: "",
  calendar_action_and_reconciliation_semantics: "",
  error_and_missing_data_semantics: "",
  known_limitations: "",
  future_review_constraints: "",
});

export function PublicAdminControlledShadowMarketDataParserImplementationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowMarketDataParserImplementationRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [fields, setFields] = createSignal(emptyFields());
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowMarketDataParserImplementations();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.registration_eligible);
      if (!eligible.some((item) => item.specification_review.review_id === selectedId())) {
        setSelectedId(eligible[0]?.specification_review.review_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 97 零能力实现登记表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find(
      (item) => item.registration_eligible && item.specification_review.review_id === selectedId(),
    ),
  );
  const disabled = createMemo(() =>
    busy() || !selected() || Object.values(fields()).some((value) => !value.trim())
      || checks().some((value) => !value),
  );

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const review = item.specification_review;
    const registration = item.specification_registration;
    const request: RegisterControlledShadowMarketDataParserImplementationRequest = {
      expected_specification_review_id: review.review_id,
      expected_specification_review_sha256: review.review_sha256,
      expected_registration_id: registration.registration_id,
      expected_registration_sha256: registration.registration_sha256,
      expected_parser_specification_sha256: registration.parser_specification.parser_specification_sha256,
      expected_validation_sha256: review.validation_sha256,
      expected_receipt_sha256: review.receipt_sha256,
      expected_claim_sha256: review.claim_sha256,
      expected_result_sha256: review.result_sha256,
      expected_adapter_authorization_sha256: review.adapter_authorization_sha256,
      expected_adapter_spec_sha256: review.adapter_spec_sha256,
      expected_canonical_request_set_sha256: review.canonical_request_set_sha256,
      ...fields(),
      exact_stage_51_through_stage_96_binding_confirmed: true,
      registrar_independent_from_stage_96_and_complete_prior_chain_confirmed: true,
      independent_recomputation_of_review_registration_and_specification_confirmed: true,
      zero_capability_contract_only_no_source_or_executable_artifact_confirmed: true,
      fixed_explicit_price_dividend_split_and_calendar_sources_preserved_confirmed: true,
      strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: true,
      duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: true,
      no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed: true,
      spy_official_calendar_coverage_subject_gap_and_cross_source_reconciliation_preserved_confirmed: true,
      all_eight_synthetic_vector_hashes_bound_confirmed: true,
      source_available_at_remains_unverified_until_separate_review_confirmed: true,
      future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
      no_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed: true,
      no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      future_independent_implementation_review_required_before_isolated_runner_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setRegistry(await registerControlledShadowMarketDataParserImplementationOnce(review.review_id, request));
      setFields(emptyFields());
      setChecks(CHECKS.map(() => false));
      setNotice("Stage 97 零能力实现契约已 create-once 写入；仍须 Stage 98 责任链外独立实现复核。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 97 零能力实现契约登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  const textFields = [
    ["implementation_name", "实现契约名称"],
    ["immutable_code_revision", "不可变代码版本标识（仅标识，不上传代码）"],
    ["implementation_description", "零能力实现说明"],
    ["deterministic_parser_semantics", "确定性解析语义"],
    ["source_schema_and_numeric_semantics", "来源 schema 与数值语义"],
    ["calendar_action_and_reconciliation_semantics", "交易日、公司行动与对账语义"],
    ["error_and_missing_data_semantics", "错误与缺失数据语义"],
    ["known_limitations", "已知限制"],
    ["future_review_constraints", "Stage 98 独立复核约束"],
  ] as const;

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="Stage 97 行情解析器零能力实现契约登记">
      <header><strong>第 97 阶段 · 行情解析器零能力实现契约登记</strong><span>{current().implementation_status}</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>独立批准规格</span><strong>{current().independently_approved_specification_count}</strong></div>
        <div><span>待登记</span><strong>{current().registration_eligible_count}</strong></div>
        <div><span>契约</span><strong>{current().implementation_contract_count}</strong></div>
        <div><span>待 Stage 98 复核</span><strong>{current().independent_implementation_review_eligible_count}</strong></div>
      </div>
      <p class="public-admin-anchor-boundary">这里只冻结纯函数标识、schema、哈希和失败关闭语义；没有源码、可执行入口、runtime 或原始载荷访问。</p>
      <Show when={current().registration_eligible_count > 0} fallback={<p>当前没有待登记的 Stage 96 独立批准规格。</p>}>
        <label><span>Stage 96 独立批准规格</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={current().items.filter((item) => item.registration_eligible)}>{(item) => <option value={item.specification_review.review_id}>{item.specification_review.review_id.slice(0, 12)}… · {item.subject_symbols.join("、")}</option>}</For></select></label>
        <For each={textFields}>{([key, label]) => <label><span>{label}</span><textarea value={fields()[key]} onInput={(event) => setFields((value) => ({ ...value, [key]: event.currentTarget.value }))} /></label>}</For>
        <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, i) => i === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在登记…" : "写入 Stage 97 零能力实现契约"}</button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().items.filter((item) => item.implementation)}>{(item) => <article class="public-admin-reward-governance"><header><strong>implementation {item.implementation!.implementation_id}</strong><span>{item.implementation!.status}</span></header><p>{item.implementation!.implementation_name} · contract {item.implementation!.implementation_contract.contract_sha256.slice(0, 16)}…</p><p class="public-admin-anchor-boundary">零能力契约已登记；parser 工件仍不存在，须先完成 Stage 98 独立复核，不能运行或读取载荷。</p></article>}</For>
    </section>
  )}</Show>;
}
