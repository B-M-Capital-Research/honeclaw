import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeTrainingExperimentRegistrationReviews,
  reviewHistoricalOutcomeTrainingExperimentRegistration,
} from "@/lib/api";
import type {
  HistoricalOutcomeTrainingExperimentRegistrationReviewRegistry,
  ReviewHistoricalOutcomeTrainingExperimentRegistrationRequest,
} from "@/lib/types";

const CHECKS = [
  ["exact_current_stage_51_registration_and_complete_chain_confirmed", "精确绑定当前 Stage 51 登记及完整上游责任链"],
  ["immutable_claim_registration_result_and_suite_hashes_confirmed", "独立重算 claim、registration、result 与实验规范哈希"],
  ["claim_first_create_once_success_and_registered_not_run_confirmed", "确认 claim-first、create-once、成功结果与 registered_not_run 状态"],
  ["registrar_and_reviewer_independence_confirmed", "确认登记人独立于上游，且本复核人独立于登记人与完整上游"],
  ["fixed_three_arm_three_seed_suite_confirmed", "确认固定零预测、岭回归、梯度提升三模型臂及 17/29/43 三种子"],
  ["exact_65_feature_nine_raw_continuous_target_contract_confirmed", "确认 65 项特征与九项原始连续结果合同未漂移"],
  ["train_fit_validation_selection_and_sealed_holdout_isolation_confirmed", "确认 train 拟合、validation 选择、sealed holdout 完全隐藏"],
  ["per_target_per_seed_metrics_without_composite_masking_confirmed", "确认逐目标逐种子报告，综合分不能掩盖单项目标失败"],
  ["fixed_resource_ceilings_and_deterministic_replay_confirmed", "确认固定资源上限与确定性重放要求"],
  ["no_scalar_reward_action_position_or_ranking_semantics_confirmed", "确认未定义标量奖励、买卖动作、仓位或排名语义"],
  ["implementation_registration_runner_and_run_authorization_remain_separate_confirmed", "确认训练实现登记、runner 登记和运行授权仍是三个后续独立门禁"],
  ["no_training_run_reward_shadow_order_broker_or_trading_confirmed", "确认本阶段不训练、不奖励、不影子、不下单、不接券商、不交易"],
] as const;

type CheckName = (typeof CHECKS)[number][0];
type Verdict = ReviewHistoricalOutcomeTrainingExperimentRegistrationRequest["verdict"];

