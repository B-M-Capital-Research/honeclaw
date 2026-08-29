import { For, Show, createSignal, onCleanup, onMount } from "solid-js";
import {
  getInvestmentValuationInputReviews,
  reviewInvestmentValuationInputs,
} from "@/lib/api";
import type {
  InvestmentSupplementalValuationInputs,
  InvestmentValuationInputReviewCandidate,
  InvestmentValuationInputReviewConfirmations,
  InvestmentValuationInputReviewRequest,
  InvestmentValuationInputReviewResponse,
  InvestmentValuationInputReviewVerdict,
} from "@/lib/types";

type ValuationReviewDraft = {
  rationale: string;
  input_as_of: string;
  diluted_shares_millions: string;
  complete_net_cash_millions: string;
  forward_eps: string;
  forward_revenue_millions: string;
  normalized_ebit_margin_percent: string;
  annual_fcf_history_millions: string;
  source_urls: string;
  source_note: string;
} & InvestmentValuationInputReviewConfirmations;

const CHECKS: Array<{
  key: keyof InvestmentValuationInputReviewConfirmations;
  label: string;
}> = [
  { key: "official_sources_opened", label: "已打开并核对全部原始来源" },
  { key: "sec_financial_values_recomputed", label: "已独立重算 SEC 财务值与自由现金流" },
  { key: "diluted_share_count_and_corporate_actions_verified", label: "已核对稀释股本、拆股、分拆与并购影响" },
  { key: "complete_net_cash_or_debt_verified", label: "已核对完整现金、短长期债务及必要调整" },
  { key: "forward_or_midcycle_inputs_verified", label: "已核对前瞻一致预期或中周期输入及日期" },
  { key: "cyclicality_and_normalization_checked", label: "已检查周期高点、一次性项目和正常化口径" },
  { key: "cross_method_comparability_checked", label: "至少两种方法的单位、期间和股本口径可比" },
  { key: "no_unresolved_material_issue", label: "不存在尚未解决的重大估值输入问题" },
];

const STATUS_LABELS: Record<string, string> = {
  sec_valuation_review_pending: "待独立估值复核",
  sec_human_reviewed_for_valuation: "估值输入已授权",
  sec_valuation_review_changes_requested: "要求修正",
  sec_valuation_review_rejected: "已拒绝",
  sec_valuation_review_stale_evidence_changed: "SEC 证据变化，授权失效",
  sec_valuation_review_stale_input_expired: "补充输入超过 7 天，授权失效",
  sec_valuation_review_audit_invalid: "审计链异常，估值关闭",
};

function today() {
  const date = new Date();
  const local = new Date(date.getTime() - date.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 10);
}

function emptyDraft(candidate: InvestmentValuationInputReviewCandidate): ValuationReviewDraft {
  const previous = candidate.latest_review?.supplemental_inputs;
  const currentFcf = candidate.financial_evidence.current_free_cash_flow;
  const priorFcf = candidate.financial_evidence.prior_free_cash_flow;
  return {
    rationale: "",
    input_as_of: previous?.input_as_of ?? today(),
    diluted_shares_millions: previous?.diluted_shares_millions?.toString() ?? "",
    complete_net_cash_millions: previous?.complete_net_cash_millions?.toString() ?? "",
    forward_eps: previous?.forward_eps?.toString() ?? "",
    forward_revenue_millions: previous?.forward_revenue_millions?.toString() ?? "",
    normalized_ebit_margin_percent:
      previous?.normalized_ebit_margin_percent?.toString() ?? "",
    annual_fcf_history_millions:
      previous?.annual_fcf_history_millions.join(", ")
      ?? [priorFcf, currentFcf].filter((value) => value != null).join(", "),
    source_urls: previous?.source_urls.join("\n")
      ?? candidate.financial_evidence.source_urls.join("\n"),
    source_note: previous?.source_note ?? "",
    official_sources_opened: false,
    sec_financial_values_recomputed: false,
    diluted_share_count_and_corporate_actions_verified: false,
    complete_net_cash_or_debt_verified: false,
    forward_or_midcycle_inputs_verified: false,
    cyclicality_and_normalization_checked: false,
    cross_method_comparability_checked: false,
    no_unresolved_material_issue: false,
  };
}

