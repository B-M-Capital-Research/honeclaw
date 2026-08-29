import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationLedgerTransitionCandidateAdmissionReviews,
  reviewControlledShadowObservationLedgerTransitionCandidateAdmission,
} from "@/lib/api";
import type {
  ControlledShadowObservationLedgerTransitionCandidateAdmissionRegistry,
  ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest,
} from "@/lib/types";

const REVIEW_CHECKS = [
  "精确绑定当前 Stage 51–123 完整责任链",
  "复核者独立于 Stage 123 验证者、Stage 122 执行人、Stage 121 认领人和全部既有责任人",
  "重新打开、重哈希并核对 Stage 123 不可变终态",
  "重新打开 Stage 122 candidate，重哈希并确认完整内容精确一致",
  "保留 exact Stage 114 已准入观察证据绑定",
  "每条非财务通知的身份、精确十进制、摘要与规范顺序均保持不变",
  "准入只新增分离的正式非财务证据记录，不修改原 candidate",
  "期初组合仍缺失、金融白名单仍为空，且没有权威 ledger event",
  "批准只开放 Stage 125 外部来源期初组合快照治理规格",
  "没有持仓、现金、净值/绩效、模型/训练/RL、reward、订单、券商或交易权限",
  "未把未确认 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationLedgerTransitionCandidateAdmissionPanel() {
  const [registry, setRegistry] =
    createSignal<ControlledShadowObservationLedgerTransitionCandidateAdmissionRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [verdict, setVerdict] = createSignal<ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest["verdict"]>(
    "admitted_as_formal_non_financial_observation_evidence_for_future_opening_portfolio_governance",
  );
  const [rationale, setRationale] = createSignal("");
  const [limitations, setLimitations] = createSignal(
    "期初组合快照尚未准入；本记录仅是正式非财务观察证据，原候选继续保持 untrusted 与 immutable。",
  );
  const [checks, setChecks] = createSignal(REVIEW_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationLedgerTransitionCandidateAdmissionReviews();
      setRegistry(next);
      if (!next.items.some((item) => item.review_eligible && item.candidate.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(next.items.find((item) => item.review_eligible)?.candidate.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 124 非财务观察候选准入复核表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.items.find(
    (item) => item.review_eligible && item.candidate.claim.attempt_id === selectedAttemptId(),
  ));
  const approval = createMemo(() => verdict()
    === "admitted_as_formal_non_financial_observation_evidence_for_future_opening_portfolio_governance");
  const disabled = createMemo(() => busy()
    || !selected()
    || rationale().trim().length === 0
    || limitations().trim().length === 0
    || !checks()[1]
    || (approval() && !checks().every(Boolean)));

  const submit = async () => {
    const item = selected();
    if (!item || disabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const previous = item.latest_review;
      const candidate = item.candidate;
      const next = await reviewControlledShadowObservationLedgerTransitionCandidateAdmission(
        candidate.claim.attempt_id,
        {
          expected_previous_review_id: previous?.review_id ?? null,
          expected_previous_review_sha256: previous?.review_sha256 ?? null,
          expected_stage_123_validation_id: candidate.validation.validation_id,
          expected_stage_123_validation_sha256: candidate.validation.validation_sha256,
          expected_stage_122_result_sha256: candidate.result.result_sha256,
          expected_stage_122_candidate_sha256: candidate.candidate.candidate_sha256,
          expected_stage_121_claim_sha256: candidate.claim.claim_sha256,
          expected_stage_114_review_sha256: candidate.validation.stage_114_review_sha256,
          expected_stage_112_output_sha256: candidate.validation.stage_112_output_sha256,
          verdict: verdict(),
          rationale: rationale().trim(),
          known_limitations: limitations().trim(),
          exact_current_stage_51_through_stage_123_binding_confirmed: checks()[0] as boolean,
          reviewer_independent_from_validator_executor_claimant_and_complete_prior_chain_confirmed: checks()[1] as boolean,
          stage_123_terminal_validation_reopened_rehashed_and_current_confirmed: checks()[2] as boolean,
          stage_122_candidate_reopened_rehashed_and_exact_match_confirmed: checks()[3] as boolean,
          exact_stage_114_admitted_observation_binding_preserved_confirmed: checks()[4] as boolean,
          every_non_financial_notice_identity_decimal_hash_and_order_preserved_confirmed: checks()[5] as boolean,
          admission_creates_separate_formal_non_financial_evidence_record_without_mutating_candidate_confirmed: checks()[6] as boolean,
          opening_portfolio_absent_financial_allowlist_empty_and_no_authoritative_ledger_event_confirmed: checks()[7] as boolean,
          approval_only_opens_stage_125_opening_portfolio_snapshot_governance_specification_confirmed: checks()[8] as boolean,
          no_position_cash_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed: checks()[9] as boolean,
          no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[10] as boolean,
        },
      );
      setRegistry(next);
      setRationale("");
      setChecks(REVIEW_CHECKS.map(() => false));
      setNotice(approval()
        ? "正式非财务观察证据已准入；下一步仅登记期初组合快照治理规格，尚未创建任何财务状态。"
        : "复核意见已追加保存；当前候选没有取得 Stage 125 资格。");
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 124 非财务观察候选独立准入复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="非财务观察候选独立准入复核">
      <header><strong>第 124 阶段 · 非财务观察候选独立准入</strong><span>正式证据 · 不建财务账</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>独立验证候选</span><strong>{current().independently_validated_candidate_count}</strong></div>
        <div><span>待复核</span><strong>{current().review_eligible_candidate_count}</strong></div>
        <div><span>正式非财务证据</span><strong>{current().admitted_non_financial_observation_evidence_count}</strong></div>
        <div><span>退回/拒绝</span><strong>{current().changes_requested_or_rejected_count}</strong></div>
      </div>
      <Show when={current().items.some((item) => item.review_eligible)} fallback={<p>当前没有待复核的 Stage 123 独立验证 candidate。</p>}>
        <label><span>Stage 123 候选</span><select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
          <For each={current().items.filter((item) => item.review_eligible)}>{(item) => (
            <option value={item.candidate.claim.attempt_id}>{item.candidate.claim.attempt_id.slice(0, 12)}… · {item.candidate.validation.validation_sha256.slice(0, 12)}…</option>
          )}</For>
        </select></label>
        <Show when={selected()}>{(item) => <article class="public-admin-reward-governance">
          <header><strong>冻结候选绑定</strong><span>{item().candidate.validation.validated_at}</span></header>
          <p>candidate {item().candidate.candidate.candidate_sha256.slice(0, 16)}… · {item().candidate.candidate.notices.length} 条非财务通知</p>
          <p class="public-admin-anchor-boundary">原 candidate 仍未受信；期初组合不存在，金融事件白名单为空。</p>
        </article>}</Show>
        <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest["verdict"])}>
          <option value="admitted_as_formal_non_financial_observation_evidence_for_future_opening_portfolio_governance">准入为正式非财务观察证据</option>
          <option value="changes_requested">退回补充</option>
          <option value="rejected">拒绝</option>
        </select></label>
        <label><span>复核依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
        <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
        <div class="public-admin-decision-checks"><For each={REVIEW_CHECKS}>{(label, index) => <label><input type="checkbox" checked={checks()[index()]} onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))} /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>{busy() ? "正在重开责任链并复核…" : "提交 Stage 124 独立准入复核"}</button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().items.filter((item) => item.latest_review)}>{(item) => <article class="public-admin-reward-governance">
        <header><strong>{item.latest_review?.formal_non_financial_observation_evidence_admitted ? "已准入正式非财务观察证据" : "未准入"}</strong><span>{item.latest_review?.submitted_at}</span></header>
        <p>{item.latest_review?.rationale}</p>
        <p class="public-admin-anchor-boundary">{item.latest_review?.formal_non_financial_observation_evidence_admitted ? "下一步只登记外部来源期初组合快照治理规格；没有持仓、现金、净值或交易状态。" : item.latest_review?.known_limitations}</p>
      </article>}</For>
    </section>
  )}</Show>;
}
