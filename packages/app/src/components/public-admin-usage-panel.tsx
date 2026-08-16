import { For, Show, createMemo, createSignal, onMount } from "solid-js";
import { CONTENT } from "@/lib/public-content";
import { useLocale } from "@/lib/i18n";
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
  if (change < 0)
    return `${prefix}${CONTENT.chat_page.admin.u_fewer.replace("{n}", String(Math.abs(change)))}`;
  if (change > 0)
    return `${prefix}${CONTENT.chat_page.admin.u_more.replace("{n}", String(change))}`;
  return `${prefix}${CONTENT.chat_page.admin.u_flat}`;
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
    ? CONTENT.chat_page.admin.u_leading_drop
        .replace("{user}", labels.get(leading.userId) ?? leading.userId)
        .replace("{drop}", String(leading.drop))
    : CONTENT.chat_page.admin.u_no_decline;
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
      ? usageComparisonText(change ?? 0, CONTENT.chat_page.admin.u_vs_last_week)
      : CONTENT.chat_page.admin.u_no_compare;
    const decline = hasComparison
      ? usageLeadingDeclineText(selectedRows, previousRows, labels)
      : "";
    const text = CONTENT.chat_page.admin.u_day_summary
      .replace("{date}", formatUsageDate(selectedDate))
      .replace("{channel}", channelPrefix)
      .replace("{users}", String(totals.activeUsers))
      .replace("{questions}", String(totals.questionCount))
      .replace("{pushes}", String(totals.deliveredPushCount))
      .replace("{comparison}", comparison)
      .replace("{decline}", decline ? `；${decline}` : "");
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
  const comparison = usageComparisonText(change, CONTENT.chat_page.admin.u_vs_prev_7);
  const decline = usageLeadingDeclineText(currentRows, previousRows, labels);
  return {
    active_users: totals.activeUsers,
    question_count: totals.questionCount,
    delivered_push_count: totals.deliveredPushCount,
    comparison_user_change: change,
    text: CONTENT.chat_page.admin.u_range_summary
      .replace("{days}", String(report.period_days))
      .replace("{channel}", channelPrefix)
      .replace("{users}", String(totals.activeUsers))
      .replace("{questions}", String(totals.questionCount))
      .replace("{pushes}", String(totals.deliveredPushCount))
      .replace("{comparison}", comparison)
      .replace("{decline}", decline),
  };
}