function optionalNumber(value: string) {
  const normalized = value.trim();
  if (!normalized) return undefined;
  const parsed = Number(normalized);
  return Number.isFinite(parsed) ? parsed : undefined;
}

function fcfHistory(value: string) {
  return value
    .split(/[，,\n]/)
    .map((item) => Number(item.trim()))
    .filter(Number.isFinite);
}

function supplementalInputs(draft: ValuationReviewDraft): InvestmentSupplementalValuationInputs {
  return {
    input_as_of: draft.input_as_of,
    currency: "USD",
    diluted_shares_millions: optionalNumber(draft.diluted_shares_millions),
    complete_net_cash_millions: optionalNumber(draft.complete_net_cash_millions),
    forward_eps: optionalNumber(draft.forward_eps),
    forward_revenue_millions: optionalNumber(draft.forward_revenue_millions),
    normalized_ebit_margin_percent: optionalNumber(draft.normalized_ebit_margin_percent),
    annual_fcf_history_millions: fcfHistory(draft.annual_fcf_history_millions),
    source_urls: draft.source_urls
      .split(/\n/)
      .map((value) => value.trim())
      .filter(Boolean),
    source_note: draft.source_note.trim(),
  };
}

export function valuationPreparedMethods(draft: ValuationReviewDraft) {
  const inputs = supplementalInputs(draft);
  const methods: string[] = [];
  if ((inputs.forward_eps ?? 0) > 0) methods.push("前瞻 P/E");
  if (
    (inputs.forward_revenue_millions ?? 0) > 0
    && (inputs.normalized_ebit_margin_percent ?? 0) > 0
  ) methods.push("EV/EBIT");
  if (
    inputs.annual_fcf_history_millions.length >= 3
    && inputs.annual_fcf_history_millions.filter((value) => value > 0).length >= 2
  ) methods.push("周期调整 DCF");
  return methods;
}

export function valuationReviewCanApprove(draft: ValuationReviewDraft) {
  const inputs = supplementalInputs(draft);
  return (
    draft.rationale.trim().length >= 8
    && Boolean(inputs.input_as_of)
    && (inputs.diluted_shares_millions ?? 0) > 0
    && inputs.complete_net_cash_millions != null
    && valuationPreparedMethods(draft).length >= 2
    && inputs.source_urls.length > 0
    && inputs.source_note.length >= 8
    && CHECKS.every(({ key }) => draft[key])
  );
}

export function buildValuationInputReviewRequest(
  candidate: InvestmentValuationInputReviewCandidate,
  draft: ValuationReviewDraft,
  verdict: InvestmentValuationInputReviewVerdict,
): InvestmentValuationInputReviewRequest {
  return {
    expected_review_id: candidate.latest_review?.review_id,
    expected_financial_evidence_fingerprint_sha256:
      candidate.financial_evidence_fingerprint_sha256,
    verdict,
    rationale: draft.rationale.trim(),
    confirmations: Object.fromEntries(
      CHECKS.map(({ key }) => [key, draft[key]]),
    ) as InvestmentValuationInputReviewConfirmations,
    supplemental_inputs: supplementalInputs(draft),
  };
}

