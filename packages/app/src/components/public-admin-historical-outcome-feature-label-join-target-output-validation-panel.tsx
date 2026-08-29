import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getHistoricalOutcomeFeatureLabelJoinTargetOutputValidations,
  validateHistoricalOutcomeFeatureLabelJoinTargetOutput,
} from "@/lib/api";
import type { HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRegistry } from "@/lib/types";

const VALIDATION_CHECKS = [
  "确认校验器独立重算一对一连接、65 项特征和九项目标，不复用第 42 阶段投影或信封校验算法",
  "确认 validation 与 sealed holdout 目标值继续隐藏，仅重算并核对承诺",
  "确认通过后仍是不可信候选，只进入未来准入复核",
  "确认不创建正式 joined dataset，不复制训练库，不训练、奖励、影子、订单、券商或交易",
] as const;

export function PublicAdminHistoricalOutcomeFeatureLabelJoinTargetOutputValidationPanel() {
  const [registry, setRegistry] =
    createSignal<HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [checks, setChecks] = createSignal(VALIDATION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const eligibleItems = createMemo(() =>
    registry()?.items.filter((item) => item.validation_eligible) ?? [],
  );

  const load = async () => {
    try {
      const next = await getHistoricalOutcomeFeatureLabelJoinTargetOutputValidations();
      setRegistry(next);
      if (!next.items.some((item) => item.validation_eligible && item.attempt.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(next.items.find((item) => item.validation_eligible)?.attempt.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 独立输出校验读取失败");
    }
  };

  onMount(() => void load());

  const selected = createMemo(() =>
    eligibleItems().find((item) => item.attempt.claim.attempt_id === selectedAttemptId()),
  );
  const disabled = createMemo(
    () => busy() || !selected() || checks().some((confirmed) => !confirmed),
  );

  const toggleCheck = (index: number, checked: boolean) => {
    setChecks((current) =>
      current.map((value, currentIndex) => (currentIndex === index ? checked : value)),
    );
  };

  const validate = async () => {
    const item = selected();
    const envelope = item?.attempt.result.untrusted_candidate_envelope;
    const outputSha256 = item?.attempt.result.output_sha256;
    if (!item || !envelope || !outputSha256 || disabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const next = await validateHistoricalOutcomeFeatureLabelJoinTargetOutput(
        item.attempt.claim.attempt_id,
        {
          expected_claim_sha256: item.attempt.claim.claim_sha256,
          expected_result_sha256: item.attempt.result.result_sha256,
          expected_output_sha256: outputSha256,
          expected_authorization_review_sha256: envelope.authorization_review_sha256,
          expected_split_manifest_sha256: envelope.split_manifest_sha256,
          expected_feature_bundle_sha256: envelope.feature_bundle_sha256,
          expected_combined_artifact_sha256: envelope.combined_artifact_sha256,
          expected_dataset_content_sha256: envelope.dataset_content_sha256,
          expected_dataset_manifest_sha256: envelope.dataset_manifest_sha256,
          expected_candidate_set_sha256: envelope.candidate_set_sha256,
          independent_recomputation_confirmed: true,
          validation_and_sealed_holdout_targets_remain_withheld_confirmed: true,
          output_remains_untrusted_pending_admission_confirmed: true,
          no_training_reward_shadow_order_broker_or_trading_confirmed: true,
        },
      );
      setRegistry(next);
      setChecks(VALIDATION_CHECKS.map(() => false));
      setSelectedAttemptId(next.items.find((candidate) => candidate.validation_eligible)?.attempt.claim.attempt_id ?? "");
      setNotice("独立重算记录已不可变保存；通过也只开放下一阶段候选准入复核资格。");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "join/target 独立输出校验失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return (
    <Show when={registry()}>
      {(currentRegistry) => (
        <section class="public-admin-reward-governance" aria-label="join/target 独立输出校验">
          <header>
            <strong>第 43 阶段 · join/target 独立输出校验</strong>
            <span>{currentRegistry().validation_status}</span>
          </header>
          <p>{currentRegistry().scope}</p>
          <div class="public-admin-decision-metrics">
            <div><span>待独立校验</span><strong>{currentRegistry().validation_eligible_count}</strong></div>
            <div><span>校验记录</span><strong>{currentRegistry().validation_count}</strong></div>
            <div><span>独立通过</span><strong>{currentRegistry().independently_validated_untrusted_candidate_count}</strong></div>
            <div><span>待准入复核</span><strong>{currentRegistry().future_candidate_admission_review_eligible_count}</strong></div>
          </div>

          <article class="public-admin-reward-governance">
            <header><strong>独立性与隐藏边界</strong><span>逐行 · 逐位 · create-once</span></header>
            <p>
              校验器从当前原始结果数据集和正式 split/feature 工件重建每一行；train
              只核对九项原始位模式，validation 与 sealed holdout 只核对承诺，不打开目标值。
            </p>
            <p class="public-admin-anchor-boundary">
              校验通过不等于可训练、可奖励或可交易；下一步仍需独立候选准入复核。
            </p>
          </article>

          <Show when={eligibleItems().length > 0} fallback={<p class="public-admin-anchor-boundary">当前没有待独立校验的不可信候选。</p>}>
            <label>
              <span>待校验 attempt</span>
              <select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
                <For each={eligibleItems()}>
                  {(item) => <option value={item.attempt.claim.attempt_id}>{item.attempt.claim.attempt_id} · {item.attempt.result.completed_at}</option>}
                </For>
              </select>
            </label>
            <div class="public-admin-decision-checks">
              <For each={VALIDATION_CHECKS}>
                {(label, index) => (
                  <label>
                    <input type="checkbox" checked={checks()[index()]} onChange={(event) => toggleCheck(index(), event.currentTarget.checked)} />
                    <span>{label}</span>
                  </label>
                )}
              </For>
            </div>
            <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void validate()}>
              独立重算并保存校验记录
            </button>
          </Show>

          <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
          <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>

          <For each={currentRegistry().items}>
            {(item) => (
              <Show when={item.validation}>
                {(validation) => (
                  <article class="public-admin-reward-governance">
                    <header><strong>validation {validation().validation_id}</strong><span>{validation().verdict}</span></header>
                    <p>attempt {validation().attempt_id} · {validation().validated_at} · 校验人 {validation().validated_by}</p>
                    <div class="public-admin-decision-metrics">
                      <div><span>一对一连接</span><strong>{validation().exact_one_to_one_entry_join_recomputed ? "通过" : "失败"}</strong></div>
                      <div><span>65 项特征</span><strong>{validation().exact_65_feature_catalog_recomputed ? "通过" : "失败"}</strong></div>
                      <div><span>九维位模式</span><strong>{validation().exact_nine_raw_f64_target_bits_recomputed ? "通过" : "失败"}</strong></div>
                      <div><span>目标隐藏</span><strong>{validation().validation_targets_withheld_verified && validation().sealed_holdout_targets_withheld_verified ? "通过" : "失败"}</strong></div>
                    </div>
                    <Show when={validation().mismatch_reasons.length > 0}>
                      <p class="public-admin-error">差异：{validation().mismatch_reasons.join("；")}</p>
                    </Show>
                    <p class="public-admin-anchor-boundary">
                      准入复核资格：{validation().future_candidate_admission_review_eligible ? "已开放" : "未开放"}；训练：{validation().training_authorized ? "异常开启" : "关闭"}；交易：{validation().trading_authorized ? "异常开启" : "关闭"}。
                    </p>
                  </article>
                )}
              </Show>
            )}
          </For>
        </section>
      )}
    </Show>
  );
}
