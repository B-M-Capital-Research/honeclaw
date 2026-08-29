import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeSealedHoldoutEvaluationImplementations,
  registerHistoricalOutcomeSealedHoldoutEvaluationImplementation,
} from "@/lib/api";
import type {
  HistoricalOutcomeSealedHoldoutEvaluationImplementationRegistry,
  RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest,
} from "@/lib/types";

const CHECKS = [
  ["exact_current_stage_51_through_stage_66_binding_confirmed", "精确绑定当前 Stage 51–66 完整链、协议复核和候选证据"],
  ["registrar_independent_from_stage_66_and_complete_prior_chain_confirmed", "登记人独立于 Stage 66 及完整上游角色"],
  ["immutable_artifact_revision_protocol_and_serialization_confirmed", "实现工件、代码版本、协议与序列化格式均不可变"],
  ["one_target_one_algorithm_three_frozen_seeds_only_confirmed", "单一目标只允许一种已批准算法与 17、29、43 三个冻结种子"],
  ["no_callable_entrypoint_mount_data_adapter_or_holdout_access_confirmed", "没有调用入口、输入挂载、数据适配器或 sealed-holdout 访问"],
  ["one_shot_no_feedback_reuse_and_fail_closed_sample_rules_preserved_confirmed", "保留一次性、无反馈复用与样本不足失败关闭规则"],
  ["fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed", "固定指标、门槛、组件 bootstrap 与三项假设 Holm 校正"],
  ["no_tuning_refit_reselection_or_cross_target_composite_confirmed", "不得调参、重拟合、重选候选或跨目标综合掩盖失败"],
  ["future_output_create_once_untrusted_and_independent_validation_required_confirmed", "未来输出只能 create-once，先视为不可信并强制独立验证"],
  ["independent_review_runner_and_one_shot_authorization_remain_separate_confirmed", "独立实现复核、runner 与一次性授权保持分门"],
  ["no_selection_store_reward_shadow_order_broker_or_trading_confirmed", "不正式选模，不写模型/指标库，不产生奖励、影子、订单、券商或交易"],
] as const;

type CheckName = (typeof CHECKS)[number][0];

