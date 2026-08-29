import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowExperimentFirstExecutionAuthorizations,
  reviewControlledShadowExperimentFirstExecutionAuthorization,
} from "@/lib/api";
import type {
  ControlledShadowExperimentFirstExecutionAuthorizationRegistry,
  ControlledShadowExperimentFirstExecutionAuthorizationVerdict,
} from "@/lib/types";

const AUTHORIZATION_CHECKS = [
  "已逐项核对精确 Stage 51–78、runner 规格、实现复核和完整上游内容寻址绑定",
  "复核者独立于 Stage 78 登记人、Stage 77 复核者及完整上游全部历史角色",
  "已独立重算 runner 规格、runner 合同、实现、实现合同、实现复核和完整上游哈希链",
  "已独立复现 runner 可执行工件摘要",
  "已确认代码版本可复现且精确工件可获得",
  "当前没有 callable entrypoint 或输入挂载",
  "未来输入最多单次、点时、只读、内容寻址并受 allowlist 限制；当前不附加输入",
  "未来输出必须 create-once、不可信且另行独立校验，不得包含订单或券商 payload",
  "确定性回放、只做多、仓位上限、成本、反事实、观察点与停止规则全部固定",
  "非特权身份、只读根文件系统、临时工作目录和全部资源上限固定",
  "不继承环境，不注入密钥，无网络、工具、子进程或生产读写",
  "不写模型/指标库，不回流训练，不定义综合分或奖励",
  "授权只在 24 小时内有效并且最多消费一次",
  "授权、claim、执行与输出独立校验职责严格分离",
  "本复核不附加输入、不运行影子盘、不建账本/持仓/订单，不接券商或交易",
  "批准只开放未来 Stage 80 claim-first 单次隔离影子执行尝试",
  "没有把未经老王确认的候选或外部材料冒充 Hari/老王确认逻辑",
] as const;

