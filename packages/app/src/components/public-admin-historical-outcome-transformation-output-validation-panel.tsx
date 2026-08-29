import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeOfflineDatasetTransformationOutputValidations,
  validateHistoricalOutcomeOfflineDatasetTransformationOutput,
} from "@/lib/api";
import type { HistoricalOutcomeOfflineDatasetTransformationOutputValidationRegistry } from "@/lib/types";

export function PublicAdminHistoricalOutcomeTransformationOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeOfflineDatasetTransformationOutputValidationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [confirmed, setConfirmed] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeOfflineDatasetTransformationOutputValidations();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.validation_eligible);
      if (!eligible.some((item) => item.attempt.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(eligible[0]?.attempt.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "离线转换输出独立校验读取失败");
    }
  };

  onMount(() => void load());

  const eligibleItems = createMemo(() =>
    registry()?.items.filter((item) => item.validation_eligible) ?? [],
  );
  const selected = createMemo(() =>
    eligibleItems().find((item) => item.attempt.claim.attempt_id === selectedAttemptId()),
  );

  const validate = async () => {
    const item = selected();
    const outputSha256 = item?.attempt.result.output_sha256;
    if (!item || !outputSha256 || !confirmed() || busy()) return;
    const claim = item.attempt.claim;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await validateHistoricalOutcomeOfflineDatasetTransformationOutput(
        claim.attempt_id,
        {
          expected_claim_sha256: claim.claim_sha256,
          expected_result_sha256: item.attempt.result.result_sha256,
          expected_output_sha256: outputSha256,
          expected_dataset_content_sha256: claim.dataset_content_sha256,
          expected_dataset_manifest_sha256: claim.dataset_manifest_sha256,
          expected_candidate_set_sha256: claim.candidate_set_sha256,
          expected_transformation_spec_sha256: claim.transformation_spec_sha256,
          expected_split_specification_sha256: claim.split_specification_sha256,
          expected_feature_specification_sha256: claim.feature_specification_sha256,
        },
      );
      setRegistry(next);
      setConfirmed(false);
      const remaining = next.items.filter((candidate) => candidate.validation_eligible);
      setSelectedAttemptId(remaining[0]?.attempt.claim.attempt_id ?? "");
      const validation = next.items.find(
        (candidate) => candidate.attempt.claim.attempt_id === claim.attempt_id,
      )?.validation;
      setNotice(
        validation?.untrusted_candidate_envelope_validated
          ? "独立重算通过：候选仍不是真正的 manifest、特征包或训练输入。"
          : "独立重算未通过：记录已不可变保存，必须失败关闭并审计。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "离线转换输出独立校验失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="离线转换输出独立校验">
          <header>
            <strong>第 32 阶段 · 离线转换输出独立重算</strong>
            <span>{currentRegistry().validation_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待独立校验</span><strong>{currentRegistry().validation_eligible_count}</strong></div>
            <div><span>校验记录</span><strong>{currentRegistry().validation_count}</strong></div>
            <div><span>重算通过</span><strong>{currentRegistry().validated_candidate_envelope_count}</strong></div>
            <div><span>失败关闭</span><strong>{currentRegistry().failed_validation_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>独立算法 · 不复用执行代码</strong><span>图遍历重算</span></header>
            <p>校验器重新打开精确当前数据集、封存行情、runner 与已消费授权，用图遍历重算传递连通分量，再重算连续时间边界、250 交易日 purge/embargo 和 65 项显式缺失值来源。</p>
            <p class="public-admin-anchor-boundary">正式 manifest：未创建；正式 feature bundle：未创建；特征连接、目标、训练、奖励、影子、订单、券商和交易：全部关闭。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有待独立校验的完整未信任候选。</p>}>
            <label>
              <span>待校验 attempt</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={eligibleItems()}>
                  {(item) => (
                    <option value={item.attempt.claim.attempt_id}>
                      {item.attempt.claim.attempt_id.slice(0, 12)}… · {item.attempt.claim.claimed_at}
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
              <span>我确认自己不是执行人或完整上游链角色；本操作只形成一次不可变独立校验记录。</span>
            </label>
            <button type="button" disabled={!confirmed() || !selected() || busy()} onClick={() => void validate()}>
              {busy() ? "正在独立重算…" : "独立重算并校验一次"}
            </button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>

          <For each={currentRegistry().items}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>attempt {item.attempt.claim.attempt_id.slice(0, 12)}…</strong>
                  <span>{item.validation?.verdict ?? "waiting_independent_validation"}</span>
                </header>
                <p>执行人 {item.attempt.claim.invoked_by} · output {item.attempt.result.output_sha256?.slice(0, 16)}…</p>
                <Show when={item.validation}>{(validation) => (
                  <>
                    <p>校验人 {validation().validated_by} · {validation().validated_at} · 分量/边界/purge/特征重算 {validation().independent_component_recomputation_match && validation().independent_boundary_recomputation_match && validation().independent_purge_embargo_recomputation_match && validation().independent_feature_recomputation_match ? "全部一致" : "存在不一致"}</p>
                    <Show when={validation().mismatch_reasons.length > 0}>
                      <p class="public-admin-decision-error">{validation().mismatch_reasons.join("；")}</p>
                    </Show>
                  </>
                )}</Show>
                <p class="public-admin-anchor-boundary">通过仅表示未信任候选可重复；正式 manifest / feature bundle / 训练权限仍为关闭。</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
