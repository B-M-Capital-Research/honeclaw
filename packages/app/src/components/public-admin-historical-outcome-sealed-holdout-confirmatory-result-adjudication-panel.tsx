import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudications,
  reviewHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudication,
} from "@/lib/api";
import type {
  HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRegistry,
  HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–72 全链、claim/result/output 与独立验证记录",
  "确认 Stage 72 使用第二实现逐位复算，不是执行者自证",
  "确认只评估一个目标、一个冻结算法和 17/29/43 三个种子",
  "逐项复核三个种子的预登记检验与阈值，定量失败不得人工覆盖",
  "复核样本数、独立分量、component bootstrap 与 Holm 多重检验是否充分",
  "复核目标语义和经济相关性，不把统计通过等同于可赚钱",
  "同时检查效应量，而不是只看 p 值或通过/失败标签",
  "复核数据覆盖、选择偏差、市场状态偏差和主要失败模式",
  "没有把未确认的 Hari/老王逻辑包装成裁决依据",
  "可复现不等于泛化、盈利或操盘能力",
  "批准只开放未来受控影子实验设计登记，不启动影子盘",
  "不正式选模、不写库、不反馈训练/reward，不生成仓位、订单、券商访问或交易",
] as const;

const emptyTexts = () => ({
  statistical_interpretation: "",
  economic_interpretation: "",
  known_limitations: "",
  falsification_conditions: "",
  next_experiment_constraints: "",
});

