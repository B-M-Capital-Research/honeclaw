import { For, Show, createMemo, createSignal, onMount } from "solid-js";
import {
  getPublicAdminUsage,
  type PublicAdminUsageRangeDays,
} from "@/lib/api";
import type {
  PublicAdminUsageReport,
  PublicAdminUsageRow,
} from "@/lib/types";

export function publicAdminUsageDates(
  report: Pick<
    PublicAdminUsageReport,
    "period_days" | "period_start" | "period_end"
  > | null,
) {
  if (!report || report.period_start > report.period_end) return [];
  return Array.from({ length: report.period_days }, (_, index) =>
    shiftUsageDate(report.period_end, -index),
  ).filter(
    (date) =>
      Boolean(date) && date >= report.period_start && date <= report.period_end,
  );
}

export function publicAdminUsageDateIsAvailable(
  report: Pick<PublicAdminUsageReport, "period_start" | "period_end">,
  selectedDate: string,
) {
  return (
    selectedDate === "all" ||
    (selectedDate >= report.period_start && selectedDate <= report.period_end)
  );
}

export function filterPublicAdminUsageRows(
  rows: PublicAdminUsageRow[],
  selectedDate: string,
  selectedChannel = "all",
) {
  return rows.filter(
    (row) =>
      (selectedDate === "all" || row.date === selectedDate) &&
      (selectedChannel === "all" || row.channel === selectedChannel),
  );
}

type UsageTotals = {
  activeUsers: number;
  questionCount: number;
  deliveredPushCount: number;
};

export type PublicAdminUsageSelectionSummary = {
  active_users: number;
  question_count: number;
  delivered_push_count: number;
  comparison_user_change: number | null;
  text: string;
};

export type PublicAdminUsageTrendPoint = {
  date: string;
  active_users: number;
  question_count: number;
};

function usageActorKey(row: Pick<PublicAdminUsageRow, "channel" | "user_id">) {
  return `${row.channel}\u0000${row.user_id}`;
}

function shiftUsageDate(value: string, days: number) {
  const date = new Date(`${value}T00:00:00Z`);
  if (Number.isNaN(date.getTime())) return "";
  date.setUTCDate(date.getUTCDate() + days);
  return date.toISOString().slice(0, 10);
}

export function publicAdminUsageTrend(
  report: Pick<
    PublicAdminUsageReport,
    "period_days" | "period_start" | "period_end" | "rows"
  >,
  selectedChannel = "all",
): PublicAdminUsageTrendPoint[] {
  const start = report.period_start;
  if (!start) return [];

  const activeUsers = new Map<string, Set<string>>();
  const questionCounts = new Map<string, number>();
  for (const row of report.rows) {
    if (selectedChannel !== "all" && row.channel !== selectedChannel) continue;
    if (row.date < start || row.date > report.period_end) continue;
    if (row.question_count > 0) {
      const users = activeUsers.get(row.date) ?? new Set<string>();
      users.add(usageActorKey(row));
      activeUsers.set(row.date, users);
    }
    questionCounts.set(
      row.date,
      (questionCounts.get(row.date) ?? 0) + row.question_count,
    );
  }

  return Array.from({ length: report.period_days }, (_, index) => {
    const date = shiftUsageDate(start, index);
    return {
      date,
      active_users: activeUsers.get(date)?.size ?? 0,
      question_count: questionCounts.get(date) ?? 0,
    };
  });
}

function usageTotals(rows: PublicAdminUsageRow[]): UsageTotals {
  const activeUsers = new Set(
    rows.filter((row) => row.question_count > 0).map(usageActorKey),
  );
  return {
    activeUsers: activeUsers.size,
    questionCount: rows.reduce((total, row) => total + row.question_count, 0),
    deliveredPushCount: rows.reduce(
      (total, row) => total + row.delivered_push_count,
      0,
    ),
  };
}

