import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  copyHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreOnce,
  getHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopies,
} from "@/lib/api";
import type { HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRegistry } from "@/lib/types";

const CHECKS = [
  ["exact_current_stage_47_admission_and_complete_chain_confirmed", "精确绑定当前 Stage 47 准入及完整上游责任链"],
  ["claim_first_create_once_and_failure_consumes_confirmed", "先写不可变 claim；成功、失败或中断都消费唯一复制资格"],
  ["exact_official_dataset_copy_without_recompute_repair_or_imputation_confirmed", "只原样复制正式数据集，不重算、不修补、不插补、不改写"],
  ["validation_and_sealed_holdout_targets_remain_withheld_confirmed", "validation 与 sealed holdout 目标继续隐藏"],
  ["independent_post_copy_validation_required_confirmed", "复制完成后仍需完整角色链之外的另一实现独立校验"],
  ["no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed", "本次不登记或运行训练，不奖励、不影子、不下单、不接券商、不交易"],
] as const;

type CheckName = (typeof CHECKS)[number][0];

export function PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checked, setChecked] = createSignal<Record<CheckName, boolean>>(
    Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
  );
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopies();
      setRegistry(next);
      if (!next.items.some((item) => item.admitted_dataset.admission_review.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(
          next.items.find((item) => item.copy_eligible)?.admitted_dataset.admission_review.attempt_id
            ?? next.items[0]?.admitted_dataset.admission_review.attempt_id
            ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练存储复制注册表读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() =>
    registry()?.items.find(
      (item) => item.admitted_dataset.admission_review.attempt_id === selectedAttemptId(),
    ),
  );
  const allConfirmed = createMemo(() => CHECKS.every(([name]) => checked()[name]));

  const submit = async () => {
    const item = selected();
    if (!item || !item.copy_eligible || busy() || !allConfirmed()) return;
    const review = item.admitted_dataset.admission_review;
    const validation = item.admitted_dataset.dataset.validation;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await copyHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreOnce(
        review.attempt_id,
        {
          expected_admission_review_id: review.review_id,
          expected_admission_review_sha256: review.review_sha256,
          expected_output_validation_id: validation.validation_id,
          expected_output_validation_sha256: validation.validation_sha256,
          expected_materialization_id: validation.materialization_id,
          expected_materialization_claim_sha256: validation.materialization_claim_sha256,
          expected_materialization_result_sha256: validation.materialization_result_sha256,
          expected_official_joined_dataset_sha256: validation.official_joined_dataset_sha256,
          expected_source_validation_sha256: validation.source_validation_sha256,
          expected_source_output_sha256: validation.source_output_sha256,
          expected_dataset_content_sha256: validation.dataset_content_sha256,
          expected_dataset_manifest_sha256: validation.dataset_manifest_sha256,
          expected_candidate_set_sha256: validation.candidate_set_sha256,
          expected_rows_sha256: validation.recomputed_rows_sha256,
          expected_excluded_rows_sha256: validation.recomputed_excluded_rows_sha256,
          expected_target_commitments_sha256:
            validation.recomputed_target_commitments_sha256,
          ...checked(),
        },
      );
      setRegistry(next);
      setChecked(
        Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
      );
      setNotice("一次性复制结果已保存。训练仍然关闭，下一步必须完成独立复制后校验。 ");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练存储复制失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="训练存储一次性复制">
          <header>
            <strong>第 48 阶段 · 训练存储一次性复制</strong>
            <span>{currentRegistry().copy_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>Stage 47 已准入</span><strong>{currentRegistry().admitted_dataset_count}</strong></div>
            <div><span>可领取复制</span><strong>{currentRegistry().copy_eligible_count}</strong></div>
            <div><span>已复制</span><strong>{currentRegistry().completed_copy_count}</strong></div>
            <div><span>待复制后校验</span><strong>{currentRegistry().pending_independent_post_copy_validation_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>复制不是训练</strong><span>claim first · create once</span></header>
            <p>系统只把精确正式数据集原样复制到独立、内容寻址目录。复制失败同样消费资格，不能靠重试悄悄修补数据。</p>
            <p class="public-admin-anchor-boundary">训练登记、训练运行、奖励、影子组合、订单、券商和交易：全部关闭。</p>
          </article>

          <Show when={currentRegistry().items.length > 0} fallback={<p>当前没有 Stage 47 已准入的正式 joined dataset。</p>}>
            <label>
              <span>已准入正式 joined dataset</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>{(item) => (
                  <option value={item.admitted_dataset.admission_review.attempt_id}>
                    {item.admitted_dataset.admission_review.attempt_id.slice(0, 12)}… · {item.attempt?.result?.status ?? "待复制"}
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
            <button type="button" disabled={busy() || !selected()?.copy_eligible || !allConfirmed()} onClick={() => void submit()}>
              {busy() ? "正在写入不可变 claim 并复制…" : "领取一次性 claim 并复制"}
            </button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={currentRegistry().items}>{(item) => (
            <article class="public-admin-reward-governance">
              <header>
                <strong>dataset {item.admitted_dataset.admission_review.official_joined_dataset_sha256.slice(0, 16)}…</strong>
                <span>{item.attempt?.result?.status ?? "waiting_create_once_copy"}</span>
              </header>
              <p>准入复核人 {item.admitted_dataset.admission_review.reviewer_id} · 复制人 {item.attempt?.claim.copied_by ?? "—"}</p>
              <Show when={item.attempt?.training_store_dataset}>{(dataset) => (
                <p>rows {dataset().active_row_count} · excluded {dataset().excluded_purge_or_embargo_row_count} · features {dataset().feature_catalog_count} · targets {dataset().target_count}</p>
              )}</Show>
              <Show when={item.attempt?.result?.error}><p>失败原因：{item.attempt?.result?.error}</p></Show>
              <p class="public-admin-anchor-boundary">复制成功仍只表示等待独立逐行逐位校验；训练没有开始。</p>
            </article>
          )}</For>
        </section>
      )}
    </Show>
  );
}
