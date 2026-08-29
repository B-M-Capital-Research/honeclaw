import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowExperimentExecutionAttempts,
  getControlledShadowExperimentFirstExecutionAuthorizations,
  invokeControlledShadowExperimentOnce,
} from "@/lib/api";
import type {
  ControlledShadowExperimentExecutionAttemptRegistry,
  ControlledShadowExperimentFirstExecutionAuthorizationRegistry,
  ControlledShadowPointInTimeInputEnvelope,
} from "@/lib/types";

const EXECUTION_CHECKS = [
  "确认先 create-once 写 claim；成功、失败或中断都会永久消费本次授权",
  "确认 Stage 51–79 精确绑定未变化，执行者独立于 Stage 79 复核者和完整上游",
  "确认 claim 落盘后再次复核当前执行二进制摘要，漂移即失败且不得重放",
  "确认输入为点时、只读、内容寻址、白名单来源，不含信号截止后的资料",
  "确认只执行冻结三种子、只做多、仓位/主题/现金和成本约束下的首次初始化",
  "确认本次没有未来观察期，不能生成 21/63/126/252 日收益或晋级结论",
  "确认输出 create-once 且不可信，必须进入 Stage 81 责任链外独立复算",
  "确认不建账本、不写持仓/模型/指标、不回流训练或奖励，不生成订单、不接券商、不交易",
] as const;

