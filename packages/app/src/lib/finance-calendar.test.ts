import { describe, expect, it } from "bun:test";
import {
  defaultFinanceCalendarMonth,
  financeCalendarMonthGrid,
  financeCalendarMonthsForYear,
  financeCalendarStatusLabel,
  groupFinanceCalendarEvents,
  parseFinanceCalendarMonth,
} from "./finance-calendar";

describe("finance calendar helpers", () => {
  it("parses YYYY-MM months", () => {
    expect(parseFinanceCalendarMonth("2026-07")).toEqual({
      year: 2026,
      month: 7,
    });
    expect(parseFinanceCalendarMonth("2026-7")).toBeNull();
    expect(parseFinanceCalendarMonth("2026-13")).toBeNull();
    expect(parseFinanceCalendarMonth("bad")).toBeNull();
  });

  it("defaults to next month inside the final seven days", () => {
    expect(defaultFinanceCalendarMonth(new Date(2026, 5, 23))).toBe("2026-06");
    expect(defaultFinanceCalendarMonth(new Date(2026, 5, 24))).toBe("2026-07");
    expect(defaultFinanceCalendarMonth(new Date(2026, 11, 29))).toBe("2027-01");
  });

  it("builds a twelve-month picker for one year", () => {
    const months = financeCalendarMonthsForYear(2026);
    expect(months).toHaveLength(12);
    expect(months[0]).toEqual({ value: "2026-01", label: "2026年1月" });
    expect(months[11]).toEqual({ value: "2026-12", label: "2026年12月" });
  });

  it("builds monday-first month grid cells", () => {
    const cells = financeCalendarMonthGrid("2026-07");
    expect(cells).toHaveLength(35);
    expect(cells[0].date).toBeNull();
    expect(cells[2]).toEqual({
      key: "2026-07-01",
      day: 1,
      date: "2026-07-01",
    });
  });

  it("groups events by date", () => {
    const grouped = groupFinanceCalendarEvents([
      {
        date: "2026-07-30",
        title: "AAPL 财报",
        kind: "earnings",
        source: "fmp",
      },
      {
        date: "2026-07-30",
        title: "美联储",
        kind: "macro",
        source: "seed",
      },
    ]);
    expect(grouped["2026-07-30"]).toHaveLength(2);
  });

  it("maps earnings status to user-facing Chinese labels", () => {
    expect(financeCalendarStatusLabel("ok")).toBe("财报数据已同步");
    expect(financeCalendarStatusLabel("missing_key")).toContain("FMP");
  });
});