function formatUsageDate(value: string) {
  const parsed = new Date(`${value}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return value;
  return parsed.toLocaleDateString(useLocale() === "en" ? "en-US" : "zh-CN", {
    month: "short",
    day: "numeric",
    weekday: "short",
  });
}

function formatUsageTime(value: string) {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return "—";
  return parsed.toLocaleTimeString(useLocale() === "en" ? "en-US" : "zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
}

function formatUsageChannel(channel: string) {
  switch (channel) {
    case "web":
      return CONTENT.chat_page.admin.u_web;
    case "feishu":
      return CONTENT.chat_page.admin.u_feishu;
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
  return parsed.toLocaleString(useLocale() === "en" ? "en-US" : "zh-CN", {
    month: "numeric",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
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
  const parsed = new Date(`${value}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return value;
  return parsed.toLocaleDateString(useLocale() === "en" ? "en-US" : "zh-CN", {
    month: "numeric",
    day: "numeric",
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
          {CONTENT.chat_page.admin.u_latest} <strong>{latest()}</strong> {props.unit} · {CONTENT.chat_page.admin.u_peak} {peak()} {props.unit}
        </span>
      </header>
      <svg
        class="public-admin-trend-chart-svg"
        viewBox={`0 0 ${width} ${height}`}
        role="img"
        aria-label={CONTENT.chat_page.admin.u_chart_aria
          .replace("{title}", props.title)
          .replace("{days}", String(props.points.length))
          .replace("{unit}", props.unit)}
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
      setError(cause instanceof Error ? cause.message : CONTENT.chat_page.admin.u_read_failed);
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
          <span class="public-workspace-eyebrow">{CONTENT.chat_page.admin.u_live}</span>
          <h2 id="public-admin-usage-title">HONE {CONTENT.chat_page.admin.u_title}</h2>
          <p>{CONTENT.chat_page.admin.u_subtitle}</p>
        </span>
        <span class="public-admin-section-toggle-label" aria-hidden="true">
          <span class="when-open">{CONTENT.chat_page.admin.u_collapse}</span>
          <span class="when-closed">{CONTENT.chat_page.admin.u_expand}</span>
          <span class="public-admin-section-chevron" />
        </span>
      </summary>

      <div class="public-admin-section-body">
        <Show when={error()}>
          <p class="public-admin-feedback is-error" role="alert">{error()}</p>
        </Show>

        <Show
          when={!loading() || report()}
          fallback={<div class="public-admin-loading">{CONTENT.chat_page.admin.u_generating}</div>}
        >
          <Show when={report()}>
            {(current) => (
              <>
                <div class="public-admin-usage-toolbar">
                  <label>
                    <span>{CONTENT.chat_page.admin.u_range}</span>
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
                        {(days) => (
                          <option value={days}>
                            {CONTENT.chat_page.admin.u_recent} {days}{CONTENT.chat_page.admin.u_days}
                          </option>
                        )}
                      </For>
                    </select>
                  </label>
                  <label>
                    <span>{CONTENT.chat_page.admin.u_channel_group}</span>
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
                            {channel === "all" ? CONTENT.chat_page.admin.u_all_channels : formatUsageChannel(channel)}
                          </option>
                        )}
                      </For>
                    </select>
                  </label>
                  <label>
                    <span>{CONTENT.chat_page.admin.u_stat_date}</span>
                    <select
                      value={selectedDate()}
                      onChange={(event) => {
                        const date = event.currentTarget.value;
                        setSelectedDate(date);
                        if (date !== "all") setSelectedTrendDate(date);
                      }}
                    >
                      <option value="all">{CONTENT.chat_page.admin.u_all_dates}</option>
                      <For each={dates()}>
                        {(date) => <option value={date}>{formatUsageDate(date)}</option>}
                      </For>
                    </select>
                  </label>
                  <div class="public-admin-usage-actions">
                    <small>{CONTENT.chat_page.admin.u_updated_at} {formatGeneratedAt(current().generated_at)}</small>
                    <button
                      type="button"
                      class="public-admin-refresh"
                      disabled={loading()}
                      onClick={() => void load()}
                    >
                      {loading() ? CONTENT.chat_page.admin.u_refreshing : CONTENT.chat_page.admin.u_refresh}
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
                  aria-label={`${CONTENT.chat_page.admin.u_recent} ${current().period_days}${CONTENT.chat_page.admin.u_trend_suffix}`}
                >
                  <PublicAdminUsageTrendChart
                    points={trend()}
                    metric="active_users"
                    title={CONTENT.chat_page.admin.u_daily_users}
                    description={CONTENT.chat_page.admin.u_users_tip}
                    unit={CONTENT.chat_page.admin.u_people}
                    tone="users"
                    selectedDate={selectedTrendDate()}
                    onSelectDate={setSelectedTrendDate}
                  />
                  <PublicAdminUsageTrendChart
                    points={trend()}
                    metric="question_count"
                    title={CONTENT.chat_page.admin.u_daily_questions}
                    description={CONTENT.chat_page.admin.u_questions_tip}
                    unit={CONTENT.chat_page.admin.u_count}
                    tone="questions"
                    selectedDate={selectedTrendDate()}
                    onSelectDate={setSelectedTrendDate}
                  />
                </div>

                <Show
                  when={selectedTrendPoint()}
                  fallback={
                    <p class="public-admin-trend-hint">
                      {CONTENT.chat_page.admin.u_point_hint}
                    </p>
                  }
                >
                  {(point) => (
                    <div class="public-admin-trend-detail" role="status">
                      <dl>
                        <div>
                          <dt>{CONTENT.chat_page.admin.u_date_axis}</dt>
                          <dd>{formatUsageDate(point().date)}</dd>
                        </div>
                        <div>
                          <dt>{CONTENT.chat_page.admin.u_active_users}</dt>
                          <dd>
                            {point().active_users}
                            {CONTENT.chat_page.admin.u_people}
                          </dd>
                        </div>
                        <div>
                          <dt>{CONTENT.chat_page.admin.u_question_vol}</dt>
                          <dd>
                            {point().question_count}
                            {CONTENT.chat_page.admin.u_count}
                          </dd>
                        </div>
                      </dl>
                      <button
                        type="button"
                        class="public-admin-refresh"
                        onClick={() => setSelectedDate(point().date)}
                      >
                        {CONTENT.chat_page.admin.u_view_day}
                      </button>
                    </div>
                  )}
                </Show>

                <Show
                  when={rows().length > 0}
                  fallback={<div class="public-admin-empty">{CONTENT.chat_page.admin.u_no_rows}</div>}
                >
                  <div class="public-admin-table-wrap public-admin-usage-table-wrap">
                    <table class="public-admin-table public-admin-usage-table">
                      <thead>
                        <tr>
                          <th>{CONTENT.chat_page.admin.u_date}</th>
                          <th>{CONTENT.chat_page.admin.u_channel}</th>
                          <th>{CONTENT.chat_page.admin.u_user}</th>
                          <th>{CONTENT.chat_page.admin.u_questions}</th>
                          <th>{CONTENT.chat_page.admin.u_user_question}</th>
                          <th>{CONTENT.chat_page.admin.u_scheduled}</th>
                          <th>{CONTENT.chat_page.admin.u_delivered}</th>
                          <th>{CONTENT.chat_page.admin.u_failed}</th>
                        </tr>
                      </thead>
                      <tbody>
                        <For each={rows()}>
                          {(row) => (
                            <tr>
                              <td data-label={CONTENT.chat_page.admin.u_date}>{formatUsageDate(row.date)}</td>
                              <td data-label={CONTENT.chat_page.admin.u_channel}>
                                <span class={`public-admin-channel is-${row.channel}`}>
                                  {formatUsageChannel(row.channel)}
                                </span>
                              </td>
                              <td data-label={CONTENT.chat_page.admin.u_user} class="public-admin-usage-user">
                                <strong>{row.user_label}</strong>
                                <small>{row.user_id}</small>
                              </td>
                              <td data-label={CONTENT.chat_page.admin.u_questions}><b>{row.question_count}</b></td>
                              <td data-label={CONTENT.chat_page.admin.u_user_question} class="public-admin-question-cell">
                                <Show when={row.questions.length > 0} fallback={<span>—</span>}>
                                  <details>
                                    <summary>
                                      {CONTENT.chat_page.admin.u_view} {row.questions.length}
                                      {CONTENT.chat_page.admin.u_question_unit}
                                    </summary>
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
                              <td data-label={CONTENT.chat_page.admin.u_scheduled}>{row.scheduled_run_count}</td>
                              <td data-label={CONTENT.chat_page.admin.u_delivered}><b>{row.delivered_push_count}</b></td>
                              <td data-label={CONTENT.chat_page.admin.u_failed}>
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
