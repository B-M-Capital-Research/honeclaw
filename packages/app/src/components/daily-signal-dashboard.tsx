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
 *
 * `model_version` is deliberately absent here. It read as a fifth metadata
 * segment (`hone-daily-signals-v2`) directly under the verdict, where it means
 * nothing to a reader and crowds out the dates that do — it now lives in
 * 「证据与口径」, next to the rest of the methodology.
 */
function provenanceLine(report: DailySignalReport) {
  const generated = [report.generated_at_local, report.timezone].filter(Boolean).join(" ");
  return [
    // 数据截止排在最前：它决定结论的时效，其余是它的注脚。
    `数据截止 ${report.data_cutoff ?? "—"}`,
    // 市场日通常就是数据截止那天，一样就不再说第二遍。
    report.market_date && report.market_date !== report.data_cutoff
      ? `市场日 ${report.market_date}`
      : "",
    generated ? `报告 ${generated}` : `报告日 ${report.report_date}`,
    report.stale ? "数据已过期" : "",
  ]
    .filter(Boolean)
    .join(" · ");
}

/**
 * The level series behind a dimension, drawn plainly.
 *
 * One caveat has to stay visible, because it is the reason this component is
 * deliberately understated: `trend` carries the raw series level (an index, a
 * rate, a dollar amount), while the `trend_label` printed right next to it —
 * 改善 / 走弱 / 持平 — is computed from year-over-year momentum, as is `score`.
 * Level and momentum genuinely disagree in the normal case: real disposable
 * income can rise every single month while its YoY growth decelerates, which
 * is exactly the 「走弱 + 一路上扬的线」 pairing that read as a rendering bug.
 * The line is not a picture of the label and must not be dressed up as one —
 * hence a neutral stroke rather than the dimension's signal color (the score
 * beside it already carries that), and a label that names the quantity.
 *
 * Each line is normalised to its own min/max on purpose: dimensions carry
 * different units, so a shared domain would be meaningless. Slopes are
 * therefore not comparable across cards, which is a further reason to keep
 * this small, unlabelled and free of chart furniture.
 */
function Sparkline(props: { points: DailySignalTrendPoint[]; label: string }) {
  const series = createMemo(() => props.points.slice(-36));
  const geometry = createMemo(() => {
    const points = series();
    if (points.length < 2) return undefined;
    const values = points.map((point) => point.value);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const span = max - min || 1;
    // The viewBox matches the rendered box 1:1 so the drawn slope is the real
    // slope. The old 100×32 box was squeezed into 80px by
    // `preserveAspectRatio="none"`, steepening every line by 25%.
    const coordinates = points.map((point, index): [number, number] => [
      2 + (index / (points.length - 1)) * 76,
      29 - ((point.value - min) / span) * 26,
    ]);
    return {
      path: coordinates.map(([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`).join(" "),
      latest: coordinates[coordinates.length - 1],
    };
  });
  return (
    <Show when={geometry()} fallback={<span class="daily-signal-no-trend">—</span>}>
      {(shape) => (
        <svg
          class="daily-signal-sparkline"
          viewBox="0 0 80 32"
          role="img"
          aria-label={`${props.label}：近 ${series().length} 期水平走势。水平与同比动量是两个量，判断以趋势文字与健康分为准。`}
        >
          <polyline points={shape().path} />
          {/* Which end is today — without it the line has no reading direction. */}
          <circle
            class="daily-signal-sparkline__now"
            cx={shape().latest[0]}
            cy={shape().latest[1]}
            r="2"
          />
        </svg>
      )}
    </Show>
  );
}

/**
 * Where the score sits on 0–100, as a bar rather than a dial.
 *
 * The dial printed the score a second time in 34px type directly under the
 * head, which already leads with it — and it claimed a fixed 240px column, so
 * the badges, the summary and the three deltas were all squeezed into what was
 * left. A bar carries the one thing the number alone cannot (position on the
 * scale) in a fraction of the height, and the hero gets its full width back.
 */
function ScoreScale(props: { report: DailySignalReport }) {
  const value = () => Math.max(0, Math.min(100, props.report.score ?? 0));
  return (
    <div class="daily-signal-scale">
      <div
        class="daily-signal-scale__track"
        role="img"
        aria-label={`健康分 ${scoreLabel(props.report.score)} / 100，越高越健康`}
      >
        <i class="daily-signal-scale__fill" style={{ width: `${value()}%` }} />
      </div>
      <div class="daily-signal-scale__ticks" aria-hidden="true">
        <span>0</span>
        <span>健康分 / 100 · 越高越健康</span>
        <span>100</span>
      </div>
    </div>
  );
}

export function DailySignalPanel(props: Props) {
  const [report, setReport] = createSignal<DailySignalReport>();
  const [history, setHistory] = createSignal<DailySignalHistoryItem[]>();
  const [tab, setTab] = createSignal<"overview" | "history" | "sources">("overview");
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal("");
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
                            <ScoreScale report={current()} />
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
                                <Show when={current().kind === "macro"}><div><dt>原始风险 <small>越高越险</small></dt><dd>{scoreLabel(current().raw_score)} / 10</dd></div></Show>
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
                                    <Sparkline points={dimension.trend} label={dimension.label} />
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
                          <section class="daily-signal-sources"><h3>报告正文</h3><ResearchLongform text={current().full_report} /><h3>数据源</h3><For each={current().sources}>{(source) => <a href={source.url} target="_blank" rel="noreferrer"><strong>{source.label}</strong><span>{sourceTypeLabel(source.source_type)}</span></a>}</For><h3>证据口径</h3><p>已报告事实来自外部原始资料；模型推断会明确标注；暂不可用表示当前没有有效数据。缺失值不计为零，也不自动改变正式分数。</p><Show when={current().model_version}><p class="daily-signal-sources__version">评分模型 {current().model_version}</p></Show></section>
                        </Show>
                      </>
                    )}
                  </Show>
                </div>

                <Show when={report()}>
                  {(current) => (
                    <footer class="daily-signal-footer">
                      <p>{current().disclaimer}</p>
                    </footer>
                  )}
                </Show>
      </>
    </ResearchPanel>
  );
}
