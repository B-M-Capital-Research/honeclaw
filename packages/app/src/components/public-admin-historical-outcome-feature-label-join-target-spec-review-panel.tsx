import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeFeatureLabelJoinTargetSpecReviews,
  reviewHistoricalOutcomeFeatureLabelJoinTargetSpec,
} from "@/lib/api";
import type {
  HistoricalOutcomeFeatureLabelJoinTargetSpecReviewRegistry,
  HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict,
} from "@/lib/types";

const CHECKS = [
  "当前规范、独立校验正式工件、原始数据集与全部 SHA-256 绑定仍精确一致",
  "复核人未参与规范登记、工件生产/校验、完整上游或此前复核",
  "使用独立实现重算 record、body、join 与 target 指纹，而非复用登记校验器",
  "每个 entry 只允许一个 split、一个 raw outcome 和每个 allowlist feature 一条记录；重复或缺失失败关闭",
  "official split 是唯一分区权威，purge/embargo 条目完全排除且不得重新分配",
  "特征严格点时可用，65 项显式缺失完整保留，不插值、不回填、不删列",
  "未来/结果/holdout/当前组合/模型回填均禁止作为连接输入",
  "训练、验证、sealed holdout 的目标可见性分离，sealed holdout 不用于训练或调参",
  "目标精确覆盖 20/60/250 日资产收益、相对 SPY 超额收益和最大回撤，共九项连续值",
  "250 日超额收益主目标与 250 日最大回撤风险目标只是工程候选，不是老王确认逻辑或策略真理",
  "保留精确 f64 位，不标准化、不 winsorize、不排名，也不定义买卖阈值",
  "复核、实现登记、join 执行与输出校验继续分门，批准不等于执行",
  "本阶段不 join、不分配标签、不创建训练行，也不训练、奖励、影子、订单、券商或交易",
] as const;

