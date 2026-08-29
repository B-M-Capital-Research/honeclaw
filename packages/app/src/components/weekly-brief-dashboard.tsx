import { For, Show, createMemo, createSignal, onCleanup } from "solid-js";
import { Portal } from "solid-js/web";
import { getPublicWeeklyBrief } from "@/lib/api";
import type { WeeklyBriefItem, WeeklyBriefPayload } from "@/lib/types";
import "./weekly-brief-dashboard.css";
import "./model-analysis-health.css";

type Props = { onAsk: (message: string) => void };
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

function AgendaPanel(props: {
  title: string;
  range: string;
  kicker: string;
  tone: "review" | "outlook" | "ai";
  items: WeeklyBriefItem[];
}) {
  const groups = createMemo(() => groupByDate(props.items));
  return <section class="weekly-brief-week" data-tone={props.tone}>
    <header>
      <div><span>{props.kicker}</span><h3>{props.title}</h3></div>
      <time>{props.range}</time>
    </header>
    <Show when={groups().length > 0} fallback={<div class="weekly-brief-empty"><strong>本周暂无可核验条目</strong><p>没有可靠日期就不补造事件；可稍后刷新查看。</p></div>}>
      <div class="weekly-brief-agenda"><For each={groups()}>{group => <section class="weekly-brief-day">
        <div class="weekly-brief-date"><strong>{shortDate(group.date)}</strong><span>{group.events[0]?.weekday}</span></div>
        <div class="weekly-brief-events"><For each={group.events}>{item => <article data-category={item.category} data-importance={item.importance}>
          <div class="weekly-brief-event-meta">
            <span>{categoryLabel(item.category)}</span>
            <Show when={item.importance === "high"}><b>重点</b></Show>
            <em>{evidenceLabel(item.evidence_status)}</em>
            <Show when={(item.deduplicated_source_count ?? 0) > 0}><em data-dedup="merged">同一事件 · {item.source_count} 个来源</em></Show>
            <Show when={item.analysis_status === "source_only"}><em data-analysis="blocked">影响待分析</em></Show>
          </div>
          <h4><Show when={item.ticker}><code>{item.ticker}</code></Show>{item.title}</h4>
          <Show when={item.subtitle}><p class="weekly-brief-subtitle">{item.subtitle}</p></Show>
          <p class="weekly-brief-analysis">{item.analysis}</p>
          <aside><strong>提醒关注：</strong>{item.attention}</aside>
          <div class="weekly-brief-evidence"><span>{item.evidence_note}</span><Show when={item.source_url} fallback={<small>{item.source_name}</small>}>{url => <a href={url()!} target="_blank" rel="noreferrer">{item.source_name} ↗</a>}</Show></div>
          <Show when={(item.deduplicated_source_count ?? 0) > 0}><details class="weekly-brief-source-cluster"><summary>查看同一事件的全部来源</summary><For each={item.supporting_sources ?? []}>{source => <a href={source.source_url} target="_blank" rel="noreferrer">{source.source_name} · {source.published_at_beijing} ↗</a>}</For></details></Show>
        </article>}</For></div>
      </section>}</For></div>
    </Show>
  </section>;
}

