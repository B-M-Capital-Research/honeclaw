import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeFeatureLabelJoinTargetSpecs,
  registerHistoricalOutcomeFeatureLabelJoinTargetSpec,
} from "@/lib/api";
import type { HistoricalOutcomeFeatureLabelJoinTargetSpecRegistry } from "@/lib/types";

const CHECKS = [
  "精确绑定当前独立校验通过的正式 manifest、feature bundle、物化结果与原始离线数据集",
  "登记者不是正式工件校验人、物化人或完整上游角色",
  "每个 dataset_entry_id 只允许一个切分记录、一个原始结果记录和每个 allowlist feature 一条记录",
  "purged / embargoed 条目完全排除且保留审计，不得重新分区",
  "所有 feature 的 available_at 不晚于历史 decision_available_at",
  "sealed holdout 标签在模型和评测协议冻结前持续隔离，不进入训练或调参",
  "20/60/250 日原始指标保留精确 f64 位，不标准化、不 winsorize、不排名",
  "目标是连续前瞻结果向量，不是买卖动作、仓位或奖励",
  "65 项 allowlist feature 全部保留显式缺失，不插值、不回填、不因缺失删列",
  "规范登记、独立复核、join 物化、输出校验和训练授权必须继续分门",
  "本阶段不 join、不分配目标、不创建训练行，也不训练、奖励、影子、订单、券商或交易",
] as const;