export function PublicAdminValuationInputReview() {
  const [report, setReport] = createSignal<InvestmentValuationInputReviewResponse>();
  const [drafts, setDrafts] = createSignal<Record<string, ValuationReviewDraft>>({});
  const [loading, setLoading] = createSignal(true);
  const [submitting, setSubmitting] = createSignal("");
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");
  const controller = new AbortController();

  const load = async () => {
    const next = await getInvestmentValuationInputReviews(undefined, controller.signal);
    setReport(next);
    setDrafts((current) => {
      const merged = { ...current };
      for (const candidate of next.candidates) {
        merged[candidate.symbol] ??= emptyDraft(candidate);
      }
      return merged;
    });
  };

  const updateDraft = (
    candidate: InvestmentValuationInputReviewCandidate,
    patch: Partial<ValuationReviewDraft>,
  ) => {
    const symbol = candidate.symbol;
    setDrafts((current) => ({
      ...current,
      [symbol]: { ...(current[symbol] ?? emptyDraft(candidate)), ...patch },
    }));
  };

  const submit = async (
    candidate: InvestmentValuationInputReviewCandidate,
    verdict: InvestmentValuationInputReviewVerdict,
  ) => {
    const draft = drafts()[candidate.symbol] ?? emptyDraft(candidate);
    if (draft.rationale.trim().length < 8) {
      setError("请先写明估值复核依据或需要修正的问题（至少 8 个字符）");
      return;
    }
    if (verdict === "approved_for_valuation" && !valuationReviewCanApprove(draft)) {
      setError("批准前必须补齐股本、完整净现金、至少两种方法、来源说明和全部八项确认");
      return;
    }
    setSubmitting(candidate.symbol);
    setError("");
    setNotice("");
    try {
      await reviewInvestmentValuationInputs(
        candidate.symbol,
        buildValuationInputReviewRequest(candidate, draft, verdict),
      );
      await load();
      setNotice(
        verdict === "approved_for_valuation"
          ? `${candidate.symbol} 的精确输入包已获 7 天估值用途授权；证据或输入变化会自动失效。`
          : `${candidate.symbol} 的估值复核意见已写入不可变审计链。`,
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存估值输入复核失败");
    } finally {
      setSubmitting("");
    }
  };

  onMount(() => void load().catch((cause) => {
    setError(cause instanceof Error ? cause.message : "读取估值输入复核失败");
  }).finally(() => setLoading(false)));
  onCleanup(() => controller.abort());

  return (
    <section class="public-admin-financial-review public-admin-valuation-review" aria-labelledby="valuation-review-title">
      <header>
        <div>
          <span>独立估值用途门禁</span>
          <h3 id="valuation-review-title">SEC 估值输入复核</h3>
          <p>补齐并核对稀释股本、完整净现金、前瞻/中周期输入和至少两种方法。评级财务批准不能替代这里；通过也不授权训练、奖励、组合、影子组合或交易。</p>
        </div>
        <strong>7 天有效 · 交易关闭</strong>
      </header>
      <Show when={report()}>
        {(value) => (
          <>
            <div class="public-admin-financial-review-summary">
              <span>已观察 <strong>{value().observed}</strong></span>
              <span>待复核 <strong>{value().pending}</strong></span>
              <span>估值授权 <strong>{value().authorized}</strong></span>
              <span>已失效 <strong>{value().stale}</strong></span>
            </div>
            <p class="public-admin-financial-review-scope">{value().scope}</p>
            <div class="public-admin-financial-review-list">
              <For each={value().candidates}>
                {(candidate, index) => {
                  const draft = () => drafts()[candidate.symbol] ?? emptyDraft(candidate);
                  const methods = () => valuationPreparedMethods(draft());
                  return (
                    <details open={index() === 0}>
                      <summary>
                        <span><strong>{candidate.symbol}</strong><small>SEC 截止 {candidate.financial_evidence.financial_as_of ?? "未知"}</small></span>
                        <span class="public-admin-financial-review-status">
                          <small>{methods().length}/3 种方法输入</small>
                          <em classList={{ "is-approved": candidate.valuation_authorized }}>
                            {STATUS_LABELS[candidate.review_status] ?? candidate.review_status}
                          </em>
                        </span>
                      </summary>
                      <Show when={candidate.blocking_reasons.length > 0}>
                        <p class="public-admin-financial-review-priority">
                          <For each={candidate.blocking_reasons}>{(reason) => <span>{reason}</span>}</For>
                        </p>
                      </Show>
                      <div class="public-admin-financial-review-metrics">
                        <span>SEC 本期 FCF<strong>{candidate.financial_evidence.current_free_cash_flow?.toLocaleString() ?? "—"} 百万美元</strong></span>
                        <span>SEC 上期 FCF<strong>{candidate.financial_evidence.prior_free_cash_flow?.toLocaleString() ?? "—"} 百万美元</strong></span>
                        <span>现金<strong>{candidate.financial_evidence.cash_and_equivalents?.toLocaleString() ?? "—"} 百万美元</strong></span>
                        <span>当前 XBRL 长债<strong>{candidate.financial_evidence.long_term_debt?.toLocaleString() ?? "—"} 百万美元</strong></span>
                      </div>
                      <div class="public-admin-valuation-review-fields">
                        <label><span>输入日期</span><input type="date" value={draft().input_as_of} onInput={(event) => updateDraft(candidate, { input_as_of: event.currentTarget.value })} /></label>
                        <label><span>稀释股本（百万股）</span><input inputmode="decimal" value={draft().diluted_shares_millions} onInput={(event) => updateDraft(candidate, { diluted_shares_millions: event.currentTarget.value })} /></label>
                        <label><span>完整净现金/负债（百万美元）</span><input inputmode="decimal" value={draft().complete_net_cash_millions} onInput={(event) => updateDraft(candidate, { complete_net_cash_millions: event.currentTarget.value })} /></label>
                        <label><span>下一财年 EPS</span><input inputmode="decimal" value={draft().forward_eps} onInput={(event) => updateDraft(candidate, { forward_eps: event.currentTarget.value })} /></label>
                        <label><span>下一财年收入（百万美元）</span><input inputmode="decimal" value={draft().forward_revenue_millions} onInput={(event) => updateDraft(candidate, { forward_revenue_millions: event.currentTarget.value })} /></label>
                        <label><span>正常化 EBIT 利润率（%）</span><input inputmode="decimal" value={draft().normalized_ebit_margin_percent} onInput={(event) => updateDraft(candidate, { normalized_ebit_margin_percent: event.currentTarget.value })} /></label>
                        <label class="is-wide"><span>年度 FCF 历史（百万美元，逗号分隔，旧→新）</span><input value={draft().annual_fcf_history_millions} onInput={(event) => updateDraft(candidate, { annual_fcf_history_millions: event.currentTarget.value })} /></label>
                        <label class="is-wide"><span>补充输入来源（每行一个 HTTPS 链接）</span><textarea value={draft().source_urls} onInput={(event) => updateDraft(candidate, { source_urls: event.currentTarget.value })} /></label>
                        <label class="is-wide"><span>口径、期间与推导说明</span><textarea value={draft().source_note} onInput={(event) => updateDraft(candidate, { source_note: event.currentTarget.value })} /></label>
                      </div>
                      <p class="public-admin-financial-review-priority">
                        <span>已准备：{methods().length > 0 ? methods().join("、") : "尚无完整方法"}</span>
                        <span>批准门槛：至少两种方法</span>
                      </p>
                      <div class="public-admin-financial-review-checks">
                        <For each={CHECKS}>{(check) => (
                          <label>
                            <input type="checkbox" checked={Boolean(draft()[check.key])} onChange={(event) => updateDraft(candidate, { [check.key]: event.currentTarget.checked })} />
                            <span>{check.label}</span>
                          </label>
                        )}</For>
                      </div>
                      <label class="public-admin-financial-review-rationale">
                        <span>复核依据或问题</span>
                        <textarea value={draft().rationale} onInput={(event) => updateDraft(candidate, { rationale: event.currentTarget.value })} placeholder="写明使用了哪些来源、如何重算、周期与口径风险" />
                      </label>
                      <div class="public-admin-financial-review-actions">
                        <button type="button" disabled={submitting() === candidate.symbol || !valuationReviewCanApprove(draft())} onClick={() => void submit(candidate, "approved_for_valuation")}>批准进入估值模型</button>
                        <button type="button" disabled={submitting() === candidate.symbol || draft().rationale.trim().length < 8} onClick={() => void submit(candidate, "changes_requested")}>要求修正</button>
                        <button type="button" class="is-danger" disabled={submitting() === candidate.symbol || draft().rationale.trim().length < 8} onClick={() => void submit(candidate, "rejected")}>拒绝输入包</button>
                      </div>
                    </details>
                  );
                }}
              </For>
            </div>
          </>
        )}
      </Show>
      <Show when={loading()}><p class="public-admin-decision-empty">正在读取估值输入复核…</p></Show>
      <Show when={error()}><p class="public-admin-decision-message is-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-decision-message is-success">{notice()}</p></Show>
    </section>
  );
}
