import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowExperimentImplementations,
  registerControlledShadowExperimentImplementation,
} from "@/lib/api";
import type { ControlledShadowExperimentImplementationRegistry } from "@/lib/types";

const REGISTRATION_CHECKS = [
  "精确绑定当前 Stage 51–75 完整责任链",
  "登记人独立于 Stage 75 复核者及全部上游角色",
  "已独立复算设计复核、设计登记和设计规格指纹",
  "本次只登记零能力规格，不声称存在可执行工件",
  "保留点时成分股、退市与无前视语义",
  "保留信号、成交、成本、分红、调仓和全部反事实语义",
  "保留多头普通股、集中度、现金底线及无期权/杠杆/做空边界",
  "保留样本、检查点、分项指标、多重检验和停止规则",
  "确定性重放合同内容寻址且只能登记一次",
  "无入口、runtime、环境继承、密钥、网络、工具、子进程或生产读写",
  "不写模型/指标库，不反馈训练，不定义综合分或标量奖励",
  "不运行影子盘，不建账本/持仓/订单，不接券商或交易",
  "runner 登记前仍必须完成未来独立实现复核",
  "未把未确认 Hari/老王观点写成工程规则",
] as const;

const emptyForm = () => ({
  implementation_name: "受控影子前向重放零能力规格",
  immutable_code_revision: "",
  implementation_description: "",
  deterministic_replay_notes: "",
  known_limitations: "",
  future_review_constraints: "",
});

