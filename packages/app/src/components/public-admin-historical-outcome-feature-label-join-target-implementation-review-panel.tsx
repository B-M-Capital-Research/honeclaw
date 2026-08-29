import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeFeatureLabelJoinTargetImplementationReviews,
  reviewHistoricalOutcomeFeatureLabelJoinTargetImplementation,
} from "@/lib/api";
import type {
  HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewRegistry,
  HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "已重新核对当前实现、实现合同以及完整上游复核、规范、正式工件和数据集哈希绑定",
  "复核人未参与完整上游登记、复核与实现登记链",
  "已独立复现实现记录与实现合同 SHA-256，没有只信任登记字段",
  "实现工件摘要和不可变代码版本可获取、可复现且一致",
  "严格一对一 entry join 与重复键、缺键失败关闭语义一致",
  "九维目标严格保留原始 f64 位，不做归一化、截尾、排序或动作阈值",
  "点时可用、显式缺失、purge/embargo 与官方 split 隔离均有效",
  "sealed holdout 标签不能用于训练、调参或模型选择",
  "规范化序列化器、固定 schema 与单数据集/4096 MiB 资源上限一致",
  "实现不包含动作、仓位、阈值、排名或奖励语义",
  "实现没有入口、环境、密钥、网络、工具、子进程或数据存储访问",
  "独立复核、runner 登记、首次授权、join 执行、输出校验和训练治理彼此分离",
  "当前没有 runner、标签访问、join rows、训练、奖励、影子、订单、券商或交易权限",
] as const;

