import { Title } from "@solidjs/meta";
import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";

import { PublicLoginForm } from "@/components/public-login-form";
import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import { getPublicValuationLab, isUnauthorizedApiError } from "@/lib/api";
import type { ValuationLabItem, ValuationLabSnapshot } from "@/lib/types";

import "./public-foundation.css";
import "./public-site.css";
import "./public-polish.css";
import "./public-valuation-lab.css";
import "./public-valuation-lab-v2.css";

type ViewState = "loading" | "ready" | "login" | "error";
type Filter = "all" | "ready" | "review" | "unavailable";

function money(value: number | null | undefined, currency = "USD") {
  if (value == null) return "—";
  return new Intl.NumberFormat("zh-CN", {
    style: "currency",
    currency,
    maximumFractionDigits: 2,
  }).format(value);
}

function percent(value: number | null | undefined) {
  return value == null ? "—" : `${(value * 100).toFixed(1)}%`;
}

function pointPercent(value: number | null | undefined) {
  return value == null ? "—" : `${value >= 0 ? "+" : ""}${value.toFixed(1)}%`;
}

function statusLabel(item: ValuationLabItem) {
  if (item.status === "ready") return "可进入评级";
  if (item.status === "review_required") return "需要复核";
  if (item.status === "stale") return "已过期";
  return "无法估值";
}

function financialReviewLabel(status: string) {
  if (status === "sec_human_reviewed_for_rating") return "已复核，仅授权评级";
  if (status === "sec_structured_pending_human_review") return "SEC 财务待人工复核";
  if (status === "sec_review_stale_evidence_changed") return "证据变化，需重审";
  if (status === "sec_review_changes_requested") return "财务证据待修正";
  if (status === "sec_review_rejected") return "财务证据未通过";
  if (status === "sec_review_audit_invalid") return "财务复核链异常";
  return "尚无 SEC 财务观察";
}

function valuationReviewLabel(status: string) {
  if (status === "sec_human_reviewed_for_valuation") return "估值输入已独立授权";
  if (status === "sec_valuation_review_stale_evidence_changed") return "SEC 证据变化，估值授权失效";
  if (status === "sec_valuation_review_stale_input_expired") return "估值输入超过 7 天，需重审";
  if (status === "sec_valuation_review_changes_requested") return "估值输入待修正";
  if (status === "sec_valuation_review_rejected") return "估值输入未通过";
  if (status === "sec_valuation_review_audit_invalid") return "估值复核链异常";
  return "估值用途待独立复核";
}

