import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReviews,
  reviewHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmission,
} from "@/lib/api";
import type {
  HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRegistry,
  HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict,
} from "@/lib/types";

const CHECKS = [
  ["exact_current_stage_46_validation_and_complete_chain_confirmed", "精确绑定当前 Stage 46 独立校验和完整上游责任链"],
  ["immutable_official_dataset_fingerprint_confirmed", "正式数据集、claim 与 result 指纹不可变且完全一致"],
  ["exact_one_to_one_entry_join_and_cardinality_confirmed", "逐 entry 一对一连接及总数、有效行、排除行完全对账"],
  ["exact_65_feature_catalog_confirmed", "65 项特征目录、顺序、schema 与显式缺失口径未漂移"],
  ["point_in_time_and_explicit_missingness_confirmed", "点时可得性与显式缺失继续失败关闭，不补数、不穿越"],
  ["official_split_purge_and_embargo_confirmed", "正式 train/validation/holdout 切分及 purge/embargo 未改变"],
  ["exact_nine_raw_f64_bits_and_commitments_confirmed", "九项原始 f64 位模式及目标承诺逐位一致"],
  ["validation_and_sealed_holdout_targets_remain_withheld_confirmed", "validation 与 sealed holdout 目标继续隐藏"],
  ["schema_contract_suitable_for_future_copy_only_confirmed", "只确认数据合同适合未来复制，不确认模型质量、收益或动作"],
  ["no_action_position_or_reward_semantics_confirmed", "数据中没有买卖动作、仓位或 reward 语义"],
  ["create_once_copy_and_post_copy_validation_remain_separate_confirmed", "未来复制必须 create-once，复制后仍需另一实现独立校验"],
  ["no_copy_training_reward_shadow_order_broker_or_trading_confirmed", "本次不复制、不训练、不奖励、不影子、不下单、不接券商、不交易"],
] as const;

type CheckName = (typeof CHECKS)[number][0];

