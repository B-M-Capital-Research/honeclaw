import { Title } from "@solidjs/meta";
import { useNavigate } from "@solidjs/router";
import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";

import { PublicLoginForm } from "@/components/public-login-form";
import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import { ApiError, getPublicAuthMe, getPublicValuationLab } from "@/lib/api";
import {
  cachedPublicUser,
  setCachedPublicUser,
} from "@/lib/public-session-cache";
import type {
  PublicAuthUserInfo,
  ValuationLabItem,
  ValuationLabSnapshot,
} from "@/lib/types";

import "./public-foundation.css";
import "./public-site.css";
import "./public-polish.css";
import "./public-valuation-lab.css";

type ViewState = "loading" | "ready" | "login" | "forbidden" | "error";
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

export default function PublicValuationLabPage() {
  const navigate = useNavigate();
  const [user, setUser] = createSignal<PublicAuthUserInfo | null>(cachedPublicUser());
  const [view, setView] = createSignal<ViewState>(
    cachedPublicUser()?.is_admin ? "loading" : cachedPublicUser() ? "forbidden" : "loading",
  );
  const [snapshot, setSnapshot] = createSignal<ValuationLabSnapshot>();
  const [filter, setFilter] = createSignal<Filter>("all");
  const [query, setQuery] = createSignal("");
  const [error, setError] = createSignal("");
  let controller: AbortController | undefined;

  const bootstrap = async () => {
    try {
      const me = await getPublicAuthMe();
      setUser(me);
      setCachedPublicUser(me);
      if (!me.is_admin) {
        setView("forbidden");
        return;
      }
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      setUser(null);
      setCachedPublicUser(null);
      setView("login");
      return;
    }
    await load();
  };

  const load = async () => {
    if (user()?.is_admin !== true) {
      setView("forbidden");
      return;
    }
    controller?.abort();
    controller = new AbortController();
    setError("");
    try {
      setSnapshot(await getPublicValuationLab(controller.signal));
      setView("ready");
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      if (cause instanceof ApiError && cause.status === 401) setView("login");
      else if (cause instanceof ApiError && cause.status === 403) setView("forbidden");
      else {
        setError(cause instanceof Error ? cause.message : String(cause));
        setView("error");
      }
    }
  };

  onMount(() => void bootstrap());
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
          fallback={<PublicLoginForm title="登录后查看估值实验室" subtitle="估值区间只用于研究，不会自动执行交易。" onLogin={() => void bootstrap()} />}
        >
          <PublicWorkspaceShell active="research" topbarLabel="估值实验室">
            <Show
              when={view() !== "forbidden"}
              fallback={
                <main class="valuation-lab-page">
                  <section class="valuation-lab-empty">
                    <strong>暂未对全部用户开放</strong>
                    <span>估值实验室先放在研究台的管理分类里，仅管理员可见。</span>
                    <button type="button" onClick={() => navigate("/research")}>返回研究台</button>
                  </section>
                </main>
              }
            >
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
                            <section><h3>数据与来源</h3><div class="valuation-lab-evidence"><For each={item.evidence}>{(evidence) => <a href={evidence.source_url} target="_blank" rel="noreferrer"><strong>{evidence.label}</strong><span>{evidence.display_value}</span><small>{evidence.source} · {evidence.as_of}</small></a>}</For></div></section>
                          </div>
                        </div>
                      </details>
                    )}</For>
                  </section>
                </Show>
              </Show>

              <footer class="valuation-lab-footer">
                <span>报告日 {snapshot()?.report_date} · {snapshot()?.timezone} {snapshot()?.generated_at_local} 更新</span>
                <p>{snapshot()?.disclaimer}</p>
              </footer>
            </main>
            </Show>
          </PublicWorkspaceShell>
        </Show>
      </Show>
    </>
  );
}
