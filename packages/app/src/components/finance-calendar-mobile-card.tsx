import { For, Show, createMemo } from "solid-js";

import {
  financeCalendarEventCategory,
  financeCalendarHighlights,
  financeCalendarMonthGrid,
  groupFinanceCalendarEvents,
  parseFinanceCalendarMonth,
} from "@/lib/finance-calendar";
import type { FinanceCalendarEvent, FinanceCalendarPayload } from "@/lib/types";

export const FINANCE_CALENDAR_MOBILE_WIDTH = 750;
export const FINANCE_CALENDAR_MOBILE_HEIGHT = 1334;

const WEEKDAYS = ["一", "二", "三", "四", "五", "六", "日"];
const CATEGORY_COLOR = {
  earnings: "#3976d3",
  policy: "#9560bd",
  inflation: "#ee6a4b",
  labor: "#2f9b87",
  growth: "#7da242",
  housing: "#d6a92d",
  other: "#829198",
} as const;

function mobileEventLabel(event: FinanceCalendarEvent) {
  return event.kind === "earnings" && event.ticker
    ? `${event.ticker} 财报`
    : event.title;
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
      5,
    );
    const earnings = props.payload.events
      .filter((event) => event.kind === "earnings")
      .sort((left, right) => left.date.localeCompare(right.date))
      .slice(0, 2);
    const selected = [...highlights, ...earnings].filter(
      (event, index, all) =>
        all.findIndex(
          (candidate) =>
            candidate.date === event.date && candidate.title === event.title,
        ) === index,
    );
    return selected
      .sort(
        (left, right) =>
          left.date.localeCompare(right.date) || left.title.localeCompare(right.title),
      )
      .slice(0, 7);
  });
  const macroCount = createMemo(
    () => props.payload.events.filter((event) => event.kind === "macro").length,
  );
  const earningsCount = createMemo(
    () => props.payload.events.filter((event) => event.kind === "earnings").length,
  );

  return (
    <div
      ref={(element) => props.registerRef?.(element)}
      style={{
        width: `${FINANCE_CALENDAR_MOBILE_WIDTH}px`,
        height: `${FINANCE_CALENDAR_MOBILE_HEIGHT}px`,
        overflow: "hidden",
        background: "#edf1f2",
        color: "#182023",
        "box-sizing": "border-box",
        "font-family": "-apple-system, BlinkMacSystemFont, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
        ...(props.hidden
          ? { position: "fixed", left: "-12000px", top: "0", "pointer-events": "none", "z-index": "-1" }
          : {}),
      }}
    >
      <header style={{ height: "218px", padding: "34px 38px", background: "#151b1e", color: "#fff", "box-sizing": "border-box" }}>
        <div style={{ display: "flex", "align-items": "center", "justify-content": "space-between" }}>
          <span style={{ color: "#bbc4c7", "font-size": "14px", "font-weight": "800", "letter-spacing": "0.16em" }}>HONE / 财经日历</span>
          <span style={{ color: "#f47a55", "font-size": "14px", "font-weight": "800" }}>北京时间</span>
        </div>
        <div style={{ display: "flex", "align-items": "flex-end", "justify-content": "space-between", "margin-top": "25px" }}>
          <div style={{ display: "flex", "align-items": "flex-end", gap: "18px" }}>
            <strong style={{ color: "#f06a4b", "font-size": "92px", "line-height": "0.82", "font-variant-numeric": "tabular-nums" }}>
              {String(parsedMonth()?.month ?? "--").padStart(2, "0")}
            </strong>
            <div style={{ "padding-bottom": "4px" }}>
              <div style={{ "font-size": "32px", "font-weight": "850" }}>我的财经日历</div>
              <div style={{ "margin-top": "7px", color: "#9fa9ac", "font-size": "17px", "font-weight": "650" }}>{parsedMonth()?.year ?? ""} 年</div>
            </div>
          </div>
          <div style={{ "padding-bottom": "6px", "text-align": "right" }}>
            <strong style={{ display: "block", "font-size": "25px" }}>{macroCount()} / {earningsCount()}</strong>
            <span style={{ color: "#9fa9ac", "font-size": "12px", "font-weight": "700" }}>宏观 / 财报</span>
          </div>
        </div>
      </header>
      <div style={{ height: "7px", background: "#f06a4b" }} />

      <section style={{ height: "386px", padding: "25px 30px 24px", background: "#fff", "box-sizing": "border-box" }}>
        <div style={{ display: "grid", "grid-template-columns": "repeat(7, 1fr)", "margin-bottom": "12px" }}>
          <For each={WEEKDAYS}>{(day, index) => <span style={{ color: index() >= 5 ? "#b15442" : "#7b878a", "font-size": "14px", "font-weight": "850", "text-align": "center" }}>周{day}</span>}</For>
        </div>
        <div style={{ height: "300px", display: "grid", "grid-template-columns": "repeat(7, 1fr)", "grid-template-rows": "repeat(6, 1fr)", gap: "5px" }}>
          <For each={cells()}>
            {(cell, index) => {
              const events = () => (cell.inMonth ? grouped()[cell.date] ?? [] : []);
              const isToday = () => cell.inMonth && cell.date === props.payload.today;
              return (
                <div style={{ position: "relative", padding: "7px 6px", border: isToday() ? "2px solid #f06a4b" : "1px solid #dfe5e6", "border-radius": "8px", background: cell.inMonth ? (index() % 7 >= 5 ? "#faf7f6" : "#f8faf9") : "#edf1f2", "box-sizing": "border-box" }}>
                  <span style={{ color: cell.inMonth ? (isToday() ? "#c54831" : "#253034") : "#aab3b6", "font-size": "16px", "font-weight": "850" }}>{cell.day}</span>
                  <Show when={events().length > 0}>
                    <div style={{ position: "absolute", left: "6px", right: "6px", bottom: "7px", display: "flex", gap: "3px" }}>
                      <For each={events().slice(0, 3)}>{(event) => <i style={{ width: "7px", height: "7px", "border-radius": "50%", background: CATEGORY_COLOR[financeCalendarEventCategory(event)] }} />}</For>
                    </div>
                  </Show>
                </div>
              );
            }}
          </For>
        </div>
      </section>

      <main style={{ height: "667px", padding: "25px 30px 20px", "box-sizing": "border-box" }}>
        <div style={{ display: "flex", "align-items": "center", "justify-content": "space-between", "margin-bottom": "16px" }}>
          <strong style={{ "font-size": "24px", "font-weight": "850" }}>本月重点日程</strong>
          <span style={{ color: "#788487", "font-size": "13px", "font-weight": "700" }}>按日期排列</span>
        </div>
        <div style={{ display: "grid", gap: "9px" }}>
          <For each={agenda()}>
            {(event) => (
              <div style={{ height: "75px", display: "grid", "grid-template-columns": "82px minmax(0,1fr)", gap: "15px", padding: "10px 15px", border: "1px solid #dce3e5", "border-radius": "13px", background: "#fff", "box-sizing": "border-box" }}>
                <div style={{ display: "flex", "align-items": "center", "justify-content": "center", color: "#e15f42", "font-size": "22px", "font-weight": "900", "font-variant-numeric": "tabular-nums" }}>{event.date.slice(5).replace("-", ".")}</div>
                <div style={{ display: "flex", "align-items": "center", gap: "10px", "min-width": "0" }}>
                  <i style={{ width: "9px", height: "9px", "border-radius": "50%", background: CATEGORY_COLOR[financeCalendarEventCategory(event)], flex: "0 0 9px" }} />
                  <strong style={{ "overflow-wrap": "anywhere", "font-size": "18px", "font-weight": "780", "line-height": "1.3" }}>{mobileEventLabel(event)}</strong>
                </div>
              </div>
            )}
          </For>
        </div>
      </main>

      <footer style={{ height: "56px", display: "flex", "align-items": "center", "justify-content": "space-between", padding: "0 32px", border: "1px solid #dde3e5", background: "#fff", color: "#748084", "font-size": "11px", "font-weight": "700", "box-sizing": "border-box" }}>
        <span>政策 · 通胀 · 就业 · 增长 · 财报</span>
        <span>BLS · BEA · FED · FMP</span>
      </footer>
    </div>
  );
}