export function WeeklyBriefDashboard(props: Props) {
  const [open, setOpen] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");
  const [report, setReport] = createSignal<WeeklyBriefPayload>();
  const [question, setQuestion] = createSignal("");
  const [activeView, setActiveView] = createSignal<BriefView>("next");
  let controller: AbortController | undefined;

  const load = async () => {
    controller?.abort();
    controller = new AbortController();
    setLoading(true);
    setError("");
    try {
      setReport(await getPublicWeeklyBrief(controller.signal));
    } catch (value) {
      if (value instanceof Error && value.name === "AbortError") return;
      setError(value instanceof Error ? value.message : "周度简报暂时无法加载");
    } finally {
      setLoading(false);
    }
  };
  onCleanup(() => controller?.abort());

  const show = () => {
    setActiveView("next");
    setOpen(true);
    if (!report() && !loading()) void load();
  };

  const ask = () => {
    const current = report();
    const query = question().trim();
    if (!current || !query) return;
    const saved = {
      reportDate: current.report_date,
      generatedAtBJT: current.generated_at_beijing,
      previousWeek: current.previous_week,
      nextWeek: current.next_week,
      aiOutlook: current.ai_outlook,
      lastWeekItems: current.last_week_items,
      nextWeekItems: current.next_week_items,
      aiOutlookItems: current.ai_outlook_items,
      earningsStatus: current.earnings_status,
      industryAnalysisHealth: current.industry_analysis_health,
      methodologyNote: current.methodology_note,
    };
    props.onAsk(`<!-- HONE_SAVED_WEEKLY_BRIEF
${JSON.stringify(saved)}
END_HONE_SAVED_WEEKLY_BRIEF -->
基于已保存的 HONE 周度简报回答：${query}
要求：区分一手确认的产业变化、已经过去但结果尚未核验的日程、以及尚未发生的未来日程；未来日程不是预测，过去日程也不能据此补造公布值。source_count>1 只是同一事件的多个来源，不得按来源数重复加权。涉及财报或宏观结果时优先核对公司 IR、监管文件和官方数据，附来源与北京时间，再给出对基本面、估值和持仓风险的适当分析。`);
    setOpen(false);
    setQuestion("");
  };

  return <>
    <div class="weekly-brief-launcher">
      <button type="button" onClick={show}>
        <span class="weekly-brief-icon" aria-hidden="true">7</span>
        <span><strong>周度简报</strong><small>{report() ? `下周 ${report()!.next_week_items.length} 项 · 未来30天 AI ${report()!.ai_outlook_items.length} 项` : "上周复盘 · 下周宏观、AI 财报与产业会议"}</small></span>
        <b>{report() ? `${report()!.next_week_items.length}+${report()!.ai_outlook_items.length}` : "周更"}</b><i>›</i>
      </button>
    </div>
    <Show when={open()}><Portal><div class="weekly-brief-backdrop" onClick={() => setOpen(false)}>
      <section class="weekly-brief-dialog" role="dialog" aria-modal="true" aria-labelledby="weekly-brief-title" onClick={event => event.stopPropagation()}>
        <header class="weekly-brief-dialog-head"><div><p>每周决策日程</p><h2 id="weekly-brief-title">周度简报</h2><span>按时间看清上周变化、下周风险与未来 30 天 AI 节点</span></div><button aria-label="关闭周度简报" onClick={() => setOpen(false)}>×</button></header>
        <Show when={report()} fallback={<div class="weekly-brief-loading"><Show when={!error()} fallback={<><strong>读取失败</strong><p>{error()}</p><button onClick={() => void load()}>重新读取</button></>}><strong>{loading() ? "正在整理两周事件…" : "等待读取"}</strong></Show></div>}>{current => <>
          <div class="weekly-brief-meta"><b>{statusLabel(current().status)}</b><span>报告日 {current().report_date}</span><span>{current().generated_at_beijing} 北京时间</span><button onClick={() => void load()} disabled={loading()}>{loading() ? "读取中…" : "重新读取"}</button></div>
          <section class="weekly-brief-hero"><div><span>本期判断</span><h3>{current().summary}</h3></div><aside><strong>跟踪公司</strong><b>{current().earnings_scope_count}</b></aside></section>
          <Show when={current().industry_analysis_health?.decision_use_allowed === false}><div class="weekly-brief-coverage is-model-blocked" role="status"><strong>产业影响分析门禁关闭</strong><span>一手里程碑仍可阅读，但未完成的影响分析不会进入周度判断或仓位含义。</span></div></Show>
          <Show when={current().earnings_status !== "ok"}><div class="weekly-brief-coverage" role="status"><strong>财报覆盖未完全就绪</strong><span>{current().errors[0] ?? "当前数据源没有返回全部重点公司财报日期；缺失日期不会被猜测补全。"}</span></div></Show>
          <nav class="weekly-brief-tabs" aria-label="周度简报视图">
            <button classList={{ active: activeView() === "last" }} onClick={() => setActiveView("last")}><span>上周复盘</span><b>{current().last_week_items.length}</b></button>
            <button classList={{ active: activeView() === "next" }} onClick={() => setActiveView("next")}><span>下周关注</span><b>{current().next_week_items.length}</b></button>
            <button classList={{ active: activeView() === "ai" }} onClick={() => setActiveView("ai")}><span>未来30天 AI</span><b>{current().ai_outlook_items.length}</b></button>
          </nav>
          <main class="weekly-brief-content">
            <Show when={activeView() === "last"}><AgendaPanel title="上周重要事项" range={`${current().previous_week.start} — ${current().previous_week.end}`} kicker="发生了什么变化" tone="review" items={current().last_week_items} /></Show>
            <Show when={activeView() === "next"}><AgendaPanel title="下周重要事件点" range={`${current().next_week.start} — ${current().next_week.end}`} kicker="需要关注什么" tone="outlook" items={current().next_week_items} /></Show>
            <Show when={activeView() === "ai"}><AgendaPanel title="重要 AI 公司财报与产业会议" range={`${current().ai_outlook.start} — ${current().ai_outlook.end}`} kicker="AI 财报与产业事件" tone="ai" items={current().ai_outlook_items} /></Show>
          </main>
          <div class="weekly-brief-method"><strong>口径：</strong>{current().methodology_note}</div>
          <footer class="weekly-brief-footer"><p>{current().disclaimer}</p><div><input aria-label="基于周度简报提问" value={question()} onInput={event => setQuestion(event.currentTarget.value)} placeholder="例如：下周最可能影响我的持仓的是哪三件事？" /><button disabled={!question().trim()} onClick={ask}>发送到对话</button></div></footer>
        </>}</Show>
      </section>
    </div></Portal></Show>
  </>;
}
