import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeOfflineDatasetTransformationExecutionAttempts,
  invokeHistoricalOutcomeOfflineDatasetTransformationOnce,
} from "@/lib/api";
import type { HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptRegistry } from "@/lib/types";

export function PublicAdminHistoricalOutcomeTransformationExecutionAttemptPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptRegistry>();
  const [selectedRunnerId, setSelectedRunnerId] = createSignal("");
  const [confirmed, setConfirmed] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeOfflineDatasetTransformationExecutionAttempts();
      setRegistry(next);
      if (
        !next.eligible_authorizations.some(
          (authorization) => authorization.runner.isolated_runner_id === selectedRunnerId(),
        )
      ) {
        setSelectedRunnerId(
          next.eligible_authorizations[0]?.runner.isolated_runner_id ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "隔离转换执行记录读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.eligible_authorizations.find(
      (authorization) => authorization.runner.isolated_runner_id === selectedRunnerId(),
    ),
  );

  const invoke = async () => {
    const authorization = selected();
    if (!authorization || !confirmed() || busy()) return;
    const runner = authorization.runner;
    const implementation = runner.implementation;
    const specification = implementation.approved_review.specification;
    const subject = specification.subject;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await invokeHistoricalOutcomeOfflineDatasetTransformationOnce(
        runner.isolated_runner_id,
        {
          expected_first_execution_authorization_review_id: authorization.review.review_id,
          expected_first_execution_authorization_review_sha256: authorization.review.review_sha256,
          expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256,
          expected_runner_artifact_sha256: runner.runner_artifact_sha256,
          expected_runner_code_revision: runner.runner_code_revision,
          expected_implementation_id: implementation.implementation_id,
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_review_sha256: runner.implementation_review.review_sha256,
          expected_transformation_spec_sha256: specification.transformation_spec_sha256,
          expected_transformation_body_sha256: specification.transformation_body_sha256,
          expected_split_specification_sha256:
            specification.split_manifest_specification.specification_sha256,
          expected_feature_specification_sha256:
            specification.feature_bundle_specification.specification_sha256,
          expected_dataset_id: subject.dataset_id,
          expected_dataset_content_sha256: subject.dataset_content_sha256,
          expected_dataset_manifest_sha256: subject.manifest_sha256,
          expected_candidate_set_sha256: subject.candidate_set_sha256,
        },
      );
      setRegistry(next);
      setConfirmed(false);
      setSelectedRunnerId(next.eligible_authorizations[0]?.runner.isolated_runner_id ?? "");
      const latest = next.attempts[0]?.result;
      setNotice(
        latest?.status === "completed_with_untrusted_candidate_envelope"
          ? "一次性转换已完成：仅生成待独立校验候选；不是正式 manifest、特征包或训练输入。"
          : "转换尝试失败且授权已经消费；请检查失败原因，修复后重新走独立授权。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "隔离转换一次性执行失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="隔离转换一次性执行尝试">
          <header>
            <strong>第 31 阶段 · 隔离转换一次性执行尝试</strong>
            <span>{currentRegistry().execution_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可领取授权</span><strong>{currentRegistry().invocation_eligible_authorization_count}</strong></div>
            <div><span>执行尝试</span><strong>{currentRegistry().attempt_count}</strong></div>
            <div><span>成功候选</span><strong>{currentRegistry().untrusted_candidate_envelope_count}</strong></div>
            <div><span>失败且已消费</span><strong>{currentRegistry().failed_attempt_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>一次领取 · 成败都消费</strong><span>输出仍不可信</span></header>
            <p>执行前会重开当前数据集、治理、规范、实现、runner 与授权的完整精确绑定，并重算当前运行制品摘要。</p>
            <p class="public-admin-anchor-boundary">只执行固定纯函数：确定性分组、连续时间切分、250 交易日 purge/embargo，以及 65 项显式缺失特征候选。不会抓取新数据、补值、连接特征、定义目标或开始训练。</p>
          </article>

          <Show when={currentRegistry().eligible_authorizations.length > 0} fallback={
            <p>当前没有未过期且未 claim 的一次性授权。</p>
          }>
            <label>
              <span>待消费授权</span>
              <select value={selectedRunnerId()} onChange={(event) => setSelectedRunnerId(event.currentTarget.value)}>
                <For each={currentRegistry().eligible_authorizations}>
                  {(authorization) => (
                    <option value={authorization.runner.isolated_runner_id}>
                      {authorization.runner.runner_name} · 授权至 {authorization.review.authorization_valid_until}
                    </option>
                  )}
                </For>
              </select>
            </label>
            <label class="public-admin-anchor-check">
              <input
                type="checkbox"
                checked={confirmed()}
                onChange={(event) => setConfirmed(event.currentTarget.checked)}
              />
              <span>我确认：点击后先写不可变 claim；成功或失败都会消费授权，结果必须另行独立校验。</span>
            </label>
            <button type="button" disabled={!confirmed() || !selected() || busy()} onClick={() => void invoke()}>
              {busy() ? "正在执行固定纯函数…" : "领取授权并执行一次（失败也消费）"}
            </button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>

          <For each={currentRegistry().attempts}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>attempt {item.claim.attempt_id.slice(0, 12)}…</strong>
                  <span>{item.result?.status ?? "claimed_incomplete_fail_closed"}</span>
                </header>
                <p>runner {item.claim.isolated_runner_id} · {item.claim.claimed_at} · 当前授权绑定 {item.current_authorization_binding ? "是" : "否/已过期"}</p>
                <Show when={item.result?.bounded_error}>
                  {(message) => <p class="public-admin-decision-error">{message()}</p>}
                </Show>
                <Show when={item.result?.untrusted_candidate_envelope}>
                  {(envelope) => (
                    <p>候选记录 {envelope().entry_count} · 连通分量 {envelope().component_count} · 特征目录 {envelope().feature_catalog_count} · 独立校验：未完成</p>
                  )}
                </Show>
                <p class="public-admin-anchor-boundary">正式 manifest：未创建；正式 feature bundle：未创建；训练、奖励、影子、订单、券商与交易：全部关闭。</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
