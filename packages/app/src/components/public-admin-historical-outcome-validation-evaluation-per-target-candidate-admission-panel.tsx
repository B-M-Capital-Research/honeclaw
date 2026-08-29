import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReviews,
  reviewHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmission,
} from "@/lib/api";
import type {
  HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRegistry,
  HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–64 全链与 Stage 64 独立复算记录",
  "该目标单独具有三种算法 × 三个冻结种子的九项指标，且无重复或缺项",
  "已逐项核对该目标的证据状态和全部预注册门槛",
  "已核对建议算法、三种子全部通过与 validation MAE 中位数",
  "未用综合分、其他目标或平均表现掩盖该目标失败",
  "sealed holdout 特征和标签仍未读取",
  "下一步只是 sealed-holdout 评估协议复核，不是留出集执行",
  "不正式选模、不写模型/指标库，不产生 reward、影子、订单、券商访问或交易",
] as const;

export function PublicAdminHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRegistry>();
  const [selectedKey, setSelectedKey] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict>(
      "changes_requested",
    );
  const [rationale, setRationale] = createSignal("");
  const [limitations, setLimitations] = createSignal("");
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const keyFor = (attemptId: string, targetId: string) => `${attemptId}::${targetId}`;

  const load = async () => {
    try {
      const next =
        await getHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReviews();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.review_eligible);
      if (
        !eligible.some(
          (item) =>
            keyFor(item.candidate.source.attempt.claim.attempt_id, item.candidate.target_id) ===
            selectedKey(),
        )
      ) {
        const first = eligible[0];
        setSelectedKey(
          first
            ? keyFor(first.candidate.source.attempt.claim.attempt_id, first.candidate.target_id)
            : "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "逐目标候选准入复核表读取失败");
    }
  };

  onMount(() => void load());

  const eligibleItems = createMemo(
    () => registry()?.items.filter((item) => item.review_eligible) ?? [],
  );
  const selected = createMemo(() =>
    eligibleItems().find(
      (item) =>
        keyFor(item.candidate.source.attempt.claim.attempt_id, item.candidate.target_id) ===
        selectedKey(),
    ),
  );
  const approving = createMemo(
    () => verdict() === "admitted_for_future_sealed_holdout_evaluation_protocol_review",
  );
  const disabled = createMemo(
    () =>
      busy() ||
      !selected() ||
      !rationale().trim() ||
      !limitations().trim() ||
      checks().some((value) => !value) ||
      (approving() && !selected()?.candidate.recommendation_admissible),
  );

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) =>
      current.map((value, currentIndex) => (currentIndex === index ? checked : value)),
    );
  };

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const candidate = item.candidate;
    const source = candidate.source;
    const validation = source.validation;
    const claim = source.attempt.claim;
    const result = source.attempt.result;
    if (!result.output_sha256) {
      setError("Stage 63 结果缺少输出 SHA-256，不能准入");
      return;
    }
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next =
        await reviewHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmission(
          claim.attempt_id,
          candidate.target_id,
          {
            expected_review_id: item.latest_review?.review_id,
            expected_review_sha256: item.latest_review?.review_sha256,
            expected_output_validation_id: validation.validation_id,
            expected_output_validation_sha256: validation.validation_sha256,
            expected_claim_sha256: validation.claim_sha256,
            expected_result_sha256: validation.result_sha256,
            expected_output_sha256: result.output_sha256,
            expected_authorization_review_sha256: validation.authorization_review_sha256,
            expected_isolated_runner_spec_sha256: validation.isolated_runner_spec_sha256,
            expected_implementation_sha256: validation.implementation_sha256,
            expected_implementation_review_sha256: validation.implementation_review_sha256,
            expected_candidate_set_sha256: validation.candidate_set_sha256,
            expected_upstream_validation_sha256: validation.upstream_validation_sha256,
            expected_training_store_dataset_sha256: validation.training_store_dataset_sha256,
            expected_validation_projection_sha256: validation.validation_projection_sha256,
            expected_target_bundle_sha256: candidate.target_bundle_sha256,
            expected_recommendation_sha256: candidate.recommendation_sha256,
            verdict: verdict(),
            rationale: rationale().trim(),
            known_limitations: limitations().trim(),
            exact_current_stage_51_through_stage_64_binding_confirmed: confirmed[0] as true,
            exact_target_only_nine_metrics_three_algorithms_three_seeds_confirmed:
              confirmed[1] as true,
            target_evidence_status_and_thresholds_confirmed: confirmed[2] as true,
            recommended_algorithm_and_three_seed_median_confirmed: confirmed[3] as true,
            no_cross_target_composite_or_masking_confirmed: confirmed[4] as true,
            sealed_holdout_remains_unread_confirmed: confirmed[5] as true,
            next_gate_is_protocol_review_not_holdout_execution_confirmed: confirmed[6] as true,
            no_selection_store_reward_shadow_order_broker_or_trading_confirmed:
              confirmed[7] as true,
          },
        );
      setRegistry(next);
      setRationale("");
      setLimitations("");
      setChecks(REVIEW_CHECKS.map(() => false));
      setVerdict("changes_requested");
      setNotice(
        approving()
          ? "该目标已单独准入，但只可进入未来 sealed-holdout 评估协议复核；留出集仍未开放。"
          : "已追加保存该目标的复核结论；其他目标不会被这一结论带过。",
      );
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "逐目标候选准入复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="逐目标候选准入复核">
          <header>
            <strong>第 65 阶段 · 逐目标候选准入复核</strong>
            <span>{currentRegistry().admission_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>目标候选</span><strong>{currentRegistry().target_candidate_count}</strong></div>
            <div><span>已复核</span><strong>{currentRegistry().reviewed_target_count}</strong></div>
            <div><span>已准入</span><strong>{currentRegistry().admitted_target_count}</strong></div>
            <div><span>证据不足</span><strong>{currentRegistry().insufficient_evidence_target_count}</strong></div>
            <div><span>三种子未全过</span><strong>{currentRegistry().no_candidate_passed_target_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>九个目标，九道独立门</strong><span>不做综合分</span></header>
            <p>每个目标只由自己的三种算法 × 三个冻结种子、证据门槛和建议决定；一个目标失败，不能被其他目标的优秀结果抵消。</p>
            <p class="public-admin-anchor-boundary">准入 ≠ 正式选模，也不开放 sealed holdout；下一步仍先冻结并独立审查留出集评估协议。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有待复核的逐目标候选。</p>}>
            <label>
              <span>待复核目标</span>
              <select value={selectedKey()} onChange={(event) => setSelectedKey(event.currentTarget.value)}>
                <For each={eligibleItems()}>
                  {(item) => (
                    <option value={keyFor(item.candidate.source.attempt.claim.attempt_id, item.candidate.target_id)}>
                      {item.candidate.target_id} · {item.candidate.recommendation.status}
                    </option>
                  )}
                </For>
              </select>
            </label>
            <Show when={selected()}>
              {(item) => (
                <article class="public-admin-reward-governance">
                  <header><strong>{item().candidate.target_id}</strong><span>{item().candidate.recommendation_admissible ? "可审议准入" : "不可准入"}</span></header>
                  <p>九项指标：{item().candidate.metrics.length} · 建议：{item().candidate.recommendation.status} · 算法：{item().candidate.recommendation.recommended_algorithm ?? "无"}</p>
                  <p>{item().candidate.recommendation.rationale}</p>
                </article>
              )}
            </Show>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict)}>
                <option value="changes_requested">要求补证或修正</option>
                <option value="rejected">拒绝该目标候选</option>
                <option value="admitted_for_future_sealed_holdout_evaluation_protocol_review" disabled={!selected()?.candidate.recommendation_admissible}>准入未来留出集协议复核</option>
              </select>
            </label>
            <label><span>复核依据</span><textarea maxlength={2400} value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限与偏差</span><textarea maxlength={2400} value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
            <div class="public-admin-decision-checks">
              <For each={REVIEW_CHECKS}>
                {(label, index) => (
                  <label>
                    <input type="checkbox" checked={checks()[index()]} onChange={(event) => toggleCheck(index(), event.currentTarget.checked)} />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在保存逐目标审计…" : "提交这一目标的独立复核"}</button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={currentRegistry().items}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header><strong>{item.candidate.target_id}</strong><span>{item.latest_review?.verdict ?? item.candidate.recommendation.status}</span></header>
                <p>目标包 {item.candidate.target_bundle_sha256.slice(0, 12)}… · 9 指标 {item.candidate.exact_nine_metrics_three_algorithms_three_seeds ? "完整" : "异常"}</p>
                <p class="public-admin-anchor-boundary">{item.per_target_candidate_admitted ? "仅获未来 sealed-holdout 评估协议复核资格" : "未准入；不会读取 sealed holdout 或形成正式选择"}</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
