import { describe, expect, it } from "bun:test";
import {
  calendarToWorkspaceEvents,
  communityToWorkspaceInsights,
  daySeparatorLabel,
  groupResearchByDate,
  workspaceGreeting,
  workspaceUserName,
} from "./public-agent-workspace";

describe("public Agent workspace helpers", () => {
  it("turns community posts into compact insight rows", () => {
    const insights = communityToWorkspaceInsights([
      {
        content_id: 8,
        author_name: "HONE",
        content_type: "post",
        body_text: "老王更新了 HBM 深度解读。已经匹配到 NVDA、AMD 与 TSMC 投资主线。",
        body_blocks: [],
        crawl_status: "complete",
        resources: [],
      },
    ]);
    expect(insights[0]).toEqual({
      id: "8",
      eyebrow: "社区新帖",
      title: "老王更新了 HBM 深度解读",
      summary: "已经匹配到 NVDA、AMD 与 TSMC 投资主线。",
    });
  });

  it("keeps only upcoming calendar events in chronological order", () => {
    const events = calendarToWorkspaceEvents(
      [
        { date: "2026-07-24T09:00", title: "NVIDIA GTC", kind: "macro", source: "HONE" },
        { date: "2026-07-10T08:00", title: "过去事件", kind: "macro", source: "HONE" },
        { date: "2026-07-17T08:00", title: "TSMC Q2 财报", kind: "earnings", ticker: "TSM", source: "FMP" },
      ],
      "2026-07-13",
    );
    expect(events.map((event) => event.title)).toEqual([
      "TSMC Q2 财报",
      "NVIDIA GTC",
    ]);
    expect(events[0]?.date).toBe("07/17");
  });

  it("uses privacy-safe display names and time-aware greetings", () => {
    expect(workspaceUserName("web-user-e05f5e5f74a3")).toBe("HONE 用户");
    expect(workspaceUserName("13871396421")).toBe("用户 6421");
    expect(workspaceGreeting(14, "老王")).toBe("下午好，老王");
  });

  it("groups research records into today / yesterday / week / earlier buckets", () => {
    const now = new Date("2026-07-26T15:00:00+08:00").getTime();
    const groups = groupResearchByDate(
      [
        { id: "a", title: "今天的问题", at: "2026-07-26T09:00:00+08:00" },
        { id: "b", title: "昨天的问题", at: "2026-07-25T22:00:00+08:00" },
        { id: "c", title: "周内的问题", at: "2026-07-21T08:00:00+08:00" },
        { id: "d", title: "很久以前", at: "2026-06-01T08:00:00+08:00" },
        { id: "e", title: "没有时间戳" },
      ],
      now,
    );

    expect(groups.map((group) => group.label)).toEqual([
      "今天",
      "昨天",
      "近 7 天",
      "更早",
    ]);
    expect(groups[3]?.items.map((item) => item.id)).toEqual(["d", "e"]);
  });

  it("omits empty research groups", () => {
    const now = new Date("2026-07-26T15:00:00+08:00").getTime();
    const groups = groupResearchByDate(
      [{ id: "a", title: "今天", at: "2026-07-26T09:00:00+08:00" }],
      now,
    );
    expect(groups).toHaveLength(1);
    expect(groups[0]?.label).toBe("今天");
  });

  it("emits a day separator only when the timeline crosses a day", () => {
    const now = new Date("2026-07-26T15:00:00+08:00").getTime();
    expect(daySeparatorLabel(undefined, "2026-07-26T09:00:00+08:00", now)).toBe("今天");
    expect(
      daySeparatorLabel(
        "2026-07-26T09:00:00+08:00",
        "2026-07-26T10:00:00+08:00",
        now,
      ),
    ).toBeNull();
    expect(
      daySeparatorLabel(
        "2026-07-24T09:00:00+08:00",
        "2026-07-25T10:00:00+08:00",
        now,
      ),
    ).toBe("昨天");
    expect(
      daySeparatorLabel(
        "2025-12-31T23:00:00+08:00",
        "2026-07-01T10:00:00+08:00",
        now,
      ),
    ).toBe("7月1日");
    expect(daySeparatorLabel(undefined, undefined, now)).toBeNull();
  });
});

describe("a workspace header never takes down its route", () => {
  it("falls back to the generic name instead of throwing", () => {
    // Every workspace surface renders its header through this, so an
    // exception here blanks the whole page rather than one label.
    expect(workspaceUserName(undefined)).toBe("HONE 用户");
    expect(workspaceUserName(null)).toBe("HONE 用户");
    expect(workspaceUserName("   ")).toBe("HONE 用户");
  });
});
