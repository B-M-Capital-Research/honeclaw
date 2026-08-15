import { describe, expect, it } from "bun:test";
import { buildChatStarterPrompts } from "./chat-empty-prompts";

describe("empty conversation prompts", () => {
  it("always returns five actionable investment hooks", () => {
    const prompts = buildChatStarterPrompts({ today: "2026-08-11" });
    expect(prompts).toHaveLength(5);
    expect(prompts.map((item) => item.id)).toEqual([
      "macro",
      "portfolio",
      "calendar",
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
    expect(prompts[4]?.title).toContain("NVDA");
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

  it("returns English hooks when the workspace locale is English", () => {
    const prompts = buildChatStarterPrompts({
      holdings: ["NVDA"],
      today: "2026-08-11",
      locale: "en",
    });
    expect(prompts).toHaveLength(5);
    expect(prompts[1]?.title).toContain("NVDA");
    expect(prompts.map((item) => `${item.eyebrow} ${item.title} ${item.question}`).join("\n"))
      .not.toMatch(/[\u3400-\u9fff]/);
  });
});
