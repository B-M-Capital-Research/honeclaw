import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationLedgerTransitionImplementations,
  registerControlledShadowObservationLedgerTransitionImplementationOnce,
} from "@/lib/api";
import type {
  ControlledShadowObservationLedgerTransitionImplementationRegistry,
  RegisterControlledShadowObservationLedgerTransitionImplementationRequest,
} from "@/lib/types";

const CHECKS = [
  "精确绑定 Stage 51–116 当前不可变责任链",
  "登记者不是 Stage 116 复核者或此前完整责任链成员",
  "独立重算 review、registration、specification 与 audit 指纹",
  "只登记零能力契约，不提交源码或可执行制品",
  "未来观察输入仍只能是 exact Stage 114 已准入 envelope",
  "Stage 88 绑定只作为初始化来源，不能当作 opening positions",
  "opening portfolio snapshot 必须另行准入，不默认或推断本金、现金、持仓、股数或权重",
  "raw close 是唯一证券会计口径；adjusted prices 非会计且不重复计入",
  "显式 gap 阻断 NAV，不前填、不插值、不跨口径替代",
  "分红、拆股在持仓与有效条款获准前只保持 notice",
  "保留 exact decimal、append-only、幂等事件、稳定顺序与双重记账",
  "更正只允许由新准入证据追加 superseding 或 reversal 事件，绝不改写历史",
  "保守 available_at 与未验证 provider 发布时间限制保持不变",
  "未来 ledger/event stream create-once 且非可信，必须独立校验",
  "没有入口、runtime、输入挂载/读取、环境、秘密、网络、工具、子进程或生产 I/O",
  "不创建 opening snapshot、账本/事件、持仓、现金、NAV/绩效、模型、训练/RL、奖励、订单、券商或交易事实",
  "隔离 runner 登记前必须先完成 Stage 118 责任链外独立实现复核",
  "没有把未确认的 Hari/老王观点写成系统规则",
] as const;

const emptyFields = () => ({
  implementation_name: "",
  immutable_code_revision: "",
  implementation_description: "",
  deterministic_projection_semantics: "",
  session_price_basis_and_gap_semantics: "",
  corporate_action_decimal_order_and_hash_semantics: "",
  initial_allocation_and_availability_semantics: "",
  error_and_missing_data_semantics: "",
  known_limitations: "",
  future_review_constraints: "",
});

export function PublicAdminControlledShadowObservationLedgerTransitionImplementationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowObservationLedgerTransitionImplementationRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [fields, setFields] = createSignal(emptyFields());
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationLedgerTransitionImplementations();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.registration_eligible);
      if (!eligible.some((item) => item.specification_review.review_id === selectedId())) {
        setSelectedId(eligible[0]?.specification_review.review_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 117 零能力实现登记表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.items.find(
    (item) => item.registration_eligible && item.specification_review.review_id === selectedId(),
  ));
  const disabled = createMemo(() => busy() || !selected()
    || Object.values(fields()).some((value) => !value.trim())
    || checks().some((value) => !value));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const review = item.specification_review;
    const registration = item.specification_registration;
    const request: RegisterControlledShadowObservationLedgerTransitionImplementationRequest = {
      expected_specification_review_id: review.review_id,
      expected_specification_review_sha256: review.review_sha256,
      expected_independent_audit_sha256: review.independent_audit.audit_sha256,
      expected_registration_id: registration.registration_id,
      expected_registration_sha256: registration.registration_sha256,
      expected_specification_sha256: registration.specification.specification_sha256,
      ...fields(),
      exact_stage_51_through_stage_116_binding_confirmed: true,
      registrar_independent_from_stage_116_and_complete_prior_chain_confirmed: true,
      independent_recomputation_of_review_registration_specification_and_audit_confirmed: true,
      zero_capability_contract_only_no_source_or_executable_artifact_confirmed: true,
      exact_stage_114_admitted_output_is_only_future_input_confirmed: true,
      official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
      subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: true,
      dividends_splits_and_price_bases_remain_separate_confirmed: true,
      decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: true,
      initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed: true,
      conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: true,
      one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: true,
      future_output_untrusted_and_independent_validation_required_confirmed: true,
      no_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
      no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
      future_independent_implementation_review_required_before_isolated_runner_registration_confirmed: true,
      no_unconfirmed_hari_or_old_wang_logic_claimed: true,
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setRegistry(await registerControlledShadowObservationLedgerTransitionImplementationOnce(review.review_id, request));
      setFields(emptyFields());
      setChecks(CHECKS.map(() => false));
      setNotice("Stage 117 零能力实现契约已 create-once 写入；仍须 Stage 118 责任链外独立实现复核。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 117 零能力实现契约登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  const textFields = [
    ["implementation_name", "实现契约名称"],
    ["immutable_code_revision", "不可变代码版本标识（仅标识，不上传代码）"],
    ["implementation_description", "零能力实现说明"],
    ["deterministic_projection_semantics", "确定性观察到账本转换语义"],
    ["session_price_basis_and_gap_semantics", "raw/adjusted 价格、显式缺口与 NAV 语义"],
    ["corporate_action_decimal_order_and_hash_semantics", "公司行动、exact decimal、幂等、顺序与双分录语义"],
    ["initial_allocation_and_availability_semantics", "opening portfolio 前置门槛与可用时间语义"],
    ["error_and_missing_data_semantics", "错误与缺失数据语义"],
    ["known_limitations", "已知限制"],
    ["future_review_constraints", "Stage 118 独立复核约束"],
  ] as const;

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="Stage 117 观察到账本转换零能力实现契约登记">
      <header><strong>第 117 阶段 · 观察到账本转换零能力实现契约登记</strong><span>{current().implementation_status}</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>独立批准规格</span><strong>{current().independently_approved_specification_count}</strong></div>
        <div><span>待登记</span><strong>{current().registration_eligible_count}</strong></div>
        <div><span>契约</span><strong>{current().implementation_contract_count}</strong></div>
        <div><span>待 Stage 118 复核</span><strong>{current().independent_implementation_review_eligible_count}</strong></div>
      </div>
      <p class="public-admin-anchor-boundary">这里只冻结纯函数标识、schema、哈希和失败关闭语义；opening snapshot 仍缺失，也没有源码、可执行入口、runtime、输入读取、账本事件或任何财务输出。</p>
      <Show when={current().registration_eligible_count > 0} fallback={<p>当前没有待登记的 Stage 116 独立批准规格。</p>}>
        <label><span>Stage 116 独立批准规格</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={current().items.filter((item) => item.registration_eligible)}>{(item) => <option value={item.specification_review.review_id}>{item.specification_review.review_id.slice(0, 12)}… · {item.specification_registration.specification.subject_symbols.join("、")}</option>}</For></select></label>
        <For each={textFields}>{([key, label]) => <label><span>{label}</span><textarea value={fields()[key]} onInput={(event) => setFields((value) => ({ ...value, [key]: event.currentTarget.value }))} /></label>}</For>
        <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, i) => i === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在登记…" : "写入 Stage 117 零能力实现契约"}</button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().items.filter((item) => item.implementation)}>{(item) => <article class="public-admin-reward-governance"><header><strong>implementation {item.implementation!.implementation_id}</strong><span>{item.implementation!.status}</span></header><p>{item.implementation!.implementation_name} · contract {item.implementation!.implementation_contract.contract_sha256.slice(0, 16)}…</p><p class="public-admin-anchor-boundary">零能力契约已登记；opening snapshot 与源码工件仍不存在，须先完成 Stage 118 独立复核，不能运行、读取输入或产生财务分录。</p></article>}</For>
    </section>
  )}</Show>;
}
