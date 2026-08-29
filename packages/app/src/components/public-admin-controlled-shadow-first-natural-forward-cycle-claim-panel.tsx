import { For, Show, createSignal, onMount } from "solid-js";

import {
  claimControlledShadowFirstNaturalForwardCycleOnce,
  getControlledShadowFirstNaturalForwardCycleClaims,
} from "@/lib/api";
import type {
  ClaimControlledShadowFirstNaturalForwardCycleRequest,
  ControlledShadowFirstNaturalForwardCycleClaimRegistry,
} from "@/lib/types";

const CLAIM_CHECKS = [
  "确认精确绑定当前 Stage 51–90 完整哈希链",
  "确认领取者独立于 Stage 90 复核者及完整既有责任链",
  "确认授权当前已生效、未过期且只能使用一次",
  "确认先写 claim，之后才可能解析日历或接触行情",
  "确认行情适配器必须另经明确、只读、内容寻址白名单授权",
  "确认只允许自然前向、禁止回填，任务 create-once 不可重放",
  "确认当前不启动 runtime/观察，不建账、不写持仓或绩效",
  "确认不写模型/指标、不反馈训练/reward，不生成订单、不接券商、不交易",
  "确认没有把未确认的 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowFirstNaturalForwardCycleClaimPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowFirstNaturalForwardCycleClaimRegistry>();
  const [checks, setChecks] = createSignal(CLAIM_CHECKS.map(() => false));
  const [reason, setReason] = createSignal("");
  const [busyId, setBusyId] = createSignal("");
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      setRegistry(await getControlledShadowFirstNaturalForwardCycleClaims());
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 91 任务声明表读取失败");
    }
  };
  onMount(() => void load());

  const submit = async (
    item: ControlledShadowFirstNaturalForwardCycleClaimRegistry["eligible_authorizations"][number],
  ) => {
    if (checks().some((value) => !value) || !reason().trim()) return;
    const authorization = item.authorization;
    const request: ClaimControlledShadowFirstNaturalForwardCycleRequest = {
      expected_authorization_review_sha256: authorization.review_sha256,
      expected_validation_sha256: item.validation.validation_sha256,
      expected_stage_88_attempt_id: authorization.attempt_id,
      expected_stage_88_claim_sha256: authorization.claim_sha256,
      expected_stage_88_result_sha256: authorization.result_sha256,
      expected_stage_88_output_sha256: authorization.output_sha256,
      expected_initialization_manifest_sha256: authorization.initialization_manifest_sha256,
      claim_reason: reason().trim(),
      exact_stage_51_through_stage_90_binding_confirmed: true,
      claimant_independence_from_stage_90_and_complete_prior_chain_confirmed: true,
      authorization_current_unexpired_and_single_use_confirmed: true,
      claim_first_before_calendar_or_market_data_confirmed: true,
      separate_read_only_market_data_adapter_authorization_required_confirmed: true,
      natural_forward_only_no_backfill_and_create_once_confirmed: true,
      no_runtime_observation_ledger_position_or_performance_confirmed: true,
      no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    setBusyId(authorization.review_id);
    setError("");
    setNotice("");
    try {
      setRegistry(await claimControlledShadowFirstNaturalForwardCycleOnce(
        authorization.review_id,
        request,
      ));
      setChecks(CLAIM_CHECKS.map(() => false));
      setReason("");
      setNotice("Stage 91 任务 claim 已不可逆写入；授权已消费，但没有读取日历或行情，也没有开始观察。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 91 任务领取失败");
    } finally {
      setBusyId("");
      await load();
    }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="Stage 91 首个自然前向周期任务声明">
        <header><strong>第 91 阶段 · 首周期任务声明</strong><span>{current().claim_status}</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>可领取</span><strong>{current().claim_eligible_count}</strong></div>
          <div><span>已领取</span><strong>{current().claim_count}</strong></div>
          <div><span>已消费授权</span><strong>{current().authorization_consumed_count}</strong></div>
          <div><span>等待适配器授权</span><strong>{current().waiting_for_separate_market_data_adapter_authorization_count}</strong></div>
        </div>
        <p class="public-admin-anchor-boundary">领取是不可逆操作：它只建立不可执行任务，不解析日历、不读取行情。后续行情适配器必须单独审批。</p>
        <Show when={current().eligible_authorizations.length > 0}>
          <div class="public-admin-decision-checks"><For each={CLAIM_CHECKS}>{(label, index) => (
            <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
          )}</For></div>
          <textarea class="public-admin-decision-textarea" value={reason()} onInput={(event) => setReason(event.currentTarget.value)} placeholder="写明为何现在领取首周期任务（必填）" />
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().eligible_authorizations}>{(item) => (
          <article class="public-admin-reward-governance">
            <header><strong>authorization {item.authorization.review_id}</strong><span>有效至 {item.authorization.authorization_valid_until}</span></header>
            <p>Stage 89 validation {item.validation.validation_id} · observation anchor {item.authorization.observation_not_before}</p>
            <div class="public-admin-decision-actions">
              <button type="button" class="public-admin-decision-submit" disabled={busyId() !== "" || checks().some((value) => !value) || !reason().trim()} onClick={() => void submit(item)}>{busyId() === item.authorization.review_id ? "正在不可逆领取…" : "领取并永久消费授权"}</button>
            </div>
          </article>
        )}</For>
        <For each={current().claims}>{(claim) => (
          <article class="public-admin-reward-governance">
            <header><strong>claim {claim.cycle_claim_id}</strong><span>{claim.task_status}</span></header>
            <p>{claim.claim_reason} · {claim.claimed_at}</p>
            <p class="public-admin-anchor-boundary">等待只读行情适配器授权；日历未解析、行情未读取、观察未开始。</p>
          </article>
        )}</For>
      </section>
    )}</Show>
  );
}
