import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import {
  getControlledShadowObservationInputAdmissionReviews,
  reviewControlledShadowObservationInputAdmission,
} from "@/lib/api";
import type {
  ControlledShadowObservationInputAdmissionRegistry,
  ControlledShadowObservationInputAdmissionVerdict,
} from "@/lib/types";

const ADMISSION_CHECKS = [
  "精确绑定当前 Stage 51–103 完整证据链",
  "复核者独立于 validator、executor 和全部既有责任人",
  "Stage 103 第二实现全量重解析仍为当前且通过",
  "周期确为自然前向，未做历史回填或改写",
  "标的、SPY、窗口和请求身份均与冻结输入一致",
  "逐份复核原始载荷的 HONE 保管取得时间",
  "只把保管取得时间当作保守下限，不冒充供应商发布时间",
  "所有准入行都在冻结窗口内且不晚于本次准入时间",
  "至少一个官方交易日，SPY 三套价格逐日完整",
  "标的缺失全部显式记 gap，未填充或跨序列替代",
  "分红、拆股和三套价格口径继续独立保存",
  "不重写输出、不纠错回填、不事后补历史数据",
  "批准只开放 Stage 105 create-once 观察物化规格登记",
  "不生成观察、账本、持仓、绩效、训练、reward、订单、券商或交易能力",
  "未把未确认的 Hari/老王观点写成系统规则",
] as const;

