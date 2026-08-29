import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeOfflineDatasetTransformationImplementationReviews,
  reviewHistoricalOutcomeOfflineDatasetTransformationImplementation,
} from "@/lib/api";
import type {
  HistoricalOutcomeOfflineDatasetTransformationImplementationReviewRegistry,
  HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "已重新核对当前实现、实现合同、工件以及完整上游哈希绑定",
  "复核人未参与数据集、治理、规范登记/复核或实现登记链",
  "独立复现实现工件 SHA-256，没有只信任登记器字段",
  "不可变代码版本可获取、可重现并与登记值一致",
  "确定性切分实现 ID、版本与精确边界规范一致",
  "点时特征实现与七层固定 65 个 feature ID 白名单逐项一致",
  "规范化序列化器和固定输入/输出 schema 可确定性复现",
  "输入保持 sealed read-only，未来输出只能 create-once、内容寻址",
  "资源上限固定为单 subject、2048 MiB，不可静默扩容",
  "没有入口、环境继承/变量、密钥、网络、外部工具、子进程或生产访问",
  "复核、runner 登记、执行授权、输出校验、目标定义和训练相互分离",
  "本阶段不登记 runner、不运行、不生成 manifest/bundle、不 join、不写目标、不训练、奖励、影子、订单、券商或交易",
] as const;

