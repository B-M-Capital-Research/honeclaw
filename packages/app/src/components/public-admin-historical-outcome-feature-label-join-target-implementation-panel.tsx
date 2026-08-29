import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeFeatureLabelJoinTargetImplementations,
  registerHistoricalOutcomeFeatureLabelJoinTargetImplementation,
} from "@/lib/api";
import type {
  HistoricalOutcomeFeatureLabelJoinTargetImplementationRegistry,
} from "@/lib/types";

const REGISTRATION_CHECKS = [
  "已精确绑定当前独立批准复核、规范、正式工件和数据集哈希",
  "登记人未参与完整上游登记与复核链",
  "实现工件 SHA-256 和代码版本不可变且可复现",
  "一对一 entry join 已冻结，重复键或缺键必须失败关闭",
  "点时可用、显式缺失、purge/embargo 与官方 split 隔离规则已冻结",
  "九维目标只投影原始 f64 位，不归一化、截尾、排序或设动作阈值",
  "sealed holdout 标签不可用于训练、调参或模型选择",
  "规范化序列化器和固定输入输出 schema 已冻结",
  "本登记没有可调用入口、环境、密钥、网络、工具或子进程",
  "登记、独立实现复核、runner、执行与输出验证彼此分离",
  "当前不读取标签、不执行 join、不创建 joined/training rows，不训练、奖励、影子、订单、券商或交易",
] as const;

