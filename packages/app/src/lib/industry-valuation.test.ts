import { describe, expect, it } from "bun:test";

import type { Industry, IndustrySubtype } from "./types";
import {
  askHoneHref,
  askHonePrompt,
  findMember,
  isInferredMember,
  isLatestStale,
  isSubtypeId,
  latestAgeDays,
  matchLabel,
  methodologyOf,
  splitSymbols,
  subtypeOf,
  valuationOf,
} from "./industry-valuation";

function subtype(partial: Partial<IndustrySubtype> & Pick<IndustrySubtype, "id" | "name">): IndustrySubtype {
  return {
    members: [],
    inferred_members: [],
    primary: "",
    secondary: "",
    when: "",
    note: "",
    ...partial,
  };
}

function industry(
  partial: Partial<Industry> & Pick<Industry, "id" | "name" | "members">,
): Industry {
  return {
    parent: "root",
    one_liner: "",
    ai_valuation_logic: {
      driver_chain: "",
      key_variables: [],
      multiple_anchor: "",
      anti_pattern: "",
      multiple_anchor_short: "",
      anti_pattern_short: "",
    },
    core_watch: [],
    sources: [],
    upstream_signals: [],
    valuation: valuationOf(undefined),
    ...partial,
  };
}

const storage = industry({
  id: "storage",
  name: "存储",
  members: [
    { symbol: "MU", name: "美光（Micron）", role: "" },
    { symbol: "SNDK", name: "闪迪（SanDisk）", role: "" },
    { symbol: "STX", name: "希捷（Seagate）", role: "" },
    { symbol: "WDC", name: "西部数据（Western Digital）", role: "" },
  ],
  valuation: {
    ...valuationOf(undefined),
    subtypes: [
      subtype({ id: "dram-nand", name: "DRAM+NAND", members: ["MU"], inferred_members: [] }),
      subtype({ id: "pure-nand", name: "纯NAND/eSSD", members: ["SNDK"], inferred_members: ["WDC"] }),
    ],
  },
});

const optical = industry({
  id: "optical",
  name: "光通信",
  members: [
    { symbol: "COHR", name: "Coherent", role: "" },
    { symbol: "LITE", name: "Lumentum", role: "" },
  ],
});

// Deliberately listed after storage so a prefix hit in storage wins over a name hit here.
const industries = [storage, optical];

describe("findMember", () => {
  it("matches a symbol exactly regardless of case", () => {
    const hit = findMember(industries, "sndk");
    expect(hit?.member.symbol).toBe("SNDK");
    expect(hit?.industry.id).toBe("storage");
    expect(hit?.subtype?.id).toBe("pure-nand");
  });

  it("prefers an exact symbol over a prefix on a longer one", () => {
    // "ST" is a prefix of STX; an exact "STX" must not be beaten by an earlier prefix hit.
    expect(findMember(industries, "STX")?.member.symbol).toBe("STX");
    expect(findMember(industries, "st")?.member.symbol).toBe("STX");
  });

  it("matches a symbol prefix before falling back to a name substring", () => {
    const hit = findMember(industries, "LI");
    expect(hit?.member.symbol).toBe("LITE");
    expect(hit?.industry.id).toBe("optical");
    // No subtypes on this row: the label degrades to two parts.
    expect(hit && matchLabel(hit)).toBe("LITE · 光通信");
  });

  it("matches a Chinese name substring", () => {
    const hit = findMember(industries, "闪迪");
    expect(hit?.member.symbol).toBe("SNDK");
    expect(hit && matchLabel(hit)).toBe("SNDK · 存储 · 纯NAND/eSSD");
    expect(findMember(industries, "western")?.member.symbol).toBe("WDC");
  });

  it("returns nothing for an empty query or one nobody matches", () => {
    expect(findMember(industries, "")).toBeUndefined();
    expect(findMember(industries, "   ")).toBeUndefined();
    expect(findMember(industries, "NVDA")).toBeUndefined();
    expect(findMember(industries, "台积电")).toBeUndefined();
    expect(findMember([], "SNDK")).toBeUndefined();
  });
});