export function PublicAdminControlledShadowExperimentImplementationPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowExperimentImplementationRegistry>();
  const [selectedReviewId, setSelectedReviewId] = createSignal("");
  const [form, setForm] = createSignal(emptyForm());
  const [checks, setChecks] = createSignal(REGISTRATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const eligibleDesigns = createMemo(() => {
    const current = registry();
    if (!current) return [];
    const registeredReviewIds = new Set(
      current.items
        .filter((item) => item.upstream_binding_current)
        .map((item) => item.implementation.upstream_design_review.review_id),
    );
    return current.eligible_designs.filter(
      (source) => !registeredReviewIds.has(source.design_review.review_id),
    );
  });

  const load = async () => {
    try {
      const next = await getControlledShadowExperimentImplementations();
      setRegistry(next);
      const registeredReviewIds = new Set(
        next.items
          .filter((item) => item.upstream_binding_current)
          .map((item) => item.implementation.upstream_design_review.review_id),
      );
      const eligible = next.eligible_designs.filter(
        (source) => !registeredReviewIds.has(source.design_review.review_id),
      );
      if (!eligible.some((source) => source.design_review.review_id === selectedReviewId())) {
        setSelectedReviewId(eligible[0]?.design_review.review_id ?? "");
      }
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "零能力影子实现登记表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    eligibleDesigns().find((source) => source.design_review.review_id === selectedReviewId()),
  );
  const formComplete = createMemo(() =>
    Object.values(form()).every((value) => value.trim().length > 0),
  );
  const disabled = createMemo(
    () => busy() || !selected() || !formComplete() || !checks().every(Boolean),
  );

  const submit = async () => {
    const source = selected();
    if (!source || disabled()) return;
    const registration = source.design_registration;
    const review = source.design_review;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await registerControlledShadowExperimentImplementation(
        registration.attempt_id,
        {
          expected_design_review_id: review.review_id,
          expected_design_review_sha256: review.review_sha256,
          expected_design_registration_id: registration.registration_id,
          expected_design_registration_sha256: registration.registration_sha256,
          expected_design_specification_sha256:
            registration.design_specification.specification_sha256,
          expected_adjudication_review_sha256: registration.adjudication_review_sha256,
          expected_output_validation_sha256: registration.output_validation_sha256,
          expected_claim_sha256: registration.claim_sha256,
          expected_result_sha256: registration.result_sha256,
          expected_output_sha256: registration.output_sha256,
          expected_envelope_sha256: registration.envelope_sha256,
          expected_candidate_set_sha256: registration.candidate_set_sha256,
          expected_training_store_dataset_sha256:
            registration.training_store_dataset_sha256,
          expected_selected_algorithm_three_seed_binding_sha256:
            registration.selected_algorithm_three_seed_binding_sha256,
          expected_sealed_holdout_split_commitment_sha256:
            registration.sealed_holdout_split_commitment_sha256,
          expected_sealed_holdout_projection_sha256:
            registration.sealed_holdout_projection_sha256,
          expected_feature_order_sha256: registration.feature_order_sha256,
          expected_preprocessing_sha256: registration.preprocessing_sha256,
          expected_target_id: registration.target_id,
          expected_frozen_candidate_algorithm_id: registration.frozen_candidate_algorithm_id,
          ...form(),
          exact_current_stage_51_through_stage_75_binding_confirmed: checks()[0] as boolean,
          registrar_independent_from_stage_75_and_complete_prior_chain_confirmed:
            checks()[1] as boolean,
          independent_recomputation_of_design_review_registration_and_specification_confirmed:
            checks()[2] as boolean,
          zero_capability_specification_only_not_executable_artifact_confirmed:
            checks()[3] as boolean,
          point_in_time_universe_delisting_and_no_lookahead_semantics_preserved_confirmed:
            checks()[4] as boolean,
          signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_preserved_confirmed:
            checks()[5] as boolean,
          long_only_caps_cash_floor_no_options_leverage_or_shorting_preserved_confirmed:
            checks()[6] as boolean,
          observation_sample_checkpoint_metric_multiple_testing_and_stop_rules_preserved_confirmed:
            checks()[7] as boolean,
          deterministic_create_once_content_addressed_replay_contract_confirmed:
            checks()[8] as boolean,
          no_entrypoint_runtime_environment_secret_network_tool_subprocess_or_production_io_confirmed:
            checks()[9] as boolean,
          no_model_store_metric_store_training_feedback_composite_or_reward_confirmed:
            checks()[10] as boolean,
          no_shadow_run_ledger_position_order_broker_or_trading_confirmed:
            checks()[11] as boolean,
          future_independent_implementation_review_required_before_runner_registration_confirmed:
            checks()[12] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[13] as boolean,
        },
      );
      setRegistry(next);
      setChecks(REGISTRATION_CHECKS.map(() => false));
      setForm(emptyForm());
      setNotice("零能力实现规格已登记；它不可执行，只开放未来独立实现复核。 ");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "零能力影子实现登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(current) => (
        <section class="public-admin-reward-governance" aria-label="受控影子实验零能力实现登记">
          <header>
            <strong>第 76 阶段 · 零能力影子实现规格登记</strong>
            <span>{current().implementation_status}</span>
          </header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可登记设计</span><strong>{current().registration_eligible_count}</strong></div>
            <div><span>已登记规格</span><strong>{current().implementation_count}</strong></div>
            <div><span>当前绑定</span><strong>{current().current_binding_implementation_count}</strong></div>
            <div><span>可独立复核</span><strong>{current().independent_implementation_review_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>硬边界</strong><span>规格，不是程序</span></header>
            <p>登记只冻结纯函数语义、不可变输入绑定和未来不可信输出信封；没有二进制、入口、runtime、数据挂载或联网能力。</p>
            <p class="public-admin-anchor-boundary">无模型库、训练、reward、影子运行、账本、持仓、订单、券商或真实交易权限。</p>
          </article>

          <Show when={eligibleDesigns().length > 0} fallback={<p>当前没有可登记的 Stage 75 独立批准设计。</p>}>
            <label>
              <span>已独立批准设计</span>
              <select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
                <For each={eligibleDesigns()}>{(source) => (
                  <option value={source.design_review.review_id}>
                    {source.design_registration.attempt_id.slice(0, 12)}… · {source.design_registration.target_id} · {source.design_registration.experiment_name}
                  </option>
                )}</For>
              </select>
            </label>
            <label><span>规格名称</span><input value={form().implementation_name} onInput={(event) => setForm((value) => ({ ...value, implementation_name: event.currentTarget.value }))} /></label>
            <label><span>不可变代码版本</span><input placeholder="例如 git:<commit>" value={form().immutable_code_revision} onInput={(event) => setForm((value) => ({ ...value, immutable_code_revision: event.currentTarget.value }))} /></label>
            <label><span>实现说明</span><textarea value={form().implementation_description} onInput={(event) => setForm((value) => ({ ...value, implementation_description: event.currentTarget.value }))} /></label>
            <label><span>确定性重放说明</span><textarea value={form().deterministic_replay_notes} onInput={(event) => setForm((value) => ({ ...value, deterministic_replay_notes: event.currentTarget.value }))} /></label>
            <label><span>已知局限</span><textarea value={form().known_limitations} onInput={(event) => setForm((value) => ({ ...value, known_limitations: event.currentTarget.value }))} /></label>
            <label><span>未来独立复核约束</span><textarea value={form().future_review_constraints} onInput={(event) => setForm((value) => ({ ...value, future_review_constraints: event.currentTarget.value }))} /></label>
            <div class="public-admin-decision-checks">
              <For each={REGISTRATION_CHECKS}>{(label, index) => (
                <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
              )}</For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在登记…" : "登记零能力实现规格"}</button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={current().items}>{(item) => {
            const value = item.implementation;
            return (
              <article class="public-admin-reward-governance">
                <header><strong>{value.implementation_name}</strong><span>{value.status}</span></header>
                <p>{value.registered_at} · {value.registered_by} · {value.implementation_contract.immutable_code_revision}</p>
                <p>{value.implementation_description}</p>
                <p><strong>重放：</strong>{value.deterministic_replay_notes}</p>
                <p><strong>局限：</strong>{value.known_limitations}</p>
                <p class="public-admin-anchor-boundary">规格 SHA {value.implementation_contract.contract_sha256.slice(0, 16)}…；入口、runtime、影子运行、账本、订单、券商与交易全部关闭。</p>
              </article>
            );
          }}</For>
        </section>
      )}
    </Show>
  );
}
