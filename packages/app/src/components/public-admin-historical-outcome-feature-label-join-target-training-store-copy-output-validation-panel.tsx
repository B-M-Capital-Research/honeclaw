import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidations,
  validateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopy,
} from "@/lib/api";
import type { HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRegistry } from "@/lib/types";

const CHECKS = [
  ["independent_reopen_and_recomputation_confirmed", "由复制人和完整上游之外的新角色独立重开并重算全部工件"],
  ["exact_current_stage_47_and_stage_48_binding_confirmed", "精确绑定当前 Stage 47 准入链和 Stage 48 claim/result/副本"],
  ["validation_and_sealed_holdout_targets_remain_withheld_confirmed", "validation 与 sealed holdout 目标继续隐藏"],
  ["no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed", "通过也不登记或运行训练，不奖励、不影子、不下单、不接券商、不交易"],
] as const;

type CheckName = (typeof CHECKS)[number][0];

export function PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checked, setChecked] = createSignal<Record<CheckName, boolean>>(
    Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
  );
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidations();
      setRegistry(next);
      if (!next.items.some((item) => item.copied_dataset.attempt.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(
          next.items.find((item) => item.validation_eligible)?.copied_dataset.attempt.claim.attempt_id
            ?? next.items[0]?.copied_dataset.attempt.claim.attempt_id
            ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练存储副本独立校验注册表读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() => registry()?.items.find(
    (item) => item.copied_dataset.attempt.claim.attempt_id === selectedAttemptId(),
  ));
  const allConfirmed = createMemo(() => CHECKS.every(([name]) => checked()[name]));

  const submit = async () => {
    const item = selected();
    if (!item || !item.validation_eligible || busy() || !allConfirmed()) return;
    const claim = item.copied_dataset.attempt.claim;
    const result = item.copied_dataset.attempt.result;
    const dataset = item.copied_dataset.attempt.training_store_dataset;
    if (!result || !dataset) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await validateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopy(
        claim.attempt_id,
        {
          expected_copy_id: claim.copy_id,
          expected_copy_claim_sha256: claim.claim_sha256,
          expected_copy_result_sha256: result.result_sha256,
          expected_training_store_dataset_sha256: dataset.training_store_dataset_sha256,
          expected_admission_review_sha256: claim.admission_review_sha256,
          expected_output_validation_sha256: claim.output_validation_sha256,
          expected_official_joined_dataset_sha256: claim.official_joined_dataset_sha256,
          expected_rows_sha256: item.copied_dataset.admitted_dataset.dataset.validation.recomputed_rows_sha256,
          expected_excluded_rows_sha256: item.copied_dataset.admitted_dataset.dataset.validation.recomputed_excluded_rows_sha256,
          expected_target_commitments_sha256: item.copied_dataset.admitted_dataset.dataset.validation.recomputed_target_commitments_sha256,
          ...checked(),
        },
      );
      setRegistry(next);
      setChecked(Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>);
      setNotice("独立校验记录已不可覆盖保存。通过只进入未来训练登记准入复核，训练仍未登记或启动。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练存储副本独立校验失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="训练存储副本独立校验">
          <header>
            <strong>第 49 阶段 · 训练存储副本独立校验</strong>
            <span>{currentRegistry().validation_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待独立校验</span><strong>{currentRegistry().validation_eligible_count}</strong></div>
            <div><span>已形成记录</span><strong>{currentRegistry().validation_count}</strong></div>
            <div><span>独立通过</span><strong>{currentRegistry().independently_validated_training_store_copy_count}</strong></div>
            <div><span>待训练登记复核</span><strong>{currentRegistry().future_training_registration_review_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>复制一致 ≠ 模型有效</strong><span>independent · bit exact</span></header>
            <p>校验器重新计算 claim、result、副本、rows、excluded rows 与目标承诺指纹，并与精确正式数据集逐行逐位核对。</p>
            <p class="public-admin-anchor-boundary">通过只证明复制一致；训练登记、训练授权、训练运行、奖励、影子组合、订单、券商和交易全部关闭。</p>
          </article>

          <Show when={currentRegistry().items.length > 0} fallback={<p>当前没有已完成并等待独立校验的训练存储副本。</p>}>
            <label>
              <span>已完成训练存储副本</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>{(item) => (
                  <option value={item.copied_dataset.attempt.claim.attempt_id}>
                    {item.copied_dataset.attempt.claim.attempt_id.slice(0, 12)}… · {item.validation?.verdict ?? "待独立校验"}
                  </option>
                )}</For>
              </select>
            </label>
            <For each={CHECKS}>{([name, label]) => (
              <label class="public-admin-anchor-check">
                <input type="checkbox" checked={checked()[name]} onChange={(event) => setChecked({ ...checked(), [name]: event.currentTarget.checked })} />
                <span>{label}</span>
              </label>
            )}</For>
            <button type="button" disabled={busy() || !selected()?.validation_eligible || !allConfirmed()} onClick={() => void submit()}>
              {busy() ? "正在独立重开并逐位校验…" : "写入一次性独立校验记录"}
            </button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={currentRegistry().items}>{(item) => (
            <article class="public-admin-reward-governance">
              <header>
                <strong>copy {item.copied_dataset.attempt.claim.copy_id.slice(0, 16)}…</strong>
                <span>{item.validation?.verdict ?? "waiting_independent_validation"}</span>
              </header>
              <p>复制人 {item.copied_dataset.attempt.claim.copied_by} · 校验人 {item.validation?.validated_by ?? "—"}</p>
              <p>rows {item.copied_dataset.attempt.training_store_dataset?.active_row_count ?? 0} · features {item.copied_dataset.attempt.training_store_dataset?.feature_catalog_count ?? 0} · targets {item.copied_dataset.attempt.training_store_dataset?.target_count ?? 0}</p>
              <Show when={item.validation?.mismatch_reasons.length}><p>不一致：{item.validation?.mismatch_reasons.join("；")}</p></Show>
              <p class="public-admin-anchor-boundary">训练登记与训练运行：未开放；奖励、影子、订单、券商和交易：全部关闭。</p>
            </article>
          )}</For>
        </section>
      )}
    </Show>
  );
}
