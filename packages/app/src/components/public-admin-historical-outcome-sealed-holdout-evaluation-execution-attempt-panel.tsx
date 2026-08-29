import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempts,
  getHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizations,
  invokeHistoricalOutcomeSealedHoldoutEvaluationOnce,
} from "@/lib/api";
import type {
  HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry,
  HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRegistry,
} from "@/lib/types";

const EXECUTION_CHECKS = [
  "先写不可逆 claim；读取、评估、输出失败或中断都会消费授权，不得重放",
  "只投影一个冻结目标、一个冻结算法和 17/29/43 三个候选模型",
  "只读取 sealed-holdout；不得读取训练/验证分区、其他目标或未限定数据",
  "只计算预注册指标、成分块 bootstrap、三项 Holm 校正和样本门槛",
  "结果不得反馈调参、重训、换种子、重选候选或合成跨目标总分",
  "输出是内容寻址的不可信确认结果，必须由下一位独立角色完整复算",
  "不写模型/指标库、reward、影子仓位或订单，不访问券商，不交易",
] as const;

export function PublicAdminHistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry>();
  const [authorizations, setAuthorizations] =
    createSignal<HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRegistry>();
  const [selectedRunnerId, setSelectedRunnerId] = createSignal("");
  const [checks, setChecks] = createSignal(EXECUTION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const [nextRegistry, nextAuthorizations] = await Promise.all([
        getHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempts(),
        getHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizations(),
      ]);
      setRegistry(nextRegistry);
      setAuthorizations(nextAuthorizations);
      const eligible = nextAuthorizations.items.filter((item) => item.execution_attempt_eligible);
      if (!eligible.some((item) => item.runner.isolated_runner_id === selectedRunnerId())) {
        setSelectedRunnerId(eligible[0]?.runner.isolated_runner_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 一次性执行记录读取失败");
    }
  };

  onMount(() => void load());

  const eligibleItems = createMemo(() =>
    authorizations()?.items.filter((item) => item.execution_attempt_eligible) ?? [],
  );
  const selected = createMemo(() =>
    eligibleItems().find((item) => item.runner.isolated_runner_id === selectedRunnerId()),
  );
  const disabled = createMemo(() => busy() || !selected() || checks().some((value) => !value));

  const submit = async () => {
    const current = selected();
    const authorization = current?.latest_review;
    if (!current || !authorization || disabled()) return;
    const runner = current.runner;
    const implementation = runner.implementation;
    const contract = implementation.implementation_contract;
    const review = runner.implementation_review;
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await invokeHistoricalOutcomeSealedHoldoutEvaluationOnce(
        runner.isolated_runner_id,
        {
          expected_first_execution_authorization_review_id: authorization.review_id,
          expected_first_execution_authorization_review_sha256: authorization.review_sha256,
          expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256,
          expected_runner_artifact_sha256: runner.runner_artifact_sha256,
          expected_runner_code_revision: runner.runner_code_revision,
          expected_runner_contract_sha256: runner.runner_contract.contract_sha256,
          expected_implementation_id: implementation.implementation_id,
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_contract_sha256: contract.contract_sha256,
          expected_implementation_review_id: review.review_id,
          expected_implementation_review_sha256: review.review_sha256,
          expected_implementation_independent_audit_sha256: review.independent_audit.audit_sha256,
          expected_protocol_sha256: implementation.upstream_protocol.protocol_sha256,
          expected_candidate_set_sha256: contract.candidate_set_sha256,
          expected_training_store_dataset_sha256: contract.training_store_dataset_sha256,
          expected_selected_algorithm_three_seed_binding_sha256:
            contract.selected_algorithm_three_seed_binding_sha256,
          expected_sealed_holdout_split_commitment_sha256:
            contract.sealed_holdout_split_commitment_sha256,
          expected_target_id: contract.target_id,
          expected_frozen_candidate_algorithm_id: contract.frozen_candidate_algorithm_id,
          claim_first_single_use_and_failure_consumes_confirmed: confirmed[0],
          exact_one_target_one_algorithm_three_seed_projection_confirmed: confirmed[1],
          sealed_holdout_only_and_no_other_partition_or_target_access_confirmed: confirmed[2],
          frozen_metrics_component_bootstrap_holm_and_sample_gates_confirmed: confirmed[3],
          no_feedback_tuning_refit_reselection_or_composite_confirmed: confirmed[4],
          untrusted_content_addressed_output_and_independent_validation_confirmed: confirmed[5],
          no_store_reward_shadow_order_broker_or_trading_confirmed: confirmed[6],
        },
      );
      setRegistry(next);
      setChecks(EXECUTION_CHECKS.map(() => false));
      setNotice("授权已经永久消费。结果仍是不可信确认，只能送独立复算，不能反馈或正式选模。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "执行失败；授权可能已消费，请刷新核对 claim");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="sealed-holdout 一次性确认执行">
          <header><strong>第 71 阶段 · sealed-holdout 一次性确认执行</strong><span>{currentRegistry().execution_status}</span></header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可领取授权</span><strong>{currentRegistry().invocation_eligible_authorization_count}</strong></div>
            <div><span>不可逆 claim</span><strong>{currentRegistry().claim_count}</strong></div>
            <div><span>完成/失败</span><strong>{currentRegistry().completed_attempt_count}/{currentRegistry().failed_attempt_count}</strong></div>
            <div><span>待独立复算</span><strong>{currentRegistry().independent_output_validation_eligible_count}</strong></div>
          </div>
          <article class="public-admin-reward-governance">
            <header><strong>这是一次开卷机会，不是正式选模</strong><span>one target · one algorithm · three seeds</span></header>
            <p>claim 落盘后，标签代理只投影已冻结目标的 sealed-holdout 行；不把训练/验证分区、其他目标或其他算法交给评估 worker。</p>
            <p class="public-admin-anchor-boundary">无论通过、失败还是样本不足，同一授权都不能重放；结果也不能反馈训练或直接成为投资动作。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有未过期且未消费的一次性授权。</p>}>
            <label>
              <span>可消费 runner 授权</span>
              <select value={selectedRunnerId()} onChange={(event) => setSelectedRunnerId(event.currentTarget.value)}>
                <For each={eligibleItems()}>{(item) => <option value={item.runner.isolated_runner_id}>{item.runner.runner_name} · {item.runner.implementation.implementation_contract.target_id} · 截止 {item.latest_review?.authorization_valid_until}</option>}</For>
              </select>
            </label>
            <div class="public-admin-decision-checks">
              <For each={EXECUTION_CHECKS}>{(label, index) => (
                <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, current) => current === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
              )}</For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
              {busy() ? "正在 claim 并执行一次确认…" : "永久消费授权并执行 sealed-holdout 确认"}
            </button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={currentRegistry().attempts}>{(attempt) => (
            <article class="public-admin-reward-governance">
              <header><strong>{attempt.claim.target_id} · {attempt.claim.attempt_id.slice(0, 12)}…</strong><span>{attempt.result?.status ?? "claim 已消费 · 结果缺失失败关闭"}</span></header>
              <p>{attempt.claim.frozen_candidate_algorithm_id} · seeds {attempt.claim.exact_random_seeds.join("/")} · {attempt.claim.claimed_at}</p>
              <Show when={attempt.result?.untrusted_confirmation_envelope}>{(envelope) => (
                <>
                  <p>{envelope().sealed_holdout_row_count} 行 / {envelope().independent_component_count} 个独立成分 · {envelope().confirmation_status}</p>
                  <For each={envelope().metrics}>{(metric) => <p>seed {metric.random_seed} · {metric.evidence_status} · 全门槛 {metric.all_preregistered_thresholds_passed ? "通过" : "未通过"}</p>}</For>
                </>
              )}</Show>
              <Show when={attempt.result?.bounded_error}>{(message) => <p class="public-admin-error">{message()}</p>}</Show>
              <p class="public-admin-anchor-boundary">未经独立输出验证；不得反馈、重跑、正式选模、评级、建仓或交易。</p>
            </article>
          )}</For>
        </section>
      )}
    </Show>
  );
}
