import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeValidationEvaluationIsolatedRunners,
  registerHistoricalOutcomeValidationEvaluationIsolatedRunner,
} from "@/lib/api";
import type {
  HistoricalOutcomeValidationEvaluationIsolatedRunnerRegistry,
  RegisterHistoricalOutcomeValidationEvaluationIsolatedRunnerRequest,
} from "@/lib/types";

const CHECKS = [
  ["exact_current_approved_review_and_complete_upstream_binding_confirmed", "精确绑定当前 Stage 60 批准复核及完整上游"],
  ["registrar_independence_confirmed", "登记人独立于实现、验证、执行与复核完整角色链"],
  ["runner_artifact_code_runtime_and_protocol_immutable_confirmed", "runner 工件、代码、运行时和冻结统计协议不可变"],
  ["future_exact_read_only_validation_and_candidate_mounts_confirmed", "未来只允许精确 validation 与九候选工件只读挂载"],
  ["sealed_holdout_and_training_update_isolation_confirmed", "sealed holdout 始终隔离且禁止更新训练或预处理"],
  ["per_target_per_seed_untrusted_output_and_independent_validation_confirmed", "只输出逐目标逐种子不可信包并另做独立校验"],
  ["fixed_runtime_identity_and_bounded_resource_contract_confirmed", "固定运行时身份、单任务和静态资源上限"],
  ["no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed", "当前无入口、环境、密钥、网络、工具、子进程或生产访问"],
  ["registration_first_execution_and_output_validation_separation_confirmed", "登记、首次执行授权与输出验证严格分离"],
  ["no_label_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed", "当前无标签、评估、选模、存储、奖励、影子、订单、券商或交易权限"],
] as const;

type CheckName = (typeof CHECKS)[number][0];

export function PublicAdminHistoricalOutcomeValidationEvaluationIsolatedRunnerPanel() {
  const [registry, setRegistry] = createSignal<HistoricalOutcomeValidationEvaluationIsolatedRunnerRegistry>();
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

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeValidationEvaluationIsolatedRunners();
      setRegistry(next);
      if (!next.eligible_reviews.some((item) => item.review.review_id === selectedReviewId())) {
        setSelectedReviewId(next.eligible_reviews[0]?.review.review_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "validation 评估隔离 runner 登记表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.eligible_reviews.find((item) => item.review.review_id === selectedReviewId()),
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
      setError("请填写 64 位 runner 工件 SHA-256，并完成全部十项确认。");
      return;
    }
    const implementation = item.implementation;
    const contract = implementation.implementation_contract;
    const request: RegisterHistoricalOutcomeValidationEvaluationIsolatedRunnerRequest = {
      expected_implementation_id: implementation.implementation_id,
      expected_implementation_sha256: implementation.implementation_sha256,
      expected_implementation_review_id: item.review.review_id,
      expected_implementation_review_sha256: item.review.review_sha256,
      expected_independent_audit_sha256: item.review.independent_audit.audit_sha256,
      expected_implementation_contract_sha256: contract.contract_sha256,
      expected_implementation_artifact_sha256: contract.implementation_artifact_sha256,
      expected_immutable_code_revision: contract.immutable_code_revision,
      expected_candidate_set_sha256: contract.candidate_set_sha256,
      expected_upstream_validation_sha256: implementation.upstream_validation.validation_sha256,
      expected_upstream_output_sha256: implementation.upstream_validation.output_sha256,
      runner_name: runnerName().trim(),
      runner_kind: "ephemeral_deterministic_per_target_validation_evaluator",
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
      setRegistry(await registerHistoricalOutcomeValidationEvaluationIsolatedRunner(request));
      setChecked(Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>);
      setNotice("runner 规格已不可变登记；仍无入口、validation 标签访问、评估或选模权限。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "validation 评估隔离 runner 登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(current) => (
        <section class="public-admin-reward-governance" aria-label="validation 评估隔离 runner 登记">
          <header><strong>第 61 阶段 · validation 评估隔离 runner</strong><span>{current().runner_status}</span></header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可登记复核</span><strong>{current().eligible_reviews.length}</strong></div>
            <div><span>runner 记录</span><strong>{current().runner_count}</strong></div>
            <div><span>当前绑定</span><strong>{current().current_binding_runner_count}</strong></div>
            <div><span>下一门禁资格</span><strong>{current().first_execution_authorization_review_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>登记不是执行</strong><span>create-once · no entrypoint</span></header>
            <p>未来输入合同只允许精确 validation 分区与 9 个候选工件只读挂载；当前连这些挂载都不存在。</p>
            <p class="public-admin-anchor-boundary">sealed holdout 始终不可见；未来输出也只是逐目标、逐种子的不可信评估包，必须另行独立校验。</p>
          </article>

          <Show when={current().registration_allowed && current().eligible_reviews.length > 0} fallback={<p>当前没有已独立批准且绑定有效的 Stage 60 实现复核。</p>}>
            <label><span>批准复核</span><select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
              <For each={current().eligible_reviews}>{(item) => <option value={item.review.review_id}>{item.implementation.implementation_name} · {item.review.review_id.slice(0, 12)}…</option>}</For>
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
            <p>资源上限：{item.runner.runner_contract.maximum_memory_mib} MiB · {item.runner.runner_contract.maximum_cpu_millicores} mCPU · {item.runner.runner_contract.maximum_wall_clock_seconds}s · {item.runner.runner_contract.maximum_process_count} processes</p>
            <p class="public-admin-anchor-boundary">{item.approved_review_binding_current ? "当前绑定有效，只可进入独立首次执行授权复核" : "绑定已变化，晋级关闭"}；当前无标签、评估、选模或交易权限。</p>
          </article>}</For>
        </section>
      )}
    </Show>
  );
}
