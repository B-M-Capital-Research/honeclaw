import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationLedgerTransitionFirstExecutionAuthorizations,
  reviewControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationOnce,
} from "@/lib/api";
import type {
  ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationRegistry,
  ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–119 完整责任链",
  "复核者独立于 Stage 119 登记人、工件构建者和全部上游角色",
  "服务端已重新读取只读常规工件并计算 SHA-256，摘要和长度完全匹配",
  "自哈希 manifest、代码版本、runtime 与复现步骤摘要完全匹配",
  "工件构建者与 Stage 120 复核者相互分离",
  "八个观察到账本转换纯函数与 canonical schema 继续由 Stage 119 合同绑定",
  "交易日、三价格口径、显式缺口、公司行动、初始分配、可得时间与失败关闭语义保持不变",
  "禁止覆盖、回填、前填、插值、跨口径替代或推断公司行动",
  "固定无特权身份、只读根目录、临时工作区和资源上限保持不变",
  "未来输入仅限 Stage 114 已准入、只读且内容寻址的标准化输出",
  "未来输出 create-once、非可信、独立验证且不含市场解释或订单意图",
  "期初组合快照仍不存在，金融事件白名单保持为空",
  "任何未来金融账本事件都必须先有单独准入的期初组合快照",
  "未来尝试仅可产生非金融通知候选，不能产生权威持仓、现金、净值或绩效状态",
  "provider_publication_time 仍未验证，必须由后续独立证据确认",
  "授权 24 小时、仅一次，并与 Stage 121 claim 严格分离",
  "当前无 runtime/entrypoint/挂载/输入读取/观察到账本转换执行或候选输出",
  "无环境继承、secret、网络、工具、子进程或生产读写",
  "无权威账本事件、持仓、现金、净值/绩效、模型/指标、训练、reward、订单、券商或交易权限",
  "批准只开放未来 Stage 121 claim-first 单次尝试候选",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationRegistry>();
  const [selectedRunnerId, setSelectedRunnerId] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationVerdict>(
      "changes_requested_rebuild_artifact",
    );
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [artifactEvidence, setArtifactEvidence] = createSignal("");
  const [sandboxEvidence, setSandboxEvidence] = createSignal("");
  const [rationale, setRationale] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationLedgerTransitionFirstExecutionAuthorizations();
      setRegistry(next);
      if (!next.items.some((item) => item.runner.isolated_runner_id === selectedRunnerId())) {
        setSelectedRunnerId(next.items[0]?.runner.isolated_runner_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 120 观察到账本转换首次执行授权表读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() => registry()?.items.find(
    (item) => item.runner.isolated_runner_id === selectedRunnerId(),
  ));
  const approving = createMemo(() => verdict()
    === "approved_for_one_future_claim_first_observation_ledger_transition_attempt");
  const disabled = createMemo(() => busy()
    || !selected()?.artifact_inspection.artifact_verified
    || !selected()?.artifact_inspection.manifest
    || artifactEvidence().trim().length === 0
    || sandboxEvidence().trim().length === 0
    || rationale().trim().length === 0
    || (approving() && !checks().every(Boolean)));

  const submit = async () => {
    const item = selected();
    const manifest = item?.artifact_inspection.manifest;
    if (!item || !manifest || disabled()) return;
    const runner = item.runner;
    const runnerContract = runner.runner_contract;
    const implementation = runner.implementation;
    const implementationContract = implementation.implementation_contract;
    const implementationReview = runner.implementation_review;
    const registration = implementation.upstream_specification_registration;
    const specification = implementationContract.exact_observation_ledger_transition_specification;
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await reviewControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationOnce(
        runner.isolated_runner_id,
        {
          expected_review_id: item.latest_review?.review_id,
          expected_review_sha256: item.latest_review?.review_sha256,
          expected_isolated_runner_id: runner.isolated_runner_id,
          expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256,
          expected_runner_contract_sha256: runnerContract.contract_sha256,
          expected_runner_spec_revision: runnerContract.runner_spec_revision,
          expected_runner_code_revision: runnerContract.proposed_runner_code_revision,
          expected_runner_artifact_sha256: runnerContract.proposed_runner_artifact_sha256,
          expected_implementation_id: implementation.implementation_id,
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_contract_sha256: implementationContract.contract_sha256,
          expected_implementation_review_id: implementationReview.review_id,
          expected_implementation_review_sha256: implementationReview.review_sha256,
          expected_independent_audit_sha256: implementationReview.independent_audit.audit_sha256,
          expected_specification_review_sha256: implementation.upstream_specification_review.review_sha256,
          expected_specification_registration_sha256: registration.registration_sha256,
          expected_observation_ledger_transition_specification_sha256: registration.specification.specification_sha256,
          expected_stage_114_admission_review_sha256: specification.stage_114_review_sha256,
          expected_stage_113_validation_sha256: specification.stage_113_validation_sha256,
          expected_stage_112_result_sha256: specification.stage_112_result_sha256,
          expected_stage_111_claim_sha256: specification.stage_111_claim_sha256,
          expected_artifact_manifest_sha256: manifest.manifest_sha256,
          artifact_reproduction_review_evidence: artifactEvidence().trim(),
          sandbox_contract_review_evidence: sandboxEvidence().trim(),
          verdict: verdict(),
          rationale: rationale().trim(),
          exact_current_stage_51_through_stage_119_binding_confirmed: checks()[0] as boolean,
          reviewer_independent_from_stage_119_builder_and_complete_prior_chain_confirmed: checks()[1] as boolean,
          server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: checks()[2] as boolean,
          self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: checks()[3] as boolean,
          artifact_builder_and_reviewer_separation_confirmed: checks()[4] as boolean,
          all_eight_observation_ledger_transition_functions_and_canonical_schemas_remain_bound_confirmed: checks()[5] as boolean,
          session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed: checks()[6] as boolean,
          no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_confirmed: checks()[7] as boolean,
          fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: checks()[8] as boolean,
          future_input_only_stage_114_admitted_read_only_content_addressed_output_confirmed: checks()[9] as boolean,
          future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: checks()[10] as boolean,
          opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: checks()[11] as boolean,
          future_financial_events_require_separately_admitted_opening_snapshot_confirmed: checks()[12] as boolean,
          future_attempt_limited_to_non_financial_notice_candidate_without_authoritative_state_confirmed: checks()[13] as boolean,
          provider_publication_time_remains_unverified_until_separate_evidence_confirmed: checks()[14] as boolean,
          authorization_single_use_24_hour_expiry_and_stage_121_claim_separation_confirmed: checks()[15] as boolean,
          no_runtime_entrypoint_mount_input_read_observation_ledger_transition_execution_or_candidate_output_confirmed: checks()[16] as boolean,
          no_environment_secret_network_tool_subprocess_or_production_io_confirmed: checks()[17] as boolean,
          no_authoritative_ledger_event_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed: checks()[18] as boolean,
          approval_only_opens_future_stage_121_claim_first_attempt_confirmed: checks()[19] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[20] as boolean,
        },
      );
      setRegistry(next); setChecks(REVIEW_CHECKS.map(() => false));
      setArtifactEvidence(""); setSandboxEvidence(""); setRationale("");
      setNotice(approving()
        ? "已签发 24 小时内一次性的未来 Stage 121 claim 候选；本次没有执行观察到账本转换或读取 Stage 114 输入。"
        : "复核已 append-only 保存；没有开放执行能力。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 120 观察到账本转换首次执行授权复核失败");
      await load();
    } finally { setBusy(false); }
  };

  return (
    <Show when={registry()}>{(current) => (
      <section class="public-admin-reward-governance" aria-label="观察到账本转换首次执行授权独立复核">
        <header><strong>第 120 阶段 · 观察到账本转换首次执行授权独立复核</strong><span>{current().authorization_status}</span></header>
        <p>{current().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>待真实工件</span><strong>{current().artifact_pending_runner_count}</strong></div>
          <div><span>服务端已核验</span><strong>{current().artifact_verified_runner_count}</strong></div>
          <div><span>已复核</span><strong>{current().reviewed_runner_count}</strong></div>
          <div><span>未来 claim 候选</span><strong>{current().future_claim_eligible_count}</strong></div>
        </div>
        <article class="public-admin-reward-governance">
          <header><strong>手填 SHA 不能通过</strong><span>服务端重哈希 · 24 小时 · 一次</span></header>
          <p>工件与 manifest 必须以只读常规文件出现在服务端派生的内容寻址保管位置；符号链接、可写文件、摘要或长度不一致都会失败关闭。</p>
          <p class="public-admin-anchor-boundary">本阶段只审查工件身份和隔离合同；期初组合仍不存在、金融事件白名单仍为空，不创建入口、runtime、挂载、候选输出或交易权限。</p>
        </article>
        <Show when={current().items.length > 0} fallback={<p>当前没有可进入 Stage 120 的 Stage 119 runner。</p>}>
          <label><span>Stage 119 runner</span><select value={selectedRunnerId()} onChange={(event) => setSelectedRunnerId(event.currentTarget.value)}>
            <For each={current().items}>{(item) => <option value={item.runner.isolated_runner_id}>{item.runner.runner_name} · {item.artifact_inspection.status}</option>}</For>
          </select></label>
          <Show when={selected()}>{(item) => <article class="public-admin-reward-governance">
            <header><strong>内容寻址工件</strong><span>{item().artifact_inspection.artifact_verified ? "服务端核验通过" : "尚不可复核"}</span></header>
            <p>{item().artifact_inspection.custody_locator}</p>
            <p>manifest：{item().artifact_inspection.manifest_present ? "存在" : "缺失"} · artifact：{item().artifact_inspection.artifact_present ? "存在" : "缺失"}</p>
            <Show when={item().artifact_inspection.manifest}>{(manifest) => <p>构建者 {manifest().reproduced_by} · 代码 {manifest().runner_code_revision} · {manifest().artifact_byte_length} bytes</p>}</Show>
          </article>}</Show>
          <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationVerdict)}>
            <option value="changes_requested_rebuild_artifact">要求修改并重建工件</option>
            <option value="rejected">拒绝</option>
            <option value="approved_for_one_future_claim_first_observation_ledger_transition_attempt">批准未来一次 claim-first 尝试候选</option>
          </select></label>
          <label><span>工件复现复核证据</span><textarea value={artifactEvidence()} onInput={(event) => setArtifactEvidence(event.currentTarget.value)} /></label>
          <label><span>隔离合同复核证据</span><textarea value={sandboxEvidence()} onInput={(event) => setSandboxEvidence(event.currentTarget.value)} /></label>
          <label><span>复核依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
          <div class="public-admin-decision-checks"><For each={REVIEW_CHECKS}>{(label, index) => (
            <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
          )}</For></div>
          <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在保存复核…" : "保存 Stage 120 独立复核"}</button>
        </Show>
        <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
        <For each={current().items}>{(item) => <article class="public-admin-reward-governance">
          <header><strong>{item.runner.runner_name}</strong><span>{item.latest_review?.verdict ?? item.artifact_inspection.status}</span></header>
          <p>冻结工件 {item.runner.runner_contract.proposed_runner_artifact_sha256.slice(0, 16)}… · manifest {item.artifact_inspection.manifest?.manifest_sha256.slice(0, 16) ?? "未就绪"}…</p>
          <Show when={item.latest_review}>{(review) => <p>服务端摘要 {review().server_computed_artifact_sha256.slice(0, 16)}… · 有效至 {review().authorization_valid_until}</p>}</Show>
          <p class="public-admin-anchor-boundary">{item.future_claim_eligible ? "仅获得未来 Stage 121 一次性 claim-first 候选，尚未执行。" : "当前没有执行资格。"}</p>
        </article>}</For>
      </section>
    )}</Show>
  );
}
