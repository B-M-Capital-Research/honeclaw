import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidations,
  validateHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
} from "@/lib/api";
import type { HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRegistry } from "@/lib/types";

const CHECKS = [
  "由物化者与完整上游链之外的新角色独立重开并重算，不复用第 45 阶段校验辅助函数",
  "核对精确当前准入候选、claim/result/dataset 指纹、rows、排除项与目标承诺",
  "validation 与 sealed holdout 目标继续隐藏，九项目标只按原始 f64 位模式复核",
  "通过只开放未来训练库复制准入复核；本次不复制、不训练、不奖励、不影子、不交易",
] as const;

export function PublicAdminHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checked, setChecked] = createSignal<boolean[]>(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next =
        await getHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidations();
      setRegistry(next);
      if (!next.items.some((item) => item.materialization.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(
          next.items.find((item) => item.validation_eligible)?.materialization.claim.attempt_id
            ?? next.items[0]?.materialization.claim.attempt_id
            ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "正式 joined dataset 独立校验注册表读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() =>
    registry()?.items.find((item) => item.materialization.claim.attempt_id === selectedAttemptId()),
  );
  const allChecked = createMemo(() => checked().every(Boolean));

  const validate = async () => {
    const item = selected();
    if (!item || !item.validation_eligible || busy() || !allChecked()) return;
    const materialization = item.materialization;
    const sourceValidation = materialization.admitted_candidate.candidate.validation;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await validateHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset(
        materialization.claim.attempt_id,
        {
          expected_materialization_id: materialization.claim.materialization_id,
          expected_materialization_claim_sha256: materialization.claim.claim_sha256,
          expected_materialization_result_sha256: materialization.result.result_sha256,
          expected_official_joined_dataset_sha256:
            materialization.official_joined_dataset.official_joined_dataset_sha256,
          expected_admission_review_sha256:
            materialization.admitted_candidate.admission_review.review_sha256,
          expected_source_validation_sha256: sourceValidation.validation_sha256,
          expected_source_output_sha256: sourceValidation.output_sha256,
          expected_recomputed_rows_sha256: sourceValidation.recomputed_rows_sha256,
          expected_recomputed_excluded_rows_sha256:
            sourceValidation.recomputed_excluded_rows_sha256,
          expected_recomputed_target_commitments_sha256:
            sourceValidation.recomputed_target_commitments_sha256,
          independent_reopen_and_recomputation_confirmed: true,
          exact_current_admitted_candidate_binding_confirmed: true,
          validation_and_sealed_holdout_targets_remain_withheld_confirmed: true,
          no_training_store_copy_training_or_trading_confirmed: true,
        },
      );
      setRegistry(next);
      setChecked(CHECKS.map(() => false));
      setNotice("独立复核记录已不可变保存；通过也只进入未来训练库复制准入复核，不会复制或训练。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "正式 joined dataset 独立校验失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="正式 joined dataset 独立输出校验">
          <header>
            <strong>第 46 阶段 · 正式 joined dataset 独立输出校验</strong>
            <span>{currentRegistry().validation_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待独立校验</span><strong>{currentRegistry().validation_eligible_count}</strong></div>
            <div><span>校验记录</span><strong>{currentRegistry().validation_count}</strong></div>
            <div><span>独立通过</span><strong>{currentRegistry().independently_validated_official_joined_dataset_count}</strong></div>
            <div><span>待复制准入复核</span><strong>{currentRegistry().future_training_store_copy_admission_review_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>独立通过仍不是训练库准入</strong><span>independent reopen · fail closed</span></header>
            <p>校验者自行重开不可变工件，独立重算三层工件指纹、行、排除项和目标承诺，并核对 65 项特征与九项目标可见性。</p>
            <p class="public-admin-anchor-boundary">训练库复制、训练、奖励、影子、订单、券商与交易：全部关闭。</p>
          </article>

          <Show when={currentRegistry().items.length > 0} fallback={<p>当前没有已完成的 Stage 45 正式 joined dataset。</p>}>
            <label>
              <span>正式 joined dataset</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>{(item) => (
                  <option value={item.materialization.claim.attempt_id}>
                    {item.materialization.claim.attempt_id.slice(0, 12)}… · {item.validation ? item.validation.verdict : "待校验"}
                  </option>
                )}</For>
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
            <button type="button" disabled={busy() || !selected()?.validation_eligible || !allChecked()} onClick={() => void validate()}>
              {busy() ? "正在独立重开并逐行逐位复核…" : "独立校验正式 joined dataset"}
            </button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={currentRegistry().items}>{(item) => (
            <article class="public-admin-reward-governance">
              <header>
                <strong>dataset {item.materialization.official_joined_dataset.official_joined_dataset_sha256.slice(0, 16)}…</strong>
                <span>{item.validation?.verdict ?? "waiting_independent_validation"}</span>
              </header>
              <p>物化人 {item.materialization.claim.materialized_by} · 校验人 {item.validation?.validated_by ?? "—"}</p>
              <p>active {item.materialization.official_joined_dataset.active_row_count} · excluded {item.materialization.official_joined_dataset.excluded_purge_or_embargo_row_count} · features {item.materialization.official_joined_dataset.feature_catalog_count} · targets {item.materialization.official_joined_dataset.target_count}</p>
              <Show when={item.validation?.mismatch_reasons.length}><p>失败项：{item.validation?.mismatch_reasons.join("；")}</p></Show>
              <p class="public-admin-anchor-boundary">通过只开放未来训练库复制准入复核；没有复制、训练或交易权限。</p>
            </article>
          )}</For>
        </section>
      )}
    </Show>
  );
}