export function PublicAdminHistoricalOutcomeFeatureLabelJoinTargetSpecPanel() {
  const [registry, setRegistry] = createSignal<HistoricalOutcomeFeatureLabelJoinTargetSpecRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [name, setName] = createSignal("HONE 连续前瞻结果 join/target 治理规范 v1");
  const [revision, setRevision] = createSignal("stage-36-governance-spec-v1");
  const [rationale, setRationale] = createSignal("以精确点时特征预测 20/60/250 个共同交易日的连续资产路径与相对基准结果，同时保持 sealed holdout 隔离。");
  const [limitations, setLimitations] = createSignal("本规范尚未经过独立复核，未执行任何连接或目标分配；样本规模、行业覆盖与缺失率仍可能不足以支持训练。");
  const [checks, setChecks] = createSignal<boolean[]>(CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeFeatureLabelJoinTargetSpecs();
      setRegistry(next);
      if (!next.subjects.some((subject) => subject.transformation_attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(
          next.subjects.find((subject) => subject.registration_eligible)?.transformation_attempt_id
            ?? next.subjects[0]?.transformation_attempt_id
            ?? "",
        );
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 治理规范注册表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.subjects.find((subject) => subject.transformation_attempt_id === selectedAttemptId()),
  );
  const allChecked = createMemo(() => checks().every(Boolean));
  const canRegister = createMemo(() =>
    Boolean(
      selected()?.registration_eligible
      && allChecked()
      && name().trim()
      && revision().trim()
      && rationale().trim()
      && limitations().trim(),
    ),
  );

  const register = async () => {
    const subject = selected();
    if (!subject || !canRegister() || busy()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await registerHistoricalOutcomeFeatureLabelJoinTargetSpec(
        subject.transformation_attempt_id,
        {
          expected_validation_id: subject.validation_id,
          expected_validation_sha256: subject.validation_sha256,
          expected_materialization_id: subject.materialization_id,
          expected_materialization_claim_sha256: subject.materialization_claim_sha256,
          expected_materialization_result_sha256: subject.materialization_result_sha256,
          expected_split_manifest_sha256: subject.split_manifest_sha256,
          expected_feature_bundle_sha256: subject.feature_bundle_sha256,
          expected_combined_artifact_sha256: subject.combined_artifact_sha256,
          expected_dataset_id: subject.dataset_id,
          expected_dataset_content_sha256: subject.dataset_content_sha256,
          expected_dataset_manifest_sha256: subject.dataset_manifest_sha256,
          expected_candidate_set_sha256: subject.candidate_set_sha256,
          specification_name: name().trim(),
          code_revision: revision().trim(),
          rationale: rationale().trim(),
          known_limitations: limitations().trim(),
          exact_validated_artifact_pair_binding_confirmed: true,
          registrar_independence_confirmed: true,
          exact_dataset_entry_one_to_one_join_confirmed: true,
          purged_and_embargoed_rows_excluded_confirmed: true,
          point_in_time_feature_availability_confirmed: true,
          sealed_holdout_target_isolation_confirmed: true,
          exact_raw_metric_bits_without_transform_confirmed: true,
          continuous_target_vector_not_action_or_reward_confirmed: true,
          explicit_missingness_without_imputation_confirmed: true,
          registration_review_execution_separation_confirmed: true,
          no_join_target_assignment_training_reward_shadow_order_broker_or_trading_confirmed: true,
        },
      );
      setRegistry(next);
      setChecks(CHECKS.map(() => false));
      setNotice("join/target 治理规范已不可变登记；下一步只能由另一名管理员独立复核，当前仍未执行连接或训练。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 治理规范登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="特征标签连接与目标治理规范">
          <header>
            <strong>第 36 阶段 · join/target 治理规范登记</strong>
            <span>{currentRegistry().registration_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可登记</span><strong>{currentRegistry().registration_eligible_count}</strong></div>
            <div><span>规范总数</span><strong>{currentRegistry().specification_count}</strong></div>
            <div><span>当前绑定</span><strong>{currentRegistry().current_binding_specification_count}</strong></div>
            <div><span>待独立复核</span><strong>{currentRegistry().independent_review_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>目标不是“买/卖标签”</strong><span>连续结果向量</span></header>
            <p>主目标候选为 250 日相对 SPY 超额收益；风险目标为 250 日最大回撤；20/60 日资产收益、超额收益和回撤，以及 250 日资产收益作为路径辅助目标。</p>
            <p>九个目标只保留原始连续值和精确浮点位，不定义买入、持有、卖出、仓位、阈值或奖励。</p>
            <p class="public-admin-anchor-boundary">本阶段只登记语义与防泄漏合同；join、目标分配、训练行、训练与交易全部关闭。</p>
          </article>

          <Show when={currentRegistry().subjects.length > 0} fallback={<p>当前没有通过独立校验的正式工件可登记规范。</p>}>
            <label>
              <span>正式工件 attempt</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={currentRegistry().subjects}>{(subject) => (
                  <option value={subject.transformation_attempt_id}>
                    {subject.transformation_attempt_id.slice(0, 12)}… · {subject.registration_eligible ? "待登记" : "已登记"}
                  </option>
                )}</For>
              </select>
            </label>
            <label><span>规范名称</span><input value={name()} onInput={(event) => setName(event.currentTarget.value)} /></label>
            <label><span>代码/合同版本</span><input value={revision()} onInput={(event) => setRevision(event.currentTarget.value)} /></label>
            <label><span>登记理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
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
            <button type="button" class="public-admin-decision-submit" disabled={busy() || !canRegister()} onClick={() => void register()}>
              {busy() ? "正在登记不可变规范…" : "登记 join/target 治理规范"}
            </button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>

          <For each={currentRegistry().subjects}>{(subject) => (
            <Show when={subject.registered_specification}>{(record) => (
              <article class="public-admin-reward-governance">
                <header><strong>{record().specification_name}</strong><span>{record().status}</span></header>
                <p>规范 {record().specification_id} · 登记者 {record().registered_by} · {record().registered_at}</p>
                <p>主目标 {record().target_specification.primary_supervised_target_id} · 风险目标 {record().target_specification.risk_target_id} · 共 {record().target_specification.target_definitions.length} 项连续目标</p>
                <p>65 项特征目录：{record().join_specification.feature_catalog_count} · horizon：{record().join_specification.allowed_label_horizons_market_sessions.join(" / ")}</p>
                <p>{record().rationale}</p>
                <p>局限：{record().known_limitations}</p>
                <p class="public-admin-anchor-boundary">仅可进入未来独立规范复核；join、目标分配、训练、奖励、影子、订单、券商和交易仍关闭。</p>
              </article>
            )}</Show>
          )}</For>
        </section>
      )}
    </Show>
  );
}