export function PublicAdminHistoricalOutcomeSealedHoldoutEvaluationImplementationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeSealedHoldoutEvaluationImplementationRegistry>();
  const [selectedReviewId, setSelectedReviewId] = createSignal("");
  const [implementationName, setImplementationName] = createSignal("");
  const [immutableCodeRevision, setImmutableCodeRevision] = createSignal("");
  const [artifactSha256, setArtifactSha256] = createSignal("");
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
      const next = await getHistoricalOutcomeSealedHoldoutEvaluationImplementations();
      setRegistry(next);
      if (!next.eligible_protocols.some(
        (item) => item.protocol_review.review_id === selectedReviewId(),
      )) {
        setSelectedReviewId(next.eligible_protocols[0]?.protocol_review.review_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 评估实现登记表读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() => registry()?.eligible_protocols.find(
    (item) => item.protocol_review.review_id === selectedReviewId(),
  ));
  const allConfirmed = createMemo(() => CHECKS.every(([name]) => checked()[name]));

  const submit = async () => {
    const source = selected();
    if (!source || busy()) return;
    if (!implementationName().trim() || !immutableCodeRevision().trim()
      || !rationale().trim() || !knownLimitations().trim()) {
      setError("请填写实现名称、不可变代码版本、登记依据和已知局限。");
      return;
    }
    if (!/^[a-f0-9]{64}$/i.test(artifactSha256().trim())) {
      setError("实现工件必须填写 64 位 SHA-256。");
      return;
    }
    if (!allConfirmed()) {
      setError("登记前必须逐项确认全部十一项边界。");
      return;
    }
    const protocol = source.protocol;
    const review = source.protocol_review;
    const request: RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest = {
      expected_protocol_review_id: review.review_id,
      expected_protocol_review_sha256: review.review_sha256,
      expected_protocol_sha256: protocol.protocol_sha256,
      expected_stage_65_admission_review_sha256: protocol.stage_65_admission_review_sha256,
      expected_output_validation_sha256: protocol.output_validation_sha256,
      expected_candidate_set_sha256: protocol.candidate_set_sha256,
      expected_training_store_dataset_sha256: protocol.training_store_dataset_sha256,
      expected_target_bundle_sha256: protocol.target_bundle_sha256,
      expected_recommendation_sha256: protocol.recommendation_sha256,
      expected_selected_algorithm_three_seed_binding_sha256:
        protocol.selected_algorithm_three_seed_binding_sha256,
      expected_sealed_holdout_split_commitment_sha256:
        protocol.sealed_holdout_split_commitment_sha256,
      implementation_name: implementationName().trim(),
      immutable_code_revision: immutableCodeRevision().trim(),
      implementation_artifact_sha256: artifactSha256().trim().toLowerCase(),
      rationale: rationale().trim(),
      known_limitations: knownLimitations().trim(),
      exact_current_stage_51_through_stage_66_binding_confirmed:
        checked().exact_current_stage_51_through_stage_66_binding_confirmed as true,
      registrar_independent_from_stage_66_and_complete_prior_chain_confirmed:
        checked().registrar_independent_from_stage_66_and_complete_prior_chain_confirmed as true,
      immutable_artifact_revision_protocol_and_serialization_confirmed:
        checked().immutable_artifact_revision_protocol_and_serialization_confirmed as true,
      one_target_one_algorithm_three_frozen_seeds_only_confirmed:
        checked().one_target_one_algorithm_three_frozen_seeds_only_confirmed as true,
      no_callable_entrypoint_mount_data_adapter_or_holdout_access_confirmed:
        checked().no_callable_entrypoint_mount_data_adapter_or_holdout_access_confirmed as true,
      one_shot_no_feedback_reuse_and_fail_closed_sample_rules_preserved_confirmed:
        checked().one_shot_no_feedback_reuse_and_fail_closed_sample_rules_preserved_confirmed as true,
      fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed:
        checked().fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed as true,
      no_tuning_refit_reselection_or_cross_target_composite_confirmed:
        checked().no_tuning_refit_reselection_or_cross_target_composite_confirmed as true,
      future_output_create_once_untrusted_and_independent_validation_required_confirmed:
        checked().future_output_create_once_untrusted_and_independent_validation_required_confirmed as true,
      independent_review_runner_and_one_shot_authorization_remain_separate_confirmed:
        checked().independent_review_runner_and_one_shot_authorization_remain_separate_confirmed as true,
      no_selection_store_reward_shadow_order_broker_or_trading_confirmed:
        checked().no_selection_store_reward_shadow_order_broker_or_trading_confirmed as true,
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setRegistry(await registerHistoricalOutcomeSealedHoldoutEvaluationImplementation(request));
      setChecked(
        Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
      );
      setNotice("零能力实现已登记为 registered_not_reviewed_not_run；登记不是执行，也没有接触 sealed holdout。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 评估实现登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="sealed-holdout 评估实现登记">
          <header>
            <strong>第 67 阶段 · sealed-holdout 评估实现登记</strong>
            <span>{currentRegistry().implementation_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可登记</span><strong>{currentRegistry().registration_eligible_count}</strong></div>
            <div><span>实现记录</span><strong>{currentRegistry().implementation_count}</strong></div>
            <div><span>当前绑定</span><strong>{currentRegistry().current_binding_implementation_count}</strong></div>
            <div><span>待独立复核</span><strong>{currentRegistry().independent_implementation_review_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>登记不是执行</strong><span>zero capability · no entrypoint</span></header>
            <p>只冻结 65 个特征、1 个目标、1 种已批准算法、3 个种子，以及指标、门槛、bootstrap、Holm 校正与序列化协议。</p>
            <p class="public-admin-anchor-boundary">没有入口、挂载、adapter 或留出集访问；未来输出也必须 create-once、先视为不可信，再由独立角色验证。</p>
          </article>

          <Show when={currentRegistry().eligible_protocols.length > 0} fallback={<p>当前没有通过 Stage 66 且尚未登记实现的逐目标协议。</p>}>
            <label>
              <span>Stage 66 独立批准协议</span>
              <select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
                <For each={currentRegistry().eligible_protocols}>
                  {(item) => <option value={item.protocol_review.review_id}>{item.protocol.target_id} · {item.protocol.frozen_candidate_algorithm_id} · {item.protocol_review.review_id.slice(0, 12)}…</option>}
                </For>
              </select>
            </label>
            <label><span>实现名称</span><input value={implementationName()} onInput={(event) => setImplementationName(event.currentTarget.value)} /></label>
            <label><span>不可变代码版本</span><input value={immutableCodeRevision()} onInput={(event) => setImmutableCodeRevision(event.currentTarget.value)} placeholder="commit / content-addressed revision" /></label>
            <label><span>实现工件 SHA-256</span><input value={artifactSha256()} onInput={(event) => setArtifactSha256(event.currentTarget.value)} placeholder="64 位十六进制摘要" /></label>
            <label><span>登记依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限</span><textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} /></label>
            <For each={CHECKS}>
              {([name, label]) => <label class="public-admin-anchor-check"><input type="checkbox" checked={checked()[name]} onChange={(event) => setChecked({ ...checked(), [name]: event.currentTarget.checked })} /><span>{label}</span></label>}
            </For>
            <button type="button" disabled={busy() || !selected() || !allConfirmed()} onClick={() => void submit()}>{busy() ? "正在冻结并登记…" : "登记零能力实现（不运行）"}</button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={currentRegistry().items}>
            {(item) => <article class="public-admin-reward-governance">
              <header><strong>{item.implementation.implementation_name}</strong><span>{item.implementation.status}</span></header>
              <p>{item.implementation.implementation_contract.target_id} · {item.implementation.implementation_contract.frozen_candidate_algorithm_id} · seeds {item.implementation.implementation_contract.exact_random_seeds.join("/")}</p>
              <p>特征 {item.implementation.implementation_contract.exact_feature_count} · 目标 {item.implementation.implementation_contract.exact_target_count} · bootstrap {item.implementation.implementation_contract.bootstrap_replications}</p>
              <p class="public-admin-anchor-boundary">{item.future_independent_implementation_review_eligible ? "只可进入未来独立实现复核" : "上游绑定已失效"}；runner、访问、评估、正式选模和交易全部关闭。</p>
            </article>}
          </For>
        </section>
      )}
    </Show>
  );
}
