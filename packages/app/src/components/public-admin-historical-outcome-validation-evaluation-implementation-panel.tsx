import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeValidationEvaluationImplementations,
  registerHistoricalOutcomeValidationEvaluationImplementation,
} from "@/lib/api";
import type {
  HistoricalOutcomeValidationEvaluationImplementationRegistry,
  RegisterHistoricalOutcomeValidationEvaluationImplementationRequest,
} from "@/lib/types";

const CHECKS = [
  ["exact_stage_58_validation_and_stage_57_output_binding_confirmed", "精确绑定 Stage 58 独立验证、Stage 57 输出和完整上游哈希"],
  ["registrar_independent_from_complete_prior_chain_confirmed", "登记人独立于验证者、执行者和完整上游角色"],
  ["immutable_artifact_revision_and_protocol_confirmed", "实现工件、代码版本和评估协议均不可变"],
  ["evaluation_rules_frozen_before_validation_label_access_confirmed", "所有统计门槛在读取 validation 标签前冻结"],
  ["all_nine_artifacts_targets_seeds_and_metrics_reported_separately_confirmed", "9 个工件、9 项目标、3 个种子及全部指标分别报告"],
  ["zero_baseline_paired_component_block_bootstrap_and_holm_correction_confirmed", "零预测配对基准、component block bootstrap 与 Holm 修正已冻结"],
  ["no_seed_shopping_hyperparameter_tuning_or_composite_masking_confirmed", "禁止挑种子、调参、改阈值或用综合分遮蔽失败目标"],
  ["validation_only_and_sealed_holdout_isolation_confirmed", "未来评估最多只触及 validation，sealed holdout 继续完全隔离"],
  ["independent_review_runner_and_one_shot_authorization_required_confirmed", "独立复核、隔离 runner 和单次授权仍是后续硬门禁"],
  ["no_label_access_selection_store_reward_shadow_order_broker_or_trading_confirmed", "本登记无标签访问、评估、选模、存储、奖励、影子、订单、券商或交易"],
] as const;

type CheckName = (typeof CHECKS)[number][0];

