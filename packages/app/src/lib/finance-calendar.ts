import type { FinanceCalendarEvent } from "./types";

export type MonthOption = {
  value: string;
  label: string;
};

export type MonthGridCell = {
  key: string;
  day: number;
  date: string;
  inMonth: boolean;
};

export type FinanceCalendarEventCategory =
  | "earnings"
  | "policy"
  | "inflation"
  | "labor"
  | "growth"
  | "housing"
  | "other";

const FINANCE_CALENDAR_MESSAGE_PATTERN =
  /(?:这是你的|your)\s*(\d{4}-\d{2})\s*(?:财经日历|finance calendar)/i;

export const FINANCE_CALENDAR_ZOOM_LEVELS = [1, 1.25, 1.5, 2, 2.5, 3] as const;

export function clampFinanceCalendarZoom(value: number): number {
  if (!Number.isFinite(value)) return 1;
  return Math.min(3, Math.max(1, value));
}

export function financeCalendarPinchZoom(
  startZoom: number,
  currentDistance: number,
  startDistance: number,
): number {
  if (startDistance <= 0) return clampFinanceCalendarZoom(startZoom);
  return clampFinanceCalendarZoom(startZoom * (currentDistance / startDistance));
}

export function clampFinanceCalendarPan(input: {
  imageWidth: number;
  imageHeight: number;
  viewportWidth: number;
  viewportHeight: number;
  zoom: number;
  x: number;
  y: number;
}): { x: number; y: number } {
  if (input.zoom <= 1) return { x: 0, y: 0 };
  const maxX = Math.max(
    0,
    (input.imageWidth * input.zoom - input.viewportWidth) / 2,
  );
  const maxY = Math.max(
    0,
    (input.imageHeight * input.zoom - input.viewportHeight) / 2,
  );
  return {
    x: Math.min(maxX, Math.max(-maxX, input.x)),
    y: Math.min(maxY, Math.max(-maxY, input.y)),
  };
}

export function financeCalendarAnchoredTransform(input: {
  startZoom: number;
  nextZoom: number;
  startX: number;
  startY: number;
  startCenterX: number;
  startCenterY: number;
  nextCenterX: number;
  nextCenterY: number;
  viewportWidth: number;
  viewportHeight: number;
}): { x: number; y: number } {
  const viewportCenterX = input.viewportWidth / 2;
  const viewportCenterY = input.viewportHeight / 2;
  const contentX =
    (input.startCenterX - viewportCenterX - input.startX) / input.startZoom;
  const contentY =
    (input.startCenterY - viewportCenterY - input.startY) / input.startZoom;
  return {
    x: input.nextCenterX - viewportCenterX - contentX * input.nextZoom,
    y: input.nextCenterY - viewportCenterY - contentY * input.nextZoom,
  };
}

export function selectFinanceCalendarImageSource(
  desktopSource: string,
  mobileSource: string | undefined,
  preferMobile: boolean,
): string {
  return preferMobile && mobileSource ? mobileSource : desktopSource;
}

export function shouldUpgradeFinanceCalendarMobileSource(
  mobileSource?: string,
): boolean {
  return !mobileSource || !mobileSource.includes("-mobile-v4");
}

export function financeCalendarMessageMonth(content: string): string | null {
  return FINANCE_CALENDAR_MESSAGE_PATTERN.exec(content)?.[1] ?? null;
}

export function isFinanceCalendarMessage(content: string): boolean {
  return financeCalendarMessageMonth(content) !== null;
}

export function stepFinanceCalendarZoom(
  current: number,
  direction: -1 | 1,
): number {
  const nearestIndex = FINANCE_CALENDAR_ZOOM_LEVELS.reduce(
    (best, level, index) =>
      Math.abs(level - current) <
      Math.abs(FINANCE_CALENDAR_ZOOM_LEVELS[best]! - current)
        ? index
        : best,
    0,
  );
  const nextIndex = Math.min(
    FINANCE_CALENDAR_ZOOM_LEVELS.length - 1,
    Math.max(0, nearestIndex + direction),
  );
  return FINANCE_CALENDAR_ZOOM_LEVELS[nextIndex]!;
}

export function parseFinanceCalendarMonth(value: string): {
  year: number;
  month: number;
} | null {
  const match = /^(\d{4})-(\d{2})$/.exec(value.trim());
  if (!match) return null;
  const year = Number(match[1]);
  const month = Number(match[2]);
  if (!Number.isFinite(year) || month < 1 || month > 12) return null;
  return { year, month };
}

export function defaultFinanceCalendarMonth(now = new Date()): string {
  const year = now.getFullYear();
  return `${year}-${String(now.getMonth() + 1).padStart(2, "0")}`;
}

export function financeCalendarMonthsForYear(year: number): MonthOption[] {
  return Array.from({ length: 12 }, (_, index) => {
    const month = index + 1;
    return {
      value: `${year}-${String(month).padStart(2, "0")}`,
      label: `${year}年${month}月`,
    };
  });
}

export function monthOptionsForSelection(value: string): MonthOption[] {
  const parsed = parseFinanceCalendarMonth(value);
  const year = parsed?.year ?? new Date().getFullYear();
  return financeCalendarMonthsForYear(year);
}

