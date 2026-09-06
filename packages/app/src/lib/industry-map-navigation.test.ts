import { describe, expect, it } from "bun:test";
import industryMap from "../../../../skills/industry-map/references/industry-map.json";
import { resolveIndustryMapLens, resolveIndustryMapSelection } from "./industry-map-navigation";

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

describe("industry map company lens", () => {
  const storage = industries.find((industry) => industry.id === "storage")!;

  it("accepts a member of the selected industry in any case and returns the canonical symbol", () => {
    expect(resolveIndustryMapLens(storage, "mu")).toBe("MU");
    expect(resolveIndustryMapLens(storage, " SNDK ")).toBe("SNDK");
  });

  it("ignores symbols from other industries, empty values, arrays and a missing industry", () => {
    expect(resolveIndustryMapLens(storage, "NVDA")).toBeUndefined();
    expect(resolveIndustryMapLens(storage, "")).toBeUndefined();
    expect(resolveIndustryMapLens(storage, ["MU", "SNDK"])).toBeUndefined();
    expect(resolveIndustryMapLens(storage, undefined)).toBeUndefined();
    expect(resolveIndustryMapLens(undefined, "MU")).toBeUndefined();
  });
});
