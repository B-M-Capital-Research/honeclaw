import { beforeEach, describe, expect, it } from "bun:test";

import { setLocale } from "@/lib/i18n";
import {
  formatScheduleSummary,
  formatScheduleTime,
  parseScheduleTime,
} from "./public-subscription-model";

describe("public subscription schedule labels", () => {
  // 这些断言的是英文标签，而 `locale` 是模块级 signal、整个 bun test 进程共用一份。
  // 其它测试文件会 `setLocale("zh")`，谁先跑取决于文件顺序 —— 本地和 CI 不一致，
  // 于是出现「本地全绿、CI 挂 2 个」。测试必须自己声明所需 locale，不能靠环境。
  beforeEach(() => {
    setLocale("en");
  });

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
