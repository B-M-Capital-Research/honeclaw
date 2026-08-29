import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowMarketDataParserIsolatedRunners,
  registerControlledShadowMarketDataParserIsolatedRunnerOnce,
} from "@/lib/api";
import type {
  ControlledShadowMarketDataParserIsolatedRunnerRegistry,
  RegisterControlledShadowMarketDataParserIsolatedRunnerRequest,
} from "@/lib/types";

const CHECKS = [
  "精确绑定 Stage 51–98 当前不可变责任链",
  "登记人独立于 Stage 98 和完整此前责任链",
  "已复算实现、复核、审计、规格与收据全链哈希",
  "未来工件身份、代码版本和复现步骤已绑定，但当前工件不存在",
  "八个 parser 函数身份和规范化 schema 保持不变",
  "未来输入只允许 Stage 94 已验证、只读、内容寻址的收据载荷",
  "来源、官方日历、公司行动、数值和失败关闭语义保持不变",
  "禁止静默去重、前填、插值、回退或推断公司行动",
  "未来输出必须内容寻址、create-once、不可信并独立验证",
  "source_available_at 仍未验证，须等待独立证据",
  "固定非特权身份、只读根目录、临时工作目录和资源上限",
  "没有源码、可执行工件、入口、runtime、挂载、读取、环境、秘密、网络、工具或子进程",
  "没有解析行、观察、账本、持仓、绩效、模型、训练、奖励、订单、券商或交易权限",
  "登记只开放 Stage 100 责任链外首次执行授权复核资格",
  "没有把未确认 Hari/老王观点写成系统规则",
] as const;

const initialFields = () => ({
  runner_name: "自然前向行情隔离解析器",
  runner_spec_revision: "v1",
  proposed_runner_code_revision: "",
  proposed_runner_artifact_sha256: "",
  artifact_reproduction_procedure: "",
  rationale: "",
  known_limitations: "",
  future_input_constraints: "",
  future_output_constraints: "",
});

export function PublicAdminControlledShadowMarketDataParserIsolatedRunnerPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowMarketDataParserIsolatedRunnerRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [fields, setFields] = createSignal(initialFields());
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowMarketDataParserIsolatedRunners();
      setRegistry(next);
      if (!next.eligible_implementations.some((item) => item.implementation.implementation_id === selectedId())) {
        setSelectedId(next.eligible_implementations[0]?.implementation.implementation_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 99 runner 规格登记表读取失败");
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
    const request: RegisterControlledShadowMarketDataParserIsolatedRunnerRequest = {
      expected_implementation_id: implementation.implementation_id,
      expected_implementation_sha256: implementation.implementation_sha256,
      expected_implementation_contract_sha256: contract.contract_sha256,
      expected_implementation_review_id: review.review_id,
      expected_implementation_review_sha256: review.review_sha256,
      expected_independent_audit_sha256: review.independent_audit.audit_sha256,
      expected_specification_review_sha256: implementation.upstream_specification_review.review_sha256,
      expected_specification_registration_sha256: registration.registration_sha256,
      expected_parser_specification_sha256: registration.parser_specification.parser_specification_sha256,
      expected_validation_sha256: contract.validation_sha256,
      expected_receipt_sha256: contract.receipt_sha256,
      expected_claim_sha256: contract.claim_sha256,
      expected_result_sha256: contract.result_sha256,
      runner_kind: "ephemeral_deterministic_market_data_parser_specification",
      ...fields(),
      exact_current_stage_51_through_stage_98_binding_confirmed: true,
      registrar_independent_from_stage_98_and_complete_prior_chain_confirmed: true,
      implementation_review_audit_contract_and_parser_specification_hashes_reproduced_confirmed: true,
      proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: true,
      all_eight_parser_functions_and_canonical_schemas_preserved_confirmed: true,
      future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed: true,
      strict_source_calendar_action_numeric_and_failure_semantics_preserved_confirmed: true,
      no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed: true,
      future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
      source_available_at_remains_unverified_until_separate_evidence_confirmed: true,
      fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: true,
      no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      registration_only_opens_chain_external_first_execution_authorization_review_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setRegistry(
        await registerControlledShadowMarketDataParserIsolatedRunnerOnce(
          implementation.implementation_id,
          request,
        ),
      );
      setFields(initialFields());
      setChecks(CHECKS.map(() => false));
      setNotice("Stage 99 runner 规格已 create-once 登记；当前仍没有工件、入口、runtime 或数据访问能力。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 99 runner 规格登记失败");
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
        <section class="public-admin-reward-governance" aria-label="Stage 99 行情解析器隔离 runner 规格登记">
          <header>
            <strong>第 99 阶段 · 行情解析器隔离 runner 规格登记</strong>
            <span>{current().runner_status}</span>
          </header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可登记</span><strong>{current().registration_eligible_count}</strong></div>
            <div><span>已登记</span><strong>{current().runner_count}</strong></div>
            <div><span>当前绑定</span><strong>{current().current_binding_runner_count}</strong></div>
            <div><span>待 Stage 100</span><strong>{current().first_execution_authorization_review_eligible_count}</strong></div>
          </div>
          <p class="public-admin-anchor-boundary">
            这里只冻结未来 runner 身份与沙箱合同。登记后仍没有 parser 工件、入口、runtime、挂载或原始载荷读取权限。
          </p>
          <Show when={current().registration_eligible_count > 0} fallback={<p>当前没有可登记的 Stage 98 独立批准实现。</p>}>
            <label>
              <span>Stage 98 独立批准实现</span>
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
              {busy() ? "正在登记…" : "写入 Stage 99 create-once 规格"}
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
                <p>工件存在：否 · runtime 实例化：否 · 载荷读取：否 · 下单/券商/交易：否</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
