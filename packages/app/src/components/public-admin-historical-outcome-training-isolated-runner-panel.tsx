import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeTrainingIsolatedRunners,
  registerHistoricalOutcomeTrainingIsolatedRunner,
} from "@/lib/api";
import type {
  HistoricalOutcomeTrainingIsolatedRunnerKind,
  HistoricalOutcomeTrainingIsolatedRunnerRegistry,
} from "@/lib/types";

const REGISTRATION_CHECKS = [
  "已核对当前 Stage 54 批准、Stage 53 实现以及 Stage 51–52 完整上游绑定",
  "登记人未参与训练数据、实验登记、实现或独立复核完整角色链",
  "runner 工件 SHA-256 与不可变代码版本已经固定",
  "未来只能挂载精确 training-store dataset 只读输入并 create-once 写候选输出",
  "train 拟合、validation 选模、sealed holdout 隐藏的挂载边界已经固定",
  "运行时身份和单实验、8192 MiB、3600 秒等资源上限已经固定",
  "没有入口、环境继承/变量、密钥、网络、工具、子进程或生产访问",
  "runner 登记、首次执行授权和输出校验严格分开",
  "当前不读数据、不训练、不创建模型/指标、不奖励、影子、订单、券商或交易",
] as const;

export function PublicAdminHistoricalOutcomeTrainingIsolatedRunnerPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeTrainingIsolatedRunnerRegistry>();
  const [selectedReviewId, setSelectedReviewId] = createSignal("");
  const [runnerName, setRunnerName] = createSignal("");
  const [runnerKind, setRunnerKind] =
    createSignal<HistoricalOutcomeTrainingIsolatedRunnerKind>(
      "ephemeral_deterministic_training_process",
    );
  const [runnerCodeRevision, setRunnerCodeRevision] = createSignal("");
  const [runnerArtifactSha256, setRunnerArtifactSha256] = createSignal("");
  const [rationale, setRationale] = createSignal("");
  const [knownLimitations, setKnownLimitations] = createSignal("");
  const [checks, setChecks] = createSignal(REGISTRATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next =
        await getHistoricalOutcomeTrainingIsolatedRunners();
      setRegistry(next);
      if (!next.eligible_reviews.some((value) => value.review.review_id === selectedReviewId())) {
        setSelectedReviewId(next.eligible_reviews[0]?.review.review_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练隔离 runner 规范登记表读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.eligible_reviews.find((value) => value.review.review_id === selectedReviewId()),
  );
  const disabled = createMemo(
    () =>
      busy() ||
      !selected() ||
      !runnerName().trim() ||
      !runnerCodeRevision().trim() ||
      !/^[a-fA-F0-9]{64}$/.test(runnerArtifactSha256().trim()) ||
      !rationale().trim() ||
      !knownLimitations().trim() ||
      checks().some((value) => !value),
  );

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) =>
      current.map((value, currentIndex) => (currentIndex === index ? checked : value)),
    );
  };

  const submit = async () => {
    const current = selected();
    if (!current || disabled()) return;
    const implementation = current.implementation;
    const review = current.review;
    const implementationContract = implementation.implementation_contract;
    const stage52 = implementation.approved_registration_review;
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next =
        await registerHistoricalOutcomeTrainingIsolatedRunner({
          expected_implementation_id: implementation.implementation_id,
          expected_implementation_sha256: implementation.implementation_sha256,
          expected_implementation_review_id: review.review_id,
          expected_implementation_review_sha256: review.review_sha256,
          expected_implementation_contract_sha256: implementationContract.contract_sha256,
          expected_implementation_artifact_sha256:
            implementationContract.implementation_artifact_sha256,
          expected_immutable_code_revision: implementationContract.immutable_code_revision,
          expected_stage_52_review_sha256: stage52.review_sha256,
          expected_stage_51_registration_sha256: stage52.registration_sha256,
          expected_stage_51_claim_sha256: stage52.claim_sha256,
          expected_stage_51_result_sha256: stage52.result_sha256,
          expected_suite_specification_sha256: stage52.suite_specification_sha256,
          expected_training_store_dataset_sha256: stage52.training_store_dataset_sha256,
          expected_rows_sha256: stage52.rows_sha256,
          expected_excluded_rows_sha256: stage52.excluded_rows_sha256,
          expected_target_commitments_sha256: stage52.target_commitments_sha256,
          expected_review_contract_sha256: review.review_contract.contract_sha256,
          expected_independent_audit_sha256: review.independent_audit.audit_sha256,
          runner_name: runnerName().trim(),
          runner_kind: runnerKind(),
          runner_code_revision: runnerCodeRevision().trim(),
          runner_artifact_sha256: runnerArtifactSha256().trim().toLowerCase(),
          rationale: rationale().trim(),
          known_limitations: knownLimitations().trim(),
          exact_current_approved_review_and_complete_upstream_binding_confirmed: confirmed[0],
          registrar_independence_confirmed: confirmed[1],
          runner_artifact_and_code_revision_immutable_confirmed: confirmed[2],
          exact_read_only_training_input_and_content_addressed_create_once_output_confirmed:
            confirmed[3],
          train_validation_and_sealed_holdout_mount_isolation_confirmed: confirmed[4],
          fixed_runtime_identity_and_bounded_resource_contract_confirmed: confirmed[5],
          no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed:
            confirmed[6],
          registration_first_execution_and_output_validation_separation_confirmed: confirmed[7],
          no_data_read_training_model_metrics_reward_shadow_order_broker_or_trading_confirmed:
            confirmed[8],
        });
      setRegistry(next);
      setRunnerName("");
      setRunnerCodeRevision("");
      setRunnerArtifactSha256("");
      setRationale("");
      setKnownLimitations("");
      setChecks(REGISTRATION_CHECKS.map(() => false));
      setNotice(
        "runner 规范已 create-once 登记为 registered_not_run；没有调用入口。唯一下一门禁是独立首次执行授权复核。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "训练隔离 runner 规范登记失败");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section
          class="public-admin-reward-governance"
          aria-label="训练隔离 runner 规范登记"
        >
          <header>
            <strong>第 55 阶段 · 训练隔离 runner 规范登记</strong>
            <span>{currentRegistry().runner_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可登记批准</span><strong>{currentRegistry().eligible_reviews.length}</strong></div>
            <div><span>历史 runner</span><strong>{currentRegistry().runner_count}</strong></div>
            <div><span>当前绑定</span><strong>{currentRegistry().current_binding_runner_count}</strong></div>
            <div><span>可送首次执行复核</span><strong>{currentRegistry().first_execution_authorization_review_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>固定零能力运行合同</strong><span>无入口 · 不执行</span></header>
            <p>runner 登记只冻结工件、代码、运行时、未来精确只读训练输入、切分隔离、create-once 候选输出与资源上限。它不会挂载数据、拟合或选择模型、创建模型或指标。</p>
            <p class="public-admin-anchor-boundary">唯一下一门禁：独立首次执行授权复核。登记本身绝不等于执行授权。</p>
          </article>

          <Show when={currentRegistry().registration_allowed}>
            <label>
              <span>当前已批准实现复核</span>
              <select value={selectedReviewId()} onChange={(event) => setSelectedReviewId(event.currentTarget.value)}>
                <option value="">请选择当前批准</option>
                <For each={currentRegistry().eligible_reviews}>
                  {(value) => (
                    <option value={value.review.review_id}>
                      {value.implementation.implementation_name} · {value.review.review_id.slice(0, 12)}…
                    </option>
                  )}
                </For>
              </select>
            </label>
            <label><span>runner 名称</span><input value={runnerName()} onInput={(event) => setRunnerName(event.currentTarget.value)} placeholder="例如：九目标三模型臂确定性训练 runner" /></label>
            <label>
              <span>runner 类型</span>
              <select value={runnerKind()} onChange={(event) => setRunnerKind(event.currentTarget.value as HistoricalOutcomeTrainingIsolatedRunnerKind)}>
                <option value="ephemeral_deterministic_training_process">一次性确定性非特权训练进程</option>
              </select>
            </label>
            <label><span>不可变 runner 代码版本</span><input value={runnerCodeRevision()} onInput={(event) => setRunnerCodeRevision(event.currentTarget.value)} placeholder="例如 oldwang@commit-sha" /></label>
            <label><span>runner 工件 SHA-256</span><input value={runnerArtifactSha256()} onInput={(event) => setRunnerArtifactSha256(event.currentTarget.value)} placeholder="64 位十六进制摘要" /></label>
            <label><span>登记理由</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} placeholder="说明为什么这一工件与固定沙箱适合进入下一轮独立首次执行复核" /></label>
            <label><span>已知局限</span><textarea value={knownLimitations()} onInput={(event) => setKnownLimitations(event.currentTarget.value)} placeholder="登记尚未执行；说明构建、运行时或输出验证局限" /></label>
            <div class="public-admin-decision-checks">
              <For each={REGISTRATION_CHECKS}>
                {(label, index) => (
                  <label>
                    <input type="checkbox" checked={checks()[index()]} onChange={(event) => toggleCheck(index(), event.currentTarget.checked)} />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
              登记 runner 规范（无入口、不执行）
            </button>
          </Show>

          <For each={currentRegistry().items}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.runner.runner_name}</strong>
                  <span>{item.approved_review_binding_current ? "当前绑定 · 可送首次执行复核" : "上游绑定失效"}</span>
                </header>
                <p>runner {item.runner.isolated_runner_id} · {item.runner.status} · 登记人 {item.runner.registered_by}</p>
                <p>代码 {item.runner.runner_code_revision} · 工件 {item.runner.runner_artifact_sha256}</p>
                <p>{item.runner.rationale}</p>
                <p>局限：{item.runner.known_limitations}</p>
                <p class="public-admin-anchor-boundary">无调用入口；数据读取、首次执行、训练、validation 选择、sealed holdout、模型、指标、输出校验、奖励、影子、订单、券商和交易全部关闭。</p>
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
