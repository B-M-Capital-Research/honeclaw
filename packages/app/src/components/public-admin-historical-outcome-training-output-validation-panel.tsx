import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeTrainingOutputValidations,
  validateHistoricalOutcomeTrainingOutput,
} from "@/lib/api";
import type { HistoricalOutcomeTrainingOutputValidationRegistry } from "@/lib/types";

const VALIDATION_CHECKS = [
  "由训练执行者和完整上游链之外的新管理员，重开不可变 claim/result 并使用第二套实现复算",
  "精确绑定 Stage 51–57 的训练副本、冻结套件、实现、runner、授权与执行结果",
  "独立复算 65 项预处理、9 个模型工件和 81 项 train-only 诊断，并按 f64 位模式逐项核对",
  "validation 与 sealed holdout 标签继续隐藏；本阶段绝不读取、推断或用来选模",
  "不选模、不写模型/指标库，不产生 reward、影子仓位、订单、券商访问或交易",
] as const;

export function PublicAdminHistoricalOutcomeTrainingOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeTrainingOutputValidationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checks, setChecks] = createSignal(VALIDATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeTrainingOutputValidations();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.validation_eligible);
      if (!eligible.some((item) => item.attempt.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(eligible[0]?.attempt.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练产物独立验证注册表读取失败");
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
    if (!result.output_sha256) {
      setError("该执行结果缺少输出 SHA-256，不能独立验证");
      return;
    }
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await validateHistoricalOutcomeTrainingOutput(claim.attempt_id, {
        expected_claim_sha256: claim.claim_sha256,
        expected_result_sha256: result.result_sha256,
        expected_output_sha256: result.output_sha256,
        expected_authorization_review_sha256: claim.authorization_review_sha256,
        expected_isolated_runner_spec_sha256: claim.isolated_runner_spec_sha256,
        expected_implementation_sha256: claim.implementation_sha256,
        expected_implementation_review_sha256: claim.implementation_review_sha256,
        expected_suite_specification_sha256: claim.suite_specification_sha256,
        expected_training_store_dataset_sha256: claim.training_store_dataset_sha256,
        expected_rows_sha256: claim.rows_sha256,
        expected_excluded_rows_sha256: claim.excluded_rows_sha256,
        expected_target_commitments_sha256: claim.target_commitments_sha256,
        independent_reopen_and_second_implementation_recomputation_confirmed: confirmed[0],
        exact_current_stage_51_through_stage_57_binding_confirmed: confirmed[1],
        all_nine_model_artifacts_and_eighty_one_diagnostics_bitwise_recomputed_confirmed:
          confirmed[2],
        validation_and_sealed_holdout_targets_remain_withheld_confirmed: confirmed[3],
        no_model_selection_store_reward_shadow_order_broker_or_trading_confirmed: confirmed[4],
      });
      setRegistry(next);
      setChecks(VALIDATION_CHECKS.map(() => false));
      setNotice(
        "已写入不可变独立验证记录。通过只说明 train-only 产物可重现；仍未读取 validation 标签，也未完成选模或模型有效性验证。",
      );
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练产物独立验证失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="训练产物独立复算验证">
          <header>
            <strong>第 58 阶段 · 训练产物独立复算验证</strong>
            <span>{currentRegistry().validation_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待验证</span><strong>{currentRegistry().validation_eligible_count}</strong></div>
            <div><span>验证记录</span><strong>{currentRegistry().validation_count}</strong></div>
            <div><span>逐位通过</span><strong>{currentRegistry().independently_validated_train_only_artifact_envelope_count}</strong></div>
            <div><span>失败关闭</span><strong>{currentRegistry().failed_validation_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>可重现 ≠ 有效，更不等于可交易</strong><span>第二实现 · 全量逐位核对</span></header>
            <p>验证器不调用 Stage 57 的拟合或诊断私有函数，独立复算全部 9 个工件与 81 项 train-only 诊断。任一权重、阈值、叶值或指标相差一个位模式都失败关闭。</p>
            <p class="public-admin-anchor-boundary">validation/holdout 标签、选模、模型库、指标库、reward、影子组合、订单、券商和交易全部关闭。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有待独立验证的完整 Stage 57 训练产物。</p>}>
            <label>
              <span>待验证训练 attempt</span>
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
              {busy() ? "正在独立复算并逐位核对…" : "独立复算 9 个工件与 81 项诊断"}
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
                      <p>{validation().validated_at} · {validation().validated_by} · 复算 {validation().recomputed_model_artifact_count} 个工件 / {validation().recomputed_fit_diagnostic_count} 项诊断</p>
                      <Show when={validation().mismatch_reasons.length > 0}>
                        <p class="public-admin-error">{validation().mismatch_reasons.join("；")}</p>
                      </Show>
                    </>
                  )}
                </Show>
                <p class="public-admin-anchor-boundary">通过只开放未来 validation 评估实现登记资格；当前仍未选模，也没有投资或交易权限。</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