describe("subtypeOf", () => {
  it("finds explicit members, then inferred ones, case-insensitively", () => {
    expect(subtypeOf(storage, "MU")?.id).toBe("dram-nand");
    expect(subtypeOf(storage, "wdc")?.id).toBe("pure-nand");
    expect(subtypeOf(storage, "STX")).toBeUndefined();
    expect(subtypeOf(optical, "COHR")).toBeUndefined();
    expect(subtypeOf({}, "SNDK")).toBeUndefined();
  });

  it("tells an inferred member from an explicit one", () => {
    const pure = storage.valuation.subtypes[1];
    expect(isInferredMember(pure, "WDC")).toBe(true);
    expect(isInferredMember(pure, "SNDK")).toBe(false);
    expect(isInferredMember(pure, "MU")).toBe(false);
  });
});

describe("older backends", () => {
  it("renders empty structures instead of crashing on a snapshot without V3 fields", () => {
    const legacy = { id: "x" } as Partial<Industry>;
    expect(valuationOf(legacy).subtypes).toEqual([]);
    expect(valuationOf(legacy).anchor.forbidden).toEqual([]);
    expect(valuationOf(legacy).logic.summary).toBe("");
    expect(methodologyOf(undefined).execution_rules).toEqual([]);
    expect(methodologyOf({}).demand_chain).toEqual([]);
  });
});

describe("latest_as_of freshness", () => {
  const today = new Date("2026-09-06T00:00:00Z");

  it("ages a day or a month stamp and flags anything older than 100 days", () => {
    expect(latestAgeDays("2026-08-26", today)).toBe(11);
    expect(latestAgeDays("2026-06", today)).toBe(97);
    expect(isLatestStale("2026-08-26", today)).toBe(false);
    expect(isLatestStale("2026-06", today)).toBe(false);
    expect(isLatestStale("2026-05-01", today)).toBe(true);
    expect(isLatestStale("2025-12", today)).toBe(true);
  });

  it("never flags a stamp it cannot parse", () => {
    for (const text of ["", "  ", "FY26Q3", "最近", "2026年8月"]) {
      expect(latestAgeDays(text, today)).toBeUndefined();
      expect(isLatestStale(text, today)).toBe(false);
    }
  });
});

describe("ask HONE", () => {
  it("carries the symbol, name and subtype into an auto-sent chat prompt", () => {
    const prompt = askHonePrompt("SNDK", "闪迪（SanDisk）", "纯NAND/eSSD");
    expect(prompt).toBe(
      "按行业本体给 SNDK（闪迪（SanDisk））做前瞻估值：先判 State 与 Capacity Unlock Gate，按子类型「纯NAND/eSSD」的主锚次锚给 Valuation Horizon、Bear/Base/Bull 与 Market Implied，写明上游最近动作。",
    );
    const href = askHoneHref(prompt);
    expect(href.startsWith("/chat?q=")).toBe(true);
    expect(href.endsWith("&send=1")).toBe(true);
    expect(new URLSearchParams(href.slice(href.indexOf("?"))).get("q")).toBe(prompt);
  });
});

describe("subtype form inputs", () => {
  it("accepts only lowercase kebab-case ids, like the backend", () => {
    for (const ok of ["pure-nand", "dram", "hbm-3e", "a1"]) expect(isSubtypeId(ok)).toBe(true);
    for (const bad of ["", "Pure-NAND", "pure_nand", "-nand", "nand-", "纯nand", "a b"]) {
      expect(isSubtypeId(bad)).toBe(false);
    }
  });

  it("reads members one per line or comma-separated, uppercased and deduplicated", () => {
    expect(splitSymbols("sndk\nWDC, mu，stx\n\nSNDK")).toEqual(["SNDK", "WDC", "MU", "STX"]);
    expect(splitSymbols("")).toEqual([]);
  });
});
