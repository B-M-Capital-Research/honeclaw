import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowExperimentIsolatedRunners,
  registerControlledShadowExperimentIsolatedRunner,
} from "@/lib/api";
import type { ControlledShadowExperimentIsolatedRunnerRegistry } from "@/lib/types";

const REGISTRATION_CHECKS = [
  "精确绑定当前 Stage 51–77 完整责任链",
  "登记人独立于 Stage 77 复核者和全部上游角色",
  "已复算实现复核、独立审计、实现合同和设计各层指纹",
  "runner 可执行工件、代码版本、runtime、协议和序列化均已冻结",
  "当前没有 callable entrypoint 或输入挂载",
  "未来输入必须是点时、只读、内容寻址且白名单化",
  "未来输出必须创建一次、不可信并接受独立验证",
  "确定性复演、多头仓位上限、成本、反事实和停止规则保持不变",
  "固定非特权身份、只读根目录、临时工作区和资源上限",
  "无环境继承、密钥、网络、工具、子进程或生产读写",
  "不写模型/指标库，不训练、不反馈、不合成综合分或奖励",
  "不运行影子盘，不建账本/持仓/订单，不接券商或交易",
  "登记只开放独立首次影子执行授权复核",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

const emptyText = () => ({
  runner_name: "受控影子前向复演隔离 runner 规格",
  runner_spec_revision: "controlled-shadow-runner-spec-v1",
  runner_code_revision: "",
  runner_artifact_sha256: "",
  rationale: "",
  known_limitations: "",
  future_mount_constraints: "",
  future_output_constraints: "",
});

export function PublicAdminControlledShadowExperimentIsolatedRunnerPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowExperimentIsolatedRunnerRegistry>();
  const [selectedImplementationId, setSelectedImplementationId] = createSignal("");
  const [checks, setChecks] = createSignal(REGISTRATION_CHECKS.map(() => false));
  const [text, setText] = createSignal(emptyText());
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowExperimentIsolatedRunners();
      setRegistry(next);
      if (!next.eligible_implementations.some(
        (item) => item.implementation.implementation_id === selectedImplementationId(),
      )) {
        setSelectedImplementationId(
          next.eligible_implementations[0]?.implementation.implementation_id ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "受控影子隔离 runner 规格表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() => registry()?.eligible_implementations.find(
    (item) => item.implementation.implementation_id === selectedImplementationId(),
  ));
  const disabled = createMemo(() => busy()
    || !selected()
    || !checks().every(Boolean)
    || Object.values(text()).some((value) => value.trim().length === 0));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const implementation = item.implementation;
    const review = item.review;
    const contract = implementation.implementation_contract;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await registerControlledShadowExperimentIsolatedRunner(
        implementation.implementation_id,
        {
          expected_implementation_id: implementation.implementation_id,
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_contract_sha256: contract.contract_sha256,
          expected_implementation_review_id: review.review_id,
          expected_implementation_review_sha256: review.review_sha256,
          expected_independent_audit_sha256: review.independent_audit.audit_sha256,
          expected_design_review_sha256: contract.stage_75_design_review_sha256,
          expected_design_registration_sha256: contract.stage_74_design_registration_sha256,
          expected_design_specification_sha256: contract.design_specification_sha256,
          expected_selected_algorithm_three_seed_binding_sha256:
            contract.selected_algorithm_three_seed_binding_sha256,
          expected_sealed_holdout_split_commitment_sha256:
            contract.sealed_holdout_split_commitment_sha256,
          expected_feature_order_sha256: contract.feature_order_sha256,
          expected_preprocessing_sha256: contract.preprocessing_sha256,
          expected_target_id: contract.target_id,
          expected_frozen_candidate_algorithm_id: contract.frozen_candidate_algorithm_id,
          runner_kind: "ephemeral_deterministic_forward_replay_specification",
          ...text(),
          exact_current_stage_51_through_stage_77_binding_confirmed: checks()[0] as boolean,
          registrar_independent_from_stage_77_and_complete_prior_chain_confirmed:
            checks()[1] as boolean,
          implementation_review_audit_contract_and_design_hashes_reproduced_confirmed:
            checks()[2] as boolean,
          runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed:
            checks()[3] as boolean,
          no_callable_entrypoint_or_current_mount_confirmed: checks()[4] as boolean,
          future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed:
            checks()[5] as boolean,
          future_create_once_untrusted_independently_validated_output_confirmed:
            checks()[6] as boolean,
          deterministic_replay_long_only_caps_costs_counterfactuals_and_stop_rules_preserved_confirmed:
            checks()[7] as boolean,
          fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed:
            checks()[8] as boolean,
          no_environment_secret_network_tool_subprocess_or_production_io_confirmed:
            checks()[9] as boolean,
          no_model_metric_store_training_feedback_composite_or_reward_confirmed:
            checks()[10] as boolean,
          no_shadow_run_ledger_position_order_broker_or_trading_confirmed:
            checks()[11] as boolean,
          registration_only_opens_independent_first_execution_authorization_review_confirmed:
            checks()[12] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[13] as boolean,
        },
      );
      setRegistry(next);
      setChecks(REGISTRATION_CHECKS.map(() => false));
      setText(emptyText());
      setNotice("隔离 runner 规格已绑定可执行工件并创建一次写入；没有入口、挂载或运行权限。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "受控影子隔离 runner 规格登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(current) => (
        <section class="public-admin-reward-governance" aria-label="受控影子隔离 runner 规格登记">
          <header>
            <strong>第 78 阶段 · 隔离 runner 规格登记</strong>
            <span>{current().runner_status}</span>
          </header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待登记规格</span><strong>{current().registration_eligible_count}</strong></div>
            <div><span>已登记规格</span><strong>{current().runner_count}</strong></div>
            <div><span>当前绑定</span><strong>{current().current_binding_runner_count}</strong></div>
            <div><span>可进入首次授权复核</span><strong>{current().first_execution_authorization_review_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>工件已绑定，入口仍关闭</strong><span>零能力</span></header>
            <p>本阶段冻结 runner 工件摘要、代码版本、runtime、未来输入输出、身份和资源约束。</p>
            <p class="public-admin-anchor-boundary">当前无 callable entrypoint、挂载、数据访问、影子运行、账本、持仓、订单、券商或交易权限。</p>
          </article>

          <Show when={current().eligible_implementations.length > 0} fallback={<p>当前没有可登记的 Stage 77 独立批准实现。</p>}>
            <label>
              <span>已独立批准实现</span>
              <select value={selectedImplementationId()} onChange={(event) => setSelectedImplementationId(event.currentTarget.value)}>
                <For each={current().eligible_implementations}>{(item) => (
                  <option value={item.implementation.implementation_id}>
                    {item.implementation.implementation_id.slice(0, 12)}… · {item.implementation.implementation_contract.target_id}
                  </option>
                )}</For>
              </select>
            </label>
            <label><span>runner 规格名称</span><input value={text().runner_name} onInput={(event) => setText((value) => ({ ...value, runner_name: event.currentTarget.value }))} /></label>
            <label><span>规格版本</span><input value={text().runner_spec_revision} onInput={(event) => setText((value) => ({ ...value, runner_spec_revision: event.currentTarget.value }))} /></label>
            <label><span>代码版本</span><input value={text().runner_code_revision} onInput={(event) => setText((value) => ({ ...value, runner_code_revision: event.currentTarget.value }))} /></label>
            <label><span>可执行工件 SHA-256</span><input value={text().runner_artifact_sha256} onInput={(event) => setText((value) => ({ ...value, runner_artifact_sha256: event.currentTarget.value }))} /></label>
            <label><span>登记理由</span><textarea value={text().rationale} onInput={(event) => setText((value) => ({ ...value, rationale: event.currentTarget.value }))} /></label>
            <label><span>已知局限</span><textarea value={text().known_limitations} onInput={(event) => setText((value) => ({ ...value, known_limitations: event.currentTarget.value }))} /></label>
            <label><span>未来挂载约束</span><textarea value={text().future_mount_constraints} onInput={(event) => setText((value) => ({ ...value, future_mount_constraints: event.currentTarget.value }))} /></label>
            <label><span>未来输出约束</span><textarea value={text().future_output_constraints} onInput={(event) => setText((value) => ({ ...value, future_output_constraints: event.currentTarget.value }))} /></label>
            <div class="public-admin-decision-checks">
              <For each={REGISTRATION_CHECKS}>{(label, index) => (
                <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
              )}</For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在登记规格…" : "登记工件绑定 runner 规格"}</button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={current().items}>{(item) => (
            <article class="public-admin-reward-governance">
              <header><strong>{item.runner.runner_name}</strong><span>{item.runner.status}</span></header>
              <p>{item.runner.registered_at} · {item.runner.registered_by} · 工件 {item.runner.runner_artifact_sha256.slice(0, 16)}…</p>
              <p>未来输入：只读/点时/内容寻址/白名单；未来输出：创建一次/不可信/独立验证。</p>
              <p>资源上限：{item.runner.runner_contract.maximum_memory_mib} MiB · {item.runner.runner_contract.maximum_cpu_millicores} mCPU · {item.runner.runner_contract.maximum_wall_clock_seconds} 秒 · 单进程。</p>
              <p class="public-admin-anchor-boundary">规格已登记 ≠ 可执行；下一步仅为责任链外首次影子执行授权复核。</p>
            </article>
          )}</For>
        </section>
      )}
    </Show>
  );
}
