import { For, Show, createSignal, onCleanup, onMount } from "solid-js";
import {
  getInvestmentFinancialEvidenceReviews,
  reviewInvestmentFinancialEvidence,
} from "@/lib/api";
import type {
  InvestmentFinancialEvidenceReviewCandidate,
  InvestmentFinancialEvidenceReviewConfirmations,
  InvestmentFinancialEvidenceReviewRequest,
  InvestmentFinancialEvidenceReviewResponse,
  InvestmentFinancialEvidenceReviewVerdict,
} from "@/lib/types";

type FinancialReviewDraft = InvestmentFinancialEvidenceReviewConfirmations & {
  rationale: string;
};

const EMPTY_DRAFT: FinancialReviewDraft = {
  rationale: "",
  official_filings_opened: false,
  identity_periods_and_units_verified: false,
  calculations_recomputed: false,
  corporate_actions_and_restatements_checked: false,
  quality_warnings_resolved: false,
  no_unresolved_material_issue: false,
};

const CHECKS: Array<{
  key: keyof InvestmentFinancialEvidenceReviewConfirmations;
  label: string;
}> = [
  { key: "official_filings_opened", label: "已打开并核对全部 SEC 原文" },
  { key: "identity_periods_and_units_verified", label: "公司身份、期间、单位与正负号正确" },
  { key: "calculations_recomputed", label: "已独立重算同比、利润率与现金流公式" },
  { key: "corporate_actions_and_restatements_checked", label: "已检查拆股、分拆、并购、重述和口径变化" },
  { key: "quality_warnings_resolved", label: "页面列出的异常警告已有可核验解释" },
  { key: "no_unresolved_material_issue", label: "不存在尚未解决的重大数据问题" },
];

const STATUS_LABELS: Record<string, string> = {
  sec_structured_pending_human_review: "待财务复核",
  sec_human_reviewed_for_rating: "已批准计分",
  sec_review_changes_requested: "要求修正",
  sec_review_rejected: "已拒绝",
  sec_review_stale_evidence_changed: "证据变化，旧复核失效",
  sec_review_audit_invalid: "审计链异常，禁止计分",
};

export function financialReviewCanApprove(draft: FinancialReviewDraft) {
  return (
    draft.rationale.trim().length >= 8
    && CHECKS.every(({ key }) => draft[key])
  );
}

export function buildFinancialEvidenceReviewRequest(
  candidate: InvestmentFinancialEvidenceReviewCandidate,
  draft: FinancialReviewDraft,
  verdict: InvestmentFinancialEvidenceReviewVerdict,
): InvestmentFinancialEvidenceReviewRequest {
  return {
    expected_review_id: candidate.latest_review?.review_id,
    expected_evidence_fingerprint_sha256: candidate.evidence_fingerprint_sha256,
    verdict,
    rationale: draft.rationale.trim(),
    confirmations: {
      official_filings_opened: draft.official_filings_opened,
      identity_periods_and_units_verified: draft.identity_periods_and_units_verified,
      calculations_recomputed: draft.calculations_recomputed,
      corporate_actions_and_restatements_checked:
        draft.corporate_actions_and_restatements_checked,
      quality_warnings_resolved: draft.quality_warnings_resolved,
      no_unresolved_material_issue: draft.no_unresolved_material_issue,
    },
  };
}

function metric(value: number | undefined, suffix = "%") {
  return value == null || !Number.isFinite(value) ? "—" : `${value.toFixed(1)}${suffix}`;
}

function claimBasisSummary(candidate: InvestmentFinancialEvidenceReviewCandidate) {
  const claims = candidate.evidence.source_claims ?? [];
  const standards = [...new Set(claims.map((claim) => claim.metric_basis.split(":", 1)[0]))];
  const units = [...new Set(claims.map((claim) => claim.unit))];
  return [
    standards.length > 0 ? `会计口径 ${standards.join(" / ")}` : undefined,
    units.length > 0 ? `原始单位 ${units.join(" / ")}` : undefined,
  ].filter(Boolean).join(" · ");
}

