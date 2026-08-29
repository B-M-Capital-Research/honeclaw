import { For, Show, createSignal, onMount } from "solid-js";

import {
  getControlledShadowFirstNaturalForwardCycleAuthorizations,
  reviewControlledShadowFirstNaturalForwardCycleAuthorization,
} from "@/lib/api";
import type {
  ControlledShadowFirstNaturalForwardCycleAuthorizationRegistry,
  ReviewControlledShadowFirstNaturalForwardCycleAuthorizationRequest,
} from "@/lib/types";

const AUTHORIZATION_CHECKS = [
  "确认精确绑定当前 Stage 51–89，不接受摘要字段替代完整哈希链",
  "确认复核者独立于 Stage 89 验证者、Stage 88 执行者、Stage 87 复核者和完整既有责任链",
  "确认 Stage 89 已独立验证零行情初始化收据",
  "确认只允许自然前向、禁止回填，并遵守 observation-not-before",
  "确认未来必须使用官方 HTTPS 日历内容身份，证券与 SPY 同步观察",
  "确认未来输入必须 point-in-time、只读、内容寻址且在白名单内",
  "确认公司行动必须留存证据，修订只能追加更正",
  "确认未来任务必须 claim-first/create-once，失败也消费资格，输出仍须独立验证",
  "确认 long-only、敞口上限、成本、反事实、检查点和停止规则保持确定性",
  "确认固定非特权身份、只读根目录、临时工作目录和资源上限",
  "确认未来行情适配器必须另行获得明确、只读、白名单授权",
  "确认授权仅在首个合格自然周期起算 7 天内有效且最多一次",
  "确认本次复核不读取日历或行情、不启动 runtime/观察、不建账、不写持仓或绩效",
  "确认不写模型/指标、不反馈训练/reward，不生成订单、不接券商、不交易",
  "确认通过只开放未来 claim-first 周期尝试，不直接开始观察",
  "确认没有把未确认的 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowFirstNaturalForwardCycleAuthorizationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowFirstNaturalForwardCycleAuthorizationRegistry>();
  const [checks, setChecks] = createSignal(AUTHORIZATION_CHECKS.map(() => false));
  const [rationale, setRationale] = createSignal("");
  const [busyId, setBusyId] = createSignal("");
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      setRegistry(await getControlledShadowFirstNaturalForwardCycleAuthorizations());
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 90 授权复核表读取失败");
    }
  };
  onMount(() => void load());

  const submit = async (
    item: ControlledShadowFirstNaturalForwardCycleAuthorizationRegistry["items"][number],
    verdict: ReviewControlledShadowFirstNaturalForwardCycleAuthorizationRequest["verdict"],
  ) => {
    if (checks().some((value) => !value) || !rationale().trim()) return;
    const { claim, result } = item.attempt;
    const validation = item.validation;
    if (!result.output_sha256) return;
    setBusyId(validation.validation_id);
    setError("");
    setNotice("");
    try {
      setRegistry(await reviewControlledShadowFirstNaturalForwardCycleAuthorization(
        validation.validation_id,
        {
          expected_review_id: item.latest_review?.review_id,
          expected_review_sha256: item.latest_review?.review_sha256,
          expected_validation_sha256: validation.validation_sha256,
          expected_attempt_id: claim.attempt_id,
          expected_claim_sha256: claim.claim_sha256,
          expected_result_sha256: result.result_sha256,
          expected_output_sha256: result.output_sha256,
          expected_authorization_review_sha256: validation.authorization_review_sha256,
          expected_isolated_runner_spec_sha256: validation.isolated_runner_spec_sha256,
          expected_runner_artifact_sha256: validation.runner_artifact_sha256,
          expected_implementation_contract_sha256: validation.implementation_contract_sha256,
          expected_protocol_specification_sha256: validation.protocol_specification_sha256,
          expected_design_specification_sha256: validation.design_specification_sha256,
          expected_initial_observation_validation_sha256: validation.initial_observation_validation_sha256,
          expected_initialization_manifest_sha256: validation.initialization_manifest_sha256,
          verdict,
          rationale: rationale().trim(),
          exact_current_stage_51_through_stage_89_binding_confirmed: true,
          reviewer_independence_from_stage_89_stage_88_stage_87_and_complete_prior_chain_confirmed: true,
          zero_market_initialization_receipt_independently_validated_confirmed: true,
          natural_forward_only_no_backfill_and_observation_not_before_confirmed: true,
          official_https_calendar_content_identity_and_security_spy_sync_confirmed: true,
          point_in_time_read_only_content_addressed_allowlisted_inputs_confirmed: true,
          corporate_action_evidence_and_append_only_corrections_confirmed: true,
          claim_first_create_once_failure_consumes_and_independent_output_validation_confirmed: true,
          deterministic_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed: true,
          fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: true,
          future_market_data_adapter_requires_separate_explicit_read_only_authorization_confirmed: true,
          single_use_seven_day_window_and_future_attempt_separation_confirmed: true,
          current_review_has_no_calendar_market_data_runtime_observation_ledger_position_or_performance_confirmed: true,
          no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: true,
          approval_only_opens_future_claim_first_cycle_attempt_confirmed: true,
          no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        },
      ));
      setChecks(AUTHORIZATION_CHECKS.map(() => false));
      setRationale("");
      setNotice(verdict.startsWith("approved")
        ? "Stage 90 一次性授权复核已记录；它没有开始观察，也没有读取日历或行情。"
        : "Stage 90 复核意见已追加；当前未开放自然前向周期尝试资格。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 90 授权复核失败");
    } finally {
      setBusyId("");
      await load();
    }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="Stage 90 首个自然前向周期一次性授权复核">
        <header><strong>第 90 阶段 · 首个自然前向周期授权复核</strong><span>{current().authorization_status}</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>可复核</span><strong>{current().review_eligible_initialization_count}</strong></div>
          <div><span>已复核</span><strong>{current().reviewed_initialization_count}</strong></div>
          <div><span>已批准</span><strong>{current().approved_initialization_count}</strong></div>
          <div><span>未来单次资格</span><strong>{current().future_attempt_eligible_count}</strong></div>
        </div>
        <p class="public-admin-anchor-boundary">7 天窗口从首个合格自然周期开始计算；本阶段本身没有日历、行情适配器、runtime、账本、持仓、绩效或交易能力。Stage 91 只能另行领取不可执行任务。</p>
        <Show when={current().items.some((item) => !item.latest_review?.one_future_claim_first_natural_forward_cycle_attempt_authorized)}>
          <div class="public-admin-decision-checks"><For each={AUTHORIZATION_CHECKS}>{(label, index) => (
            <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
          )}</For></div>
          <textarea class="public-admin-decision-textarea" value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} placeholder="写明授权或退回依据（必填）" />
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().items}>{(item) => (
          <article class="public-admin-reward-governance">
            <header><strong>validation {item.validation.validation_id}</strong><span>{item.authorization_claimed ? "授权已由 Stage 91 永久消费" : item.latest_review?.verdict ?? "等待独立授权复核"}</span></header>
            <p>Stage 89 {item.validation.validation_sha256} · observation-not-before {item.attempt.result.untrusted_initialization_receipt?.observation_not_before}</p>
            <Show when={item.latest_review}>{(review) => <p>{review().rationale} · 有效窗口 {review().authorization_not_before} 至 {review().authorization_valid_until}</p>}</Show>
            <Show when={!item.latest_review?.one_future_claim_first_natural_forward_cycle_attempt_authorized}>
              <div class="public-admin-decision-actions">
                <button type="button" class="public-admin-decision-submit" disabled={busyId() !== "" || checks().some((value) => !value) || !rationale().trim()} onClick={() => void submit(item, "approved_for_one_future_claim_first_natural_forward_cycle_attempt")}>{busyId() === item.validation.validation_id ? "正在写入…" : "批准未来单次尝试"}</button>
                <button type="button" disabled={busyId() !== "" || checks().some((value) => !value) || !rationale().trim()} onClick={() => void submit(item, "changes_requested_revalidate_initialization")}>退回重新验证</button>
              </div>
            </Show>
          </article>
        )}</For>
      </section>
    )}</Show>
  );
}
