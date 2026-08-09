import { describe, expect, it } from "bun:test";

import {
  formatScheduleSummary,
  formatScheduleTime,
  parseScheduleTime,
} from "./public-subscription-model";

describe("public subscription schedule labels", () => {
  it("uses the scheduler's Monday-first weekday numbering", () => {
    expect(
      formatScheduleSummary({ hour: 7, minute: 5, repeat: "weekly", weekday: 0 }),
    ).toContain("Mon 07:05");
    expect(
      formatScheduleSummary({ hour: 7, minute: 5, repeat: "weekly", weekday: 6 }),
    ).toContain("Sun 07:05");
  });

  it("does not describe special schedules as daily", () => {
    expect(formatScheduleSummary({ hour: 7, minute: 5, repeat: "workday" })).toContain(
      "Weekdays",
    );
    expect(formatScheduleSummary({ hour: 7, minute: 5, repeat: "trading_day" })).toContain(
      "US trading days",
    );
    expect(formatScheduleSummary({ hour: 0, minute: 0, repeat: "heartbeat" })).toBe(
      "Continuous monitoring",
    );
  });
});

describe("public subscription schedule time", () => {
  it("formats and parses valid times", () => {
    expect(formatScheduleTime(7, 5)).toBe("07:05");
    expect(parseScheduleTime(" 7:05 ")).toEqual({ hour: 7, minute: 5 });
  });

  it("rejects out-of-range or malformed times", () => {
    expect(parseScheduleTime("24:00")).toBeUndefined();
    expect(parseScheduleTime("10:60")).toBeUndefined();
    expect(parseScheduleTime("tomorrow")).toBeUndefined();
  });
});