export function PublicAdminHistoricalOutcomeFeatureLabelJoinTargetImplementationReviewPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewRegistry>();
  const [selectedImplementationId, setSelectedImplementationId] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict>(
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
      const next = await getHistoricalOutcomeFeatureLabelJoinTargetImplementationReviews();
      setRegistry(next);
      if (!next.items.some((item) => item.review_eligible && item.implementation.implementation_id === selectedImplementationId())) {
        setSelectedImplementationId(next.items.find((item) => item.review_eligible)?.implementation.implementation_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 实现独立复核注册表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() => registry()?.items.find(
    (item) => item.review_eligible && item.implementation.implementation_id === selectedImplementationId(),
  ));
  const approving = createMemo(
    () => verdict() === "approved_for_future_isolated_join_target_runner_registration",
  );
  const disabled = createMemo(() =>
    busy() || !selected() || !rationale().trim() || !knownLimitations().trim()
    || (approving() && checks().some((value) => !value)),
  );

  const submit = async () => {
    const item = selected();
    const currentRegistry = registry();
    if (!item || !currentRegistry || disabled()) return;
    const implementation = item.implementation;
    const implementationContract = implementation.implementation_contract;
    const approvedReview = implementation.approved_review;
    const specification = approvedReview.specification;
    const prior = item.latest_review;
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await reviewHistoricalOutcomeFeatureLabelJoinTargetImplementation(
        implementation.implementation_id,
        {
          expected_previous_review_id: prior?.review_id,
          expected_previous_review_sha256: prior?.review_sha256,
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_contract_sha256: implementationContract.contract_sha256,
          expected_implementation_artifact_sha256: implementationContract.implementation_artifact_sha256,
          expected_immutable_code_revision: implementationContract.immutable_code_revision,
          expected_specification_review_sha256: approvedReview.review_sha256,
          expected_specification_review_audit_sha256: approvedReview.independent_audit.audit_sha256,
          expected_specification_sha256: specification.specification_sha256,
          expected_specification_body_sha256: specification.specification_body_sha256,
          expected_join_specification_sha256: specification.join_specification.specification_sha256,
          expected_target_specification_sha256: specification.target_specification.specification_sha256,
          expected_combined_artifact_sha256: specification.combined_artifact_sha256,
          expected_dataset_content_sha256: specification.dataset_content_sha256,
          expected_review_contract_sha256: currentRegistry.review_contract.contract_sha256,
          expected_independent_audit_sha256: item.current_independent_audit.audit_sha256,
          verdict: verdict(),
          rationale: rationale().trim(),
          known_limitations: knownLimitations().trim(),
          exact_current_implementation_and_complete_upstream_binding_confirmed: confirmed[0],
          reviewer_independence_from_complete_prior_chain_confirmed: confirmed[1],
          implementation_record_and_contract_hashes_independently_reproduced_confirmed: confirmed[2],
          implementation_artifact_digest_and_code_revision_reproducible_confirmed: confirmed[3],
          exact_one_to_one_join_and_fail_closed_key_semantics_confirmed: confirmed[4],
          exact_nine_raw_f64_target_projection_without_transform_confirmed: confirmed[5],
          point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: confirmed[6],
          sealed_holdout_labels_inaccessible_to_training_tuning_and_model_selection_confirmed: confirmed[7],
          canonical_serializer_fixed_schemas_and_resource_limits_confirmed: confirmed[8],
          no_action_position_threshold_rank_or_reward_semantics_confirmed: confirmed[9],
          no_entrypoint_environment_secrets_network_tools_child_process_or_data_store_access_confirmed: confirmed[10],
          review_runner_authorization_execution_output_validation_and_training_separation_confirmed: confirmed[11],
          no_runner_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed: confirmed[12],
        },
      );
      setRegistry(next);
      setSelectedImplementationId(next.items.find((candidate) => candidate.review_eligible)?.implementation.implementation_id ?? "");
      setRationale("");
      setKnownLimitations("");
      setChecks(REVIEW_CHECKS.map(() => false));
      setNotice(approving()
        ? "实现已通过独立复核；只允许未来登记隔离 runner 规格，当前仍没有执行或训练权限。"
        : "复核意见已不可覆盖写入；实现没有获得 runner 规格登记资格。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 实现独立复核失败");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="join target 实现独立复核">
          <header>
            <strong>第 39 阶段 · join/target 实现独立复核</strong>
            <span>{currentRegistry().review_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待复核</span><strong>{currentRegistry().review_eligible_count}</strong></div>
            <div><span>已复核</span><strong>{currentRegistry().reviewed_count}</strong></div>
            <div><span>当前批准</span><strong>{currentRegistry().current_binding_approved_count}</strong></div>
            <div><span>runner 规格登记资格</span><strong>{currentRegistry().runner_registration_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>独立指纹、语义与沙箱审计合同</strong><span>目标仍是工程候选</span></header>
            <p>指纹 {currentRegistry().review_contract.required_fingerprint_checks.length} 项 · 语义 {currentRegistry().review_contract.required_semantic_checks.length} 项 · 沙箱 {currentRegistry().review_contract.required_sandbox_checks.length} 项</p>
            <p class="public-admin-anchor-boundary">批准只允许未来登记隔离 runner 规格。九维目标仍是工程候选，不是策略真理；首次执行、join、输出校验、训练和奖励继续独立治理。</p>
          </article>

          <Show when={currentRegistry().review_eligible_count > 0}>
            <label>
              <span>当前待复核实现</span>
              <select value={selectedImplementationId()} onChange={(event) => setSelectedImplementationId(event.currentTarget.value)}>
                <option value="">请选择待复核实现</option>
                <For each={currentRegistry().items.filter((item) => item.review_eligible)}>
                  {(item) => <option value={item.implementation.implementation_id}>{item.implementation.implementation_name} · {item.implementation.implementation_id.slice(0, 12)}…</option>}
                </For>
              </select>
            </label>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict)}>
                <option value="changes_requested">要求修改</option>
                <option value="rejected">拒绝</option>
                <option value="approved_for_future_isolated_join_target_runner_registration">批准未来隔离 runner 规格登记</option>
              </select>
            </label>
            <label><span>复核理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} placeholder="说明独立重算的指纹、join/target 语义和沙箱检查结果" /></label>
            <label><span>已知局限</span><textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} placeholder="必须说明工程目标不等于投资策略真理，并列出复现或覆盖局限" /></label>
            <div class="public-admin-decision-checks">
              <For each={REVIEW_CHECKS}>{(label, index) => (
                <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((current) => current.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>
              )}</For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
              写入独立实现复核（不登记 runner、不执行）
            </button>
          </Show>

          <For each={currentRegistry().items}>{(item) => (
            <article class="public-admin-reward-governance">
              <header>
                <strong>{item.implementation.implementation_name}</strong>
                <span>{item.future_isolated_join_target_runner_registration_eligible ? "已批准 · 仅可登记 runner 规格" : item.review_eligible ? "待独立复核" : "绑定失效或未批准"}</span>
              </header>
              <p>实现 {item.implementation.implementation_id} · 代码 {item.implementation.implementation_contract.immutable_code_revision}</p>
              <p>独立审计 {item.current_independent_audit.audit_sha256} · 不一致 {item.current_independent_audit.mismatch_reasons.length} 项</p>
              <Show when={item.latest_review}>{(review) => (<><p>最新复核 {review().review_id} · {review().reviewer_id} · {review().verdict}</p><p>{review().rationale}</p><p>局限：{review().known_limitations}</p></>)}</Show>
              <p class="public-admin-anchor-boundary">没有 runner、标签访问、join、joined/training rows、输出校验、训练、奖励、影子、订单、券商或交易权限。</p>
            </article>
          )}</For>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
        </section>
      )}
    </Show>
  );
}