export function PublicAdminControlledShadowExperimentExecutionAttemptPanel() {
  const [authorizations, setAuthorizations] =
    createSignal<ControlledShadowExperimentFirstExecutionAuthorizationRegistry>();
  const [registry, setRegistry] =
    createSignal<ControlledShadowExperimentExecutionAttemptRegistry>();
  const [selectedRunnerId, setSelectedRunnerId] = createSignal("");
  const [inputJson, setInputJson] = createSignal("");
  const [checks, setChecks] = createSignal(EXECUTION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const [nextAuthorizations, nextRegistry] = await Promise.all([
        getControlledShadowExperimentFirstExecutionAuthorizations(),
        getControlledShadowExperimentExecutionAttempts(),
      ]);
      setAuthorizations(nextAuthorizations);
      setRegistry(nextRegistry);
      const eligible = nextAuthorizations.items.find((item) => item.execution_attempt_eligible);
      if (!nextAuthorizations.items.some((item) => item.runner.isolated_runner_id === selectedRunnerId())) {
        setSelectedRunnerId(eligible?.runner.isolated_runner_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 80 单次执行记录读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    authorizations()?.items.find(
      (item) => item.runner.isolated_runner_id === selectedRunnerId() && item.execution_attempt_eligible,
    ),
  );
  const parsedInput = createMemo(() => {
    try {
      return JSON.parse(inputJson()) as ControlledShadowPointInTimeInputEnvelope;
    } catch {
      return undefined;
    }
  });
  const disabled = createMemo(
    () => busy() || !selected() || !parsedInput() || checks().some((value) => !value),
  );

  const submit = async () => {
    const current = selected();
    const input = parsedInput();
    if (!current || !input || disabled()) return;
    const runner = current.runner;
    const authorization = current.latest_review;
    if (!authorization) return;
    const contract = runner.implementation.implementation_contract;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await invokeControlledShadowExperimentOnce(runner.isolated_runner_id, {
        expected_authorization_review_id: authorization.review_id,
        expected_authorization_review_sha256: authorization.review_sha256,
        expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256,
        expected_runner_artifact_sha256: runner.runner_artifact_sha256,
        expected_runner_code_revision: runner.runner_code_revision,
        expected_runner_contract_sha256: runner.runner_contract.contract_sha256,
        expected_implementation_sha256: runner.implementation.implementation_sha256,
        expected_implementation_contract_sha256: contract.contract_sha256,
        expected_design_specification_sha256: contract.design_specification_sha256,
        expected_candidate_set_sha256: contract.candidate_set_sha256,
        expected_feature_order_sha256: contract.feature_order_sha256,
        expected_preprocessing_sha256: contract.preprocessing_sha256,
        expected_target_id: contract.target_id,
        expected_frozen_candidate_algorithm_id: contract.frozen_candidate_algorithm_id,
        expected_input_manifest_sha256: input.input_manifest_sha256,
        input,
        claim_first_single_use_and_failure_consumes_confirmed: true,
        exact_stage_51_through_stage_79_binding_confirmed: true,
        current_binary_digest_reverification_after_claim_confirmed: true,
        point_in_time_read_only_content_addressed_allowlisted_input_confirmed: true,
        deterministic_three_seed_long_only_initialization_confirmed: true,
        no_future_performance_or_checkpoint_fabrication_confirmed: true,
        create_once_untrusted_output_requires_independent_validation_confirmed: true,
        no_ledger_position_order_broker_or_trading_confirmed: true,
        no_model_metric_store_feedback_composite_or_reward_confirmed: true,
      });
      setRegistry(next);
      setInputJson("");
      setChecks(EXECUTION_CHECKS.map(() => false));
      setNotice("Stage 79 授权已永久消费。若执行成功，当前仅得到等待 Stage 81 独立复算的不可信初始化观察；若失败也不得重放。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 80 单次隔离影子初始化失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="Stage 80 受控影子单次执行">
          <header>
            <strong>第 80 阶段 · claim-first 单次隔离影子初始化</strong>
            <span>{currentRegistry().execution_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可执行单次资格</span><strong>{currentRegistry().invocation_eligible_authorization_count}</strong></div>
            <div><span>已 claim</span><strong>{currentRegistry().claim_count}</strong></div>
            <div><span>完成初始化</span><strong>{currentRegistry().completed_attempt_count}</strong></div>
            <div><span>待独立复算</span><strong>{currentRegistry().independent_output_validation_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>不可逆门禁</strong><span>claim 后失败也永久消费</span></header>
            <p>上传内容必须由点时输入流水线预先按服务端 canonical schema 生成并计算 input_manifest_sha256。点击后先落盘 claim，再打开输入和冻结模型；不能把历史未来数据当成前向结果。</p>
            <p class="public-admin-anchor-boundary">本阶段只生成虚拟权重观察，不创建真实影子账本或持仓；0 个已观察交易日意味着没有任何收益、回撤、命中率或晋级结论。</p>
          </article>

          <Show when={(authorizations()?.items.filter((item) => item.execution_attempt_eligible).length ?? 0) > 0}>
            <label>
              <span>当前未 claim 的 Stage 79 授权</span>
              <select value={selectedRunnerId()} onChange={(event) => setSelectedRunnerId(event.currentTarget.value)}>
                <For each={authorizations()?.items.filter((item) => item.execution_attempt_eligible) ?? []}>
                  {(item) => <option value={item.runner.isolated_runner_id}>{item.runner.runner_name} · 截止 {item.latest_review?.authorization_valid_until}</option>}
                </For>
              </select>
            </label>
            <label>
              <span>点时只读输入 JSON（包含已计算的 input_manifest_sha256）</span>
              <textarea
                value={inputJson()}
                onInput={(event) => setInputJson(event.currentTarget.value)}
                placeholder='{"schema_version":"controlled_shadow_point_in_time_read_only_input_v1_not_mounted","input_manifest_sha256":"…"}'
              />
            </label>
            <div class="public-admin-decision-checks">
              <For each={EXECUTION_CHECKS}>
                {(label, index) => (
                  <label>
                    <input
                      type="checkbox"
                      checked={checks()[index()]}
                      onChange={(event) =>
                        setChecks((current) => current.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))
                      }
                    />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
              永久消费授权并执行一次初始化
            </button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={currentRegistry().attempts}>
            {(attempt) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>attempt {attempt.claim.attempt_id}</strong>
                  <span>{attempt.result?.status ?? "claim 已落盘 · 中断失败关闭"}</span>
                </header>
                <p>input {attempt.claim.input_manifest_sha256} · executor {attempt.claim.invoked_by}</p>
                <Show when={attempt.result?.bounded_error}><p class="public-admin-error">{attempt.result?.bounded_error}</p></Show>
                <p class="public-admin-anchor-boundary">Stage 81 独立复算完成前，不得把该输出解释为有效组合、收益、评级或交易建议。</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
