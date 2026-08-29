import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowForwardObservationFirstExecutionAuthorizations,
  reviewControlledShadowForwardObservationFirstExecutionAuthorization,
} from "@/lib/api";
import type {
  ControlledShadowForwardObservationFirstExecutionAuthorizationRegistry,
  ControlledShadowForwardObservationFirstExecutionAuthorizationVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–86 完整责任链",
  "复核者独立于 Stage 86 登记人和全部上游角色",
  "已独立复算 runner 规格、合同与完整上游哈希链",
  "已独立复现 runner 工件 SHA-256，且与冻结摘要完全一致",
  "代码版本不可变，工件可按登记程序取得并复现",
  "自然前向、不回填和 observation-not-before 边界保持不变",
  "每周 claim-first/create-once、官方日历与 SPY 同步规则保持不变",
  "未来输入只允许点时、只读、内容寻址和白名单化",
  "公司行动证据与 append-only 更正链保持不变",
  "输出创建一次、不可信、独立验证且不含订单或券商载荷",
  "信号、组合、成交成本、反事实、检查点和停止规则保持不变",
  "固定非特权身份、只读根目录、临时工作区和资源上限保持不变",
  "无环境继承、密钥、网络、工具、子进程或生产读写",
  "授权 24 小时、仅一次，且与 Stage 88 claim 和执行严格分离",
  "当前不实例化 runtime、不挂载、不访问数据、不观察、不建账、不写持仓或绩效",
  "无模型/指标写入、训练反馈、奖励、订单、券商或交易权限",
  "批准只开放未来 Stage 88 claim-first 尝试候选",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowForwardObservationFirstExecutionAuthorizationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowForwardObservationFirstExecutionAuthorizationRegistry>();
  const [selectedRunnerId, setSelectedRunnerId] = createSignal("");
  const [verdict, setVerdict] = createSignal<ControlledShadowForwardObservationFirstExecutionAuthorizationVerdict>(
    "changes_requested_rebuild_runner",
  );
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [reproducedSha256, setReproducedSha256] = createSignal("");
  const [reproductionEvidence, setReproductionEvidence] = createSignal("");
  const [rationale, setRationale] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowForwardObservationFirstExecutionAuthorizations();
      setRegistry(next);
      if (!next.items.some((item) => item.runner.isolated_runner_id === selectedRunnerId())) {
        setSelectedRunnerId(next.items[0]?.runner.isolated_runner_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 87 前向观察首跑授权表读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() => registry()?.items.find(
    (item) => item.runner.isolated_runner_id === selectedRunnerId(),
  ));
  const approving = createMemo(() => verdict()
    === "approved_for_one_future_claim_first_forward_observation_attempt");
  const disabled = createMemo(() => busy()
    || !selected()
    || rationale().trim().length === 0
    || reproductionEvidence().trim().length === 0
    || reproducedSha256().trim().length !== 64
    || (approving() && !checks().every(Boolean)));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const runner = item.runner;
    const implementation = runner.implementation;
    const contract = runner.runner_contract;
    const audit = runner.implementation_review.independent_audit;
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await reviewControlledShadowForwardObservationFirstExecutionAuthorization(
        runner.isolated_runner_id,
        {
          expected_review_id: item.latest_review?.review_id,
          expected_review_sha256: item.latest_review?.review_sha256,
          expected_isolated_runner_id: runner.isolated_runner_id,
          expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256,
          expected_runner_contract_sha256: contract.contract_sha256,
          expected_runner_spec_revision: runner.runner_spec_revision,
          expected_runner_code_revision: runner.runner_code_revision,
          expected_runner_artifact_sha256: runner.runner_artifact_sha256,
          expected_implementation_id: implementation.implementation_id,
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_review_id: runner.implementation_review.review_id,
          expected_implementation_review_sha256: runner.implementation_review.review_sha256,
          expected_independent_audit_sha256: audit.audit_sha256,
          expected_protocol_review_sha256: contract.stage_83_protocol_review_sha256,
          expected_protocol_registration_sha256: contract.stage_82_protocol_registration_sha256,
          expected_protocol_specification_sha256: contract.stage_82_protocol_specification_sha256,
          expected_design_specification_sha256: contract.stage_74_design_specification_sha256,
          independently_reproduced_runner_artifact_sha256: reproducedSha256().trim(),
          artifact_reproduction_evidence: reproductionEvidence().trim(), verdict: verdict(),
          rationale: rationale().trim(),
          exact_current_stage_51_through_stage_86_binding_confirmed: checks()[0] as boolean,
          reviewer_independence_from_stage_86_and_complete_prior_chain_confirmed: checks()[1] as boolean,
          runner_spec_contract_and_complete_hash_chain_independently_reproduced_confirmed: checks()[2] as boolean,
          runner_artifact_digest_independently_reproduced_and_matched_confirmed: checks()[3] as boolean,
          immutable_code_revision_and_artifact_availability_confirmed: checks()[4] as boolean,
          natural_forward_no_backfill_and_observation_not_before_confirmed: checks()[5] as boolean,
          weekly_claim_first_create_once_official_calendar_and_spy_sync_confirmed: checks()[6] as boolean,
          point_in_time_read_only_content_addressed_allowlisted_input_confirmed: checks()[7] as boolean,
          corporate_action_evidence_and_append_only_corrections_confirmed: checks()[8] as boolean,
          create_once_untrusted_independently_validated_no_order_payload_output_confirmed: checks()[9] as boolean,
          deterministic_replay_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed: checks()[10] as boolean,
          fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: checks()[11] as boolean,
          no_environment_secret_network_tool_subprocess_or_production_io_confirmed: checks()[12] as boolean,
          authorization_single_use_24_hour_expiry_and_stage_88_claim_separation_confirmed: checks()[13] as boolean,
          no_runtime_mount_data_access_observation_ledger_position_performance_or_execution_confirmed: checks()[14] as boolean,
          no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: checks()[15] as boolean,
          approval_only_opens_future_stage_88_claim_first_attempt_confirmed: checks()[16] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[17] as boolean,
        },
      );
      setRegistry(next); setChecks(REVIEW_CHECKS.map(() => false)); setRationale("");
      setReproductionEvidence(""); setReproducedSha256("");
      setNotice(approving()
        ? "已签发 24 小时内一次性的未来 Stage 88 claim-first 候选；本次没有执行或挂载。"
        : "复核已 append-only 保存；没有开放执行能力。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 87 授权复核失败");
      await load();
    } finally { setBusy(false); }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="前向观察首次执行授权独立复核">
        <header><strong>第 87 阶段 · 前向观察首次执行授权独立复核</strong><span>{current().authorization_status}</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>待独立复核</span><strong>{current().review_eligible_runner_count}</strong></div>
          <div><span>已复核</span><strong>{current().reviewed_runner_count}</strong></div>
          <div><span>未过期一次性授权</span><strong>{current().unexpired_authorization_count}</strong></div>
          <div><span>未来尝试候选</span><strong>{current().future_attempt_eligible_count}</strong></div>
        </div>
        <article class="public-admin-reward-governance">
          <header><strong>批准不等于执行</strong><span>24 小时 · 最多一次</span></header>
          <p>必须提交独立复现得到的 runner 工件 SHA-256 和复现证据；摘要不一致时后端拒绝批准。</p>
          <p class="public-admin-anchor-boundary">当前无 callable entrypoint、runtime、挂载、数据访问、观察、账本、持仓、绩效、订单、券商或交易权限。</p>
        </article>
        <Show when={current().items.length > 0} fallback={<p>当前没有可进入 Stage 87 的 Stage 86 runner。</p>}>
          <label><span>Stage 86 runner</span><select value={selectedRunnerId()} onChange={(event) => setSelectedRunnerId(event.currentTarget.value)}>
            <For each={current().items}>{(item) => <option value={item.runner.isolated_runner_id}>{item.runner.runner_name} · {item.runner.isolated_runner_id.slice(0, 12)}…</option>}</For>
          </select></label>
          <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ControlledShadowForwardObservationFirstExecutionAuthorizationVerdict)}>
            <option value="changes_requested_rebuild_runner">要求修改并重建 runner</option>
            <option value="rejected">拒绝</option>
            <option value="approved_for_one_future_claim_first_forward_observation_attempt">批准未来一次 claim-first 尝试候选</option>
          </select></label>
          <label><span>独立复现 runner 工件 SHA-256</span><input value={reproducedSha256()} onInput={(event) => setReproducedSha256(event.currentTarget.value)} /></label>
          <label><span>工件复现证据</span><textarea value={reproductionEvidence()} onInput={(event) => setReproductionEvidence(event.currentTarget.value)} /></label>
          <label><span>复核依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
          <div class="public-admin-decision-checks"><For each={REVIEW_CHECKS}>{(label, index) => (
            <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
          )}</For></div>
          <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在保存复核…" : "保存 Stage 87 独立复核"}</button>
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().items}>{(item) => <article class="public-admin-reward-governance">
          <header><strong>{item.runner.runner_name}</strong><span>{item.latest_review?.verdict ?? "待复核"}</span></header>
          <p>冻结工件 {item.runner.runner_artifact_sha256.slice(0, 16)}… · 代码 {item.runner.runner_code_revision}</p>
          <Show when={item.latest_review}>{(review) => <p>复现摘要匹配：{review().artifact_digest_matches_registered_runner ? "是" : "否"} · 有效至 {review().authorization_valid_until}</p>}</Show>
          <p class="public-admin-anchor-boundary">{item.future_attempt_eligible ? "仅获得未来 Stage 88 一次性 claim-first 候选，尚未执行。" : "当前没有执行资格。"}</p>
        </article>}</For>
      </section>
    )}</Show>
  );
}
