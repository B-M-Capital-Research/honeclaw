import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializations,
  materializeHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOnce,
} from "@/lib/api";
import type { HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationRegistry } from "@/lib/types";

const CHECKS = [
  "只逐字节复制 Stage 44 精确准入候选，不重算、不修补、不插补、不改写",
  "create-once claim 先落盘；成功、失败或中断都会永久消费本次资格",
  "验证分区与封存留出分区的目标值继续隐藏，只保留原有承诺",
  "正式 joined dataset 落盘后必须由完整上游链之外的新角色独立逐行逐位校验",
  "本次不复制训练库、不训练、不奖励、不建立影子组合，也不生成订单或交易",
] as const;

export function PublicAdminHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checked, setChecked] = createSignal<boolean[]>(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next =
        await getHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializations();
      setRegistry(next);
      if (
        !next.items.some(
          (item) =>
            item.admitted_candidate.candidate.attempt.claim.attempt_id
              === selectedAttemptId(),
        )
      ) {
        const eligible = next.items.find((item) => item.materialization_eligible);
        setSelectedAttemptId(
          eligible?.admitted_candidate.candidate.attempt.claim.attempt_id
            ?? next.items[0]?.admitted_candidate.candidate.attempt.claim.attempt_id
            ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "正式 joined dataset 物化注册表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find(
      (item) =>
        item.admitted_candidate.candidate.attempt.claim.attempt_id
          === selectedAttemptId(),
    ),
  );
  const allChecked = createMemo(() => checked().every(Boolean));

  const materialize = async () => {
    const item = selected();
    if (!item || !item.materialization_eligible || busy() || !allChecked()) return;
    const source = item.admitted_candidate.candidate;
    const admission = item.admitted_candidate.admission_review;
    const claim = source.attempt.claim;
    const result = source.attempt.result;
    const validation = source.validation;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next =
        await materializeHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOnce(
          claim.attempt_id,
          {
            expected_admission_review_id: admission.review_id,
            expected_admission_review_sha256: admission.review_sha256,
            expected_validation_sha256: validation.validation_sha256,
            expected_claim_sha256: claim.claim_sha256,
            expected_result_sha256: result.result_sha256,
            expected_output_sha256: validation.output_sha256,
            expected_authorization_review_sha256: validation.authorization_review_sha256,
            expected_isolated_runner_spec_sha256: validation.isolated_runner_spec_sha256,
            expected_implementation_sha256: validation.implementation_sha256,
            expected_specification_sha256: validation.specification_sha256,
            expected_join_specification_sha256: validation.join_specification_sha256,
            expected_target_specification_sha256: validation.target_specification_sha256,
            expected_split_manifest_sha256: validation.split_manifest_sha256,
            expected_feature_bundle_sha256: validation.feature_bundle_sha256,
            expected_combined_artifact_sha256: validation.combined_artifact_sha256,
            expected_dataset_content_sha256: validation.dataset_content_sha256,
            expected_dataset_manifest_sha256: validation.dataset_manifest_sha256,
            expected_candidate_set_sha256: validation.candidate_set_sha256,
            expected_recomputed_rows_sha256: validation.recomputed_rows_sha256,
            expected_recomputed_excluded_rows_sha256:
              validation.recomputed_excluded_rows_sha256,
            expected_recomputed_target_commitments_sha256:
              validation.recomputed_target_commitments_sha256,
            exact_admitted_candidate_copy_only_confirmed: true,
            create_once_and_failure_consumes_confirmed: true,
            validation_and_sealed_holdout_targets_remain_withheld_confirmed: true,
            independent_post_materialization_validation_required_confirmed: true,
            no_training_reward_shadow_order_broker_or_trading_confirmed: true,
          },
        );
      setRegistry(next);
      setChecked(CHECKS.map(() => false));
      setNotice(
        "正式 joined dataset 已 create-once 落盘；当前仍是未验证工件，不能复制训练库或训练。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "正式 joined dataset 一次性物化失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="正式 joined dataset 一次性物化">
          <header>
            <strong>第 45 阶段 · 正式 joined dataset 一次性物化</strong>
            <span>{currentRegistry().materialization_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>已准入候选</span><strong>{currentRegistry().admitted_candidate_count}</strong></div>
            <div><span>可物化</span><strong>{currentRegistry().materialization_eligible_count}</strong></div>
            <div><span>已完成</span><strong>{currentRegistry().completed_materialization_count}</strong></div>
            <div><span>待独立校验</span><strong>{currentRegistry().pending_independent_validation_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>正式数据集仍不是训练数据</strong><span>claim-first · failure consumes</span></header>
            <p>本门只复制 Stage 44 已准入的精确 rows、排除审计和目标承诺。验证与封存留出的目标值继续隐藏。</p>
            <p class="public-admin-anchor-boundary">物化后独立校验：未完成；训练库复制、训练、奖励、影子、订单、券商与交易：全部关闭。</p>
          </article>

          <Show when={currentRegistry().items.length > 0} fallback={<p>当前没有 Stage 44 已准入候选。</p>}>
            <label>
              <span>已准入候选</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>{(item) => {
                  const attemptId = item.admitted_candidate.candidate.attempt.claim.attempt_id;
                  return <option value={attemptId}>{attemptId.slice(0, 12)}… · {item.materialization_eligible ? "可物化" : item.attempt?.result?.status ?? "claim 已消费"}</option>;
                }}</For>
              </select>
            </label>
            <For each={CHECKS}>{(label, index) => (
              <label class="public-admin-anchor-check">
                <input type="checkbox" checked={checked()[index()]} onChange={(event) => {
                  const next = [...checked()];
                  next[index()] = event.currentTarget.checked;
                  setChecked(next);
                }} />
                <span>{label}</span>
              </label>
            )}</For>
            <button type="button" disabled={busy() || !selected()?.materialization_eligible || !allChecked()} onClick={() => void materialize()}>
              {busy() ? "正在先消费 claim 并物化…" : "一次性物化正式 joined dataset（失败也消费）"}
            </button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>

          <For each={currentRegistry().items}>{(item) => {
            const source = item.admitted_candidate;
            return (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>attempt {source.candidate.attempt.claim.attempt_id.slice(0, 12)}…</strong>
                  <span>{item.attempt?.result?.status ?? (item.attempt ? "claimed_incomplete" : "not_materialized")}</span>
                </header>
                <p>admission {source.admission_review.review_id.slice(0, 12)}… · validation {source.candidate.validation.validation_id.slice(0, 12)}…</p>
                <Show when={item.attempt?.result}>{(result) => (
                  <>
                    <p>dataset {result().official_joined_dataset_sha256?.slice(0, 16) ?? "—"}… · {result().official_joined_dataset_bytes} bytes</p>
                    <p>物化人 {item.attempt?.claim.materialized_by} · {result().completed_at}</p>
                    <Show when={result().error}><p>失败：{result().error}</p></Show>
                  </>
                )}</Show>
                <p class="public-admin-anchor-boundary">正式 joined dataset 只完成 create-once 落盘；物化后独立校验与全部下游权限仍关闭。</p>
              </article>
            );
          }}</For>
        </section>
      )}
    </Show>
  );
}