function usageQuestionsByUser(rows: PublicAdminUsageRow[]) {
  const counts = new Map<string, number>();
  for (const row of rows) {
    const actorKey = usageActorKey(row);
    counts.set(actorKey, (counts.get(actorKey) ?? 0) + row.question_count);
  }
  return counts;
}

function usageComparisonText(change: number, prefix: string) {
  if (change < 0) return `${prefix}少 ${Math.abs(change)} 人`;
  if (change > 0) return `${prefix}多 ${change} 人`;
  return `${prefix}持平`;
}

function usageLeadingDeclineText(
  currentRows: PublicAdminUsageRow[],
  previousRows: PublicAdminUsageRow[],
  labels: Map<string, string>,
) {
  const current = usageQuestionsByUser(currentRows);
  const previous = usageQuestionsByUser(previousRows);
  let leading: { userId: string; drop: number } | null = null;
  for (const [userId, previousCount] of previous) {
    const drop = previousCount - (current.get(userId) ?? 0);
    if (
      drop > 0 &&
      (!leading || drop > leading.drop || (drop === leading.drop && userId < leading.userId))
    ) {
      leading = { userId, drop };
    }
  }
  return leading
    ? `主要是 ${labels.get(leading.userId) ?? leading.userId} 使用频率降低（少 ${leading.drop} 次）`
    : "暂无明显降频用户";
}

export function summarizePublicAdminUsage(
  report: PublicAdminUsageReport,
  selectedDate: string,
  selectedChannel = "all",
): PublicAdminUsageSelectionSummary {
  const channelRows = filterPublicAdminUsageRows(
    report.rows,
    "all",
    selectedChannel,
  );
  const labels = new Map(channelRows.map((row) => [usageActorKey(row), row.user_label]));
  const selectedRows = filterPublicAdminUsageRows(
    channelRows,
    selectedDate,
  );
  const totals = usageTotals(selectedRows);
  const channelPrefix =
    selectedChannel === "all" ? "HONE" : `${formatUsageChannel(selectedChannel)} HONE`;

  if (selectedDate !== "all") {
    const comparisonDate = shiftUsageDate(selectedDate, -7);
    const hasComparison =
      Boolean(comparisonDate) &&
      comparisonDate >= report.period_start &&
      comparisonDate <= report.period_end;
    const previousRows = hasComparison
      ? channelRows.filter((row) => row.date === comparisonDate)
      : [];
    const previousTotals = usageTotals(previousRows);
    const change = hasComparison
      ? totals.activeUsers - previousTotals.activeUsers
      : null;
    const comparison = hasComparison
      ? usageComparisonText(change ?? 0, "比上周同日")
      : "暂无上周同日可比数据";
    const decline = hasComparison
      ? usageLeadingDeclineText(selectedRows, previousRows, labels)
      : "";
    const text = `${formatUsageDate(selectedDate)} ${channelPrefix} 使用人数 ${totals.activeUsers} 人，提问问题总共 ${totals.questionCount} 个，定时任务成功推送 ${totals.deliveredPushCount} 条，${comparison}${decline ? `；${decline}` : ""}。`;
    return {
      active_users: totals.activeUsers,
      question_count: totals.questionCount,
      delivered_push_count: totals.deliveredPushCount,
      comparison_user_change: change,
      text,
    };
  }

  const recentStart = shiftUsageDate(report.period_end, -6);
  const previousEnd = shiftUsageDate(recentStart, -1);
  const previousStart = shiftUsageDate(previousEnd, -6);
  const currentRows = channelRows.filter((row) => row.date >= recentStart);
  const previousRows = channelRows.filter(
    (row) => row.date >= previousStart && row.date <= previousEnd,
  );
  const currentTotals = usageTotals(currentRows);
  const previousTotals = usageTotals(previousRows);
  const change = currentTotals.activeUsers - previousTotals.activeUsers;
  const comparison = usageComparisonText(change, "最近 7 天比前 7 天");
  const decline = usageLeadingDeclineText(currentRows, previousRows, labels);
  return {
    active_users: totals.activeUsers,
    question_count: totals.questionCount,
    delivered_push_count: totals.deliveredPushCount,
    comparison_user_change: change,
    text: `最近 ${report.period_days} 天 ${channelPrefix} 总使用人数 ${totals.activeUsers} 人，提问问题总共 ${totals.questionCount} 个，定时任务成功推送 ${totals.deliveredPushCount} 条，${comparison}；${decline}。`,
  };
}

