import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowMarketDataParserSpecifications,
  registerControlledShadowMarketDataParserSpecificationOnce,
} from "@/lib/api";
import type {
  ControlledShadowMarketDataParserSpecificationRegistry,
  RegisterControlledShadowMarketDataParserSpecificationRequest,
} from "@/lib/types";

const CHECKS = [
  "精确绑定 Stage 51–94 当前完整责任链",
  "登记者不是 Stage 94 验证者、Stage 93 执行者、Stage 92 复核者或此前责任人",
  "已独立重算 validation、receipt、claim、result 与请求集合绑定",
  "价格、原始价、分红调整价、分红、拆股和 NYSE 官方日历均有独立来源",
  "严格限定 UTF-8、JSON/HTML schema、日期和有限正数价格",
  "重复、越界、缺字段或格式错误必须失败关闭",
  "不去重、不前填、不插值，也不回退到未调整收盘价",
  "SPY 必须与官方交易日同步，并进行跨来源对账",
  "合成测试向量不包含真实行情事实或凭据",
  "这里只登记规格，没有 parser 代码、工件、入口或 runtime",
  "不读取/挂载原始载荷，不访问网络、工具、子进程或生产写入",
  "不生成日历行、行情行、观察、账本、持仓、绩效或模型指标",
  "不训练、不反馈 reward、不生成订单、不接券商、不交易",
  "实现前必须先经新的责任链外规格独立复核",
  "没有把未确认的 Hari/老王观点写成系统规则",
] as const;

const emptyFields = () => ({
  registration_reason: "",
  known_limitations: "",
  future_review_constraints: "",
});

export function PublicAdminControlledShadowMarketDataParserSpecificationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowMarketDataParserSpecificationRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [fields, setFields] = createSignal(emptyFields());
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowMarketDataParserSpecifications();
      setRegistry(next);
      if (!next.candidates.some((item) => item.validation_id === selectedId())) {
        setSelectedId(next.candidates[0]?.validation_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 95 行情解析器规格登记表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.candidates.find((item) => item.validation_id === selectedId()),
  );
  const disabled = createMemo(() =>
    busy() || !selected() || Object.values(fields()).some((value) => !value.trim())
      || checks().some((value) => !value),
  );

  const submit = async () => {
    const candidate = selected();
    if (!candidate || disabled()) return;
    const request: RegisterControlledShadowMarketDataParserSpecificationRequest = {
      expected_validation_sha256: candidate.validation_sha256,
      expected_receipt_sha256: candidate.receipt_sha256,
      expected_claim_sha256: candidate.claim_sha256,
      expected_result_sha256: candidate.result_sha256,
      expected_adapter_authorization_sha256: candidate.adapter_authorization_sha256,
      expected_adapter_spec_sha256: candidate.adapter_spec_sha256,
      expected_canonical_request_set_sha256: candidate.canonical_request_set_sha256,
      ...fields(),
      exact_stage_51_through_stage_94_binding_confirmed: true,
      registrar_independent_from_validator_executor_stage_92_and_complete_prior_chain_confirmed: true,
      independent_recomputation_of_validation_receipt_claim_and_request_bindings_confirmed: true,
      explicit_price_dividend_split_and_official_calendar_sources_confirmed: true,
      strict_utf8_json_html_schema_and_bounded_decimal_rules_confirmed: true,
      duplicate_out_of_window_missing_and_malformed_rows_fail_closed_confirmed: true,
      no_forward_fill_interpolation_deduplication_or_unadjusted_fallback_confirmed: true,
      spy_calendar_sync_and_cross_source_reconciliation_required_confirmed: true,
      synthetic_vectors_contain_no_market_fact_or_credential_confirmed: true,
      specification_only_no_parser_code_artifact_entrypoint_or_runtime_confirmed: true,
      no_raw_payload_read_mount_network_tool_subprocess_or_production_write_confirmed: true,
      no_calendar_market_row_observation_ledger_position_performance_or_model_metric_created_confirmed: true,
      no_training_feedback_reward_order_broker_or_trading_confirmed: true,
      future_chain_external_specification_review_required_before_implementation_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    setBusy(true); setError(""); setNotice("");
    try {
      setRegistry(await registerControlledShadowMarketDataParserSpecificationOnce(candidate.validation_id, request));
      setFields(emptyFields()); setChecks(CHECKS.map(() => false));
      setNotice("Stage 95 零能力解析器规格已 create-once 登记；没有运行 parser，也没有解析真实行情。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 95 行情解析器规格登记失败");
      await load();
    } finally { setBusy(false); }
  };

  const textFields = [
    ["registration_reason", "登记理由"],
    ["known_limitations", "已知限制（合成向量不证明供应商语义）"],
    ["future_review_constraints", "下一阶段责任链外复核约束"],
  ] as const;

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="Stage 95 零能力行情解析器规格登记">
      <header><strong>第 95 阶段 · 零能力行情解析器规格登记</strong><span>{current().parser_specification_status}</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>独立验证收据</span><strong>{current().independently_validated_receipt_count}</strong></div>
        <div><span>待登记</span><strong>{current().registration_eligible_count}</strong></div>
        <div><span>已登记</span><strong>{current().parser_specification_registered_count}</strong></div>
        <div><span>待独立复核</span><strong>{current().future_chain_external_specification_review_eligible_count}</strong></div>
      </div>
      <p class="public-admin-anchor-boundary">规格，不是解析器：当前无代码、工件、入口、runtime 或载荷挂载；不读取真实行情，不生成任何交易日、价格、收益或动作事实。</p>
      <Show when={current().candidates.length > 0} fallback={<p>当前没有经 Stage 94 独立验证通过、可登记规格的原始收据。</p>}>
        <label><span>Stage 94 独立验证记录</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={current().candidates}>{(item) => <option value={item.validation_id}>{item.validation_id.slice(0, 12)}… · {item.subject_symbols.join("、")} · {item.raw_payload_count} 份</option>}</For></select></label>
        <For each={textFields}>{([key, label]) => <label><span>{label}</span><textarea value={fields()[key]} onInput={(event) => setFields((value) => ({ ...value, [key]: event.currentTarget.value }))} /></label>}</For>
        <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, i) => i === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在冻结规格…" : "登记 Stage 95 零能力解析器规格"}</button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().registrations}>{(registration) => (
        <article class="public-admin-reward-governance">
          <header><strong>parser spec {registration.registration_id}</strong><span>{registration.status}</span></header>
          <p>{registration.parser_specification.source_contract_revision} · {registration.parser_specification.synthetic_test_vectors.length} 个合成向量 · spec {registration.parser_specification.parser_specification_sha256.slice(0, 16)}…</p>
          <p><strong>明确禁止：</strong>前填 {String(registration.parser_specification.forward_fill_allowed)} · 插值 {String(registration.parser_specification.interpolation_allowed)} · 未调整价回退 {String(registration.parser_specification.unadjusted_close_fallback_allowed)} · 推断分红拆股 {String(registration.parser_specification.inferred_dividend_or_split_allowed)}</p>
          <p class="public-admin-anchor-boundary">已登记但未复核、未实现、未运行；下一步只能由新角色做规格独立复核。</p>
        </article>
      )}</For>
    </section>
  )}</Show>;
}
