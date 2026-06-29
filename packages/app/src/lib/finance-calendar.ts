import type { FinanceCalendarEvent } from "./types";

export type MonthOption = {
  value: string;
  label: string;
};

export type MonthGridCell = {
  key: string;
  day: number | null;
  date: string | null;
};

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
  const monthIndex = now.getMonth();
  const lastDay = new Date(year, monthIndex + 1, 0).getDate();
  const useNext = lastDay - now.getDate() < 7;
  const target = useNext
    ? new Date(year, monthIndex + 1, 1)
    : new Date(year, monthIndex, 1);
  return `${target.getFullYear()}-${String(target.getMonth() + 1).padStart(2, "0")}`;
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
  const days = new Date(year, month, 0).getDate();
  // Monday-first offset: Sun(0) should sit at the end of the first week.
  const offset = (first.getDay() + 6) % 7;
  const cells: MonthGridCell[] = [];
  for (let i = 0; i < offset; i++) {
    cells.push({ key: `blank-${i}`, day: null, date: null });
  }
  for (let day = 1; day <= days; day++) {
    const date = `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
    cells.push({ key: date, day, date });
  }
  while (cells.length % 7 !== 0) {
    cells.push({ key: `blank-${cells.length}`, day: null, date: null });
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
