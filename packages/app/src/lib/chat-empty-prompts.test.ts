import { describe, expect, it } from "bun:test";
import { buildChatStarterPrompts } from "./chat-empty-prompts";
import type { InfluencerDigestSnapshot } from "./types";

const digest = (text: string, tickers: string[] = []): InfluencerDigestSnapshot => ({
  report_date: "2026-09-06",
  generated_at: "2026-09-06T04:40:00Z",
  generated_at_local: "2026-09-06 12:40",
  timezone: "Asia/Shanghai",
  next_refresh_at: "2026-09-06T04:55:00Z",
  lookback_hours: 168,
  refresh_interval_minutes: 15,
  model_version: "hone-influencer-digest-v1",
  status: "live",
  summary: "",
  coverage: { authors: 3, configured: 2, succeeded: 2, items: 1, analyzed: 1 },
  authors: [],
  items: [
    {
      id: "serenity:1",
      author_id: "serenity",
      author_name: "Serenity / 白毛",
      public_handle: "@aleabitoreddit",
      title: text,
      published_at: "2026-09-06T04:26:54Z",
      published_at_local: "09-06 12:26",
      source_url: "https://x.com/aleabitoreddit/status/1",
      post_kind: "original",
      source_excerpt: text,
      source_text_cn: text,
      summary: "",
      stance: "bullish",
      horizon: "medium",
      content_type: "opinion",
      topics: [],
      tickers,
      counterpoint: "",
      analysis_status: "model_analyzed",
    },
  ],
  disclaimer: "",
});

describe("empty conversation prompts", () => {
  it("always returns seven actionable investment hooks", () => {
    const prompts = buildChatStarterPrompts({ today: "2026-08-11" });
    expect(prompts).toHaveLength(7);
    expect(prompts.map((item) => item.id)).toEqual([
      "macro",
      "portfolio",
      "calendar",
      "influencer",
      "ai-infra",
      "industry",
      "valuation",
    ]);
    expect(prompts.every((item) => item.question.length > item.title.length)).toBe(true);
  });

  it("uses actor holdings without inventing unsupported symbols", () => {
    const prompts = buildChatStarterPrompts({
      holdings: ["nvda", "AMD", "NVDA", "mu"],
      today: "2026-08-11",
    });
    expect(prompts[1]?.title).toContain("NVDA、AMD、MU");
    expect(prompts[6]?.title).toContain("NVDA");
    expect(prompts.map((item) => item.question).join("\n")).not.toContain("TSLA");
  });

  it("uses the nearest sourced calendar row and preserves uncertainty", () => {
    const prompts = buildChatStarterPrompts({
      today: "2026-08-11",
      events: [
        { date: "2026-08-20T02:00:00", title: "FOMC 会议纪要", kind: "macro", source: "Fed" },
        { date: "2026-08-18T20:30:00", title: "美国新屋开工", kind: "macro", source: "Census" },
      ],
    });
    expect(prompts[2]?.title).toContain("美国新屋开工");
    expect(prompts[2]?.question).toContain("2026-08-18");
    expect(prompts[2]?.question).toContain("尚未公布的结果不要猜");
  });

  it("quotes Serenity's newest post when the digest is available", () => {
    const withPost = buildChatStarterPrompts({
      today: "2026-09-06",
      influencer: digest("现在 $MU 到 $SNDK 已经大幅反弹。\n内存瓶颈并未改变！ https://t.co/x", ["MU", "SNDK"]),
    });
    const hook = withPost[3]!;
    expect(hook.id).toBe("influencer");
    expect(hook.title).toContain("白毛 09-06 12:26");
    // First sentence, whole — never a cut mid-clause.
    expect(hook.title).toContain("已经大幅反弹。");
    expect(hook.title).not.toContain("内存瓶");
    expect(hook.question).toContain("内存瓶颈并未改变");
    // Links are noise in a question; the post reads as one line.
    expect(hook.question).not.toContain("https://t.co");
    expect(hook.question).toContain("$MU、$SNDK");
    expect(hook.question).toContain("哪些是事实、哪些是作者判断");

    // Without a snapshot the hook still asks a concrete question.
    const without = buildChatStarterPrompts({ today: "2026-09-06", influencer: null })[3]!;
    expect(without.title).toContain("Serenity");
    expect(without.question).toContain("最近三天");
  });

  it("offers an AI infrastructure chain question", () => {
    const hook = buildChatStarterPrompts({ today: "2026-09-06" })[4]!;
    expect(hook.id).toBe("ai-infra");
    expect(hook.question).toContain("HBM");
    expect(hook.question).toContain("光模块");
    expect(hook.question).toContain("资本开支");
  });

  it("returns English hooks when the workspace locale is English", () => {
    const prompts = buildChatStarterPrompts({
      holdings: ["NVDA"],
      today: "2026-08-11",
      locale: "en",
      influencer: digest("Memory bottleneck has not changed", ["MU"]),
    });
    expect(prompts).toHaveLength(7);
    expect(prompts[1]?.title).toContain("NVDA");
    expect(prompts[3]?.title).toContain("Serenity");
    expect(prompts.map((item) => `${item.eyebrow} ${item.title} ${item.question}`).join("\n"))
      .not.toMatch(/[㐀-鿿]/);
  });
});
