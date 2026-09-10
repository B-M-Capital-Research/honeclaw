import { describe, expect, it } from "bun:test";

import type { Industry, IndustrySubtype } from "./types";
import {
  askHoneHref,
  askHonePrompt,
  askNextStepPrompt,
  askSignalPrompt,
  askSourcePrompt,
  askWatchPrompt,
  relationLabel,
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
    // V5.3 fields default to empty on old snapshots …
    expect(valuationOf(legacy).observables).toEqual([]);
    expect(valuationOf(legacy).transmission).toBe("");
    expect(methodologyOf({}).acceptance_cases).toEqual([]);
    // … and survive normalisation when the backend sends them: the dossier
    // renders from valuationOf(), so dropping them here hid every V5.3 block.
    const observable = {
      name: "净 ASP",
      definition: "按产品的季度均价",
      source: "财报",
      cadence: "季度",
      transmission: "收入 = bits × ASP",
    };
    const v53 = valuationOf({
      valuation: {
        ...valuationOf(undefined),
        observables: [observable],
        transmission: "先量后价",
        upstream_summary: "需求从训练任务传到 HBM",
        subtype_intro: "按角色分",
        sources_note: "看 10-K",
      },
    });
    expect(v53.observables).toEqual([observable]);
    expect(v53.transmission).toBe("先量后价");
    expect(v53.upstream_summary).toBe("需求从训练任务传到 HBM");
    expect(v53.subtype_intro).toBe("按角色分");
    expect(v53.sources_note).toBe("看 10-K");
    const rule = { rule: "HBM 晶圆强度", requirement: "按层数与良率折算" };
    expect(
      methodologyOf({
        methodology: { ...methodologyOf(undefined), technical_conventions: [rule], references: [rule] },
      }).technical_conventions,
    ).toEqual([rule]);
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

describe("relation labels", () => {
  it("translates the four relations and echoes anything else unchanged", () => {
    expect(["demand_source", "capex_source", "supply_gate", "peer_signal"].map(relationLabel)).toEqual([
      "需求来源",
      "资本开支来源",
      "供给卡口",
      "同业信号",
    ]);
    expect(relationLabel("customer")).toBe("customer");
  });
});

describe("contextual ask HONE prompts", () => {
  const nvda = {
    symbol: "NVDA",
    name: "英伟达",
    relation: "peer_signal",
    latest: "FY27Q2：数据中心收入 $89.0B（同比 +117%），Q3 指引 $108B。",
    latest_as_of: "2026-08-26",
    pull: ["数据中心收入与环比", " 毛利率指引 ", ""],
  };

  it("asks about a dated upstream action, naming the focus company when there is one", () => {
    const prompt = askSignalPrompt("存储", nvda, { symbol: "MU", name: "美光科技" });
    expect(prompt).toContain("NVDA（英伟达）截至 2026-08-26 的最近动作：「FY27Q2：数据中心收入 $89.0B");
    expect(prompt).toContain("它是存储的同业信号。");
    expect(prompt).toContain("这会影响 MU（美光科技）哪部分业务、传导要几个季度？");
    expect(prompt).toContain("再列要核的读数：数据中心收入与环比；毛利率指引");
    expect(prompt).not.toContain("undefined");
    expect(askHoneHref(prompt)).toMatch(/^\/chat\?q=.*&send=1$/);
    expect(askHoneHref(prompt).length).toBeLessThan(2000);
  });

  it("degrades cleanly when the action, the date, the relation or the pull list are missing", () => {
    const prompt = askSignalPrompt("存储", { symbol: "TSM", name: "", relation: "", latest: "", latest_as_of: "", pull: [] });
    expect(prompt).toBe("TSM最近一季实际做了什么？这先影响存储里哪几家公司、哪部分业务？按行业本体给结论。");
    expect(prompt).not.toContain("截至");
    expect(prompt).not.toContain("（）");
  });

  it("asks about a source with its takeaway clipped and the focus company named", () => {
    const prompt = askSourcePrompt(
      "AI 芯片",
      { house: "Broadcom（SEC 8-K）", title: "FY2026 Q3", date: "2026-09-02", takeaway: "x".repeat(300) },
      { symbol: "AVGO", name: "博通" },
    );
    expect(prompt.startsWith("Broadcom（SEC 8-K）｜FY2026 Q3（2026-09-02）。要点：")).toBe(true);
    expect(prompt).toContain("这对 AVGO（博通） 意味着什么、改变了哪条假设？");
    expect(prompt.endsWith("按行业本体给结论，并说明还要核什么。")).toBe(true);
    expect(prompt.length).toBeLessThan(400);
    expect(askSourcePrompt("AI 芯片", { house: "h", title: "t", date: "", takeaway: "" })).toBe(
      "h｜t。这改变了AI 芯片的哪条假设、先影响哪几家公司？按行业本体给结论，并说明还要核什么。",
    );
  });

  it("asks about a watch item with the reason clipped, defaulting the target to the industry", () => {
    const long = "理由".repeat(200);
    const prompt = askWatchPrompt("电力", { what: "PJM 容量拍卖", why: long, cadence: "每年 7 月" });
    expect(prompt.startsWith("电力的关注点「PJM 容量拍卖」（每年 7 月）：最近一期读数是多少、相对上一期变了什么、对 这一行 的估值结论要不要改？看它的原因：")).toBe(true);
    expect(prompt.endsWith("…")).toBe(true);
    expect(prompt.length).toBeLessThan(long.length);
    expect(askWatchPrompt("电力", { what: "w", why: "", cadence: "" }, { symbol: "VST", name: "" })).toBe(
      "电力的关注点「w」：最近一期读数是多少、相对上一期变了什么、对 VST 的估值结论要不要改？",
    );
  });

  it("turns a next step into a task with the brief's lead as context", () => {
    expect(askNextStepPrompt("AI 芯片", "订单增加之后，收入兑现还卡在哪里？", " 9 月下旬 MU 财报 ")).toBe(
      "AI 芯片当前重点：订单增加之后，收入兑现还卡在哪里？。请完成下一步「9 月下旬 MU 财报」，给出数据、来源与对估值结论的影响。",
    );
    expect(askNextStepPrompt("AI 芯片", "", "x")).toBe("AI 芯片：请完成下一步「x」，给出数据、来源与对估值结论的影响。");
  });
});