export function PublicAdminHistoricalOutcomeValidationEvaluationImplementationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeValidationEvaluationImplementationRegistry>();
  const [selectedValidationId, setSelectedValidationId] = createSignal("");
  const [implementationName, setImplementationName] = createSignal("");
  const [immutableCodeRevision, setImmutableCodeRevision] = createSignal("");
  const [artifactSha256, setArtifactSha256] = createSignal("");
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
      const next = await getHistoricalOutcomeValidationEvaluationImplementations();
      setRegistry(next);
      if (!next.eligible_outputs.some(
        (output) => output.validation.validation_id === selectedValidationId(),
      )) {
        setSelectedValidationId(next.eligible_outputs[0]?.validation.validation_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "validation 评估实现登记表读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() => registry()?.eligible_outputs.find(
    (output) => output.validation.validation_id === selectedValidationId(),
  ));
  const allConfirmed = createMemo(() => CHECKS.every(([name]) => checked()[name]));

  const submit = async () => {
    const output = selected();
    if (!output || busy()) return;
    if (!implementationName().trim() || !immutableCodeRevision().trim()
      || !rationale().trim() || !knownLimitations().trim()) {
      setError("请填写实现名称、不可变代码版本、登记依据和已知局限。");
      return;
    }
    if (!/^[a-f0-9]{64}$/i.test(artifactSha256().trim())) {
      setError("实现工件必须填写 64 位 SHA-256。");
      return;
    }
    if (!allConfirmed()) {
      setError("登记前必须逐项确认全部十项边界。");
      return;
    }
    const validation = output.validation;
    const request: RegisterHistoricalOutcomeValidationEvaluationImplementationRequest = {
      expected_validation_id: validation.validation_id,
      expected_validation_sha256: validation.validation_sha256,
      expected_attempt_id: validation.attempt_id,
      expected_claim_sha256: validation.claim_sha256,
      expected_result_sha256: validation.result_sha256,
      expected_output_sha256: validation.output_sha256,
      expected_suite_specification_sha256: validation.suite_specification_sha256,
      expected_training_store_dataset_sha256: validation.training_store_dataset_sha256,
      expected_rows_sha256: validation.rows_sha256,
      expected_excluded_rows_sha256: validation.excluded_rows_sha256,
      expected_target_commitments_sha256: validation.target_commitments_sha256,
      expected_candidate_set_sha256: output.candidate_set_sha256,
      implementation_name: implementationName().trim(),
      immutable_code_revision: immutableCodeRevision().trim(),
      implementation_artifact_sha256: artifactSha256().trim().toLowerCase(),
      rationale: rationale().trim(),
      known_limitations: knownLimitations().trim(),
      ...checked(),
    };
    setBusy(true);
    setError("");
    setNotice("");
    try {
      setRegistry(await registerHistoricalOutcomeValidationEvaluationImplementation(request));
      setChecked(
        Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>,
      );
      setNotice("评估实现已登记为 registered_not_reviewed_not_run；统计规则已预注册，但没有读取 validation 标签或运行选模。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "validation 评估实现登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="validation 评估实现登记">
          <header>
            <strong>第 59 阶段 · validation 评估实现登记</strong>
            <span>{currentRegistry().implementation_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可登记</span><strong>{currentRegistry().registration_eligible_count}</strong></div>
            <div><span>实现记录</span><strong>{currentRegistry().implementation_count}</strong></div>
            <div><span>当前绑定</span><strong>{currentRegistry().current_binding_implementation_count}</strong></div>
            <div><span>待独立复核</span><strong>{currentRegistry().independent_implementation_review_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>先冻结规则，再看 validation</strong><span>pre-registered · no entrypoint</span></header>
            <p>逐目标逐种子保留全部结果；零预测作配对基准；固定 10,000 次 component block bootstrap，并对 54 项候选检验作 Holm 修正。</p>
            <p class="public-admin-anchor-boundary">validation 少于 100 行或 20 个独立 component 只输出证据不足；禁止挑 seed、临时调参或用综合分掩盖失败目标。</p>
          </article>

          <Show when={currentRegistry().eligible_outputs.length > 0} fallback={<p>当前没有通过 Stage 58 且尚未登记评估实现的训练产物。</p>}>
            <label>
              <span>Stage 58 独立验证</span>
              <select value={selectedValidationId()} onChange={(event) => setSelectedValidationId(event.currentTarget.value)}>
                <For each={currentRegistry().eligible_outputs}>
                  {(output) => <option value={output.validation.validation_id}>{output.validation.validation_id.slice(0, 12)}… · {output.candidate_bindings.length} 个工件</option>}
                </For>
              </select>
            </label>
            <label><span>实现名称</span><input value={implementationName()} onInput={(event) => setImplementationName(event.currentTarget.value)} /></label>
            <label><span>不可变代码版本</span><input value={immutableCodeRevision()} onInput={(event) => setImmutableCodeRevision(event.currentTarget.value)} placeholder="commit / content-addressed revision" /></label>
            <label><span>实现工件 SHA-256</span><input value={artifactSha256()} onInput={(event) => setArtifactSha256(event.currentTarget.value)} placeholder="64 位十六进制摘要" /></label>
            <label><span>登记依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限</span><textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} /></label>
            <For each={CHECKS}>
              {([name, label]) => <label class="public-admin-anchor-check"><input type="checkbox" checked={checked()[name]} onChange={(event) => setChecked({ ...checked(), [name]: event.currentTarget.checked })} /><span>{label}</span></label>}
            </For>
            <button type="button" disabled={busy() || !selected() || !allConfirmed()} onClick={() => void submit()}>{busy() ? "正在冻结并登记协议…" : "登记评估实现（不读取标签）"}</button>
          </Show>

          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={currentRegistry().items}>
            {(item) => <article class="public-admin-reward-governance">
              <header><strong>{item.implementation.implementation_name}</strong><span>{item.implementation.status}</span></header>
              <p>登记人 {item.implementation.registered_by} · 代码 {item.implementation.implementation_contract.immutable_code_revision}</p>
              <p>工件 {item.implementation.implementation_contract.exact_artifact_count} · 目标 {item.implementation.implementation_contract.exact_target_count} · seeds {item.implementation.implementation_contract.exact_random_seeds.join("/")} · bootstrap {item.implementation.implementation_contract.bootstrap_replications}</p>
              <p class="public-admin-anchor-boundary">{item.future_independent_implementation_review_eligible ? "只可进入未来独立实现复核" : "上游绑定已失效"}；标签、评估、选模、sealed holdout、模型/指标库和交易全部关闭。</p>
            </article>}
          </For>
        </section>
      )}
    </Show>
  );
}
