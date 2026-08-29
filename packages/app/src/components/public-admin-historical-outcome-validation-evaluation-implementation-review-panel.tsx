import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeValidationEvaluationImplementationReviews,
  reviewHistoricalOutcomeValidationEvaluationImplementation,
} from "@/lib/api";
import type {
  HistoricalOutcomeValidationEvaluationImplementationReviewRegistry,
  HistoricalOutcomeValidationEvaluationImplementationReviewVerdict,
  ReviewHistoricalOutcomeValidationEvaluationImplementationRequest,
} from "@/lib/types";

const CHECKS = [
  ["exact_stage_57_through_stage_59_chain_confirmed", "精确核对 Stage 57–59 及完整上游链"],
  ["reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed", "复核人独立于登记、验证、执行和此前完整角色链"],
  ["implementation_contract_and_candidate_set_hashes_independently_reproduced_confirmed", "独立复算实现、合同和候选集合三个指纹"],
  ["exact_nine_artifact_three_algorithm_three_seed_matrix_confirmed", "精确核对三算法 × 三种子的 9 个不可变工件"],
  ["exact_65_feature_nine_target_and_per_target_metric_contract_confirmed", "核对 65 项特征、9 项目标和逐目标逐种子指标"],
  ["component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed", "核对 10,000 次 component-block bootstrap、固定种子、54 项 Holm 与样本门槛"],
  ["minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed", "核对 5% MAE、秩相关、方向、校准和三个种子全通过门槛"],
  ["no_seed_shopping_tuning_or_composite_masking_confirmed", "确认禁止挑种子、调参和综合分遮蔽失败目标"],
  ["rules_frozen_before_validation_label_access_confirmed", "确认全部规则在读取 validation 标签前冻结"],
  ["independent_runner_authorization_and_output_validation_separation_confirmed", "确认 runner、单次授权和输出验证仍是独立后续门禁"],
  ["no_entrypoint_label_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed", "确认无入口、标签、评估、选模、存储、奖励、影子、订单、券商或交易权限"],
] as const;

type CheckName = (typeof CHECKS)[number][0];

