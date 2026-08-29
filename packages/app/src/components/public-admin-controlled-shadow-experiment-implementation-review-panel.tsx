import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowExperimentImplementationReviews,
  reviewControlledShadowExperimentImplementation,
} from "@/lib/api";
import type {
  ControlledShadowExperimentImplementationReviewRegistry,
  ControlledShadowExperimentImplementationReviewVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–76 完整责任链",
  "复核人独立于 Stage 76 登记人和全部上游角色",
  "已独立复算实现、合同、设计复核、设计登记和设计规格五层指纹",
  "只是纯规格，没有可执行工件、入口或 runtime",
  "点时成分股、退市处理和禁止前视语义保持不变",
  "信号、成交、成本、分红、调仓和反事实语义保持不变",
  "仅多头普通股；仓位上限、现金底线且无期权、杠杆或做空",
  "观察期、样本门槛、检查点、六项分开指标和多重检验保持不变",
  "停止、证伪和禁止原位重启规则是确定性的",
  "未来输入只读，输出创建一次、不可信、需独立验证且无订单载荷",
  "无环境、密钥、网络、工具、子进程或生产读写能力",
  "不写模型/指标库，不训练、不反馈、不合成综合分或奖励",
  "不运行影子盘，不写账本/持仓，不生成订单，不接券商或交易",
  "批准只开放未来隔离影子 runner 规格登记",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

const emptyTexts = () => ({
  rationale: "",
  implementation_verification_notes: "",
  risk_assessment: "",
  known_limitations: "",
  future_runner_constraints: "",
});

export function PublicAdminControlledShadowExperimentImplementationReviewPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowExperimentImplementationReviewRegistry>();
  const [selectedImplementationId, setSelectedImplementationId] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<ControlledShadowExperimentImplementationReviewVerdict>("changes_requested");
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [texts, setTexts] = createSignal(emptyTexts());
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowExperimentImplementationReviews();
      setRegistry(next);
      const eligible = next.items.filter((item) => item.review_eligible);
      if (!eligible.some((item) => item.implementation.implementation_id === selectedImplementationId())) {
        setSelectedImplementationId(eligible[0]?.implementation.implementation_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "受控影子实验零能力实现独立复核表读取失败");
    }
  };

  onMount(() => void load());

  const eligibleItems = createMemo(
    () => registry()?.items.filter((item) => item.review_eligible) ?? [],
  );
  const selected = createMemo(() =>
    eligibleItems().find(
      (item) => item.implementation.implementation_id === selectedImplementationId(),
    ),
  );
  const allTextsPresent = createMemo(() =>
    Object.values(texts()).every((value) => value.trim().length > 0),
  );
  const approvalChecksComplete = createMemo(
    () => verdict() !== "approved_for_future_isolated_shadow_runner_specification_registration"
      || checks().every(Boolean),
  );
  const disabled = createMemo(
    () => busy() || !selected() || !allTextsPresent() || !approvalChecksComplete(),
  );

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    const implementation = item.implementation;
    const audit = item.current_independent_audit;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewControlledShadowExperimentImplementation(
        implementation.implementation_id,
        {
          expected_previous_review_id: item.latest_review?.review_id,
          expected_previous_review_sha256: item.latest_review?.review_sha256,
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_contract_sha256:
            implementation.implementation_contract.contract_sha256,
          expected_design_review_sha256:
            implementation.upstream_design_review.review_sha256,
          expected_design_registration_sha256:
            implementation.upstream_design_registration.registration_sha256,
          expected_design_specification_sha256:
            implementation.upstream_design_registration.design_specification.specification_sha256,
          expected_independent_audit_sha256: audit.audit_sha256,
          verdict: verdict(),
          ...texts(),
          exact_current_stage_51_through_stage_76_binding_confirmed: checks()[0] as boolean,
          reviewer_independent_from_stage_76_and_complete_prior_chain_confirmed:
            checks()[1] as boolean,
          implementation_contract_design_review_registration_and_spec_hashes_independently_reproduced_confirmed:
            checks()[2] as boolean,
          pure_specification_no_executable_artifact_entrypoint_or_runtime_confirmed:
            checks()[3] as boolean,
          point_in_time_universe_delisting_and_no_lookahead_semantics_confirmed:
            checks()[4] as boolean,
          signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_confirmed:
            checks()[5] as boolean,
          long_only_caps_cash_floor_no_options_leverage_or_shorting_confirmed:
            checks()[6] as boolean,
          observation_sample_checkpoint_separate_metrics_and_multiple_testing_confirmed:
            checks()[7] as boolean,
          deterministic_stop_falsification_and_no_in_place_restart_confirmed:
            checks()[8] as boolean,
          future_input_read_only_output_create_once_untrusted_validated_and_no_order_payload_confirmed:
            checks()[9] as boolean,
          no_environment_secret_network_tool_subprocess_or_production_io_confirmed:
            checks()[10] as boolean,
          no_model_metric_store_training_feedback_composite_or_reward_confirmed:
            checks()[11] as boolean,
          no_shadow_run_ledger_position_order_broker_or_trading_confirmed:
            checks()[12] as boolean,
          approval_only_opens_future_isolated_runner_specification_registration_confirmed:
            checks()[13] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[14] as boolean,
        },
      );
      setRegistry(next);
      setChecks(REVIEW_CHECKS.map(() => false));
      setTexts(emptyTexts());
      setVerdict("changes_requested");
      setNotice("独立复核已追加写入。即使批准，也只开放未来隔离影子 runner 规格登记。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "受控影子实验零能力实现独立复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(current) => (
        <section class="public-admin-reward-governance" aria-label="受控影子实验零能力实现独立复核">
          <header>
            <strong>第 77 阶段 · 零能力影子实现独立复核</strong>
            <span>{current().review_status}</span>
          </header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>已登记实现</span><strong>{current().implementation_count}</strong></div>
            <div><span>待独立复核</span><strong>{current().review_eligible_count}</strong></div>
            <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
            <div><span>独立通过</span><strong>{current().independently_approved_count}</strong></div>
            <div><span>待改/拒绝</span><strong>{current().changes_requested_or_rejected_count}</strong></div>
            <div><span>可登记隔离 runner 规格</span><strong>{current().future_isolated_shadow_runner_specification_registration_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>独立复核，不是运行授权</strong><span>五层指纹重算</span></header>
            <p>复核者必须与 Stage 76 登记人及完整上游责任链隔离，并独立重算实现、实现合同、设计复核、设计登记和设计规格五层哈希。</p>
            <p class="public-admin-anchor-boundary">通过后仍无可执行 runner、输入挂载、生产读写、影子账本、持仓、订单、券商或真实交易；下一阶段只能登记隔离 runner 的规格。</p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p>当前没有待独立复核的 Stage 76 零能力实现。</p>}>
            <label>
              <span>待复核实现</span>
              <select value={selectedImplementationId()} onChange={(event) => setSelectedImplementationId(event.currentTarget.value)}>
                <For each={eligibleItems()}>{(item) => {
                  const implementation = item.implementation;
                  return <option value={implementation.implementation_id}>{implementation.implementation_id.slice(0, 12)}… · {implementation.implementation_contract.target_id} · {implementation.implementation_name}</option>;
                }}</For>
              </select>
            </label>
            <Show when={selected()}>{(item) => (
              <article class="public-admin-reward-governance">
                <header><strong>独立审计摘要</strong><span>{item().current_independent_audit.audit_sha256.slice(0, 12)}…</span></header>
                <p>实现 {item().current_independent_audit.implementation_sha256.slice(0, 12)}… · 合同 {item().current_independent_audit.implementation_contract_sha256.slice(0, 12)}… · 设计 {item().current_independent_audit.design_specification_sha256.slice(0, 12)}…</p>
                <p class="public-admin-anchor-boundary">{item().current_independent_audit.mismatch_reasons.length === 0 ? "五层指纹和零权限合同复算一致" : item().current_independent_audit.mismatch_reasons.join("；")}</p>
              </article>
            )}</Show>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ControlledShadowExperimentImplementationReviewVerdict)}>
                <option value="changes_requested">要求修改（不可覆盖原记录，须新建上游责任链）</option>
                <option value="rejected">拒绝</option>
                <option value="approved_for_future_isolated_shadow_runner_specification_registration">批准进入未来隔离 runner 规格登记</option>
              </select>
            </label>
            <label><span>复核理由</span><textarea value={texts().rationale} onInput={(event) => setTexts((value) => ({ ...value, rationale: event.currentTarget.value }))} /></label>
            <label><span>实现核验记录</span><textarea value={texts().implementation_verification_notes} onInput={(event) => setTexts((value) => ({ ...value, implementation_verification_notes: event.currentTarget.value }))} /></label>
            <label><span>风险评估</span><textarea value={texts().risk_assessment} onInput={(event) => setTexts((value) => ({ ...value, risk_assessment: event.currentTarget.value }))} /></label>
            <label><span>已知局限</span><textarea value={texts().known_limitations} onInput={(event) => setTexts((value) => ({ ...value, known_limitations: event.currentTarget.value }))} /></label>
            <label><span>未来 runner 约束</span><textarea value={texts().future_runner_constraints} onInput={(event) => setTexts((value) => ({ ...value, future_runner_constraints: event.currentTarget.value }))} /></label>
            <div class="public-admin-decision-checks">
              <For each={REVIEW_CHECKS}>{(label, index) => (
                <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
              )}</For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在追加复核…" : "提交独立复核"}</button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={current().items}>{(item) => (
            <Show when={item.latest_review}>{(review) => (
              <article class="public-admin-reward-governance">
                <header><strong>{item.implementation.implementation_name}</strong><span>{review().verdict}</span></header>
                <p>{review().submitted_at} · {review().reviewer_id}</p>
                <p><strong>理由：</strong>{review().rationale}</p>
                <p><strong>风险：</strong>{review().risk_assessment}</p>
                <p><strong>未来 runner 约束：</strong>{review().future_runner_constraints}</p>
                <p class="public-admin-anchor-boundary">复核记录只能开放未来规格登记；运行、账本、持仓、订单、券商和交易权限仍全部关闭。</p>
              </article>
            )}</Show>
          )}</For>
        </section>
      )}
    </Show>
  );
}