export function PublicAdminFinancialEvidenceReview() {
  const [report, setReport] =
    createSignal<InvestmentFinancialEvidenceReviewResponse | null>(null);
  const [drafts, setDrafts] = createSignal<Record<string, FinancialReviewDraft>>({});
  const [loading, setLoading] = createSignal(true);
  const [selection, setSelection] =
    createSignal<"active_batch" | "full_queue">("active_batch");
  const [submitting, setSubmitting] = createSignal("");
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");
  const controller = new AbortController();

  const load = async (mode = selection()) => {
    const next = await getInvestmentFinancialEvidenceReviews(
      { selection: mode, limit: 5 },
      controller.signal,
    );
    setReport(next);
    setDrafts((current) => {
      const merged = { ...current };
      for (const candidate of next.candidates) {
        merged[candidate.symbol] ??= { ...EMPTY_DRAFT };
      }
      return merged;
    });
  };

  const changeSelection = async (mode: "active_batch" | "full_queue") => {
    if (mode === selection()) return;
    setSelection(mode);
    setLoading(true);
    setError("");
    try {
      await load(mode);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "读取财务证据复核失败");
    } finally {
      setLoading(false);
    }
  };

  const updateDraft = (symbol: string, patch: Partial<FinancialReviewDraft>) => {
    setDrafts((current) => ({
      ...current,
      [symbol]: { ...(current[symbol] ?? EMPTY_DRAFT), ...patch },
    }));
  };

  const submit = async (
    candidate: InvestmentFinancialEvidenceReviewCandidate,
    verdict: InvestmentFinancialEvidenceReviewVerdict,
  ) => {
    const draft = drafts()[candidate.symbol] ?? EMPTY_DRAFT;
    if (draft.rationale.trim().length < 8) {
      setError("请先写明复核依据或需要修正的问题（至少 8 个字符）");
      return;
    }
    if (verdict === "approved_for_rating" && !financialReviewCanApprove(draft)) {
      setError("批准计分前必须逐项完成全部六项财务质量确认");
      return;
    }
    setSubmitting(candidate.symbol);
    setError("");
    setNotice("");
    try {
      await reviewInvestmentFinancialEvidence(
        candidate.symbol,
        buildFinancialEvidenceReviewRequest(candidate, draft, verdict),
      );
      await load();
      setNotice(
        verdict === "approved_for_rating"
          ? `${candidate.symbol} 当前证据已获准进入每日评级因子；财报变化后会自动失效。`
          : `${candidate.symbol} 复核意见已写入不可变审计链。`,
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "保存财务证据复核失败");
    } finally {
      setSubmitting("");
    }
  };

  onMount(() => {
    void load()
      .catch((cause) => {
        setError(cause instanceof Error ? cause.message : "读取财务证据复核失败");
      })
      .finally(() => setLoading(false));
  });
  onCleanup(() => controller.abort());

  return (
    <section class="public-admin-financial-review" aria-labelledby="financial-review-title">
      <header>
        <div>
          <span>独立数据质量门禁</span>
          <h3 id="financial-review-title">SEC 财务证据复核</h3>
          <p>只审核数字、期间、单位和公式。通过后仅可进入每日评级因子，不代表投资判断正确。</p>
        </div>
        <strong>训练、RL、交易关闭</strong>
      </header>
      <Show when={report()}>
        {(value) => (
          <>
            <div class="public-admin-financial-review-summary">
              <span>已观察 <strong>{value().summary.observed}</strong></span>
              <span>待复核 <strong>{value().summary.pending}</strong></span>
              <span>可计分 <strong>{value().summary.approved_for_rating}</strong></span>
              <span>证据变化失效 <strong>{value().summary.stale_after_evidence_change}</strong></span>
            </div>
            <p class="public-admin-financial-review-scope">{value().scope}</p>
            <div class="public-admin-financial-review-selection">
              <div>
                <strong>{value().selection_mode === "active_batch" ? "本批优先审核" : "完整审核队列"}</strong>
                <span>
                  {value().selection_mode === "active_batch"
                    ? `显示 ${value().returned} / ${value().eligible_queue} 个可处理项目`
                    : `显示全部 ${value().returned} 个项目`}
                </span>
                <small>{value().selection_scope}</small>
              </div>
              <nav aria-label="财务审核队列范围">
                <button
                  type="button"
                  classList={{ "is-active": selection() === "active_batch" }}
                  onClick={() => void changeSelection("active_batch")}
                >优先 5 家</button>
                <button
                  type="button"
                  classList={{ "is-active": selection() === "full_queue" }}
                  onClick={() => void changeSelection("full_queue")}
                >查看全部</button>
              </nav>
            </div>
            <div class="public-admin-financial-review-list">
              <For each={value().candidates}>
                {(candidate, index) => {
                  const draft = () => drafts()[candidate.symbol] ?? EMPTY_DRAFT;
                  return (
                    <details open={value().selection_mode === "active_batch" && index() === 0}>
                      <summary>
                        <span>
                          <strong>{candidate.symbol}</strong>
                          <small>截止 {candidate.evidence.financial_as_of ?? "未知"}</small>
                        </span>
                        <span class="public-admin-financial-review-status">
                          <small>审核顺位 {candidate.review_priority_rank + 1}</small>
                          <em classList={{ "is-approved": candidate.score_eligible }}>
                            {STATUS_LABELS[candidate.review_status] ?? candidate.review_status}
                          </em>
                        </span>
                      </summary>
                      <p class="public-admin-financial-review-priority">
                        <For each={candidate.review_priority_reasons}>{(reason) => <span>{reason}</span>}</For>
                      </p>
                      <div class="public-admin-financial-review-metrics">
                        <span>收入同比 <strong>{metric(candidate.evidence.revenue_growth_percent)}</strong></span>
                        <span>毛利率 <strong>{metric(candidate.evidence.gross_margin_percent)}</strong></span>
                        <span>毛利率同比 <strong>{metric(candidate.evidence.gross_margin_change_pp, " 个百分点")}</strong></span>
                        <span>营业利润率 <strong>{metric(candidate.evidence.ebit_margin_percent)}</strong></span>
                        <span>应收同比 <strong>{metric(candidate.evidence.accounts_receivable_growth_percent)}</strong></span>
                        <span>库存同比 <strong>{metric(candidate.evidence.inventory_growth_percent)}</strong></span>
                        <span>经营现金流同比 <strong>{metric(candidate.evidence.operating_cash_flow_growth_percent)}</strong></span>
                        <span>资本开支同比 <strong>{metric(candidate.evidence.capital_expenditure_growth_percent)}</strong></span>
                      </div>
                      <Show when={claimBasisSummary(candidate)}>
                        <p class="public-admin-financial-review-basis">{claimBasisSummary(candidate)}</p>
                      </Show>
                      <Show when={candidate.evidence.quality_warnings.length > 0}>
                        <div class="public-admin-financial-review-warnings">
                          <strong>必须解释的异常</strong>
                          <For each={candidate.evidence.quality_warnings}>{(warning) => <span>{warning}</span>}</For>
                        </div>
                      </Show>
                      <details class="public-admin-financial-review-trace">
                        <summary>查看逐项口径、计算、SEC 原文和证据指纹</summary>
                        <For each={candidate.evidence.source_calculations}>{(calculation) => <p>{calculation}</p>}</For>
                        <div class="public-admin-financial-review-claims">
                          <For each={candidate.evidence.source_claims ?? []}>
                            {(claim) => (
                              <p>
                                <strong>{claim.metric_id}</strong> · {claim.period} · {claim.numeric_value.toLocaleString()} {claim.unit}
                                <small>{claim.metric_basis} · 发布 {new Date(claim.published_at).toLocaleString("zh-CN", { hour12: false })}</small>
                                <a href={claim.source_url} target="_blank" rel="noreferrer">对应 SEC 原文</a>
                              </p>
                            )}
                          </For>
                        </div>
                        <For each={candidate.evidence.source_urls}>
                          {(url) => <a href={url} target="_blank" rel="noreferrer">打开 SEC 原文</a>}
                        </For>
                        <code>{candidate.evidence_fingerprint_sha256}</code>
                      </details>
                      <div class="public-admin-financial-review-checks">
                        <For each={CHECKS}>
                          {(check) => (
                            <label>
                              <input
                                type="checkbox"
                                checked={Boolean(draft()[check.key])}
                                onChange={(event) => updateDraft(candidate.symbol, {
                                  [check.key]: event.currentTarget.checked,
                                })}
                              />
                              <span>{check.label}</span>
                            </label>
                          )}
                        </For>
                      </div>
                      <label class="public-admin-financial-review-rationale">
                        <span>复核依据或问题</span>
                        <textarea
                          value={draft().rationale}
                          onInput={(event) => updateDraft(candidate.symbol, {
                            rationale: event.currentTarget.value,
                          })}
                          placeholder="写明核对了哪份表、异常原因和仍存在的限制"
                        />
                      </label>
                      <div class="public-admin-financial-review-actions">
                        <button
                          type="button"
                          disabled={submitting() === candidate.symbol || !financialReviewCanApprove(draft())}
                          onClick={() => void submit(candidate, "approved_for_rating")}
                        >批准进入每日评级</button>
                        <button
                          type="button"
                          disabled={submitting() === candidate.symbol || draft().rationale.trim().length < 8}
                          onClick={() => void submit(candidate, "changes_requested")}
                        >要求修正</button>
                        <button
                          type="button"
                          class="is-danger"
                          disabled={submitting() === candidate.symbol || draft().rationale.trim().length < 8}
                          onClick={() => void submit(candidate, "rejected")}
                        >拒绝本次证据</button>
                      </div>
                    </details>
                  );
                }}
              </For>
            </div>
          </>
        )}
      </Show>
      <Show when={loading()}><p class="public-admin-decision-empty">正在读取 SEC 财务证据…</p></Show>
      <Show when={error()}><p class="public-admin-decision-message is-error">{error()}</p></Show>
      <Show when={notice()}><p class="public-admin-decision-message is-success">{notice()}</p></Show>
    </section>
  );
}
