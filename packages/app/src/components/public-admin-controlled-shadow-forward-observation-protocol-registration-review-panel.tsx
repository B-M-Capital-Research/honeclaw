import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowForwardObservationProtocolRegistrationReviews,
  reviewControlledShadowForwardObservationProtocolRegistration,
} from "@/lib/api";
import type {
  ControlledShadowForwardObservationProtocolRegistrationReviewRegistry,
  ControlledShadowForwardObservationProtocolRegistrationReviewVerdict,
} from "@/lib/types";

const CHECKS = [
  "精确绑定当前 Stage 51–82 完整责任链",
  "复核人独立于 Stage 82 登记人和全部上游角色",
  "已独立重算登记、前向协议和完整 Stage 74 设计指纹",
  "最早观察时点有效，禁止回填登记或批准前日期",
  "每周 claim-first、create-once",
  "官方美股日历、半日市、停牌和证券/SPY 同步语义完整",
  "点时白名单来源内容寻址，并保存来源可用性证据",
  "原始/复权价格、分红、拆股、公司行动和追加更正完整",
  "下一完整交易日、单边 25bp 成本和全部反事实保持冻结",
  "仅多头普通股，仓位上限、现金底线且无期权、杠杆或做空",
  "21/63/126/252 检查点和 252/40/12/4 最低门槛生效，禁止提前晋级",
  "指标分开报告、处理多重检验且无综合分或标量奖励",
  "停止与证伪失败关闭，停止后不得原位重启",
  "未把未确认 Hari/老王观点写成工程规则",
  "批准只开放未来零能力观察实现规格登记",
  "本阶段不观察、不建账、不写持仓/绩效/模型/指标，不反馈、不奖励、不下单、不接券商或交易",
] as const;

const emptyTexts = () => ({
  rationale: "",
  natural_forward_assessment: "",
  calendar_and_timing_assessment: "",
  source_custody_and_correction_assessment: "",
  metric_and_stop_assessment: "",
  known_limitations: "",
  future_implementation_constraints: "",
});

