import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowExperimentOutputValidations,
  validateControlledShadowExperimentOutput,
} from "@/lib/api";
import type {
  ControlledShadowExperimentOutputValidationRegistry,
  ControlledShadowPointInTimeInputEnvelope,
} from "@/lib/types";

const VALIDATION_CHECKS = [
  "确认由 Stage 80 执行者和完整 Stage 51–80 责任链之外的新管理员重开不可变 claim/result，并使用第二实现复算",
  "确认精确核对当前 Stage 51–80 数据集、冻结训练工件、设计、实现、runner、授权和执行绑定",
  "确认校验者独立于 Stage 80 executor 和全部上游登记、复核及执行角色",
  "确认重新提交与 Stage 80 claim 完全相同的内容寻址点时输入，不从 Stage 80 输出反推输入",
  "确认不复用 Stage 80 投影、预测或权重函数，逐位复算 17/29/43 三种子、排序、tie-break 和五重组合上限",
  "确认当前仍为 0 个前向交易日，不生成或暗示收益、回撤、命中率或晋级结论",
  "确认通过后仍是不可信初始观察，只能等待未来前向观察协议登记",
  "确认不建账本/持仓，不写模型/指标，不反馈训练/reward，不生成订单、不接券商、不交易",
] as const;

export function PublicAdminControlledShadowExperimentOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowExperimentOutputValidationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [inputJson, setInputJson] = createSignal("");
  const [checks, setChecks] = createSignal(VALIDATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowExperimentOutputValidations();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.validation_eligible);
      if (!eligible.some((item) => item.attempt.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(eligible[0]?.attempt.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 81 独立输出校验表读取失败");
    }
  };

  onMount(() => void load());

  const eligibleItems = createMemo(
    () => registry()?.items.filter((item) => item.validation_eligible) ?? [],
  );
  const selected = createMemo(() =>
    eligibleItems().find((item) => item.attempt.claim.attempt_id === selectedAttemptId()),
  );
  const parsedInput = createMemo(() => {
    try {
      return JSON.parse(inputJson()) as ControlledShadowPointInTimeInputEnvelope;
    } catch {
      return undefined;
    }
  });
  const exactManifest = createMemo(
    () => parsedInput()?.input_manifest_sha256 === selected()?.attempt.claim.input_manifest_sha256,
  );
  const disabled = createMemo(
    () => busy() || !selected() || !parsedInput() || !exactManifest() || checks().some((value) => !value),
  );

  const submit = async () => {
    const current = selected();
    const input = parsedInput();
    if (!current || !input || disabled()) return;
    const claim = current.attempt.claim;
    const result = current.attempt.result;
    if (!result.output_sha256) {
      setError("该 Stage 80 结果缺少输出 SHA-256，不能独立验证");
      return;
    }
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await validateControlledShadowExperimentOutput(claim.attempt_id, {
        expected_claim_sha256: claim.claim_sha256,
        expected_result_sha256: result.result_sha256,
        expected_output_sha256: result.output_sha256,
        expected_authorization_review_sha256: claim.authorization_review_sha256,
        expected_isolated_runner_spec_sha256: claim.isolated_runner_spec_sha256,
        expected_runner_artifact_sha256: claim.runner_artifact_sha256,
        expected_implementation_contract_sha256: claim.implementation_contract_sha256,
        expected_design_specification_sha256: claim.design_specification_sha256,
        expected_candidate_set_sha256: claim.candidate_set_sha256,
        expected_feature_order_sha256: claim.feature_order_sha256,
        expected_preprocessing_sha256: claim.preprocessing_sha256,
        expected_target_id: claim.target_id,
        expected_frozen_candidate_algorithm_id: claim.frozen_candidate_algorithm_id,
        expected_input_manifest_sha256: claim.input_manifest_sha256,
        input,
        independent_reopen_and_second_implementation_recomputation_confirmed: true,
        exact_current_stage_51_through_stage_80_binding_confirmed: true,
        validator_independent_from_executor_and_complete_prior_chain_confirmed: true,
        exact_content_addressed_point_in_time_input_resubmitted_confirmed: true,
        exact_three_seed_predictions_ranking_and_five_caps_recomputed_confirmed: true,
        zero_forward_sessions_and_no_performance_fabrication_confirmed: true,
        validated_output_remains_untrusted_pending_forward_observation_confirmed: true,
        no_ledger_position_store_feedback_reward_order_broker_or_trading_confirmed: true,
      });
      setRegistry(next);
      setInputJson("");
      setChecks(VALIDATION_CHECKS.map(() => false));
      setNotice("已写入不可变 Stage 81 校验记录。通过只证明初始化输出可复现，仅可等待未来前向观察协议登记。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 81 独立第二实现复算失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="Stage 81 受控影子初始观察独立输出校验">
          <header>
            <strong>第 81 阶段 · 初始影子观察独立第二实现复算</strong>
            <span>{currentRegistry().validation_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待复算</span><strong>{currentRegistry().validation_eligible_count}</strong></div>
            <div><span>验证记录</span><strong>{currentRegistry().validation_count}</strong></div>
            <div><span>逐位通过</span><strong>{currentRegistry().independently_validated_initial_observation_count}</strong></div>
            <div><span>失败关闭</span><strong>{currentRegistry().failed_validation_count}</strong></div>
            <div><span>可登记前向协议</span><strong>{currentRegistry().future_forward_observation_protocol_registration_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>执行路径 ≠ 验证路径</strong><span>同一输入 · 第二实现 · bitwise</span></header>
            <p>校验器要求重新提交原始内容寻址输入，并独立重建预处理、三种子预测、排序、tie-break 与单股/主题/总敞口/现金/持仓数量五重上限。</p>
            <p class="public-admin-anchor-boundary">失败记录不可重放；通过也不产生前向绩效、影子账本、持仓或任何交易权限。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有待独立复算的完整 Stage 80 初始化产物。</p>}>
            <label>
              <span>待验证 Stage 80 attempt</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={eligibleItems()}>
                  {(item) => <option value={item.attempt.claim.attempt_id}>{item.attempt.claim.attempt_id.slice(0, 12)}… · executor {item.attempt.claim.invoked_by}</option>}
                </For>
              </select>
            </label>
            <label>
              <span>同一内容寻址点时输入 JSON（manifest 必须与 Stage 80 claim 完全一致）</span>
              <textarea
                value={inputJson()}
                onInput={(event) => setInputJson(event.currentTarget.value)}
                placeholder='{"schema_version":"controlled_shadow_point_in_time_read_only_input_v1_not_mounted","input_manifest_sha256":"…"}'
              />
            </label>
            <Show when={parsedInput() && !exactManifest()}>
              <p class="public-admin-error">输入 manifest 与 Stage 80 claim 不一致，校验保持关闭。</p>
            </Show>
            <div class="public-admin-decision-checks">
              <For each={VALIDATION_CHECKS}>
                {(label, index) => (
                  <label>
                    <input
                      type="checkbox"
                      checked={checks()[index()]}
                      onChange={(event) => setChecks((current) => current.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))}
                    />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
              {busy() ? "正在用第二实现逐位复算…" : "独立复算三种子、排序与五重上限"}
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
                      <p>{validation().validated_at} · {validation().validated_by} · 复算 {validation().independently_recomputed_allocation_count} 个候选 · gross {validation().independently_recomputed_virtual_gross_exposure_bps} bps · cash {validation().independently_recomputed_virtual_cash_weight_bps} bps</p>
                      <Show when={validation().mismatch_reasons.length > 0}>
                        <p class="public-admin-error">{validation().mismatch_reasons.join("；")}</p>
                      </Show>
                    </>
                  )}
                </Show>
                <p class="public-admin-anchor-boundary">通过只开放未来前向观察协议登记，不是收益证明、实盘组合或交易授权。</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