export function PublicAdminHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checked, setChecked] = createSignal<Record<CheckName, boolean>>(
    Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
  );
  const [verdict, setVerdict] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict>(
      "changes_requested",
    );
  const [rationale, setRationale] = createSignal("");
  const [knownLimitations, setKnownLimitations] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next =
        await getHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReviews();
      setRegistry(next);
      if (!next.items.some((item) => item.dataset.validation.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(
          next.items.find((item) => item.review_eligible)?.dataset.validation.attempt_id
            ?? next.items[0]?.dataset.validation.attempt_id
            ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练存储复制准入复核注册表读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() =>
    registry()?.items.find((item) => item.dataset.validation.attempt_id === selectedAttemptId()),
  );
  const approvalReady = createMemo(() =>
    verdict() !== "approved_for_future_create_once_training_store_copy"
      || CHECKS.every(([name]) => checked()[name]),
  );

  const submit = async () => {
    const item = selected();
    if (!item || !item.review_eligible || busy() || !approvalReady()) return;
    if (!rationale().trim() || !knownLimitations().trim()) {
      setError("请填写复核依据和已知局限；批准不能只靠勾选。 ");
      return;
    }
    const validation = item.dataset.validation;
    const latest = item.latest_review;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmission(
        validation.attempt_id,
        {
          expected_review_id: latest?.review_id,
          expected_review_sha256: latest?.review_sha256,
          expected_materialization_id: validation.materialization_id,
          expected_materialization_claim_sha256: validation.materialization_claim_sha256,
          expected_materialization_result_sha256: validation.materialization_result_sha256,
          expected_official_joined_dataset_sha256: validation.official_joined_dataset_sha256,
          expected_output_validation_id: validation.validation_id,
          expected_output_validation_sha256: validation.validation_sha256,
          expected_admission_review_sha256: validation.admission_review_sha256,
          expected_source_validation_sha256: validation.source_validation_sha256,
          expected_source_output_sha256: validation.source_output_sha256,
          expected_dataset_content_sha256: validation.dataset_content_sha256,
          expected_dataset_manifest_sha256: validation.dataset_manifest_sha256,
          expected_candidate_set_sha256: validation.candidate_set_sha256,
          expected_recomputed_rows_sha256: validation.recomputed_rows_sha256,
          expected_recomputed_excluded_rows_sha256:
            validation.recomputed_excluded_rows_sha256,
          expected_recomputed_target_commitments_sha256:
            validation.recomputed_target_commitments_sha256,
          verdict: verdict(),
          rationale: rationale().trim(),
          known_limitations: knownLimitations().trim(),
          ...checked(),
        },
      );
      setRegistry(next);
      setChecked(
        Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
      );
      setRationale("");
      setKnownLimitations("");
      setNotice("复制准入复核已不可变保存；即使批准，也没有执行复制或训练。 ");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练存储复制准入复核提交失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="训练存储复制独立准入复核">
          <header>
            <strong>第 47 阶段 · 训练存储复制独立准入复核</strong>
            <span>{currentRegistry().admission_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>Stage 46 独立通过</span><strong>{currentRegistry().independently_validated_official_joined_dataset_count}</strong></div>
            <div><span>待复核</span><strong>{currentRegistry().review_eligible_count}</strong></div>
            <div><span>已复核</span><strong>{currentRegistry().reviewed_count}</strong></div>
            <div><span>未来复制候选</span><strong>{currentRegistry().admitted_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>准入不是复制，更不是训练</strong><span>independent review · append only</span></header>
            <p>复核只判断这份精确、不可变、独立校验过的数据合同，是否值得进入下一道 create-once 训练存储复制门禁。</p>
            <p class="public-admin-anchor-boundary">模型训练、奖励设计、影子组合、订单、券商和交易：全部关闭。</p>
          </article>

          <Show when={currentRegistry().items.length > 0} fallback={<p>当前没有 Stage 46 独立通过的正式 joined dataset。</p>}>
            <label>
              <span>正式 joined dataset</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>{(item) => (
                  <option value={item.dataset.validation.attempt_id}>
                    {item.dataset.validation.attempt_id.slice(0, 12)}… · {item.latest_review?.verdict ?? "待复核"}
                  </option>
                )}</For>
              </select>
            </label>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict)}>
                <option value="changes_requested">退回修改</option>
                <option value="rejected">拒绝</option>
                <option value="approved_for_future_create_once_training_store_copy">批准进入未来复制门禁</option>
              </select>
            </label>
            <label><span>复核依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限与偏差</span><textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} /></label>
            <For each={CHECKS}>{([name, label]) => (
              <label class="public-admin-anchor-check">
                <input type="checkbox" checked={checked()[name]} onChange={(event) => setChecked({ ...checked(), [name]: event.currentTarget.checked })} />
                <span>{label}</span>
              </label>
            )}</For>
            <button type="button" disabled={busy() || !selected()?.review_eligible || !approvalReady()} onClick={() => void submit()}>
              {busy() ? "正在保存不可变复核…" : "提交独立复制准入复核"}
            </button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={currentRegistry().items}>{(item) => (
            <article class="public-admin-reward-governance">
              <header>
                <strong>dataset {item.dataset.validation.official_joined_dataset_sha256.slice(0, 16)}…</strong>
                <span>{item.latest_review?.verdict ?? "waiting_independent_admission_review"}</span>
              </header>
              <p>物化人 {item.dataset.materialization.claim.materialized_by} · Stage 46 校验人 {item.dataset.validation.validated_by} · 复制准入复核人 {item.latest_review?.reviewer_id ?? "—"}</p>
              <p>rows {item.dataset.materialization.official_joined_dataset.active_row_count} · excluded {item.dataset.materialization.official_joined_dataset.excluded_purge_or_embargo_row_count} · features {item.dataset.materialization.official_joined_dataset.feature_catalog_count} · targets {item.dataset.materialization.official_joined_dataset.target_count}</p>
              <Show when={item.latest_review}><p>局限：{item.latest_review?.known_limitations}</p></Show>
              <p class="public-admin-anchor-boundary">批准也只开放未来 create-once 复制门禁；没有复制、训练或交易权限。</p>
            </article>
          )}</For>
        </section>
      )}
    </Show>
  );
}
