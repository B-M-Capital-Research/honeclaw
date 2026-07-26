import { describe, expect, it } from "bun:test";

import {
  canAddHolding,
  formatHoldingCost,
  formatHoldingWeight,
  holdingAskPrompt,
  totalHoldingWeight,
  validateHoldingForm,
  type HoldingRow,
} from "./public-holdings-model";

const position: HoldingRow = {
  symbol: "AAPL",
  name: "苹果",
  weight: 25.5,
  avg_cost: 180.25,
  tracking_only: false,
};
const watch: HoldingRow = {
  symbol: "TSLA",
  name: null,
  weight: null,
  avg_cost: null,
  tracking_only: true,
};

describe("holdings model", () => {
  it("shows a percentage for positions and 自选 for watchlist rows", () => {
    expect(formatHoldingWeight(position)).toBe("25.5%");
    expect(formatHoldingWeight(watch)).toBe("自选");
    expect(formatHoldingWeight({ ...position, weight: null })).toBe("自选");
  });

  it("only shows cost when it is a positive number", () => {
    expect(formatHoldingCost(position)).toBe("成本 180.25");
    expect(formatHoldingCost(watch)).toBeNull();
    expect(formatHoldingCost({ ...position, avg_cost: 0 })).toBeNull();
  });

  it("sums position weights and ignores watchlist rows", () => {
    expect(totalHoldingWeight([position, watch, { ...position, symbol: "MSFT", weight: 10 }]))
      .toBeCloseTo(35.5);
    expect(totalHoldingWeight([])).toBe(0);
  });

  it("always carries the company name and ticker into the Agent prompt", () => {
    for (const kind of ["news", "valuation", "earnings"] as const) {
      const prompt = holdingAskPrompt(position, kind);
      expect(prompt).toContain("苹果");
      expect(prompt).toContain("AAPL");
    }
    expect(holdingAskPrompt(position, "news")).toContain("新闻");
    expect(holdingAskPrompt(position, "valuation")).toContain("估值");
    expect(holdingAskPrompt(position, "earnings")).toContain("财报");
    // 没有公司名时退回只用代码，不出现空括号
    expect(holdingAskPrompt(watch, "news")).toContain("TSLA");
    expect(holdingAskPrompt(watch, "news")).not.toContain("（）");
  });

  it("normalizes the ticker and treats blank fields as a watchlist entry", () => {
    const result = validateHoldingForm({
      symbol: " aapl ",
      name: "  苹果 ",
      weight: "",
      avgCost: "",
    });
    expect(result).toEqual({
      ok: true,
      value: { symbol: "AAPL", name: "苹果" },
    });
  });

  it("accepts a full position and keeps both numbers", () => {
    const result = validateHoldingForm({
      symbol: "brk.b",
      name: "",
      weight: "12.5",
      avgCost: "410",
    });
    expect(result).toEqual({
      ok: true,
      value: { symbol: "BRK.B", weight: 12.5, avg_cost: 410 },
    });
  });

  it("rejects invalid numbers and out-of-range weights", () => {
    expect(validateHoldingForm({ symbol: "", name: "", weight: "", avgCost: "" }).ok).toBe(false);
    expect(
      validateHoldingForm({ symbol: "AAPL", name: "", weight: "120", avgCost: "" }),
    ).toEqual({ ok: false, error: "仓位占比不能超过 100%" });
    expect(
      validateHoldingForm({ symbol: "AAPL", name: "", weight: "-3", avgCost: "" }).ok,
    ).toBe(false);
    expect(
      validateHoldingForm({ symbol: "AAPL", name: "", weight: "", avgCost: "abc" }).ok,
    ).toBe(false);
  });

  it("blocks adding beyond the entry limit", () => {
    expect(canAddHolding(49, 50)).toBe(true);
    expect(canAddHolding(50, 50)).toBe(false);
  });
});
