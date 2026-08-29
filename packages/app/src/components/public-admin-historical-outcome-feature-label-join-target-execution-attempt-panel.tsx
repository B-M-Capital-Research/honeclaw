import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempts,
  invokeHistoricalOutcomeFeatureLabelJoinTargetOnce,
} from "@/lib/api";
import type { HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptRegistry } from "@/lib/types";

const EXECUTION_CHECKS = [
  "确认先以 create-once claim 消费这条授权；成功或失败都不能重放",
  "确认只做 official split、65 项特征与当前原始结果的一对一纯函数连接，并投影九项原始目标位模式",
  "确认 validation 与 sealed holdout 的目标值继续隐藏，只留下内容承诺",
  "确认输出只是不可信候选，不授权训练、奖励、影子组合、订单、券商或交易",
] as const;

export function PublicAdminHistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptRegistry>();
  const [selectedRunnerId, setSelectedRunnerId] = createSignal("");
  const [checks, setChecks] = createSignal(EXECUTION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempts();
      setRegistry(next);
      if (
        !next.eligible_authorizations.some(
          (authorization) => authorization.runner.isolated_runner_id === selectedRunnerId(),
        )
      ) {
        setSelectedRunnerId(next.eligible_authorizations[0]?.runner.isolated_runner_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 一次性执行记录读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.eligible_authorizations.find(
      (authorization) => authorization.runner.isolated_runner_id === selectedRunnerId(),
    ),
  );
  const disabled = createMemo(
    () => busy() || !selected() || checks().some((confirmed) => !confirmed),
  );

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) =>
      current.map((value, currentIndex) => (currentIndex === index ? checked : value)),
    );
  };

  const invokeOnce = async () => {
    const authorization = selected();
    if (!authorization || disabled()) return;
    const runner = authorization.runner;
    const implementation = runner.implementation;
    const specification = implementation.approved_review.specification;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await invokeHistoricalOutcomeFeatureLabelJoinTargetOnce(
        runner.isolated_runner_id,
        {
          expected_first_execution_authorization_review_id: authorization.review.review_id,
          expected_first_execution_authorization_review_sha256:
            authorization.review.review_sha256,
          expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256,
          expected_runner_artifact_sha256: runner.runner_artifact_sha256,
          expected_runner_code_revision: runner.runner_code_revision,
          expected_implementation_id: implementation.implementation_id,
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_review_sha256: runner.implementation_review.review_sha256,
          expected_specification_id: specification.specification_id,
          expected_specification_sha256: specification.specification_sha256,
          expected_specification_body_sha256: specification.specification_body_sha256,
          expected_join_specification_sha256:
            specification.join_specification.specification_sha256,
          expected_target_specification_sha256:
            specification.target_specification.specification_sha256,
          expected_validation_id: specification.validation_id,
          expected_validation_sha256: specification.validation_sha256,
          expected_split_manifest_sha256: specification.split_manifest_sha256,
          expected_feature_bundle_sha256: specification.feature_bundle_sha256,
          expected_combined_artifact_sha256: specification.combined_artifact_sha256,
          expected_dataset_id: specification.dataset_id,
          expected_dataset_content_sha256: specification.dataset_content_sha256,
          expected_dataset_manifest_sha256: specification.dataset_manifest_sha256,
          expected_candidate_set_sha256: specification.candidate_set_sha256,
          create_once_claim_and_failure_consumes_confirmed: true,
          exact_one_to_one_join_and_nine_raw_target_projection_confirmed: true,
          validation_and_sealed_holdout_target_values_withheld_confirmed: true,
          no_training_reward_shadow_order_broker_or_trading_confirmed: true,
        },
      );
      setRegistry(next);
      setChecks(EXECUTION_CHECKS.map(() => false));
      setSelectedRunnerId(next.eligible_authorizations[0]?.runner.isolated_runner_id ?? "");
      setNotice(
        "授权已经消费。若执行成功，输出仍只是不可信候选，必须经过下一阶段独立逐位重算后才能继续。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 一次性执行失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="join/target 一次性执行尝试">
          <header>
            <strong>第 42 阶段 · join/target 一次性执行尝试</strong>
            <span>{currentRegistry().execution_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div>
              <span>可领取授权</span>
              <strong>{currentRegistry().invocation_eligible_authorization_count}</strong>
            </div>
            <div>
              <span>执行尝试</span>
              <strong>{currentRegistry().attempt_count}</strong>
            </div>
            <div>
              <span>不可信候选</span>
              <strong>{currentRegistry().untrusted_candidate_envelope_count}</strong>
            </div>
            <div>
              <span>待独立校验</span>
              <strong>{currentRegistry().independent_output_validation_eligible_count}</strong>
            </div>
          </div>

          <article class="public-admin-reward-governance">
            <header>
              <strong>执行边界</strong>
              <span>单次 · 失败也消费</span>
            </header>
            <p>
              只读取精确绑定的正式 split、65 项点时特征与当前原始结果数据集。train
              可见九项原始位模式；validation 与 sealed holdout 仅保留承诺哈希。
            </p>
            <p class="public-admin-anchor-boundary">
              本阶段不创建正式 joined dataset 或训练数据，不启动训练，不写奖励、影子仓位、订单，也不访问券商。
            </p>
          </article>

          <Show
            when={currentRegistry().eligible_authorizations.length > 0}
            fallback={<p class="public-admin-anchor-boundary">当前没有可领取的一次性授权。</p>}
          >
            <label>
              <span>待执行 runner</span>
              <select
                value={selectedRunnerId()}
                onChange={(event) => setSelectedRunnerId(event.currentTarget.value)}
              >
                <For each={currentRegistry().eligible_authorizations}>
                  {(authorization) => (
                    <option value={authorization.runner.isolated_runner_id}>
                      {authorization.runner.runner_name} · 授权至 {authorization.review.authorization_valid_until}
                    </option>
                  )}
                </For>
              </select>
            </label>
            <div class="public-admin-decision-checks">
              <For each={EXECUTION_CHECKS}>
                {(label, index) => (
                  <label>
                    <input
                      type="checkbox"
                      checked={checks()[index()]}
                      onChange={(event) => toggleCheck(index(), event.currentTarget.checked)}
                    />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button
              type="button"
              class="public-admin-decision-submit"
              disabled={disabled()}
              onClick={() => void invokeOnce()}
            >
              领取授权并执行一次（失败也消费）
            </button>
          </Show>

          <Show when={error()}>
            <p class="public-admin-error">{error()}</p>
          </Show>
          <Show when={notice()}>
            <p class="public-admin-success">{notice()}</p>
          </Show>

          <For each={currentRegistry().attempts}>
            {(attempt) => {
              const envelope = () => attempt.result?.untrusted_candidate_envelope;
              return (
                <article class="public-admin-reward-governance">
                  <header>
                    <strong>attempt {attempt.claim.attempt_id}</strong>
                    <span>{attempt.result?.status ?? "claim 已创建，等待结果"}</span>
                  </header>
                  <p>
                    授权 {attempt.claim.authorization_review_id} · {attempt.claim.claimed_at} ·
                    操作者 {attempt.claim.invoked_by}
                  </p>
                  <Show when={attempt.result?.bounded_error}>
                    {(message) => <p class="public-admin-error">{message()}</p>}
                  </Show>
                  <Show when={envelope()}>
                    {(candidate) => (
                      <>
                        <div class="public-admin-decision-metrics">
                          <div>
                            <span>有效候选行</span>
                            <strong>{candidate().active_candidate_row_count}</strong>
                          </div>
                          <div>
                            <span>train 目标向量</span>
                            <strong>{candidate().train_target_vector_count}</strong>
                          </div>
                          <div>
                            <span>validation 隐藏</span>
                            <strong>{candidate().validation_target_withheld_count}</strong>
                          </div>
                          <div>
                            <span>封存隐藏</span>
                            <strong>{candidate().sealed_holdout_target_withheld_count}</strong>
                          </div>
                        </div>
                        <p class="public-admin-anchor-boundary">
                          输出为不可信候选：{candidate().output_is_untrusted ? "是" : "否"}；独立校验：
                          {candidate().independent_output_validation_completed ? "已完成" : "未完成"}；训练：
                          {candidate().training_started ? "已开始" : "关闭"}。
                        </p>
                      </>
                    )}
                  </Show>
                </article>
              );
            }}
          </For>
        </section>
      )}
    </Show>
  );
}
