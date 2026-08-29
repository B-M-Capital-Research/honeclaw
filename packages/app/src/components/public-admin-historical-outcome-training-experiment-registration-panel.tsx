import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeTrainingExperimentRegistrations,
  registerHistoricalOutcomeTrainingExperimentSuiteOnce,
} from "@/lib/api";
import type {
  HistoricalOutcomeTrainingExperimentRegistrationRegistry,
  RegisterHistoricalOutcomeTrainingExperimentSuiteRequest,
} from "@/lib/types";

const CHECKS = [
  ["exact_current_stage_50_admission_and_complete_chain_confirmed", "精确绑定当前 Stage 50 准入复核与完整责任链"],
  ["claim_first_create_once_and_failure_consumes_confirmed", "先保存不可变 claim；成功、失败或中断都永久消费资格"],
  ["fixed_three_arm_three_seed_suite_confirmed", "固定零预测基线、岭回归、梯度提升三模型臂和 17/29/43 三个种子"],
  ["train_fit_validation_selection_and_sealed_holdout_isolation_confirmed", "train 只用于拟合、validation 只用于选择，sealed holdout 对训练器完全隐藏"],
  ["exact_65_feature_nine_raw_target_contract_confirmed", "保持 65 项点时特征与九项原始连续结果的精确合同"],
  ["no_scalar_reward_action_position_or_ranking_semantics_confirmed", "不把九项目标压成奖励，不引入买卖动作、仓位或排名标签"],
  ["independent_registration_review_required_before_training_authorization_confirmed", "登记后必须由另一角色独立复核，才能考虑 runner 与训练授权"],
  ["no_training_run_reward_shadow_order_broker_or_trading_confirmed", "本阶段不训练、不奖励、不影子、不下单、不接券商、不交易"],
] as const;

type CheckName = (typeof CHECKS)[number][0];

