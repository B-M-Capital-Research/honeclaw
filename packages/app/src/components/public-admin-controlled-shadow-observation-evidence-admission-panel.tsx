import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationEvidenceAdmissionReviews,
  reviewControlledShadowObservationEvidenceAdmission,
} from "@/lib/api";
import type {
  ControlledShadowObservationEvidenceAdmissionRegistry,
  ReviewControlledShadowObservationEvidenceAdmissionRequest,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–113 完整责任链",
  "复核者独立于 Stage 113 验证者、Stage 112 执行人和全部既有责任人",
  "重新打开、重哈希并核对 Stage 113 不可变终态",
  "重新打开 Stage 112 envelope 并再次执行完整独立重投影",
  "保留 exact Stage 104 准入输入绑定",
  "sessions、三价格口径、缺口、公司行动、初始分配和 available-at 精确不变",
  "仅自然前向，不重新抓取、填充、替代、改写、修正或历史回填",
  "供应商发布时间仍未验证，只保留 custody-time 保守下限",
  "准入只新增分离证据记录，不修改原 envelope",
  "批准只开放 Stage 115 账本转换规格登记",
  "没有账本、持仓、净值/绩效、模型/训练/RL、reward、订单、券商或交易权限",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationEvidenceAdmissionPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowObservationEvidenceAdmissionRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [verdict, setVerdict] = createSignal<ReviewControlledShadowObservationEvidenceAdmissionRequest["verdict"]>(
    "admitted_for_future_observation_ledger_transition_specification_registration",
  );
  const [rationale, setRationale] = createSignal("");
  const [limitations, setLimitations] = createSignal("供应商发布时间未验证；仅保留 Stage 104 custody-time 保守可用时间下限。原 envelope 继续保持 untrusted 与 immutable。");
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationEvidenceAdmissionReviews();
      setRegistry(next);
      if (!next.items.some((item) => item.review_eligible && item.candidate.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(next.items.find((item) => item.review_eligible)?.candidate.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 114 观察证据准入复核表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.items.find(
    (item) => item.review_eligible && item.candidate.claim.attempt_id === selectedAttemptId(),
  ));
  const approval = createMemo(() => verdict() === "admitted_for_future_observation_ledger_transition_specification_registration");
  const disabled = createMemo(() => busy()
    || !selected()
    || rationale().trim().length === 0
    || limitations().trim().length === 0
    || !checks()[1]
    || (approval() && !checks().every(Boolean)));

  const submit = async () => {
    const item = selected();
    const outputSha = item?.candidate.result.output_sha256;
    if (!item || !outputSha || disabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const previous = item.latest_review;
      const next = await reviewControlledShadowObservationEvidenceAdmission(
        item.candidate.claim.attempt_id,
        {
          expected_previous_review_id: previous?.review_id ?? null,
          expected_previous_review_sha256: previous?.review_sha256 ?? null,
          expected_stage_113_validation_id: item.candidate.validation.validation_id,
          expected_stage_113_validation_sha256: item.candidate.validation.validation_sha256,
          expected_stage_112_result_sha256: item.candidate.result.result_sha256,
          expected_stage_112_output_sha256: outputSha,
          expected_stage_111_claim_sha256: item.candidate.claim.claim_sha256,
          verdict: verdict(),
          rationale: rationale().trim(),
          known_limitations: limitations().trim(),
          exact_current_stage_51_through_stage_113_binding_confirmed: checks()[0] as boolean,
          reviewer_independent_from_validator_executor_and_complete_prior_chain_confirmed: checks()[1] as boolean,
          stage_113_terminal_validation_reopened_rehashed_and_current_confirmed: checks()[2] as boolean,
          stage_112_envelope_reopened_rehashed_and_reprojected_confirmed: checks()[3] as boolean,
          exact_stage_104_admitted_input_binding_preserved_confirmed: checks()[4] as boolean,
          sessions_prices_gaps_actions_allocation_and_available_at_exactly_preserved_confirmed: checks()[5] as boolean,
          natural_forward_only_no_refetch_fill_substitution_rewrite_correction_or_backfill_confirmed: checks()[6] as boolean,
          provider_publication_time_unverified_and_custody_time_floor_preserved_confirmed: checks()[7] as boolean,
          admission_preserves_original_envelope_and_only_creates_separate_evidence_record_confirmed: checks()[8] as boolean,
          approval_only_opens_future_observation_ledger_transition_specification_registration_confirmed: checks()[9] as boolean,
          no_ledger_position_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed: checks()[10] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[11] as boolean,
        },
      );
      setRegistry(next);
      setRationale("");
      setChecks(REVIEW_CHECKS.map(() => false));
      setNotice(approval()
        ? "精确观察证据已准入；仅开放 Stage 115 账本转换规格登记，尚未建账或计算绩效。"
        : "复核意见已追加保存；该观察证据没有取得后续资格。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 114 观察证据准入复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="观察证据独立准入复核">
      <header><strong>第 114 阶段 · 观察证据独立准入</strong><span>分离证据 · 不建账</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>独立验证候选</span><strong>{current().independently_validated_candidate_count}</strong></div>
        <div><span>待复核</span><strong>{current().review_eligible_candidate_count}</strong></div>
        <div><span>正式证据</span><strong>{current().admitted_observation_evidence_count}</strong></div>
        <div><span>退回/拒绝</span><strong>{current().changes_requested_or_rejected_count}</strong></div>
      </div>
      <Show when={current().items.some((item) => item.review_eligible)} fallback={<p>当前没有待复核的 Stage 113 独立验证观察 envelope。</p>}>
        <label><span>Stage 113 候选</span><select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
          <For each={current().items.filter((item) => item.review_eligible)}>{(item) => (
            <option value={item.candidate.claim.attempt_id}>{item.candidate.claim.attempt_id.slice(0, 12)}… · {item.candidate.validation.validation_sha256.slice(0, 12)}…</option>
          )}</For>
        </select></label>
        <Show when={selected()}>{(item) => <article class="public-admin-reward-governance">
          <header><strong>冻结证据绑定</strong><span>{item().candidate.validation.validated_at}</span></header>
          <p>output {item().candidate.result.output_sha256?.slice(0, 16)}… · {item().candidate.validation.observed_price_count} 个价格观察 · {item().candidate.validation.observed_gap_count} 个显式缺口</p>
          <p class="public-admin-anchor-boundary">原 envelope 不改写；供应商发布时间仍未验证。</p>
        </article>}</Show>
        <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ReviewControlledShadowObservationEvidenceAdmissionRequest["verdict"])}>
          <option value="admitted_for_future_observation_ledger_transition_specification_registration">准入为正式观察证据</option>
          <option value="changes_requested">退回补充</option>
          <option value="rejected">拒绝</option>
        </select></label>
        <label><span>复核依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
        <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
        <div class="public-admin-decision-checks"><For each={REVIEW_CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在重开证据并复核…" : "提交 Stage 114 独立准入复核"}</button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().items.filter((item) => item.latest_review)}>{(item) => <article class="public-admin-reward-governance">
        <header><strong>{item.latest_review?.observation_evidence_admitted ? "已准入正式观察证据" : "未准入"}</strong><span>{item.latest_review?.submitted_at}</span></header>
        <p>{item.latest_review?.rationale}</p>
        <p class="public-admin-anchor-boundary">{item.latest_review?.observation_evidence_admitted ? "下一步仅登记账本转换规格；尚未建账、计算绩效或训练。" : item.latest_review?.known_limitations}</p>
      </article>}</For>
    </section>
  )}</Show>;
}
