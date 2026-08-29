import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowForwardObservationImplementations,
  registerControlledShadowForwardObservationImplementation,
} from "@/lib/api";
import type { ControlledShadowForwardObservationImplementationRegistry } from "@/lib/types";

const CHECKS = [
  "精确绑定当前 Stage 51–83 完整责任链",
  "登记人独立于 Stage 83 复核人和全部上游角色",
  "已独立重算 Stage 83 复核、Stage 82 登记、协议与完整 Stage 74 设计指纹",
  "本次只登记零能力规格，不声称存在可执行工件",
  "保持自然前向、禁止回填及 observation_not_before",
  "保持每周 claim-first、create-once 和内容寻址点时输入",
  "保持官方交易日历、证券/SPY 同步及公司行动证据",
  "保持下一完整交易日、单边 25bp、反事实与仅多头组合上限",
  "保持检查点、最低样本、分项指标、多重检验和停止规则",
  "未来输入、claim、输出和更正均确定性、内容寻址且追加写入",
  "无入口、可执行工件、runtime、挂载、适配器、环境继承、密钥、网络、工具或子进程",
  "无生产读写、观察写入、账本创建、持仓或绩效写入",
  "不写模型/指标库，不反馈训练，不定义综合 reward，不生成订单、不接券商或交易",
  "未来必须先经独立实现复核，才可考虑隔离 runner 规格登记",
  "未把未确认 Hari/老王观点写成工程规则",
] as const;

const emptyFields = () => ({
  implementation_name: "",
  immutable_code_revision: "",
  implementation_description: "",
  deterministic_observation_semantics: "",
  evidence_custody_and_correction_semantics: "",
  known_limitations: "",
  future_review_constraints: "",
});

