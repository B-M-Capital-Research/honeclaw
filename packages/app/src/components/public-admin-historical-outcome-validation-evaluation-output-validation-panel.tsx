import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeValidationEvaluationOutputValidations,
  validateHistoricalOutcomeValidationEvaluationOutput,
} from "@/lib/api";
import type { HistoricalOutcomeValidationEvaluationOutputValidationRegistry } from "@/lib/types";

const VALIDATION_CHECKS = [
  "由 Stage 63 执行者和完整上游之外的新管理员，重开不可变 claim/result 并使用第二套实现复算",
  "精确绑定 Stage 51–63 的训练副本、九候选、冻结统计合同、runner、授权与执行结果",
  "独立重构 validation-only 投影，并重新运行精确九候选预测",
  "逐位复算 81 项指标、54 项 component bootstrap/Holm 检验和 9 项逐目标建议",
  "sealed holdout 不进入验证器输入，也不得读取其特征或标签",
  "不正式选模、不写模型/指标库，不产生 reward、影子仓位、订单、券商访问或交易",
] as const;

export function PublicAdminHistoricalOutcomeValidationEvaluationOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeValidationEvaluationOutputValidationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checks, setChecks] = createSignal(VALIDATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeValidationEvaluationOutputValidations();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.validation_eligible);
      if (!eligible.some((item) => item.attempt.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(eligible[0]?.attempt.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "validation 评估输出独立验证表读取失败");
    }
  };

  onMount(() => void load());

  const eligibleItems = createMemo(
    () => registry()?.items.filter((item) => item.validation_eligible) ?? [],
  );
  const selected = createMemo(() =>
    eligibleItems().find((item) => item.attempt.claim.attempt_id === selectedAttemptId()),
  );
  const disabled = createMemo(
    () => busy() || !selected() || checks().some((value) => !value),
  );

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) =>
      current.map((value, currentIndex) => (currentIndex === index ? checked : value)),
    );
  };

  const submit = async () => {
    const current = selected();
    if (!current || disabled()) return;
    const claim = current.attempt.claim;
    const result = current.attempt.result;
    const envelope = result.untrusted_evaluation_envelope;
    if (!result.output_sha256 || !envelope) {
      setError("该执行结果缺少输出 SHA-256 或评估 envelope，不能独立验证");
      return;
    }
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await validateHistoricalOutcomeValidationEvaluationOutput(claim.attempt_id, {
        expected_claim_sha256: claim.claim_sha256,
        expected_result_sha256: result.result_sha256,
        expected_output_sha256: result.output_sha256,
        expected_authorization_review_sha256: envelope.authorization_review_sha256,
        expected_isolated_runner_spec_sha256: claim.isolated_runner_spec_sha256,
        expected_implementation_sha256: claim.implementation_sha256,
        expected_implementation_review_sha256: claim.implementation_review_sha256,
        expected_candidate_set_sha256: claim.candidate_set_sha256,
        expected_upstream_validation_sha256: claim.upstream_validation_sha256,
        expected_upstream_output_sha256: claim.upstream_output_sha256,
        expected_training_store_dataset_sha256: claim.training_store_dataset_sha256,
        expected_rows_sha256: claim.rows_sha256,
        expected_excluded_rows_sha256: claim.excluded_rows_sha256,
        expected_target_commitments_sha256: claim.target_commitments_sha256,
        expected_validation_projection_sha256: envelope.validation_projection_sha256,
        expected_feature_order_sha256: envelope.feature_order_sha256,
        expected_preprocessing_sha256: envelope.preprocessing_sha256,
        independent_reopen_and_second_implementation_recomputation_confirmed: confirmed[0] as true,
        exact_current_stage_51_through_stage_63_binding_confirmed: confirmed[1] as true,
        exact_validation_projection_and_nine_candidate_predictions_confirmed: confirmed[2] as true,
        all_eighty_one_metrics_fifty_four_hypotheses_and_nine_recommendations_bitwise_recomputed_confirmed:
          confirmed[3] as true,
        sealed_holdout_remains_unread_confirmed: confirmed[4] as true,
        no_selection_store_reward_shadow_order_broker_or_trading_confirmed: confirmed[5] as true,
      });
      setRegistry(next);
      setChecks(VALIDATION_CHECKS.map(() => false));
      setNotice(
        "已写入不可变独立验证记录。通过只说明这份 validation 评估可逐位复现；仍不是正式选模，也没有投资或交易权限。",
      );
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "validation 评估输出独立验证失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="validation 评估输出独立复算验证">
          <header>
            <strong>第 64 阶段 · validation 评估输出独立复算</strong>
            <span>{currentRegistry().validation_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待验证</span><strong>{currentRegistry().validation_eligible_count}</strong></div>
            <div><span>验证记录</span><strong>{currentRegistry().validation_count}</strong></div>
            <div><span>逐位通过</span><strong>{currentRegistry().independently_validated_untrusted_envelope_count}</strong></div>
            <div><span>失败关闭</span><strong>{currentRegistry().failed_validation_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>评估可复现 ≠ 模型已有效</strong><span>第二路径 · 81 / 54 / 9</span></header>
            <p>验证器不调用 Stage 63 的投影、预测或统计 helper，重新打开精确证据链并逐位核对全部指标、统计检验与逐目标建议。</p>
            <p class="public-admin-anchor-boundary">通过只开放未来逐目标准入复核；holdout、正式选模、模型/指标库、reward、影子组合和交易全部关闭。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有待独立验证的完整 Stage 63 评估产物。</p>}>
            <label>
              <span>待验证 evaluation attempt</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={eligibleItems()}>
                  {(item) => <option value={item.attempt.claim.attempt_id}>{item.attempt.claim.attempt_id.slice(0, 12)}… · {item.attempt.result.completed_at}</option>}
                </For>
              </select>
            </label>
            <div class="public-admin-decision-checks">
              <For each={VALIDATION_CHECKS}>
                {(label, index) => (
                  <label>
                    <input type="checkbox" checked={checks()[index()]} onChange={(event) => toggleCheck(index(), event.currentTarget.checked)} />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
              {busy() ? "正在独立重构并逐位复算…" : "独立复算 81 指标、54 检验与 9 建议"}
            </button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={currentRegistry().items}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>attempt {item.attempt.claim.attempt_id.slice(0, 12)}…</strong>
                  <span>{item.validation?.verdict ?? "等待独立复算"}</span>
                </header>
                <Show when={item.validation}>
                  {(validation) => (
                    <>
                      <p>{validation().validated_at} · {validation().validated_by} · 复算 {validation().recomputed_metric_count} 指标 / {validation().recomputed_candidate_hypothesis_count} 检验 / {validation().recomputed_per_target_recommendation_count} 建议</p>
                      <Show when={validation().mismatch_reasons.length > 0}>
                        <p class="public-admin-error">{validation().mismatch_reasons.join("；")}</p>
                      </Show>
                    </>
                  )}
                </Show>
                <p class="public-admin-anchor-boundary">通过只开放未来逐目标准入复核，不是正式选模、收益证明或交易授权。</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