export function PublicAdminHistoricalOutcomeFeatureLabelJoinTargetImplementationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetImplementationRegistry>();
  const [selectedReviewId, setSelectedReviewId] = createSignal("");
  const [implementationName, setImplementationName] = createSignal("");
  const [codeRevision, setCodeRevision] = createSignal("");
  const [artifactSha256, setArtifactSha256] = createSignal("");
  const [rationale, setRationale] = createSignal("");
  const [knownLimitations, setKnownLimitations] = createSignal("");
  const [checks, setChecks] = createSignal(REGISTRATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const eligibleReviews = createMemo(() => {
    const current = registry();
    if (!current) return [];
    const registered = new Set(
      current.items
        .filter((item) => item.upstream_binding_current)
        .map((item) => item.implementation.approved_review.review_id),
    );
    return current.eligible_reviews.filter((review) => !registered.has(review.review_id));
  });

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeFeatureLabelJoinTargetImplementations();
      setRegistry(next);
      setSelectedReviewId((current) =>
        next.eligible_reviews.some((review) => review.review_id === current) ? current : "",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 隔离实现登记表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    eligibleReviews().find((review) => review.review_id === selectedReviewId()),
  );
  const disabled = createMemo(
    () =>
      busy()
      || !selected()
      || !implementationName().trim()
      || !codeRevision().trim()
      || !/^[a-fA-F0-9]{64}$/.test(artifactSha256().trim())
      || !rationale().trim()
      || !knownLimitations().trim()
      || checks().some((value) => !value),
  );

  const submit = async () => {
    const review = selected();
    if (!review || disabled()) return;
    const specification = review.specification;
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await registerHistoricalOutcomeFeatureLabelJoinTargetImplementation({
        expected_review_id: review.review_id,
        expected_review_sha256: review.review_sha256,
        expected_review_contract_sha256: review.review_contract.contract_sha256,
        expected_independent_audit_sha256: review.independent_audit.audit_sha256,
        expected_specification_id: specification.specification_id,
        expected_specification_sha256: specification.specification_sha256,
        expected_specification_body_sha256: specification.specification_body_sha256,
        expected_join_specification_sha256: specification.join_specification.specification_sha256,
        expected_target_specification_sha256: specification.target_specification.specification_sha256,
        expected_combined_artifact_sha256: specification.combined_artifact_sha256,
        expected_dataset_id: specification.dataset_id,
        expected_dataset_content_sha256: specification.dataset_content_sha256,
        implementation_name: implementationName().trim(),
        immutable_code_revision: codeRevision().trim(),
        implementation_artifact_sha256: artifactSha256().trim().toLowerCase(),
        rationale: rationale().trim(),
        known_limitations: knownLimitations().trim(),
        exact_approved_review_specification_and_artifact_binding_confirmed: confirmed[0],
        registrar_independence_confirmed: confirmed[1],
        implementation_artifact_and_code_revision_immutable_confirmed: confirmed[2],
        exact_one_to_one_join_and_fail_closed_duplicate_missing_keys_confirmed: confirmed[3],
        point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: confirmed[4],
        exact_nine_raw_f64_target_projection_without_transform_confirmed: confirmed[5],
        sealed_holdout_labels_inaccessible_to_training_and_tuning_confirmed: confirmed[6],
        canonical_serialization_and_fixed_input_output_schema_confirmed: confirmed[7],
        no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: confirmed[8],
        registration_review_runner_execution_and_output_validation_separation_confirmed: confirmed[9],
        no_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed: confirmed[10],
      });
      setRegistry(next);
      setSelectedReviewId("");
      setImplementationName("");
      setCodeRevision("");
      setArtifactSha256("");
      setRationale("");
      setKnownLimitations("");
      setChecks(REGISTRATION_CHECKS.map(() => false));
      setNotice("join/target 隔离实现已不可覆盖登记；没有读取标签、执行 join 或创建任何数据行。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 隔离实现登记失败");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="join target 隔离实现登记">
          <header>
            <strong>第 38 阶段 · join/target 隔离实现登记</strong>
            <span>{currentRegistry().implementation_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可登记</span><strong>{currentRegistry().registration_eligible_count}</strong></div>
            <div><span>历史实现</span><strong>{currentRegistry().implementation_count}</strong></div>
            <div><span>当前绑定</span><strong>{currentRegistry().current_binding_implementation_count}</strong></div>
            <div><span>待独立实现复核</span><strong>{currentRegistry().independent_implementation_review_eligible_count}</strong></div>
          </div>

          <Show when={eligibleReviews().length > 0}>
            <label>
              <span>当前独立批准规范</span>
              <select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
                <option value="">请选择批准复核</option>
                <For each={eligibleReviews()}>
                  {(review) => <option value={review.review_id}>{review.specification.specification_name} · {review.review_id.slice(0, 12)}…</option>}
                </For>
              </select>
            </label>
            <label><span>实现名称</span><input value={implementationName()} onInput={(event) => setImplementationName(event.currentTarget.value)} placeholder="例如：严格一对一 join 与连续目标投影器 v1" /></label>
            <label><span>不可变代码版本</span><input value={codeRevision()} onInput={(event) => setCodeRevision(event.currentTarget.value)} placeholder="git commit / release digest" /></label>
            <label><span>实现工件 SHA-256</span><input value={artifactSha256()} onInput={(event) => setArtifactSha256(event.currentTarget.value)} placeholder="64 位十六进制摘要" /></label>
            <label><span>登记理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} placeholder="说明实现如何忠实落地已独立批准规范" /></label>
            <label><span>已知局限</span><textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} placeholder="必须说明尚未独立复核、未运行且工程目标不代表策略真理" /></label>
            <div class="public-admin-decision-checks">
              <For each={REGISTRATION_CHECKS}>
                {(label, index) => (
                  <label>
                    <input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((current) => current.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
              登记实现合同（无入口、不执行）
            </button>
          </Show>

          <For each={currentRegistry().items}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.implementation.implementation_name}</strong>
                  <span>{item.upstream_binding_current ? "当前绑定 · 等待独立实现复核" : "历史绑定"}</span>
                </header>
                <p>实现 {item.implementation.implementation_id} · 登记人 {item.implementation.registered_by} · {item.implementation.status}</p>
                <p>工件 {item.implementation.implementation_contract.implementation_artifact_sha256} · 代码 {item.implementation.implementation_contract.immutable_code_revision}</p>
                <p>固定规模：65 项特征 · 9 项连续目标 · 20/60/250 个市场日。</p>
                <p>{item.implementation.rationale}</p>
                <p>局限：{item.implementation.known_limitations}</p>
                <p class="public-admin-anchor-boundary">没有可调用入口或标签权限；join、目标分配、joined/training rows、训练、奖励、影子、订单、券商和交易全部关闭。下一步只有独立实现复核。</p>
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