export function PublicAdminControlledShadowForwardObservationProtocolRegistrationReviewPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowForwardObservationProtocolRegistrationReviewRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<ControlledShadowForwardObservationProtocolRegistrationReviewVerdict>(
      "changes_required_rebuild_forward_observation_protocol",
    );
  const [texts, setTexts] = createSignal(emptyTexts());
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowForwardObservationProtocolRegistrationReviews();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.review_eligible);
      if (!eligible.some((item) => item.registered_protocol.registration.protocol_registration_id === selectedId())) {
        setSelectedId(eligible[0]?.registered_protocol.registration.protocol_registration_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 83 前向观察协议独立复核表读取失败");
    }
  };
  onMount(() => void load());

  const eligible = createMemo(() => registry()?.items.filter((item) => item.review_eligible) ?? []);
  const selected = createMemo(() => eligible().find(
    (item) => item.registered_protocol.registration.protocol_registration_id === selectedId(),
  ));
  const approval = createMemo(() =>
    verdict() === "approved_for_future_zero_capability_forward_observation_implementation_registration"
  );
  const disabled = createMemo(() => busy() || !selected()
    || Object.values(texts()).some((value) => !value.trim())
    || (approval() && checks().some((value) => !value)));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const r = item.registered_protocol.registration;
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await reviewControlledShadowForwardObservationProtocolRegistration(
        r.protocol_registration_id,
        {
          expected_review_id: item.latest_review?.review_id,
          expected_review_sha256: item.latest_review?.review_sha256,
          expected_protocol_registration_id: r.protocol_registration_id,
          expected_protocol_registration_sha256: r.protocol_registration_sha256,
          expected_protocol_specification_sha256: r.protocol_specification.specification_sha256,
          expected_validation_sha256: r.validation_sha256,
          expected_claim_sha256: r.claim_sha256,
          expected_result_sha256: r.result_sha256,
          expected_output_sha256: r.output_sha256,
          expected_input_manifest_sha256: r.input_manifest_sha256,
          expected_authorization_review_sha256: r.authorization_review_sha256,
          expected_isolated_runner_spec_sha256: r.isolated_runner_spec_sha256,
          expected_runner_artifact_sha256: r.runner_artifact_sha256,
          expected_implementation_contract_sha256: r.implementation_contract_sha256,
          expected_design_specification_sha256: r.design_specification_sha256,
          expected_candidate_set_sha256: r.candidate_set_sha256,
          expected_feature_order_sha256: r.feature_order_sha256,
          expected_preprocessing_sha256: r.preprocessing_sha256,
          expected_target_id: r.target_id,
          expected_frozen_candidate_algorithm_id: r.frozen_candidate_algorithm_id,
          verdict: verdict(),
          ...texts(),
          exact_current_stage_51_through_stage_82_binding_confirmed: checks()[0] as boolean,
          reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: checks()[1] as boolean,
          independent_recomputation_of_registration_protocol_and_design_fingerprints_confirmed: checks()[2] as boolean,
          observation_not_before_and_no_retroactive_backfill_reviewed: checks()[3] as boolean,
          weekly_claim_first_create_once_reviewed: checks()[4] as boolean,
          official_us_market_calendar_half_days_halts_and_spy_sync_reviewed: checks()[5] as boolean,
          point_in_time_allowlist_content_addressing_and_source_availability_reviewed: checks()[6] as boolean,
          raw_adjusted_prices_dividends_splits_corporate_actions_and_append_only_corrections_reviewed: checks()[7] as boolean,
          next_full_session_fill_25bps_cost_and_counterfactuals_reviewed: checks()[8] as boolean,
          long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed: checks()[9] as boolean,
          checkpoints_and_252_40_12_4_minimums_without_early_promotion_reviewed: checks()[10] as boolean,
          separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed: checks()[11] as boolean,
          stop_falsification_fail_closed_and_no_in_place_restart_reviewed: checks()[12] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[13] as boolean,
          approval_only_opens_future_zero_capability_observation_implementation_registration_confirmed: checks()[14] as boolean,
          no_observation_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed: checks()[15] as boolean,
        },
      );
      setRegistry(next); setTexts(emptyTexts()); setChecks(CHECKS.map(() => false));
      setVerdict("changes_required_rebuild_forward_observation_protocol");
      setNotice("Stage 83 独立复核已追加写入；批准也不会开始观察或创建任何持仓。 ");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 83 前向观察协议独立复核失败");
      await load();
    } finally { setBusy(false); }
  };

  const textFields = [
    ["rationale", "复核理由"],
    ["natural_forward_assessment", "自然前向与禁止回填评估"],
    ["calendar_and_timing_assessment", "交易日历、SPY 同步与成交时点评估"],
    ["source_custody_and_correction_assessment", "点时来源、公司行动与追加更正评估"],
    ["metric_and_stop_assessment", "指标、样本门槛与停止规则评估"],
    ["known_limitations", "已知局限"],
    ["future_implementation_constraints", "未来实现约束"],
  ] as const;

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="Stage 83 前向观察协议独立复核">
      <header><strong>第 83 阶段 · 前向观察协议责任链外独立复核</strong><span>{current().review_status}</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>已登记协议</span><strong>{current().protocol_registered_count}</strong></div>
        <div><span>待独立复核</span><strong>{current().review_eligible_count}</strong></div>
        <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
        <div><span>独立通过</span><strong>{current().independently_approved_count}</strong></div>
        <div><span>待重建/拒绝</span><strong>{current().changes_required_or_rejected_count}</strong></div>
        <div><span>可登记零能力实现</span><strong>{current().future_zero_capability_forward_observation_implementation_registration_eligible_count}</strong></div>
      </div>
      <p class="public-admin-anchor-boundary">通过只证明协议已被独立复算和审查，不证明模型有效，更不代表允许观察、建账、建仓或交易。</p>
      <Show when={eligible().length > 0} fallback={<p>当前没有待独立复核的 Stage 82 协议。</p>}>
        <label><span>待复核协议</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={eligible()}>{(item) => { const r = item.registered_protocol.registration; return <option value={r.protocol_registration_id}>{r.protocol_registration_id.slice(0, 12)}… · {r.target_id}</option>; }}</For></select></label>
        <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ControlledShadowForwardObservationProtocolRegistrationReviewVerdict)}>
          <option value="changes_required_rebuild_forward_observation_protocol">要求修改并重建协议</option>
          <option value="rejected_forward_observation_protocol">拒绝协议</option>
          <option value="approved_for_future_zero_capability_forward_observation_implementation_registration">批准进入零能力观察实现规格登记</option>
        </select></label>
        <For each={textFields}>{([key, label]) => <label><span>{label}</span><textarea value={texts()[key]} onInput={(event) => setTexts((value) => ({ ...value, [key]: event.currentTarget.value }))} /></label>}</For>
        <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, i) => i === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在追加复核…" : "提交 Stage 83 独立复核"}</button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().items}>{(item) => <Show when={item.latest_review}>{(review) => <article class="public-admin-reward-governance"><header><strong>协议 {item.registered_protocol.registration.protocol_registration_id.slice(0, 12)}…</strong><span>{review().verdict}</span></header><p>{review().submitted_at} · {review().reviewer_id}</p><p><strong>理由：</strong>{review().rationale}</p><p><strong>自然前向：</strong>{review().natural_forward_assessment}</p><p><strong>未来实现约束：</strong>{review().future_implementation_constraints}</p><p class="public-admin-anchor-boundary">观察、账本、持仓、绩效、订单、券商与交易权限仍全部关闭。</p></article>}</Show>}</For>
    </section>
  )}</Show>;
}
