import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeTrainingImplementations,
  registerHistoricalOutcomeTrainingImplementation,
} from "@/lib/api";
import type {
  HistoricalOutcomeTrainingImplementationRegistry,
  RegisterHistoricalOutcomeTrainingImplementationRequest,
} from "@/lib/types";

const CHECKS = [
  ["exact_stage_52_review_and_stage_51_registration_binding_confirmed", "精确绑定 Stage 52 独立批准、Stage 51 登记及完整上游"],
  ["registrar_independent_from_complete_prior_chain_confirmed", "登记人独立于登记复核者、实验登记者及完整上游角色"],
  ["immutable_artifact_and_code_revision_confirmed", "实现工件 SHA-256 与不可变代码版本均已冻结"],
  ["fixed_three_arm_three_seed_implementation_confirmed", "固定零预测、岭回归、梯度提升三臂与 17/29/43 三种子"],
  ["exact_65_feature_nine_raw_continuous_target_contract_confirmed", "固定 65 项特征与九项原始连续目标"],
  ["train_only_preprocessing_and_fit_confirmed", "预处理与拟合参数只能由训练集产生"],
  ["validation_selection_and_sealed_holdout_isolation_confirmed", "验证集只用于选择，封存集对拟合与选择完全不可见"],
  ["per_target_per_seed_metrics_without_composite_masking_confirmed", "逐目标逐种子报告，综合结果不能掩盖单目标失败"],
  ["deterministic_replay_and_fixed_resource_ceilings_confirmed", "确定性重放与固定资源上限已冻结"],
  ["no_scalar_reward_action_position_or_ranking_semantics_confirmed", "没有标量奖励、买卖动作、仓位或排名语义"],
  ["implementation_review_runner_and_run_authorization_separation_confirmed", "实现复核、runner 登记和运行授权保持独立门禁"],
  ["no_data_access_training_reward_shadow_order_broker_or_trading_confirmed", "本登记不访问数据、不训练、不奖励、不影子、不下单、不接券商、不交易"],
] as const;

type CheckName = (typeof CHECKS)[number][0];

