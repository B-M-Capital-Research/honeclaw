import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getOpeningPortfolioSnapshotGovernanceSpecifications,
  registerOpeningPortfolioSnapshotGovernanceSpecification,
} from "@/lib/api";
import type {
  OpeningPortfolioExternalSourceKind,
  OpeningPortfolioSnapshotGovernanceSpecificationRegistry,
} from "@/lib/types";

const SPECIFICATION_CHECKS = [
  "精确绑定当前 Stage 51–124 完整责任链",
  "登记人独立于 Stage 124 复核者和全部既有责任人",
  "已重新打开、重哈希并确认 Stage 124 准入记录仍为当前记录",
  "未来必须提供外部原始文件，禁止手填余额和持仓",
  "账户范围完整，组合别名不含真实账号",
  "现金、持仓、负债和未结算活动必须完整覆盖",
  "数量与金额使用精确十进制和有符号数量，不补默认值、不推断",
  "证券身份与公司行动必须完成对账",
  "对账单市值只作信息参考，不能直接成为会计估值",
  "计算净值前必须取得独立行情、汇率与衍生品估值",
  "原始文件验收、快照物化、输出验证和快照准入是后续分离关卡",
  "本阶段仅登记规格，不上传、读取、解析文件或生成快照",
  "不创建账本事件、持仓、现金、净值/绩效、模型、训练/RL、订单、券商或交易权限",
  "登记后必须进入 Stage 126 责任链外独立规格复核",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

function localDateTimeValue() {
  const date = new Date();
  date.setMinutes(date.getMinutes() - date.getTimezoneOffset());
  return date.toISOString().slice(0, 16);
}

export function PublicAdminOpeningPortfolioSnapshotGovernanceSpecificationPanel() {
  const [registry, setRegistry] =
    createSignal<OpeningPortfolioSnapshotGovernanceSpecificationRegistry>();
  const [selectedReviewId, setSelectedReviewId] = createSignal("");
  const [sourceKind, setSourceKind] = createSignal<OpeningPortfolioExternalSourceKind>(
    "broker_or_custodian_machine_export",
  );
  const [provider, setProvider] = createSignal("");
  const [scopeAlias, setScopeAlias] = createSignal("primary_portfolio");
  const [currency, setCurrency] = createSignal("USD");
  const [timezone, setTimezone] = createSignal("America/New_York");
  const [snapshotAsOf, setSnapshotAsOf] = createSignal(localDateTimeValue());
  const [accountCount, setAccountCount] = createSignal(1);
  const [reason, setReason] = createSignal("");
  const [limitations, setLimitations] = createSignal(
    "当前没有外部原始文件、期初组合快照、持仓、现金或净值；本记录只定义未来接入规则。",
  );
  const [constraints, setConstraints] = createSignal(
    "Stage 126 必须由责任链外人员重新打开并独立复核规格；复核通过前不得接收来源文件。",
  );
  const [checks, setChecks] = createSignal(SPECIFICATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const eligible = createMemo(() => {
    const current = registry();
    if (!current) return [];
    const registered = new Set(current.registrations.map((item) => item.stage_124_review_id));
    return current.candidates.filter((item) => !registered.has(item.stage_124_review_id));
  });
  const selected = createMemo(() => eligible().find(
    (item) => item.stage_124_review_id === selectedReviewId(),
  ));

  const load = async () => {
    try {
      const next = await getOpeningPortfolioSnapshotGovernanceSpecifications();
      setRegistry(next);
      const registered = new Set(next.registrations.map((item) => item.stage_124_review_id));
      const nextEligible = next.candidates.filter((item) => !registered.has(item.stage_124_review_id));
      if (!nextEligible.some((item) => item.stage_124_review_id === selectedReviewId())) {
        setSelectedReviewId(nextEligible[0]?.stage_124_review_id ?? "");
      }
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 125 期初组合治理规格读取失败");
    }
  };
  onMount(() => void load());

  const disabled = createMemo(() => busy()
    || !selected()
    || provider().trim().length === 0
    || scopeAlias().trim().length === 0
    || currency().trim().length !== 3
    || timezone().trim().length === 0
    || !snapshotAsOf()
    || accountCount() < 1
    || reason().trim().length === 0
    || limitations().trim().length === 0
    || constraints().trim().length === 0
    || !checks().every(Boolean));

  const submit = async () => {
    const candidate = selected();
    if (!candidate || disabled()) return;
    const parsedAsOf = new Date(snapshotAsOf());
    if (Number.isNaN(parsedAsOf.getTime())) {
      setError("快照时点格式无效");
      return;
    }
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = checks();
      const next = await registerOpeningPortfolioSnapshotGovernanceSpecification(
        candidate.stage_124_review_id,
        {
          expected_stage_124_review_id: candidate.stage_124_review_id,
          expected_stage_124_review_sha256: candidate.stage_124_review_sha256,
          expected_stage_123_validation_sha256: candidate.stage_123_validation_sha256,
          expected_stage_122_candidate_sha256: candidate.stage_122_candidate_sha256,
          expected_stage_114_review_sha256: candidate.stage_114_review_sha256,
          expected_stage_112_output_sha256: candidate.stage_112_output_sha256,
          source_kind: sourceKind(),
          source_provider_name: provider().trim(),
          portfolio_scope_alias: scopeAlias().trim(),
          reporting_currency: currency().trim().toUpperCase(),
          source_timezone: timezone().trim(),
          snapshot_as_of_utc: parsedAsOf.toISOString(),
          expected_account_count: accountCount(),
          registration_reason: reason().trim(),
          known_limitations: limitations().trim(),
          future_review_constraints: constraints().trim(),
          exact_current_stage_51_through_stage_124_binding_confirmed: values[0] as boolean,
          registrar_independent_from_stage_124_reviewer_and_complete_prior_chain_confirmed: values[1] as boolean,
          stage_124_admission_reopened_rehashed_and_current_confirmed: values[2] as boolean,
          external_source_artifact_required_and_manual_balances_forbidden_confirmed: values[3] as boolean,
          account_scope_complete_and_opaque_alias_contains_no_account_number_confirmed: values[4] as boolean,
          all_cash_positions_liabilities_and_unsettled_activity_required_confirmed: values[5] as boolean,
          exact_decimal_signed_quantities_and_no_default_or_inference_confirmed: values[6] as boolean,
          instrument_identity_and_corporate_action_reconciliation_required_confirmed: values[7] as boolean,
          statement_market_values_are_informational_not_accounting_marks_confirmed: values[8] as boolean,
          complete_independent_marks_fx_and_derivative_valuation_required_before_nav_confirmed: values[9] as boolean,
          source_artifact_receipt_validation_and_snapshot_admission_are_separate_future_gates_confirmed: values[10] as boolean,
          specification_only_no_artifact_upload_read_parse_or_snapshot_materialization_confirmed: values[11] as boolean,
          no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: values[12] as boolean,
          future_stage_126_independent_specification_review_required_confirmed: values[13] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: values[14] as boolean,
        },
      );
      setRegistry(next);
      setReason("");
      setChecks(SPECIFICATION_CHECKS.map(() => false));
      setNotice("期初组合来源与快照治理规格已登记；当前仍没有读取来源文件或生成任何持仓、现金与净值。下一步仅为 Stage 126 独立规格复核。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 125 期初组合治理规格登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="外部来源期初组合快照治理规格">
      <header><strong>第 125 阶段 · 外部来源期初组合快照治理规格</strong><span>只定规则 · 不接数据</span></header>
      <p>{current().scope}</p>
      <p class="public-admin-anchor-boundary">当前明确为空：外部原始文件、期初组合快照、金融事件白名单、账本、持仓、现金、净值与绩效。</p>
      <div class="public-admin-decision-metrics">
        <div><span>Stage 124 正式证据</span><strong>{current().stage_124_admitted_evidence_count}</strong></div>
        <div><span>可登记</span><strong>{current().registration_eligible_count}</strong></div>
        <div><span>已登记规格</span><strong>{current().registered_specification_count}</strong></div>
        <div><span>待 Stage 126 复核</span><strong>{current().future_stage_126_independent_specification_review_eligible_count}</strong></div>
      </div>
      <Show when={eligible().length > 0} fallback={<p>当前没有可登记的 Stage 124 正式非财务观察证据。</p>}>
        <label><span>Stage 124 准入记录</span><select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
          <For each={eligible()}>{(item) => <option value={item.stage_124_review_id}>{item.stage_124_review_id.slice(0, 12)}… · {item.formal_non_financial_observation_notice_count} 条通知</option>}</For>
        </select></label>
        <label><span>外部来源类型</span><select value={sourceKind()} onChange={(event) => setSourceKind(event.currentTarget.value as OpeningPortfolioExternalSourceKind)}>
          <option value="broker_or_custodian_machine_export">券商/托管机构机器导出</option>
          <option value="broker_or_custodian_statement">券商/托管机构对账单</option>
          <option value="verified_portfolio_accounting_system_export">已核验组合会计系统导出</option>
        </select></label>
        <label><span>来源机构</span><input value={provider()} onInput={(event) => setProvider(event.currentTarget.value)} placeholder="例如独立托管机构名称" /></label>
        <label><span>组合范围别名</span><input value={scopeAlias()} onInput={(event) => setScopeAlias(event.currentTarget.value)} /></label>
        <label><span>报告币种</span><input value={currency()} maxlength="3" onInput={(event) => setCurrency(event.currentTarget.value)} /></label>
        <label><span>来源时区</span><input value={timezone()} onInput={(event) => setTimezone(event.currentTarget.value)} /></label>
        <label><span>快照时点</span><input type="datetime-local" value={snapshotAsOf()} onInput={(event) => setSnapshotAsOf(event.currentTarget.value)} /></label>
        <label><span>预期账户数</span><input type="number" min="1" max="32" value={accountCount()} onInput={(event) => setAccountCount(Number(event.currentTarget.value))} /></label>
        <label><span>登记理由</span><textarea value={reason()} onInput={(event) => setReason(event.currentTarget.value)} /></label>
        <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
        <label><span>后续复核约束</span><textarea value={constraints()} onInput={(event) => setConstraints(event.currentTarget.value)} /></label>
        <div class="public-admin-decision-checks"><For each={SPECIFICATION_CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在冻结治理规格…" : "登记 Stage 125 治理规格"}</button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().registrations}>{(item) => <article class="public-admin-reward-governance">
        <header><strong>{item.specification.source_contract.source_provider_name} · {item.specification.source_contract.portfolio_scope_alias}</strong><span>{item.registered_at}</span></header>
        <p>{item.specification.source_contract.reporting_currency} · {item.specification.source_contract.snapshot_as_of_utc} · {item.specification.source_contract.expected_account_count} 个账户</p>
        <p class="public-admin-anchor-boundary">规格已冻结，等待 Stage 126 独立复核；尚无来源文件、期初组合、持仓、现金、净值或交易权限。</p>
      </article>}</For>
    </section>
  )}</Show>;
}