function formatUsageDate(value: string) {
  const parsed = new Date(`${value}T00:00:00+08:00`);
  if (Number.isNaN(parsed.getTime())) return value;
  return parsed.toLocaleDateString("zh-CN", {
    month: "short",
    day: "numeric",
    weekday: "short",
    timeZone: "Asia/Shanghai",
  });
}

function formatUsageTime(value: string) {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return "—";
  return parsed.toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
    timeZone: "Asia/Shanghai",
  });
}

function formatUsageChannel(channel: string) {
  switch (channel) {
    case "web":
      return "网页";
    case "feishu":
      return "飞书";
    case "telegram":
      return "Telegram";
    case "discord":
      return "Discord";
    case "imessage":
      return "iMessage";
    default:
      return channel;
  }
}

function formatGeneratedAt(value?: string) {
  if (!value) return "";
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return "";
  return parsed.toLocaleString("zh-CN", {
    month: "numeric",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
    timeZone: "Asia/Shanghai",
  });
}

type UsageTrendMetric = "active_users" | "question_count";

const PUBLIC_ADMIN_USAGE_RANGES: PublicAdminUsageRangeDays[] = [14, 30, 90];
const PUBLIC_ADMIN_USAGE_CHANNELS = [
  "all",
  "web",
  "feishu",
  "telegram",
  "discord",
  "imessage",
] as const;

function formatTrendDate(value: string) {
  const parsed = new Date(`${value}T00:00:00+08:00`);
  if (Number.isNaN(parsed.getTime())) return value;
  return parsed.toLocaleDateString("zh-CN", {
    month: "numeric",
    day: "numeric",
    timeZone: "Asia/Shanghai",
  });
}

