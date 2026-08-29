import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializations,
  materializeHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsOnce,
} from "@/lib/api";
import type { HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationRegistry } from "@/lib/types";

const CHECKS = [
  "只精确复制已独立校验并准入的候选，不重算、不补数、不改写",
  "create-once claim 先落盘；成功、失败或中断都会永久消费本次资格",
  "本次不连接结果标签、不定义语义目标，也不训练、奖励、影子或交易",
  "生成的正式 manifest 与 feature bundle 必须由另一实现独立校验后才能继续",
] as const;

export function PublicAdminHistoricalOutcomeTransformationOfficialArtifactMaterializationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checked, setChecked] = createSignal<boolean[]>(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializations();
      setRegistry(next);
      const eligible = next.items.find((item) => item.materialization_eligible);
      if (!next.items.some(
        (item) =>
          item.admitted_candidate.candidate.attempt.claim.attempt_id === selectedAttemptId(),
      )) {
        setSelectedAttemptId(
          eligible?.admitted_candidate.candidate.attempt.claim.attempt_id
            ?? next.items[0]?.admitted_candidate.candidate.attempt.claim.attempt_id
            ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "正式工件物化注册表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find(
      (item) =>
        item.admitted_candidate.candidate.attempt.claim.attempt_id === selectedAttemptId(),
    ),
  );
  const allChecked = createMemo(() => checked().every(Boolean));

  const materialize = async () => {
    const item = selected();
    if (!item || !item.materialization_eligible || busy() || !allChecked()) return;
    const sourceAttempt = item.admitted_candidate.candidate.attempt;
    const validation = item.admitted_candidate.candidate.validation;
    const admission = item.admitted_candidate.admission_review;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next =
        await materializeHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsOnce(
          sourceAttempt.claim.attempt_id,
          {
            expected_admission_review_id: admission.review_id,
            expected_admission_review_sha256: admission.review_sha256,
            expected_validation_sha256: validation.validation_sha256,
            expected_output_sha256: validation.output_sha256,
            expected_dataset_content_sha256: validation.dataset_content_sha256,
            expected_dataset_manifest_sha256: validation.dataset_manifest_sha256,
            expected_candidate_set_sha256: validation.candidate_set_sha256,
            expected_transformation_spec_sha256: validation.transformation_spec_sha256,
            expected_split_specification_sha256: validation.split_specification_sha256,
            expected_feature_specification_sha256: validation.feature_specification_sha256,
            exact_copy_only_confirmed: true,
            create_once_and_failure_consumes_confirmed: true,
            no_join_target_training_or_trading_confirmed: true,
            independent_output_validation_required_confirmed: true,
          },
        );
      setRegistry(next);
      setChecked(CHECKS.map(() => false));
      setNotice(
        "正式 split manifest 与 feature bundle 已 create-once 物化；两者仍待独立输出校验，不能连接或训练。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "正式工件一次性物化失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="离线转换正式工件物化">
          <header>
            <strong>第 34 阶段 · 正式 manifest / feature bundle 一次性物化</strong>
            <span>{currentRegistry().materialization_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>已准入候选</span><strong>{currentRegistry().admitted_candidate_count}</strong></div>
            <div><span>可物化</span><strong>{currentRegistry().materialization_eligible_candidate_count}</strong></div>
            <div><span>已完成</span><strong>{currentRegistry().completed_materialization_count}</strong></div>
            <div><span>失败/不完整</span><strong>{currentRegistry().failed_or_incomplete_materialization_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>正式物化不是训练准入</strong><span>先物化，后独立校验</span></header>
            <p>本阶段只把已验证候选逐字段复制成两个内容寻址工件；不重新搜索、不调用模型、不补缺失，也不接触结果标签。</p>
            <p class="public-admin-anchor-boundary">物化后独立校验：未完成；join、语义目标、训练、奖励、影子、订单、券商与交易：全部关闭。</p>
          </article>

          <Show when={currentRegistry().items.length > 0} fallback={<p>当前没有已独立准入的转换候选。</p>}>
            <label>
              <span>已准入候选</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>{(item) => {
                  const attemptId = item.admitted_candidate.candidate.attempt.claim.attempt_id;
                  return <option value={attemptId}>{attemptId.slice(0, 12)}… · {item.materialization_eligible ? "可物化" : item.attempt?.result?.status ?? "已 claim"}</option>;
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
            <button
              type="button"
              disabled={busy() || !selected()?.materialization_eligible || !allChecked()}
              onClick={() => void materialize()}
            >
              {busy() ? "正在写入不可逆 claim 并物化…" : "一次性物化正式工件（失败也消费）"}
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
                    <p>manifest {result().split_manifest_sha256?.slice(0, 16) ?? "—"}… · bundle {result().feature_bundle_sha256?.slice(0, 16) ?? "—"}…</p>
                    <p>物化人 {item.attempt?.claim.materialized_by} · {result().completed_at}</p>
                    <Show when={result().error}><p>失败：{result().error}</p></Show>
                  </>
                )}</Show>
                <p class="public-admin-anchor-boundary">正式工件仅完成 create-once 复制；独立输出校验与全部下游权限仍关闭。</p>
              </article>
            );
          }}</For>
        </section>
      )}
    </Show>
  );
}
