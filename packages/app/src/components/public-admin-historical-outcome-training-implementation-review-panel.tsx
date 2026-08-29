import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeTrainingImplementationReviews,
  reviewHistoricalOutcomeTrainingImplementation,
} from "@/lib/api";
import type {
  HistoricalOutcomeTrainingImplementationReviewRegistry,
  HistoricalOutcomeTrainingImplementationReviewVerdict,
  ReviewHistoricalOutcomeTrainingImplementationRequest,
} from "@/lib/types";

const CHECKS = [
  ["exact_current_implementation_and_complete_upstream_binding_confirmed", "精确绑定当前 Stage 53 实现、Stage 52 复核与 Stage 51 claim/registration/result"],
  ["reviewer_independence_from_registrar_and_complete_prior_chain_confirmed", "复核人独立于实现登记人、完整上游和此前复核人"],
  ["implementation_record_and_contract_hashes_independently_reproduced_confirmed", "已独立重算实现记录与实现合同 SHA-256"],
  ["immutable_artifact_digest_and_code_revision_reproducible_confirmed", "实现工件摘要和不可变代码版本可复现"],
  ["fixed_three_arm_three_seed_implementation_confirmed", "三模型臂与 17/29/43 三种子逐项一致"],
  ["exact_65_feature_nine_raw_continuous_target_contract_confirmed", "65 项特征与九项原始连续目标逐项一致"],
  ["train_only_preprocessing_and_fit_confirmed", "预处理和拟合只使用 train"],
  ["validation_only_selection_and_sealed_holdout_isolation_confirmed", "validation 只用于选择，sealed holdout 对拟合与选择不可见"],
  ["per_target_per_seed_metrics_without_composite_masking_confirmed", "逐目标逐种子指标齐全，综合结果不能掩盖单目标失败"],
  ["deterministic_replay_and_fixed_resource_ceilings_confirmed", "确定性重放和全部资源上限固定"],
  ["no_scalar_reward_action_position_or_ranking_semantics_confirmed", "没有标量奖励、动作、仓位或排名语义"],
  ["no_entrypoint_environment_secrets_network_tools_child_process_or_data_access_confirmed", "没有入口、环境、密钥、网络、工具、子进程或数据访问"],
  ["review_runner_data_access_training_output_validation_and_reward_separation_confirmed", "复核、runner、数据授权、训练、输出校验与奖励治理相互独立"],
  ["no_runner_data_access_training_artifact_metrics_reward_shadow_order_broker_or_trading_confirmed", "本阶段不创建 runner、不读数据、不训练、不产出模型/指标、不奖励、不影子、不下单、不接券商、不交易"],
] as const;

type CheckName = (typeof CHECKS)[number][0];

