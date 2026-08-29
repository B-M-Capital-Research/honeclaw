import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowMarketDataParserSpecificationReviews,
  reviewControlledShadowMarketDataParserSpecificationOnce,
} from "@/lib/api";
import type {
  ControlledShadowMarketDataParserSpecificationReviewRegistry,
  ReviewControlledShadowMarketDataParserSpecificationRequest,
} from "@/lib/types";

const CHECKS = [
  "精确绑定 Stage 51–95 当前不可变责任链",
  "复核者不是登记者、验证者、执行者或此前完整责任链成员",
  "已独立重算 validation、claim、result、receipt、registration 与 specification",
  "已独立重建价格、原始价、分红调整价、分红、拆股和 NYSE 日历请求",
  "已独立重建八组合成向量的输入与预期输出哈希",
  "已检查 UTF-8、JSON/HTML、日期和有限数值边界",
  "重复、越界、缺失、格式错误均失败关闭",
  "不去重、不前填、不插值、不回退、不推断公司行动",
  "三条价格序列、显式公司行动及跨来源对账彼此分离",
  "SPY 覆盖官方交易日，标的缺口必须显式失败",
  "source_available_at 仍未验证，须留待独立证据链",
  "这里只有规格复核，无 parser 工件、入口、runtime 或原始载荷访问",
  "通过只开放未来零能力 parser 实现登记资格",
  "不生成行情行、观察、账本、持仓、绩效、模型、训练、奖励、订单或交易",
  "没有把未确认的 Hari/老王观点写成系统规则",
] as const;

const emptyFields = () => ({
  rationale: "",
  source_contract_assessment: "",
  schema_and_numeric_assessment: "",
  calendar_and_reconciliation_assessment: "",
  synthetic_vector_assessment: "",
  failure_and_missing_data_assessment: "",
  known_limitations: "",
  future_implementation_constraints: "",
});