export function PublicAdminControlledShadowExperimentFirstExecutionAuthorizationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowExperimentFirstExecutionAuthorizationRegistry>();
  const [selectedRunnerId, setSelectedRunnerId] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<ControlledShadowExperimentFirstExecutionAuthorizationVerdict>(
      "changes_requested",
    );
  const [rationale, setRationale] = createSignal("");
  const [checks, setChecks] = createSignal(AUTHORIZATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowExperimentFirstExecutionAuthorizations();
      setRegistry(next);
      if (!next.items.some((item) => item.runner.isolated_runner_id === selectedRunnerId())) {
        setSelectedRunnerId(next.items[0]?.runner.isolated_runner_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "受控影子首次执行授权复核读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find((item) => item.runner.isolated_runner_id === selectedRunnerId()),
  );
  const approvalSelected = createMemo(
    () =>
      verdict() ===
      "approved_for_one_future_isolated_controlled_shadow_execution_attempt",
  );
  const disabled = createMemo(
    () =>
      busy() ||
      !selected() ||
      !rationale().trim() ||
      !checks()[1] ||
      (approvalSelected() && checks().some((value) => !value)),
  );

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) =>
      current.map((value, currentIndex) => (currentIndex === index ? checked : value)),
    );
  };

  const submit = async () => {
    const current = selected();
    if (!current || disabled()) return;
    const runner = current.runner;
    const contract = runner.runner_contract;
    const implementation = runner.implementation;
    const implementationReview = runner.implementation_review;
    const latest = current.latest_review;
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewControlledShadowExperimentFirstExecutionAuthorization(
        runner.isolated_runner_id,
        {
          expected_review_id: latest?.review_id,
          expected_review_sha256: latest?.review_sha256,
          expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256,
          expected_runner_contract_sha256: contract.contract_sha256,
          expected_runner_spec_revision: runner.runner_spec_revision,
          expected_runner_code_revision: runner.runner_code_revision,
          expected_runner_artifact_sha256: runner.runner_artifact_sha256,
          expected_implementation_id: implementation.implementation_id,
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_contract_sha256:
            implementation.implementation_contract.contract_sha256,
          expected_implementation_review_id: implementationReview.review_id,
          expected_implementation_review_sha256: implementationReview.review_sha256,
          expected_independent_audit_sha256: implementationReview.independent_audit.audit_sha256,
          expected_design_review_sha256: contract.stage_75_design_review_sha256,
          expected_design_registration_sha256: contract.stage_74_design_registration_sha256,
          expected_design_specification_sha256: contract.design_specification_sha256,
          expected_selected_algorithm_three_seed_binding_sha256:
            contract.exact_approved_implementation_contract
              .selected_algorithm_three_seed_binding_sha256,
          expected_sealed_holdout_split_commitment_sha256:
            contract.exact_approved_implementation_contract
              .sealed_holdout_split_commitment_sha256,
          expected_feature_order_sha256:
            contract.exact_approved_implementation_contract.feature_order_sha256,
          expected_preprocessing_sha256:
            contract.exact_approved_implementation_contract.preprocessing_sha256,
          expected_target_id: contract.exact_approved_implementation_contract.target_id,
          expected_frozen_candidate_algorithm_id:
            contract.exact_approved_implementation_contract.frozen_candidate_algorithm_id,
          verdict: verdict(),
          rationale: rationale().trim(),
          exact_current_stage_51_through_stage_78_binding_confirmed: confirmed[0],
          reviewer_independence_from_stage_78_and_complete_prior_chain_confirmed: confirmed[1],
          runner_specification_contract_and_complete_hash_chain_independently_reproduced_confirmed:
            confirmed[2],
          runner_artifact_digest_independently_reproduced:
            confirmed[3],
          immutable_code_revision_reproducible_and_artifact_available_confirmed: confirmed[4],
          no_callable_entrypoint_or_current_mount_confirmed: confirmed[5],
          future_single_use_point_in_time_read_only_content_addressed_allowlisted_input_confirmed:
            confirmed[6],
          future_create_once_untrusted_independently_validated_no_order_payload_output_confirmed:
            confirmed[7],
          deterministic_replay_long_only_caps_costs_counterfactuals_observations_and_stop_rules_confirmed:
            confirmed[8],
          fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
            confirmed[9],
          no_environment_secret_network_tool_subprocess_or_production_io_confirmed: confirmed[10],
          no_model_metric_store_training_feedback_composite_or_reward_confirmed: confirmed[11],
          authorization_single_use_and_24_hour_expiry_confirmed: confirmed[12],
          authorization_claim_execution_and_output_validation_separation_confirmed: confirmed[13],
          no_input_attachment_shadow_run_ledger_position_order_broker_or_trading_confirmed:
            confirmed[14],
          approval_only_opens_future_stage_80_claim_first_execution_attempt_confirmed:
            confirmed[15],
          no_unconfirmed_hari_or_old_wang_logic_claimed: confirmed[16],
        },
      );
      setRegistry(next);
      setRationale("");
      setChecks(AUTHORIZATION_CHECKS.map(() => false));
      setNotice(
        approvalSelected()
          ? "已追加独立批准：24 小时内最多允许一次未来 Stage 80 claim-first 尝试；当前未 claim、未附加输入、未运行。"
          : "已追加复核记录；没有授予未来执行尝试资格。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "受控影子首次执行授权复核失败");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="受控影子首次执行授权复核">
          <header>
            <strong>第 79 阶段 · 受控影子首次执行授权复核</strong>
            <span>{currentRegistry().authorization_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可复核规格</span><strong>{currentRegistry().review_eligible_runner_count}</strong></div>
            <div><span>已复核</span><strong>{currentRegistry().reviewed_runner_count}</strong></div>
            <div><span>有效单次资格</span><strong>{currentRegistry().unexpired_authorization_count}</strong></div>
            <div><span>Stage 80 候选</span><strong>{currentRegistry().execution_attempt_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>独立授权，不是影子执行</strong><span>24 小时 · 最多一次</span></header>
            <p>Stage 78 已绑定精确可执行工件。本页必须独立复现工件摘要与代码版本后才建立未来一次性资格；当前仍没有 claim、执行入口、input manifest 或点时数据访问。</p>
            <p class="public-admin-anchor-boundary">下一门禁只能是另行开发和复核的 Stage 80 claim-first 尝试；影子运行、账本、持仓、模型/指标库、奖励、订单、券商和交易全部关闭。</p>
          </article>

          <Show when={currentRegistry().items.length > 0}>
            <label>
              <span>当前 registered_not_run runner 规格</span>
              <select value={selectedRunnerId()} onChange={(event) => setSelectedRunnerId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>
                  {(item) => <option value={item.runner.isolated_runner_id}>{item.runner.runner_name} · {item.runner.isolated_runner_id.slice(0, 12)}…</option>}
                </For>
              </select>
            </label>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ControlledShadowExperimentFirstExecutionAuthorizationVerdict)}>
                <option value="changes_requested">要求修改</option>
                <option value="rejected">拒绝</option>
                <option value="approved_for_one_future_isolated_controlled_shadow_execution_attempt">批准未来一次 Stage 80 claim-first 隔离影子执行尝试</option>
              </select>
            </label>
            <label><span>复核依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} placeholder="记录独立重算、Stage 51–78 完整绑定、纯规格边界、点时输入与一次性输出约束" /></label>
            <div class="public-admin-decision-checks">
              <For each={AUTHORIZATION_CHECKS}>
                {(label, index) => (
                  <label>
                    <input type="checkbox" checked={checks()[index()]} onChange={(event) => toggleCheck(index(), event.currentTarget.checked)} />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
              追加首次执行授权复核（不 claim、不执行）
            </button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={currentRegistry().items}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.runner.runner_name}</strong>
                  <span>{item.authorization_unexpired ? "单次资格有效 · 未 claim" : item.latest_review ? "已复核未授权" : "等待独立复核"}</span>
                </header>
                <p>runner spec {item.runner.isolated_runner_id} · {item.runner.isolated_runner_spec_sha256}</p>
                <Show when={item.latest_review}>{(review) => <p>review {review().review_id} · 复核人 {review().reviewer_id} · 截止 {review().authorization_valid_until} · {review().rationale}</p>}</Show>
                <p class="public-admin-anchor-boundary">本页不能 claim 或执行；输入、影子运行、账本、持仓、模型/指标库、输出校验、奖励、订单、券商和交易仍全部关闭。</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
