import { For, Show, createSignal, onMount } from "solid-js";

import {
  getControlledShadowMarketDataAdapterAuthorizations,
  reviewControlledShadowMarketDataAdapterAuthorization,
} from "@/lib/api";
import type {
  ControlledShadowMarketDataAdapterAuthorizationRegistry,
  ReviewControlledShadowMarketDataAdapterAuthorizationRequest,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "确认精确绑定当前 Stage 51–91 完整哈希链",
  "确认复核者独立于任务领取者和完整既有责任链",
  "确认只允许固定 HTTPS 路径白名单与 GET 请求",
  "确认只允许交易日历、证券/SPY 价格、分红、拆股和公司行动",
  "确认未来精确股票集合与时间窗口必须先内容寻址",
  "确认凭据不写入授权、收据、日志或响应",
  "确认未来请求、响应、来源正文和可用时间必须分别留哈希",
  "确认只允许自然前向，禁止回填或改写历史",
  "确认批准仅开放未来 claim-first、create-once 数据收据资格",
  "确认本次不解析日历、不发请求、不读行情、不启动 runtime",
  "确认本次不开始观察、不建账、不写持仓/绩效/模型/指标",
  "确认不训练、不反馈 reward、不生成订单、不接券商、不交易",
  "确认没有把未确认的 Hari/老王观点写成系统规则",
] as const;

type TextFields = Pick<
  ReviewControlledShadowMarketDataAdapterAuthorizationRequest,
  | "rationale"
  | "source_allowlist_assessment"
  | "credential_and_request_minimization_assessment"
  | "content_addressing_and_custody_assessment"
  | "known_limitations"
  | "future_receipt_constraints"
>;

const EMPTY_TEXT_FIELDS: TextFields = {
  rationale: "",
  source_allowlist_assessment: "",
  credential_and_request_minimization_assessment: "",
  content_addressing_and_custody_assessment: "",
  known_limitations: "",
  future_receipt_constraints: "",
};

