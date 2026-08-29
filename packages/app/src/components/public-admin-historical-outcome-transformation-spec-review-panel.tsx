import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeOfflineDatasetTransformationSpecReviews,
  reviewHistoricalOutcomeOfflineDatasetTransformationSpec,
} from "@/lib/api";
import type {
  HistoricalOutcomeOfflineDatasetTransformationSpecReviewRegistry,
  HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "已重新核对当前 spec、body、数据集、manifest、候选集、治理及两份子规范哈希",
  "复核人未参与数据集、标签、治理或转换规范登记链",
  "使用独立代码重放了规范 schema 与哈希，没有复用登记生成器作语义审计",
  "公司、历史事件、来源族传递连通分量身份完整且不可跨分区",
  "枚举全部时间连续边界，并按总误差、最大误差、较早边界的整数目标唯一选择",
  "组件 SHA-256 只用于最晚与最早判断时点都相同的排序并列",
  "共同资产/SPY 交易日历、250 日 purge/embargo 与处理后空分区失败规则明确",
  "sealed holdout 标签不会暴露给未来训练 worker",
  "七层中只允许固定的 65 个 feature ID，namespace 不能覆盖 feature 语义",
  "制品、来源、版本、观察与可用时间完整，后续重述版本不能回写历史",
  "定性、行情和组合特征分别满足人工证据链、冻结行情和历史持仓快照要求",
  "缺失与可用时间歧义显式记录，禁止回填或插值",
  "结果、标签、未来行情及其校验/准入字段不能改名塞进白名单层",
  "未来输出必须 create-once、内容寻址，并由另一阶段独立验证",
  "复核、实现登记、执行、输出验证、目标定义和训练是独立权限",
  "本阶段不登记实现、不生成 manifest/bundle、不 join、不写目标、不训练、奖励、影子、订单、券商或交易",
] as const;

