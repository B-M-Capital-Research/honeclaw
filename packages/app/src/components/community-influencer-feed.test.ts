import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";
import { communityViewFromParam } from "@/lib/public-community-timeline";

const read = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");
const feed = read("./community-influencer-feed.tsx");
const page = read("../pages/public-community.tsx");

describe("Insights influencer stream", () => {
  it("parses the ?view= deep link and falls back to the official feed", () => {
    expect(communityViewFromParam("influencer")).toBe("influencer");
    expect(communityViewFromParam("forum")).toBe("forum");
    expect(communityViewFromParam("official")).toBe("official");
    expect(communityViewFromParam(undefined)).toBe("official");
    expect(communityViewFromParam("anything")).toBe("official");
  });

  it("renders the 大V速报 snapshot as its own tab, mounted only when chosen", () => {
    expect(page).toContain("<strong>大V速报</strong>");
    expect(page).toContain('<Show when={communityView() === "influencer"}>');
    expect(page).toContain("<CommunityInfluencerFeed query={query()} />");
    // The background refresh of the official feed still skips other tabs.
    expect(page).toContain('communityView() !== "official"');
  });

  it("reads the shared influencer snapshot and never touches the official timeline", () => {
    expect(feed).toContain("createInfluencerDigestView({ query: () => props.query })");
    expect(feed).toContain("<InfluencerTimeline");
    expect(feed).toContain("/chat?q=${encodeURIComponent(question)}&send=1");
    // Official-feed cursor, read receipts and edge snapshots stay HONE-only.
    expect(feed).not.toContain("getPublicCommunity");
    expect(feed).not.toContain("markPublicCommunitySeen");
    expect(feed).not.toContain("mergeCommunityTimeline");
    // Author opinion is never presented as an official read-only card.
    expect(feed).not.toContain("public-community-card");
    expect(feed).not.toContain('role="alert"');
    expect(feed).toContain("不代表 HONE 判断");
  });
});
