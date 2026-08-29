import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowExperimentDesignRegistrationReviews,
  reviewControlledShadowExperimentDesignRegistration,
} from "@/lib/api";
import type {
  ControlledShadowExperimentDesignRegistrationReviewRegistry,
  ControlledShadowExperimentDesignRegistrationReviewVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–74 完整责任链",
  "已用独立实现复算登记和设计指纹",
  "复核人独立于登记人和全部上游角色",
  "实验候选不是正式模型选择",
  "已审查点时数据、成分股、幸存者偏差、退市和前视泄漏",
  "已审查 SPY、现金、等权与冻结规则的反事实语义",
  "已审查信号时点、成交、成本、分红和调仓",
  "仅多头普通股；仓位上限、现金底线且无期权、杠杆或做空",
  "252 日及样本、公司、季度门槛生效，禁止提前晋级",
  "指标分开报告、处理多重检验且无综合分或标量奖励",
  "停止、证伪和禁止原位重启规则完整",
  "未把未确认 Hari/老王观点写成规则",
  "批准只开放未来零能力影子实现规格登记",
  "不写模型/指标库，不训练、不奖励、不建仓、不下单、不接券商或交易",
] as const;

const emptyTexts = () => ({
  rationale: "",
  risk_assessment: "",
  known_limitations: "",
  falsification_assessment: "",
  future_implementation_constraints: "",
});