export function PublicAdminHistoricalOutcomeTrainingExperimentRegistrationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeTrainingExperimentRegistrationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [experimentName, setExperimentName] = createSignal("九目标三模型首轮对照");
  const [researchHypothesis, setResearchHypothesis] = createSignal("");
  const [knownLimitations, setKnownLimitations] = createSignal("");
  const [checked, setChecked] = createSignal<Record<CheckName, boolean>>(
    Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
  );
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeTrainingExperimentRegistrations();
      setRegistry(next);
      if (!next.items.some((item) => item.admitted_dataset.admission_review.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(
          next.items.find((item) => item.registration_eligible)?.admitted_dataset.admission_review.attempt_id
            ?? next.items[0]?.admitted_dataset.admission_review.attempt_id
            ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练实验登记注册表读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() => registry()?.items.find(
    (item) => item.admitted_dataset.admission_review.attempt_id === selectedAttemptId(),
  ));
  const allConfirmed = createMemo(() => CHECKS.every(([name]) => checked()[name]));

  const submit = async () => {
    const item = selected();
    if (!item || !item.registration_eligible || !allConfirmed() || busy()) return;
    if (!experimentName().trim() || !researchHypothesis().trim() || !knownLimitations().trim()) {
      setError("请填写实验名称、可证伪研究假设和已知局限。");
      return;
    }
    const review = item.admitted_dataset.admission_review;
    const request: RegisterHistoricalOutcomeTrainingExperimentSuiteRequest = {
      expected_admission_review_id: review.review_id,
      expected_admission_review_sha256: review.review_sha256,
      expected_copy_output_validation_id: review.copy_output_validation_id,
      expected_copy_output_validation_sha256: review.copy_output_validation_sha256,
      expected_copy_id: review.copy_id,
      expected_training_store_dataset_sha256: review.training_store_dataset_sha256,
      expected_recomputed_rows_sha256: item.admitted_dataset.dataset.validation.recomputed_rows_sha256,
      expected_recomputed_excluded_rows_sha256:
        item.admitted_dataset.dataset.validation.recomputed_excluded_rows_sha256,
      expected_recomputed_target_commitments_sha256:
        item.admitted_dataset.dataset.validation.recomputed_target_commitments_sha256,
      experiment_name: experimentName().trim(),
      research_hypothesis: researchHypothesis().trim(),
      known_limitations: knownLimitations().trim(),
      ...checked(),
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await registerHistoricalOutcomeTrainingExperimentSuiteOnce(
        review.attempt_id,
        request,
      );
      setRegistry(next);
      setChecked(
        Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
      );
      setNotice("训练实验套件已不可变登记为 registered_not_run；没有启动训练或打开 sealed holdout。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练实验登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="训练实验一次性登记">
          <header>
            <strong>第 51 阶段 · 训练实验一次性登记</strong>
            <span>{currentRegistry().registration_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>Stage 50 已准入</span><strong>{currentRegistry().admitted_candidate_count}</strong></div>
            <div><span>可登记</span><strong>{currentRegistry().registration_eligible_count}</strong></div>
            <div><span>claim</span><strong>{currentRegistry().claim_count}</strong></div>
            <div><span>登记待独立复核</span><strong>{currentRegistry().pending_independent_registration_review_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>登记 ≠ 训练运行</strong><span>claim first · create once</span></header>
            <p>固定对照套件：零预测基线、岭回归、梯度提升；每个模型臂使用 17 / 29 / 43 三个确定性种子。</p>
            <p>只预测 20 / 60 / 250 日的资产收益、超额收益和最大回撤九项连续结果；逐目标、逐种子报告，不生成单一奖励。</p>
            <p class="public-admin-anchor-boundary">状态只能是 registered_not_run；sealed holdout、训练运行、奖励、影子、订单、券商和交易全部关闭。</p>
          </article>

          <Show when={currentRegistry().items.length > 0} fallback={<p>当前没有 Stage 50 已准入的训练存储副本。</p>}>
            <label>
              <span>已准入副本</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>{(item) => (
                  <option value={item.admitted_dataset.admission_review.attempt_id}>
                    {item.admitted_dataset.admission_review.attempt_id.slice(0, 12)}… · {item.attempt?.result?.status ?? "待登记"}
                  </option>
                )}</For>
              </select>
            </label>
            <label><span>实验名称</span><input value={experimentName()} onInput={(event) => setExperimentName(event.currentTarget.value)} /></label>
            <label><span>可证伪研究假设</span><textarea value={researchHypothesis()} onInput={(event) => setResearchHypothesis(event.currentTarget.value)} /></label>
            <label><span>已知局限</span><textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} /></label>
            <For each={CHECKS}>{([name, label]) => (
              <label class="public-admin-anchor-check">
                <input type="checkbox" checked={checked()[name]} onChange={(event) => setChecked({ ...checked(), [name]: event.currentTarget.checked })} />
                <span>{label}</span>
              </label>
            )}</For>
            <button type="button" disabled={busy() || !selected()?.registration_eligible || !allConfirmed()} onClick={() => void submit()}>
              {busy() ? "正在保存 claim 与登记…" : "claim 并一次性登记实验套件"}
            </button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={currentRegistry().items}>{(item) => (
            <article class="public-admin-reward-governance">
              <header>
                <strong>{item.attempt?.registration?.experiment_name ?? `dataset ${item.admitted_dataset.admission_review.attempt_id.slice(0, 12)}…`}</strong>
                <span>{item.attempt?.registration?.status ?? item.attempt?.result?.status ?? "waiting_registration"}</span>
              </header>
              <p>Stage 50 复核人 {item.admitted_dataset.admission_review.reviewer_id} · 登记者 {item.attempt?.claim.registered_by ?? "—"}</p>
              <Show when={item.attempt?.registration}>{(registration) => (
                <>
                  <p>模型臂 {registration().suite_specification.arms.length} · seeds 17/29/43 · features {registration().suite_specification.feature_catalog_count} · targets {registration().suite_specification.target_count}</p>
                  <p>研究假设：{registration().research_hypothesis}</p>
                  <p>已知局限：{registration().known_limitations}</p>
                </>
              )}</Show>
              <Show when={item.attempt?.result?.error}><p class="public-admin-decision-error">{item.attempt?.result?.error}</p></Show>
              <p class="public-admin-anchor-boundary">登记成功也必须独立复核；没有 runner、训练授权、训练运行或交易权限。</p>
            </article>
          )}</For>
        </section>
      )}
    </Show>
  );
}
