import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeValidationEvaluationExecutionAttempts,
  getHistoricalOutcomeValidationEvaluationFirstExecutionAuthorizations,
  invokeHistoricalOutcomeValidationEvaluationOnce,
} from "@/lib/api";
import type {
  HistoricalOutcomeValidationEvaluationExecutionAttemptRegistry,
  HistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationRegistry,
} from "@/lib/types";

const EXECUTION_CHECKS = [
  "先写不可逆 claim；读取失败、评估失败或输出失败都会消费授权，不得自动重放",
  "只向固定评估 worker 投影精确 validation 特征/标签和九个已校验候选",
  "只计算冻结的逐目标指标、成分块 bootstrap 与 54 项 Holm 校正",
  "不得挑种子、调参、合成总分或宣称整个模型有效",
  "本次只验证 validation；不更新训练，sealed holdout 特征和标签继续隐藏",
  "输出是内容寻址的不可信结果，必须另由独立角色复算校验",
  "不写模型/指标库、reward、影子仓位或订单，不访问券商，不交易",
] as const;

export function PublicAdminHistoricalOutcomeValidationEvaluationExecutionAttemptPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeValidationEvaluationExecutionAttemptRegistry>();
  const [authorizations, setAuthorizations] =
    createSignal<HistoricalOutcomeValidationEvaluationFirstExecutionAuthorizationRegistry>();
  const [selectedRunnerId, setSelectedRunnerId] = createSignal("");
  const [checks, setChecks] = createSignal(EXECUTION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const [nextRegistry, nextAuthorizations] = await Promise.all([
        getHistoricalOutcomeValidationEvaluationExecutionAttempts(),
        getHistoricalOutcomeValidationEvaluationFirstExecutionAuthorizations(),
      ]);
      setRegistry(nextRegistry);
      setAuthorizations(nextAuthorizations);
      const eligible = nextAuthorizations.items.filter((item) => item.execution_attempt_eligible);
      if (!eligible.some((item) => item.runner.isolated_runner_id === selectedRunnerId())) {
        setSelectedRunnerId(eligible[0]?.runner.isolated_runner_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "validation 评估执行记录读取失败");
    }
  };

  onMount(() => void load());

  const eligibleItems = createMemo(() =>
    authorizations()?.items.filter((item) => item.execution_attempt_eligible) ?? [],
  );
  const selected = createMemo(() =>
    eligibleItems().find((item) => item.runner.isolated_runner_id === selectedRunnerId()),
  );
  const disabled = createMemo(
    () => busy() || !selected() || checks().some((value) => !value),
  );

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) =>
      current.map((value, currentIndex) => (currentIndex === index ? checked : value)),
    );
  };

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
      const next = await invokeHistoricalOutcomeValidationEvaluationOnce(
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
          expected_candidate_set_sha256: contract.candidate_set_sha256,
          expected_upstream_validation_sha256:
            implementation.upstream_validation.validation_sha256,
          expected_upstream_output_sha256: implementation.upstream_validation.output_sha256,
          claim_first_create_once_and_failure_consumes_confirmed: confirmed[0],
          exact_validation_features_labels_and_nine_candidates_only_confirmed: confirmed[1],
          frozen_metrics_component_bootstrap_and_holm_confirmed: confirmed[2],
          no_seed_shopping_tuning_composite_or_global_claim_confirmed: confirmed[3],
          validation_only_no_training_update_and_sealed_holdout_hidden_confirmed: confirmed[4],
          untrusted_content_addressed_output_and_independent_validation_confirmed: confirmed[5],
          no_store_reward_shadow_order_broker_or_trading_confirmed: confirmed[6],
        },
      );
      setRegistry(next);
      setChecks(EXECUTION_CHECKS.map(() => false));
      setNotice("授权已消费。成功结果仍是不可信 validation 证据，必须独立复算后才能讨论逐目标候选。 ");
      await load();
    } catch (cause) {
      setError(
        cause instanceof Error
          ? cause.message
          : "validation 评估失败；授权可能已消费，请刷新核对不可变记录",
      );
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="validation 评估一次性执行">
          <header>
            <strong>第 63 阶段 · validation 评估一次性执行</strong>
            <span>{currentRegistry().execution_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可领取授权</span><strong>{currentRegistry().invocation_eligible_authorization_count}</strong></div>
            <div><span>不可逆 claim</span><strong>{currentRegistry().claim_count}</strong></div>
            <div><span>完成/失败</span><strong>{currentRegistry().completed_attempt_count}/{currentRegistry().failed_attempt_count}</strong></div>
            <div><span>待独立复算</span><strong>{currentRegistry().independent_output_validation_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>逐目标验证 ≠ 正式选模</strong><span>claim-first · sealed holdout 隐藏</span></header>
            <p>宿主标签代理只在 claim 落盘后重开原始结果；固定 worker 只接收 validation 行。当前是进程内能力隔离，不是操作系统级沙箱。</p>
            <p class="public-admin-anchor-boundary">即使九个目标全部产生建议，也没有综合分、全局有效性结论、模型库写入或任何投资执行权限。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有未过期且未消费的一次性授权。</p>}>
            <label>
              <span>可消费 runner 授权</span>
              <select value={selectedRunnerId()} onChange={(event) => setSelectedRunnerId(event.currentTarget.value)}>
                <For each={eligibleItems()}>
                  {(item) => <option value={item.runner.isolated_runner_id}>{item.runner.runner_name} · 截止 {item.latest_review?.authorization_valid_until}</option>}
                </For>
              </select>
            </label>
            <div class="public-admin-decision-checks">
              <For each={EXECUTION_CHECKS}>
                {(label, index) => (
                  <label>
                    <input type="checkbox" checked={checks()[index()]} onChange={(event) => toggleCheck(index(), event.currentTarget.checked)} />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
              {busy() ? "正在消费授权并冻结评估…" : "消费一次授权并运行 validation 评估"}
            </button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={currentRegistry().attempts}>
            {(attempt) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>attempt {attempt.claim.attempt_id.slice(0, 12)}…</strong>
                  <span>{attempt.result?.status ?? "claim 已消费 · 结果缺失失败关闭"}</span>
                </header>
                <p>{attempt.claim.claimed_at} · {attempt.claim.invoked_by} · {attempt.claim.isolation_backend}</p>
                <Show when={attempt.result?.untrusted_evaluation_envelope}>
                  {(envelope) => (
                    <>
                      <p>validation {envelope().validation_row_count} 行 / {envelope().independent_component_count} 个独立成分 · {envelope().exact_metric_count} 条指标 · {envelope().exact_candidate_hypothesis_count} 个校正假设</p>
                      <For each={envelope().per_target_recommendations}>
                        {(recommendation) => (
                          <p><strong>{recommendation.target_id}</strong> · {recommendation.status} · {recommendation.recommended_algorithm ?? "不建议候选"} · 三种子 {recommendation.all_three_seeds_passed ? "通过" : "未通过"}</p>
                        )}
                      </For>
                    </>
                  )}
                </Show>
                <Show when={attempt.result?.bounded_error}>{(message) => <p class="public-admin-error">{message()}</p>}</Show>
                <p class="public-admin-anchor-boundary">结果未经独立输出校验；不得视为正式选模、评级、仓位或交易依据。</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
