import { describe, expect, it } from "bun:test";
import industryMap from "../../../../skills/industry-map/references/industry-map.json";
import { DATA_CENTER_ZONES, industryHref } from "./data-center-model";

describe("data center industry navigation", () => {
  it("links every spatial area to a real industry with its canonical name", () => {
    const industries = new Map(industryMap.industries.map((industry) => [industry.id, industry.name]));
    for (const zone of DATA_CENTER_ZONES) {
      expect(zone.industries.length).toBeGreaterThan(0);
      for (const industry of zone.industries) {
        expect(industries.get(industry.id)).toBe(industry.name);
        const target = new URL(industryHref(industry.id), "https://hone.example");
        expect(target.pathname).toBe("/industry-map");
        expect(target.searchParams.get("industry")).toBe(industry.id);
      }
    }
  });

  it("keeps cooling and software on their existing industry research routes", () => {
    const industryIds = (zoneId: string) =>
      DATA_CENTER_ZONES.find((zone) => zone.id === zoneId)?.industries.map((industry) => industry.id);
    expect(industryIds("cooling")).toEqual(["power"]);
    expect(industryIds("software")).toEqual(["hyperscaler", "neocloud"]);
    expect(industryIds("chip")).toEqual(["ai-chip", "server-oem", "equipment"]);
  });

  it("gives model selection unique identities and makes all existing industries reachable", () => {
    const zoneIds = DATA_CENTER_ZONES.map((zone) => zone.id);
    expect(new Set(zoneIds).size).toBe(zoneIds.length);
    const reachable = new Set(DATA_CENTER_ZONES.flatMap((zone) => zone.industries.map((industry) => industry.id)));
    expect([...reachable].sort()).toEqual(industryMap.industries.map((industry) => industry.id).sort());
  });

  it("encodes industry IDs as a single query value", () => {
    const id = "光通信 & advanced/network?view=full#detail";
    const target = new URL(industryHref(id), "https://hone.example");
    expect(target.pathname).toBe("/industry-map");
    expect([...target.searchParams]).toEqual([["industry", id]]);
    expect(target.hash).toBe("");
  });
});
