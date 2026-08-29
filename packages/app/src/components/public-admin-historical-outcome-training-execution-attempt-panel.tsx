import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeTrainingExecutionAttempts,
  getHistoricalOutcomeTrainingFirstExecutionAuthorizations,
  invokeHistoricalOutcomeTrainingOnce,
} from "@/lib/api";
import type {
  HistoricalOutcomeTrainingExecutionAttemptRegistry,
  HistoricalOutcomeTrainingFirstExecutionAuthorizationRegistry,
} from "@/lib/types";

const EXECUTION_CHECKS = [
  "先 create-once 写入不可逆 claim；失败同样消耗授权，绝不自动重放",
  "只读取授权精确绑定且已独立校验的 training-store dataset，不读取其它历史或生产数据",
  "预处理和拟合只看 train；显式缺失保持为缺失，不用均值伪装成真实观测",
  "validation 与 sealed holdout 标签继续隐藏，本次不做 validation 选模",
  "只运行冻结的零预测、岭回归、梯度提升三臂以及 17/29/43 三种子",
  "输出仅为未验证、内容寻址候选；必须另行独立校验，train 指标不代表模型有效",
  "不写 reward、影子仓位或模型/指标库，不生成订单，不接券商，不交易",
] as const;

export function PublicAdminHistoricalOutcomeTrainingExecutionAttemptPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeTrainingExecutionAttemptRegistry>();
  const [authorizations, setAuthorizations] =
    createSignal<HistoricalOutcomeTrainingFirstExecutionAuthorizationRegistry>();
  const [selectedRunnerId, setSelectedRunnerId] = createSignal("");
  const [checks, setChecks] = createSignal(EXECUTION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const [nextRegistry, nextAuthorizations] = await Promise.all([
        getHistoricalOutcomeTrainingExecutionAttempts(),
        getHistoricalOutcomeTrainingFirstExecutionAuthorizations(),
      ]);
      setRegistry(nextRegistry);
      setAuthorizations(nextAuthorizations);
      const eligible = nextAuthorizations.items.filter((item) => item.execution_attempt_eligible);
      if (!eligible.some((item) => item.runner.isolated_runner_id === selectedRunnerId())) {
        setSelectedRunnerId(eligible[0]?.runner.isolated_runner_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练一次性执行注册表读取失败");
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
    const review = current?.latest_review;
    if (!current || !review || disabled()) return;
    const runner = current.runner;
    const implementation = runner.implementation;
    const stage52 = implementation.approved_registration_review;
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await invokeHistoricalOutcomeTrainingOnce(runner.isolated_runner_id, {
        expected_first_execution_authorization_review_id: review.review_id,
        expected_first_execution_authorization_review_sha256: review.review_sha256,
        expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256,
        expected_runner_artifact_sha256: runner.runner_artifact_sha256,
        expected_implementation_id: implementation.implementation_id,
        expected_implementation_sha256: implementation.implementation_sha256,
        expected_implementation_review_sha256: runner.implementation_review.review_sha256,
        expected_suite_specification_sha256: stage52.suite_specification_sha256,
        expected_training_store_dataset_sha256: stage52.training_store_dataset_sha256,
        expected_rows_sha256: stage52.rows_sha256,
        expected_excluded_rows_sha256: stage52.excluded_rows_sha256,
        expected_target_commitments_sha256: stage52.target_commitments_sha256,
        claim_first_create_once_and_failure_consumes_confirmed: confirmed[0],
        exact_read_only_training_store_dataset_only_confirmed: confirmed[1],
        train_only_fit_and_explicit_missingness_preserved_confirmed: confirmed[2],
        validation_and_sealed_holdout_labels_remain_withheld_confirmed: confirmed[3],
        fixed_three_arm_three_seed_suite_confirmed: confirmed[4],
        untrusted_content_addressed_output_and_independent_validation_confirmed: confirmed[5],
        no_reward_shadow_order_broker_or_trading_confirmed: confirmed[6],
      });
      setRegistry(next);
      setChecks(EXECUTION_CHECKS.map(() => false));
      setNotice(
        "一次性授权已经消费。若执行成功，当前只得到待独立校验的 train-only 内容寻址候选；真实拟合 ≠ 模型有效。",
      );
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练一次性执行失败；授权可能已经消费，请刷新核对");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="训练一次性执行尝试">
          <header>
            <strong>第 57 阶段 · 训练一次性执行尝试</strong>
            <span>{currentRegistry().execution_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可调用授权</span><strong>{currentRegistry().invocation_eligible_authorization_count}</strong></div>
            <div><span>不可逆 claim</span><strong>{currentRegistry().claim_count}</strong></div>
            <div><span>完成/失败</span><strong>{currentRegistry().completed_attempt_count}/{currentRegistry().failed_attempt_count}</strong></div>
            <div><span>待独立校验</span><strong>{currentRegistry().independent_output_validation_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>真实拟合 ≠ 模型有效</strong><span>只看 train · 不可选模</span></header>
            <p>本次只证明固定代码可以在精确训练副本上拟合并产生可复核工件。validation 与 sealed holdout 标签不可见，train 指标不能用于比较模型优劣或投资决策。</p>
            <p class="public-admin-anchor-boundary">所有输出保持未验证；模型/指标库、reward、影子组合、订单、券商和交易全部关闭。</p>
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
              {busy() ? "正在执行并封存候选…" : "消费一次性授权并运行 train-only 拟合"}
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
                <p>claim {attempt.claim.claim_sha256} · {attempt.claim.claimed_at} · {attempt.claim.invoked_by}</p>
                <Show when={attempt.result?.untrusted_artifact_envelope}>
                  {(envelope) => <p>train {envelope().train_row_count} 行 · 9 个模型工件 · {envelope().fit_diagnostics.length} 条 train-only 诊断 · validation/holdout 标签均未访问</p>}
                </Show>
                <Show when={attempt.result?.bounded_error}>{(message) => <p class="public-admin-error">{message()}</p>}</Show>
                <p class="public-admin-anchor-boundary">该结果未经独立校验，不得用于模型有效性、收益、评级、仓位或交易主张。</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
