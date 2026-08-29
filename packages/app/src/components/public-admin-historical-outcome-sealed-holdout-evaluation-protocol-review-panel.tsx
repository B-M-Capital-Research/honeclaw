import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeSealedHoldoutEvaluationProtocolReviews,
  reviewHistoricalOutcomeSealedHoldoutEvaluationProtocol,
} from "@/lib/api";
import type {
  HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewRegistry,
  HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–65 全链、当前逐目标准入记录与候选证据",
  "复核人独立于 Stage 65 复核人及完整上游复核链",
  "每个目标只冻结一种算法与 17、29、43 三个种子",
  "候选、65 个特征顺序、预处理和单一目标承诺均不可变",
  "sealed holdout 只允许未来一次性评估，结果不得反馈复用",
  "指标、门槛、按独立组件 bootstrap 与三项假设 Holm 校正均已冻结",
  "三个种子必须全部通过，任何失败都必须保留并使该目标失败",
  "少于 100 行或 20 个独立组件时必须证据不足并关闭失败",
  "不得跨目标综合、调参、重新拟合、重选候选或改变阈值",
  "本次协议复核不读取、挂载、投影或执行 sealed holdout",
  "下一道门只是评估实现登记，不是数据访问或一次性执行授权",
  "不正式选模、不写模型或指标库，不产生 reward、影子、订单、券商访问或交易",
] as const;