export function PublicAdminControlledShadowMarketDataAdapterAuthorizationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowMarketDataAdapterAuthorizationRegistry>();
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [fields, setFields] = createSignal<TextFields>({ ...EMPTY_TEXT_FIELDS });
  const [verdict, setVerdict] = createSignal<ReviewControlledShadowMarketDataAdapterAuthorizationRequest["verdict"]>(
    "approved_for_future_claim_first_read_only_market_data_receipt",
  );
  const [busyId, setBusyId] = createSignal("");
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      setRegistry(await getControlledShadowMarketDataAdapterAuthorizations());
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 92 行情适配器授权表读取失败");
    }
  };
  onMount(() => void load());

  const updateField = (key: keyof TextFields, value: string) => {
    setFields((current) => ({ ...current, [key]: value }));
  };
  const formReady = () =>
    checks().every(Boolean) && Object.values(fields()).every((value) => value.trim().length > 0);

  const submit = async (
    item: ControlledShadowMarketDataAdapterAuthorizationRegistry["items"][number],
  ) => {
    if (!item.review_eligible || !formReady()) return;
    const claim = item.claim;
    const request: ReviewControlledShadowMarketDataAdapterAuthorizationRequest = {
      expected_cycle_claim_sha256: claim.cycle_claim_sha256,
      expected_authorization_review_sha256: claim.authorization_review_sha256,
      expected_validation_sha256: claim.validation_sha256,
      expected_initialization_manifest_sha256: claim.initialization_manifest_sha256,
      verdict: verdict(),
      ...fields(),
      exact_stage_51_through_stage_91_binding_confirmed: true,
      reviewer_independent_from_claimant_and_complete_prior_chain_confirmed: true,
      fixed_get_only_https_origin_and_path_allowlist_confirmed: true,
      calendar_security_spy_price_dividend_split_only_confirmed: true,
      exact_future_symbol_set_and_time_window_must_be_content_addressed_confirmed: true,
      credentials_never_persisted_forwarded_or_returned_confirmed: true,
      request_response_source_and_retrieval_time_hashes_required_confirmed: true,
      natural_forward_only_no_backfill_or_history_rewrite_confirmed: true,
      approval_only_opens_future_claim_first_read_only_receipt_confirmed: true,
      no_data_request_calendar_resolution_or_runtime_started_confirmed: true,
      no_observation_ledger_position_performance_or_model_metric_write_confirmed: true,
      no_training_feedback_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    setBusyId(claim.cycle_claim_id);
    setError("");
    setNotice("");
    try {
      setRegistry(await reviewControlledShadowMarketDataAdapterAuthorization(
        claim.cycle_claim_id,
        request,
      ));
      setChecks(REVIEW_CHECKS.map(() => false));
      setFields({ ...EMPTY_TEXT_FIELDS });
      setNotice("Stage 92 复核已不可覆盖写入；即使批准，也没有解析日历或读取任何行情。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 92 行情适配器授权失败");
    } finally {
      setBusyId("");
      await load();
    }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="Stage 92 只读行情适配器授权">
        <header><strong>第 92 阶段 · 只读行情适配器授权</strong><span>{current().authorization_status}</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>已领取任务</span><strong>{current().claimed_task_count}</strong></div>
          <div><span>待复核</span><strong>{current().review_eligible_count}</strong></div>
          <div><span>已批准合同</span><strong>{current().approved_count}</strong></div>
          <div><span>未来收据资格</span><strong>{current().future_claim_first_read_only_market_data_receipt_eligible_count}</strong></div>
        </div>
        <p class="public-admin-anchor-boundary">
          固定合同：{current().adapter_specification.allowed_http_methods.join(" / ")}；基准 {current().adapter_specification.benchmark_symbol}；
          仅 {current().adapter_specification.allowed_data_classes.join("、")}。查询参数仅允许 {current().adapter_specification.allowed_query_parameter_names.join("、")}，
          其中凭据参数必须脱敏且排除在规范请求哈希之外；任意 URL、任意股票、重定向和历史回填均关闭。
        </p>
        <For each={current().adapter_specification.allowed_https_origin_and_path_prefixes}>{(source) => (
          <code class="public-admin-anchor-boundary">{source}</code>
        )}</For>
        <Show when={current().review_eligible_count > 0}>
          <div class="public-admin-decision-checks"><For each={REVIEW_CHECKS}>{(label, index) => (
            <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
          )}</For></div>
          <select class="public-admin-decision-select" value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ReviewControlledShadowMarketDataAdapterAuthorizationRequest["verdict"])}>
            <option value="approved_for_future_claim_first_read_only_market_data_receipt">批准未来一次只读数据收据资格</option>
            <option value="rejected_market_data_adapter_contract">拒绝该适配器合同</option>
          </select>
          <textarea class="public-admin-decision-textarea" value={fields().rationale} onInput={(event) => updateField("rationale", event.currentTarget.value)} placeholder="复核理由（必填）" />
          <textarea class="public-admin-decision-textarea" value={fields().source_allowlist_assessment} onInput={(event) => updateField("source_allowlist_assessment", event.currentTarget.value)} placeholder="来源白名单评估（必填）" />
          <textarea class="public-admin-decision-textarea" value={fields().credential_and_request_minimization_assessment} onInput={(event) => updateField("credential_and_request_minimization_assessment", event.currentTarget.value)} placeholder="凭据与请求最小化评估（必填）" />
          <textarea class="public-admin-decision-textarea" value={fields().content_addressing_and_custody_assessment} onInput={(event) => updateField("content_addressing_and_custody_assessment", event.currentTarget.value)} placeholder="内容寻址与证据保管评估（必填）" />
          <textarea class="public-admin-decision-textarea" value={fields().known_limitations} onInput={(event) => updateField("known_limitations", event.currentTarget.value)} placeholder="已知局限（必填）" />
          <textarea class="public-admin-decision-textarea" value={fields().future_receipt_constraints} onInput={(event) => updateField("future_receipt_constraints", event.currentTarget.value)} placeholder="未来收据约束（必填）" />
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().items}>{(item) => (
          <article class="public-admin-reward-governance">
            <header><strong>claim {item.claim.cycle_claim_id}</strong><span>{item.authorization?.verdict ?? "等待责任链外复核"}</span></header>
            <p>claim {item.claim.cycle_claim_sha256.slice(0, 16)}… · manifest {item.claim.initialization_manifest_sha256.slice(0, 16)}…</p>
            <Show when={item.authorization}>{(authorization) => (
              <p class="public-admin-anchor-boundary">合同授权有效至 {authorization().authorized_valid_until}；HTTP 请求、日历解析和行情读取仍为 false。</p>
            )}</Show>
            <Show when={item.review_eligible}>
              <div class="public-admin-decision-actions">
                <button type="button" class="public-admin-decision-submit" disabled={busyId() !== "" || !formReady()} onClick={() => void submit(item)}>{busyId() === item.claim.cycle_claim_id ? "正在写入不可覆盖复核…" : "提交 Stage 92 复核"}</button>
              </div>
            </Show>
          </article>
        )}</For>
      </section>
    )}</Show>
  );
}