export function PublicAdminControlledShadowObservationInputAdmissionPanel() {
  const [registry, setRegistry] = createSignal<ControlledShadowObservationInputAdmissionRegistry>();
  const [selectedAttemptId, setSelectedAttemptId] = createSignal("");
  const [verdict, setVerdict] = createSignal<ControlledShadowObservationInputAdmissionVerdict>("changes_requested");
  const [rationale, setRationale] = createSignal("");
  const [limitations, setLimitations] = createSignal("供应商发布时间未验证；仅使用 HONE 保管取得时间作为保守可用下限。");
  const [checks, setChecks] = createSignal(ADMISSION_CHECKS.map(() => false));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    try {
      const next = await getControlledShadowObservationInputAdmissionReviews();
      setRegistry(next);
      if (!next.items.some((item) => item.review_eligible && item.candidate.parser_output.claim.attempt_id === selectedAttemptId())) {
        setSelectedAttemptId(next.items.find((item) => item.review_eligible)?.candidate.parser_output.claim.attempt_id ?? "");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 104 观察输入准入复核表读取失败");
    }
  };
  onMount(() => void load());

  const selected = createMemo(() => registry()?.items.find(
    (item) => item.review_eligible && item.candidate.parser_output.claim.attempt_id === selectedAttemptId(),
  ));
  const approval = createMemo(() => verdict() === "approved_for_future_create_once_observation_materialization_specification_registration");
  const disabled = createMemo(() => busy() || !selected() || !rationale().trim() || !limitations().trim()
    || !checks()[1] || (approval() && !checks().every(Boolean)));

  const submit = async () => {
    const item = selected();
    const outputSha = item?.candidate.parser_output.result.output_sha256;
    if (!item || !outputSha || disabled()) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const parser = item.candidate.parser_output;
      const next = await reviewControlledShadowObservationInputAdmission(parser.claim.attempt_id, {
        expected_previous_review_id: item.latest_review?.review_id ?? null,
        expected_previous_review_sha256: item.latest_review?.review_sha256 ?? null,
        expected_stage_103_validation_id: parser.validation.validation_id,
        expected_stage_103_validation_sha256: parser.validation.validation_sha256,
        expected_stage_102_result_sha256: parser.result.result_sha256,
        expected_stage_102_output_sha256: outputSha,
        expected_stage_101_claim_sha256: parser.claim.claim_sha256,
        expected_stage_101_input_manifest_sha256: parser.claim.fixed_input_manifest.input_manifest_sha256,
        expected_cycle_claim_sha256: item.candidate.cycle_claim.cycle_claim_sha256,
        verdict: verdict(), rationale: rationale().trim(), known_limitations: limitations().trim(),
        exact_current_stage_51_through_stage_103_binding_confirmed: checks()[0] as boolean,
        reviewer_independent_from_validator_executor_and_complete_prior_chain_confirmed: checks()[1] as boolean,
        stage_103_full_reparse_validation_current_and_passed_confirmed: checks()[2] as boolean,
        cycle_claim_natural_forward_only_and_no_backfill_confirmed: checks()[3] as boolean,
        fixed_subject_spy_window_and_request_identities_confirmed: checks()[4] as boolean,
        every_raw_payload_custody_retrieval_timestamp_reviewed_confirmed: checks()[5] as boolean,
        custody_retrieval_time_used_as_conservative_availability_not_provider_publication_confirmed: checks()[6] as boolean,
        admitted_rows_within_frozen_window_and_available_before_admission_confirmed: checks()[7] as boolean,
        official_sessions_and_spy_three_price_bases_complete_confirmed: checks()[8] as boolean,
        subject_gaps_explicit_and_no_fill_or_cross_series_substitution_confirmed: checks()[9] as boolean,
        dividends_splits_and_three_price_bases_remain_separate_confirmed: checks()[10] as boolean,
        exact_output_no_rewrite_correction_or_retroactive_backfill_confirmed: checks()[11] as boolean,
        approval_only_opens_future_materialization_specification_registration_confirmed: checks()[12] as boolean,
        no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: checks()[13] as boolean,
        no_unconfirmed_hari_or_old_wang_logic_claimed: checks()[14] as boolean,
      });
      setRegistry(next);
      setNotice(approval()
        ? "精确输入已准入；仅进入 Stage 105 物化规格登记，观察尚未开始。"
        : "复核意见已追加到不可覆盖责任链；该输入仍未准入。");
      setRationale("");
      setChecks(ADMISSION_CHECKS.map(() => false));
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Stage 104 准入复核失败");
      await load();
    } finally {
      setBusy(false);
    }
  };

  return <Show when={registry()}>{(current) => (
    <section class="public-admin-reward-governance" aria-label="首次自然前向周期观察输入独立准入复核">
      <header><strong>第 104 阶段 · 观察输入独立准入</strong><span>保管时间下限 · 不冒充来源发布时间</span></header>
      <p>{current().scope}</p>
      <div class="public-admin-decision-metrics">
        <div><span>候选</span><strong>{current().independently_validated_input_candidate_count}</strong></div>
        <div><span>待复核</span><strong>{current().review_eligible_candidate_count}</strong></div>
        <div><span>已准入</span><strong>{current().admitted_input_count}</strong></div>
        <div><span>修改/拒绝</span><strong>{current().changes_requested_or_rejected_count}</strong></div>
      </div>
      <p class="public-admin-anchor-boundary">供应商发布时间：未验证。HONE 只把实际保管取得时间作为 point-in-time 的保守下限。</p>
      <Show when={current().items.some((item) => item.review_eligible)} fallback={<p>当前没有待 Stage 104 独立复核的输入。</p>}>
        <label><span>Stage 103 精确输出</span><select value={selectedAttemptId()} onChange={(event) => setSelectedAttemptId(event.currentTarget.value)}>
          <For each={current().items.filter((item) => item.review_eligible)}>{(item) => (
            <option value={item.candidate.parser_output.claim.attempt_id}>
              {item.candidate.parser_output.claim.fixed_input_manifest.subject_symbols.join(", ")} · {item.candidate.parser_output.result.output_sha256?.slice(0, 12)}…
            </option>
          )}</For>
        </select></label>
        <Show when={selected()}>{(item) => <article class="public-admin-reward-governance">
          <header><strong>冻结输入</strong><span>{item().candidate.parser_output.claim.fixed_input_manifest.raw_payload_count} 个原始载荷</span></header>
          <p>{item().candidate.parser_output.claim.fixed_input_manifest.window_start_date} 至 {item().candidate.parser_output.claim.fixed_input_manifest.window_end_date} · 基准 SPY</p>
          <p>Stage 103 {item().candidate.parser_output.validation.validation_sha256.slice(0, 16)}…</p>
        </article>}</Show>
        <label><span>复核结论</span><select value={verdict()} onChange={(event) => setVerdict(event.currentTarget.value as ControlledShadowObservationInputAdmissionVerdict)}>
          <option value="changes_requested">要求修改</option>
          <option value="rejected">拒绝</option>
          <option value="approved_for_future_create_once_observation_materialization_specification_registration">准入，仅开放 Stage 105</option>
        </select></label>
        <label><span>复核依据</span><textarea value={rationale()} onInput={(event) => setRationale(event.currentTarget.value)} /></label>
        <label><span>已知局限</span><textarea value={limitations()} onInput={(event) => setLimitations(event.currentTarget.value)} /></label>
        <div class="public-admin-decision-checks"><For each={ADMISSION_CHECKS}>{(label, index) => <label><input
          type="checkbox" checked={checks()[index()]}
          onChange={(event) => setChecks((values) => values.map((value, currentIndex) => currentIndex === index() ? event.currentTarget.checked : value))}
        /><span>{label}</span></label>}</For></div>
        <button type="button" class="public-admin-decision-submit" disabled={disabled()} onClick={() => void submit()}>
          {busy() ? "正在追加复核记录…" : "提交 Stage 104 独立准入复核"}
        </button>
      </Show>
      <Show when={error()}><p class="public-admin-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-success">{notice()}</p></Show>
      <For each={current().items.filter((item) => item.latest_review)}>{(item) => <article class="public-admin-reward-governance">
        <header><strong>{item.latest_review?.observation_input_admitted ? "输入已准入 · 等待 Stage 105" : "输入未准入"}</strong><span>{item.latest_review?.submitted_at}</span></header>
        <p>官方交易日 {item.latest_review?.official_market_session_count} · 价格行 {item.latest_review?.price_row_count} · 显式缺口 {item.latest_review?.explicit_gap_count}</p>
        <p>保守 available_at：{item.latest_review?.admitted_available_at_utc}</p>
        <p class="public-admin-anchor-boundary">{item.latest_review?.known_limitations}</p>
      </article>}</For>
    </section>
  )}</Show>;
}
