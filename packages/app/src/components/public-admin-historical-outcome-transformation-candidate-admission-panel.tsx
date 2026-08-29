import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReviews,
  reviewHistoricalOutcomeOfflineDatasetTransformationCandidateAdmission,
} from "@/lib/api";
import type {
  HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRegistry,
  HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict,
} from "@/lib/types";

const CHECKS = [
  "已重开精确当前候选、独立校验与完整不可变上游链",
  "传递连通分量完整隔离，没有分量跨越训练、验证和封存留出",
  "连续时间边界与全部候选目标审计可复算且确定",
  "250 交易日 purge/embargo 正确，清理后三个分区均非空",
  "封存留出标签保持隐藏，没有进入候选特征",
  "65 项点时特征白名单、来源、可用时间和变换身份完整",
  "所有缺失均显式记录，没有插值、回填或猜测",
  "结果、标签、未来来源和当前组合状态均已排除",
  "正式产物合同固定为内容寻址、create-once 且只复制精确候选",
  "准入、正式物化和正式产物独立输出校验是三道独立门禁",
  "特征连接、语义目标、训练、奖励、影子、订单、券商和交易全部关闭",
] as const;

export function PublicAdminHistoricalOutcomeTransformationCandidateAdmissionPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checked, setChecked] = createSignal<boolean[]>(CHECKS.map(() => false));
  const [verdict, setVerdict] = createSignal<HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict>(
    "changes_requested",
  );
  const [rationale, setRationale] = createSignal("");
  const [limitations, setLimitations] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReviews();
      setRegistry(next);
      if (!next.items.some((item) => item.candidate.attempt.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(next.items[0]?.candidate.attempt.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "离线转换候选准入复核读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find(
      (item) => item.candidate.attempt.claim.attempt_id === selectedAttemptId(),
    ),
  );
  const allChecked = createMemo(() => checked().every(Boolean));

  const review = async () => {
    const item = selected();
    if (!item || busy() || !rationale().trim() || !limitations().trim()) return;
    if (verdict() === "approved_for_future_create_once_official_artifact_materialization" && !allChecked()) return;
    const { claim, result } = item.candidate.attempt;
    const validation = item.candidate.validation;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const values = checked();
      const next = await reviewHistoricalOutcomeOfflineDatasetTransformationCandidateAdmission(
        claim.attempt_id,
        {
          expected_review_id: item.latest_review?.review_id,
          expected_review_sha256: item.latest_review?.review_sha256,
          expected_validation_id: validation.validation_id,
          expected_validation_sha256: validation.validation_sha256,
          expected_claim_sha256: claim.claim_sha256,
          expected_result_sha256: result.result_sha256,
          expected_output_sha256: validation.output_sha256,
          expected_dataset_content_sha256: validation.dataset_content_sha256,
          expected_dataset_manifest_sha256: validation.dataset_manifest_sha256,
          expected_candidate_set_sha256: validation.candidate_set_sha256,
          expected_transformation_spec_sha256: validation.transformation_spec_sha256,
          expected_split_specification_sha256: validation.split_specification_sha256,
          expected_feature_specification_sha256: validation.feature_specification_sha256,
          verdict: verdict(),
          rationale: rationale(),
          known_limitations: limitations(),
          exact_current_candidate_and_validation_chain_confirmed: values[0],
          transitive_component_isolation_confirmed: values[1],
          deterministic_chronological_boundary_and_full_objective_audit_confirmed: values[2],
          purge_embargo_and_non_empty_partitions_confirmed: values[3],
          sealed_holdout_labels_withheld_confirmed: values[4],
          point_in_time_feature_allowlist_and_provenance_confirmed: values[5],
          explicit_missingness_without_imputation_confirmed: values[6],
          outcome_future_and_current_portfolio_exclusion_confirmed: values[7],
          official_artifact_contract_and_create_once_scope_confirmed: values[8],
          admission_materialization_and_output_validation_separation_confirmed: values[9],
          downstream_authority_remains_closed_confirmed: values[10],
        },
      );
      setRegistry(next);
      setChecked(CHECKS.map(() => false));
      setRationale("");
      setLimitations("");
      setNotice(
        verdict() === "approved_for_future_create_once_official_artifact_materialization"
          ? "准入已记录：只具备未来 create-once 正式产物物化资格，当前没有生成任何正式产物。"
          : "复核意见已不可变追加；候选没有获得正式产物物化资格。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "离线转换候选准入复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="离线转换候选准入复核">
          <header>
            <strong>第 33 阶段 · 离线转换候选独立准入复核</strong>
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
            <header><strong>准入不是正式物化</strong><span>三门分离</span></header>
            <p>本门只审查候选适用性、隔离、点时特征与正式产物合同。后续仍须单独 create-once 物化，再由另一套独立校验器核对正式 manifest 与 feature bundle。</p>
            <p class="public-admin-anchor-boundary">正式 manifest / feature bundle：未创建；join、目标、训练、奖励、影子、订单、券商与交易：全部关闭。</p>
          </article>

          <Show when={currentRegistry().items.length > 0} fallback={<p>当前没有可进入准入复核的独立校验候选。</p>}>
            <label>
              <span>候选 attempt</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>
                  {(item) => <option value={item.candidate.attempt.claim.attempt_id}>{item.candidate.attempt.claim.attempt_id.slice(0, 12)}… · {item.latest_review?.verdict ?? "未复核"}</option>}
                </For>
              </select>
            </label>
            <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict)}><option value="changes_requested">要求修改</option><option value="rejected">拒绝</option><option value="approved_for_future_create_once_official_artifact_materialization">批准未来一次性正式产物物化</option></select></label>
            <label><span>复核依据（必填）</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限与偏差（必填）</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
            <For each={CHECKS}>{(label, index) => (
              <label class="public-admin-anchor-check"><input type="checkbox" checked={checked()[index()]} onChange={(event) => { const next = [...checked()]; next[index()] = event.currentTarget.checked; setChecked(next); }} /><span>{label}</span></label>
            )}</For>
            <button type="button" disabled={busy() || !selected() || !rationale().trim() || !limitations().trim() || (verdict() === "approved_for_future_create_once_official_artifact_materialization" && !allChecked())} onClick={() => void review()}>{busy() ? "正在追加准入复核…" : "追加候选准入复核（不物化）"}</button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={currentRegistry().items}>{(item) => (
            <article class="public-admin-reward-governance">
              <header><strong>attempt {item.candidate.attempt.claim.attempt_id.slice(0, 12)}…</strong><span>{item.latest_review?.verdict ?? "waiting_admission_review"}</span></header>
              <p>validation {item.candidate.validation.validation_id.slice(0, 12)}… · output {item.candidate.validation.output_sha256.slice(0, 16)}…</p>
              <Show when={item.latest_review}>{(reviewRecord) => <><p>复核人 {reviewRecord().reviewer_id} · {reviewRecord().submitted_at}</p><p>{reviewRecord().rationale}</p><p>局限：{reviewRecord().known_limitations}</p></>}</Show>
              <p class="public-admin-anchor-boundary">准入只产生未来物化资格；当前正式产物、训练与全部交易权限仍关闭。</p>
            </article>
          )}</For>
        </section>
      )}
    </Show>
  );
}
