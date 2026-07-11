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

export const FINANCE_CALENDAR_MOBILE_WIDTH = 750;
export const FINANCE_CALENDAR_MOBILE_HEIGHT = 1334;

const WEEKDAYS = ["一", "二", "三", "四", "五", "六", "日"];
const EVENT_META: Record<
  FinanceCalendarEventCategory,
  { color: string; soft: string; label: string }
> = {
  earnings: { color: "#3267a8", soft: "#dce8f7", label: "财报" },
  policy: { color: "#7f4aa0", soft: "#eadff1", label: "政策" },
  inflation: { color: "#d6543a", soft: "#f5ddd5", label: "通胀" },
  labor: { color: "#16806f", soft: "#d6ece7", label: "就业" },
  growth: { color: "#668b35", soft: "#e3ecd5", label: "增长" },
  housing: { color: "#ad7d16", soft: "#f3e8c9", label: "住房" },
  other: { color: "#647276", soft: "#e2e7e6", label: "事件" },
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
    ?.trim() || "待公布";
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

  return (
    <div
      ref={(element) => props.registerRef?.(element)}
      style={{
        width: `${FINANCE_CALENDAR_MOBILE_WIDTH}px`,
        height: `${FINANCE_CALENDAR_MOBILE_HEIGHT}px`,
        overflow: "hidden",
        background: "#f4f0e8",
        color: "#17201f",
        "box-sizing": "border-box",
        "font-family":
          "-apple-system, BlinkMacSystemFont, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
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
          position: "relative",
          height: "254px",
          padding: "28px 34px 24px",
          overflow: "hidden",
          background: "#16201e",
          color: "#f8f4eb",
          "box-sizing": "border-box",
        }}
      >
        <div
          style={{
            display: "flex",
            "align-items": "center",
            "justify-content": "space-between",
          }}
        >
          <div
            style={{
              display: "flex",
              "align-items": "center",
              gap: "10px",
              "font-size": "13px",
              "font-weight": "850",
              "letter-spacing": "0.17em",
            }}
          >
            <i
              style={{
                width: "9px",
                height: "9px",
                background: "#ff704f",
                transform: "rotate(45deg)",
              }}
            />
            HONE / MONTHLY BRIEF
          </div>
          <span
            style={{
              color: "#ff8a68",
              "font-size": "13px",
              "font-weight": "800",
            }}
          >
            北京时间 · {parsedMonth()?.year ?? ""}
          </span>
        </div>

        <div
          style={{
            display: "grid",
            "grid-template-columns": "145px minmax(0, 1fr) 184px",
            "align-items": "end",
            gap: "22px",
            "margin-top": "22px",
          }}
        >
          <strong
            style={{
              color: "#ff6848",
              "font-size": "112px",
              "font-weight": "760",
              "line-height": "0.82",
              "letter-spacing": "-0.07em",
              "font-variant-numeric": "tabular-nums",
            }}
          >
            {String(parsedMonth()?.month ?? "--").padStart(2, "0")}
          </strong>
          <div style={{ "padding-bottom": "3px" }}>
            <div
              style={{
                "font-size": "37px",
                "font-weight": "850",
                "letter-spacing": "-0.035em",
              }}
            >
              我的财经日历
            </div>
            <div
              style={{
                "margin-top": "12px",
                color: "#9da9a5",
                "font-size": "14px",
                "font-weight": "680",
              }}
            >
              宏观、政策与持仓财报的月度投资窗口
            </div>
          </div>
          <Show when={nextEvent()}>
            {(event) => {
              const meta = () => EVENT_META[financeCalendarEventCategory(event())];
              return (
                <div
                  style={{
                    padding: "13px 15px 12px",
                    border: "1px solid #35423f",
                    "border-radius": "13px",
                    background: "#1d2926",
                  }}
                >
                  <div
                    style={{
                      display: "flex",
                      "align-items": "center",
                      "justify-content": "space-between",
                      color: "#8f9c98",
                      "font-size": "10px",
                      "font-weight": "850",
                      "letter-spacing": "0.12em",
                    }}
                  >
                    <span>NEXT WINDOW</span>
                    <i
                      style={{
                        width: "7px",
                        height: "7px",
                        "border-radius": "50%",
                        background: meta().color,
                      }}
                    />
                  </div>
                  <strong
                    style={{
                      display: "block",
                      "margin-top": "8px",
                      color: "#fff",
                      "font-size": "21px",
                      "font-weight": "850",
                    }}
                  >
                    {event().date.slice(5).replace("-", ".")}
                  </strong>
                  <span
                    style={{
                      display: "block",
                      "margin-top": "5px",
                      overflow: "hidden",
                      color: "#c0cbc7",
                      "font-size": "12px",
                      "font-weight": "700",
                      "text-overflow": "ellipsis",
                      "white-space": "nowrap",
                    }}
                  >
                    {eventLabel(event())}
                  </span>
                </div>
              );
            }}
          </Show>
        </div>

        <div
          style={{
            position: "absolute",
            left: "34px",
            right: "34px",
            bottom: "18px",
            display: "flex",
            "align-items": "center",
            gap: "24px",
            color: "#9aa6a2",
            "font-size": "11px",
            "font-weight": "750",
          }}
        >
          <span><b style={{ color: "#fff", "font-size": "17px" }}>{eventDayCount()}</b> 个事件日</span>
          <span><b style={{ color: "#fff", "font-size": "17px" }}>{macroCount()}</b> 项宏观</span>
          <span><b style={{ color: "#fff", "font-size": "17px" }}>{earningsCount()}</b> 项财报</span>
        </div>
      </header>
      <div style={{ height: "8px", background: "#ff6848" }} />

      <section
        style={{
          height: "376px",
          padding: "18px 30px",
          background: "#fbf8f2",
          "box-sizing": "border-box",
        }}
      >
        <div
          style={{
            height: "32px",
            display: "flex",
            "align-items": "center",
            "justify-content": "space-between",
          }}
        >
          <div style={{ display: "flex", "align-items": "baseline", gap: "12px" }}>
            <strong style={{ "font-size": "21px", "font-weight": "850" }}>月度扫描</strong>
            <span style={{ color: "#84908c", "font-size": "11px", "font-weight": "700", "letter-spacing": "0.08em" }}>MONTH AT A GLANCE</span>
          </div>
          <span style={{ color: "#697572", "font-size": "11px", "font-weight": "750" }}>彩色标记为事件类别</span>
        </div>
        <div
          style={{
            height: "22px",
            display: "grid",
            "grid-template-columns": "repeat(7, 1fr)",
            "margin-top": "7px",
          }}
        >
          <For each={WEEKDAYS}>
            {(weekday, index) => (
              <span
                style={{
                  color: index() >= 5 ? "#bd5e4b" : "#7d8985",
                  "font-size": "12px",
                  "font-weight": "850",
                  "text-align": "center",
                }}
              >
                {weekday}
              </span>
            )}
          </For>
        </div>
        <div
          style={{
            height: "278px",
            display: "grid",
            "grid-template-columns": "repeat(7, 1fr)",
            "grid-template-rows": "repeat(6, 1fr)",
            gap: "6px",
            "margin-top": "1px",
          }}
        >
          <For each={cells()}>
            {(cell, index) => {
              const events = () => (cell.inMonth ? grouped()[cell.date] ?? [] : []);
              const isToday = () => cell.inMonth && cell.date === props.payload.today;
              return (
                <div
                  style={{
                    position: "relative",
                    padding: "7px 8px",
                    border: isToday() ? "none" : "1px solid #e1e5e1",
                    "border-radius": "8px",
                    background: isToday()
                      ? "#ff6848"
                      : !cell.inMonth
                        ? "#edf0ed"
                        : index() % 7 >= 5
                          ? "#faf3f0"
                          : "#fff",
                    "box-sizing": "border-box",
                  }}
                >
                  <span
                    style={{
                      color: isToday()
                        ? "#fff"
                        : !cell.inMonth
                          ? "#aab2af"
                          : index() % 7 >= 5
                            ? "#9f5546"
                            : "#26302e",
                      "font-size": "15px",
                      "font-weight": "850",
                    }}
                  >
                    {cell.day}
                  </span>
                  <Show when={events().length > 0}>
                    <div
                      style={{
                        position: "absolute",
                        left: "8px",
                        bottom: "6px",
                        display: "flex",
                        gap: "3px",
                      }}
                    >
                      <For each={events().slice(0, 4)}>
                        {(event) => (
                          <i
                            style={{
                              width: "6px",
                              height: "6px",
                              "border-radius": "50%",
                              background: isToday()
                                ? "#fff"
                                : EVENT_META[financeCalendarEventCategory(event)].color,
                            }}
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

      <main
        style={{
          height: "632px",
          padding: "24px 30px 20px",
          background: "#e7ede8",
          "box-sizing": "border-box",
        }}
      >
        <div
          style={{
            height: "52px",
            display: "flex",
            "align-items": "flex-start",
            "justify-content": "space-between",
          }}
        >
          <div>
            <strong style={{ "font-size": "25px", "font-weight": "880", "letter-spacing": "-0.02em" }}>关键投资窗口</strong>
            <span style={{ "margin-left": "12px", color: "#77827e", "font-size": "11px", "font-weight": "750", "letter-spacing": "0.08em" }}>KEY DATES</span>
          </div>
          <span style={{ color: "#68736f", "font-size": "11px", "font-weight": "750" }}>按北京时间排序</span>
        </div>
        <div
          style={{
            height: "536px",
            display: "grid",
            "grid-template-rows": `repeat(${Math.max(1, agenda().length)}, minmax(0, 1fr))`,
            border: "1px solid #cfd7d1",
            "border-radius": "16px",
            overflow: "hidden",
            background: "rgba(255,255,255,0.52)",
          }}
        >
          <For each={agenda()}>
            {(event, index) => {
              const meta = () => EVENT_META[financeCalendarEventCategory(event)];
              return (
                <div
                  style={{
                    display: "grid",
                    "grid-template-columns": "92px 26px minmax(0,1fr) 78px",
                    "align-items": "center",
                    gap: "10px",
                    padding: "0 20px",
                    border: index() === 0 ? "none" : "1px solid #d8ded9",
                    background: index() % 2 === 0 ? "rgba(255,255,255,0.32)" : "transparent",
                    "box-sizing": "border-box",
                  }}
                >
                  <div>
                    <strong
                      style={{
                        display: "block",
                        color: "#26312e",
                        "font-size": "23px",
                        "font-weight": "900",
                        "line-height": "1",
                        "font-variant-numeric": "tabular-nums",
                      }}
                    >
                      {event.date.slice(8)}
                    </strong>
                    <span
                      style={{
                        display: "block",
                        "margin-top": "6px",
                        color: "#8a9591",
                        "font-size": "10px",
                        "font-weight": "800",
                        "letter-spacing": "0.08em",
                      }}
                    >
                      JUL / {parsedMonth()?.year ?? ""}
                    </span>
                  </div>
                  <div
                    style={{
                      height: "100%",
                      display: "flex",
                      "align-items": "center",
                      "justify-content": "center",
                      position: "relative",
                    }}
                  >
                    <i style={{ position: "absolute", top: "0", bottom: "0", width: "1px", background: "#cbd3cd" }} />
                    <i style={{ position: "relative", width: "11px", height: "11px", border: "4px solid #eef2ee", "border-radius": "50%", background: meta().color, "box-sizing": "content-box" }} />
                  </div>
                  <div style={{ "min-width": "0" }}>
                    <span
                      style={{
                        display: "inline-block",
                        padding: "3px 7px",
                        "border-radius": "5px",
                        background: meta().soft,
                        color: meta().color,
                        "font-size": "11px",
                        "font-weight": "850",
                      }}
                    >
                      {meta().label}
                    </span>
                    <strong
                      style={{
                        display: "block",
                        "margin-top": "7px",
                        color: "#18211f",
                        "font-size": "20px",
                        "font-weight": "820",
                        "line-height": "1.28",
                        "overflow-wrap": "anywhere",
                      }}
                    >
                      {eventLabel(event)}
                    </strong>
                  </div>
                  <span
                    style={{
                      color: "#66716d",
                      "font-size": "13px",
                      "font-weight": "800",
                      "text-align": "right",
                      "font-variant-numeric": "tabular-nums",
                    }}
                  >
                    {eventTime(event)}
                  </span>
                </div>
              );
            }}
          </For>
        </div>
      </main>

      <footer
        style={{
          height: "64px",
          display: "flex",
          "align-items": "center",
          "justify-content": "space-between",
          padding: "0 32px",
          background: "#16201e",
          color: "#9ba7a3",
          "font-size": "10px",
          "font-weight": "750",
          "letter-spacing": "0.03em",
          "box-sizing": "border-box",
        }}
      >
        <span>HONE · 仅作日程参考，不构成投资建议</span>
        <span>BLS · BEA · FED · CENSUS · FMP</span>
      </footer>
    </div>
  );
}
