import { For, Show, createMemo } from "solid-js";

import {
  financeCalendarEventCategory,
  financeCalendarHighlights,
  financeCalendarMonthGrid,
  groupFinanceCalendarEvents,
  parseFinanceCalendarMonth,
  type FinanceCalendarEventCategory,
} from "@/lib/finance-calendar";
import type { FinanceCalendarEvent, FinanceCalendarPayload } from "@/lib/types";
import "./finance-calendar-mobile-card.css";

export const FINANCE_CALENDAR_MOBILE_WIDTH = 750;
export const FINANCE_CALENDAR_MOBILE_HEIGHT = 1334;

const WEEKDAYS = ["一", "二", "三", "四", "五", "六", "日"];
const MONTHS = [
  "JAN", "FEB", "MAR", "APR", "MAY", "JUN",
  "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

const EVENT_META: Record<
  FinanceCalendarEventCategory,
  { color: string; label: string }
> = {
  earnings: { color: "#3267a8", label: "EARNINGS / 财报" },
  policy: { color: "#7f4aa0", label: "POLICY / 政策" },
  inflation: { color: "#d6543a", label: "INFLATION / 通胀" },
  labor: { color: "#16806f", label: "LABOR / 就业" },
  growth: { color: "#668b35", label: "GROWTH / 增长" },
  housing: { color: "#ad7d16", label: "HOUSING / 住房" },
  other: { color: "#647276", label: "MARKET / 事件" },
};

function eventLabel(event: FinanceCalendarEvent) {
  return event.kind === "earnings" && event.ticker
    ? `${event.ticker} 财报`
    : event.title;
}

function eventTime(event: FinanceCalendarEvent) {
  return event.subtitle
    ?.replace("北京时间 ", "")
    .split(" · ")[0]
    ?.trim()
    .replace(" / ", "\n") || "待公布";
}

export function FinanceCalendarMobileCard(props: {
  payload: FinanceCalendarPayload;
  hidden?: boolean;
  registerRef?: (element: HTMLDivElement) => void;
}) {
  const parsedMonth = createMemo(() => parseFinanceCalendarMonth(props.payload.month));
  const cells = createMemo(() => financeCalendarMonthGrid(props.payload.month));
  const grouped = createMemo(() => groupFinanceCalendarEvents(props.payload.events));
  const agenda = createMemo(() => {
    const highlights = financeCalendarHighlights(
      props.payload.events,
      props.payload.today,
      6,
    );
    const earnings = props.payload.events
      .filter((event) => event.kind === "earnings")
      .sort((left, right) => left.date.localeCompare(right.date))
      .slice(0, 2);
    return [...highlights, ...earnings]
      .filter(
        (event, index, all) =>
          all.findIndex(
            (candidate) =>
              candidate.date === event.date && candidate.title === event.title,
          ) === index,
      )
      .sort(
        (left, right) =>
          left.date.localeCompare(right.date) || left.title.localeCompare(right.title),
      )
      .slice(0, 6);
  });
  const macroCount = createMemo(
    () => props.payload.events.filter((event) => event.kind === "macro").length,
  );
  const earningsCount = createMemo(
    () => props.payload.events.filter((event) => event.kind === "earnings").length,
  );
  const eventDayCount = createMemo(
    () =>
      Object.keys(grouped()).filter((date) => date.startsWith(props.payload.month))
        .length,
  );
  const nextEvent = createMemo(() => agenda()[0]);
  const monthLabel = createMemo(() => {
    const parsed = parsedMonth();
    return parsed ? MONTHS[parsed.month - 1] : "MONTH";
  });

  return (
    <div
      ref={(element) => props.registerRef?.(element)}
      class="fcm-card"
      classList={{ "fcm-card--hidden": props.hidden }}
    >
      <header class="fcm-cover">
        <div class="fcm-cover__eyebrow">
          <span>HONE / SIGNAL CALENDAR</span>
          <strong>BEIJING TIME · {parsedMonth()?.year ?? ""}</strong>
        </div>
        <div class="fcm-cover__body">
          <strong class="fcm-cover__month">
            {String(parsedMonth()?.month ?? "--").padStart(2, "0")}
          </strong>
          <div class="fcm-cover__title">
            <h1>我的财经日历</h1>
            <p>把宏观、政策与持仓财报，放进同一条投资时间轴。</p>
          </div>
          <div class="fcm-cover__issue">
            <span>MONTHLY ISSUE</span>
            <strong>{monthLabel()} / {parsedMonth()?.year ?? ""}</strong>
          </div>
        </div>
        <div class="fcm-cover__signal">
          <span><strong>{eventDayCount()}</strong> 事件日</span>
          <span><strong>{macroCount()}</strong> 宏观</span>
          <span><strong>{earningsCount()}</strong> 财报</span>
          <Show when={nextEvent()}>
            {(event) => (
              <span class="fcm-cover__next">
                <b>NEXT {event().date.slice(5).replace("-", ".")}</b>
                {eventLabel(event())}
              </span>
            )}
          </Show>
        </div>
      </header>

      <section class="fcm-scan">
        <div class="fcm-section-heading">
          <h2>月度信号图</h2>
          <span>COLOR DOTS MARK EVENT CATEGORIES</span>
        </div>
        <div class="fcm-weekdays">
          <For each={WEEKDAYS}>{(weekday) => <span>{weekday}</span>}</For>
        </div>
        <div class="fcm-month-grid">
          <For each={cells()}>
            {(cell, index) => {
              const events = () => (cell.inMonth ? grouped()[cell.date] ?? [] : []);
              const isToday = () => cell.inMonth && cell.date === props.payload.today;
              return (
                <div
                  class="fcm-day"
                  classList={{
                    "fcm-day--outside": !cell.inMonth,
                    "fcm-day--weekend": cell.inMonth && index() % 7 >= 5,
                    "fcm-day--today": isToday(),
                  }}
                >
                  <span class="fcm-day__number">{cell.day}</span>
                  <Show when={events().length > 0}>
                    <div class="fcm-day__marks">
                      <For each={events().slice(0, 4)}>
                        {(event) => (
                          <i
                            style={`--event-color: ${EVENT_META[financeCalendarEventCategory(event)].color}`}
                          />
                        )}
                      </For>
                    </div>
                  </Show>
                </div>
              );
            }}
          </For>
        </div>
      </section>

      <main class="fcm-agenda">
        <div class="fcm-section-heading">
          <h2>关键投资窗口</h2>
          <span>KEY DATES · CHRONOLOGICAL</span>
        </div>
        <div
          class="fcm-agenda-list"
          style={`--agenda-count: ${Math.max(1, agenda().length)}`}
        >
          <For each={agenda()}>
            {(event) => {
              const meta = () => EVENT_META[financeCalendarEventCategory(event)];
              return (
                <div class="fcm-agenda-row" style={`--event-color: ${meta().color}`}>
                  <div class="fcm-agenda-row__date">
                    <strong>{event.date.slice(8)}</strong>
                    <span>{monthLabel()} / {parsedMonth()?.year ?? ""}</span>
                  </div>
                  <i class="fcm-agenda-row__rail" />
                  <div class="fcm-agenda-row__content">
                    <span class="fcm-agenda-row__category">{meta().label}</span>
                    <strong class="fcm-agenda-row__title">{eventLabel(event)}</strong>
                  </div>
                  <span class="fcm-agenda-row__time">{eventTime(event)}</span>
                </div>
              );
            }}
          </For>
        </div>
      </main>

      <footer class="fcm-footer">
        <span>HONE · 仅作日程参考，不构成投资建议</span>
        <span>BLS · BEA · FED · CENSUS · FMP</span>
      </footer>
    </div>
  );
}
