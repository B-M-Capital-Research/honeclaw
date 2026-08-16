import {
  For,
  Show,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import {
  getPublicDailySignal,
  getPublicDailySignalHistory,
} from "@/lib/api";
import {
  ResearchLongform,
  ResearchPanel,
  ResearchPanelHead,
} from "@/components/research/research-panel";
import { ResearchState } from "@/components/research/research-state";
import { buildSavedReportPrompt } from "@/lib/saved-report-prompt";
import type {
  DailySignalHistoryItem,
  DailySignalKind,
  DailySignalReport,
  DailySignalTrendPoint,
} from "@/lib/types";
import "./daily-signal-dashboard.css";

type Props = {
  kind: DailySignalKind;
  onClose: () => void;
  onAsk?: (message: string) => void;
};

const KIND_COPY = {
  macro: {
    title: "宏观红绿灯",
    kicker: "领先周期判断",
    description: "收入 → 消费 → 生产 → 利润 → 资本开支",
  },
  ai: {
    title: "AI 红绿灯",
    kicker: "AI 增长可持续性",
    description: "需求旁证 · 商业化 · 融资 · 资本开支",
  },
} satisfies Record<DailySignalKind, Record<string, string>>;

function statusLabel(status?: string) {
  if (status === "live") return "数据完整";
  if (status === "partial") return "部分数据";
  if (status === "stale") return "沿用上次成功快照";
  return "等待数据源";
}

function effectiveStatus(report: DailySignalReport) {
  if (
    report.kind === "ai" &&
    report.status === "live" &&
    report.company_scores.some(
      (company) => company.metric_total <= 0 || company.coverage < company.metric_total,
    )
  ) {
    return "partial";
  }
  return report.status;
}

function dimensionRoleLabel(role?: string) {
  const labels: Record<string, string> = {
    leading: "领先指标",
    confirmation: "确认指标",
    lagging: "滞后指标",
    risk: "风险指标",
    ai_layer: "AI 财务层",
    financial_conditions: "金融条件",
    market_risk: "市场风险",
  };
  return role ? labels[role] ?? role.replaceAll("_", " ") : "指标";
}

function signalLabel(signal?: string) {
  if (signal === "green") return "绿灯";
  if (signal === "yellow") return "黄灯";
  if (signal === "orange") return "橙灯";
  if (signal === "red") return "红灯";
  return "待定";
}

function sourceTypeLabel(sourceType?: string) {
  if (sourceType === "reported_fact") return "已报告事实";
  if (sourceType === "model_inference") return "模型推断";
  if (sourceType === "unavailable") return "暂不可用";
  return sourceType || "数据来源";
}

function deltaLabel(value?: number | null) {
  if (value == null) return "—";
  if (value === 0) return "持平";
  return `${value > 0 ? "+" : ""}${value.toFixed(1)}`;
}

function scoreLabel(value?: number | null) {
  return value == null ? "—" : value.toFixed(1);
}

/**
 * The head shows one sentence; the body shows whatever else the summary said.
 *
 * Report summaries are not always a single line — the AI report opens with a
 * verdict and then spends five more sentences on coverage caveats. Truncating
 * would hide the caveats, and printing the whole paragraph in the head is what
 * made the panel read as undifferentiated. So the lead sentence goes above and
 * the remainder stays in the hero, each shown exactly once.
 */
function leadSentence(summary: string) {
  return summary.match(/^[\s\S]*?[。！？!?]/)?.[0].trim() ?? summary.trim();
}

function trailingDetail(summary: string) {
  return summary.slice(leadSentence(summary).length).trim();
}

/**
 * Provenance collapsed onto one line. The panel used to spend a whole strip on
 * five dates that nobody reads before the verdict; they belong under it.
 *
 * A field the snapshot did not carry is dropped rather than printed as a hole:
 * an absent timestamp is not the same claim as an unknown one.
 */
function provenanceLine(report: DailySignalReport) {
  return [
    `报告日 ${report.report_date}`,
    `市场日 ${report.market_date ?? "—"}`,
    `数据截止 ${report.data_cutoff ?? "—"}`,
    report.generated_at_local
      ? `生成 ${[report.generated_at_local, report.timezone].filter(Boolean).join(" ")}`
      : "",
    report.model_version,
    report.stale ? "数据已过期" : "",
  ]
    .filter(Boolean)
    .join(" · ");
}

function Sparkline(props: { points: DailySignalTrendPoint[] }) {
  const path = createMemo(() => {
    const points = props.points.slice(-36);
    if (points.length < 2) return "";
    const values = points.map((point) => point.value);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const span = max - min || 1;
    return points
      .map((point, index) => {
        const x = (index / (points.length - 1)) * 100;
        const y = 28 - ((point.value - min) / span) * 24;
        return `${x.toFixed(1)},${y.toFixed(1)}`;
      })
      .join(" ");
  });
  return (
    <Show when={path()} fallback={<span class="daily-signal-no-trend">—</span>}>
      <svg class="daily-signal-sparkline" viewBox="0 0 100 32" preserveAspectRatio="none">
        <polyline points={path()} />
      </svg>
    </Show>
  );
}

function Gauge(props: { report: DailySignalReport }) {
  const value = () => Math.max(0, Math.min(100, props.report.score ?? 0));
  return (
    <div class={`daily-signal-gauge is-${props.report.signal}`}>
      <svg viewBox="0 0 220 122" aria-hidden="true">
        <path class="daily-signal-gauge__track" d="M20 108 A90 90 0 0 1 200 108" pathLength="100" />
        <path
          class="daily-signal-gauge__value"
          d="M20 108 A90 90 0 0 1 200 108"
          pathLength="100"
          stroke-dasharray={`${value()} 100`}
        />
      </svg>
      <div class="daily-signal-gauge__copy">
        <strong>{scoreLabel(props.report.score)}</strong>
        <span>健康分 / 100</span>
      </div>
    </div>
  );
}

function reportContext(report: DailySignalReport, question: string) {
  const rules = [
    "只使用上述已保存报告和当前对话",
    "事实引用报告中的来源链接",
    "明确区分事实、模型推断与情景",
    "如果问题涉及数据截止时间之后的信息，先说明报告未覆盖，并询问是否另行查询最新资料",
    "不要在回答中改写正式评分",
  ];
  const compact = {
    kind: report.kind,
    reportDate: report.report_date,
    marketDate: report.market_date,
    dataCutoff: report.data_cutoff,
    generatedAtLocal: report.generated_at_local,
    modelVersion: report.model_version,
    status: report.status,
    score: report.score,
    rawScore: report.raw_score,
    signal: report.signal,
    phase: report.phase,
    summary: report.summary,
    dimensions: report.dimensions.map((item) => ({
      label: item.label,
      score: item.score,
      signal: item.signal,
      reason: item.reason,
      evidence: item.evidence,
    })),
    companyScores: report.company_scores,
    alerts: report.alerts,
    sources: report.sources,
  };
  return buildSavedReportPrompt({
    marker: "HONE_SAVED_DAILY_SIGNAL_REPORT",
    payload: compact,
    subject: `已保存的${report.title}（报告日 ${report.report_date}）`,
    question,
    rules,
  });
}

export function DailySignalPanel(props: Props) {
  const [report, setReport] = createSignal<DailySignalReport>();
  const [history, setHistory] = createSignal<DailySignalHistoryItem[]>();
  const [tab, setTab] = createSignal<"overview" | "history" | "sources">("overview");
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal("");
  const [question, setQuestion] = createSignal("");
  let controller: AbortController | undefined;

  // One request paints the panel; the 14-day history is a separate tab and
  // loads the first time someone actually switches to it.
  const load = async () => {
    controller?.abort();
    controller = new AbortController();
    setLoading(true);
    setError("");
    try {
      setReport(await getPublicDailySignal(props.kind, controller.signal));
    } catch (cause) {
      if (cause instanceof Error && cause.name === "AbortError") return;
      setError(cause instanceof Error ? cause.message : "每日信号暂时无法加载");
    } finally {
      setLoading(false);
    }
  };

  const loadHistory = async () => {
    if (history() !== undefined) return;
    try {
      const payload = await getPublicDailySignalHistory(props.kind, 14);
      setHistory(payload.items);
    } catch {
      setHistory([]);
    }
  };

  const openTab = (value: "overview" | "history" | "sources") => {
    setTab(value);
    if (value === "history") void loadHistory();
  };

  onMount(() => void load());
  onCleanup(() => controller?.abort());

  const kind = createMemo(() => props.kind);

  const ask = () => {
    const current = report();
    const text = question().trim();
    if (!current || !text || !props.onAsk) return;
    props.onAsk(reportContext(current, text));
    setQuestion("");
    props.onClose();
  };

  return (
    <ResearchPanel
      onClose={props.onClose}
      labelledBy="daily-signal-title"
      backdropClass="daily-signal-backdrop"
      dialogClass={`daily-signal-dialog is-${kind()}`}
    >
      <>
                <ResearchPanelHead
                  id="daily-signal-title"
                  kicker={KIND_COPY[kind()].kicker}
                  title={KIND_COPY[kind()].title}
                  headline={report() ? scoreLabel(report()!.score) : undefined}
                  signal={report()?.signal}
                  signalLabel={report() ? signalLabel(report()!.signal) : undefined}
                  summary={
                    report() ? leadSentence(report()!.summary) : KIND_COPY[kind()].description
                  }
                  meta={report() ? provenanceLine(report()!) : undefined}
                  onClose={props.onClose}
                  action={
                    <button type="button" disabled={loading()} onClick={() => void load()}>
                      {loading() ? "读取中…" : "重新读取快照"}
                    </button>
                  }
                />

                <nav class="daily-signal-tabs research-scroller" aria-label="报告视图">
                  <For each={[["overview", "概览"], ["history", "历史"], ["sources", "证据与口径"]] as const}>
                    {([value, label]) => <button classList={{ active: tab() === value }} onClick={() => openTab(value)}>{label}</button>}
                  </For>
                </nav>

                <div class="daily-signal-body">
                  <Show when={loading() && !report()}>
                    <ResearchState kind="loading" message="正在读取已保存报告" detail="只读取当日快照，不会触发重新计算。" />
                  </Show>
                  <Show when={error() && !report()}>
                    <ResearchState kind="error" message="报告读取失败" detail={error()} onRetry={() => void load()} />
                  </Show>
                  <Show when={report()}>
                    {(current) => (
                      <>
                        <Show when={tab() === "overview"}>
                          <section class={`daily-signal-hero is-${current().signal}`}>
                            <Gauge report={current()} />
                            <div class="daily-signal-hero__summary">
                              <div class="daily-signal-badges">
                                <span>{current().phase}</span>
                                <span class={`is-${effectiveStatus(current())}`}>{statusLabel(effectiveStatus(current()))}</span>
                              </div>
                              <Show when={trailingDetail(current().summary)}>
                                {(detail) => <p>{detail()}</p>}
                              </Show>
                              <dl>
                                <div><dt>较昨日</dt><dd>{deltaLabel(current().comparison_yesterday)}</dd></div>
                                <div><dt>较一周</dt><dd>{deltaLabel(current().comparison_week)}</dd></div>
                                <Show when={current().kind === "macro"}><div><dt>原始风险</dt><dd>{scoreLabel(current().raw_score)} / 10</dd></div></Show>
                              </dl>
                            </div>
                          </section>

                          <Show when={current().alerts.length}>
                            <section class="daily-signal-alerts"><strong>触发提醒</strong><For each={current().alerts}>{(alert) => <p>{alert}</p>}</For></section>
                          </Show>

                          <section class="daily-signal-grid">
                            <For each={current().dimensions}>
                              {(dimension) => (
                                <details class={`daily-signal-card is-${dimension.signal}`}>
                                  <summary>
                                    <i />
                                    <span><strong>{dimension.label}</strong><small>{dimension.trend_label} · {dimensionRoleLabel(dimension.role)}</small></span>
                                    <Sparkline points={dimension.trend} />
                                    <b>{scoreLabel(dimension.score)}</b>
                                  </summary>
                                  <div><ResearchLongform text={dimension.reason} /><em>{dimension.threshold}</em>
                                    <For each={dimension.evidence}>{(item) => <a href={item.source_url} target="_blank" rel="noreferrer">{item.source} · {item.period ?? "待定"} · {item.display_value} {item.unit}</a>}</For>
                                  </div>
                                </details>
                              )}
                            </For>
                          </section>

                          <Show when={current().company_scores.length}>
                            <section class="daily-signal-section"><h3>云厂商可核验财务框架</h3><div class="daily-company-grid">
                              <For each={current().company_scores}>{(company) => <details class={`daily-company-card is-${company.signal}`}><summary><i /><span><strong>{company.symbol}</strong><small>{company.name}</small></span><b>{scoreLabel(company.score)}</b></summary><div class="daily-company-card__meta"><span>覆盖 {company.coverage}/{company.metric_total}</span><span>Capex {company.capex == null ? "—" : `$${company.capex.toFixed(1)}`}</span><span>同比 {company.capex_growth == null ? "—" : `${company.capex_growth.toFixed(1)}%`}</span><span>{company.capex_peak_status}</span></div><For each={company.metrics}>{(metric) => <p><span>{metric.label}<small>{metric.display_value}</small></span><b>{metric.score == null ? "—" : `${metric.score.toFixed(1)}/10`}</b></p>}</For></details>}</For>
                            </div></section>
                          </Show>
                        </Show>

                        <Show when={tab() === "history"}>
                          <Show when={history()} fallback={<ResearchState kind="loading" message="正在读取历史快照" />}>
                            {(items) => <section class="daily-signal-history"><h3>最近 14 个每日快照</h3><For each={items()} fallback={<p>还没有历史记录。</p>}>{(item) => <article class={`is-${item.signal}`}><i /><time>{item.report_date}</time><strong>{scoreLabel(item.score)}</strong><span>{item.phase}</span><small>{item.summary}</small></article>}</For></section>}
                          </Show>
                        </Show>

                        <Show when={tab() === "sources"}>
                          <section class="daily-signal-sources"><h3>报告正文</h3><ResearchLongform text={current().full_report} /><h3>数据源</h3><For each={current().sources}>{(source) => <a href={source.url} target="_blank" rel="noreferrer"><strong>{source.label}</strong><span>{sourceTypeLabel(source.source_type)}</span></a>}</For><h3>证据口径</h3><p>已报告事实来自外部原始资料；模型推断会明确标注；暂不可用表示当前没有有效数据。缺失值不计为零，也不自动改变正式分数。</p></section>
                        </Show>
                      </>
                    )}
                  </Show>
                </div>

                <Show when={props.onAsk && report()}>
                  {(_) => (
                    <footer class="daily-signal-footer">
                      <form onSubmit={(event) => { event.preventDefault(); ask(); }}>
                        <input value={question()} onInput={(event) => setQuestion(event.currentTarget.value)} placeholder={`基于这份${report()!.title}继续提问…`} aria-label="基于保存报告提问" />
                        <button type="submit" disabled={!question().trim()}>发送到对话</button>
                      </form>
                      <p>{report()!.disclaimer}</p>
                    </footer>
                  )}
                </Show>
      </>
    </ResearchPanel>
  );
}
