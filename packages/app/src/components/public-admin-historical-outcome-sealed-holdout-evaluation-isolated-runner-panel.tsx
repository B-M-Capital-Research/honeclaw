import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunners,
  registerHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunner,
} from "@/lib/api";
import type {
  HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRegistry,
  RegisterHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRequest,
} from "@/lib/types";

const CHECKS = [
  ["exact_current_stage_51_through_stage_68_binding_confirmed", "精确绑定当前 Stage 51–68 完整证据链"],
  ["registrar_independent_from_stage_68_and_complete_prior_chain_confirmed", "登记人独立于 Stage 68 复核人与完整上游角色链"],
  ["runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed", "runner 工件、代码、运行时、协议和序列化合同不可变"],
  ["future_exact_read_only_one_target_holdout_and_three_candidate_mounts_confirmed", "未来只允许一个目标留出集和 17/29/43 三候选精确只读挂载"],
  ["training_validation_cross_target_and_feedback_isolation_confirmed", "训练、validation、跨目标读取与反馈复用保持隔离"],
  ["one_algorithm_three_seed_metrics_bootstrap_holm_and_sample_gates_confirmed", "单算法三种子、指标、bootstrap、Holm 与样本门禁完全冻结"],
  ["create_once_untrusted_output_and_independent_validation_confirmed", "未来输出 create-once、视为不可信并必须独立校验"],
  ["fixed_runtime_identity_and_bounded_resource_contract_confirmed", "运行时身份固定且资源上限静态受控"],
  ["no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed", "当前无入口、环境、密钥、网络、工具、子进程或生产访问"],
  ["registration_access_authorization_execution_and_output_validation_separation_confirmed", "登记、访问授权、执行和输出校验严格分离"],
  ["no_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed", "当前无留出集访问、评估、选模、存储、奖励、影子、订单、券商或交易权限"],
] as const;

type CheckName = (typeof CHECKS)[number][0];

