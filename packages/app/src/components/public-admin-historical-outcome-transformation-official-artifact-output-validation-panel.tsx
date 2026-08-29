import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidations,
  validateHistoricalOutcomeOfflineDatasetTransformationOfficialArtifacts,
} from "@/lib/api";
import type { HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRegistry } from "@/lib/types";

const CHECKS = [
  "我不是物化人、准入复核人、源输出校验人、执行人或完整上游角色",
  "本次会重新读取 claim、result 和两个正式文件，独立重算摘要并逐字段对照源候选",
  "通过只开放未来 join/target 治理规范登记，不执行 join、目标、训练、奖励、影子或交易",
] as const;

export function PublicAdminHistoricalOutcomeTransformationOfficialArtifactOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checked, setChecked] = createSignal<boolean[]>(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next =
        await getHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidations();
      setRegistry(next);
      if (!next.items.some(
        (item) =>
          item.artifact_pair.claim.transformation_attempt_id === selectedAttemptId(),
      )) {
        setSelectedAttemptId(
          next.items.find((item) => item.validation_eligible)?.artifact_pair.claim
            .transformation_attempt_id
            ?? next.items[0]?.artifact_pair.claim.transformation_attempt_id
            ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "正式工件独立校验注册表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find(
      (item) =>
        item.artifact_pair.claim.transformation_attempt_id === selectedAttemptId(),
    ),
  );
  const allChecked = createMemo(() => checked().every(Boolean));

  const validate = async () => {
    const item = selected();
    if (!item || !item.validation_eligible || busy() || !allChecked()) return;
    const pair = item.artifact_pair;
    const admission = pair.admitted_candidate.admission_review;
    const sourceValidation = pair.admitted_candidate.candidate.validation;
    const combined = pair.result.combined_artifact_sha256;
    if (!combined) {
      setError("物化结果缺少 combined artifact SHA-256，不能开始独立校验");
      return;
    }
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next =
        await validateHistoricalOutcomeOfflineDatasetTransformationOfficialArtifacts(
          pair.claim.transformation_attempt_id,
          {
            expected_materialization_id: pair.claim.materialization_id,
            expected_materialization_claim_sha256: pair.claim.claim_sha256,
            expected_materialization_result_sha256: pair.result.result_sha256,
            expected_admission_review_sha256: admission.review_sha256,
            expected_source_validation_sha256: sourceValidation.validation_sha256,
            expected_source_output_sha256: sourceValidation.output_sha256,
            expected_split_manifest_sha256: pair.split_manifest.manifest_sha256,
            expected_feature_bundle_sha256: pair.feature_bundle.feature_bundle_sha256,
            expected_combined_artifact_sha256: combined,
            exact_artifact_pair_binding_confirmed: true,
            independent_validator_confirmed: true,
            no_join_target_training_or_trading_confirmed: true,
          },
        );
      setRegistry(next);
      setChecked(CHECKS.map(() => false));
      setNotice("独立校验记录已不可变保存；即使通过，也只可进入未来 join/target 治理规范登记。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "正式工件独立校验失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="正式工件物化后独立校验">
          <header>
            <strong>第 35 阶段 · 正式工件物化后独立校验</strong>
            <span>{currentRegistry().validation_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待独立校验</span><strong>{currentRegistry().validation_eligible_count}</strong></div>
            <div><span>校验记录</span><strong>{currentRegistry().validation_count}</strong></div>
            <div><span>通过</span><strong>{currentRegistry().independently_validated_artifact_pair_count}</strong></div>
            <div><span>失败关闭</span><strong>{currentRegistry().failed_validation_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>校验通过仍不是训练输入</strong><span>先验证，再治理 join/target</span></header>
            <p>校验器不调用物化器的校验函数，而是独立重算 claim、result、manifest、bundle 和组合摘要，并对照精确源候选。</p>
            <p class="public-admin-anchor-boundary">feature join、语义目标、训练、奖励、影子、订单、券商与交易：全部关闭。</p>
          </article>

          <Show when={currentRegistry().items.length > 0} fallback={<p>当前没有完成物化且可独立校验的正式工件。</p>}>
            <label>
              <span>正式工件对</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>{(item) => {
                  const pair = item.artifact_pair;
                  return <option value={pair.claim.transformation_attempt_id}>{pair.claim.transformation_attempt_id.slice(0, 12)}… · {item.validation_eligible ? "待校验" : item.validation?.verdict ?? "已校验"}</option>;
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
              disabled={busy() || !selected()?.validation_eligible || !allChecked()}
              onClick={() => void validate()}
            >
              {busy() ? "正在独立重算并校验…" : "独立校验正式工件一次"}
            </button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>

          <For each={currentRegistry().items}>{(item) => {
            const pair = item.artifact_pair;
            return (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>attempt {pair.claim.transformation_attempt_id.slice(0, 12)}…</strong>
                  <span>{item.validation?.verdict ?? "waiting_independent_validation"}</span>
                </header>
                <p>manifest {pair.split_manifest.manifest_sha256.slice(0, 16)}… · bundle {pair.feature_bundle.feature_bundle_sha256.slice(0, 16)}…</p>
                <Show when={item.validation}>{(validation) => (
                  <>
                    <p>校验人 {validation().validated_by} · {validation().validated_at}</p>
                    <p>未来 join/target 规范登记资格：{validation().future_feature_label_join_specification_registration_eligible ? "是" : "否"}</p>
                    <Show when={validation().mismatch_reasons.length > 0}>
                      <p>失败原因：{validation().mismatch_reasons.join(" · ")}</p>
                    </Show>
                  </>
                )}</Show>
                <p class="public-admin-anchor-boundary">本阶段只验证正式工件；不执行 join、不定义目标、不训练或交易。</p>
              </article>
            );
          }}</For>
        </section>
      )}
    </Show>
  );
}
