import { canvasToPngBlob } from "@/components/chat-share-export";
import { CONTENT } from "@/lib/public-content";
import {
  financeCalendarEventCategory,
  financeCalendarMobileAgenda,
  financeCalendarMonthGrid,
  groupFinanceCalendarEvents,
  parseFinanceCalendarMonth,
  type FinanceCalendarEventCategory,
} from "@/lib/finance-calendar";
import type { FinanceCalendarEvent, FinanceCalendarPayload } from "@/lib/types";

export const FINANCE_CALENDAR_MOBILE_CANVAS_WIDTH = 750;
export const FINANCE_CALENDAR_MOBILE_CANVAS_HEIGHT = 1334;
export const FINANCE_CALENDAR_MOBILE_CANVAS_SCALE = 2;

const FONT = '"PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", sans-serif';
const MONO = '"SFMono-Regular", "JetBrains Mono", monospace';
const weekdays = () => [
  CONTENT.chat_page.calendar.wd_mon,
  CONTENT.chat_page.calendar.wd_tue,
  CONTENT.chat_page.calendar.wd_wed,
  CONTENT.chat_page.calendar.wd_thu,
  CONTENT.chat_page.calendar.wd_fri,
  CONTENT.chat_page.calendar.wd_sat,
  CONTENT.chat_page.calendar.wd_sun,
];
const MONTHS = [
  "JAN", "FEB", "MAR", "APR", "MAY", "JUN",
  "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

const EVENT_COLORS: Record<
  FinanceCalendarEventCategory,
  string
> = {
  earnings: "#3267a8",
  policy: "#7f4aa0",
  inflation: "#d6543a",
  labor: "#16806f",
  growth: "#668b35",
  housing: "#ad7d16",
  other: "#647276",
};

function eventMeta(category: FinanceCalendarEventCategory) {
  const label = {
    earnings: CONTENT.chat_page.calendar.earnings,
    policy: CONTENT.chat_page.calendar.cat_policy,
    inflation: CONTENT.chat_page.calendar.cat_inflation,
    labor: CONTENT.chat_page.calendar.cat_jobs,
    growth: CONTENT.chat_page.calendar.cat_growth,
    housing: CONTENT.chat_page.calendar.cat_housing,
    other: CONTENT.chat_page.calendar.cat_other,
  }[category];
  return { color: EVENT_COLORS[category], label };
}

function setFont(
  context: CanvasRenderingContext2D,
  size: number,
  weight: 400 | 600 | 700,
  family = FONT,
) {
  context.font = `${weight} ${size}px ${family}`;
  context.textBaseline = "alphabetic";
}

function roundedRect(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number,
) {
  const r = Math.min(radius, width / 2, height / 2);
  context.beginPath();
  context.moveTo(x + r, y);
  context.lineTo(x + width - r, y);
  context.quadraticCurveTo(x + width, y, x + width, y + r);
  context.lineTo(x + width, y + height - r);
  context.quadraticCurveTo(x + width, y + height, x + width - r, y + height);
  context.lineTo(x + r, y + height);
  context.quadraticCurveTo(x, y + height, x, y + height - r);
  context.lineTo(x, y + r);
  context.quadraticCurveTo(x, y, x + r, y);
  context.closePath();
}

function fillRoundedRect(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number,
  fill: string,
  stroke?: string,
) {
  roundedRect(context, x, y, width, height, radius);
  context.fillStyle = fill;
  context.fill();
  if (stroke) {
    context.strokeStyle = stroke;
    context.lineWidth = 1;
    context.stroke();
  }
}

function eventLabel(event: FinanceCalendarEvent) {
  return event.kind === "earnings" && event.ticker
    ? `${event.ticker} ${CONTENT.chat_page.calendar.earnings}`
    : event.title;
}

function eventTime(event: FinanceCalendarEvent) {
  return event.subtitle
    ?.replace("北京时间 ", "")
    .split(" · ")[0]
    ?.trim() || CONTENT.chat_page.calendar.pending;
}

export function wrapFinanceCalendarCanvasText(
  context: Pick<CanvasRenderingContext2D, "measureText">,
  text: string,
  maxWidth: number,
  maxLines = 2,
): string[] {
  const lines: string[] = [];
  let current = "";
  for (const character of [...text.trim()]) {
    const candidate = current + character;
    if (current && context.measureText(candidate).width > maxWidth) {
      if (lines.length >= maxLines - 1) break;
      lines.push(current);
      current = character;
    } else {
      current = candidate;
    }
  }
  if (current && lines.length < maxLines) lines.push(current);
  return lines;
}

function drawCover(
  context: CanvasRenderingContext2D,
  payload: FinanceCalendarPayload,
  agenda: FinanceCalendarEvent[],
  eventDayCount: number,
) {
  const parsed = parseFinanceCalendarMonth(payload.month);
  const month = parsed?.month ?? 0;
  const year = parsed?.year ?? "";
  const macroCount = payload.events.filter((event) => event.kind === "macro").length;
  const earningsCount = payload.events.filter((event) => event.kind === "earnings").length;

  context.fillStyle = "#17201f";
  context.fillRect(0, 0, 750, 236);
  context.fillStyle = "#a7b1ad";
  setFont(context, 13, 700, MONO);
  context.fillText("HONE / 财经日历", 34, 44);
  context.fillStyle = "#ff7a5d";
  context.textAlign = "right";
  context.fillText(`${CONTENT.chat_page.calendar.beijing_time} · ${year}`, 716, 44);
  context.textAlign = "left";

  context.fillStyle = "#ff7052";
  setFont(context, 88, 700);
  context.fillText(String(month || "--").padStart(2, "0"), 34, 145);

  context.fillStyle = "#fffaf1";
  setFont(context, 40, 700);
  context.fillText(CONTENT.chat_page.calendar.title, 184, 105);
  context.fillStyle = "#abb5b1";
  setFont(context, 16, 600);
  context.fillText(CONTENT.chat_page.calendar.subtitle, 184, 137);

  context.strokeStyle = "#3a4542";
  context.beginPath();
  context.moveTo(596, 76);
  context.lineTo(596, 143);
  context.stroke();
  context.fillStyle = "#87938f";
  setFont(context, 11, 700, MONO);
  context.fillText(CONTENT.chat_page.calendar.monthly_issue, 614, 92);
  context.fillStyle = "#fffaf1";
  setFont(context, 18, 700);
  context.fillText(`${MONTHS[Math.max(0, month - 1)]} / ${year}`, 614, 125);

  context.strokeStyle = "#33403d";
  context.beginPath();
  context.moveTo(34, 176);
  context.lineTo(716, 176);
  context.stroke();
  setFont(context, 14, 600);
  context.fillStyle = "#a7b1ad";
  context.fillText(`${eventDayCount} ${CONTENT.chat_page.calendar.event_day}`, 34, 216);
  context.fillText(`${macroCount} ${CONTENT.chat_page.calendar.macro}`, 120, 216);
  context.fillText(`${earningsCount} ${CONTENT.chat_page.calendar.earnings}`, 200, 216);
  const next = agenda[0];
  if (next) {
    context.textAlign = "right";
    context.fillStyle = "#dfe5e1";
    const nextText = `NEXT ${next.date.slice(5).replace("-", ".")}  ${eventLabel(next)}`;
    context.fillText(nextText, 716, 216, 400);
    context.textAlign = "left";
  }
  context.fillStyle = "#f06449";
  context.fillRect(0, 236, 750, 7);
}

function drawMonthGrid(
  context: CanvasRenderingContext2D,
  payload: FinanceCalendarPayload,
  grouped: Record<string, FinanceCalendarEvent[]>,
) {
  context.fillStyle = "#fffdf8";
  context.fillRect(0, 243, 750, 391);
  context.fillStyle = "#17201f";
  setFont(context, 27, 700);
  context.fillText(CONTENT.chat_page.calendar.signal_chart, 30, 288);
  context.fillStyle = "#7f8985";
  setFont(context, 11, 700, MONO);
  context.textAlign = "right";
  context.fillText(CONTENT.chat_page.calendar.legend, 720, 286);
  context.textAlign = "center";
  setFont(context, 15, 700);
  weekdays().forEach((weekday, index) => {
    context.fillStyle = index >= 5 ? "#b45b4b" : "#78837f";
    context.fillText(weekday, 30 + (index + 0.5) * (690 / 7), 326);
  });

  const cells = financeCalendarMonthGrid(payload.month);
  const gap = 6;
  const cellWidth = (690 - gap * 6) / 7;
  const cellHeight = 42;
  const top = 338;
  cells.forEach((cell, index) => {
    const column = index % 7;
    const row = Math.floor(index / 7);
    const x = 30 + column * (cellWidth + gap);
    const y = top + row * (cellHeight + gap);
    const today = cell.inMonth && cell.date === payload.today;
    const weekend = column >= 5;
    fillRoundedRect(
      context,
      x,
      y,
      cellWidth,
      cellHeight,
      8,
      today ? "#f06449" : !cell.inMonth ? "#eff1ed" : weekend ? "#fcf5f2" : "#ffffff",
      today ? "#f06449" : "#e4e6e0",
    );
    context.textAlign = "left";
    context.fillStyle = today ? "#ffffff" : !cell.inMonth ? "#aeb5b1" : "#17201f";
    setFont(context, 18, 700);
    context.fillText(String(cell.day), x + 8, y + 23);
    const events = cell.inMonth ? grouped[cell.date] ?? [] : [];
    events.slice(0, 4).forEach((event, eventIndex) => {
      context.fillStyle = today
        ? "#ffffff"
        : eventMeta(financeCalendarEventCategory(event)).color;
      context.beginPath();
      context.arc(x + 9 + eventIndex * 9, y + 34, 2.6, 0, Math.PI * 2);
      context.fill();
    });
  });
  context.textAlign = "left";
}

function drawAgenda(
  context: CanvasRenderingContext2D,
  payload: FinanceCalendarPayload,
  agenda: FinanceCalendarEvent[],
) {
  const parsed = parseFinanceCalendarMonth(payload.month);
  const monthLabel = MONTHS[Math.max(0, (parsed?.month ?? 1) - 1)];
  context.fillStyle = "#e8eee8";
  context.fillRect(0, 634, 750, 636);
  context.fillStyle = "#17201f";
  setFont(context, 27, 700);
  context.fillText(CONTENT.chat_page.calendar.key_window, 30, 681);
  context.fillStyle = "#7f8985";
  setFont(context, 11, 700, MONO);
  context.textAlign = "right";
  context.fillText(CONTENT.chat_page.calendar.key_dates, 720, 679);
  context.textAlign = "left";

  const listX = 30;
  const listY = 704;
  const listWidth = 690;
  const listHeight = 544;
  fillRoundedRect(context, listX, listY, listWidth, listHeight, 15, "#f8faf7", "#cbd4cd");
  const rowHeight = listHeight / Math.max(1, agenda.length);
  agenda.forEach((event, index) => {
    const top = listY + index * rowHeight;
    const center = top + rowHeight / 2;
    const meta = eventMeta(financeCalendarEventCategory(event));
    if (index > 0) {
      context.strokeStyle = "#d4dbd5";
      context.beginPath();
      context.moveTo(listX, top);
      context.lineTo(listX + listWidth, top);
      context.stroke();
    }
    context.fillStyle = "#17201f";
    setFont(context, 27, 700);
    context.fillText(event.date.slice(8), 50, center - 2);
    context.fillStyle = "#84908b";
    setFont(context, 10, 700, MONO);
    context.fillText(`${monthLabel} / ${parsed?.year ?? ""}`, 50, center + 17);

    fillRoundedRect(context, 138, center - 28, 4, 56, 2, meta.color);
    context.fillStyle = meta.color;
    setFont(context, 11, 700, MONO);
    context.fillText(meta.label, 158, center - 13);
    context.fillStyle = "#18211f";
    setFont(context, 21, 700);
    const lines = wrapFinanceCalendarCanvasText(context, eventLabel(event), 415, 2);
    const titleStart = lines.length > 1 ? center + 3 : center + 12;
    lines.forEach((line, lineIndex) => {
      context.fillText(line, 158, titleStart + lineIndex * 25);
    });

    context.fillStyle = "#68736f";
    setFont(context, 14, 700);
    context.textAlign = "right";
    context.fillText(eventTime(event), 698, center + 5, 78);
    context.textAlign = "left";
  });
}

function drawFooter(context: CanvasRenderingContext2D) {
  context.fillStyle = "#17201f";
  context.fillRect(0, 1270, 750, 64);
  context.fillStyle = "#9da8a4";
  setFont(context, 10, 600);
  context.fillText(`HONE · ${CONTENT.chat_page.calendar.disclaimer}`, 32, 1308);
  context.textAlign = "right";
  context.fillText("BLS · BEA · FED · CENSUS · FMP", 718, 1308);
  context.textAlign = "left";
}

export function renderFinanceCalendarMobileCanvas(
  payload: FinanceCalendarPayload,
  ownerDocument: Document = document,
): HTMLCanvasElement {
  const canvas = ownerDocument.createElement("canvas");
  canvas.width = FINANCE_CALENDAR_MOBILE_CANVAS_WIDTH * FINANCE_CALENDAR_MOBILE_CANVAS_SCALE;
  canvas.height = FINANCE_CALENDAR_MOBILE_CANVAS_HEIGHT * FINANCE_CALENDAR_MOBILE_CANVAS_SCALE;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("mobile finance calendar canvas unavailable");
  context.scale(FINANCE_CALENDAR_MOBILE_CANVAS_SCALE, FINANCE_CALENDAR_MOBILE_CANVAS_SCALE);
  const grouped = groupFinanceCalendarEvents(payload.events);
  const agenda = financeCalendarMobileAgenda(payload.events, payload.today, 6);
  const eventDayCount = Object.keys(grouped).filter((date) => date.startsWith(payload.month)).length;
  drawCover(context, payload, agenda, eventDayCount);
  drawMonthGrid(context, payload, grouped);
  drawAgenda(context, payload, agenda);
  drawFooter(context);
  return canvas;
}

export async function renderFinanceCalendarMobilePng(
  payload: FinanceCalendarPayload,
  ownerDocument: Document = document,
): Promise<Blob> {
  return canvasToPngBlob(renderFinanceCalendarMobileCanvas(payload, ownerDocument));
}
