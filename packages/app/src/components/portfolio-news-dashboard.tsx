import {
  For,
  Show,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { getPublicPortfolioNews } from "@/lib/api";
import {
  ResearchLongform,
  ResearchPanel,
  ResearchPanelHead,
  shortLocalTimestamp,
} from "@/components/research/research-panel";
import { ResearchState } from "@/components/research/research-state";
import { buildSavedReportPrompt } from "@/lib/saved-report-prompt";
import type {
  PortfolioNewsImpact,
  PortfolioNewsItem,
  PortfolioNewsSnapshot,
} from "@/lib/types";
import "./portfolio-news-dashboard.css";

type Props = {
  onClose: () => void;
  onAsk?: (message: string) => void;
};
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

function statusLabel(status: string) {
  if (status === "live") return "数据与模型已更新";
  if (status === "partial") return "部分新闻已分析";
  if (status === "source_only") return "仅来源事实 · 待模型分析";
  if (status === "no_material_news") return "无重点新闻";
  if (status === "no_portfolio") return "尚无持仓";
  if (status === "waiting_refresh") return "等待首次生成";
  if (status === "portfolio_changed") return "持仓已变化";
  if (status === "stale") return "上次成功快照";
  if (status === "data_unavailable") return "等待新闻数据源";
  return "数据暂不可用";
}

/**
 * The head shows one sentence; the count strip shows whatever else the summary
 * said. Truncating would hide coverage caveats, and printing a whole paragraph
 * in the head is what made the panel read as undifferentiated — so the lead
 * sentence goes above and the remainder stays below, each shown exactly once.
 */
function leadSentence(summary: string) {
  return summary.match(/^[\s\S]*?[。！？!?]/)?.[0].trim() ?? summary.trim();
}

function trailingDetail(summary: string) {
  return summary.slice(leadSentence(summary).length).trim();
}

/** Traffic-light key for the status badge the head now carries. */
function statusSignal(status: string) {
  if (status === "live" || status === "no_material_news") return "green";
  if (status === "partial" || status === "source_only" || status === "portfolio_changed") {
    return "yellow";
  }
  if (status === "stale" || status === "data_unavailable") return "orange";
  return undefined;
}

/**
 * Provenance collapsed to one line under the verdict. A field the snapshot did
 * not carry is dropped rather than printed as a hole.
 */
function provenanceLine(snapshot: PortfolioNewsSnapshot) {
  return [
    `报告日 ${snapshot.report_date}`,
    snapshot.generated_at_local
      ? `生成 ${[snapshot.generated_at_local, snapshot.timezone].filter(Boolean).join(" ")}`
      : "",
    `持仓 ${snapshot.holdings_count}`,
    snapshot.lookback_hours ? `回溯 ${snapshot.lookback_hours} 小时` : "",
    snapshot.model_version,
  ]
    .filter(Boolean)
    .join(" · ");
}

function reportContext(snapshot: PortfolioNewsSnapshot, question: string) {
  const compact = {
    reportDate: snapshot.report_date,
    generatedAtLocal: snapshot.generated_at_local,
    status: snapshot.status,
    lookbackHours: snapshot.lookback_hours,
    summary: snapshot.summary,
    coveredSymbols: snapshot.covered_symbols,
    missingSymbols: snapshot.missing_symbols,
    items: snapshot.items.map((item) => ({
      symbol: item.symbol,
      title: item.title,
      publishedAtLocal: item.published_at_local,
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
  return buildSavedReportPrompt({
    marker: "HONE_SAVED_PORTFOLIO_NEWS_REPORT",
    payload: compact,
    subject: `已保存的持仓重点新闻分析（报告日 ${snapshot.report_date}）`,
    question,
    rules: [
      "只使用上述快照和当前对话",
      "引用新闻时附来源链接和运行时时区",
      "明确区分来源事实与模型影响判断",
      "待分析项目不得补造结论",
      "数据截止时间之后的内容先说明未覆盖",
      "不要自动修改仓位",
    ],
  });
}

export function PortfolioNewsPanel(props: Props) {
  const [snapshot, setSnapshot] = createSignal<PortfolioNewsSnapshot>();
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

  // Panel mounts only while open, so loading on mount means "fetch when the
  // panel is opened" — one cached snapshot read, no recomputation.
  onMount(() => void load());
  onCleanup(() => controller?.abort());

  const visible = createMemo(() => {
    const items = snapshot()?.items ?? [];
    return filter() === "all" ? items : items.filter((item) => item.impact === filter());
  });

  const emptyDetail = () => {
    const current = snapshot();
    if (!current) return undefined;
    const missing = current.missing_symbols.length
      ? `未发现重点新闻：${current.missing_symbols.join("、")}`
      : "";
    return [current.summary, missing].filter(Boolean).join(" ") || undefined;
  };

  const sendQuestion = () => {
    const current = snapshot();
    const text = question().trim();
    if (!current || !text || !props.onAsk) return;
    props.onAsk(reportContext(current, text));
    setQuestion("");
    props.onClose();
  };

  return (
    <ResearchPanel
      onClose={props.onClose}
      labelledBy="portfolio-news-title"
      backdropClass="portfolio-news-backdrop"
      dialogClass="portfolio-news-dialog"
    >
      <>
        <ResearchPanelHead
          id="portfolio-news-title"
          kicker="持仓情报分析"
          title="持仓重点新闻分析"
          headline={
            // With no holdings there is nothing to have found; "0 条" would
            // read as a scanned-and-clear result rather than an empty portfolio.
            snapshot()?.holdings_count ? `${snapshot()!.counts.total} 条` : undefined
          }
          signal={snapshot() ? statusSignal(snapshot()!.status) : undefined}
          signalLabel={snapshot() ? statusLabel(snapshot()!.status) : undefined}
          summary={
            snapshot()
              ? leadSentence(snapshot()!.summary)
              : "近 48 小时可信来源 · 模型影响判断 · 每日 20:00 更新"
          }
          meta={snapshot() ? provenanceLine(snapshot()!) : undefined}
          onClose={props.onClose}
          action={
            <button type="button" onClick={() => void load()} disabled={loading()}>
              {loading() ? "读取中…" : "重新读取"}
            </button>
          }
        />

        <div class="portfolio-news-summary">
          <div class="is-negative">
            <strong>{snapshot()?.counts.negative ?? 0}</strong><span>负面</span>
          </div>
          <div class="is-positive">
            <strong>{snapshot()?.counts.positive ?? 0}</strong><span>正面</span>
          </div>
          <div class="is-review">
            <strong>{snapshot()?.counts.immediate_review ?? 0}</strong><span>立即复核</span>
          </div>
          <Show when={snapshot() && trailingDetail(snapshot()!.summary)}>
            {(detail) => <p>{detail()}</p>}
          </Show>
        </div>

        <div class="portfolio-news-filters research-scroller" role="group" aria-label="按新闻影响筛选">
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
          <Show when={loading() && !snapshot()}>
            <ResearchState
              kind="loading"
              message="正在读取持仓重点新闻"
              detail="只读取当日快照，不会触发重新分析。"
            />
          </Show>
          <Show when={error() && !snapshot()}>
            <ResearchState
              kind="error"
              message="持仓新闻读取失败"
              detail={error()}
              onRetry={() => void load()}
            />
          </Show>
          <Show when={snapshot()}>
            <Show
              when={visible().length > 0}
              fallback={
                <ResearchState
                  kind="empty"
                  message={snapshot()?.status === "no_portfolio" ? "先添加持仓" : "当前没有可展示的重点新闻"}
                  detail={emptyDetail()}
                />
              }
            >
              <div class="portfolio-news-list">
                <For each={visible()}>
                  {(item) => (
                    <article class={`is-${item.impact}`}>
                      {/* One chip — the impact call. Horizon is a qualifier of
                          that call, not a peer of it, and the run timezone is
                          already in the head's provenance line. */}
                      <div class="portfolio-news-item__top">
                        <b>{item.symbol}</b>
                        <span class={`is-${item.impact}`}>{IMPACT_LABEL[item.impact]}</span>
                        <small>{HORIZON_LABEL[item.horizon]}</small>
                        <time>{shortLocalTimestamp(item.published_at_local)}</time>
                      </div>
                      <h3>{item.title}</h3>
                      <ResearchLongform class="portfolio-news-item__summary" text={item.summary} />
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

        <Show when={props.onAsk}>
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
        </Show>
      </>
    </ResearchPanel>
  );
}