export function financeCalendarMonthGrid(monthValue: string): MonthGridCell[] {
  const parsed = parseFinanceCalendarMonth(monthValue);
  if (!parsed) return [];
  const { year, month } = parsed;
  const first = new Date(year, month - 1, 1);
  // Monday-first offset: Sun(0) should sit at the end of the first week.
  const offset = (first.getDay() + 6) % 7;
  const cells: MonthGridCell[] = [];
  for (let index = 0; index < 42; index++) {
    const cellDate = new Date(year, month - 1, 1 - offset + index);
    const cellYear = cellDate.getFullYear();
    const cellMonth = cellDate.getMonth() + 1;
    const day = cellDate.getDate();
    const date = `${cellYear}-${String(cellMonth).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
    cells.push({
      key: date,
      day,
      date,
      inMonth: cellYear === year && cellMonth === month,
    });
  }
  return cells;
}

export function groupFinanceCalendarEvents(
  events: readonly FinanceCalendarEvent[],
): Record<string, FinanceCalendarEvent[]> {
  return events.reduce<Record<string, FinanceCalendarEvent[]>>((acc, event) => {
    if (!event.date) return acc;
    (acc[event.date] ??= []).push(event);
    return acc;
  }, {});
}

export function visibleFinanceCalendarEventsForDay(
  events: readonly FinanceCalendarEvent[],
  limit = 3,
): FinanceCalendarEvent[] {
  if (events.length <= limit) return [...events];
  const visibleLimit = Math.max(1, limit - 1);
  const macro = events.filter((event) => event.kind !== "earnings");
  const earnings = events.filter((event) => event.kind === "earnings");
  if (macro.length > 0 && earnings.length > 0 && visibleLimit >= 2) {
    return [macro[0], earnings[0], ...earnings.slice(1, visibleLimit - 1)];
  }
  return events.slice(0, visibleLimit);
}

export function financeCalendarStatusLabel(status: string): string {
  switch (status) {
    case "ok":
      return "财报数据已同步";
    case "partial":
      return "部分财报数据暂不可用";
    case "missing_key":
      return "未配置 FMP，已展示宏观事件";
    case "empty_portfolio":
      return "暂无持仓/关注，已展示宏观事件";
    case "failed":
      return "财报数据暂不可用";
    default:
      return status || "数据状态未知";
  }
}

export function financeCalendarEventCategory(
  event: FinanceCalendarEvent,
): FinanceCalendarEventCategory {
  if (event.kind === "earnings") return "earnings";
  const text = `${event.title} ${event.subtitle ?? ""}`.toUpperCase();
  if (/FOMC|联储|利率|褐皮书/.test(text)) return "policy";
  if (/CPI|PPI|PCE|通胀|物价/.test(text)) return "inflation";
  if (/非农|就业|失业|薪资/.test(text)) return "labor";
  if (/房屋|住房|新屋|地产/.test(text)) return "housing";
  if (/GDP|PMI|零售|工业|贸易|耐用品/.test(text)) return "growth";
  return "other";
}

function financeCalendarEventImportance(event: FinanceCalendarEvent): number {
  if (event.kind === "earnings") return 0;
  const title = event.title.toUpperCase();
  if (/FOMC|利率决议/.test(title)) return 100;
  if (/非农|CPI|GDP/.test(title)) return 95;
  if (/PCE/.test(title)) return 92;
  if (/PPI|零售销售/.test(title)) return 88;
  if (/PMI|就业成本/.test(title)) return 82;
  if (/会议纪要|工业产出|耐用品/.test(title)) return 76;
  return 60;
}

export function financeCalendarHighlights(
  events: readonly FinanceCalendarEvent[],
  today: string,
  limit = 4,
): FinanceCalendarEvent[] {
  const macroEvents = events.filter((event) => event.kind === "macro");
  if (macroEvents.length === 0) {
    const earnings = events.filter((event) => event.kind === "earnings");
    const upcomingEarnings = earnings.filter((event) => event.date >= today);
    return [...(upcomingEarnings.length > 0 ? upcomingEarnings : earnings)]
      .sort((a, b) => a.date.localeCompare(b.date))
      .slice(0, Math.max(0, limit));
  }
  const currentMonth = today.slice(0, 7);
  const eventMonth = macroEvents[0]?.date.slice(0, 7);
  const upcoming =
    currentMonth === eventMonth
      ? macroEvents.filter((event) => event.date >= today)
      : macroEvents;
  const rank = (items: readonly FinanceCalendarEvent[]) =>
    [...items].sort(
      (a, b) =>
        financeCalendarEventImportance(b) -
          financeCalendarEventImportance(a) || a.date.localeCompare(b.date),
    );
  const selected = rank(upcoming).slice(0, Math.max(0, limit));
  if (selected.length < limit) {
    const selectedKeys = new Set(selected.map((event) => `${event.date}:${event.title}`));
    selected.push(
      ...rank(macroEvents)
        .filter((event) => !selectedKeys.has(`${event.date}:${event.title}`))
        .slice(0, limit - selected.length),
    );
  }
  return selected.sort((a, b) => a.date.localeCompare(b.date));
}

export function financeCalendarMobileAgenda(
  events: readonly FinanceCalendarEvent[],
  today: string,
  limit = 6,
): FinanceCalendarEvent[] {
  const highlights = financeCalendarHighlights(events, today, limit);
  const earnings = events
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
    .slice(0, Math.max(0, limit));
}