export function PublicAdminHistoricalOutcomeFeatureLabelJoinTargetSpecReviewPanel() {
  const [registry, setRegistry] = createSignal<HistoricalOutcomeFeatureLabelJoinTargetSpecReviewRegistry>();
  const [selectedSpecificationId, setSelectedSpecificationId] = createSignal("");
  const [verdict, setVerdict] = createSignal<HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict>("changes_requested");
  const [rationale, setRationale] = createSignal("独立核对规范与正式工件绑定、连接防泄漏合同和九维连续目标语义，仅在全部检查通过时批准未来隔离实现登记。");
  const [limitations, setLimitations] = createSignal("主目标和风险目标仍是工程候选；样本量、行业覆盖、缺失率、目标可预测性及经济价值尚未通过训练外验证。");
  const [checks, setChecks] = createSignal<boolean[]>(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeFeatureLabelJoinTargetSpecReviews();
      setRegistry(next);
      if (!next.items.some((item) => item.specification.specification_id === selectedSpecificationId())) {
        setSelectedSpecificationId(
          next.items.find((item) => item.review_eligible)?.specification.specification_id
            ?? next.items[0]?.specification.specification_id
            ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 规范独立复核注册表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find((item) => item.specification.specification_id === selectedSpecificationId()),
  );
  const allChecked = createMemo(() => checks().every(Boolean));
  const canSubmit = createMemo(() => Boolean(
    selected()?.review_eligible
      && rationale().trim()
      && limitations().trim()
      && (verdict() !== "approved_for_future_isolated_join_target_implementation_registration" || allChecked()),
  ));

  const submit = async () => {
    const currentRegistry = registry();
    const item = selected();
    if (!currentRegistry || !item || !canSubmit() || busy()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const prior = item.latest_review;
      const next = await reviewHistoricalOutcomeFeatureLabelJoinTargetSpec(
        item.specification.specification_id,
        {
          expected_review_id: prior?.review_id,
          expected_review_sha256: prior?.review_sha256,
          expected_specification_sha256: item.specification.specification_sha256,
          expected_specification_body_sha256: item.specification.specification_body_sha256,
          expected_join_specification_sha256: item.specification.join_specification.specification_sha256,
          expected_target_specification_sha256: item.specification.target_specification.specification_sha256,
          expected_validation_sha256: item.specification.validation_sha256,
          expected_combined_artifact_sha256: item.specification.combined_artifact_sha256,
          expected_review_contract_sha256: currentRegistry.review_contract.contract_sha256,
          verdict: verdict(),
          rationale: rationale().trim(),
          known_limitations: limitations().trim(),
          exact_current_specification_and_artifact_binding_confirmed: checks()[0],
          reviewer_independence_confirmed: checks()[1],
          independent_record_join_target_hash_reproduction_confirmed: checks()[2],
          one_to_one_entry_join_and_duplicate_missing_failure_confirmed: checks()[3],
          purge_embargo_exclusion_and_official_split_authority_confirmed: checks()[4],
          point_in_time_feature_and_explicit_missingness_confirmed: checks()[5],
          forbidden_future_outcome_holdout_portfolio_and_model_inputs_confirmed: checks()[6],
          split_specific_target_visibility_and_sealed_holdout_confirmed: checks()[7],
          exact_nine_continuous_target_semantics_confirmed: checks()[8],
          primary_and_risk_targets_are_engineering_candidates_not_strategy_truth_confirmed: checks()[9],
          exact_f64_identity_without_normalization_ranking_or_thresholds_confirmed: checks()[10],
          review_implementation_execution_and_output_validation_separation_confirmed: checks()[11],
          no_join_assignment_training_reward_shadow_order_broker_or_trading_confirmed: checks()[12],
        },
      );
      setRegistry(next);
      setChecks(CHECKS.map(() => false));
      setNotice(
        verdict() === "approved_for_future_isolated_join_target_implementation_registration"
          ? "独立复核已不可变记录；批准范围仅为未来隔离实现登记，当前没有执行 join 或训练。"
          : "复核意见已不可变记录；规范未获执行或训练权限。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 规范独立复核提交失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="特征标签连接与目标规范独立复核">
          <header>
            <strong>第 37 阶段 · join/target 规范独立复核</strong>
            <span>{currentRegistry().review_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待复核</span><strong>{currentRegistry().review_eligible_count}</strong></div>
            <div><span>已复核</span><strong>{currentRegistry().reviewed_count}</strong></div>
            <div><span>已批准</span><strong>{currentRegistry().current_binding_approved_count}</strong></div>
            <div><span>可登记实现</span><strong>{currentRegistry().implementation_registration_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>独立审计合同</strong><span>不复用登记校验器</span></header>
            <p>审计器：{currentRegistry().review_contract.semantic_audit_implementation}</p>
            <p>批准范围：{currentRegistry().review_contract.approval_scope}</p>
            <p class="public-admin-anchor-boundary">250 日超额收益只是待验证的工程主目标候选，不是已确认投资规则；批准不证明可预测、可赚钱或可交易。</p>
          </article>

          <Show when={currentRegistry().items.length > 0} fallback={<p>当前没有可独立复核的 join/target 规范。</p>}>
            <label>
              <span>待复核规范</span>
              <select value={selectedSpecificationId()} onChange={(event) => setSelectedSpecificationId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>{(item) => (
                  <option value={item.specification.specification_id}>
                    {item.specification.specification_id.slice(0, 12)}… · {item.review_eligible ? "待复核" : "已批准"}
                  </option>
                )}</For>
              </select>
            </label>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict)}>
                <option value="changes_requested">要求修订</option>
                <option value="rejected">拒绝</option>
                <option value="approved_for_future_isolated_join_target_implementation_registration">批准未来隔离实现登记</option>
              </select>
            </label>
            <label><span>复核理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
            <div class="public-admin-decision-checks">
              <For each={CHECKS}>{(label, index) => (
                <label>
                  <input type="checkbox" checked={checks()[index()]} onChange={(event) => {
                    const next = [...checks()];
                    next[index()] = event.currentTarget.checked;
                    setChecks(next);
                  }} />
                  <span>{label}</span>
                </label>
              )}</For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={busy() || !canSubmit()} onClick={() => void submit()}>
              {busy() ? "正在写入不可变复核…" : "提交独立复核"}
            </button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>

          <For each={currentRegistry().items}>{(item) => (
            <article class="public-admin-reward-governance">
              <header>
                <strong>{item.specification.specification_name}</strong>
                <span>{item.latest_review?.verdict ?? "待独立复核"}</span>
              </header>
              <p>独立审计 {item.current_independent_audit.audit_sha256} · 目标 {item.current_independent_audit.target_ids.length} 项</p>
              <p>record/body/join/target 指纹：{item.current_independent_audit.record_hash_independently_reproduced && item.current_independent_audit.specification_body_hash_independently_reproduced && item.current_independent_audit.join_hash_independently_reproduced && item.current_independent_audit.target_hash_independently_reproduced ? "全部独立复现" : "失败关闭"}</p>
              <p>正式工件与特征目录：{item.current_independent_audit.exact_current_artifact_binding_reproduced && item.current_independent_audit.exact_feature_catalog_binding_reproduced ? "当前精确绑定" : "绑定漂移"}</p>
              <Show when={item.latest_review}>{(review) => (
                <>
                  <p>复核人 {review().reviewer_id} · {review().submitted_at}</p>
                  <p>{review().rationale}</p>
                  <p>局限：{review().known_limitations}</p>
                </>
              )}</Show>
              <p class="public-admin-anchor-boundary">即使批准，也只允许未来登记隔离实现；当前 join、标签、训练、奖励、影子、订单、券商与交易全部关闭。</p>
            </article>
          )}</For>
        </section>
      )}
    </Show>
  );
}