export function PublicAdminHistoricalOutcomeTrainingImplementationReviewPanel() {
  const [registry, setRegistry] = createSignal<HistoricalOutcomeTrainingImplementationReviewRegistry>();
  const [selectedImplementationId, setSelectedImplementationId] = createSignal("");
  const [verdict, setVerdict] = createSignal<HistoricalOutcomeTrainingImplementationReviewVerdict>(
    "approved_for_future_isolated_training_runner_registration",
  );
  const [rationale, setRationale] = createSignal("");
  const [knownLimitations, setKnownLimitations] = createSignal("");
  const [checked, setChecked] = createSignal<Record<CheckName, boolean>>(
    Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
  );
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeTrainingImplementationReviews();
      setRegistry(next);
      if (!next.items.some((item) => item.implementation.implementation_id === selectedImplementationId() && item.review_eligible)) {
        setSelectedImplementationId(next.items.find((item) => item.review_eligible)?.implementation.implementation_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练实现独立复核读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() => registry()?.items.find(
    (item) => item.implementation.implementation_id === selectedImplementationId(),
  ));
  const allConfirmed = createMemo(() => CHECKS.every(([name]) => checked()[name]));

  const submit = async () => {
    const item = selected();
    const current = registry();
    if (!item || !current || busy()) return;
    if (!rationale().trim() || !knownLimitations().trim()) {
      setError("请填写独立复核依据和已知局限。");
      return;
    }
    if (verdict() === "approved_for_future_isolated_training_runner_registration" && !allConfirmed()) {
      setError("批准前必须逐项确认全部十四项边界。");
      return;
    }
    const implementation = item.implementation;
    const stage52 = implementation.approved_registration_review;
    const contract = implementation.implementation_contract;
    const prior = item.latest_review;
    const request: ReviewHistoricalOutcomeTrainingImplementationRequest = {
      expected_previous_review_id: prior?.review_id,
      expected_previous_review_sha256: prior?.review_sha256,
      expected_implementation_sha256: implementation.implementation_sha256,
      expected_implementation_contract_sha256: contract.contract_sha256,
      expected_implementation_artifact_sha256: contract.implementation_artifact_sha256,
      expected_immutable_code_revision: contract.immutable_code_revision,
      expected_stage_52_review_sha256: stage52.review_sha256,
      expected_stage_51_registration_sha256: stage52.registration_sha256,
      expected_stage_51_claim_sha256: stage52.claim_sha256,
      expected_stage_51_result_sha256: stage52.result_sha256,
      expected_suite_specification_sha256: stage52.suite_specification_sha256,
      expected_review_contract_sha256: current.review_contract.contract_sha256,
      expected_independent_audit_sha256: item.current_independent_audit.audit_sha256,
      verdict: verdict(),
      rationale: rationale().trim(),
      known_limitations: knownLimitations().trim(),
      ...checked(),
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setRegistry(await reviewHistoricalOutcomeTrainingImplementation(
        implementation.implementation_id,
        request,
      ));
      setChecked(Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>);
      setNotice(verdict() === "approved_for_future_isolated_training_runner_registration"
        ? "训练实现已独立批准；只开放未来隔离 runner 规格登记，没有数据访问或训练授权。"
        : "复核结论已追加保存；旧记录未覆盖。",
      );
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练实现独立复核提交失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="训练实现独立复核">
          <header><strong>第 54 阶段 · 训练实现独立复核</strong><span>{currentRegistry().review_status}</span></header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可复核</span><strong>{currentRegistry().review_eligible_count}</strong></div>
            <div><span>已复核</span><strong>{currentRegistry().reviewed_count}</strong></div>
            <div><span>独立批准</span><strong>{currentRegistry().current_binding_approved_count}</strong></div>
            <div><span>退回或拒绝</span><strong>{currentRegistry().changes_requested_or_rejected_count}</strong></div>
          </div>
          <article class="public-admin-reward-governance">
            <header><strong>实现复核 ≠ runner 或训练授权</strong><span>independent · fail closed</span></header>
            <p>复核会独立重算实现和合同摘要，并逐项审计三臂三种子、65/9、数据切分、逐目标指标和资源边界。</p>
            <p class="public-admin-anchor-boundary">即使批准，也只有未来 runner 规格登记资格；数据访问、训练、模型工件、指标、奖励、影子、订单、券商和交易仍全部关闭。</p>
          </article>
          <Show when={currentRegistry().items.some((item) => item.review_eligible)} fallback={<p>当前没有可提交独立复核的 Stage 53 实现。</p>}>
            <label><span>待复核训练实现</span><select value={selectedImplementationId()} onChange={(event) => setSelectedImplementationId(event.currentTarget.value)}>
              <For each={currentRegistry().items.filter((item) => item.review_eligible)}>{(item) => <option value={item.implementation.implementation_id}>{item.implementation.implementation_name} · {item.implementation.implementation_id.slice(0, 12)}</option>}</For>
            </select></label>
            <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeTrainingImplementationReviewVerdict)}>
              <option value="approved_for_future_isolated_training_runner_registration">批准未来隔离 runner 规格登记</option>
              <option value="changes_requested">退回修改</option>
              <option value="rejected">拒绝</option>
            </select></label>
            <label><span>独立复核依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限</span><textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} /></label>
            <For each={CHECKS}>{([name, label]) => <label class="public-admin-anchor-check"><input type="checkbox" checked={checked()[name]} onChange={(event) => setChecked({ ...checked(), [name]: event.currentTarget.checked })} /><span>{label}</span></label>}</For>
            <button type="button" disabled={busy() || !selected() || (verdict() === "approved_for_future_isolated_training_runner_registration" && !allConfirmed())} onClick={() => void submit()}>{busy() ? "正在保存不可变复核…" : "追加独立复核（不运行）"}</button>
          </Show>
          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={currentRegistry().items}>{(item) => <article class="public-admin-reward-governance">
            <header><strong>{item.implementation.implementation_name}</strong><span>{item.latest_review?.verdict ?? "待复核"}</span></header>
            <p>实现 {item.implementation.implementation_id.slice(0, 16)} · 审计 {item.current_independent_audit.audit_sha256.slice(0, 16)} · 差异 {item.current_independent_audit.mismatch_reasons.length}</p>
            <p class="public-admin-anchor-boundary">{item.future_isolated_training_runner_registration_eligible ? "只可进入未来隔离 runner 规格登记" : "尚未独立批准"}；本阶段没有任何运行、数据、训练、奖励或交易能力。</p>
          </article>}</For>
        </section>
      )}
    </Show>
  );
}
