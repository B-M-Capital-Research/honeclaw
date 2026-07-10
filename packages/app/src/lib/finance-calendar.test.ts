import { describe, expect, it } from "bun:test";
import {
  defaultFinanceCalendarMonth,
  financeCalendarEventCategory,
  financeCalendarHighlights,
  financeCalendarMonthGrid,
  financeCalendarMonthsForYear,
  financeCalendarStatusLabel,
  groupFinanceCalendarEvents,
  parseFinanceCalendarMonth,
  visibleFinanceCalendarEventsForDay,
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

  it("always defaults to the current month", () => {
    expect(defaultFinanceCalendarMonth(new Date(2026, 5, 23))).toBe("2026-06");
    expect(defaultFinanceCalendarMonth(new Date(2026, 5, 30))).toBe("2026-06");
    expect(defaultFinanceCalendarMonth(new Date(2026, 11, 31))).toBe("2026-12");
  });

  it("builds a twelve-month picker for one year", () => {
    const months = financeCalendarMonthsForYear(2026);
    expect(months).toHaveLength(12);
    expect(months[0]).toEqual({ value: "2026-01", label: "2026年1月" });
    expect(months[11]).toEqual({ value: "2026-12", label: "2026年12月" });
  });

  it("builds monday-first month grid cells", () => {
    const cells = financeCalendarMonthGrid("2026-07");
    expect(cells).toHaveLength(42);
    expect(cells[0]).toEqual({
      key: "2026-06-29",
      day: 29,
      date: "2026-06-29",
      inMonth: false,
    });
    expect(cells[2]).toEqual({
      key: "2026-07-01",
      day: 1,
      date: "2026-07-01",
      inMonth: true,
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

  it("keeps a holding earnings event visible on dense dates", () => {
    const visible = visibleFinanceCalendarEventsForDay([
      { date: "2026-07-30", title: "FOMC", kind: "macro", source: "fed" },
      { date: "2026-07-30", title: "GDP", kind: "macro", source: "bea" },
      { date: "2026-07-30", title: "PCE", kind: "macro", source: "bea" },
      { date: "2026-07-30", title: "AAPL 财报", kind: "earnings", source: "fmp" },
    ]);
    expect(visible.map((event) => event.title)).toEqual(["FOMC", "AAPL 财报"]);
  });

  it("maps earnings status to user-facing Chinese labels", () => {
    expect(financeCalendarStatusLabel("ok")).toBe("财报数据已同步");
    expect(financeCalendarStatusLabel("missing_key")).toContain("FMP");
  });

  it("selects upcoming high-impact macro highlights", () => {
    const events = [
      {
        date: "2026-07-02",
        title: "非农就业报告",
        kind: "macro",
        source: "bls.gov",
      },
      { date: "2026-07-14", title: "美国 CPI", kind: "macro", source: "bls.gov" },
      {
        date: "2026-07-16",
        title: "零售销售",
        kind: "macro",
        source: "census.gov",
      },
      {
        date: "2026-07-30",
        title: "FOMC 利率决议",
        kind: "macro",
        source: "federalreserve.gov",
      },
      {
        date: "2026-07-30",
        title: "美国二季度 GDP 初值",
        kind: "macro",
        source: "bea.gov",
      },
      {
        date: "2026-07-30",
        title: "AAPL 财报",
        kind: "earnings",
        source: "fmp",
      },
    ];
    const highlights = financeCalendarHighlights(events, "2026-07-10", 4);
    expect(highlights.map((event) => event.title)).toEqual([
      "美国 CPI",
      "零售销售",
      "FOMC 利率决议",
      "美国二季度 GDP 初值",
    ]);
  });

  it("uses holding earnings as highlights when a month has no macro seed", () => {
    const highlights = financeCalendarHighlights(
      [
        { date: "2026-08-05", title: "AMD 财报", kind: "earnings", source: "fmp" },
        { date: "2026-08-20", title: "NVDA 财报", kind: "earnings", source: "fmp" },
      ],
      "2026-08-10",
      4,
    );
    expect(highlights.map((event) => event.title)).toEqual(["NVDA 财报"]);
  });

  it("categorizes macro and earnings events for the image legend", () => {
    expect(
      financeCalendarEventCategory({
        date: "2026-07-14",
        title: "美国 CPI",
        kind: "macro",
        source: "bls.gov",
      }),
    ).toBe("inflation");
    expect(
      financeCalendarEventCategory({
        date: "2026-07-30",
        title: "AAPL 财报",
        kind: "earnings",
        source: "fmp",
      }),
    ).toBe("earnings");
  });
});