export function PublicAdminHistoricalOutcomeSealedHoldoutEvaluationProtocolReviewPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewRegistry>();
  const [selectedKey, setSelectedKey] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict>(
      "changes_requested",
    );
  const [rationale, setRationale] = createSignal("");
  const [limitations, setLimitations] = createSignal("");
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const keyFor = (attemptId: string, targetId: string) => `${attemptId}::${targetId}`;

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeSealedHoldoutEvaluationProtocolReviews();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.review_eligible);
      if (
        !eligible.some((item) =>
          keyFor(item.subject.protocol.attempt_id, item.subject.protocol.target_id) ===
          selectedKey(),
        )
      ) {
        const first = eligible[0];
        setSelectedKey(
          first ? keyFor(first.subject.protocol.attempt_id, first.subject.protocol.target_id) : "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 评估协议复核表读取失败");
    }
  };

  onMount(() => void load());

  const eligibleItems = createMemo(
    () => registry()?.items.filter((item) => item.review_eligible) ?? [],
  );
  const selected = createMemo(() =>
    eligibleItems().find(
      (item) =>
        keyFor(item.subject.protocol.attempt_id, item.subject.protocol.target_id) === selectedKey(),
    ),
  );
  const approving = createMemo(
    () =>
      verdict() ===
      "approved_for_future_sealed_holdout_evaluation_implementation_registration",
  );
  const disabled = createMemo(
    () =>
      busy() ||
      !selected() ||
      !rationale().trim() ||
      !limitations().trim() ||
      checks().some((value) => !value),
  );

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) =>
      current.map((value, currentIndex) => (currentIndex === index ? checked : value)),
    );
  };

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const protocol = item.subject.protocol;
    const admissionReview = item.subject.admitted.admission_review;
    const confirmed = checks();
    const approved = approving();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewHistoricalOutcomeSealedHoldoutEvaluationProtocol(
        protocol.attempt_id,
        protocol.target_id,
        {
          expected_review_id: item.latest_review?.review_id,
          expected_review_sha256: item.latest_review?.review_sha256,
          expected_stage_65_admission_review_id: admissionReview.review_id,
          expected_stage_65_admission_review_sha256: admissionReview.review_sha256,
          expected_output_validation_sha256: protocol.output_validation_sha256,
          expected_candidate_set_sha256: protocol.candidate_set_sha256,
          expected_training_store_dataset_sha256: protocol.training_store_dataset_sha256,
          expected_target_bundle_sha256: protocol.target_bundle_sha256,
          expected_recommendation_sha256: protocol.recommendation_sha256,
          expected_protocol_sha256: protocol.protocol_sha256,
          verdict: verdict(),
          rationale: rationale().trim(),
          known_limitations: limitations().trim(),
          exact_current_stage_51_through_stage_65_binding_confirmed: confirmed[0] as true,
          reviewer_independent_from_stage_65_and_complete_prior_chain_confirmed:
            confirmed[1] as true,
          one_target_one_algorithm_three_frozen_seeds_only_confirmed: confirmed[2] as true,
          immutable_candidate_feature_preprocessing_and_target_confirmed: confirmed[3] as true,
          sealed_holdout_single_use_and_no_feedback_reuse_confirmed: confirmed[4] as true,
          fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed:
            confirmed[5] as true,
          all_three_seeds_must_pass_and_failures_remain_visible_confirmed: confirmed[6] as true,
          insufficient_sample_fails_closed_confirmed: confirmed[7] as true,
          no_cross_target_composite_tuning_refit_or_reselection_confirmed: confirmed[8] as true,
          protocol_review_does_not_read_mount_project_or_execute_holdout_confirmed:
            confirmed[9] as true,
          next_gate_is_implementation_registration_not_data_access_confirmed:
            confirmed[10] as true,
          no_selection_store_reward_shadow_order_broker_or_trading_confirmed:
            confirmed[11] as true,
        },
      );
      setRegistry(next);
      setRationale("");
      setLimitations("");
      setChecks(REVIEW_CHECKS.map(() => false));
      setVerdict("changes_requested");
      setNotice(
        approved
          ? "协议已独立批准，但只开放未来评估实现登记；sealed holdout 仍不可访问或执行。"
          : "协议复核结论已追加保存；未批准的目标不会进入下一道门。",
      );
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 评估协议复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="封存样本评估协议独立复核">
          <header>
            <strong>第 66 阶段 · 封存样本评估协议独立复核</strong>
            <span>{currentRegistry().protocol_review_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>已准入目标</span><strong>{currentRegistry().admitted_target_count}</strong></div>
            <div><span>已复核协议</span><strong>{currentRegistry().protocol_reviewed_count}</strong></div>
            <div><span>独立批准</span><strong>{currentRegistry().protocol_independently_approved_count}</strong></div>
            <div><span>待修正/拒绝</span><strong>{currentRegistry().protocol_rejected_or_changes_requested_count}</strong></div>
            <div><span>可登记实现</span><strong>{currentRegistry().future_sealed_holdout_evaluation_implementation_registration_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>只冻结尺子，不打开试卷</strong><span>one-shot</span></header>
            <p>每个目标固定一种已准入算法、三个种子、三项确认性假设、10,000 次独立组件 bootstrap 与 Holm 校正；三个种子必须全部通过。</p>
            <p class="public-admin-anchor-boundary">本阶段不读取、挂载、解密、投影或运行 sealed holdout；批准也只允许进入未来评估实现登记。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有待复核的 sealed-holdout 评估协议。</p>}>
            <label>
              <span>待复核协议</span>
              <select value={selectedKey()} onChange={(event) => setSelectedKey(event.currentTarget.value)}>
                <For each={eligibleItems()}>
                  {(item) => (
                    <option value={keyFor(item.subject.protocol.attempt_id, item.subject.protocol.target_id)}>
                      {item.subject.protocol.target_id} · {item.subject.protocol.frozen_candidate_algorithm_id}
                    </option>
                  )}
                </For>
              </select>
            </label>
            <Show when={selected()}>
              {(item) => {
                const protocol = () => item().subject.protocol;
                return (
                  <article class="public-admin-reward-governance">
                    <header><strong>{protocol().target_id}</strong><span>{protocol().protocol_version}</span></header>
                    <p>算法：{protocol().frozen_candidate_algorithm_id} · 种子：{protocol().exact_random_seeds.join(" / ")} · 特征/目标：{protocol().exact_feature_count}/{protocol().exact_target_count}</p>
                    <p>确认性假设 {protocol().exact_candidate_hypothesis_count} 项 · bootstrap {protocol().bootstrap_replications.toLocaleString()} 次 · 最少 {protocol().minimum_sealed_holdout_rows} 行、{protocol().minimum_independent_components} 个独立组件</p>
                    <p>相对 MAE 至少改善 {protocol().minimum_relative_mae_improvement_ppm / 10_000}% · Spearman &gt; 0 · 方向准确率 ≥ {protocol().minimum_directional_accuracy_millionths / 10_000}% · 校准斜率 {protocol().minimum_calibration_slope_millionths / 1_000_000}–{protocol().maximum_calibration_slope_millionths / 1_000_000}</p>
                  </article>
                );
              }}
            </Show>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict)}>
                <option value="changes_requested">要求修正协议</option>
                <option value="rejected">拒绝协议</option>
                <option value="approved_for_future_sealed_holdout_evaluation_implementation_registration">批准未来评估实现登记</option>
              </select>
            </label>
            <label><span>复核依据</span><textarea maxlength={2400} value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限与偏差</span><textarea maxlength={2400} value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
            <div class="public-admin-decision-checks">
              <For each={REVIEW_CHECKS}>
                {(label, index) => (
                  <label>
                    <input type="checkbox" checked={checks()[index()]} onChange={(event) => toggleCheck(index(), event.currentTarget.checked)} />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在保存协议审计…" : "提交协议独立复核"}</button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={currentRegistry().items}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header><strong>{item.subject.protocol.target_id}</strong><span>{item.latest_review?.verdict ?? "待独立复核"}</span></header>
                <p>协议 {item.subject.protocol.protocol_sha256.slice(0, 12)}… · Stage 65 准入 {item.subject.admitted.admission_review.review_sha256.slice(0, 12)}…</p>
                <p class="public-admin-anchor-boundary">{item.protocol_independently_approved ? "仅获未来评估实现登记资格" : "未获资格；sealed holdout 始终保持关闭"}</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
