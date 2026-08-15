import { afterAll, beforeEach, describe, expect, it } from "bun:test";

import { setLocale } from "@/lib/i18n";
import {
  filterPublicAdminUsageRows,
  publicAdminUsageDateIsAvailable,
  publicAdminUsageTrend,
  publicAdminUsageDates,
  summarizePublicAdminUsage,
} from "./public-admin-usage-panel";
import type { PublicAdminUsageReport, PublicAdminUsageRow } from "@/lib/types";

function row(
  date: string,
  userId: string,
  questionCount = 1,
  deliveredPushCount = 0,
  userLabel = userId,
  channel = "web",
): PublicAdminUsageRow {
  return {
    date,
    channel,
    user_id: userId,
    user_label: userLabel,
    question_count: questionCount,
    questions: Array.from({ length: questionCount }, (_, index) => ({
      asked_at: `${date}T10:${String(index).padStart(2, "0")}:00+08:00`,
      text: `问题 ${index + 1}`,
    })),
    scheduled_run_count: 0,
    delivered_push_count: deliveredPushCount,
    failed_delivery_count: 0,
    latest_activity_at: `${date}T10:00:00+08:00`,
  };
}

function report(rows: PublicAdminUsageRow[]): PublicAdminUsageReport {
  return {
    generated_at: "2026-08-02T12:00:00+08:00",
    period_days: 14,
    period_start: "2026-07-20",
    period_end: "2026-08-02",
    summary: {
      today: "2026-08-02",
      today_active_users: 0,
      today_question_count: 0,
      today_delivered_push_count: 0,
      last_week_same_day_active_users: 0,
      active_user_change: 0,
      leading_decline_question_delta: 0,
      text: "服务端摘要不应直接展示",
    },
    rows,
  };
}

beforeEach(() => {
  // The summary text is localized; these cases assert the Chinese wording.
  setLocale("zh");
});

