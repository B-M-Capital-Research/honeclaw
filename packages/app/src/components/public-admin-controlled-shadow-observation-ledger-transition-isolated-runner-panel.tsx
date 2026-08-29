import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationLedgerTransitionIsolatedRunners,
  registerControlledShadowObservationLedgerTransitionIsolatedRunnerOnce,
} from "@/lib/api";
import type {
  ControlledShadowObservationLedgerTransitionIsolatedRunnerRegistry,
  RegisterControlledShadowObservationLedgerTransitionIsolatedRunnerRequest,
} from "@/lib/types";

const CHECKS = [
  "精确绑定 Stage 51–118 当前不可变责任链",
  "登记人独立于 Stage 118 和完整此前责任链",
  "已复算实现、复核、审计、规格与 Stage 114 准入全链哈希",
  "未来工件身份、代码版本和复现步骤已绑定，但当前工件不存在",
  "八个观察到账本转换函数身份和规范化 schema 保持不变",
  "未来输入只允许 Stage 114 已准入、只读、内容寻址的精确输出",
  "交易日、三价格口径、缺口、公司行动、初始组合与可用时间语义保持不变",
  "禁止覆盖、回填、前填、插值、跨口径替代或推断公司行动",
  "未来输出必须内容寻址、create-once、不可信并独立验证",
  "当前期初组合快照不存在，金融事件白名单保持为空",
  "未来金融事件必须等待期初组合快照另行独立准入",
  "供应商发布时间仍未验证，须等待独立证据",
  "固定非特权身份、只读根目录、临时工作目录和资源上限",
  "没有源码、可执行工件、入口、runtime、挂载、读取、环境、秘密、网络、工具或子进程",
  "没有观察信封、账本、持仓、绩效、模型、训练、奖励、订单、券商或交易权限",
  "登记只开放 Stage 120 责任链外首次执行授权复核资格",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

const initialFields = () => ({
  runner_name: "自然前向观察到账本转换隔离 runner",
  runner_spec_revision: "v1",
  proposed_runner_code_revision: "",
  proposed_runner_artifact_sha256: "",
  artifact_reproduction_procedure: "",
  rationale: "",
  known_limitations: "",
  future_input_constraints: "",
  future_output_constraints: "",
});

export function PublicAdminControlledShadowObservationLedgerTransitionIsolatedRunnerPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowObservationLedgerTransitionIsolatedRunnerRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [fields, setFields] = createSignal(initialFields());
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationLedgerTransitionIsolatedRunners();
      setRegistry(next);
      if (!next.eligible_implementations.some((item) => item.implementation.implementation_id === selectedId())) {
        setSelectedId(next.eligible_implementations[0]?.implementation.implementation_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 119 runner 规格登记表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.eligible_implementations.find(
      (item) => item.implementation.implementation_id === selectedId(),
    ),
  );
  const disabled = createMemo(
    () =>
      busy()
      || !selected()
      || Object.values(fields()).some((value) => !value.trim())
      || !/^[0-9a-fA-F]{64}$/.test(fields().proposed_runner_artifact_sha256)
      || checks().some((value) => !value),
  );

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const implementation = item.implementation;
    const review = item.review;
    const contract = implementation.implementation_contract;
    const registration = implementation.upstream_specification_registration;
    const specification = registration.specification;
    const request: RegisterControlledShadowObservationLedgerTransitionIsolatedRunnerRequest = {
      expected_implementation_id: implementation.implementation_id,
      expected_implementation_sha256: implementation.implementation_sha256,
      expected_implementation_contract_sha256: contract.contract_sha256,
      expected_implementation_review_id: review.review_id,
      expected_implementation_review_sha256: review.review_sha256,
      expected_independent_audit_sha256: review.independent_audit.audit_sha256,
      expected_specification_review_sha256: implementation.upstream_specification_review.review_sha256,
      expected_specification_registration_sha256: registration.registration_sha256,
      expected_observation_ledger_transition_specification_sha256: specification.specification_sha256,
      expected_stage_114_admission_review_sha256: specification.stage_114_review_sha256,
      expected_stage_113_validation_sha256: specification.stage_113_validation_sha256,
      expected_stage_112_result_sha256: specification.stage_112_result_sha256,
      expected_stage_111_claim_sha256: specification.stage_111_claim_sha256,
      runner_kind: "ephemeral_deterministic_observation_ledger_transition_specification",
      ...fields(),
      exact_current_stage_51_through_stage_118_binding_confirmed: true,
      registrar_independent_from_stage_118_and_complete_prior_chain_confirmed: true,
      implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed: true,
      proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: true,
      all_eight_observation_ledger_transition_functions_and_canonical_schemas_preserved_confirmed: true,
      future_input_only_stage_114_admitted_read_only_content_addressed_output_confirmed: true,
      session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed: true,
      no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_preserved_confirmed: true,
      future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
      opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: true,
      future_financial_events_require_separately_admitted_opening_snapshot_confirmed: true,
      provider_publication_time_remains_unverified_until_separate_evidence_confirmed: true,
      fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: true,
      no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      registration_only_opens_chain_external_first_execution_authorization_review_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setRegistry(
        await registerControlledShadowObservationLedgerTransitionIsolatedRunnerOnce(
          implementation.implementation_id,
          request,
        ),
      );
      setFields(initialFields());
      setChecks(CHECKS.map(() => false));
      setNotice("Stage 119 runner 规格已 create-once 登记；当前仍没有工件、入口、runtime 或数据访问能力。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 119 runner 规格登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  const textFields = [
    ["runner_name", "runner 名称"],
    ["runner_spec_revision", "runner 规格版本"],
    ["proposed_runner_code_revision", "未来不可变代码版本"],
    ["proposed_runner_artifact_sha256", "未来工件 SHA-256（64 位）"],
    ["artifact_reproduction_procedure", "工件独立复现步骤"],
    ["rationale", "登记理由"],
    ["known_limitations", "已知限制"],
    ["future_input_constraints", "未来只读输入约束"],
    ["future_output_constraints", "未来 create-once 输出约束"],
  ] as const;

  return (
    <Show when={registry()}>
      {(current) => (
        <section class="public-admin-reward-governance" aria-label="Stage 119 观察到账本转换隔离 runner 规格登记">
          <header>
            <strong>第 119 阶段 · 观察到账本转换隔离 runner 规格登记</strong>
            <span>{current().runner_status}</span>
          </header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可登记</span><strong>{current().registration_eligible_count}</strong></div>
            <div><span>已登记</span><strong>{current().runner_count}</strong></div>
            <div><span>当前绑定</span><strong>{current().current_binding_runner_count}</strong></div>
            <div><span>待 Stage 120</span><strong>{current().first_execution_authorization_review_eligible_count}</strong></div>
          </div>
          <p class="public-admin-anchor-boundary">
            这里只冻结未来 runner 身份与沙箱合同。登记后仍没有工件、入口、runtime、挂载或准入输入读取权限；期初组合快照不存在，金融事件白名单为空。
          </p>
          <Show when={current().registration_eligible_count > 0} fallback={<p>当前没有可登记的 Stage 118 独立批准实现。</p>}>
            <label>
              <span>Stage 118 独立批准实现</span>
              <select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}>
                <For each={current().eligible_implementations}>
                  {(item) => (
                    <option value={item.implementation.implementation_id}>
                      {item.implementation.implementation_id.slice(0, 12)}… · {item.implementation.implementation_name}
                    </option>
                  )}
                </For>
              </select>
            </label>
            <For each={textFields}>
              {([key, label]) => (
                <label>
                  <span>{label}</span>
                  <textarea
                    value={fields()[key]}
                    onInput={(event) => setFields((value) => ({ ...value, [key]: event.currentTarget.value }))}
                  />
                </label>
              )}
            </For>
            <div class="public-admin-decision-checks">
              <For each={CHECKS}>
                {(label, index) => (
                  <label>
                    <input
                      type="checkbox"
                      checked={checks()[index()]}
                      onChange={(event) =>
                        setChecks((values) => values.map((value, i) => i === index() ? event.currentTarget.checked : value))
                      }
                    />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
              {busy() ? "正在登记…" : "写入 Stage 119 create-once 规格"}
            </button>
          </Show>
          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={current().items}>
            {(item) => (
              <article class="public-admin-decision-card">
                <strong>{item.runner.runner_name}</strong>
                <span>{item.runner.status}</span>
                <p>规格 {item.runner.isolated_runner_spec_sha256.slice(0, 16)}… · 工件身份 {item.runner.runner_contract.proposed_runner_artifact_sha256.slice(0, 16)}…</p>
                <p>资源：{item.runner.runner_contract.maximum_memory_mib} MiB / {item.runner.runner_contract.maximum_wall_clock_seconds}s / {item.runner.runner_contract.maximum_process_count} 进程</p>
                <p>工件存在：否 · runtime 实例化：否 · 输入读取：否 · 期初组合：否 · 金融事件白名单：空</p>
                <p>账本/事件/持仓/现金/NAV：否 · 模型/训练/RL：否 · 下单/券商/交易：否</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
