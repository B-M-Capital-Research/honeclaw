import { For, Show, createMemo } from "solid-js";
import type { FinanceCalendarEvent, FinanceCalendarPayload } from "@/lib/types";
import {
  financeCalendarEventCategory,
  financeCalendarHighlights,
  financeCalendarMonthGrid,
  financeCalendarStatusLabel,
  groupFinanceCalendarEvents,
  parseFinanceCalendarMonth,
  visibleFinanceCalendarEventsForDay,
  type FinanceCalendarEventCategory,
} from "@/lib/finance-calendar";

type FinanceCalendarCardProps = {
  payload: FinanceCalendarPayload;
  hidden?: boolean;
  registerRef?: (el: HTMLDivElement) => void;
};

const WEEKDAYS = ["一", "二", "三", "四", "五", "六", "日"];
export const FINANCE_CALENDAR_CARD_WIDTH = 1080;
export const FINANCE_CALENDAR_CARD_HEIGHT = 1350;

const EVENT_STYLE: Record<
  FinanceCalendarEventCategory,
  { background: string; color: string; dot: string }
> = {
  earnings: { background: "#e8f0ff", color: "#184f9c", dot: "#3976d3" },
  policy: { background: "#f4eaff", color: "#70419a", dot: "#9560bd" },
  inflation: { background: "#fff0eb", color: "#a8422d", dot: "#ee6a4b" },
  labor: { background: "#e7f7f3", color: "#16675b", dot: "#2f9b87" },
  growth: { background: "#edf4df", color: "#526d1f", dot: "#7da242" },
  housing: { background: "#fff5d9", color: "#8b6715", dot: "#d6a92d" },
  other: { background: "#edf1f2", color: "#526066", dot: "#829198" },
};

function eventLabel(event: FinanceCalendarEvent) {
  if (event.kind === "earnings" && event.ticker) return `${event.ticker} 财报`;
  return event.title;
}

function compactEventTime(event: FinanceCalendarEvent) {
  return event.subtitle?.replace("北京时间 ", "").split(" · ")[0] ?? "待公布";
}

function monthParts(monthValue: string) {
  const parsed = parseFinanceCalendarMonth(monthValue);
  return {
    year: parsed?.year ?? 0,
    month: parsed?.month ?? 0,
    monthText: String(parsed?.month ?? "--").padStart(2, "0"),
  };
}