export function PublicAdminHistoricalOutcomeTransformationSpecReviewPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeOfflineDatasetTransformationSpecReviewRegistry>();
  const [selectedSpecId, setSelectedSpecId] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict>(
      "changes_requested",
    );
  const [rationale, setRationale] = createSignal("");
  const [knownLimitations, setKnownLimitations] = createSignal("");
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeOfflineDatasetTransformationSpecReviews();
      setRegistry(next);
      const eligible = next.items.some(
        (item) => item.review_eligible && item.specification.transformation_spec_id === selectedSpecId(),
      );
      if (!eligible) {
        setSelectedSpecId(
          next.items.find((item) => item.review_eligible)?.specification
            .transformation_spec_id ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "转换规范独立复核注册表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find(
      (item) =>
        item.review_eligible &&
        item.specification.transformation_spec_id === selectedSpecId(),
    ),
  );
  const approving = createMemo(
    () =>
      verdict() ===
      "approved_for_future_isolated_transformation_implementation_registration",
  );
  const disabled = createMemo(
    () =>
      busy() ||
      !selected() ||
      !rationale().trim() ||
      !knownLimitations().trim() ||
      (approving() && checks().some((value) => !value)),
  );

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) =>
      current.map((value, currentIndex) => (currentIndex === index ? checked : value)),
    );
  };

  const submit = async () => {
    const current = selected();
    const currentRegistry = registry();
    if (!current || !currentRegistry || disabled()) return;
    const specification = current.specification;
    const prior = current.latest_review;
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewHistoricalOutcomeOfflineDatasetTransformationSpec(
        specification.transformation_spec_id,
        {
          expected_review_id: prior?.review_id,
          expected_review_sha256: prior?.review_sha256,
          expected_transformation_spec_sha256: specification.transformation_spec_sha256,
          expected_transformation_body_sha256: specification.transformation_body_sha256,
          expected_dataset_content_sha256: specification.subject.dataset_content_sha256,
          expected_manifest_sha256: specification.subject.manifest_sha256,
          expected_candidate_set_sha256: specification.subject.candidate_set_sha256,
          expected_governance_review_sha256: specification.governance_review_sha256,
          expected_split_specification_sha256:
            specification.split_manifest_specification.specification_sha256,
          expected_feature_specification_sha256:
            specification.feature_bundle_specification.specification_sha256,
          expected_review_contract_sha256: currentRegistry.review_contract.contract_sha256,
          verdict: verdict(),
          rationale: rationale().trim(),
          known_limitations: knownLimitations().trim(),
          exact_current_specification_binding_confirmed: confirmed[0],
          reviewer_independence_confirmed: confirmed[1],
          independent_hash_and_schema_reproduction_confirmed: confirmed[2],
          transitive_component_identity_and_indivisibility_confirmed: confirmed[3],
          chronological_contiguous_boundary_objective_confirmed: confirmed[4],
          equal_time_hash_tie_break_only_confirmed: confirmed[5],
          market_session_purge_embargo_and_empty_partition_failure_confirmed: confirmed[6],
          sealed_holdout_label_isolation_confirmed: confirmed[7],
          exact_seven_layer_feature_id_allowlist_confirmed: confirmed[8],
          point_in_time_artifact_and_revision_provenance_confirmed: confirmed[9],
          qualitative_market_and_portfolio_source_contracts_confirmed: confirmed[10],
          explicit_missingness_without_backfill_or_interpolation_confirmed: confirmed[11],
          outcome_label_future_and_namespace_smuggling_exclusion_confirmed: confirmed[12],
          content_addressed_create_once_outputs_and_later_validation_confirmed: confirmed[13],
          review_implementation_execution_target_training_separation_confirmed: confirmed[14],
          no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed:
            confirmed[15],
        },
      );
      setRegistry(next);
      setSelectedSpecId(
        next.items.find((item) => item.review_eligible)?.specification
          .transformation_spec_id ?? "",
      );
      setRationale("");
      setKnownLimitations("");
      setChecks(REVIEW_CHECKS.map(() => false));
      setNotice(
        verdict() ===
          "approved_for_future_isolated_transformation_implementation_registration"
          ? "规范已通过独立复核；只允许未来登记隔离实现，尚未执行任何转换。"
          : "复核意见已不可覆盖写入；规范没有获得实现登记资格。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "转换规范独立复核失败");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section
          class="public-admin-reward-governance"
          aria-label="离线历史结果数据集转换规范独立复核"
        >
          <header>
            <strong>第 26 阶段 · 转换规范独立复核</strong>
            <span>{currentRegistry().review_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待复核</span><strong>{currentRegistry().review_eligible_count}</strong></div>
            <div><span>已复核</span><strong>{currentRegistry().reviewed_count}</strong></div>
            <div><span>历史批准</span><strong>{currentRegistry().approved_count}</strong></div>
            <div><span>当前实现登记资格</span><strong>{currentRegistry().implementation_registration_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>独立语义审计合同</strong><span>不复用登记生成器</span></header>
            <p>切分检查 {currentRegistry().review_contract.required_split_checks.length} 项；特征检查 {currentRegistry().review_contract.required_feature_checks.length} 项；合同 SHA {currentRegistry().review_contract.contract_sha256}</p>
            <p class="public-admin-anchor-boundary">批准范围只有未来隔离实现登记。实现、执行、输出验证、目标定义和训练继续分离。</p>
          </article>

          <Show when={currentRegistry().review_eligible_count > 0}>
            <label>
              <span>当前待复核转换规范</span>
              <select value={selectedSpecId()} onChange={(event) => setSelectedSpecId(event.currentTarget.value)}>
                <option value="">请选择待复核规范</option>
                <For each={currentRegistry().items.filter((item) => item.review_eligible)}>
                  {(item) => (
                    <option value={item.specification.transformation_spec_id}>
                      {item.specification.specification_name} · {item.specification.transformation_spec_id.slice(0, 12)}…
                    </option>
                  )}
                </For>
              </select>
            </label>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict)}>
                <option value="changes_requested">要求修改</option>
                <option value="rejected">拒绝</option>
                <option value="approved_for_future_isolated_transformation_implementation_registration">批准未来隔离实现登记</option>
              </select>
            </label>
            <label>
              <span>复核理由</span>
              <textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} placeholder="说明独立重放结果、算法歧义、防泄漏和点时来源判断" />
            </label>
            <label>
              <span>已知局限</span>
              <textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} placeholder="例如样本/分量失衡、交易日边界、特征来源覆盖或历史组合缺口" />
            </label>
            <div class="public-admin-decision-checks">
              <For each={REVIEW_CHECKS}>
                {(label, index) => (
                  <label>
                    <input type="checkbox" checked={checks()[index()]} onChange={(event) => toggleCheck(index(), event.currentTarget.checked)} />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
              写入独立复核（不登记实现、不执行）
            </button>
          </Show>

          <For each={currentRegistry().items}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.specification.specification_name}</strong>
                  <span>
                    {item.future_isolated_transformation_implementation_registration_eligible
                      ? "已批准 · 仅可登记未来隔离实现"
                      : item.review_eligible
                        ? "待独立复核"
                        : "绑定失效或未批准"}
                  </span>
                </header>
                <p>spec {item.specification.transformation_spec_id} · 登记人 {item.specification.registered_by} · 已有复核角色 {item.complete_review_actor_ids.length}</p>
                <Show when={item.latest_review}>{(review) => (
                  <>
                    <p>最新复核 {review().review_id} · {review().reviewer_id} · {review().verdict}</p>
                    <p>{review().rationale}</p>
                    <p>局限：{review().known_limitations}</p>
                  </>
                )}</Show>
                <p class="public-admin-anchor-boundary">manifest、bundle、join、目标、训练、奖励、影子、订单、券商和交易均未开放。</p>
              </article>
            )}
          </For>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
        </section>
      )}
    </Show>
  );
}