export default function PublicValuationLabPage() {
  const [view, setView] = createSignal<ViewState>("loading");
  const [snapshot, setSnapshot] = createSignal<ValuationLabSnapshot>();
  const [filter, setFilter] = createSignal<Filter>("all");
  const [query, setQuery] = createSignal("");
  const [error, setError] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  let controller: AbortController | undefined;

  const load = async () => {
    controller?.abort();
    controller = new AbortController();
    setLoading(true);
    setError("");
    try {
      setSnapshot(await getPublicValuationLab(controller.signal));
      setView("ready");
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      if (isUnauthorizedApiError(cause)) setView("login");
      else {
        setError(cause instanceof Error ? cause.message : String(cause));
        setView("error");
      }
    } finally {
      setLoading(false);
    }
  };

  onMount(() => void load());
  onCleanup(() => controller?.abort());

  const visible = createMemo(() => {
    const search = query().trim().toLowerCase();
    return (snapshot()?.items ?? []).filter((item) => {
      const matchesFilter =
        filter() === "all" ||
        (filter() === "ready" && item.status === "ready") ||
        (filter() === "review" && item.status === "review_required") ||
        (filter() === "unavailable" && !["ready", "review_required"].includes(item.status));
      const matchesSearch =
        !search || `${item.symbol} ${item.name} ${item.theme}`.toLowerCase().includes(search);
      return matchesFilter && matchesSearch;
    });
  });

  return (
    <>
      <Title>每日估值实验室 · HONE</Title>
      <Show when={view() !== "loading"} fallback={<div class="valuation-lab-loading">正在读取当日估值…</div>}>
        <Show
          when={view() !== "login"}
          fallback={<PublicLoginForm title="登录后查看估值实验室" subtitle="估值区间只用于研究，不会自动执行交易。" onLogin={() => void load()} />}
        >
          <PublicWorkspaceShell active="insights" topbarLabel="估值实验室">
            <main class="valuation-lab-page">
              <header class="valuation-lab-hero">
                <div>
                  <span>VALUATION LAB</span>
                  <h1>每日估值实验室</h1>
                  <p>{snapshot()?.summary ?? "读取每日估值覆盖情况。"}</p>
                </div>
                <dl>
                  <div><dt>覆盖公司</dt><dd>{snapshot()?.coverage.companies ?? 0}</dd></div>
                  <div><dt>完成计算</dt><dd>{snapshot()?.coverage.calculated ?? 0}</dd></div>
                  <div><dt>进入评级</dt><dd>{snapshot()?.coverage.eligible_for_rating ?? 0}</dd></div>
                </dl>
              </header>

              <section class="valuation-lab-method">
                <div><strong>周期制造</strong><span>前瞻 P/E · EV/EBIT · 周期 DCF</span></div>
                <div><strong>盈利成长</strong><span>前瞻 P/E · DCF · EV/EBIT</span></div>
                <div><strong>收入转型</strong><span>EV/S · DCF / 盈利拐点</span></div>
                <p>{snapshot()?.methodology_note}</p>
              </section>

              <section class="valuation-lab-toolbar">
                <label>
                  <span aria-hidden="true">⌕</span>
                  <input value={query()} onInput={(event) => setQuery(event.currentTarget.value)} placeholder="搜索公司、Ticker 或主题" />
                </label>
                <div>
                  <For each={[
                    ["all", "全部"],
                    ["ready", "可进入评级"],
                    ["review", "需要复核"],
                    ["unavailable", "无法估值"],
                  ] as Array<[Filter, string]>}>
                    {([id, label]) => <button type="button" classList={{ "is-active": filter() === id }} onClick={() => setFilter(id)}>{label}</button>}
                  </For>
                </div>
                <button type="button" class="valuation-lab-refresh" disabled={loading()} onClick={() => void load()}>{loading() ? "读取中…" : "重新读取"}</button>
              </section>

              <Show when={view() !== "error"} fallback={<section class="valuation-lab-empty"><strong>暂时无法读取估值</strong><span>{error()}</span><button type="button" onClick={() => void load()}>重试</button></section>}>
                <Show when={visible().length > 0} fallback={<section class="valuation-lab-empty"><strong>没有符合条件的公司</strong><span>缺失数据不会被当成零，也不会沿用旧目标价。</span></section>}>
                  <section class="valuation-lab-list">
                    <For each={visible()}>{(item) => (
                      <details class={`valuation-lab-card is-${item.status}`}>
                        <summary>
                          <span class="valuation-lab-status">{statusLabel(item)}</span>
                          <span class="valuation-lab-company"><strong>{item.symbol}</strong><small>{item.name} · {item.theme}</small></span>
                          <span class="valuation-lab-position"><strong>{money(item.current_price, item.currency)}</strong><small>{item.current_position}</small></span>
                          <Show when={item.scenarios.length === 3} fallback={<span class="valuation-lab-no-value">—</span>}>
                            <span class="valuation-lab-range"><b>{money(item.scenarios[0].fair_value, item.currency)}</b><i>—</i><b>{money(item.scenarios[2].fair_value, item.currency)}</b></span>
                          </Show>
                          <span class="valuation-lab-chevron">›</span>
                        </summary>
                        <div class="valuation-lab-detail">
                          <section class="valuation-readiness" aria-label="估值输入准备度">
                            <header>
                              <div>
                                <span>输入准备度</span>
                                <strong>{valuationReviewLabel(item.readiness.valuation_review_status)}</strong>
                                <small>{financialReviewLabel(item.readiness.financial_review_status)}</small>
                              </div>
                              <p>{item.readiness.scope}</p>
                            </header>
                            <div class="valuation-readiness__gates">
                              <span classList={{ "is-ready": item.current_price != null }}>行情 {item.current_price == null ? "缺失" : "已有"}</span>
                              <span classList={{ "is-ready": item.readiness.available_inputs.length > 1 }}>SEC 财务 {item.readiness.available_inputs.length > 1 ? "已有" : "缺失"}</span>
                              <span classList={{ "is-ready": item.readiness.rating_factor_authorized }}>评级财务 {item.readiness.rating_factor_authorized ? "已授权" : "未授权"}</span>
                              <span title="评级财务复核不等于估值授权" classList={{ "is-ready": item.readiness.sec_valuation_use_authorized }}>SEC 估值用途 {item.readiness.sec_valuation_use_authorized ? "已授权" : "未授权"}</span>
                            </div>
                            <div class="valuation-readiness__methods">
                              <For each={item.readiness.methods}>{(method) => (
                                <article classList={{ "is-ready": method.status === "prepared" }}>
                                  <strong>{method.label}</strong>
                                  <span>{method.status === "prepared" ? "输入已准备" : "暂不可计算"}</span>
                                  <small>{method.missing_inputs.length > 0 ? `缺：${method.missing_inputs.join("、")}` : "仍需通过情景与离散度校验"}</small>
                                </article>
                              )}</For>
                            </div>
                            <Show when={item.readiness.missing_inputs.length > 0}>
                              <div class="valuation-readiness__missing"><strong>仍缺少</strong><ul><For each={item.readiness.missing_inputs}>{(missing) => <li>{missing}</li>}</For></ul></div>
                            </Show>
                          </section>
                          <Show when={item.scenarios.length === 3} fallback={<div class="valuation-lab-warning"><strong>本日不生成估值</strong><p>{item.unavailable_reason}</p></div>}>
                            <div class="valuation-lab-scenarios">
                              <For each={item.scenarios}>{(scenario) => (
                                <article class={`is-${scenario.id}`}>
                                  <span>{scenario.label} · 概率 {percent(scenario.probability)}</span>
                                  <strong>{money(scenario.fair_value, item.currency)}</strong>
                                  <div class="valuation-lab-method-results">
                                    <For each={scenario.methods}>{(method) => (
                                      <div><b>{method.label}</b><span>{money(method.value, item.currency)}</span><small>权重 {percent(method.weight)} · {method.metric}</small></div>
                                    )}</For>
                                  </div>
                                  <p>现金流起始变化 {percent(scenario.initial_growth_rate)} · 折现 {percent(scenario.discount_rate)} · 终值 {percent(scenario.terminal_growth_rate)}</p>
                                </article>
                              )}</For>
                            </div>
                            <div class="valuation-lab-analysis-grid">
                              <article>
                                <span>概率加权价值</span>
                                <strong>{money(item.probability_weighted_value, item.currency)} · {pointPercent(item.expected_upside_percent)}</strong>
                                <p>三情景按 20% / 55% / 25% 汇总，并作为公司评级的估值输入。</p>
                              </article>
                              <article>
                                <span>当前股价反向估值</span>
                                <strong>{item.reverse_dcf?.implied_forward_pe == null ? percent(item.reverse_dcf?.implied_growth_rate) : `${item.reverse_dcf.implied_forward_pe.toFixed(1)}x P/E`}</strong>
                                <p>{item.reverse_dcf?.explanation}</p>
                              </article>
                              <article>
                                <span>方法交叉验证</span>
                                <strong>{item.cross_check?.dispersion_percent == null ? "未完成" : `${item.cross_check.method_count} 种方法 · 离散 ${item.cross_check.dispersion_percent.toFixed(1)}%`}</strong>
                                <p>{item.cross_check?.explanation}</p>
                              </article>
                              <article>
                                <span>适用模型</span>
                                <strong>{item.method}</strong>
                                <p>公司研究卡建议：{item.company_method_hint}</p>
                              </article>
                            </div>
                            <Show when={item.unavailable_reason}><div class="valuation-lab-warning"><strong>暂不进入评级</strong><p>{item.unavailable_reason}</p></div></Show>
                          </Show>

                          <div class="valuation-lab-audit-grid">
                            <section><h3>模型假设</h3><ul><For each={item.assumptions}>{(assumption) => <li>{assumption}</li>}</For></ul></section>
                            <section><h3>数据与来源</h3><div class="valuation-lab-evidence"><For each={item.evidence.length > 0 ? item.evidence : item.readiness.available_inputs}>{(evidence) => <a href={evidence.source_url} target="_blank" rel="noreferrer"><strong>{evidence.label}</strong><span>{evidence.display_value}</span><small>{evidence.source} · {evidence.as_of}</small></a>}</For></div></section>
                          </div>
                        </div>
                      </details>
                    )}</For>
                  </section>
                </Show>
              </Show>

              <footer class="valuation-lab-footer">
                <span>报告日 {snapshot()?.report_date} · 北京时间 {snapshot()?.generated_at_beijing} 更新</span>
                <p>{snapshot()?.disclaimer}</p>
              </footer>
            </main>
          </PublicWorkspaceShell>
        </Show>
      </Show>
    </>
  );
}
