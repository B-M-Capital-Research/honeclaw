import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowExperimentDesignRegistrations,
  registerControlledShadowExperimentDesign,
} from "@/lib/api";
import type { ControlledShadowExperimentDesignRegistrationRegistry } from "@/lib/types";

const REGISTRATION_CHECKS = [
  "精确绑定 Stage 73 裁决及 Stage 51–73 完整责任链",
  "登记人独立于裁决者、验证者、执行者和全部上游角色",
  "当前只是实验候选，不构成正式选模",
  "只允许未来点时数据和前向观察，禁止回看或覆盖历史",
  "SPY/现金/等权/规则基线、成本和每周调仓全部冻结",
  "仅多头普通股，单股 5%、主题 20%、总仓 60%、现金至少 40%",
  "至少观察 252 个交易日，并满足信号、公司和季度覆盖；不得提前晋级",
  "所有指标分开展示、进行多重检验，不创建综合分掩盖失败",
  "停止规则和证伪条件在运行前冻结，停止后不得原地重启",
  "任何影子运行申请前还必须完成新的独立设计复核",
  "不写模型/指标库、不反馈训练/reward，不创建影子持仓、订单、券商访问或交易",
] as const;

const emptyTexts = () => ({
  experiment_name: "Stage 74 前向受控影子实验",
  research_hypothesis: "",
  economic_thesis: "",
  known_limitations: "",
  falsification_conditions: "",
});

