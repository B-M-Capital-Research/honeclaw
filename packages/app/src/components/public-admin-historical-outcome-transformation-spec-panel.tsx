import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeOfflineDatasetTransformationSpecs,
  registerHistoricalOutcomeOfflineDatasetTransformationSpec,
} from "@/lib/api";
import type { HistoricalOutcomeOfflineDatasetTransformationSpecRegistry } from "@/lib/types";

const REGISTRATION_CHECKS = [
  "已核对当前数据集、manifest、候选集和治理批准的精确哈希绑定",
  "登记人未参与数据集完整上游链或第 24 阶段治理复核",
  "公司、历史事件与来源家族通过传递闭包形成不可拆分连通分量",
  "枚举全部时间连续边界并按精确整数目标选择；SHA-256 只打破完全相同时间并列",
  "70 / 15 / 15、共同交易日历、250 日 purge / embargo、空分区失败和封存标签隔离均已冻结",
  "所有特征都必须保留来源并满足 available_at 不晚于历史判断时间",
  "特征只允许产业、公司、财务、估值、拥挤、宏观和组合上下文七层中的精确 feature ID",
  "结果、标签、未来行情及其校验和准入字段不得伪装到白名单 namespace 中",
  "缺失和 available_at 歧义必须显式保留并失败关闭，不回填、不插值",
  "规范登记、独立复核和未来执行是三道不同权限",
  "本阶段不切分、不连接特征、不生成目标，也不授权训练、奖励、影子、订单、券商或交易",
] as const;

