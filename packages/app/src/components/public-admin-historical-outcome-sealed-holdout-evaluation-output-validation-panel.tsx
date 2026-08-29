import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeSealedHoldoutEvaluationOutputValidations,
  validateHistoricalOutcomeSealedHoldoutEvaluationOutput,
} from "@/lib/api";
import type { HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRegistry } from "@/lib/types";

const VALIDATION_CHECKS = [
  "由 Stage 71 执行者和完整 Stage 51–71 责任链之外的新管理员，重开不可变 claim/result 并使用第二实现复算",
  "精确核对当前 Stage 51–71 数据集、训练副本、冻结候选、协议、实现、runner、授权与执行绑定",
  "核对先 claim、单次授权已消费、失败也不可重放，不允许重新打开同一留出集",
  "独立重构一个目标的 sealed-holdout 投影，并重新运行同一算法 17/29/43 三个冻结候选",
  "逐位复算三项指标、component bootstrap、三项 Holm 校正、样本门槛与全部预注册阈值",
  "输出通过后仍是不可信确认，只能等待未来裁决复核",
  "不正式选模、不写模型/指标库，不产生 reward、影子仓位、订单、券商访问或交易",
] as const;

export function PublicAdminHistoricalOutcomeSealedHoldoutEvaluationOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checks, setChecks] = createSignal(VALIDATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeSealedHoldoutEvaluationOutputValidations();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.validation_eligible);
      if (!eligible.some((item) => item.attempt.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(eligible[0]?.attempt.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 输出独立验证表读取失败");
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
    const envelope = result.untrusted_confirmation_envelope;
    if (!result.output_sha256 || !envelope) {
      setError("该执行结果缺少输出 SHA-256 或 sealed-holdout envelope，不能独立验证");
      return;
    }
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await validateHistoricalOutcomeSealedHoldoutEvaluationOutput(
        claim.attempt_id,
        {
          expected_claim_sha256: claim.claim_sha256,
          expected_result_sha256: result.result_sha256,
          expected_output_sha256: result.output_sha256,
          expected_authorization_review_sha256: claim.authorization_review_sha256,
          expected_isolated_runner_spec_sha256: claim.isolated_runner_spec_sha256,
          expected_implementation_sha256: claim.implementation_sha256,
          expected_implementation_review_sha256: claim.implementation_review_sha256,
          expected_implementation_independent_audit_sha256:
            claim.implementation_independent_audit_sha256,
          expected_protocol_sha256: claim.protocol_sha256,
          expected_candidate_set_sha256: claim.candidate_set_sha256,
          expected_training_store_dataset_sha256: claim.training_store_dataset_sha256,
          expected_selected_algorithm_three_seed_binding_sha256:
            claim.selected_algorithm_three_seed_binding_sha256,
          expected_sealed_holdout_split_commitment_sha256:
            claim.sealed_holdout_split_commitment_sha256,
          expected_target_id: claim.target_id,
          expected_frozen_candidate_algorithm_id: claim.frozen_candidate_algorithm_id,
          expected_sealed_holdout_projection_sha256:
            envelope.sealed_holdout_projection_sha256,
          expected_feature_order_sha256: envelope.feature_order_sha256,
          expected_preprocessing_sha256: envelope.preprocessing_sha256,
          independent_reopen_and_second_implementation_recomputation_confirmed:
            confirmed[0] as true,
          exact_current_stage_51_through_stage_71_binding_confirmed: confirmed[1] as true,
          claim_first_authorization_consumption_and_no_replay_confirmed: confirmed[2] as true,
          exact_one_target_one_algorithm_three_seed_prediction_recomputation_confirmed:
            confirmed[3] as true,
          exact_three_metrics_component_bootstrap_holm_and_thresholds_bitwise_recomputed_confirmed:
            confirmed[4] as true,
          output_remains_untrusted_pending_future_adjudication_confirmed:
            confirmed[5] as true,
          no_selection_store_reward_shadow_order_broker_or_trading_confirmed:
            confirmed[6] as true,
        },
      );
      setRegistry(next);
      setChecks(VALIDATION_CHECKS.map(() => false));
      setNotice(
        "已写入不可变独立验证记录。通过只说明这份 sealed-holdout 确认可由第二路径逐位复现，仍需未来裁决复核。",
      );
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 输出独立验证失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="sealed-holdout 输出独立复算验证">
          <header>
            <strong>第 72 阶段 · sealed-holdout 输出独立复算</strong>
            <span>{currentRegistry().validation_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待验证</span><strong>{currentRegistry().validation_eligible_count}</strong></div>
            <div><span>验证记录</span><strong>{currentRegistry().validation_count}</strong></div>
            <div><span>逐位通过</span><strong>{currentRegistry().independently_validated_untrusted_confirmation_count}</strong></div>
            <div><span>失败关闭</span><strong>{currentRegistry().failed_validation_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>执行结果 ≠ 独立证据</strong><span>第二路径 · 1 目标 / 3 种子 / 3 检验</span></header>
            <p>验证器不调用 Stage 71 的投影、预测或统计 helper，而是用 Stage 64 的独立实现重新打开精确证据链，逐位核对预测、指标、bootstrap、Holm 和阈值。</p>
            <p class="public-admin-anchor-boundary">通过只开放未来确认结果裁决复核；正式选模、模型/指标库、reward、影子组合和交易全部关闭。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有待独立验证的完整 Stage 71 sealed-holdout 执行产物。</p>}>
            <label>
              <span>待验证 sealed-holdout attempt</span>
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
              {busy() ? "正在独立重构并逐位复算…" : "独立复算三种子、三指标与全部阈值"}
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
                      <p>{validation().validated_at} · {validation().validated_by} · 复算 {validation().recomputed_metric_count} 指标 / {validation().recomputed_candidate_hypothesis_count} 假设</p>
                      <Show when={validation().mismatch_reasons.length > 0}>
                        <p class="public-admin-error">{validation().mismatch_reasons.join("；")}</p>
                      </Show>
                    </>
                  )}
                </Show>
                <p class="public-admin-anchor-boundary">通过只开放未来裁决复核，不是正式选模、收益证明或交易授权。</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
