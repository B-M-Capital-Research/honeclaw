import { describe, expect, it } from "bun:test";
import {
  calendarToWorkspaceEvents,
  communityToWorkspaceInsights,
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
      eyebrow: "新洞察",
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
});