export function FinanceCalendarCard(props: FinanceCalendarCardProps) {
  const cells = createMemo(() => financeCalendarMonthGrid(props.payload.month));
  const eventsByDate = createMemo(() => groupFinanceCalendarEvents(props.payload.events));
  const month = createMemo(() => monthParts(props.payload.month));
  const highlights = createMemo(() =>
    financeCalendarHighlights(props.payload.events, props.payload.today, 4),
  );
  const highlightSlots = createMemo(() =>
    Array.from({ length: 4 }, (_, index) => highlights()[index]),
  );
  const macroCount = createMemo(
    () => props.payload.events.filter((event) => event.kind === "macro").length,
  );
  const earningsCount = createMemo(
    () => props.payload.events.filter((event) => event.kind === "earnings").length,
  );
  const holdingsLabel = createMemo(() => {
    const holdings = props.payload.holdings;
    if (holdings.length === 0) return "尚未录入持仓或关注标的";
    if (holdings.length <= 7) return holdings.join("  ·  ");
    return `${holdings.slice(0, 7).join("  ·  ")}  ·  +${holdings.length - 7}`;
  });

  return (
    <div
      ref={(el) => props.registerRef?.(el)}
      style={{
        width: `${FINANCE_CALENDAR_CARD_WIDTH}px`,
        height: `${FINANCE_CALENDAR_CARD_HEIGHT}px`,
        overflow: "hidden",
        background: "#eef2f3",
        color: "#15191b",
        "box-sizing": "border-box",
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
      <header
        style={{
          height: "238px",
          padding: "36px 46px 34px",
          background: "#15191b",
          color: "#f8faf9",
          display: "flex",
          "align-items": "stretch",
          "justify-content": "space-between",
          gap: "42px",
          "box-sizing": "border-box",
        }}
      >
        <div
          style={{
            flex: "1",
            display: "flex",
            "flex-direction": "column",
            "justify-content": "space-between",
          }}
        >
          <div
            style={{
              display: "flex",
              "align-items": "center",
              gap: "11px",
              color: "#b9c3c6",
              "font-size": "13px",
              "font-weight": "800",
              "letter-spacing": "0.12em",
            }}
          >
            <span
              style={{
                width: "10px",
                height: "10px",
                background: "#f06a4b",
                display: "inline-block",
              }}
            />
            HONE / FINANCE CALENDAR
          </div>
          <div style={{ display: "flex", "align-items": "flex-end", gap: "24px" }}>
            <strong
              style={{
                color: "#f06a4b",
                "font-size": "116px",
                "font-weight": "760",
                "line-height": "0.82",
                "font-variant-numeric": "tabular-nums",
              }}
            >
              {month().monthText}
            </strong>
            <div
              style={{
                padding: "0 0 3px 24px",
                "border-left": "1px solid #4a5255",
              }}
            >
              <div
                style={{
                  "font-size": "34px",
                  "font-weight": "800",
                  "line-height": "1.15",
                }}
              >
                我的财经日历
              </div>
              <div
                style={{
                  "margin-top": "7px",
                  color: "#9da9ad",
                  "font-size": "17px",
                  "font-weight": "650",
                }}
              >
                {month().year} 年 · 北京时间
              </div>
            </div>
          </div>
        </div>

        <div
          style={{
            width: "292px",
            display: "grid",
            "grid-template-columns": "repeat(3, 1fr)",
            "align-content": "end",
            gap: "0",
          }}
        >
          <For
            each={[
              { value: macroCount(), label: "宏观事件" },
              { value: earningsCount(), label: "持仓财报" },
              { value: props.payload.holdings.length, label: "关注标的" },
            ]}
          >
            {(stat, index) => (
              <div
                style={{
                  padding: "9px 12px 8px",
                  "border-left": index() === 0 ? "none" : "1px solid #3d4548",
                  "text-align": "center",
                }}
              >
                <strong
                  style={{
                    display: "block",
                    "font-size": "30px",
                    "line-height": "1",
                    "font-variant-numeric": "tabular-nums",
                  }}
                >
                  {stat.value}
                </strong>
                <span
                  style={{
                    display: "block",
                    "margin-top": "9px",
                    color: "#9da9ad",
                    "font-size": "11px",
                    "font-weight": "750",
                  }}
                >
                  {stat.label}
                </span>
              </div>
            )}
          </For>
        </div>
      </header>

      <div style={{ height: "8px", background: "#f06a4b" }} />

      <section
        style={{
          height: "144px",
          padding: "20px 36px",
          background: "#fff",
          display: "grid",
          "grid-template-columns": "repeat(4, 1fr)",
          gap: "0",
          "box-sizing": "border-box",
        }}
      >
        <For each={highlightSlots()}>
          {(event, index) => (
            <div
              style={{
                padding: "4px 22px",
                "border-left": index() === 0 ? "none" : "1px solid #dfe5e6",
                display: "flex",
                "flex-direction": "column",
                "justify-content": "center",
                "min-width": "0",
              }}
            >
              <Show
                when={event}
                fallback={
                  <Show when={index() === highlights().length}>
                    <span
                      style={{ color: "#9ba7aa", "font-size": "12px", "font-weight": "750" }}
                    >
                      本月重点
                    </span>
                    <strong
                      style={{
                        "margin-top": "9px",
                        color: "#6f7b7e",
                        "font-size": "15px",
                        "font-weight": "700",
                      }}
                    >
                      暂无更多日程
                    </strong>
                  </Show>
                }
              >
                {(item) => (
                  <>
                    <span
                      style={{
                        color: "#f06a4b",
                        "font-size": "13px",
                        "font-weight": "850",
                        "font-variant-numeric": "tabular-nums",
                      }}
                    >
                      {item().date.slice(5).replace("-", ".")} / {compactEventTime(item())}
                    </span>
                    <strong
                      style={{
                        "margin-top": "9px",
                        color: "#202628",
                        "font-size": "16px",
                        "font-weight": "800",
                        "line-height": "1.35",
                      }}
                    >
                      {eventLabel(item())}
                    </strong>
                  </>
                )}
              </Show>
            </div>
          )}
        </For>
      </section>

      <main
        style={{
          height: "842px",
          padding: "20px 36px 18px",
          background: "#eef2f3",
          "box-sizing": "border-box",
        }}
      >
        <div
          style={{
            height: "30px",
            display: "grid",
            "grid-template-columns": "repeat(7, 1fr)",
            gap: "6px",
            "margin-bottom": "8px",
          }}
        >
          <For each={WEEKDAYS}>
            {(weekday, index) => (
              <div
                style={{
                  "text-align": "center",
                  color: index() >= 5 ? "#b05440" : "#667377",
                  "font-size": "12px",
                  "font-weight": "850",
                  "line-height": "30px",
                }}
              >
                周{weekday}
              </div>
            )}
          </For>
        </div>

        <div
          style={{
            height: "762px",
            display: "grid",
            "grid-template-columns": "repeat(7, 1fr)",
            "grid-template-rows": "repeat(6, 122px)",
            gap: "6px",
          }}
        >
          <For each={cells()}>
            {(cell, index) => {
              const dayEvents = () =>
                cell.inMonth ? eventsByDate()[cell.date] ?? [] : [];
              const visibleEvents = () =>
                visibleFinanceCalendarEventsForDay(dayEvents());
              const overflow = () =>
                Math.max(0, dayEvents().length - visibleEvents().length);
              const isToday = () => cell.inMonth && cell.date === props.payload.today;
              const weekend = () => index() % 7 >= 5;
              return (
                <div
                  style={{
                    padding: "8px",
                    border: isToday() ? "2px solid #f06a4b" : "1px solid #dbe2e4",
                    "border-radius": "6px",
                    background: !cell.inMonth
                      ? "#e7ecee"
                      : isToday()
                        ? "#fff8f5"
                        : weekend()
                          ? "#f7f9f9"
                          : "#fff",
                    display: "flex",
                    "flex-direction": "column",
                    gap: "4px",
                    "box-sizing": "border-box",
                    overflow: "hidden",
                  }}
                >
                  <Show when={cell.day}>
                    <div
                      style={{
                        height: "21px",
                        display: "flex",
                        "align-items": "center",
                        "justify-content": "space-between",
                      }}
                    >
                      <span
                        style={{
                          color: !cell.inMonth
                            ? "#99a5a8"
                            : isToday()
                              ? "#c54831"
                              : weekend()
                                ? "#a15747"
                                : "#283033",
                          "font-size": "16px",
                          "font-weight": "850",
                          "font-variant-numeric": "tabular-nums",
                        }}
                      >
                        {cell.day}
                      </span>
                      <Show when={isToday()}>
                        <span
                          style={{
                            padding: "2px 5px",
                            background: "#f06a4b",
                            color: "#fff",
                            "font-size": "9px",
                            "font-weight": "850",
                            "line-height": "1.25",
                          }}
                        >
                          TODAY
                        </span>
                      </Show>
                    </div>
                    <For each={visibleEvents()}>
                      {(event) => {
                        const eventStyle = EVENT_STYLE[financeCalendarEventCategory(event)];
                        return (
                          <div
                            style={{
                              height: "22px",
                              padding: "0 5px",
                              "border-radius": "4px",
                              background: eventStyle.background,
                              color: eventStyle.color,
                              display: "flex",
                              "align-items": "center",
                              gap: "5px",
                              "font-size": "10.5px",
                              "font-weight": "780",
                              "line-height": "1",
                              overflow: "hidden",
                              "box-sizing": "border-box",
                            }}
                          >
                            <span
                              style={{
                                width: "5px",
                                height: "5px",
                                "border-radius": "50%",
                                background: eventStyle.dot,
                                flex: "0 0 5px",
                              }}
                            />
                            <span
                              style={{
                                overflow: "hidden",
                                "text-overflow": "ellipsis",
                                "white-space": "nowrap",
                              }}
                            >
                              {eventLabel(event)}
                            </span>
                          </div>
                        );
                      }}
                    </For>
                    <Show when={overflow() > 0}>
                      <div
                        style={{
                          padding: "1px 2px 0",
                          color: "#687579",
                          "font-size": "10px",
                          "font-weight": "750",
                        }}
                      >
                        另有 {overflow()} 项
                      </div>
                    </Show>
                  </Show>
                </div>
              );
            }}
          </For>
        </div>
      </main>

      <footer
        style={{
          height: "118px",
          padding: "20px 38px",
          "border-top": "1px solid #d9e0e2",
          background: "#fff",
          display: "flex",
          "align-items": "center",
          "justify-content": "space-between",
          gap: "30px",
          "box-sizing": "border-box",
        }}
      >
        <div style={{ flex: "1", "min-width": "0" }}>
          <div
            style={{
              display: "flex",
              "align-items": "center",
              gap: "14px",
              color: "#485457",
              "font-size": "11px",
              "font-weight": "750",
            }}
          >
            <For
              each={[
                ["#9560bd", "政策"],
                ["#ee6a4b", "通胀"],
                ["#2f9b87", "就业"],
                ["#7da242", "增长"],
                ["#d6a92d", "住房"],
                ["#3976d3", "财报"],
              ]}
            >
              {(item) => (
                <span style={{ display: "inline-flex", "align-items": "center", gap: "5px" }}>
                  <i
                    style={{
                      width: "6px",
                      height: "6px",
                      "border-radius": "50%",
                      background: item[0],
                    }}
                  />
                  {item[1]}
                </span>
              )}
            </For>
          </div>
          <div
            style={{
              "margin-top": "11px",
              color: "#6d797c",
              "font-size": "12px",
              "white-space": "nowrap",
              overflow: "hidden",
              "text-overflow": "ellipsis",
            }}
          >
            持仓 / 关注：{holdingsLabel()}
          </div>
        </div>
        <div style={{ width: "340px", "text-align": "right" }}>
          <strong
            style={{
              display: "block",
              color: "#283033",
              "font-size": "12px",
              "font-weight": "850",
            }}
          >
            {financeCalendarStatusLabel(props.payload.earnings_status)}
          </strong>
          <span
            style={{
              display: "block",
              "margin-top": "9px",
              color: "#839094",
              "font-size": "10.5px",
              "line-height": "1.45",
            }}
          >
            数据源 BLS · BEA · Federal Reserve · Census · ISM · FMP
          </span>
        </div>
      </footer>
    </div>
  );
}
