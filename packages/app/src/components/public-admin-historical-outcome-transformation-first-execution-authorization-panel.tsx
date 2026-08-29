import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizations,
  reviewHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization,
} from "@/lib/api";
import type {
  HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationRegistry,
  HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationVerdict,
} from "@/lib/types";

const AUTHORIZATION_CHECKS = [
  "已核对精确 runner 规范与数据集、治理、转换规范、实现及独立复核完整绑定",
  "复核者不属于 runner 登记人或此前任何治理、实现与复核角色",
  "已经独立重算 runner 工件 SHA-256，结果与登记摘要完全一致",
  "不可变代码版本可复现，绑定工件仍可用且未被替换",
  "封存输入与根文件系统只读，不允许读取未绑定资料",
  "进程必须非特权运行并启用 no-new-privileges",
  "输出只能写入一次性隔离目录，内容寻址、create-once 且必须另行独立校验",
  "运行时身份与单 subject、2048 MiB、300 秒、单进程等资源上限固定",
  "不继承宿主环境，不注入环境变量或密钥",
  "无网络、工具、子进程、生产读写或历史状态修改能力",
  "确定性切分、65 项特征允许列表与 canonical schema/序列化合同未变化",
  "授权只在 24 小时内有效，最多消费一次",
  "授权、claim、执行、输出校验、目标与训练职责严格分离",
  "不授权 manifest/bundle/join/语义目标/训练/奖励/影子/订单/券商或交易",
] as const;

export function PublicAdminHistoricalOutcomeTransformationFirstExecutionAuthorizationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationRegistry>();
  const [selectedRunnerId, setSelectedRunnerId] = createSignal("");
  const [verdict, setVerdict] =
    createSignal<HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationVerdict>(
      "changes_requested",
    );
  const [rationale, setRationale] = createSignal("");
  const [checks, setChecks] = createSignal(AUTHORIZATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next =
        await getHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizations();
      setRegistry(next);
      if (!next.items.some((item) => item.runner.isolated_runner_id === selectedRunnerId())) {
        setSelectedRunnerId(next.items[0]?.runner.isolated_runner_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "隔离转换首次执行授权复核读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    registry()?.items.find((item) => item.runner.isolated_runner_id === selectedRunnerId()),
  );
  const approvalSelected = createMemo(
    () => verdict() === "approved_for_one_future_isolated_transformation_invocation",
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
    const specification = implementation.approved_review.specification;
    const latest = current.latest_review;
    const confirmed = checks();
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next =
        await reviewHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization(
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
            expected_implementation_review_id: runner.implementation_review.review_id,
            expected_implementation_review_sha256: runner.implementation_review.review_sha256,
            expected_transformation_spec_sha256: specification.transformation_spec_sha256,
            expected_dataset_content_sha256: specification.subject.dataset_content_sha256,
            verdict: verdict(),
            rationale: rationale().trim(),
            exact_runner_and_complete_upstream_binding_confirmed: confirmed[0],
            reviewer_independence_from_complete_prior_chain_confirmed: confirmed[1],
            runner_artifact_digest_independently_reproduced: confirmed[2],
            immutable_code_revision_reproducible_and_artifact_available_confirmed: confirmed[3],
            sealed_read_only_inputs_and_root_filesystem_confirmed: confirmed[4],
            unprivileged_and_no_new_privileges_confirmed: confirmed[5],
            ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed:
              confirmed[6],
            fixed_runtime_and_resource_limits_confirmed: confirmed[7],
            no_host_environment_variables_or_secrets_confirmed: confirmed[8],
            no_network_tools_child_process_production_or_history_access_confirmed: confirmed[9],
            deterministic_split_feature_and_canonical_schema_contract_confirmed: confirmed[10],
            authorization_single_use_and_24_hour_expiry_confirmed: confirmed[11],
            authorization_execution_output_validation_and_training_separation_confirmed:
              confirmed[12],
            no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed:
              confirmed[13],
          },
        );
      setRegistry(next);
      setRationale("");
      setChecks(AUTHORIZATION_CHECKS.map(() => false));
      setNotice(
        verdict() === "approved_for_one_future_isolated_transformation_invocation"
          ? "已追加独立批准：仅在 24 小时内允许未来最多一次隔离调用；尚未 claim、未执行、未生成输出，请转到第 31 阶段人工领取。"
          : "已追加复核记录；没有授予执行资格。",
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "隔离转换首次执行授权复核失败");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section
          class="public-admin-reward-governance"
          aria-label="隔离转换首次执行授权复核"
        >
          <header>
            <strong>第 30 阶段 · 隔离转换首次执行授权复核</strong>
            <span>{currentRegistry().authorization_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>可复核 runner</span><strong>{currentRegistry().review_eligible_runner_count}</strong></div>
            <div><span>已复核</span><strong>{currentRegistry().reviewed_runner_count}</strong></div>
            <div><span>有效单次授权</span><strong>{currentRegistry().unexpired_authorization_count}</strong></div>
            <div><span>可进入下一门禁</span><strong>{currentRegistry().execution_attempt_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>授权不是调用</strong><span>24 小时 · 最多一次</span></header>
            <p>批准只建立一个短期、单次的未来调用资格。本页没有 claim 或执行按钮，不启动进程、不创建输出。</p>
            <p class="public-admin-anchor-boundary">下一门禁是第 31 阶段的单次隔离执行尝试；输出仍须独立校验，不能直接成为 manifest、特征、目标或训练输入。</p>
          </article>

          <Show when={currentRegistry().items.length > 0}>
            <label>
              <span>当前 registered_not_run runner</span>
              <select value={selectedRunnerId()} onChange={(event) => setSelectedRunnerId(event.currentTarget.value)}>
                <For each={currentRegistry().items}>
                  {(item) => (
                    <option value={item.runner.isolated_runner_id}>
                      {item.runner.runner_name} · {item.runner.isolated_runner_id.slice(0, 12)}…
                    </option>
                  )}
                </For>
              </select>
            </label>
            <label>
              <span>复核结论</span>
              <select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as HistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorizationVerdict)}>
                <option value="changes_requested">要求修改</option>
                <option value="rejected">拒绝</option>
                <option value="approved_for_one_future_isolated_transformation_invocation">批准未来单次隔离调用</option>
              </select>
            </label>
            <label><span>复核依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} placeholder="记录工件独立复现、代码可用性、沙箱边界与局限" /></label>
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
              追加首次执行授权复核（不调用、不执行）
            </button>
          </Show>

          <For each={currentRegistry().items}>
            {(item) => (
              <article class="public-admin-reward-governance">
                <header>
                  <strong>{item.runner.runner_name}</strong>
                  <span>{item.authorization_unexpired ? "24 小时单次资格有效 · 未执行" : item.latest_review ? "已复核未授权" : "等待独立复核"}</span>
                </header>
                <p>runner {item.runner.isolated_runner_id} · 工件 {item.runner.runner_artifact_sha256}</p>
                <Show when={item.latest_review}>{(review) => (
                  <>
                    <p>review {review().review_id} · 复核人 {review().reviewer_id} · 截止 {review().authorization_valid_until}</p>
                    <p>{review().rationale}</p>
                  </>
                )}</Show>
                <p class="public-admin-anchor-boundary">本授权页没有 claim/执行按钮；只能在第 31 阶段领取一次，输出校验、正式 manifest/bundle、join、目标、训练、奖励、影子、订单、券商和交易仍全部关闭。</p>
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
