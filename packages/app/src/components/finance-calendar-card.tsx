import { For, Show, createMemo } from "solid-js";
import type { FinanceCalendarEvent, FinanceCalendarPayload } from "@/lib/types";
import {
  financeCalendarMonthGrid,
  financeCalendarStatusLabel,
  groupFinanceCalendarEvents,
  parseFinanceCalendarMonth,
} from "@/lib/finance-calendar";

type FinanceCalendarCardProps = {
  payload: FinanceCalendarPayload;
  hidden?: boolean;
  registerRef?: (el: HTMLDivElement) => void;
};

const WEEKDAYS = ["一", "二", "三", "四", "五", "六", "日"];
const CARD_WIDTH = 1080;

function eventLabel(event: FinanceCalendarEvent) {
  if (event.kind === "earnings" && event.ticker) return `${event.ticker} 财报`;
  return event.title;
}

function monthTitle(monthValue: string) {
  const parsed = parseFinanceCalendarMonth(monthValue);
  if (!parsed) return monthValue;
  return `${parsed.year} 年 ${parsed.month} 月`;
}

export function FinanceCalendarCard(props: FinanceCalendarCardProps) {
  const cells = createMemo(() => financeCalendarMonthGrid(props.payload.month));
  const eventsByDate = createMemo(() => groupFinanceCalendarEvents(props.payload.events));
  const holdingsLabel = createMemo(() => {
    const holdings = props.payload.holdings;
    if (holdings.length === 0) return "暂无持仓/关注";
    if (holdings.length <= 8) return holdings.join(" · ");
    return `${holdings.slice(0, 8).join(" · ")} · +${holdings.length - 8}`;
  });

  return (
    <div
      ref={(el) => props.registerRef?.(el)}
      style={{
        width: `${CARD_WIDTH}px`,
        padding: "36px",
        background: "#f8fafc",
        color: "#0f172a",
        "font-family":
          "-apple-system, BlinkMacSystemFont, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', 'Helvetica Neue', Arial, sans-serif",
        ...(props.hidden
          ? {
              position: "fixed",
              left: "-12000px",
              top: "0",
              "pointer-events": "none",
              "z-index": "-1",
            }
          : {}),
      }}
    >
      <div
        style={{
          background: "#fff",
          border: "1px solid #e2e8f0",
          "border-radius": "18px",
          overflow: "hidden",
          "box-shadow": "0 24px 80px rgba(15,23,42,0.10)",
        }}
      >
        <div
          style={{
            padding: "30px 32px 24px",
            background:
              "linear-gradient(135deg, #111827 0%, #263241 52%, #f59e0b 190%)",
            color: "#fff",
            display: "flex",
            "align-items": "flex-start",
            "justify-content": "space-between",
            gap: "24px",
          }}
        >
          <div>
            <div
              style={{
                "font-size": "13px",
                "font-weight": "800",
                "letter-spacing": "0.16em",
                "text-transform": "uppercase",
                color: "rgba(255,255,255,0.68)",
                "margin-bottom": "8px",
              }}
            >
              Hone Finance Calendar
            </div>
            <h1
              style={{
                margin: "0",
                "font-size": "42px",
                "line-height": "1.15",
                "letter-spacing": "0",
              }}
            >
              我的财经日历 · {monthTitle(props.payload.month)}
            </h1>
          </div>
          <div
            style={{
              "text-align": "right",
              "font-size": "14px",
              "line-height": "1.7",
              color: "rgba(255,255,255,0.78)",
              "max-width": "360px",
            }}
          >
            <strong style={{ display: "block", color: "#fff", "font-size": "16px" }}>
              今日 {props.payload.today}
            </strong>
            持仓/关注：{holdingsLabel()}
          </div>
        </div>

        <div style={{ padding: "24px 28px 28px" }}>
          <div
            style={{
              display: "grid",
              "grid-template-columns": "repeat(7, 1fr)",
              gap: "8px",
              "margin-bottom": "8px",
            }}
          >
            <For each={WEEKDAYS}>
              {(weekday) => (
                <div
                  style={{
                    "text-align": "center",
                    "font-size": "13px",
                    "font-weight": "800",
                    color: "#64748b",
                    padding: "8px 0",
                  }}
                >
                  周{weekday}
                </div>
              )}
            </For>
          </div>

          <div
            style={{
              display: "grid",
              "grid-template-columns": "repeat(7, 1fr)",
              gap: "8px",
            }}
          >
            <For each={cells()}>
              {(cell) => {
                const dayEvents = () => (cell.date ? eventsByDate()[cell.date] ?? [] : []);
                const visibleEvents = () => dayEvents().slice(0, 3);
                const overflow = () => Math.max(0, dayEvents().length - visibleEvents().length);
                const isToday = () => !!cell.date && cell.date === props.payload.today;
                return (
                  <div
                    style={{
                      "min-height": "116px",
                      padding: "10px",
                      "border-radius": "12px",
                      border: isToday() ? "2px solid #f59e0b" : "1px solid #e2e8f0",
                      background: cell.day ? "#fff" : "#f8fafc",
                      opacity: cell.day ? "1" : "0.55",
                      display: "flex",
                      "flex-direction": "column",
                      gap: "7px",
                    }}
                  >
                    <Show when={cell.day}>
                      <div
                        style={{
                          display: "flex",
                          "align-items": "center",
                          "justify-content": "space-between",
                          gap: "8px",
                        }}
                      >
                        <span
                          style={{
                            "font-size": "18px",
                            "font-weight": "850",
                            color: isToday() ? "#d97706" : "#0f172a",
                          }}
                        >
                          {cell.day}
                        </span>
                        <Show when={isToday()}>
                          <span
                            style={{
                              "font-size": "10px",
                              "font-weight": "850",
                              color: "#92400e",
                              background: "#fef3c7",
                              padding: "3px 6px",
                              "border-radius": "999px",
                            }}
                          >
                            今日
                          </span>
                        </Show>
                      </div>
                      <div style={{ display: "grid", gap: "5px" }}>
                        <For each={visibleEvents()}>
                          {(event) => (
                            <div
                              style={{
                                padding: "5px 7px",
                                "border-radius": "7px",
                                background:
                                  event.kind === "earnings"
                                    ? "rgba(37,99,235,0.10)"
                                    : "rgba(245,158,11,0.12)",
                                color:
                                  event.kind === "earnings" ? "#1d4ed8" : "#92400e",
                                "font-size": "11px",
                                "line-height": "1.25",
                                "font-weight": "750",
                                overflow: "hidden",
                                display: "-webkit-box",
                                "-webkit-line-clamp": "2",
                                "-webkit-box-orient": "vertical",
                              }}
                            >
                              {eventLabel(event)}
                            </div>
                          )}
                        </For>
                        <Show when={overflow() > 0}>
                          <div
                            style={{
                              color: "#64748b",
                              "font-size": "11px",
                              "font-weight": "700",
                            }}
                          >
                            +{overflow()} 项
                          </div>
                        </Show>
                      </div>
                    </Show>
                  </div>
                );
              }}
            </For>
          </div>

          <div
            style={{
              display: "flex",
              "align-items": "center",
              "justify-content": "space-between",
              gap: "16px",
              "margin-top": "22px",
              padding: "14px 16px",
              "border-radius": "12px",
              background: "#f8fafc",
              color: "#64748b",
              "font-size": "13px",
              "line-height": "1.5",
            }}
          >
            <span>
              <strong style={{ color: "#0f172a" }}>
                {financeCalendarStatusLabel(props.payload.earnings_status)}
              </strong>
              <Show when={props.payload.errors.length > 0}>
                {" "}· {props.payload.errors.length} 条数据源提示
              </Show>
            </span>
            <span>宏观事件 + 持仓/关注财报日期</span>
          </div>
        </div>
      </div>
    </div>
  );
}