function PublicAdminUsageTrendChart(props: {
  points: PublicAdminUsageTrendPoint[];
  metric: UsageTrendMetric;
  title: string;
  description: string;
  unit: string;
  tone: "users" | "questions";
  selectedDate: string;
  onSelectDate: (date: string) => void;
}) {
  const width = 560;
  const height = 190;
  const left = 36;
  const right = 12;
  const top = 18;
  const bottom = 34;
  const plotWidth = width - left - right;
  const plotHeight = height - top - bottom;
  const values = () => props.points.map((point) => point[props.metric]);
  const peak = () => Math.max(0, ...values());
  const scaleMax = () => Math.max(1, peak());
  const positions = () =>
    props.points.map((point, index) => ({
      point,
      value: point[props.metric],
      x:
        left +
        (props.points.length > 1
          ? (index / (props.points.length - 1)) * plotWidth
          : plotWidth / 2),
      y: top + ((scaleMax() - point[props.metric]) / scaleMax()) * plotHeight,
    }));
  const latest = () => values().at(-1) ?? 0;
  const labelStep = () => Math.max(1, Math.ceil(props.points.length / 7));

  return (
    <article class={`public-admin-trend-chart is-${props.tone}`}>
      <header>
        <div>
          <h3>{props.title}</h3>
          <p>{props.description}</p>
        </div>
        <span>
          最新 <strong>{latest()}</strong> {props.unit} · 峰值 {peak()} {props.unit}
        </span>
      </header>
      <svg
        class="public-admin-trend-chart-svg"
        viewBox={`0 0 ${width} ${height}`}
        role="img"
        aria-label={`${props.title}，横轴为最近 ${props.points.length} 天日期，纵轴单位为${props.unit}；数据点可点击查看精确数值`}
      >
        <For each={[0, 0.5, 1]}>
          {(ratio) => {
            const y = top + ratio * plotHeight;
            const value = Math.round(scaleMax() * (1 - ratio));
            return (
              <g>
                <line
                  class="public-admin-trend-gridline"
                  x1={left}
                  x2={width - right}
                  y1={y}
                  y2={y}
                />
                <text class="public-admin-trend-y-label" x={left - 8} y={y + 3} text-anchor="end">
                  {value}
                </text>
              </g>
            );
          }}
        </For>
        <polyline
          class="public-admin-trend-line"
          points={positions().map((position) => `${position.x},${position.y}`).join(" ")}
          fill="none"
        />
        <For each={positions()}>
          {(position, index) => (
            <g
              class="public-admin-trend-point-target"
              classList={{ "is-selected": props.selectedDate === position.point.date }}
              role="button"
              tabIndex={0}
              aria-label={`${formatTrendDate(position.point.date)}，${position.value} ${props.unit}`}
              onClick={() => props.onSelectDate(position.point.date)}
              onKeyDown={(event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  props.onSelectDate(position.point.date);
                }
              }}
            >
              <line
                class="public-admin-trend-tick"
                x1={position.x}
                x2={position.x}
                y1={height - bottom}
                y2={height - bottom + 4}
              />
              <circle
                class="public-admin-trend-point"
                cx={position.x}
                cy={position.y}
                r="3.5"
              >
                <title>{`${formatTrendDate(position.point.date)}：${position.value} ${props.unit}`}</title>
              </circle>
              <circle
                class="public-admin-trend-point-hitbox"
                cx={position.x}
                cy={position.y}
                r="10"
              />
              <Show
                when={
                  index() % labelStep() === 0 ||
                  index() === props.points.length - 1
                }
              >
                <text
                  class="public-admin-trend-x-label"
                  x={position.x}
                  y={height - 9}
                  text-anchor="middle"
                >
                  {formatTrendDate(position.point.date)}
                </text>
              </Show>
            </g>
          )}
        </For>
      </svg>
    </article>
  );
}