export function PublicAdminHistoricalOutcomeTransformationSpecPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeOfflineDatasetTransformationSpecRegistry>();
  const [selectedDatasetId, setSelectedDatasetId] = createSignal("");
  const [specificationName, setSpecificationName] = createSignal(
    "历史结果数据集确定性切分与七层点时特征规范",
  );
  const [codeRevision, setCodeRevision] = createSignal("");
  const [rationale, setRationale] = createSignal("");
  const [knownLimitations, setKnownLimitations] = createSignal("");
  const [checks, setChecks] = createSignal(REGISTRATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeOfflineDatasetTransformationSpecs();
      setRegistry(next);
      const eligible = next.eligible_subjects.some(
        (item) => item.subject.dataset_id === selectedDatasetId(),
      );
      if (!eligible) {
        setSelectedDatasetId(next.eligible_subjects[0]?.subject.dataset_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "转换规范注册表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.eligible_subjects.find(
      (item) => item.subject.dataset_id === selectedDatasetId(),
    ),
  );

  const disabled = createMemo(
    () =>
      busy() ||
      !selected() ||
      !specificationName().trim() ||
      !codeRevision().trim() ||
      !rationale().trim() ||
      !knownLimitations().trim() ||
      checks().some((value) => !value),
  );

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) =>
      current.map((value, currentIndex) => (currentIndex === index ? checked : value)),
    );
  };

  const register = async () => {
    const current = selected();
    if (!current || disabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const confirmed = checks();
      const next = await registerHistoricalOutcomeOfflineDatasetTransformationSpec(
        current.subject.dataset_id,
        {
          expected_dataset_content_sha256: current.subject.dataset_content_sha256,
          expected_manifest_sha256: current.subject.manifest_sha256,
          expected_candidate_set_sha256: current.subject.candidate_set_sha256,
          expected_governance_review_id: current.governance_review_id,
          expected_governance_review_sha256: current.governance_review_sha256,
          expected_split_policy_sha256: current.split_policy_sha256,
          expected_feature_join_policy_sha256: current.feature_join_policy_sha256,
          specification_name: specificationName().trim(),
          code_revision: codeRevision().trim(),
          rationale: rationale().trim(),
          known_limitations: knownLimitations().trim(),
          exact_dataset_and_governance_binding_confirmed: confirmed[0],
          registrar_independence_confirmed: confirmed[1],
          transitive_component_isolation_confirmed: confirmed[2],
          chronological_boundaries_and_hash_tie_break_confirmed: confirmed[3],
          purge_embargo_and_sealed_holdout_confirmed: confirmed[4],
          point_in_time_availability_and_provenance_confirmed: confirmed[5],
          seven_layer_namespace_allowlist_confirmed: confirmed[6],
          label_outcome_and_future_information_exclusion_confirmed: confirmed[7],
          missingness_fail_closed_without_imputation_confirmed: confirmed[8],
          registration_review_execution_separation_confirmed: confirmed[9],
          no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed:
            confirmed[10],
        },
      );
      setRegistry(next);
      setSelectedDatasetId(next.eligible_subjects[0]?.subject.dataset_id ?? "");
      setRationale("");
      setKnownLimitations("");
      setChecks(REGISTRATION_CHECKS.map(() => false));
      setNotice("不可变转换规范已登记；下一步仍需另一位独立复核人审查，尚未执行。 ");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "转换规范登记失败");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>{(currentRegistry) => (
      <section
        class="public-admin-reward-governance"
        aria-label="离线历史结果数据集不可变转换规范登记"
      >
        <header>
          <strong>第 25 阶段 · 不可变转换规范登记</strong>
          <span>{currentRegistry().transformation_spec_status}</span>
        </header>
        <p>{currentRegistry().scope}</p>
        <div class="public-admin-decision-metrics">
          <div><span>可登记</span><strong>{currentRegistry().registration_eligible_count}</strong></div>
          <div><span>历史登记</span><strong>{currentRegistry().registered_count}</strong></div>
          <div><span>当前绑定</span><strong>{currentRegistry().current_binding_registered_count}</strong></div>
          <div><span>待独立复核</span><strong>{currentRegistry().independent_review_eligible_count}</strong></div>
        </div>

        <article class="public-admin-reward-governance">
          <header><strong>确定性切分 manifest 合同</strong><span>不生成 manifest</span></header>
          <p>
            先按公司、历史事件和来源家族构建传递连通分量，再按组件最晚、最早判断时间排序；只在时间完全相同时用组件 SHA-256 破同分。枚举全部连续边界，以精确整数误差目标确定 {currentRegistry().split_manifest_specification.train_percent}% / {currentRegistry().split_manifest_specification.validation_percent}% / {currentRegistry().split_manifest_specification.sealed_holdout_percent}% 分区。
          </p>
          <p>
            共同资产/SPY 交易日历上执行 {currentRegistry().split_manifest_specification.purge_embargo_market_sessions} 日 purge / embargo；处理后任一分区为空就失败关闭，封存集标签不能暴露给未来训练 worker。规范 SHA：{currentRegistry().split_manifest_specification.specification_sha256}
          </p>
        </article>

        <article class="public-admin-reward-governance">
          <header><strong>七层点时特征 bundle 合同</strong><span>不生成 bundle</span></header>
          <p>
            七层白名单：{currentRegistry().feature_bundle_specification.allowed_feature_namespaces.join("、")}；层内只允许 {currentRegistry().feature_bundle_specification.allowed_features.length} 个精确 feature ID，不能用改名绕过语义。
          </p>
          <p>
            {currentRegistry().feature_bundle_specification.availability_rule}；后续重述版本、当前持仓替代历史组合、无人工证据链的定性状态均禁止。缺失、来源缺口和时间歧义显式保留，禁止回填与插值。规范 SHA：{currentRegistry().feature_bundle_specification.specification_sha256}
          </p>
        </article>

        <Show when={currentRegistry().registration_eligible_count > 0}>
          <label>
            <span>当前已批准治理的数据集</span>
            <select
              value={selectedDatasetId()}
              onChange={(event) => setSelectedDatasetId(event.currentTarget.value)}
            >
              <option value="">请选择待登记数据集</option>
              <For each={currentRegistry().eligible_subjects}>{(item) => (
                <option value={item.subject.dataset_id}>
                  {item.subject.dataset_version} · {item.subject.entry_count} 条 · governance {item.governance_review_id.slice(0, 12)}…
                </option>
              )}</For>
            </select>
          </label>
          <label>
            <span>规范名称</span>
            <input value={specificationName()} onInput={(event) => setSpecificationName(event.currentTarget.value)} />
          </label>
          <label>
            <span>代码版本 / revision</span>
            <input value={codeRevision()} onInput={(event) => setCodeRevision(event.currentTarget.value)} placeholder="例如 git commit SHA；只做溯源，不代表执行代码已获批" />
          </label>
          <label>
            <span>登记理由</span>
            <textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} placeholder="说明算法可复现性、防泄漏边界与为何适合进入独立复核" />
          </label>
          <label>
            <span>已知局限</span>
            <textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} placeholder="例如样本量、连通分量失衡、特征来源覆盖和点时时间精度" />
          </label>
          <div class="public-admin-decision-checks">
            <For each={REGISTRATION_CHECKS}>{(label, index) => (
              <label>
                <input type="checkbox" checked={checks()[index()]} onChange={(event) => toggleCheck(index(), event.currentTarget.checked)} />
                <span>{label}</span>
              </label>
            )}</For>
          </div>
          <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void register()}>
            登记不可变转换规范（不执行）
          </button>
        </Show>

        <For each={currentRegistry().items}>{(item) => (
          <article class="public-admin-reward-governance">
            <header>
              <strong>{item.specification.specification_name}</strong>
              <span>{item.upstream_binding_current ? "当前绑定 · 待独立复核" : "上游绑定已失效"}</span>
            </header>
            <p>spec {item.specification.transformation_spec_id} · 登记人 {item.specification.registered_by} · 治理复核人 {item.specification.governance_reviewer_id}</p>
            <p>body SHA {item.specification.transformation_body_sha256} · revision {item.specification.code_revision}</p>
            <p>{item.specification.rationale}</p>
            <p>局限：{item.specification.known_limitations}</p>
            <p class="public-admin-anchor-boundary">独立复核：未完成；切分 manifest：未生成；点时特征 bundle：未生成；语义目标、训练、奖励、影子、订单、券商和交易：全部关闭。</p>
          </article>
        )}</For>
        <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
        <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
      </section>
    )}</Show>
  );
}