export function PublicAdminControlledShadowExperimentDesignRegistrationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowExperimentDesignRegistrationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checks, setChecks] = createSignal(REGISTRATION_CHECKS.map(() => false));
  const [texts, setTexts] = createSignal(emptyTexts());
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowExperimentDesignRegistrations();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.registration_eligible);
      if (!eligible.some((item) => item.source.review.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(eligible[0]?.source.review.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "受控影子实验设计登记表读取失败");
    }
  };

  onMount(() => void load());

  const eligibleItems = createMemo(
    () => registry()?.items.filter((item) => item.registration_eligible) ?? [],
  );
  const selected = createMemo(() =>
    eligibleItems().find((item) => item.source.review.attempt_id === selectedAttemptId()),
  );
  const allTextsPresent = createMemo(() =>
    Object.values(texts()).every((value) => value.trim().length > 0),
  );
  const disabled = createMemo(
    () => busy() || !selected() || !allTextsPresent() || checks().some((value) => !value),
  );

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const { candidate, review } = item.source;
    const validation = candidate.source.validation;
    const envelope = candidate.source.attempt.result.untrusted_confirmation_envelope;
    if (!envelope) {
      setError("当前候选缺少 sealed-holdout 确认信封");
      return;
    }
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await registerControlledShadowExperimentDesign(
        review.attempt_id,
        {
          expected_adjudication_review_id: review.review_id,
          expected_adjudication_review_sha256: review.review_sha256,
          expected_output_validation_id: validation.validation_id,
          expected_output_validation_sha256: validation.validation_sha256,
          expected_claim_sha256: validation.claim_sha256,
          expected_result_sha256: validation.result_sha256,
          expected_output_sha256: validation.output_sha256,
          expected_envelope_sha256: candidate.envelope_sha256,
          expected_candidate_set_sha256: validation.candidate_set_sha256,
          expected_selected_algorithm_three_seed_binding_sha256:
            validation.selected_algorithm_three_seed_binding_sha256,
          expected_target_id: envelope.target_id,
          expected_frozen_candidate_algorithm_id: envelope.frozen_candidate_algorithm_id,
          ...texts(),
          exact_stage_73_adjudication_and_complete_chain_confirmed: checks()[0] as true,
          registrar_independent_from_complete_prior_chain_confirmed: checks()[1] as true,
          experimental_candidate_not_official_model_selection_confirmed: checks()[2] as true,
          point_in_time_forward_only_and_no_retroactive_revision_confirmed: checks()[3] as true,
          benchmark_comparators_costs_and_rebalance_frozen_confirmed: checks()[4] as true,
          portfolio_caps_cash_floor_and_long_only_boundary_confirmed: checks()[5] as true,
          minimum_observation_windows_and_no_early_promotion_confirmed: checks()[6] as true,
          separate_metrics_multiple_testing_and_no_composite_confirmed: checks()[7] as true,
          stop_rules_and_falsification_are_frozen_confirmed: checks()[8] as true,
          independent_design_review_required_before_any_shadow_run_request_confirmed:
            checks()[9] as true,
          no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed:
            checks()[10] as true,
        },
      );
      setRegistry(next);
      setChecks(REGISTRATION_CHECKS.map(() => false));
      setTexts(emptyTexts());
      setNotice("设计已不可变登记。下一步只能进行独立设计复核；尚未创建影子账本或持仓。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "受控影子实验设计登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(current) => (
        <section class="public-admin-reward-governance" aria-label="受控影子实验设计登记">
          <header>
            <strong>第 74 阶段 · 受控影子实验设计登记</strong>
            <span>{current().registration_status}</span>
          </header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>已裁决候选</span><strong>{current().adjudicated_candidate_count}</strong></div>
            <div><span>待登记</span><strong>{current().registration_eligible_count}</strong></div>
            <div><span>已登记</span><strong>{current().registered_design_count}</strong></div>
            <div><span>待独立复核</span><strong>{current().future_independent_design_review_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>冻结前向实验协议</strong><span>仍不运行</span></header>
            <p>虚拟本金 100 万美元；仅多头普通股；单股 5%、主题 20%、总仓 60%、现金至少 40%；每周调仓，下一完整交易日模拟成交，每边滑点 25bp。</p>
            <p>至少观察 252 个交易日，在 21/63/126/252 日检查；净超额收益、回撤、下行捕获、换手成本、集中度和方向命中率分别报告，不生成综合分。</p>
            <p class="public-admin-anchor-boundary">设计登记不是正式选模，更不是开始模拟持仓。还需要另一位独立管理员复核。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有待登记的 Stage 73 裁决结果。</p>}>
            <label>
              <span>待登记 attempt</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={eligibleItems()}>{(item) => <option value={item.source.review.attempt_id}>{item.source.review.attempt_id.slice(0, 12)}… · {item.source.review.target_id}</option>}</For>
              </select>
            </label>
            <label><span>实验名称</span><input value={texts().experiment_name} onInput={(event) => setTexts((value) => ({ ...value, experiment_name: event.currentTarget.value }))} /></label>
            <label><span>研究假设</span><textarea value={texts().research_hypothesis} onInput={(event) => setTexts((value) => ({ ...value, research_hypothesis: event.currentTarget.value }))} /></label>
            <label><span>经济假设</span><textarea value={texts().economic_thesis} onInput={(event) => setTexts((value) => ({ ...value, economic_thesis: event.currentTarget.value }))} /></label>
            <label><span>已知局限</span><textarea value={texts().known_limitations} onInput={(event) => setTexts((value) => ({ ...value, known_limitations: event.currentTarget.value }))} /></label>
            <label><span>证伪条件</span><textarea value={texts().falsification_conditions} onInput={(event) => setTexts((value) => ({ ...value, falsification_conditions: event.currentTarget.value }))} /></label>
            <div class="public-admin-decision-checks">
              <For each={REGISTRATION_CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在写入不可变设计…" : "登记受控影子实验设计"}</button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={current().items}>{(item) => <Show when={item.registration}>{(registration) => <article class="public-admin-reward-governance"><header><strong>{registration().experiment_name}</strong><span>等待独立设计复核</span></header><p>{registration().registered_at} · {registration().registered_by}</p><p><strong>目标：</strong>{registration().target_id} · {registration().frozen_candidate_algorithm_id}</p><p><strong>研究假设：</strong>{registration().research_hypothesis}</p><p><strong>经济假设：</strong>{registration().economic_thesis}</p><p><strong>证伪：</strong>{registration().falsification_conditions}</p><p class="public-admin-anchor-boundary">影子运行、账本、持仓、订单、券商与交易权限全部关闭。</p></article>}</Show>}</For>
        </section>
      )}
    </Show>
  );
}
