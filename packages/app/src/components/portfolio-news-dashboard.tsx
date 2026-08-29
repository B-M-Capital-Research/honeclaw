import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { Portal } from "solid-js/web";
import { getPublicPortfolioNews } from "@/lib/api";
import type {
  PortfolioNewsImpact,
  PortfolioNewsItem,
  PortfolioNewsSnapshot,
} from "@/lib/types";
import "./portfolio-news-dashboard.css";

type Props = { onAsk: (message: string) => void };
type Filter = "all" | PortfolioNewsImpact;

const FILTERS: Array<{ id: Filter; label: string }> = [
  { id: "all", label: "全部" },
  { id: "negative", label: "负面" },
  { id: "positive", label: "正面" },
  { id: "neutral", label: "中性" },
  { id: "unassessed", label: "待分析" },
];

const IMPACT_LABEL: Record<PortfolioNewsImpact, string> = {
  positive: "正面",
  neutral: "中性",
  negative: "负面",
  unassessed: "待分析",
};

const HORIZON_LABEL: Record<PortfolioNewsItem["horizon"], string> = {
  short: "短期",
  medium: "中期",
  long: "长期",
  unknown: "期限待定",
};

function launcherCopy(snapshot?: PortfolioNewsSnapshot) {
  if (!snapshot) return "读取每日持仓新闻…";
  if (snapshot.status === "no_portfolio") return "添加持仓后自动生成";
  if (snapshot.status === "waiting_refresh") return "等待每日 20:00 首次生成";
  if (snapshot.status === "data_unavailable") return "等待新闻数据源";
  if (snapshot.status === "no_material_news") return "近 48 小时无重点新闻";
  if (snapshot.status === "source_only") return `${snapshot.counts.total} 条可信新闻 · 待模型分析`;
  return `${snapshot.counts.total} 条重点新闻 · ${snapshot.counts.immediate_review} 条需复核`;
}

function statusLabel(status: string) {
  if (status === "live") return "数据与模型已更新";
  if (status === "partial") return "部分新闻已分析";
  if (status === "source_only") return "仅来源事实";
  if (status === "no_material_news") return "无重点新闻";
  if (status === "no_portfolio") return "尚无持仓";
  if (status === "waiting_refresh") return "等待首次生成";
  if (status === "portfolio_changed") return "持仓已变化";
  if (status === "stale") return "上次成功快照";
  return "数据暂不可用";
}

function analysisHealthCopy(snapshot: PortfolioNewsSnapshot) {
  const health = snapshot.analysis_health;
  if (!health) return "旧快照缺少模型健康记录，禁止用于仓位动作";
  if (health.status === "healthy") return `模型分析完成 ${health.analyzed_items}/${health.requested_items}`;
  if (health.status === "not_required") return "本期无新闻需要模型分析";
  if (health.status === "pending") return `模型分析中 0/${health.requested_items}`;
  if (health.status === "partial") return `仅完成 ${health.analyzed_items}/${health.requested_items}，未完成项目禁止用于仓位动作`;
  if (health.status === "unconfigured") return "模型分析未配置，当前内容仅作为来源事实";
  return `模型分析不可用，${health.failed_items || health.requested_items} 条内容禁止用于仓位动作`;
}

function reportContext(snapshot: PortfolioNewsSnapshot, question: string) {
  const compact = {
    reportDate: snapshot.report_date,
    generatedAtBJT: snapshot.generated_at_beijing,
    status: snapshot.status,
    lookbackHours: snapshot.lookback_hours,
    summary: snapshot.summary,
    analysisHealth: snapshot.analysis_health,
    coveredSymbols: snapshot.covered_symbols,
    missingSymbols: snapshot.missing_symbols,
    items: snapshot.items.map((item) => ({
      symbol: item.symbol,
      title: item.title,
      publishedAtBJT: item.published_at_beijing,
      source: item.source,
      sourceUrl: item.source_url,
      impact: item.impact,
      horizon: item.horizon,
      thesisEffect: item.thesis_effect,
      summary: item.summary,
      whyItMatters: item.why_it_matters,
      attention: item.attention,
      confidence: item.confidence,
      analysisStatus: item.analysis_status,
    })),
  };
  return `<!-- HONE_SAVED_PORTFOLIO_NEWS_REPORT\n${JSON.stringify(compact)}\nEND_HONE_SAVED_PORTFOLIO_NEWS_REPORT -->\n基于已保存的持仓重点新闻分析（报告日 ${snapshot.report_date}）回答：${question}\n要求：只使用上述快照和当前对话；引用新闻时附来源链接和北京时间；明确区分来源事实与模型影响判断；待分析项目不得补造结论；数据截止时间之后的内容先说明未覆盖。不要自动修改仓位。`;
}