export function PublicAdminHistoricalOutcomeTrainingExperimentRegistrationReviewPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeTrainingExperimentRegistrationReviewRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [verdict, setVerdict] = createSignal<Verdict>("changes_requested");
  const [rationale, setRationale] = createSignal("");
  const [knownLimitations, setKnownLimitations] = createSignal("");
  const [checked, setChecked] = createSignal<Record<CheckName, boolean>>(
    Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
  );
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeTrainingExperimentRegistrationReviews();
      setRegistry(next);
      if (!next.items.some((item) => item.registered_experiment.attempt.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(
          next.items.find((item) => item.review_eligible)?.registered_experiment.attempt.claim.attempt_id
            ?? next.items[0]?.registered_experiment.attempt.claim.attempt_id
            ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练实验登记独立复核注册表读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() => registry()?.items.find(
    (item) => item.registered_experiment.attempt.claim.attempt_id === selectedAttemptId(),
  ));
  const allConfirmed = createMemo(() => CHECKS.every(([name]) => checked()[name]));

  const submit = async () => {
    const item = selected();
    const registration = item?.registered_experiment.attempt.registration;
    const result = item?.registered_experiment.attempt.result;
    const claim = item?.registered_experiment.attempt.claim;
    if (!item || !registration || !result || !claim || !item.review_eligible || busy()) return;
    if (!rationale().trim() || !knownLimitations().trim()) {
      setError("请填写复核依据与已知局限。批准时还必须逐项确认全部十二项门禁。");
      return;
    }
    if (verdict() === "approved_for_future_training_implementation_registration" && !allConfirmed()) {
      setError("独立批准前必须逐项确认全部十二项门禁。");
      return;
    }
    const previous = item.latest_review;
    const admission = item.registered_experiment.admitted_dataset.admission_review;
    const request: ReviewHistoricalOutcomeTrainingExperimentRegistrationRequest = {
      expected_review_id: previous?.review_id,
      expected_review_sha256: previous?.review_sha256,
      expected_registration_id: registration.registration_id,
      expected_registration_sha256: registration.registration_sha256,
      expected_claim_sha256: claim.claim_sha256,
      expected_result_id: result.result_id,
      expected_result_sha256: result.result_sha256,
      expected_admission_review_id: admission.review_id,
      expected_admission_review_sha256: admission.review_sha256,
      expected_training_store_dataset_sha256: registration.training_store_dataset_sha256,
      expected_rows_sha256: registration.rows_sha256,
      expected_excluded_rows_sha256: registration.excluded_rows_sha256,
      expected_target_commitments_sha256: registration.target_commitments_sha256,
      expected_suite_specification_sha256: registration.suite_specification.specification_sha256,
      verdict: verdict(),
      rationale: rationale().trim(),
      known_limitations: knownLimitations().trim(),
      ...checked(),
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewHistoricalOutcomeTrainingExperimentRegistration(
        claim.attempt_id,
        request,
      );
      setRegistry(next);
      setChecked(
        Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
      );
      setNotice(verdict() === "approved_for_future_training_implementation_registration"
        ? "登记已独立批准；只开放未来训练实现登记，没有创建 runner 或启动训练。"
        : "复核结论已追加到不可变链；训练实现登记仍关闭。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练实验登记独立复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="训练实验登记独立复核">
          <header>
            <strong>第 52 阶段 · 训练实验登记独立复核</strong>
            <span>{currentRegistry().review_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待独立复核</span><strong>{currentRegistry().review_eligible_count}</strong></div>
            <div><span>已复核</span><strong>{currentRegistry().reviewed_count}</strong></div>
            <div><span>独立批准</span><strong>{currentRegistry().independently_approved_count}</strong></div>
            <div><span>可进入实现登记</span><strong>{currentRegistry().future_training_implementation_registration_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>登记复核 ≠ 训练授权</strong><span>independent · append only</span></header>
            <p>本复核会重新核对三模型臂、三种子、65 项特征、九项原始连续目标、逐目标逐种子指标与封存集隔离。</p>
            <p class="public-admin-anchor-boundary">批准仍不创建 runner、不授权或启动训练；下一步只能另行登记训练实现。</p>
          </article>

          <Show when={currentRegistry().items.length > 0} fallback={<p>当前没有已完成且等待独立复核的 Stage 51 登记。</p>}>
            <label>
              <span>训练实验登记</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>{(item) => (
                  <option value={item.registered_experiment.attempt.claim.attempt_id}>
                    {item.registered_experiment.attempt.registration?.experiment_name ?? item.registered_experiment.attempt.claim.attempt_id.slice(0, 12)} · {item.latest_review?.verdict ?? "待复核"}
                  </option>
                )}</For>
              </select>
            </label>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as Verdict)}>
                <option value="changes_requested">退回修改</option>
                <option value="rejected">拒绝</option>
                <option value="approved_for_future_training_implementation_registration">独立批准，仅开放训练实现登记</option>
              </select>
            </label>
            <label><span>复核依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限</span><textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} /></label>
            <For each={CHECKS}>{([name, label]) => (
              <label class="public-admin-anchor-check">
                <input type="checkbox" checked={checked()[name]} onChange={(event) => setChecked({ ...checked(), [name]: event.currentTarget.checked })} />
                <span>{label}</span>
              </label>
            )}</For>
            <button type="button" disabled={busy() || !selected()?.review_eligible || (verdict() === "approved_for_future_training_implementation_registration" && !allConfirmed())} onClick={() => void submit()}>
              {busy() ? "正在追加独立复核…" : "追加独立复核结论"}
            </button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={currentRegistry().items}>{(item) => {
            const registration = item.registered_experiment.attempt.registration;
            const review = item.latest_review;
            return (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{registration?.experiment_name ?? "训练实验登记"}</strong>
                  <span>{review?.verdict ?? "waiting_independent_review"}</span>
                </header>
                <p>登记者 {item.registered_experiment.attempt.claim.registered_by} · 独立复核人 {review?.reviewer_id ?? "—"}</p>
                <p>模型臂 {registration?.suite_specification.arms.length ?? 0} · seeds 17/29/43 · features {registration?.suite_specification.feature_catalog_count ?? 0} · targets {registration?.suite_specification.target_count ?? 0}</p>
                <Show when={review}><p>复核依据：{review?.rationale}</p></Show>
                <p class="public-admin-anchor-boundary">{review?.future_training_implementation_registration_eligible ? "只可进入未来训练实现登记" : "训练实现登记关闭"}；runner、训练、奖励、影子、订单、券商和交易全部关闭。</p>
              </article>
            );
          }}</For>
        </section>
      )}
    </Show>
  );
}
