import { describe, expect, it } from "bun:test";
import industryMap from "../../../../skills/industry-map/references/industry-map.json";
import { resolveIndustryMapSelection } from "./industry-map-navigation";

const industries = industryMap.industries;

describe("industry map deep links", () => {
  it("opens every canonical industry in the research tree", () => {
    for (const industry of industries) {
      const url = new URL(`/industry-map?industry=${encodeURIComponent(industry.id)}`, "https://hone.test");
      expect(resolveIndustryMapSelection(industries, url.searchParams.get("industry") ?? undefined))
        .toBe(industry.id);
    }
  });

  it("resolves each navigation independently, including a return to the unfiltered page", () => {
    const history = [undefined, "optical", "storage", "optical", undefined];
    expect(history.map((query) => resolveIndustryMapSelection(industries, query)))
      .toEqual([industries[0].id, "optical", "storage", "optical", industries[0].id]);
  });

  it("falls back for unknown, empty and ambiguous industry parameters", () => {
    for (const query of ["removed-industry", "", "Optical", ["optical", "power"]]) {
      expect(resolveIndustryMapSelection(industries, query)).toBe(industries[0].id);
    }
  });

  it("resolves against the current snapshot when administrators add or remove industries", () => {
    const added = { id: "new-industry" };
    expect(resolveIndustryMapSelection([...industries, added], added.id)).toBe(added.id);
    expect(resolveIndustryMapSelection(industries, added.id)).toBe(industries[0].id);
    expect(resolveIndustryMapSelection(industries.filter((item) => item.id !== "optical"), "optical"))
      .toBe(industries[0].id);
  });

  it("does not invent a selected industry before data arrives or for an empty tree", () => {
    expect(resolveIndustryMapSelection([], "optical")).toBeUndefined();
    expect(resolveIndustryMapSelection([], undefined)).toBeUndefined();
  });
});