export function PortfolioNewsDashboard(props: Props) {
  const [snapshot, setSnapshot] = createSignal<PortfolioNewsSnapshot>();
  const [open, setOpen] = createSignal(false);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal("");
  const [filter, setFilter] = createSignal<Filter>("all");
  const [question, setQuestion] = createSignal("");
  let controller: AbortController | undefined;

  const load = async () => {
    controller?.abort();
    controller = new AbortController();
    setLoading(true);
    setError("");
    try {
      setSnapshot(await getPublicPortfolioNews(controller.signal));
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      setError(cause instanceof Error ? cause.message : "持仓新闻暂时无法加载");
    } finally {
      setLoading(false);
    }
  };

  onMount(() => void load());
  onCleanup(() => controller?.abort());
  createEffect(() => {
    if (!open()) return;
    const onKey = (event: KeyboardEvent) => event.key === "Escape" && setOpen(false);
    document.addEventListener("keydown", onKey);
    onCleanup(() => document.removeEventListener("keydown", onKey));
  });

  const visible = createMemo(() => {
    const items = snapshot()?.items ?? [];
    return filter() === "all" ? items : items.filter((item) => item.impact === filter());
  });
  const sendQuestion = () => {
    const current = snapshot();
    const text = question().trim();
    if (!current || !text) return;
    props.onAsk(reportContext(current, text));
    setQuestion("");
    setOpen(false);
  };

  return (
    <>
      <div class="portfolio-news-launcher">
        <button type="button" onClick={() => setOpen(true)} aria-haspopup="dialog">
          <span class="portfolio-news-launcher__icon" aria-hidden="true">
            <svg viewBox="0 0 24 24"><path d="M5 4h14v16H5zM8 8h8M8 12h8M8 16h5" /></svg>
          </span>
          <span class="portfolio-news-launcher__copy">
            <strong>持仓重点新闻分析</strong>
            <small>{loading() ? "读取每日持仓新闻…" : launcherCopy(snapshot())}</small>
          </span>
          <Show when={(snapshot()?.counts.immediate_review ?? 0) > 0}>
            <b class="portfolio-news-launcher__alert">{snapshot()?.counts.immediate_review}</b>
          </Show>
          <svg class="portfolio-news-chevron" viewBox="0 0 24 24"><path d="m9 18 6-6-6-6" /></svg>
        </button>
      </div>

      <Show when={open()}>
        <Portal>
          <div class="portfolio-news-backdrop" onClick={() => setOpen(false)}>
            <section
              class="portfolio-news-dialog"
              role="dialog"
              aria-modal="true"
              aria-labelledby="portfolio-news-title"
              onClick={(event) => event.stopPropagation()}
            >
              <header>
                <div>
                  <p>持仓情报分析</p>
                  <h2 id="portfolio-news-title">持仓重点新闻分析</h2>
                  <span>近 48 小时可信来源 · 模型影响判断 · 每日 20:00 更新</span>
                </div>
                <button type="button" aria-label="关闭持仓新闻" onClick={() => setOpen(false)}>
                  <svg viewBox="0 0 24 24"><path d="M18 6 6 18M6 6l12 12" /></svg>
                </button>
              </header>

              <Show when={snapshot()}>
                {(value) => (
                  <div class="portfolio-news-meta">
                    <b class={`is-${value().status}`}>{statusLabel(value().status)}</b>
                    <span>报告日 {value().report_date}</span>
                    <span>{value().generated_at_beijing} 北京时间</span>
                    <span>持仓 {value().holdings_count}</span>
                    <button type="button" onClick={() => void load()} disabled={loading()}>
                      {loading() ? "读取中…" : "重新读取"}
                    </button>
                  </div>
                )}
              </Show>

              <Show when={snapshot()}>
                {(value) => (
                  <div
                    class="portfolio-news-analysis-health"
                    classList={{ "is-blocked": value().analysis_health?.decision_use_allowed === false }}
                    role="status"
                  >
                    <b>{value().analysis_health?.decision_use_allowed ? "分析门禁通过" : "分析门禁关闭"}</b>
                    <span>{analysisHealthCopy(value())}</span>
                    <Show when={value().analysis_health?.model}>
                      <small>
                        {value().analysis_health?.profile_name || value().analysis_health?.provider_name}
                        {" · "}{value().analysis_health?.model}
                      </small>
                    </Show>
                  </div>
                )}
              </Show>

              <div class="portfolio-news-summary">
                <div>
                  <strong>{snapshot()?.counts.total ?? 0}</strong><span>重点新闻</span>
                </div>
                <div class="is-negative">
                  <strong>{snapshot()?.counts.negative ?? 0}</strong><span>负面</span>
                </div>
                <div class="is-positive">
                  <strong>{snapshot()?.counts.positive ?? 0}</strong><span>正面</span>
                </div>
                <div class="is-review">
                  <strong>{snapshot()?.counts.immediate_review ?? 0}</strong><span>立即复核</span>
                </div>
                <p>{snapshot()?.summary ?? (error() || "正在读取当天快照…")}</p>
              </div>

              <Show when={(snapshot()?.coverage_items?.length ?? 0) > 0}>
                <div class="portfolio-news-coverage" aria-label="持仓新闻覆盖状态">
                  <For each={snapshot()?.coverage_items ?? []}>
                    {(item) => (
                      <span class={`is-${item.status}`} title={item.label}>
                        <b>{item.symbol}</b>{item.label}
                      </span>
                    )}
                  </For>
                </div>
              </Show>

              <div class="portfolio-news-filters" role="group" aria-label="按新闻影响筛选">
                <For each={FILTERS}>
                  {(item) => (
                    <button
                      type="button"
                      classList={{ "is-active": filter() === item.id }}
                      aria-pressed={filter() === item.id}
                      onClick={() => setFilter(item.id)}
                    >
                      {item.label}
                    </button>
                  )}
                </For>
              </div>

              <div class="portfolio-news-body">
                <Show
                  when={!error()}
                  fallback={<div class="portfolio-news-empty is-error">{error()}</div>}
                >
                  <Show
                    when={visible().length > 0}
                    fallback={
                      <div class="portfolio-news-empty">
                        <strong>{snapshot()?.status === "no_portfolio" ? "先添加持仓" : "当前没有可展示的重点新闻"}</strong>
                        <p>{snapshot()?.summary}</p>
                        <Show when={(snapshot()?.missing_symbols.length ?? 0) > 0}>
                          <small>未发现重点新闻：{snapshot()?.missing_symbols.join("、")}</small>
                        </Show>
                      </div>
                    }
                  >
                    <div class="portfolio-news-list">
                      <For each={visible()}>
                        {(item) => (
                          <article class={`is-${item.impact}`}>
                            <div class="portfolio-news-item__top">
                              <b>{item.symbol}</b>
                              <span class={`is-${item.impact}`}>{IMPACT_LABEL[item.impact]}</span>
                              <span>{HORIZON_LABEL[item.horizon]}</span>
                              <time>{item.published_at_beijing} 北京时间</time>
                            </div>
                            <h3>{item.title}</h3>
                            <p class="portfolio-news-item__summary">{item.summary}</p>
                            <p class="portfolio-news-item__why"><strong>为什么重要：</strong>{item.why_it_matters}</p>
                            <div class="portfolio-news-item__foot">
                              <span classList={{ "is-urgent": item.attention === "立即复核" }}>{item.attention}</span>
                              <span>置信度 {item.confidence === "high" ? "高" : item.confidence === "medium" ? "中" : "低"}</span>
                              <Show when={item.source_url} fallback={<span>{item.source}</span>}>
                                <a href={item.source_url} target="_blank" rel="noreferrer">{item.source} · 查看原文</a>
                              </Show>
                            </div>
                          </article>
                        )}
                      </For>
                    </div>
                  </Show>
                </Show>
              </div>

              <footer>
                <div>
                  <input
                    value={question()}
                    onInput={(event) => setQuestion(event.currentTarget.value)}
                    onKeyDown={(event) => event.key === "Enter" && sendQuestion()}
                    placeholder="基于这份持仓新闻继续提问…"
                    aria-label="基于持仓新闻提问"
                  />
                  <button type="button" disabled={!question().trim() || !snapshot()} onClick={sendQuestion}>发送到对话</button>
                </div>
                <p>{snapshot()?.disclaimer ?? "新闻影响分析用于研究提醒，不构成投资建议。"}</p>
              </footer>
            </section>
          </div>
        </Portal>
      </Show>
    </>
  );
}
