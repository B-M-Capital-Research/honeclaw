import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { getPublicWeeklyBrief } from "@/lib/api";
import { ResearchPanel, ResearchPanelHead } from "@/components/research/research-panel";
import { ResearchState } from "@/components/research/research-state";
import type { WeeklyBriefItem, WeeklyBriefPayload } from "@/lib/types";
import "./weekly-brief-dashboard.css";

type Props = {
  onClose: () => void;
};

type BriefView = "last" | "next" | "ai";

const categoryLabel = (value: string) => ({
  policy: "央行政策",
  inflation: "通胀",
  labor: "就业",
  growth: "增长",
  macro: "宏观",
  earnings: "公司财报",
  industry: "产业里程碑",
  ai_conference: "AI 产业会议",
}[value] ?? "重要事件");

const evidenceLabel = (value: string) => ({
  confirmed: "一手确认",
  official_schedule: "官网已确认",
  schedule_passed: "日程已发生 · 结果待核验",
  scheduled: "未来日程 · 日期或调整",
}[value] ?? "证据待核验");

const statusLabel = (value: string) => ({
  live: "日历与财报覆盖已更新",
  partial: "部分数据待补齐",
  empty: "本期暂无可核验事件",
}[value] ?? "读取中");

/** Traffic light for the panel head: earnings coverage gates the green light. */
function statusSignal(report: WeeklyBriefPayload) {
  if (report.earnings_status !== "ok") return "yellow";
  if (report.status === "empty") return "orange";
  return report.status === "live" ? "green" : "yellow";
}

function groupByDate(items: readonly WeeklyBriefItem[]) {
  const grouped = new Map<string, WeeklyBriefItem[]>();
  for (const item of items) {
    const current = grouped.get(item.date) ?? [];
    current.push(item);
    grouped.set(item.date, current);
  }
  return [...grouped.entries()].map(([date, events]) => ({ date, events }));
}

function shortDate(value: string) {
  const [, month, day] = value.split("-");
  return `${Number(month)}月${Number(day)}日`;
}

/**
 * A week of dated events.
 *
 * Deliberately not a social feed: these rows have no author and no authored
 * text, and the reader's index into them is the date — which day CPI lands,
 * which day a company reports. Feeding them would force the event title into
 * the author slot and repeat the date on every row instead of once per day.
 * So the agenda keeps its date column and loses its chrome instead.
 */
function AgendaPanel(props: { title: string; range: string; items: WeeklyBriefItem[] }) {
  const groups = createMemo(() => groupByDate(props.items));
  return (
    <section class="weekly-brief-week">
      {/* The tab bar above already names this view, so the header is the
          title and the dates it covers — the flavour kicker is gone. */}
      <header>
        <h3>{props.title}</h3>
        <time>{props.range}</time>
      </header>
      <Show
        when={groups().length > 0}
        fallback={
          <ResearchState
            kind="empty"
            message="本周暂无可核验条目"
            detail="没有可靠日期就不补造事件；可稍后刷新查看。"
          />
        }
      >
        <div class="weekly-brief-agenda">
          <For each={groups()}>
            {(group) => (
              <section class="weekly-brief-day">
                <div class="weekly-brief-date">
                  <strong>{shortDate(group.date)}</strong>
                  <span>{group.events[0]?.weekday}</span>
                </div>
                <div class="weekly-brief-events">
                  <For each={group.events}>
                    {(item) => (
                      <article>
                        <h4>
                          <Show when={item.ticker}><code>{item.ticker}</code></Show>
                          {item.title}
                        </h4>
                        {/* Whether it matters, how well it is evidenced and
                            what kind of event it is used to be three badges of
                            competing weight plus a coloured left bar. They are
                            one quiet line; only 重点 keeps a marker. */}
                        <p class="weekly-brief-event-meta">
                          <Show when={item.importance === "high"}><b>重点</b></Show>
                          <span>
                            {[evidenceLabel(item.evidence_status), categoryLabel(item.category)]
                              .join(" · ")}
                          </span>
                        </p>
                        <Show when={item.subtitle}>
                          <p class="weekly-brief-subtitle">{item.subtitle}</p>
                        </Show>
                        {/* Never folded and never clipped: the reading of the
                            event is two sentences, and a disclosure control
                            around two sentences is chrome nobody asked for. */}
                        <p class="weekly-brief-analysis">{item.analysis}</p>
                        <p class="weekly-brief-attention">
                          <strong>提醒关注：</strong>
                          {item.attention}
                        </p>
                        <div class="weekly-brief-evidence">
                          <span>{item.evidence_note}</span>
                          <Show when={item.source_url} fallback={<small>{item.source_name}</small>}>
                            {(url) => (
                              <a href={url()!} target="_blank" rel="noreferrer">{item.source_name} ↗</a>
                            )}
                          </Show>
                        </div>
                      </article>
                    )}
                  </For>
                </div>
              </section>
            )}
          </For>
        </div>
      </Show>
    </section>
  );
}