describe("public admin usage panel", () => {
  const rows = [
    row("2026-08-01", "u1"),
    row("2026-08-02", "u2"),
    row("2026-08-02", "u3"),
  ];

  it("offers every report date newest-first even when rows are sparse", () => {
    const dates = publicAdminUsageDates(report([row("2026-07-20", "u1")]));

    expect(dates).toHaveLength(14);
    expect(dates.slice(0, 3)).toEqual([
      "2026-08-02",
      "2026-08-01",
      "2026-07-31",
    ]);
    expect(dates.at(-1)).toBe("2026-07-20");
  });

  it("keeps a selected zero-activity date available across refreshes", () => {
    const usage = report([row("2026-07-20", "u1")]);

    expect(publicAdminUsageDateIsAvailable(usage, "2026-08-02")).toBe(true);
    expect(publicAdminUsageDateIsAvailable(usage, "2026-08-01")).toBe(true);
    expect(publicAdminUsageDateIsAvailable(usage, "2026-07-19")).toBe(false);
  });

  it("keeps all rows or filters one Beijing date", () => {
    expect(filterPublicAdminUsageRows(rows, "all")).toHaveLength(3);
    expect(filterPublicAdminUsageRows(rows, "2026-08-02").map((item) => item.user_id)).toEqual([
      "u2",
      "u3",
    ]);
  });

  it("filters rows by channel and date with one shared scope", () => {
    const mixedRows = [
      row("2026-08-01", "u1", 2, 0, "网页用户", "web"),
      row("2026-08-01", "u2", 3, 0, "飞书用户", "feishu"),
      row("2026-08-02", "u3", 4, 0, "飞书用户二", "feishu"),
    ];

    expect(filterPublicAdminUsageRows(mixedRows, "all", "feishu")).toHaveLength(2);
    expect(
      filterPublicAdminUsageRows(mixedRows, "2026-08-01", "feishu").map(
        (item) => item.user_id,
      ),
    ).toEqual(["u2"]);
  });

  it("uses the backend period for longer zero-filled trends and date options", () => {
    const usage = {
      ...report([row("2026-07-04", "u1", 2)]),
      period_days: 30,
      period_start: "2026-07-04",
    };

    expect(publicAdminUsageDates(usage)).toHaveLength(30);
    expect(publicAdminUsageTrend(usage)).toHaveLength(30);
    expect(publicAdminUsageTrend(usage)[0]).toEqual({
      date: "2026-07-04",
      active_users: 1,
      question_count: 2,
    });
  });

  it("builds one shared 14-day trend with zero-filled dates and daily user deduplication", () => {
    const trend = publicAdminUsageTrend(
      report([
        row("2026-07-20", "u1", 2),
        row("2026-08-01", "u1", 1),
        row("2026-08-01", "u1", 3),
        row("2026-08-01", "u2", 1),
        row("2026-08-01", "push-only", 0, 2),
      ]),
    );

    expect(trend).toHaveLength(14);
    expect(trend[0]).toEqual({
      date: "2026-07-20",
      active_users: 1,
      question_count: 2,
    });
    expect(trend[12]).toEqual({
      date: "2026-08-01",
      active_users: 2,
      question_count: 5,
    });
    expect(trend[13]).toEqual({
      date: "2026-08-02",
      active_users: 0,
      question_count: 0,
    });
    expect(trend.filter((point) => point.question_count === 0)).toHaveLength(12);
  });

  it("treats the same external id on different channels as two users", () => {
    const usage = report([
      row("2026-08-02", "shared-id", 2, 0, "网页用户", "web"),
      row("2026-08-02", "shared-id", 3, 0, "飞书用户", "feishu"),
    ]);

    expect(publicAdminUsageTrend(usage).at(-1)).toEqual({
      date: "2026-08-02",
      active_users: 2,
      question_count: 5,
    });
    expect(summarizePublicAdminUsage(usage, "2026-08-02").active_users).toBe(2);
    expect(publicAdminUsageTrend(usage, "feishu").at(-1)).toEqual({
      date: "2026-08-02",
      active_users: 1,
      question_count: 3,
    });
    expect(
      summarizePublicAdminUsage(usage, "2026-08-02", "feishu").text,
    ).toContain("飞书 HONE 使用人数 1 人");
  });

  it("recomputes the top summary for the selected date and its prior-week comparison", () => {
    const usage = report([
      row("2026-07-26", "u1", 3, 0, "用户甲"),
      row("2026-07-26", "u2", 1, 0, "用户乙"),
      row("2026-08-02", "u1", 1, 2, "用户甲"),
    ]);

    const summary = summarizePublicAdminUsage(usage, "2026-08-02");

    expect(summary.active_users).toBe(1);
    expect(summary.question_count).toBe(1);
    expect(summary.delivered_push_count).toBe(2);
    expect(summary.comparison_user_change).toBe(-1);
    expect(summary.text).toContain("比上周同日少 1 人");
    expect(summary.text).toContain("用户甲 使用频率降低（少 2 次）");
    expect(summary.text).not.toContain("服务端摘要不应直接展示");
  });

  it("summarizes all rows while comparing the latest seven days with the prior seven", () => {
    const usage = report([
      row("2026-07-26", "u1", 3, 0, "用户甲"),
      row("2026-07-26", "u2", 1, 0, "用户乙"),
      row("2026-08-02", "u1", 1, 2, "用户甲"),
    ]);

    const summary = summarizePublicAdminUsage(usage, "all");

    expect(summary.active_users).toBe(2);
    expect(summary.question_count).toBe(5);
    expect(summary.delivered_push_count).toBe(2);
    expect(summary.comparison_user_change).toBe(-1);
    expect(summary.text).toContain("最近 14 天");
    expect(summary.text).toContain("最近 7 天比前 7 天少 1 人");
  });

  it("does not widen the previous-seven-day comparison in a 30-day report", () => {
    const usage = {
      ...report([
        row("2026-07-05", "old-user", 8),
        row("2026-07-26", "previous-user", 2),
        row("2026-08-02", "current-user", 1),
      ]),
      period_days: 30,
      period_start: "2026-07-04",
    };

    const summary = summarizePublicAdminUsage(usage, "all");

    expect(summary.comparison_user_change).toBe(0);
    expect(summary.text).toContain("最近 7 天比前 7 天持平");
  });

  it("does not turn a prior-week date outside the report window into a zero baseline", () => {
    const summary = summarizePublicAdminUsage(
      report([row("2026-07-22", "u1", 1)]),
      "2026-07-22",
    );

    expect(summary.comparison_user_change).toBeNull();
    expect(summary.text).toContain("暂无上周同日可比数据");
  });

  it("summarizes a selectable zero-activity date with zero values", () => {
    const summary = summarizePublicAdminUsage(
      report([row("2026-07-20", "u1")]),
      "2026-08-01",
    );

    expect(summary.active_users).toBe(0);
    expect(summary.question_count).toBe(0);
    expect(summary.delivered_push_count).toBe(0);
    expect(summary.text).toContain("8月1日");
    expect(summary.text).toContain("使用人数 0 人");
  });
});

// `locale` 是模块级 signal，整个 bun test 进程共用；不还原会泄漏到之后运行的
// 测试文件（本地/CI 文件顺序不同，表现为随机的跨文件失败）。
afterAll(() => {
  setLocale("en");
});
