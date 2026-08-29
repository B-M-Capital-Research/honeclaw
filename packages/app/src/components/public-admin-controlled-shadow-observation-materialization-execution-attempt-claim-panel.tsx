import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  claimControlledShadowObservationMaterializationExecutionAttemptOnce,
  getControlledShadowObservationMaterializationExecutionAttemptClaims,
} from "@/lib/api";
import type { ControlledShadowObservationMaterializationExecutionAttemptClaimRegistry } from "@/lib/types";

const CLAIM_CHECKS = [
  "精确绑定当前 Stage 51–110 完整责任链",
  "声明人独立于 Stage 110 复核者、工件构建者和全部上游角色",
  "未过期单次授权会在任何执行前被永久消费",
  "当前服务端重哈希工件与自哈希 manifest 绑定保持不变",
  "精确 Stage 104 准入输入保持只读、内容寻址且本阶段不读取",
  "声明只保存既有元数据与摘要，不复制或替换输入、工件和执行参数",
  "当前无入口、runtime、输入挂载/读取或观察物化执行",
  "未来输出必须 create-once、内容寻址、非可信并接受独立验证",
  "声明后不得重试、释放或恢复这一条授权",
  "无观察、账本、持仓、绩效、模型/指标、训练、reward、订单、券商或交易权限",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationMaterializationExecutionAttemptClaimPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowObservationMaterializationExecutionAttemptClaimRegistry>();
  const [selectedReviewId, setSelectedReviewId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [checks, setChecks] = createSignal(CLAIM_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationMaterializationExecutionAttemptClaims();
      setRegistry(next);
      if (!next.eligible_authorizations.some((item) => item.authorization.review_id === selectedReviewId())) {
        setSelectedReviewId(next.eligible_authorizations[0]?.authorization.review_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 111 观察物化尝试声明表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.eligible_authorizations.find(
    (item) => item.authorization.review_id === selectedReviewId(),
  ));
  const disabled = createMemo(() => busy()
    || !selected()
    || reason().trim().length === 0
    || !checks().every(Boolean));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const authorization = item.authorization;
    const runner = authorization.runner;
    const implementation = runner.implementation;
    const contract = implementation.implementation_contract;
    const specification = contract.exact_observation_materialization_specification;
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await claimControlledShadowObservationMaterializationExecutionAttemptOnce(
        authorization.review_id,
        {
          expected_authorization_review_sha256: authorization.review_sha256,
          expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256,
          expected_runner_contract_sha256: runner.runner_contract.contract_sha256,
          expected_runner_artifact_sha256: authorization.server_computed_artifact_sha256,
          expected_artifact_manifest_sha256: authorization.artifact_manifest.manifest_sha256,
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_contract_sha256: contract.contract_sha256,
          expected_implementation_review_sha256: runner.implementation_review.review_sha256,
          expected_observation_materialization_specification_sha256: specification.specification_sha256,
          expected_stage_104_admission_review_sha256: specification.stage_104_review_sha256,
          expected_stage_103_validation_sha256: specification.stage_103_validation_sha256,
          expected_stage_102_result_sha256: specification.stage_102_result_sha256,
          expected_stage_102_output_sha256: specification.stage_102_output_sha256,
          expected_stage_101_claim_sha256: specification.stage_101_claim_sha256,
          expected_stage_101_input_manifest_sha256: specification.stage_101_input_manifest_sha256,
          expected_cycle_claim_sha256: specification.cycle_claim_sha256,
          claim_reason: reason().trim(),
          exact_current_stage_51_through_stage_110_binding_confirmed: checks()[0] as boolean,
          claimant_independent_from_stage_110_and_complete_prior_chain_confirmed: checks()[1] as boolean,
          authorization_unexpired_single_use_and_permanently_consumed_before_execution_confirmed: checks()[2] as boolean,
          current_server_rehashed_artifact_and_manifest_binding_confirmed: checks()[3] as boolean,
          exact_stage_104_admitted_input_remains_content_addressed_read_only_and_unread_confirmed: checks()[4] as boolean,
          claim_contains_only_existing_metadata_and_hashes_confirmed: checks()[5] as boolean,
          no_entrypoint_runtime_input_mount_input_read_or_observation_materialization_execution_confirmed: checks()[6] as boolean,
          future_output_create_once_content_addressed_untrusted_and_independently_validated_confirmed: checks()[7] as boolean,
          no_retry_release_or_authorization_restoration_after_claim_confirmed: checks()[8] as boolean,
          no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: checks()[9] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[10] as boolean,
        },
      );
      setRegistry(next); setReason(""); setChecks(CLAIM_CHECKS.map(() => false));
      setNotice("Stage 110 授权已永久消费；本次没有运行工件、读取 Stage 104 输入或生成观察。即使未来执行失败，该授权也不会返还。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 111 观察物化尝试声明失败");
      await load();
    } finally { setBusy(false); }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="观察物化单次尝试 claim-first 声明">
        <header><strong>第 111 阶段 · 观察物化单次尝试声明</strong><span>{current().claim_status}</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>可声明授权</span><strong>{current().claim_eligible_count}</strong></div>
          <div><span>已永久消费</span><strong>{current().authorization_consumed_count}</strong></div>
          <div><span>已建尝试身份</span><strong>{current().claim_count}</strong></div>
          <div><span>待 Stage 112</span><strong>{current().waiting_for_stage_112_execution_count}</strong></div>
        </div>
        <Show when={current().eligible_authorizations.length > 0} fallback={<p>当前没有未过期且未消费的 Stage 110 授权。</p>}>
          <label><span>Stage 110 授权</span><select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
            <For each={current().eligible_authorizations}>{(item) => <option value={item.authorization.review_id}>{item.authorization.runner.runner_name} · 有效至 {item.authorization.authorization_valid_until}</option>}</For>
          </select></label>
          <Show when={selected()}>{(item) => <article class="public-admin-reward-governance">
            <header><strong>固定工件与完整输入责任链</strong><span>只显示元数据与摘要</span></header>
            <p>artifact {item().authorization.server_computed_artifact_sha256.slice(0, 16)}… · manifest {item().authorization.artifact_manifest.manifest_sha256.slice(0, 16)}…</p>
            <p>Stage 104 {item().authorization.runner.implementation.implementation_contract.exact_observation_materialization_specification.stage_104_review_sha256.slice(0, 16)}…</p>
            <p class="public-admin-anchor-boundary">点击声明会永久消费授权；本按钮不运行工件、不挂载或读取输入，也不能撤销。</p>
          </article>}</Show>
          <label><span>声明原因</span><textarea value={reason()} onInput={(event) => setReason(event.currentTarget.value)} /></label>
          <div class="public-admin-decision-checks"><For each={CLAIM_CHECKS}>{(label, index) => (
            <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
          )}</For></div>
          <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在永久消费授权…" : "创建 Stage 111 claim-first 声明"}</button>
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().claims}>{(claim) => <article class="public-admin-reward-governance">
          <header><strong>{claim.authorization.runner.runner_name}</strong><span>授权已消费 · 未执行</span></header>
          <p>claim {claim.claim_sha256.slice(0, 16)}… · artifact {claim.authorization.server_computed_artifact_sha256.slice(0, 16)}… · {claim.claimed_at}</p>
          <p class="public-admin-anchor-boundary">{claim.task_status}；不可重试、释放或恢复。</p>
        </article>}</For>
      </section>
    )}</Show>
  );
}