export function WeeklyBriefPanel(props: Props) {
  const [error, setError] = createSignal("");
  const [report, setReport] = createSignal<WeeklyBriefPayload>();
  const [activeView, setActiveView] = createSignal<BriefView>("next");
  let controller: AbortController | undefined;

  const load = async () => {
    controller?.abort();
    controller = new AbortController();
    setError("");
    try {
      setReport(await getPublicWeeklyBrief(controller.signal));
    } catch (value) {
      if (value instanceof Error && value.name === "AbortError") return;
      setError(value instanceof Error ? value.message : "周度简报暂时无法加载");
    }
  };

  // The panel only mounts when it opens, so mounting keeps the original
  // "load on open" semantics.
  onMount(() => void load());
  onCleanup(() => controller?.abort());

  // Report day, generation clock and tracked-company scope used to occupy a
  // metadata strip plus a hero card above the tabs; they are provenance, so
  // they collapse into the head's secondary line.
  const metaLine = createMemo(() => {
    const current = report();
    if (!current) return undefined;
    return [
      `报告日 ${current.report_date}`,
      current.generated_at_local
        ? [current.generated_at_local, current.timezone].filter(Boolean).join(" ")
        : undefined,
      `跟踪 ${current.earnings_scope_count} 家公司`,
    ]
      .filter(Boolean)
      .join(" · ");
  });

  return (
    <ResearchPanel
      onClose={props.onClose}
      labelledBy="weekly-brief-title"
      backdropClass="weekly-brief-backdrop"
      dialogClass="weekly-brief-dialog"
    >
      <>
        <ResearchPanelHead
          id="weekly-brief-title"
          kicker="每周决策日程"
          title="周度简报"
          headline={report() ? `下周 ${report()!.next_week_items.length} 件` : undefined}
          signal={report() ? statusSignal(report()!) : undefined}
          signalLabel={
            report()
              ? report()!.earnings_status === "ok"
                ? statusLabel(report()!.status)
                : "财报覆盖待补齐"
              : undefined
          }
          summary={report()?.summary}
          meta={metaLine()}
          onClose={props.onClose}
        />
        <Show
          when={report()}
          fallback={
            <Show
              when={!error()}
              fallback={
                <ResearchState kind="error" message="读取失败" detail={error()} onRetry={() => void load()} />
              }
            >
              <ResearchState kind="loading" message="正在整理两周事件…" />
            </Show>
          }
        >
          {(current) => (
            <>
              <Show when={current().earnings_status !== "ok"}>
                <div class="weekly-brief-coverage" role="status">
                  <span>{current().errors[0] ?? "当前数据源没有返回全部重点公司财报日期；缺失日期不会被猜测补全。"}</span>
                </div>
              </Show>
              <nav class="weekly-brief-tabs" aria-label="周度简报视图">
                <button classList={{ active: activeView() === "last" }} onClick={() => setActiveView("last")}>
                  <span>上周复盘</span>
                  <b>{current().last_week_items.length}</b>
                </button>
                <button classList={{ active: activeView() === "next" }} onClick={() => setActiveView("next")}>
                  <span>下周关注</span>
                  <b>{current().next_week_items.length}</b>
                </button>
                <button classList={{ active: activeView() === "ai" }} onClick={() => setActiveView("ai")}>
                  <span>未来30天 AI</span>
                  <b>{current().ai_outlook_items.length}</b>
                </button>
              </nav>
              <main class="weekly-brief-content">
                <Show when={activeView() === "last"}>
                  <AgendaPanel
                    title="上周重要事项"
                    range={`${current().previous_week.start} — ${current().previous_week.end}`}
                    items={current().last_week_items}
                  />
                </Show>
                <Show when={activeView() === "next"}>
                  <AgendaPanel
                    title="下周重要事件点"
                    range={`${current().next_week.start} — ${current().next_week.end}`}
                    items={current().next_week_items}
                  />
                </Show>
                <Show when={activeView() === "ai"}>
                  <AgendaPanel
                    title="重要 AI 公司财报与产业会议"
                    range={`${current().ai_outlook.start} — ${current().ai_outlook.end}`}
                    items={current().ai_outlook_items}
                  />
                </Show>
                {/* Methodology is a footnote, not chrome: it scrolls with the
                    agenda instead of holding a fixed band above the composer. */}
                <div class="weekly-brief-method">
                  <strong>口径：</strong>
                  {current().methodology_note}
                </div>
              </main>
              <footer class="weekly-brief-footer">
                <p>{current().disclaimer}</p>
              </footer>
            </>
          )}
        </Show>
      </>
    </ResearchPanel>
  );
}