export function PublicAdminControlledShadowExperimentDesignRegistrationReviewPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowExperimentDesignRegistrationReviewRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<ControlledShadowExperimentDesignRegistrationReviewVerdict>(
      "changes_requested_requires_new_design_registration",
    );
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [texts, setTexts] = createSignal(emptyTexts());
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowExperimentDesignRegistrationReviews();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.review_eligible);
      if (!eligible.some((item) => item.registered_design.registration.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(eligible[0]?.registered_design.registration.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "受控影子实验设计独立复核表读取失败");
    }
  };

  onMount(() => void load());

  const eligibleItems = createMemo(
    () => registry()?.items.filter((item) => item.review_eligible) ?? [],
  );
  const selected = createMemo(() =>
    eligibleItems().find(
      (item) => item.registered_design.registration.attempt_id === selectedAttemptId(),
    ),
  );
  const allTextsPresent = createMemo(() =>
    Object.values(texts()).every((value) => value.trim().length > 0),
  );
  const approvalChecksComplete = createMemo(
    () => verdict() !== "approved_for_future_zero_capability_shadow_implementation_registration"
      || checks().every(Boolean),
  );
  const disabled = createMemo(
    () => busy() || !selected() || !allTextsPresent() || !approvalChecksComplete(),
  );

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const registration = item.registered_design.registration;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewControlledShadowExperimentDesignRegistration(
        registration.attempt_id,
        {
          expected_review_id: item.latest_review?.review_id,
          expected_review_sha256: item.latest_review?.review_sha256,
          expected_registration_id: registration.registration_id,
          expected_registration_sha256: registration.registration_sha256,
          expected_adjudication_review_id: registration.adjudication_review_id,
          expected_adjudication_review_sha256: registration.adjudication_review_sha256,
          expected_output_validation_id: registration.output_validation_id,
          expected_output_validation_sha256: registration.output_validation_sha256,
          expected_claim_sha256: registration.claim_sha256,
          expected_result_sha256: registration.result_sha256,
          expected_output_sha256: registration.output_sha256,
          expected_envelope_sha256: registration.envelope_sha256,
          expected_candidate_set_sha256: registration.candidate_set_sha256,
          expected_selected_algorithm_three_seed_binding_sha256:
            registration.selected_algorithm_three_seed_binding_sha256,
          expected_design_specification_sha256:
            registration.design_specification.specification_sha256,
          expected_target_id: registration.target_id,
          expected_frozen_candidate_algorithm_id: registration.frozen_candidate_algorithm_id,
          verdict: verdict(),
          ...texts(),
          exact_current_stage_51_through_stage_74_binding_confirmed: checks()[0] as boolean,
          independent_recomputation_of_registration_and_design_fingerprints_confirmed:
            checks()[1] as boolean,
          reviewer_independent_from_registrar_and_complete_prior_chain_confirmed:
            checks()[2] as boolean,
          experimental_candidate_not_official_model_selection_confirmed: checks()[3] as boolean,
          point_in_time_universe_survivorship_delisting_and_no_lookahead_reviewed:
            checks()[4] as boolean,
          benchmark_and_all_counterfactual_semantics_reviewed: checks()[5] as boolean,
          signal_timing_execution_cost_dividends_and_rebalance_reviewed: checks()[6] as boolean,
          long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed:
            checks()[7] as boolean,
          minimum_windows_sample_symbol_quarter_gates_and_no_early_promotion_reviewed:
            checks()[8] as boolean,
          separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed:
            checks()[9] as boolean,
          stop_rules_falsification_and_no_in_place_restart_reviewed: checks()[10] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[11] as boolean,
          approval_only_opens_future_zero_capability_shadow_implementation_registration_confirmed:
            checks()[12] as boolean,
          no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed:
            checks()[13] as boolean,
        },
      );
      setRegistry(next);
      setChecks(REVIEW_CHECKS.map(() => false));
      setTexts(emptyTexts());
      setVerdict("changes_requested_requires_new_design_registration");
      setNotice("独立复核已追加写入。即使批准，也只开放未来零能力影子实现规格登记。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "受控影子实验设计独立复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(current) => (
        <section class="public-admin-reward-governance" aria-label="受控影子实验设计独立复核">
          <header>
            <strong>第 75 阶段 · 受控影子实验设计独立复核</strong>
            <span>{current().review_status}</span>
          </header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>已登记设计</span><strong>{current().registered_design_count}</strong></div>
            <div><span>待独立复核</span><strong>{current().review_eligible_count}</strong></div>
            <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
            <div><span>独立通过</span><strong>{current().independently_approved_count}</strong></div>
            <div><span>待改/拒绝</span><strong>{current().changes_requested_or_rejected_count}</strong></div>
            <div><span>可登记零能力实现</span><strong>{current().future_zero_capability_shadow_implementation_registration_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>审批边界</strong><span>仍不创建影子盘</span></header>
            <p>复核者必须独立复算 Stage 74 登记和设计指纹，并逐项确认数据时点、幸存者偏差、退市、反事实、交易成本、组合边界、观察门槛、多重检验与停止规则。</p>
            <p class="public-admin-anchor-boundary">通过不等于模型有效或可交易；它只允许下一阶段登记一个零能力、无账本、无订单、无券商访问的实现规格。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有待独立复核的 Stage 74 设计。</p>}>
            <label>
              <span>待复核设计</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={eligibleItems()}>{(item) => {
                  const registration = item.registered_design.registration;
                  return <option value={registration.attempt_id}>{registration.attempt_id.slice(0, 12)}… · {registration.target_id} · {registration.experiment_name}</option>;
                }}</For>
              </select>
            </label>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ControlledShadowExperimentDesignRegistrationReviewVerdict)}>
                <option value="changes_requested_requires_new_design_registration">要求修改（必须新建设计）</option>
                <option value="rejected">拒绝</option>
                <option value="approved_for_future_zero_capability_shadow_implementation_registration">批准进入零能力实现规格登记</option>
              </select>
            </label>
            <label><span>复核理由</span><textarea value={texts().rationale} onInput={(event) => setTexts((value) => ({ ...value, rationale: event.currentTarget.value }))} /></label>
            <label><span>风险评估</span><textarea value={texts().risk_assessment} onInput={(event) => setTexts((value) => ({ ...value, risk_assessment: event.currentTarget.value }))} /></label>
            <label><span>已知局限</span><textarea value={texts().known_limitations} onInput={(event) => setTexts((value) => ({ ...value, known_limitations: event.currentTarget.value }))} /></label>
            <label><span>证伪评估</span><textarea value={texts().falsification_assessment} onInput={(event) => setTexts((value) => ({ ...value, falsification_assessment: event.currentTarget.value }))} /></label>
            <label><span>未来实现约束</span><textarea value={texts().future_implementation_constraints} onInput={(event) => setTexts((value) => ({ ...value, future_implementation_constraints: event.currentTarget.value }))} /></label>
            <div class="public-admin-decision-checks">
              <For each={REVIEW_CHECKS}>{(label, index) => (
                <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
              )}</For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在追加复核…" : "提交独立复核"}</button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={current().items}>{(item) => (
            <Show when={item.latest_review}>{(review) => (
              <article class="public-admin-reward-governance">
                <header><strong>{item.registered_design.registration.experiment_name}</strong><span>{review().verdict}</span></header>
                <p>{review().submitted_at} · {review().reviewer_id}</p>
                <p><strong>理由：</strong>{review().rationale}</p>
                <p><strong>风险：</strong>{review().risk_assessment}</p>
                <p><strong>未来实现约束：</strong>{review().future_implementation_constraints}</p>
                <p class="public-admin-anchor-boundary">影子实现、运行、账本、持仓、订单、券商与交易权限仍全部关闭。</p>
              </article>
            )}</Show>
          )}</For>
        </section>
      )}
    </Show>
  );
}
