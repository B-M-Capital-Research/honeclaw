import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizations,
  reviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization,
} from "@/lib/api";
import type {
  HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRegistry,
  HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict,
} from "@/lib/types";

const AUTHORIZATION_CHECKS = [
  "已逐项核对精确 Stage 69 runner、Stage 68/67/66 与完整 Stage 51–65、单目标三种子候选工件与完整哈希链",
  "复核者不属于评估实现登记/复核、runner 登记或完整上游的任何历史角色",
  "已经独立重算 runner 工件 SHA-256，结果与登记摘要完全一致",
  "不可变代码版本可复现，绑定实现与 runner 工件仍可用且未被替换",
  "未来只能只读挂载精确一个目标的 sealed-holdout features/labels 和 17/29/43 三个候选；当前不挂载任何输入",
  "进程必须非特权运行并启用 no-new-privileges",
  "输出只允许写入一次性隔离目录，内容寻址、create-once 且必须另行独立校验",
  "运行时身份与单实验、8192 MiB、3600 秒、4000 millicores、4 进程和 256 MiB 输出上限固定",
  "不继承宿主环境，不注入环境变量或密钥",
  "无网络、工具、子进程、生产读写或历史状态修改能力",
  "固定单算法、17/29/43、65 项特征、一个目标、指标门槛、component bootstrap/Holm 与样本门禁未变化",
  "sealed holdout 只允许未来单次确认性评估，禁止训练、调参、重选、跨目标读取或反馈复用",
  "未来只读挂载必须精确绑定一个目标的 sealed-holdout 分区和三候选，禁止读取其它训练、标签或生产数据",
  "授权只在 24 小时内有效，最多消费一次",
  "授权、claim、执行、输出独立校验与候选选择职责严格分离",
  "本复核不读取数据、不评估、不选模、不写模型/指标库，也不授权奖励、影子、订单、券商或交易",
] as const;

