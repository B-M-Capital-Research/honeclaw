import { describe, expect, it } from "bun:test";

import { briefToForm, contentAsOf, deriveIndustryBrief, recentChanges } from "./industry-brief";
import { valuationOf } from "./industry-valuation";
import type { Industry, IndustrySource, IndustryUpstreamSignal } from "./types";

const now = new Date("2026-09-06T00:00:00Z");

function signal(
  partial: Partial<IndustryUpstreamSignal> & Pick<IndustryUpstreamSignal, "symbol">,
): IndustryUpstreamSignal {
  return {
    name: "",
    relation: "peer_signal",
    why: "",
    pull: [],
    cadence: "",
    latest: "",
    latest_as_of: "",
    ...partial,
  };
}

function source(partial: Partial<IndustrySource> & Pick<IndustrySource, "url">): IndustrySource {
  return { house: "h", title: "t", date: "", takeaway: "k", ...partial };
}

function industry(partial: Partial<Industry>): Industry {
  return {
    id: "storage",
    name: "存储",
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
    members: [],
    sources: [],
    upstream_signals: [],
    valuation: valuationOf(undefined),
    ...partial,
  };
}

describe("recentChanges", () => {
  it("merges dated upstream actions and sources, newest first, actions before sources on the same day", () => {
    const view = recentChanges(
      industry({
        upstream_signals: [
          signal({ symbol: "MU", latest: "HBM 售罄", latest_as_of: "2026-06" }),
          signal({ symbol: "NVDA", latest: "数据中心 $89.0B", latest_as_of: "2026-08-26" }),
        ],
        sources: [
          source({ url: "a", date: "2026-08-26", takeaway: "同日的财报稿" }),
          source({ url: "b", date: "2026-07-16", takeaway: "台积电" }),
          source({ url: "c", date: "2024-03-25", takeaway: "两年前" }),
        ],
      }),
      now,
    );
    expect(view.items.map((item) => (item.kind === "signal" ? item.signal.symbol : item.source.url)))
      .toEqual(["NVDA", "a", "b", "MU", "c"]);
    expect(view.items.map((item) => item.asOf)).toEqual([
      "2026-08-26",
      "2026-08-26",
      "2026-07-16",
      "2026-06",
      "2024-03-25",
    ]);
    // 日粒度与月粒度混排：月按月初算。
    expect(view.items[3].ageDays).toBe(97);
    expect(view.items.map((item) => item.stale)).toEqual([false, false, false, false, true]);
    expect(view.rest).toBe(0);
    expect(view.undatedSignals).toEqual([]);
  });

  it("leaves signals without a dated action out of the list but reports them", () => {
    const empty = signal({ symbol: "TSM" });
    const undated = signal({ symbol: "AMZN", latest: "有动作但没写日期" });
    const garbage = signal({ symbol: "AMD", latest: "x", latest_as_of: "next quarter" });
    const view = recentChanges(industry({ upstream_signals: [empty, undated, garbage] }), now);
    expect(view.items).toEqual([]);
    expect(view.undatedSignals.map((item) => item.symbol)).toEqual(["TSM", "AMZN", "AMD"]);
  });

  it("caps the list and counts the rest; sources without a takeaway are not changes", () => {
    const sources = Array.from({ length: 7 }, (_, index) =>
      source({ url: String(index), date: `2026-0${(index % 6) + 1}-10`, takeaway: `t${index}` }),
    );
    sources.push(source({ url: "quiet", date: "2026-09-01", takeaway: " " }));
    const view = recentChanges(industry({ sources }), now);
    expect(view.items).toHaveLength(5);
    expect(view.rest).toBe(2);
    expect(view.items.some((item) => item.kind === "source" && item.source.url === "quiet")).toBe(false);
    // 不设上限时全部按日期倒序；同一天（url 0 与 6 都是 01-10）保持底稿顺序（稳定排序）。
    const all = recentChanges(industry({ sources }), now, 100);
    expect(all.items.map((item) => (item.kind === "source" ? item.source.url : "")))
      .toEqual(["5", "4", "3", "2", "1", "0", "6"]);
    expect(all.rest).toBe(0);
  });

  it("tolerates snapshots from older backends that have no upstream_signals field", () => {
    const view = recentChanges({ sources: [] } as unknown as Industry, now);
    expect(view).toEqual({ items: [], rest: 0, undatedSignals: [] });
  });
});

