import { describe, expect, it } from "bun:test";

import {
  lensSummary,
  memberMentionTokens,
  mentionsMember,
  rankByMention,
} from "./industry-lens";

describe("memberMentionTokens", () => {
  it("takes the symbol, the name outside the brackets and the first segment inside", () => {
    expect(memberMentionTokens({ symbol: "WDC", name: "西部数据（Western Digital）" })).toEqual([
      "WDC",
      "西部数据",
      "Western Digital",
    ]);
    expect(memberMentionTokens({ symbol: "TSM", name: "台积电（TSMC，美股 ADR；台股 2330.TW）" })).toEqual([
      "TSM",
      "台积电",
      "TSMC",
    ]);
    expect(memberMentionTokens({ symbol: "ALAB", name: "Astera Labs" })).toEqual(["ALAB", "Astera Labs"]);
    // 中文全名再给一个去掉后缀的简称，正文里通常只写简称。
    expect(memberMentionTokens({ symbol: "MU", name: "美光科技（Micron）" })).toEqual([
      "MU",
      "美光科技",
      "美光",
      "Micron",
    ]);
    expect(memberMentionTokens({ symbol: "ASX", name: "日月光投控（ASE Technology）" })).toEqual([
      "ASX",
      "日月光投控",
      "日月光",
      "ASE Technology",
    ]);
  });

  it("drops listing-venue fragments and anything shorter than two characters", () => {
    expect(memberMentionTokens({ symbol: "ARM", name: "Arm Holdings（英国公司，美股 ADS）" })).toEqual([
      "ARM",
      "Arm Holdings",
    ]);
    expect(memberMentionTokens({ symbol: "GE", name: "G" })).toEqual(["GE"]);
  });
});

describe("mentionsMember", () => {
  const mu = { symbol: "MU", name: "美光科技（Micron）" };

  it("matches the symbol only on word boundaries and the name fragments as substrings", () => {
    expect(mentionsMember("三大 HBM 厂里唯一的美股 MU", mu)).toBe(true);
    expect(mentionsMember("mu 小写也算", mu)).toBe(true);
    expect(mentionsMember("MUX 和 MUSK 都不是它", { symbol: "MU", name: "" })).toBe(false);
    expect(mentionsMember("MUX 不是，但句尾的 MU 是", { symbol: "MU", name: "" })).toBe(true);
    expect(mentionsMember("美光的 HBM 是否售罄", mu)).toBe(true);
    expect(mentionsMember("micron contract prices", mu)).toBe(true);
    expect(mentionsMember("SK 海力士与三星", mu)).toBe(false);
    expect(mentionsMember("", mu)).toBe(false);
  });

  it("does not let a company name collapse into a common word", () => {
    expect(mentionsMember("networks are everywhere", { symbol: "ANET", name: "Arista Networks" })).toBe(false);
    expect(mentionsMember("Arista Networks 的交换机", { symbol: "ANET", name: "Arista Networks" })).toBe(true);
    expect(mentionsMember("warm charm farm", { symbol: "ARM", name: "Arm Holdings（英国公司）" })).toBe(false);
    expect(mentionsMember("GE Vernova 燃机 backlog", { symbol: "GEV", name: "GE Vernova" })).toBe(true);
  });
});

describe("rankByMention", () => {
  const items = [
    { id: "a", text: "TSMC 月营收" },
    { id: "b", text: "美光 HBM" },
    { id: "c", text: "英伟达毛利率" },
    { id: "d", text: "MU capex" },
  ];

  it("moves related items to the front and keeps both groups in their original order", () => {
    const ranked = rankByMention(items, (item) => item.text, { symbol: "MU", name: "美光科技（Micron）" });
    expect(ranked.map((row) => `${row.item.id}:${row.related}`)).toEqual([
      "b:true",
      "d:true",
      "a:false",
      "c:false",
    ]);
  });

  it("is the identity without a selected company", () => {
    const ranked = rankByMention(items, (item) => item.text, undefined);
    expect(ranked.map((row) => row.item.id)).toEqual(["a", "b", "c", "d"]);
    expect(ranked.every((row) => !row.related)).toBe(true);
  });
});

describe("lensSummary", () => {
  it("counts related items per block, reading every text field the page shows", () => {
    const summary = lensSummary(
      {
        upstream_signals: [
          { symbol: "NVDA", name: "英伟达", relation: "peer_signal", why: "x", pull: ["MU 的 HBM 份额"], cadence: "", latest: "", latest_as_of: "" },
          { symbol: "MSFT", name: "微软", relation: "capex_source", why: "y", pull: [], cadence: "", latest: "", latest_as_of: "" },
        ],
        core_watch: [
          { what: "美光 capex", why: "", cadence: "" },
          { what: "PJM 拍卖", why: "", cadence: "" },
        ],
        ai_valuation_logic: {
          driver_chain: "",
          key_variables: [{ name: "HBM 占比", why: "", where: "美光季报" }],
          multiple_anchor: "",
          anti_pattern: "",
          multiple_anchor_short: "",
          anti_pattern_short: "",
        },
        sources: [{ house: "Micron", title: "", date: "", url: "u", takeaway: "" }],
      },
      { symbol: "MU", name: "美光科技（Micron）" },
    );
    expect(summary).toEqual({ signals: 1, watch: 1, variables: 1, sources: 1 });
  });
});
