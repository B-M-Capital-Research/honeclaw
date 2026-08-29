import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowForwardObservationImplementationReviews,
  reviewControlledShadowForwardObservationImplementation,
} from "@/lib/api";
import type {
  ControlledShadowForwardObservationImplementationReviewRegistry,
  ControlledShadowForwardObservationImplementationReviewVerdict,
} from "@/lib/types";

const CHECKS = [
  "精确绑定当前 Stage 51–84 完整责任链",
  "复核人独立于 Stage 84 登记人、完整上游和此前复核人",
  "独立重算实现、合同、协议复核、协议登记、协议与设计六层指纹",
  "保持自然前向、禁止回填及 observation_not_before",
  "复核每周 claim、官方日历、点时证据托管与追加更正",
  "复核信号、组合、成交成本、反事实、检查点与停止八个纯函数标识",
  "三个未来 schema 只是名称，尚未创建或挂载",
  "无工件、入口、runtime、挂载、适配器、环境、密钥、网络、工具或子进程",
  "无生产读写、观察、账本、持仓或绩效写入",
  "不写模型/指标库，不训练、不奖励、不下单、不接券商、不交易",
  "批准只开放未来隔离 runner 规格登记",
  "未把未确认 Hari/老王观点写成工程规则",
] as const;

const emptyFields = () => ({
  rationale: "",
  binding_and_recomputation_assessment: "",
  deterministic_semantics_assessment: "",
  zero_capability_assessment: "",
  known_limitations: "",
  future_runner_constraints: "",
});

export function PublicAdminControlledShadowForwardObservationImplementationReviewPanel() {
  const [registry, setRegistry] = createSignal<ControlledShadowForwardObservationImplementationReviewRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [verdict, setVerdict] = createSignal<ControlledShadowForwardObservationImplementationReviewVerdict>(
    "approved_for_future_isolated_forward_observation_runner_specification_registration",
  );
  const [fields, setFields] = createSignal(emptyFields());
  const [checks, setChecks] = createSignal(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowForwardObservationImplementationReviews();
      setRegistry(next);
      const candidates = next.items.filter((item) => item.review_eligible);
      if (!candidates.some((item) => item.implementation.implementation_id === selectedId())) {
        setSelectedId(candidates[0]?.implementation.implementation_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 85 前向观察实现独立复核表读取失败");
    }
  };
  onMount(() => void load());

  const eligible = createMemo(() => registry()?.items.filter((item) => item.review_eligible) ?? []);
  const selected = createMemo(() => eligible().find((item) => item.implementation.implementation_id === selectedId()));
  const approving = createMemo(() => verdict() === "approved_for_future_isolated_forward_observation_runner_specification_registration");
  const disabled = createMemo(() => busy() || !selected()
    || Object.values(fields()).some((value) => !value.trim())
    || (approving() && checks().some((value) => !value)));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const implementation = item.implementation;
    const previous = item.latest_review;
    const registration = implementation.upstream_protocol_registration;
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await reviewControlledShadowForwardObservationImplementation(
        implementation.implementation_id,
        {
          expected_previous_review_id: previous?.review_id,
          expected_previous_review_sha256: previous?.review_sha256,
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_contract_sha256: implementation.implementation_contract.contract_sha256,
          expected_protocol_review_sha256: implementation.upstream_protocol_review.review_sha256,
          expected_protocol_registration_sha256: registration.protocol_registration_sha256,
          expected_protocol_specification_sha256: registration.protocol_specification.specification_sha256,
          expected_design_specification_sha256: registration.protocol_specification.exact_design_specification.specification_sha256,
          expected_independent_audit_sha256: item.current_independent_audit.audit_sha256,
          verdict: verdict(),
          ...fields(),
          exact_current_stage_51_through_stage_84_binding_confirmed: checks()[0],
          reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: checks()[1],
          implementation_contract_review_registration_protocol_and_design_hashes_independently_reproduced_confirmed: checks()[2],
          natural_forward_no_backfill_and_observation_not_before_confirmed: checks()[3],
          weekly_claim_calendar_point_in_time_custody_and_corrections_confirmed: checks()[4],
          signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_functions_confirmed: checks()[5],
          future_schema_names_uninstantiated_confirmed: checks()[6],
          no_artifact_entrypoint_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed: checks()[7],
          no_production_read_write_observation_ledger_position_or_performance_write_confirmed: checks()[8],
          no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: checks()[9],
          approval_only_opens_future_isolated_runner_specification_registration_confirmed: checks()[10],
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[11],
        },
      );
      setRegistry(next); setFields(emptyFields()); setChecks(CHECKS.map(() => false));
      setNotice("Stage 85 独立复核已追加保存；即使批准，也只开放未来隔离 runner 规格登记。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 85 前向观察实现独立复核失败");
      await load();
    } finally { setBusy(false); }
  };

  const textFields = [
    ["rationale", "复核结论与理由"],
    ["binding_and_recomputation_assessment", "全链绑定与独立重算评估"],
    ["deterministic_semantics_assessment", "确定性函数与未来 schema 评估"],
    ["zero_capability_assessment", "零能力与权限边界评估"],
    ["known_limitations", "已知局限"],
    ["future_runner_constraints", "未来隔离 runner 约束"],
  ] as const;

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="Stage 85 前向观察实现责任链外独立复核">
      <header><strong>第 85 阶段 · 前向观察实现责任链外独立复核</strong><span>{current().review_status}</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>实现</span><strong>{current().implementation_count}</strong></div>
        <div><span>待复核</span><strong>{current().review_eligible_count}</strong></div>
        <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
        <div><span>独立批准</span><strong>{current().independently_approved_count}</strong></div>
      </div>
      <p class="public-admin-anchor-boundary">批准也不是运行授权：没有 runner、观察、账本、持仓、绩效、订单、券商或交易能力。</p>
      <Show when={eligible().length > 0} fallback={<p>当前没有可由责任链外角色复核的 Stage 84 实现。</p>}>
        <label><span>Stage 84 实现</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}><For each={eligible()}>{(item) => <option value={item.implementation.implementation_id}>{item.implementation.implementation_id.slice(0, 12)}… · {item.implementation.implementation_name}</option>}</For></select></label>
        <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ControlledShadowForwardObservationImplementationReviewVerdict)}>
          <option value="approved_for_future_isolated_forward_observation_runner_specification_registration">批准进入未来隔离 runner 规格登记</option>
          <option value="changes_required_rebuild_forward_observation_implementation">要求重建 Stage 84 实现</option>
          <option value="rejected_forward_observation_implementation">拒绝当前实现</option>
        </select></label>
        <For each={textFields}>{([key, label]) => <label><span>{label}</span><textarea value={fields()[key]} onInput={(event) => setFields((value) => ({ ...value, [key]: event.currentTarget.value }))} /></label>}</For>
        <div class="public-admin-decision-checks"><For each={CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, i) => i === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在追加复核…" : "提交 Stage 85 独立复核"}</button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().items}>{(item) => <Show when={item.latest_review}>{(review) => <article><header><strong>{item.implementation.implementation_name}</strong><span>{review().verdict}</span></header><p>{review().submitted_at} · {review().reviewer_id}</p><p>{review().rationale}</p></article>}</Show>}</For>
    </section>
  )}</Show>;
}
