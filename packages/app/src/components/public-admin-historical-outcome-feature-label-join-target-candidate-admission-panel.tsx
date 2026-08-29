import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReviews,
  reviewHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmission,
} from "@/lib/api";
import type {
  HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRegistry,
  HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict,
} from "@/lib/types";

const CHECKS = [
  "已重开精确当前候选、Stage 43 校验与完整不可变上游链",
  "数据条目一对一连接且行数、排除数和基数完全一致",
  "65 项特征目录精确一致，没有候选之外的派生特征",
  "点时可用性和显式缺失成立，没有插值、回填或未来信息",
  "正式 split、purge 与 embargo 边界保持不变",
  "仅训练分区可见九项目标原始值",
  "验证分区目标仍只保留承诺、未向候选开放",
  "封存留出目标仍只保留承诺、未向候选开放",
  "九项目标原始 f64 位与逐行承诺已精确绑定",
  "候选不包含动作、仓位、奖励或交易结果语义",
  "准入、create-once 正式物化和物化后独立校验三门分离",
  "训练、奖励、影子、订单、券商和交易权限仍全部关闭",
] as const;

export function PublicAdminHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checked, setChecked] = createSignal<boolean[]>(CHECKS.map(() => false));
  const [verdict, setVerdict] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict>(
      "changes_requested",
    );
  const [rationale, setRationale] = createSignal("");
  const [limitations, setLimitations] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReviews();
      setRegistry(next);
      if (
        !next.items.some(
          (item) => item.candidate.attempt.claim.attempt_id === selectedAttemptId(),
        )
      ) {
        setSelectedAttemptId(next.items[0]?.candidate.attempt.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 候选准入复核读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find(
      (item) => item.candidate.attempt.claim.attempt_id === selectedAttemptId(),
    ),
  );
  const allChecked = createMemo(() => checked().every(Boolean));
  const approving = createMemo(
    () => verdict() === "approved_for_future_create_once_official_joined_dataset_materialization",
  );

  const review = async () => {
    const item = selected();
    if (!item || !item.review_eligible || busy() || !rationale().trim() || !limitations().trim()) return;
    if (approving() && !allChecked()) return;
    const { claim, result } = item.candidate.attempt;
    const validation = item.candidate.validation;
    const values = checked();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmission(
        claim.attempt_id,
        {
          expected_review_id: item.latest_review?.review_id,
          expected_review_sha256: item.latest_review?.review_sha256,
          expected_validation_id: validation.validation_id,
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
          expected_recomputed_excluded_rows_sha256: validation.recomputed_excluded_rows_sha256,
          expected_recomputed_target_commitments_sha256:
            validation.recomputed_target_commitments_sha256,
          verdict: verdict(),
          rationale: rationale(),
          known_limitations: limitations(),
          exact_current_candidate_validation_and_complete_chain_confirmed: values[0],
          exact_one_to_one_entry_join_and_cardinality_confirmed: values[1],
          exact_65_feature_catalog_confirmed: values[2],
          point_in_time_and_explicit_missingness_confirmed: values[3],
          official_split_purge_and_embargo_confirmed: values[4],
          train_only_target_visibility_confirmed: values[5],
          validation_targets_withheld_confirmed: values[6],
          sealed_holdout_targets_withheld_confirmed: values[7],
          exact_nine_raw_f64_bits_and_commitments_confirmed: values[8],
          no_action_position_or_reward_semantics_confirmed: values[9],
          create_once_materialization_and_post_materialization_validation_separation_confirmed:
            values[10],
          downstream_authority_remains_closed_confirmed: values[11],
        },
      );
      setRegistry(next);
      setChecked(CHECKS.map(() => false));
      setRationale("");
      setLimitations("");
      setNotice(
        approving()
          ? "准入已记录：只开放未来 create-once 正式 joined dataset 物化资格；当前没有物化或训练。"
          : "复核意见已不可变追加；候选没有获得正式数据集物化资格。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 候选准入复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="join/target 候选独立准入复核">
          <header>
            <strong>第 44 阶段 · join/target 候选独立准入复核</strong>
            <span>{currentRegistry().admission_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>独立校验候选</span><strong>{currentRegistry().independently_validated_candidate_count}</strong></div>
            <div><span>复核记录</span><strong>{currentRegistry().reviewed_candidate_count}</strong></div>
            <div><span>已准入</span><strong>{currentRegistry().admitted_candidate_count}</strong></div>
            <div><span>修改/拒绝</span><strong>{currentRegistry().changes_requested_or_rejected_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>准入不是正式数据集</strong><span>三门分离</span></header>
            <p>本门只判断精确候选能否进入未来物化。后续仍需独立 create-once 物化，再由另一实现逐行校验正式数据集。</p>
            <p class="public-admin-anchor-boundary">正式 joined dataset：未创建；训练库复制、训练、奖励、影子、订单、券商与交易：全部关闭。</p>
          </article>

          <Show when={currentRegistry().items.length > 0} fallback={<p>当前没有可进入准入复核的 Stage 43 独立校验候选。</p>}>
            <label>
              <span>候选 attempt</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>{(item) => (
                  <option value={item.candidate.attempt.claim.attempt_id}>
                    {item.candidate.attempt.claim.attempt_id.slice(0, 12)}… · {item.latest_review?.verdict ?? "未复核"}
                  </option>
                )}</For>
              </select>
            </label>
            <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict)}><option value="changes_requested">要求修改</option><option value="rejected">拒绝</option><option value="approved_for_future_create_once_official_joined_dataset_materialization">批准未来一次性正式 joined dataset 物化</option></select></label>
            <label><span>复核依据（必填）</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限与偏差（必填）</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
            <For each={CHECKS}>{(label, index) => (
              <label class="public-admin-anchor-check"><input type="checkbox" checked={checked()[index()]} onChange={(event) => { const next = [...checked()]; next[index()] = event.currentTarget.checked; setChecked(next); }} /><span>{label}</span></label>
            )}</For>
            <button type="button" disabled={busy() || !selected()?.review_eligible || !rationale().trim() || !limitations().trim() || (approving() && !allChecked())} onClick={() => void review()}>{busy() ? "正在追加准入复核…" : "追加候选准入复核（不物化）"}</button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={currentRegistry().items}>{(item) => (
            <article class="public-admin-reward-governance">
              <header><strong>attempt {item.candidate.attempt.claim.attempt_id.slice(0, 12)}…</strong><span>{item.latest_review?.verdict ?? "waiting_admission_review"}</span></header>
              <p>validation {item.candidate.validation.validation_id.slice(0, 12)}… · rows {item.candidate.attempt.result.untrusted_candidate_envelope?.active_candidate_row_count ?? 0} · features 65 · targets 9</p>
              <Show when={item.latest_review}>{(record) => <><p>复核人 {record().reviewer_id} · {record().submitted_at}</p><p>{record().rationale}</p><p>局限：{record().known_limitations}</p></>}</Show>
              <p class="public-admin-anchor-boundary">准入只产生未来物化资格；当前正式数据集、训练与全部交易权限仍关闭。</p>
            </article>
          )}</For>
        </section>
      )}
    </Show>
  );
}