export function PublicAdminHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerPanel() {
  const [registry, setRegistry] = createSignal<HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRegistry>();
  const [selectedReviewId, setSelectedReviewId] = createSignal("");
  const [runnerName, setRunnerName] = createSignal("");
  const [runnerCodeRevision, setRunnerCodeRevision] = createSignal("");
  const [runnerArtifactSha256, setRunnerArtifactSha256] = createSignal("");
  const [rationale, setRationale] = createSignal("");
  const [knownLimitations, setKnownLimitations] = createSignal("");
  const [checked, setChecked] = createSignal<Record<CheckName, boolean>>(
    Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
  );
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const availableReviews = createMemo(() => {
    const current = registry();
    if (!current) return [];
    const registered = new Set(current.items.map((item) => item.runner.implementation_review.review_id));
    return current.eligible_reviews.filter((item) => !registered.has(item.review.review_id));
  });

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunners();
      setRegistry(next);
      const registered = new Set(next.items.map((item) => item.runner.implementation_review.review_id));
      const available = next.eligible_reviews.filter((item) => !registered.has(item.review.review_id));
      if (!available.some((item) => item.review.review_id === selectedReviewId())) {
        setSelectedReviewId(available[0]?.review.review_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 评估隔离 runner 登记表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() =>
    availableReviews().find((item) => item.review.review_id === selectedReviewId()),
  );
  const allConfirmed = createMemo(() => CHECKS.every(([name]) => checked()[name]));
  const shaValid = createMemo(() => /^[0-9a-fA-F]{64}$/.test(runnerArtifactSha256().trim()));

  const submit = async () => {
    const item = selected();
    if (!item || busy()) return;
    if (!runnerName().trim() || !runnerCodeRevision().trim() || !rationale().trim() || !knownLimitations().trim()) {
      setError("请填写 runner 名称、代码版本、登记理由和已知局限。");
      return;
    }
    if (!shaValid() || !allConfirmed()) {
      setError("请填写 64 位 runner 工件 SHA-256，并完成全部十一项确认。");
      return;
    }
    const implementation = item.implementation;
    const review = item.review;
    const contract = implementation.implementation_contract;
    const request: RegisterHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRequest = {
      expected_implementation_id: implementation.implementation_id,
      expected_implementation_sha256: implementation.implementation_sha256,
      expected_implementation_review_id: review.review_id,
      expected_implementation_review_sha256: review.review_sha256,
      expected_independent_audit_sha256: review.independent_audit.audit_sha256,
      expected_implementation_contract_sha256: contract.contract_sha256,
      expected_implementation_artifact_sha256: contract.implementation_artifact_sha256,
      expected_immutable_code_revision: contract.immutable_code_revision,
      expected_stage_66_protocol_review_sha256: contract.stage_66_protocol_review_sha256,
      expected_sealed_holdout_evaluation_protocol_sha256: contract.sealed_holdout_evaluation_protocol_sha256,
      expected_target_bundle_sha256: contract.target_bundle_sha256,
      expected_recommendation_sha256: contract.recommendation_sha256,
      expected_selected_algorithm_three_seed_binding_sha256: contract.selected_algorithm_three_seed_binding_sha256,
      expected_sealed_holdout_split_commitment_sha256: contract.sealed_holdout_split_commitment_sha256,
      expected_feature_order_sha256: contract.feature_order_sha256,
      expected_preprocessing_sha256: contract.preprocessing_sha256,
      expected_target_id: contract.target_id,
      expected_frozen_candidate_algorithm_id: contract.frozen_candidate_algorithm_id,
      runner_name: runnerName().trim(),
      runner_kind: "ephemeral_deterministic_one_target_three_seed_sealed_holdout_evaluator",
      runner_code_revision: runnerCodeRevision().trim(),
      runner_artifact_sha256: runnerArtifactSha256().trim().toLowerCase(),
      rationale: rationale().trim(),
      known_limitations: knownLimitations().trim(),
      ...checked(),
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setRegistry(await registerHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunner(request));
      setChecked(Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>);
      setNotice("runner 规格已不可变登记；仍无留出集访问、评估、正式选模或交易权限。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 评估隔离 runner 登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(current) => (
        <section class="public-admin-reward-governance" aria-label="sealed-holdout 评估隔离 runner 登记">
          <header><strong>第 69 阶段 · sealed-holdout 评估隔离 runner</strong><span>{current().runner_status}</span></header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可登记复核</span><strong>{current().registration_eligible_count}</strong></div>
            <div><span>runner 记录</span><strong>{current().runner_count}</strong></div>
            <div><span>当前绑定</span><strong>{current().current_binding_runner_count}</strong></div>
            <div><span>下一门禁资格</span><strong>{current().first_execution_authorization_review_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>登记不是访问，也不是执行</strong><span>create-once · no entrypoint</span></header>
            <p>未来也只能在新的链外一次性授权后，精确只读挂载一个目标的 sealed-holdout 特征/标签和一种算法的 17/29/43 三候选。</p>
            <p class="public-admin-anchor-boundary">当前没有任何挂载；不得训练、调参、反馈复用、跨目标读取、正式选模或写模型/指标库。</p>
          </article>

          <Show when={current().registration_allowed && availableReviews().length > 0} fallback={<p>当前没有尚未登记且绑定有效的 Stage 68 独立批准实现复核。</p>}>
            <label><span>批准复核</span><select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
              <For each={availableReviews()}>{(item) => <option value={item.review.review_id}>{item.implementation.implementation_name} · {item.review.review_id.slice(0, 12)}…</option>}</For>
            </select></label>
            <label><span>runner 名称</span><input value={runnerName()} onInput={(event) => setRunnerName(event.currentTarget.value)} /></label>
            <label><span>runner 代码版本</span><input value={runnerCodeRevision()} onInput={(event) => setRunnerCodeRevision(event.currentTarget.value)} /></label>
            <label><span>runner 工件 SHA-256</span><input value={runnerArtifactSha256()} onInput={(event) => setRunnerArtifactSha256(event.currentTarget.value)} /></label>
            <label><span>登记理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限</span><textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} /></label>
            <For each={CHECKS}>{([name, label]) => <label class="public-admin-anchor-check"><input type="checkbox" checked={checked()[name]} onChange={(event) => setChecked({ ...checked(), [name]: event.currentTarget.checked })} /><span>{label}</span></label>}</For>
            <button type="button" disabled={busy() || !selected() || !shaValid() || !allConfirmed()} onClick={() => void submit()}>{busy() ? "正在登记…" : "登记不可变 runner 规格"}</button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={current().items}>{(item) => <article class="public-admin-reward-governance">
            <header><strong>{item.runner.runner_name}</strong><span>{item.runner.status}</span></header>
            <p>runner / 实现 / 复核：{item.runner.isolated_runner_id.slice(0, 12)}… / {item.runner.implementation.implementation_id.slice(0, 12)}… / {item.runner.implementation_review.review_id.slice(0, 12)}…</p>
            <p>目标 / 算法 / 种子：{item.runner.runner_contract.target_id} · {item.runner.runner_contract.frozen_candidate_algorithm_id} · {item.runner.runner_contract.exact_random_seeds.join(" / ")}</p>
            <p>资源上限：{item.runner.runner_contract.maximum_memory_mib} MiB · {item.runner.runner_contract.maximum_cpu_millicores} mCPU · {item.runner.runner_contract.maximum_wall_clock_seconds}s · {item.runner.runner_contract.maximum_process_count} processes</p>
            <p class="public-admin-anchor-boundary">{item.approved_review_binding_current ? "当前绑定有效，只可进入独立一次性访问与执行授权复核" : "绑定已变化，晋级关闭"}；当前无留出集访问、评估、选模或交易权限。</p>
          </article>}</For>
        </section>
      )}
    </Show>
  );
}
