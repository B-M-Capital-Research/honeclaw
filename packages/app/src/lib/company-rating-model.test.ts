import { describe, expect, test } from "bun:test";
import {
  coverageLabel,
  filterRatings,
  lightForScore,
  ratingCounts,
} from "./company-rating-model";
import type { CompanyRating, CompanyRatingSnapshot } from "./types";

function rating(symbol: string, score: number): CompanyRating {
  return {
    symbol,
    name: `${symbol} Inc.`,
    market_scope: "us_listed",
    theme: symbol === "AAA" ? "AI platform" : "semiconductor",
    value_chain: "test",
    score,
    light: lightForScore(score),
    confidence: "medium",
    data_status: "transcript_only",
    thesis_summary: "test",
    business_model: "test",
    moat: "test",
    valuation_method: "test",
    dimensions: {
      moat: 80,
      scarcity: 80,
      fundamentals: 80,
      visibility: 80,
      valuation: null,
      market_confirmation: 60,
    },
    valuation: null,
    valuation_unavailable_reason: "本日估值未更新，不参与评分。",
    watch_items: [],
    risks: [],
    falsifiers: [],
    research_updated_at: "2026-08-10",
    data_sources: [],
  };
}

describe("company rating model", () => {
  test("uses stable traffic-light thresholds", () => {
    expect(lightForScore(75)).toBe("green");
    expect(lightForScore(74.9)).toBe("yellow");
    expect(lightForScore(55)).toBe("yellow");
    expect(lightForScore(54.9)).toBe("red");
  });

  test("counts, filters, searches and sorts ratings", () => {
    const items = [rating("BBB", 60), rating("AAA", 90), rating("CCC", 30)];
    expect(ratingCounts(items)).toEqual({ green: 1, yellow: 1, red: 1, unknown: 0 });
    expect(filterRatings(items, "all", "").map((item) => item.symbol)).toEqual([
      "AAA",
      "BBB",
      "CCC",
    ]);
    expect(filterRatings(items, "green", "platform")).toHaveLength(1);
    expect(filterRatings(items, "red", "AAA")).toHaveLength(0);
  });

  test("keeps transcript-only research baselines out of formal traffic-light counts", () => {
    const item = rating("AAA", 90);
    item.data_status = "transcript_only";
    item.light = "unknown";
    item.factor_coverage = 3;
    expect(ratingCounts([item])).toEqual({ green: 0, yellow: 0, red: 0, unknown: 1 });
    expect(filterRatings([item], "green", "")).toHaveLength(0);
    expect(filterRatings([item], "unknown", "")).toHaveLength(1);
  });

  test("makes transcript-only coverage explicit", () => {
    const snapshot = {
      data_status: "transcript_only",
      coverage: { companies: 52, quotes: 0, financials: 0, valuations: 0 },
    } as CompanyRatingSnapshot;
    expect(coverageLabel(snapshot)).toContain("仅演讲研究基线");
    expect(coverageLabel(snapshot)).toContain("当日估值 0/52");
  });

  test("labels the local Codex simulation as non-real data", () => {
    const snapshot = {
      data_status: "simulation",
      coverage: { companies: 52, quotes: 0, financials: 52, valuations: 52 },
      simulation_note: "Codex 本地模拟预览：以下为非真实数据。",
    } as CompanyRatingSnapshot;
    expect(coverageLabel(snapshot)).toContain("Codex 模拟预览");
    expect(coverageLabel(snapshot)).toContain("8/8 因子");
    expect(coverageLabel(snapshot)).toContain("非真实数据");
  });
});