export function PublicAdminHistoricalOutcomeValidationEvaluationImplementationReviewPanel() {
  const [registry, setRegistry] = createSignal<HistoricalOutcomeValidationEvaluationImplementationReviewRegistry>();
  const [selectedId, setSelectedId] = createSignal("");
  const [verdict, setVerdict] = createSignal<HistoricalOutcomeValidationEvaluationImplementationReviewVerdict>("approved_for_future_isolated_validation_evaluation_runner_registration");
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
      const next = await getHistoricalOutcomeValidationEvaluationImplementationReviews();
      setRegistry(next);
      if (!next.items.some((item) => item.implementation.implementation_id === selectedId() && item.review_eligible)) {
        setSelectedId(next.items.find((item) => item.review_eligible)?.implementation.implementation_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "validation 评估实现独立复核表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.items.find((item) => item.implementation.implementation_id === selectedId()));
  const allConfirmed = createMemo(() => CHECKS.every(([name]) => checked()[name]));

  const submit = async () => {
    const item = selected();
    if (!item || busy()) return;
    if (!rationale().trim() || !knownLimitations().trim()) {
      setError("请填写独立复核理由和已知局限。");
      return;
    }
    if (verdict().startsWith("approved") && !allConfirmed()) {
      setError("批准前必须逐项完成全部十一项独立确认。");
      return;
    }
    const implementation = item.implementation;
    const contract = implementation.implementation_contract;
    const request: ReviewHistoricalOutcomeValidationEvaluationImplementationRequest = {
      expected_previous_review_id: item.latest_review?.review_id,
      expected_previous_review_sha256: item.latest_review?.review_sha256,
      expected_implementation_sha256: implementation.implementation_sha256,
      expected_implementation_contract_sha256: contract.contract_sha256,
      expected_candidate_set_sha256: contract.candidate_set_sha256,
      expected_implementation_artifact_sha256: contract.implementation_artifact_sha256,
      expected_immutable_code_revision: contract.immutable_code_revision,
      expected_upstream_validation_sha256: implementation.upstream_validation.validation_sha256,
      expected_upstream_output_sha256: implementation.upstream_validation.output_sha256,
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
      setRegistry(await reviewHistoricalOutcomeValidationEvaluationImplementation(selectedId(), request));
      setChecked(Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>);
      setNotice(verdict().startsWith("approved")
        ? "独立复核已批准；只开放未来隔离 runner 规格登记，validation 标签与评估仍关闭。"
        : "复核结论已不可变写入链，未开放任何后续执行能力。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "validation 评估实现独立复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(current) => (
        <section class="public-admin-reward-governance" aria-label="validation 评估实现独立复核">
          <header><strong>第 60 阶段 · validation 评估实现独立复核</strong><span>{current().review_status}</span></header>
          <p>{current().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待复核</span><strong>{current().review_eligible_count}</strong></div>
            <div><span>已复核</span><strong>{current().reviewed_count}</strong></div>
            <div><span>独立批准</span><strong>{current().independently_approved_count}</strong></div>
            <div><span>下一门禁资格</span><strong>{current().future_isolated_runner_registration_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>独立复算，不接受勾选替代审计</strong><span>append-only · fail closed</span></header>
            <p>服务端会重新计算实现记录、合同和 9 个候选工件集合指纹，并独立校验固定统计门槛；任何漂移都会直接关闭晋级。</p>
            <p class="public-admin-anchor-boundary">批准仍不等于可运行。下一步只能登记无入口的隔离 runner 规格。</p>
          </article>

          <Show when={current().items.some((item) => item.review_eligible)} fallback={<p>当前没有待独立复核的 Stage 59 实现。</p>}>
            <label><span>待复核实现</span><select value={selectedId()} onChange={(event) => setSelectedId(event.currentTarget.value)}>
              <For each={current().items.filter((item) => item.review_eligible)}>{(item) => <option value={item.implementation.implementation_id}>{item.implementation.implementation_name} · {item.implementation.implementation_id.slice(0, 12)}…</option>}</For>
            </select></label>
            <label><span>结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeValidationEvaluationImplementationReviewVerdict)}>
              <option value="approved_for_future_isolated_validation_evaluation_runner_registration">批准进入未来隔离 runner 规格登记</option>
              <option value="changes_requested">退回补充</option>
              <option value="rejected">拒绝</option>
            </select></label>
            <label><span>独立复核理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限</span><textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} /></label>
            <For each={CHECKS}>{([name, label]) => <label class="public-admin-anchor-check"><input type="checkbox" checked={checked()[name]} onChange={(event) => setChecked({ ...checked(), [name]: event.currentTarget.checked })} /><span>{label}</span></label>}</For>
            <button type="button" disabled={busy() || !selected() || (verdict().startsWith("approved") && !allConfirmed())} onClick={() => void submit()}>{busy() ? "正在独立复算并写入…" : "提交不可变独立复核"}</button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={current().items}>{(item) => <article class="public-admin-reward-governance">
            <header><strong>{item.implementation.implementation_name}</strong><span>{item.latest_review?.verdict ?? "等待复核"}</span></header>
            <p>实现 / 合同 / 候选集合哈希：{item.current_independent_audit.mismatch_reasons.length === 0 ? "独立复算一致" : item.current_independent_audit.mismatch_reasons.join("、")}</p>
            <p class="public-admin-anchor-boundary">{item.future_isolated_runner_registration_eligible ? "只可进入未来隔离 runner 规格登记" : "未开放下一门禁"}；validation 标签、评估、选模、sealed holdout 与交易保持关闭。</p>
          </article>}</For>
        </section>
      )}
    </Show>
  );
}