export function PublicAdminUsagePanel() {
  const [report, setReport] = createSignal<PublicAdminUsageReport | null>(null);
  const [rangeDays, setRangeDays] =
    createSignal<PublicAdminUsageRangeDays>(14);
  const [selectedChannel, setSelectedChannel] = createSignal("all");
  const [selectedDate, setSelectedDate] = createSignal("all");
  const [selectedTrendDate, setSelectedTrendDate] = createSignal("");
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal("");
  let loadVersion = 0;

  const load = async (days = rangeDays()) => {
    const version = ++loadVersion;
    setLoading(true);
    setError("");
    try {
      const next = await getPublicAdminUsage(days);
      if (version !== loadVersion) return;
      setReport(next);
      if (PUBLIC_ADMIN_USAGE_RANGES.includes(next.period_days as PublicAdminUsageRangeDays)) {
        setRangeDays(next.period_days as PublicAdminUsageRangeDays);
      }
      if (!publicAdminUsageDateIsAvailable(next, selectedDate())) {
        setSelectedDate("all");
      }
      if (
        selectedTrendDate() &&
        !publicAdminUsageDateIsAvailable(next, selectedTrendDate())
      ) {
        setSelectedTrendDate("");
      }
    } catch (cause) {
      if (version !== loadVersion) return;
      setError(cause instanceof Error ? cause.message : "读取使用统计失败");
    } finally {
      if (version === loadVersion) setLoading(false);
    }
  };

  onMount(() => void load());

  const dates = createMemo(() => publicAdminUsageDates(report()));
  const rows = createMemo(() =>
    filterPublicAdminUsageRows(
      report()?.rows ?? [],
      selectedDate(),
      selectedChannel(),
    ),
  );
  const summary = createMemo(() => {
    const current = report();
    return current
      ? summarizePublicAdminUsage(
          current,
          selectedDate(),
          selectedChannel(),
        )
      : null;
  });
  const trend = createMemo(() => {
    const current = report();
    return current
      ? publicAdminUsageTrend(current, selectedChannel())
      : [];
  });
  const selectedTrendPoint = createMemo(() =>
    trend().find((point) => point.date === selectedTrendDate()),
  );

  return (
    <details
      class="public-workspace-panel public-admin-panel public-admin-usage-panel"
      aria-labelledby="public-admin-usage-title"
      open
    >
      <summary class="public-admin-section-summary">
        <span class="public-admin-section-copy">
          <span class="public-workspace-eyebrow">实时统计</span>
          <h2 id="public-admin-usage-title">HONE 使用统计</h2>
          <p>按北京时间汇总网页、飞书及其它接入渠道；可切换渠道和追溯周期，跨渠道未绑定的账号分别计数。</p>
        </span>
        <span class="public-admin-section-toggle-label" aria-hidden="true">
          <span class="when-open">收起</span>
          <span class="when-closed">展开</span>
          <span class="public-admin-section-chevron" />
        </span>
      </summary>

      <div class="public-admin-section-body">
        <Show when={error()}>
          <p class="public-admin-feedback is-error" role="alert">{error()}</p>
        </Show>

        <Show
          when={!loading() || report()}
          fallback={<div class="public-admin-loading">正在生成实时使用统计…</div>}
        >
          <Show when={report()}>
            {(current) => (
              <>
                <div class="public-admin-usage-toolbar">
                  <label>
                    <span>追溯范围</span>
                    <select
                      value={rangeDays()}
                      onChange={(event) => {
                        const days = Number(event.currentTarget.value) as PublicAdminUsageRangeDays;
                        setRangeDays(days);
                        setSelectedDate("all");
                        setSelectedTrendDate("");
                        void load(days);
                      }}
                    >
                      <For each={PUBLIC_ADMIN_USAGE_RANGES}>
                        {(days) => <option value={days}>最近 {days} 天</option>}
                      </For>
                    </select>
                  </label>
                  <label>
                    <span>渠道分类</span>
                    <select
                      value={selectedChannel()}
                      onChange={(event) => {
                        setSelectedChannel(event.currentTarget.value);
                        setSelectedTrendDate("");
                      }}
                    >
                      <For each={PUBLIC_ADMIN_USAGE_CHANNELS}>
                        {(channel) => (
                          <option value={channel}>
                            {channel === "all" ? "全部渠道" : formatUsageChannel(channel)}
                          </option>
                        )}
                      </For>
                    </select>
                  </label>
                  <label>
                    <span>统计日期</span>
                    <select
                      value={selectedDate()}
                      onChange={(event) => {
                        const date = event.currentTarget.value;
                        setSelectedDate(date);
                        if (date !== "all") setSelectedTrendDate(date);
                      }}
                    >
                      <option value="all">所选范围全部日期</option>
                      <For each={dates()}>
                        {(date) => <option value={date}>{formatUsageDate(date)}</option>}
                      </For>
                    </select>
                  </label>
                  <div class="public-admin-usage-actions">
                    <small>更新于 {formatGeneratedAt(current().generated_at)}</small>
                    <button
                      type="button"
                      class="public-admin-refresh"
                      disabled={loading()}
                      onClick={() => void load()}
                    >
                      {loading() ? "刷新中…" : "刷新数据"}
                    </button>
                  </div>
                </div>

                <Show when={summary()?.text}>
                  {(text) => (
                    <p class="public-admin-live-summary" role="status">
                      {text()}
                    </p>
                  )}
                </Show>

                <div
                  class="public-admin-trend-grid"
                  aria-label={`最近 ${current().period_days} 天使用趋势`}
                >
                  <PublicAdminUsageTrendChart
                    points={trend()}
                    metric="active_users"
                    title="每日使用用户数"
                    description="当天至少提出 1 个真实问题的去重用户"
                    unit="人"
                    tone="users"
                    selectedDate={selectedTrendDate()}
                    onSelectDate={setSelectedTrendDate}
                  />
                  <PublicAdminUsageTrendChart
                    points={trend()}
                    metric="question_count"
                    title="每日提问量"
                    description="当天所有真实用户问题总数"
                    unit="个"
                    tone="questions"
                    selectedDate={selectedTrendDate()}
                    onSelectDate={setSelectedTrendDate}
                  />
                </div>

                <Show
                  when={selectedTrendPoint()}
                  fallback={
                    <p class="public-admin-trend-hint">
                      点击任一折线数据点，可查看该日期的精确横纵数字。
                    </p>
                  }
                >
                  {(point) => (
                    <div class="public-admin-trend-detail" role="status">
                      <dl>
                        <div>
                          <dt>日期（横轴）</dt>
                          <dd>{formatUsageDate(point().date)}</dd>
                        </div>
                        <div>
                          <dt>使用用户数</dt>
                          <dd>{point().active_users} 人</dd>
                        </div>
                        <div>
                          <dt>提问量</dt>
                          <dd>{point().question_count} 个</dd>
                        </div>
                      </dl>
                      <button
                        type="button"
                        class="public-admin-refresh"
                        onClick={() => setSelectedDate(point().date)}
                      >
                        查看当天表格
                      </button>
                    </div>
                  )}
                </Show>

                <Show
                  when={rows().length > 0}
                  fallback={<div class="public-admin-empty">所选日期暂无提问或定时任务记录。</div>}
                >
                  <div class="public-admin-table-wrap public-admin-usage-table-wrap">
                    <table class="public-admin-table public-admin-usage-table">
                      <thead>
                        <tr>
                          <th>日期</th>
                          <th>渠道</th>
                          <th>用户</th>
                          <th>提问</th>
                          <th>用户询问的问题</th>
                          <th>定时执行</th>
                          <th>成功推送</th>
                          <th>投递失败</th>
                        </tr>
                      </thead>
                      <tbody>
                        <For each={rows()}>
                          {(row) => (
                            <tr>
                              <td data-label="日期">{formatUsageDate(row.date)}</td>
                              <td data-label="渠道">
                                <span class={`public-admin-channel is-${row.channel}`}>
                                  {formatUsageChannel(row.channel)}
                                </span>
                              </td>
                              <td data-label="用户" class="public-admin-usage-user">
                                <strong>{row.user_label}</strong>
                                <small>{row.user_id}</small>
                              </td>
                              <td data-label="提问"><b>{row.question_count}</b></td>
                              <td data-label="用户询问的问题" class="public-admin-question-cell">
                                <Show when={row.questions.length > 0} fallback={<span>—</span>}>
                                  <details>
                                    <summary>查看 {row.questions.length} 个问题</summary>
                                    <ol>
                                      <For each={row.questions}>
                                        {(question) => (
                                          <li>
                                            <time>{formatUsageTime(question.asked_at)}</time>
                                            <span>{question.text}</span>
                                          </li>
                                        )}
                                      </For>
                                    </ol>
                                  </details>
                                </Show>
                              </td>
                              <td data-label="定时执行">{row.scheduled_run_count}</td>
                              <td data-label="成功推送"><b>{row.delivered_push_count}</b></td>
                              <td data-label="投递失败">
                                <span classList={{ "is-danger": row.failed_delivery_count > 0 }}>
                                  {row.failed_delivery_count}
                                </span>
                              </td>
                            </tr>
                          )}
                        </For>
                      </tbody>
                    </table>
                  </div>
                </Show>
              </>
            )}
          </Show>
        </Show>
      </div>
    </details>
  );
}