export function PublicAdminControlledShadowForwardObservationImplementationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowForwardObservationImplementationRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [fields, setFields] = createSignal(emptyFields());
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowForwardObservationImplementations();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.registration_eligible);
      if (!eligible.some((item) => item.source.review.review_id === selectedId())) {
        setSelectedId(eligible[0]?.source.review.review_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 84 前向观察实现登记表读取失败");
    }
  };
  onMount(() => void load());

  const eligible = createMemo(() => registry()?.items.filter((item) => item.registration_eligible) ?? []);
  const selected = createMemo(() => eligible().find((item) => item.source.review.review_id === selectedId()));
  const disabled = createMemo(() => busy() || !selected()
    || Object.values(fields()).some((value) => !value.trim())
    || checks().some((value) => !value));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const registration = item.source.registered_protocol.registration;
    const review = item.source.review;
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await registerControlledShadowForwardObservationImplementation(
        review.review_id,
        {
          expected_protocol_review_id: review.review_id,
          expected_protocol_review_sha256: review.review_sha256,
          expected_protocol_registration_id: registration.protocol_registration_id,
          expected_protocol_registration_sha256: registration.protocol_registration_sha256,
          expected_protocol_specification_sha256: registration.protocol_specification.specification_sha256,
          expected_validation_sha256: registration.validation_sha256,
          expected_claim_sha256: registration.claim_sha256,
          expected_result_sha256: registration.result_sha256,
          expected_output_sha256: registration.output_sha256,
          expected_input_manifest_sha256: registration.input_manifest_sha256,
          expected_authorization_review_sha256: registration.authorization_review_sha256,
          expected_isolated_runner_spec_sha256: registration.isolated_runner_spec_sha256,
          expected_runner_artifact_sha256: registration.runner_artifact_sha256,
          expected_implementation_contract_sha256: registration.implementation_contract_sha256,
          expected_design_specification_sha256: registration.design_specification_sha256,
          expected_candidate_set_sha256: registration.candidate_set_sha256,
          expected_feature_order_sha256: registration.feature_order_sha256,
          expected_preprocessing_sha256: registration.preprocessing_sha256,
          expected_target_id: registration.target_id,
          expected_frozen_candidate_algorithm_id: registration.frozen_candidate_algorithm_id,
          ...fields(),
          exact_current_stage_51_through_stage_83_binding_confirmed: true,
          registrar_independent_from_stage_83_and_complete_prior_chain_confirmed: true,
          independent_recomputation_of_review_registration_protocol_and_design_confirmed: true,
          zero_capability_specification_only_no_executable_artifact_confirmed: true,
          natural_forward_no_backfill_and_observation_not_before_preserved_confirmed: true,
          weekly_claim_first_create_once_and_point_in_time_input_preserved_confirmed: true,
          official_market_calendar_spy_sync_and_corporate_actions_preserved_confirmed: true,
          next_full_session_25bps_cost_counterfactual_and_long_only_caps_preserved_confirmed: true,
          checkpoints_minimum_samples_separate_metrics_multiple_testing_and_stop_preserved_confirmed: true,
          deterministic_content_addressed_input_claim_output_and_correction_contract_confirmed: true,
          no_entrypoint_artifact_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed: true,
          no_production_read_write_observation_ledger_position_or_performance_write_confirmed: true,
          no_model_metric_training_feedback_composite_reward_order_broker_or_trading_confirmed: true,
          future_independent_implementation_review_required_before_runner_registration_confirmed: true,
          no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        },
      );
      setRegistry(next); setFields(emptyFields()); setChecks(CHECKS.map(() => false));
      setNotice("Stage 84 零能力实现规格已 create-once 登记；规格，不是程序，也没有开始观察。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 84 前向观察实现登记失败");
      await load();
    } finally { setBusy(false); }
  };

  const textFields = [
    ["implementation_name", "实现规格名称"],
    ["immutable_code_revision", "不可变代码版本标识（仅声明，不上传工件）"],
    ["implementation_description", "实现边界与职责"],
    ["deterministic_observation_semantics", "确定性周度观察语义"],
    ["evidence_custody_and_correction_semantics", "点时证据托管、公司行动与追加更正语义"],
    ["known_limitations", "已知局限"],
    ["future_review_constraints", "未来独立复核约束"],
  ] as const;

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="Stage 84 前向观察零能力实现规格登记">
      <header><strong>第 84 阶段 · 前向观察零能力实现规格登记</strong><span>{current().implementation_status}</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>待登记</span><strong>{current().registration_eligible_count}</strong></div>
        <div><span>已登记</span><strong>{current().implementation_count}</strong></div>
        <div><span>当前绑定</span><strong>{current().current_binding_implementation_count}</strong></div>
        <div><span>待独立复核</span><strong>{current().independent_implementation_review_eligible_count}</strong></div>
      </div>
      <p class="public-admin-anchor-boundary">本页只冻结纯函数标识与未来数据合同：不创建输入挂载、观察账本、持仓或绩效；不运行影子盘，不建账本/持仓/订单，不接券商或交易。</p>
      <Show when={eligible().length > 0} fallback={<p>当前没有经 Stage 83 独立批准、可登记的前向观察协议。</p>}>
        <label><span>Stage 83 独立批准记录</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={eligible()}>{(item) => <option value={item.source.review.review_id}>{item.source.review.review_id.slice(0, 12)}… · {item.source.registered_protocol.registration.target_id}</option>}</For></select></label>
        <For each={textFields}>{([key, label]) => <label><span>{label}</span><textarea value={fields()[key]} onInput={(event) => setFields((value) => ({ ...value, [key]: event.currentTarget.value }))} /></label>}</For>
        <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, i) => i === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在冻结规格…" : "登记 Stage 84 零能力实现规格"}</button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().items}>{(item) => <Show when={item.implementation}>{(implementation) => <article class="public-admin-reward-governance"><header><strong>{implementation().implementation_name}</strong><span>{implementation().status}</span></header><p>{implementation().registered_at} · {implementation().registered_by}</p><p><strong>版本：</strong>{implementation().implementation_contract.immutable_code_revision}</p><p><strong>确定性语义：</strong>{implementation().deterministic_observation_semantics}</p><p class="public-admin-anchor-boundary">规格，不是程序 · 无 callable entrypoint · 观察、账本、持仓、绩效、订单、券商与交易权限全部关闭。</p></article>}</Show>}</For>
    </section>
  )}</Show>;
}
