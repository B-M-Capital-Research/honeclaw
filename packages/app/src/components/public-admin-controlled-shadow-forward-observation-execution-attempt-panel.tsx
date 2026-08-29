import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowForwardObservationExecutionAttempts,
  getControlledShadowForwardObservationFirstExecutionAuthorizations,
  invokeControlledShadowForwardObservationOnce,
} from "@/lib/api";
import type {
  ControlledShadowForwardObservationExecutionAttemptRegistry,
  ControlledShadowForwardObservationFirstExecutionAuthorizationRegistry,
  ControlledShadowForwardObservationInitializationManifest,
} from "@/lib/types";

const EXECUTION_CHECKS = [
  "确认先持久化 claim；成功、失败或中断都会永久消费 Stage 87 授权",
  "确认精确绑定当前 Stage 51–87，执行者独立于 Stage 87 复核者和完整上游责任链",
  "确认 claim 落盘后重新计算当前二进制 SHA-256，漂移即失败关闭且不得重放",
  "确认 observation-not-before、自然前向和禁止回填边界未变化",
  "确认使用官方交易日历，并要求证券与 SPY 同步观察",
  "确认初始化清单没有行情行、历史收益或未来结果",
  "确认初始化收据不可信，必须进入未来 Stage 89 责任链外独立验证",
  "确认不实例化持久 runtime、不挂载、不访问行情、不开始观察、不建账、不写持仓或绩效",
  "确认不写模型/指标、不回流训练或奖励，不生成订单、不接券商、不交易",
  "确认没有把未确认的 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowForwardObservationExecutionAttemptPanel() {
  const [authorizations, setAuthorizations] =
    createSignal<ControlledShadowForwardObservationFirstExecutionAuthorizationRegistry>();
  const [registry, setRegistry] =
    createSignal<ControlledShadowForwardObservationExecutionAttemptRegistry>();
  const [selectedRunnerId, setSelectedRunnerId] = createSignal("");
  const [manifestJson, setManifestJson] = createSignal("");
  const [checks, setChecks] = createSignal(EXECUTION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const [nextAuthorizations, nextRegistry] = await Promise.all([
        getControlledShadowForwardObservationFirstExecutionAuthorizations(),
        getControlledShadowForwardObservationExecutionAttempts(),
      ]);
      setAuthorizations(nextAuthorizations);
      setRegistry(nextRegistry);
      const eligible = nextAuthorizations.items.find((item) => item.future_attempt_eligible);
      if (!nextAuthorizations.items.some((item) => item.runner.isolated_runner_id === selectedRunnerId())) {
        setSelectedRunnerId(eligible?.runner.isolated_runner_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 88 初始化尝试登记表读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() => authorizations()?.items.find(
    (item) => item.runner.isolated_runner_id === selectedRunnerId() && item.future_attempt_eligible,
  ));
  const manifest = createMemo(() => {
    try {
      return JSON.parse(manifestJson()) as ControlledShadowForwardObservationInitializationManifest;
    } catch {
      return undefined;
    }
  });
  const disabled = createMemo(() => busy() || !selected() || !manifest() || checks().some((value) => !value));

  const submit = async () => {
    const item = selected();
    const initializationManifest = manifest();
    if (!item || !item.latest_review || !initializationManifest || disabled()) return;
    const runner = item.runner;
    const implementation = runner.implementation;
    const contract = implementation.implementation_contract;
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await invokeControlledShadowForwardObservationOnce(runner.isolated_runner_id, {
        expected_authorization_review_id: item.latest_review.review_id,
        expected_authorization_review_sha256: item.latest_review.review_sha256,
        expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256,
        expected_runner_contract_sha256: runner.runner_contract.contract_sha256,
        expected_runner_code_revision: runner.runner_code_revision,
        expected_runner_artifact_sha256: runner.runner_artifact_sha256,
        expected_implementation_id: implementation.implementation_id,
        expected_implementation_sha256: implementation.implementation_sha256,
        expected_implementation_contract_sha256: contract.contract_sha256,
        expected_implementation_review_sha256: runner.implementation_review.review_sha256,
        expected_protocol_review_sha256: runner.runner_contract.stage_83_protocol_review_sha256,
        expected_protocol_registration_sha256: runner.runner_contract.stage_82_protocol_registration_sha256,
        expected_protocol_specification_sha256: runner.runner_contract.stage_82_protocol_specification_sha256,
        expected_design_specification_sha256: runner.runner_contract.stage_74_design_specification_sha256,
        expected_initial_observation_validation_sha256: contract.validation_sha256,
        expected_initialization_manifest_sha256: initializationManifest.manifest_sha256,
        initialization_manifest: initializationManifest,
        claim_first_single_use_and_failure_consumes_confirmed: true,
        exact_current_stage_51_through_stage_87_binding_confirmed: true,
        executor_independent_from_stage_87_and_complete_prior_chain_confirmed: true,
        current_binary_digest_reverification_after_claim_confirmed: true,
        natural_forward_observation_not_before_and_no_backfill_confirmed: true,
        official_calendar_and_spy_synchronization_confirmed: true,
        initialization_manifest_contains_no_market_data_confirmed: true,
        initialization_receipt_is_untrusted_and_requires_independent_validation_confirmed: true,
        no_runtime_mount_data_access_observation_ledger_position_or_performance_confirmed: true,
        no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
      });
      setRegistry(next); setManifestJson(""); setChecks(EXECUTION_CHECKS.map(() => false));
      setNotice("Stage 87 授权已永久消费；成功时也只生成零行情、零观察的不可信初始化收据，等待未来 Stage 89 独立验证。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 88 claim-first 初始化尝试失败");
      await load();
    } finally { setBusy(false); }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="Stage 88 前向观察 claim-first 初始化尝试">
        <header><strong>第 88 阶段 · claim-first 单次前向观察初始化</strong><span>{current().execution_status}</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>可用一次性资格</span><strong>{current().invocation_eligible_authorization_count}</strong></div>
          <div><span>已 claim</span><strong>{current().claim_count}</strong></div>
          <div><span>初始化完成</span><strong>{current().completed_count}</strong></div>
          <div><span>待独立验证</span><strong>{current().independent_validation_eligible_count}</strong></div>
        </div>
        <article class="public-admin-reward-governance">
          <header><strong>不可逆初始化门禁</strong><span>不是前向观察结果</span></header>
          <p>提交的 canonical JSON 必须预先计算 manifest_sha256；后端先落盘 claim，再复核二进制与清单。即使清单错误，授权也已永久消费。</p>
          <p class="public-admin-anchor-boundary">本阶段固定为 0 行行情、0 个自然前向交易日、0 个账本/持仓/绩效；不能据此形成收益、评级、训练奖励或交易结论。</p>
        </article>
        <Show when={(authorizations()?.items.filter((item) => item.future_attempt_eligible).length ?? 0) > 0} fallback={<p>当前没有未 claim 且未过期的 Stage 87 一次性授权。</p>}>
          <label><span>Stage 87 一次性授权</span><select value={selectedRunnerId()} onChange={(event) => setSelectedRunnerId(event.currentTarget.value)}>
            <For each={authorizations()?.items.filter((item) => item.future_attempt_eligible) ?? []}>{(item) => (
              <option value={item.runner.isolated_runner_id}>{item.runner.runner_name} · 截止 {item.latest_review?.authorization_valid_until}</option>
            )}</For>
          </select></label>
          <label><span>零行情初始化 manifest JSON（含已计算的 manifest_sha256）</span><textarea value={manifestJson()} onInput={(event) => setManifestJson(event.currentTarget.value)} placeholder='{"schema_version":"hone-controlled-shadow-forward-observation-initialization-manifest-v1-no-market-data","manifest_sha256":"…","market_data_rows_attached":false,"initialization_only":true}' /></label>
          <div class="public-admin-decision-checks"><For each={EXECUTION_CHECKS}>{(label, index) => (
            <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((currentChecks) => currentChecks.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
          )}</For></div>
          <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>永久消费授权并生成一次初始化收据</button>
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().attempts}>{(attempt) => (
          <article class="public-admin-reward-governance">
            <header><strong>attempt {attempt.claim.attempt_id}</strong><span>{attempt.result?.status ?? "claim 已落盘 · 中断失败关闭"}</span></header>
            <p>manifest {attempt.claim.initialization_manifest_sha256} · executor {attempt.claim.invoked_by}</p>
            <Show when={attempt.result?.failure_reason}><p class="public-admin-error">{attempt.result?.failure_reason}</p></Show>
            <p class="public-admin-anchor-boundary">初始化收据在未来 Stage 89 独立验证前始终不可信；它不代表已经开始前向观察。</p>
          </article>
        )}</For>
      </section>
    )}</Show>
  );
}