export function PublicAdminControlledShadowMarketDataParserSpecificationReviewPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowMarketDataParserSpecificationReviewRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [verdict, setVerdict] = createSignal<ReviewControlledShadowMarketDataParserSpecificationRequest["verdict"]>(
    "approved_for_future_zero_capability_parser_implementation_registration",
  );
  const [fields, setFields] = createSignal(emptyFields());
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowMarketDataParserSpecificationReviews();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.review_eligible);
      if (!eligible.some((item) => item.registration.registration_id === selectedId())) {
        setSelectedId(eligible[0]?.registration.registration_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 96 规格独立复核表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find(
      (item) => item.review_eligible && item.registration.registration_id === selectedId(),
    ),
  );
  const disabled = createMemo(() =>
    busy() || !selected() || Object.values(fields()).some((value) => !value.trim())
      || checks().some((value) => !value),
  );

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const registration = item.registration;
    const request: ReviewControlledShadowMarketDataParserSpecificationRequest = {
      expected_registration_sha256: registration.registration_sha256,
      expected_parser_specification_sha256: registration.parser_specification.parser_specification_sha256,
      expected_validation_sha256: item.validation_sha256,
      expected_receipt_sha256: item.receipt_sha256,
      expected_claim_sha256: item.claim_sha256,
      expected_result_sha256: item.result_sha256,
      expected_adapter_authorization_sha256: item.adapter_authorization_sha256,
      expected_adapter_spec_sha256: item.adapter_spec_sha256,
      expected_canonical_request_set_sha256: item.canonical_request_set_sha256,
      verdict: verdict(),
      ...fields(),
      exact_stage_51_through_stage_95_binding_confirmed: true,
      reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: true,
      independent_recomputation_of_validation_claim_result_receipt_registration_and_specification_confirmed: true,
      independent_reconstruction_of_explicit_price_dividend_split_and_calendar_requests_confirmed: true,
      independent_reconstruction_of_all_synthetic_vector_input_and_output_hashes_confirmed: true,
      strict_utf8_json_html_date_and_bounded_numeric_rules_reviewed: true,
      duplicate_out_of_window_missing_and_malformed_fail_closed_rules_reviewed: true,
      no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_reviewed: true,
      separate_price_series_explicit_actions_and_cross_source_reconciliation_reviewed: true,
      spy_official_calendar_coverage_and_explicit_subject_gap_rules_reviewed: true,
      source_available_at_remains_unverified_until_separate_review_confirmed: true,
      specification_only_no_parser_artifact_entrypoint_runtime_or_raw_payload_access_confirmed: true,
      approval_only_opens_future_zero_capability_parser_implementation_registration_confirmed: true,
      no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    setBusy(true); setError(""); setNotice("");
    try {
      setRegistry(await reviewControlledShadowMarketDataParserSpecificationOnce(registration.registration_id, request));
      setFields(emptyFields()); setChecks(CHECKS.map(() => false));
      setNotice("Stage 96 责任链外复核已 create-once 写入；通过也只开放未来零能力实现登记资格。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 96 规格独立复核失败");
      await load();
    } finally { setBusy(false); }
  };

  const textFields = [
    ["rationale", "复核结论与理由"],
    ["source_contract_assessment", "显式来源合同评估"],
    ["schema_and_numeric_assessment", "schema 与数值边界评估"],
    ["calendar_and_reconciliation_assessment", "交易日与跨来源对账评估"],
    ["synthetic_vector_assessment", "八组合成向量独立重建评估"],
    ["failure_and_missing_data_assessment", "失败关闭与缺失数据评估"],
    ["known_limitations", "已知限制"],
    ["future_implementation_constraints", "未来零能力实现登记约束"],
  ] as const;

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="Stage 96 行情解析器规格责任链外独立复核">
      <header><strong>第 96 阶段 · 行情解析器规格责任链外独立复核</strong><span>{current().review_status}</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>Stage 95 规格</span><strong>{current().parser_specification_registered_count}</strong></div>
        <div><span>待复核</span><strong>{current().review_eligible_count}</strong></div>
        <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
        <div><span>独立通过</span><strong>{current().independently_approved_count}</strong></div>
      </div>
      <p class="public-admin-anchor-boundary">第二套实现独立重建五类 FMP 请求、NYSE 交易日请求、Stage 95 规格哈希和八组合成向量；不读取任何原始载荷。</p>
      <Show when={current().review_eligible_count > 0} fallback={<p>当前没有待责任链外复核的 Stage 95 规格。</p>}>
        <label><span>Stage 95 规格登记</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={current().items.filter((item) => item.review_eligible)}>{(item) => <option value={item.registration.registration_id}>{item.registration.registration_id.slice(0, 12)}… · {item.subject_symbols.join("、")}</option>}</For></select></label>
        <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ReviewControlledShadowMarketDataParserSpecificationRequest["verdict"])}><option value="approved_for_future_zero_capability_parser_implementation_registration">独立通过，仅开放未来零能力实现登记</option><option value="changes_required_rebuild_parser_specification">要求修改并重建不可变规格</option><option value="rejected_parser_specification">拒绝规格</option></select></label>
        <For each={textFields}>{([key, label]) => <label><span>{label}</span><textarea value={fields()[key]} onInput={(event) => setFields((value) => ({ ...value, [key]: event.currentTarget.value }))} /></label>}</For>
        <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, i) => i === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在独立复核…" : "写入 Stage 96 终态复核"}</button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().items.filter((item) => item.latest_review)}>{(item) => <article class="public-admin-reward-governance"><header><strong>review {item.latest_review!.review_id}</strong><span>{item.latest_review!.verdict}</span></header><p>独立审计 {item.latest_review!.independent_audit_passed ? "通过" : "未通过"} · 规格 {item.latest_review!.parser_specification_sha256.slice(0, 16)}…</p><Show when={item.latest_review!.mismatch_reasons.length > 0}><p class="public-admin-error">{item.latest_review!.mismatch_reasons.join("；")}</p></Show><p class="public-admin-anchor-boundary">parser 实现登记资格：{item.latest_review!.future_zero_capability_parser_implementation_registration_eligible ? "已开放" : "未开放"}；仍无 parser、载荷访问或交易权限。</p></article>}</For>
    </section>
  )}</Show>;
}