export function PublicAdminHistoricalOutcomeTransformationImplementationReviewPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeOfflineDatasetTransformationImplementationReviewRegistry>();
  const [selectedImplementationId, setSelectedImplementationId] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict>(
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
      const next =
        await getHistoricalOutcomeOfflineDatasetTransformationImplementationReviews();
      setRegistry(next);
      const selectedStillEligible = next.items.some(
        (item) =>
          item.review_eligible &&
          item.implementation.implementation_id === selectedImplementationId(),
      );
      if (!selectedStillEligible) {
        setSelectedImplementationId(
          next.items.find((item) => item.review_eligible)?.implementation
            .implementation_id ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "隔离转换实现独立复核注册表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find(
      (item) =>
        item.review_eligible &&
        item.implementation.implementation_id === selectedImplementationId(),
    ),
  );
  const approving = createMemo(
    () => verdict() === "approved_for_future_isolated_transformation_runner_registration",
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
    const implementation = current.implementation;
    const contract = implementation.implementation_contract;
    const approvedReview = implementation.approved_review;
    const specification = approvedReview.specification;
    const prior = current.latest_review;
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next =
        await reviewHistoricalOutcomeOfflineDatasetTransformationImplementation(
          implementation.implementation_id,
          {
            expected_previous_review_id: prior?.review_id,
            expected_previous_review_sha256: prior?.review_sha256,
            expected_implementation_sha256: implementation.implementation_sha256,
            expected_implementation_contract_sha256: contract.contract_sha256,
            expected_implementation_artifact_sha256: contract.implementation_artifact_sha256,
            expected_immutable_code_revision: contract.immutable_code_revision,
            expected_specification_review_sha256: approvedReview.review_sha256,
            expected_transformation_spec_sha256: specification.transformation_spec_sha256,
            expected_transformation_body_sha256: specification.transformation_body_sha256,
            expected_split_specification_sha256:
              specification.split_manifest_specification.specification_sha256,
            expected_feature_specification_sha256:
              specification.feature_bundle_specification.specification_sha256,
            expected_dataset_content_sha256: specification.subject.dataset_content_sha256,
            expected_manifest_sha256: specification.subject.manifest_sha256,
            expected_candidate_set_sha256: specification.subject.candidate_set_sha256,
            expected_governance_review_sha256: specification.governance_review_sha256,
            expected_review_contract_sha256: currentRegistry.review_contract.contract_sha256,
            verdict: verdict(),
            rationale: rationale().trim(),
            known_limitations: knownLimitations().trim(),
            exact_current_implementation_and_upstream_binding_confirmed: confirmed[0],
            reviewer_independence_confirmed: confirmed[1],
            artifact_digest_independently_reproduced_confirmed: confirmed[2],
            immutable_code_revision_reproducible_confirmed: confirmed[3],
            deterministic_split_implementation_matches_specification_confirmed: confirmed[4],
            exact_65_feature_implementation_matches_allowlist_confirmed: confirmed[5],
            canonical_serializer_and_schema_determinism_confirmed: confirmed[6],
            sealed_read_only_input_and_create_once_output_contract_confirmed: confirmed[7],
            bounded_resource_contract_confirmed: confirmed[8],
            no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed:
              confirmed[9],
            review_runner_execution_output_target_and_training_separation_confirmed:
              confirmed[10],
            no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed:
              confirmed[11],
          },
        );
      setRegistry(next);
      setSelectedImplementationId(
        next.items.find((item) => item.review_eligible)?.implementation.implementation_id ??
          "",
      );
      setRationale("");
      setKnownLimitations("");
      setChecks(REVIEW_CHECKS.map(() => false));
      setNotice(
        verdict() === "approved_for_future_isolated_transformation_runner_registration"
          ? "实现已通过独立复核；只允许未来登记隔离 runner 规范，尚未运行或生成任何输出。"
          : "复核意见已不可覆盖写入；实现没有获得 runner 规范登记资格。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "隔离转换实现独立复核失败");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section
          class="public-admin-reward-governance"
          aria-label="隔离转换实现独立复核"
        >
          <header>
            <strong>第 28 阶段 · 隔离转换实现独立复核</strong>
            <span>{currentRegistry().review_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待复核</span><strong>{currentRegistry().review_eligible_count}</strong></div>
            <div><span>已复核</span><strong>{currentRegistry().reviewed_count}</strong></div>
            <div><span>当前批准</span><strong>{currentRegistry().current_binding_approved_count}</strong></div>
            <div><span>runner 规范登记资格</span><strong>{currentRegistry().runner_registration_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>独立工件与沙箱审计合同</strong><span>无入口 · 不运行</span></header>
            <p>工件检查 {currentRegistry().review_contract.required_artifact_checks.length} 项；沙箱检查 {currentRegistry().review_contract.required_sandbox_checks.length} 项；合同 SHA {currentRegistry().review_contract.contract_sha256}</p>
            <p class="public-admin-anchor-boundary">批准范围只有未来隔离 runner 规范登记。runner、执行授权、输出校验、目标定义和训练继续分离。</p>
          </article>

          <Show when={currentRegistry().review_eligible_count > 0}>
            <label>
              <span>当前待复核实现</span>
              <select value={selectedImplementationId()} onChange={(event) => setSelectedImplementationId(event.currentTarget.value)}>
                <option value="">请选择待复核实现</option>
                <For each={currentRegistry().items.filter((item) => item.review_eligible)}>
                  {(item) => (
                    <option value={item.implementation.implementation_id}>
                      {item.implementation.implementation_name} · {item.implementation.implementation_id.slice(0, 12)}…
                    </option>
                  )}
                </For>
              </select>
            </label>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict)}>
                <option value="changes_requested">要求修改</option>
                <option value="rejected">拒绝</option>
                <option value="approved_for_future_isolated_transformation_runner_registration">批准未来隔离 runner 规范登记</option>
              </select>
            </label>
            <label>
              <span>复核理由</span>
              <textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} placeholder="说明独立工件复现、代码版本、确定性算法与沙箱审计结果" />
            </label>
            <label>
              <span>已知局限</span>
              <textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} placeholder="例如可复现环境、构建链、资源边界或 schema 覆盖局限" />
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
              写入独立实现复核（不登记 runner、不执行）
            </button>
          </Show>

          <For each={currentRegistry().items}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.implementation.implementation_name}</strong>
                  <span>
                    {item.future_isolated_transformation_runner_registration_eligible
                      ? "已批准 · 仅可登记 runner 规范"
                      : item.review_eligible
                        ? "待独立复核"
                        : "绑定失效或未批准"}
                  </span>
                </header>
                <p>实现 {item.implementation.implementation_id} · 代码 {item.implementation.implementation_contract.immutable_code_revision} · 登记人 {item.implementation.registered_by}</p>
                <Show when={item.latest_review}>{(review) => (
                  <>
                    <p>最新复核 {review().review_id} · {review().reviewer_id} · {review().verdict}</p>
                    <p>{review().rationale}</p>
                    <p>局限：{review().known_limitations}</p>
                  </>
                )}</Show>
                <p class="public-admin-anchor-boundary">runner、执行、manifest、bundle、join、目标、训练、奖励、影子、订单、券商和交易均未开放。</p>
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