describe("deriveIndustryBrief", () => {
  it("prefers the administrator's brief and splits its body into paragraphs", () => {
    const view = deriveIndustryBrief(
      industry({
        one_liner: "不该出现",
        brief: {
          question: " 订单增加之后，收入兑现还卡在哪里？ ",
          body: "第一段\n\n  第二段  \n",
          next: [" 9 月下旬 MU 财报 ", "", "TSMC 月营收"],
          as_of: "2026-09-02",
        },
      }),
      now,
    );
    expect(view).toEqual({
      source: "brief",
      lead: "订单增加之后，收入兑现还卡在哪里？",
      body: ["第一段", "第二段"],
      next: ["9 月下旬 MU 财报", "TSMC 月营收"],
      asOf: "2026-09-02",
      changes: [],
    });
  });

  it("derives a first screen from the one-liner, the state, the newest two changes and the first watch item", () => {
    const view = deriveIndustryBrief(
      industry({
        one_liner: "让数据跟得上算力。",
        valuation: {
          ...valuationOf(undefined),
          logic: { ...valuationOf(undefined).logic, state_note: "量增价平" },
        },
        upstream_signals: [signal({ symbol: "NVDA", latest: "L", latest_as_of: "2026-08-26" })],
        sources: [
          source({ url: "a", date: "2026-08-04" }),
          source({ url: "b", date: "2026-06-24" }),
        ],
        core_watch: [{ what: "英伟达毛利率指引", why: "", cadence: "季度" }],
      }),
      now,
    );
    expect(view?.source).toBe("derived");
    expect(view?.lead).toBe("让数据跟得上算力。");
    // 典型 State 由页面正文单独展示，派生简报不重复。
    expect(view?.body).toEqual([]);
    expect(view?.next).toEqual(["英伟达毛利率指引（季度）"]);
    expect(view?.asOf).toBe("2026-08-26");
    expect(view?.changes.map((item) => item.asOf)).toEqual(["2026-08-26", "2026-08-04"]);
  });

  it("returns nothing for an industry with no material to summarise, and ignores an empty brief object", () => {
    expect(deriveIndustryBrief(industry({}), now)).toBeUndefined();
    // 只有一句话、没有任何带日期的变化或关注点：派生不出「当前重点」。
    expect(deriveIndustryBrief(industry({ one_liner: "一句话" }), now)).toBeUndefined();
    expect(
      deriveIndustryBrief(
        industry({ brief: { question: "", body: " ", next: [], as_of: "" } }),
        now,
      ),
    ).toBeUndefined();
    expect(
      deriveIndustryBrief(
        industry({ brief: null, one_liner: "一句话", core_watch: [{ what: "w", why: "", cadence: "" }] }),
        now,
      )?.lead,
    ).toBe("一句话");
  });
});

describe("briefToForm", () => {
  it("round-trips the four editable fields and tolerates a missing brief", () => {
    expect(briefToForm(undefined)).toEqual({ question: "", body: "", next: "", as_of: "" });
    expect(briefToForm({ question: "q", body: "b", next: ["1", "2"], as_of: "2026-09" })).toEqual({
      question: "q",
      body: "b",
      next: "1\n2",
      as_of: "2026-09",
    });
  });
});

describe("contentAsOf", () => {
  it("returns the newest parsable date across every dated field, preferring day precision on a tie", () => {
    expect(
      contentAsOf(
        industry({
          brief: { question: "q", body: "", next: [], as_of: "2026-06" },
          upstream_signals: [signal({ symbol: "NVDA", latest_as_of: "2026-08-26" })],
          core_watch: [{ what: "w", why: "", cadence: "", as_of: "garbage" }],
          sources: [source({ url: "a", date: "2026-08" })],
          ai_valuation_logic: {
            driver_chain: "",
            key_variables: [{ name: "n", why: "", where: "", as_of: "2026-09-02" }],
            multiple_anchor: "",
            anti_pattern: "",
            multiple_anchor_short: "",
            anti_pattern_short: "",
          },
        }),
        now,
      ),
    ).toBe("2026-09-02");
    expect(
      contentAsOf(
        industry({
          upstream_signals: [signal({ symbol: "NVDA", latest_as_of: "2026-08" })],
          sources: [source({ url: "a", date: "2026-08-01" })],
        }),
        now,
      ),
    ).toBe("2026-08-01");
    expect(contentAsOf(industry({}), now)).toBeUndefined();
  });
});