export function PublicAdminHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRegistry>();
  const [selectedRunnerId, setSelectedRunnerId] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict>(
      "changes_requested",
    );
  const [rationale, setRationale] = createSignal("");
  const [checks, setChecks] = createSignal(AUTHORIZATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizations();
      setRegistry(next);
      if (!next.items.some((item) => item.runner.isolated_runner_id === selectedRunnerId())) {
        setSelectedRunnerId(next.items[0]?.runner.isolated_runner_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 评估首次执行授权复核读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find((item) => item.runner.isolated_runner_id === selectedRunnerId()),
  );
  const approvalSelected = createMemo(
    () => verdict() === "approved_for_one_future_isolated_sealed_holdout_evaluation_invocation",
  );
  const disabled = createMemo(
    () =>
      busy() ||
      !selected() ||
      !rationale().trim() ||
      !checks()[1] ||
      (approvalSelected() && checks().some((value) => !value)),
  );

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) =>
      current.map((value, currentIndex) => (currentIndex === index ? checked : value)),
    );
  };

  const submit = async () => {
    const current = selected();
    if (!current || disabled()) return;
    const runner = current.runner;
    const implementation = runner.implementation;
    const implementationContract = implementation.implementation_contract;
    const implementationReview = runner.implementation_review;
    const latest = current.latest_review;
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next =
        await reviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization(
          runner.isolated_runner_id,
          {
            expected_review_id: latest?.review_id,
            expected_review_sha256: latest?.review_sha256,
            expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256,
            expected_runner_artifact_sha256: runner.runner_artifact_sha256,
            expected_runner_code_revision: runner.runner_code_revision,
            expected_runner_contract_sha256: runner.runner_contract.contract_sha256,
            expected_implementation_id: implementation.implementation_id,
            expected_implementation_sha256: implementation.implementation_sha256,
            expected_implementation_contract_sha256: implementationContract.contract_sha256,
            expected_implementation_artifact_sha256:
              implementationContract.implementation_artifact_sha256,
            expected_immutable_code_revision: implementationContract.immutable_code_revision,
            expected_implementation_review_id: implementationReview.review_id,
            expected_implementation_review_sha256: implementationReview.review_sha256,
            expected_implementation_independent_audit_sha256:
              implementationReview.independent_audit.audit_sha256,
            expected_candidate_set_sha256: implementationContract.candidate_set_sha256,
            expected_stage_66_protocol_review_sha256:
              implementationContract.stage_66_protocol_review_sha256,
            expected_sealed_holdout_evaluation_protocol_sha256:
              implementationContract.sealed_holdout_evaluation_protocol_sha256,
            expected_target_bundle_sha256: implementationContract.target_bundle_sha256,
            expected_recommendation_sha256: implementationContract.recommendation_sha256,
            expected_selected_algorithm_three_seed_binding_sha256:
              implementationContract.selected_algorithm_three_seed_binding_sha256,
            expected_sealed_holdout_split_commitment_sha256:
              implementationContract.sealed_holdout_split_commitment_sha256,
            expected_feature_order_sha256: implementationContract.feature_order_sha256,
            expected_preprocessing_sha256: implementationContract.preprocessing_sha256,
            expected_target_id: implementationContract.target_id,
            expected_frozen_candidate_algorithm_id:
              implementationContract.frozen_candidate_algorithm_id,
            verdict: verdict(),
            rationale: rationale().trim(),
            exact_runner_and_complete_upstream_binding_confirmed: confirmed[0],
            reviewer_independence_from_complete_prior_chain_confirmed: confirmed[1],
            runner_artifact_digest_independently_reproduced: confirmed[2],
            immutable_code_revision_reproducible_and_artifact_available_confirmed: confirmed[3],
            future_exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_confirmed: confirmed[4],
            unprivileged_and_no_new_privileges_confirmed: confirmed[5],
            ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed:
              confirmed[6],
            fixed_runtime_and_resource_limits_confirmed: confirmed[7],
            no_host_environment_variables_or_secrets_confirmed: confirmed[8],
            no_network_tools_child_process_production_or_history_access_confirmed: confirmed[9],
            fixed_one_algorithm_three_seed_sixty_five_feature_one_target_metrics_bootstrap_holm_and_sample_gates_confirmed:
              confirmed[10],
            one_shot_sealed_holdout_only_no_training_tuning_reselection_or_feedback_confirmed:
              confirmed[11],
            exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_and_no_other_data_access_confirmed:
              confirmed[12],
            authorization_single_use_and_24_hour_expiry_confirmed: confirmed[13],
            authorization_execution_output_validation_and_selection_separation_confirmed:
              confirmed[14],
            no_data_read_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed:
              confirmed[15],
          },
        );
      setRegistry(next);
      setRationale("");
      setChecks(AUTHORIZATION_CHECKS.map(() => false));
      setNotice(
        approvalSelected()
          ? "已追加独立批准：24 小时内最多允许一次未来隔离 sealed-holdout 访问与评估调用；尚未 claim、未读取特征/标签、未评估或选模。"
          : "已追加复核记录；没有授予未来调用资格。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "sealed-holdout 评估首次执行授权复核失败");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="sealed-holdout 评估首次执行授权复核">
          <header>
            <strong>第 70 阶段 · sealed-holdout 评估首次执行授权复核</strong>
            <span>{currentRegistry().authorization_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可复核 runner</span><strong>{currentRegistry().review_eligible_runner_count}</strong></div>
            <div><span>已复核</span><strong>{currentRegistry().reviewed_runner_count}</strong></div>
            <div><span>有效单次资格</span><strong>{currentRegistry().unexpired_authorization_count}</strong></div>
            <div><span>下一门禁候选</span><strong>{currentRegistry().execution_attempt_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>授权不等于执行</strong><span>24 小时 · 最多一次</span></header>
            <p>批准只建立短期的一次性未来资格。本页没有 claim 或执行按钮，不启动进程、不挂载 sealed-holdout 标签，也不评估或选择候选。</p>
            <p class="public-admin-anchor-boundary">下一门禁只能是另行开发和复核的第 71 阶段 claim-first 单次隔离评估尝试；sealed holdout、模型/指标库、奖励、影子组合、订单、券商和交易全部关闭。</p>
          </article>

          <Show when={currentRegistry().items.length > 0}>
            <label>
              <span>当前 registered_not_run runner</span>
              <select value={selectedRunnerId()} onChange={(event) => setSelectedRunnerId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>
                  {(item) => <option value={item.runner.isolated_runner_id}>{item.runner.runner_name} · {item.runner.isolated_runner_id.slice(0, 12)}…</option>}
                </For>
              </select>
            </label>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict)}>
                <option value="changes_requested">要求修改</option>
                <option value="rejected">拒绝</option>
                <option value="approved_for_one_future_isolated_sealed_holdout_evaluation_invocation">批准未来单次隔离 sealed-holdout 访问与评估调用</option>
              </select>
            </label>
            <label><span>复核依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} placeholder="记录独立重算、完整 Stage 51–69、单目标三种子候选、精确 sealed-holdout 只读挂载与单次边界" /></label>
            <div class="public-admin-decision-checks">
              <For each={AUTHORIZATION_CHECKS}>
                {(label, index) => (
                  <label>
                    <input type="checkbox" checked={checks()[index()]} onChange={(event) => toggleCheck(index(), event.currentTarget.checked)} />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
              追加首次执行授权复核（不 claim、不执行）
            </button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
          <For each={currentRegistry().items}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.runner.runner_name}</strong>
                  <span>{item.authorization_unexpired ? "单次资格有效 · 未执行" : item.latest_review ? "已复核未授权" : "等待独立复核"}</span>
                </header>
                <p>runner {item.runner.isolated_runner_id} · 工件 {item.runner.runner_artifact_sha256}</p>
                <Show when={item.latest_review}>{(review) => <p>review {review().review_id} · 复核人 {review().reviewer_id} · 截止 {review().authorization_valid_until} · {review().rationale}</p>}</Show>
                <p class="public-admin-anchor-boundary">本页不能 claim 或执行；sealed-holdout 标签读取、评估、选模、sealed holdout、模型/指标库、输出校验、奖励、影子、订单、券商和交易仍全部关闭。</p>
              </article>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