export function PublicAdminHistoricalOutcomeTrainingImplementationPanel() {
  const [registry, setRegistry] = createSignal<HistoricalOutcomeTrainingImplementationRegistry>();
  const [selectedReviewId, setSelectedReviewId] = createSignal("");
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
      const next = await getHistoricalOutcomeTrainingImplementations();
      setRegistry(next);
      if (!next.eligible_reviews.some((review) => review.review_id === selectedReviewId())) {
        setSelectedReviewId(next.eligible_reviews[0]?.review_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练实现登记表读取失败");
    }
  };

  onMount(() => void load());
  const selected = createMemo(() => registry()?.eligible_reviews.find(
    (review) => review.review_id === selectedReviewId(),
  ));
  const allConfirmed = createMemo(() => CHECKS.every(([name]) => checked()[name]));

  const submit = async () => {
    const review = selected();
    if (!review || busy()) return;
    if (!implementationName().trim() || !immutableCodeRevision().trim() || !rationale().trim() || !knownLimitations().trim()) {
      setError("请填写实现名称、不可变代码版本、登记依据和已知局限。");
      return;
    }
    if (!/^[a-f0-9]{64}$/i.test(artifactSha256().trim())) {
      setError("实现工件必须填写 64 位 SHA-256。");
      return;
    }
    if (!allConfirmed()) {
      setError("登记前必须逐项确认全部十二项边界。");
      return;
    }
    const request: RegisterHistoricalOutcomeTrainingImplementationRequest = {
      expected_review_id: review.review_id,
      expected_review_sha256: review.review_sha256,
      expected_attempt_id: review.attempt_id,
      expected_registration_id: review.registration_id,
      expected_registration_sha256: review.registration_sha256,
      expected_claim_sha256: review.claim_sha256,
      expected_result_id: review.result_id,
      expected_result_sha256: review.result_sha256,
      expected_suite_specification_sha256: review.suite_specification_sha256,
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
      setRegistry(await registerHistoricalOutcomeTrainingImplementation(request));
      setChecked(Object.fromEntries(CHECKS.map(([name]) => [name, false])) as Record<CheckName, boolean>);
      setNotice("训练实现已登记为 registered_not_reviewed_not_run；只开放未来独立实现复核，没有运行或训练。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练实现登记失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="训练实现登记">
          <header><strong>第 53 阶段 · 训练实现登记</strong><span>{currentRegistry().implementation_status}</span></header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可登记</span><strong>{currentRegistry().registration_eligible_count}</strong></div>
            <div><span>实现记录</span><strong>{currentRegistry().implementation_count}</strong></div>
            <div><span>当前绑定</span><strong>{currentRegistry().current_binding_implementation_count}</strong></div>
            <div><span>待独立实现复核</span><strong>{currentRegistry().independent_implementation_review_eligible_count}</strong></div>
          </div>
          <article class="public-admin-reward-governance">
            <header><strong>实现登记 ≠ 训练运行</strong><span>immutable · no entrypoint</span></header>
            <p>这里只冻结工件哈希、代码版本、三模型臂、三种子、65 项特征、九项目标、指标和资源边界。</p>
            <p class="public-admin-anchor-boundary">合同没有可调用入口、环境变量、密钥、网络、工具、子进程或数据读取能力；下一步只能独立复核实现。</p>
          </article>
          <Show when={currentRegistry().eligible_reviews.length > 0} fallback={<p>当前没有通过 Stage 52 且尚未登记实现的记录。</p>}>
            <label><span>Stage 52 独立批准</span><select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
              <For each={currentRegistry().eligible_reviews}>{(review) => <option value={review.review_id}>{review.registration_id.slice(0, 12)} · {review.reviewer_id}</option>}</For>
            </select></label>
            <label><span>实现名称</span><input value={implementationName()} onInput={(event) => setImplementationName(event.currentTarget.value)} /></label>
            <label><span>不可变代码版本</span><input value={immutableCodeRevision()} onInput={(event) => setImmutableCodeRevision(event.currentTarget.value)} placeholder="commit / content-addressed revision" /></label>
            <label><span>实现工件 SHA-256</span><input value={artifactSha256()} onInput={(event) => setArtifactSha256(event.currentTarget.value)} placeholder="64 位十六进制摘要" /></label>
            <label><span>登记依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
            <label><span>已知局限</span><textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} /></label>
            <For each={CHECKS}>{([name, label]) => <label class="public-admin-anchor-check"><input type="checkbox" checked={checked()[name]} onChange={(event) => setChecked({ ...checked(), [name]: event.currentTarget.checked })} /><span>{label}</span></label>}</For>
            <button type="button" disabled={busy() || !selected() || !allConfirmed()} onClick={() => void submit()}>{busy() ? "正在登记不可变实现…" : "登记实现（不运行）"}</button>
          </Show>
          <Show when={notice()}><p class="public-admin-decision-notice">{notice()}</p></Show>
          <Show when={error()}><p class="public-admin-decision-error">{error()}</p></Show>
          <For each={currentRegistry().items}>{(item) => <article class="public-admin-reward-governance">
            <header><strong>{item.implementation.implementation_name}</strong><span>{item.implementation.status}</span></header>
            <p>登记人 {item.implementation.registered_by} · 代码 {item.implementation.implementation_contract.immutable_code_revision}</p>
            <p>模型臂 {item.implementation.implementation_contract.algorithm_implementation_versions.length} · seeds {item.implementation.implementation_contract.exact_random_seeds.join("/")} · features {item.implementation.implementation_contract.exact_feature_count} · targets {item.implementation.implementation_contract.exact_target_count}</p>
            <p class="public-admin-anchor-boundary">{item.future_independent_implementation_review_eligible ? "只可进入未来独立实现复核" : "上游绑定已失效"}；runner、数据访问、训练、奖励、影子、订单、券商和交易全部关闭。</p>
          </article>}</For>
        </section>
      )}
    </Show>
  );
}