export function PublicAdminHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationVerdict>(
      "changes_requested",
    );
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [texts, setTexts] = createSignal(emptyTexts());
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudications();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.review_eligible);
      if (!eligible.some((item) => item.candidate.source.attempt.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(eligible[0]?.candidate.source.attempt.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 确认结果裁决表读取失败");
    }
  };

  onMount(() => void load());

  const eligibleItems = createMemo(() => registry()?.items.filter((item) => item.review_eligible) ?? []);
  const selected = createMemo(() => eligibleItems().find((item) => item.candidate.source.attempt.claim.attempt_id === selectedAttemptId()));
  const allTextsPresent = createMemo(() => Object.values(texts()).every((value) => value.trim().length > 0));
  const approving = createMemo(() => verdict() === "approved_for_future_controlled_shadow_experiment_design_registration");
  const disabled = createMemo(() => busy() || !selected() || !allTextsPresent() || checks().some((value) => !value) || (approving() && !selected()?.candidate.quantitative_approval_eligible));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const candidate = item.candidate;
    const source = candidate.source;
    const validation = source.validation;
    const envelope = source.attempt.result.untrusted_confirmation_envelope;
    if (!envelope) {
      setError("当前候选缺少 sealed-holdout 确认信封");
      return;
    }
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await reviewHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudication(
        source.attempt.claim.attempt_id,
        {
          expected_review_id: item.latest_review?.review_id,
          expected_review_sha256: item.latest_review?.review_sha256,
          expected_output_validation_id: validation.validation_id,
          expected_output_validation_sha256: validation.validation_sha256,
          expected_claim_sha256: validation.claim_sha256,
          expected_result_sha256: validation.result_sha256,
          expected_output_sha256: validation.output_sha256,
          expected_envelope_sha256: candidate.envelope_sha256,
          expected_candidate_set_sha256: validation.candidate_set_sha256,
          expected_training_store_dataset_sha256: validation.training_store_dataset_sha256,
          expected_selected_algorithm_three_seed_binding_sha256: validation.selected_algorithm_three_seed_binding_sha256,
          expected_sealed_holdout_split_commitment_sha256: validation.sealed_holdout_split_commitment_sha256,
          expected_sealed_holdout_projection_sha256: validation.sealed_holdout_projection_sha256,
          expected_feature_order_sha256: validation.feature_order_sha256,
          expected_preprocessing_sha256: validation.preprocessing_sha256,
          expected_target_id: envelope.target_id,
          expected_frozen_candidate_algorithm_id: envelope.frozen_candidate_algorithm_id,
          verdict: verdict(),
          ...texts(),
          exact_current_stage_51_through_stage_72_binding_confirmed: checks()[0] as true,
          stage_72_second_implementation_reproducibility_confirmed: checks()[1] as true,
          exact_one_target_one_algorithm_three_frozen_seeds_confirmed: checks()[2] as true,
          all_three_preregistered_seed_tests_and_thresholds_reviewed: checks()[3] as true,
          sample_component_and_multiple_testing_sufficiency_reviewed: checks()[4] as true,
          target_semantics_and_economic_relevance_reviewed: checks()[5] as true,
          effect_size_not_p_value_only_reviewed: checks()[6] as true,
          data_coverage_selection_bias_and_failure_modes_reviewed: checks()[7] as true,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[8] as true,
          reproducibility_not_profitability_or_generalization_confirmed: checks()[9] as true,
          approval_only_opens_future_controlled_shadow_experiment_design_registration_confirmed: checks()[10] as true,
          no_selection_store_training_reward_shadow_order_broker_or_trading_confirmed: checks()[11] as true,
        },
      );
      setRegistry(next); setChecks(REVIEW_CHECKS.map(() => false)); setTexts(emptyTexts());
      setNotice("已写入不可变裁决记录。即使通过，也只开放下一阶段受控影子实验设计登记。模型与交易权限仍关闭。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 确认结果裁决失败");
      await load();
    } finally { setBusy(false); }
  };

  return (
    <Show when={registry()}>
      {(current) => (
        <section class="public-admin-reward-governance" aria-label="sealed-holdout 确认结果独立裁决">
          <header><strong>第 73 阶段 · 确认结果独立裁决</strong><span>{current().adjudication_status}</span></header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>候选</span><strong>{current().candidate_count}</strong></div>
            <div><span>定量通过</span><strong>{current().quantitative_pass_count}</strong></div>
            <div><span>失败/不足</span><strong>{current().quantitative_fail_or_insufficient_count}</strong></div>
            <div><span>裁决通过</span><strong>{current().approved_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>可复现 ≠ 有经济意义</strong><span>定量硬门槛 + 独立人工解释</span></header>
            <p>Stage 72 只回答“第二实现能否得到同一结果”。本层必须再回答样本是否充分、效应是否有意义、目标是否对应真实投资问题，以及什么条件会证伪。</p>
            <p class="public-admin-anchor-boundary">人工不能覆盖定量失败；裁决通过也不是正式选模、盈利证明或影子盘启动。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有待裁决的 Stage 72 独立验证结果。</p>}>
            <label><span>待裁决 attempt</span><select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}><For each={eligibleItems()}>{(item) => <option value={item.candidate.source.attempt.claim.attempt_id}>{item.candidate.source.attempt.claim.attempt_id.slice(0, 12)}… · {item.candidate.confirmation_status}</option>}</For></select></label>
            <Show when={selected()}>{(item) => <article class="public-admin-reward-governance"><header><strong>{item().candidate.source.attempt.claim.target_id}</strong><span>{item().candidate.quantitative_approval_eligible ? "定量可进入裁决" : "定量失败关闭"}</span></header><p>样本 {item().candidate.sealed_holdout_row_count} · 独立分量 {item().candidate.independent_component_count} · 指标 {item().candidate.metric_count}</p><Show when={item().candidate.quantitative_ineligibility_reasons.length > 0}><p class="public-admin-error">{item().candidate.quantitative_ineligibility_reasons.join("；")}</p></Show></article>}</Show>
            <label><span>裁决</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationVerdict)}><option value="approved_for_future_controlled_shadow_experiment_design_registration" disabled={!selected()?.candidate.quantitative_approval_eligible}>通过，仅开放影子实验设计登记</option><option value="changes_requested">要求补充/修正</option><option value="rejected">拒绝</option></select></label>
            <label><span>统计解释</span><textarea value={texts().statistical_interpretation} onInput={(event) => setTexts((value) => ({ ...value, statistical_interpretation: event.currentTarget.value }))} /></label>
            <label><span>经济解释</span><textarea value={texts().economic_interpretation} onInput={(event) => setTexts((value) => ({ ...value, economic_interpretation: event.currentTarget.value }))} /></label>
            <label><span>已知局限与偏差</span><textarea value={texts().known_limitations} onInput={(event) => setTexts((value) => ({ ...value, known_limitations: event.currentTarget.value }))} /></label>
            <label><span>证伪条件</span><textarea value={texts().falsification_conditions} onInput={(event) => setTexts((value) => ({ ...value, falsification_conditions: event.currentTarget.value }))} /></label>
            <label><span>下一实验约束</span><textarea value={texts().next_experiment_constraints} onInput={(event) => setTexts((value) => ({ ...value, next_experiment_constraints: event.currentTarget.value }))} /></label>
            <div class="public-admin-decision-checks"><For each={REVIEW_CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在写入不可变裁决…" : "提交独立结果裁决"}</button>
          </Show>
          <Show when={error()}><p class="public-admin-error">{error()}</p></Show><Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={current().items}>{(item) => <Show when={item.latest_review}>{(review) => <article class="public-admin-reward-governance"><header><strong>裁决 {review().review_id.slice(0, 12)}…</strong><span>{review().verdict}</span></header><p>{review().submitted_at} · {review().reviewer_id}</p><p><strong>统计：</strong>{review().statistical_interpretation}</p><p><strong>经济：</strong>{review().economic_interpretation}</p><p><strong>证伪：</strong>{review().falsification_conditions}</p><p class="public-admin-anchor-boundary">通过只开放实验设计登记；模型、reward、影子账本和交易权限仍关闭。</p></article>}</Show>}</For>
        </section>
      )}
    </Show>
  );
}
