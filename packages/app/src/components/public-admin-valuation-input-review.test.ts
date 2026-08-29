import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";
import {
  buildValuationInputReviewRequest,
  valuationPreparedMethods,
  valuationReviewCanApprove,
} from "./public-admin-valuation-input-review";
import type {
  InvestmentValuationInputReviewCandidate,
  InvestmentValuationInputReviewConfirmations,
} from "@/lib/types";

const source = readFileSync(
  new URL("./public-admin-valuation-input-review.tsx", import.meta.url),
  "utf8",
);

const confirmations: InvestmentValuationInputReviewConfirmations = {
  official_sources_opened: true,
  sec_financial_values_recomputed: true,
  diluted_share_count_and_corporate_actions_verified: true,
  complete_net_cash_or_debt_verified: true,
  forward_or_midcycle_inputs_verified: true,
  cyclicality_and_normalization_checked: true,
  cross_method_comparability_checked: true,
  no_unresolved_material_issue: true,
};

const candidate: InvestmentValuationInputReviewCandidate = {
  symbol: "SNDK",
  financial_evidence_fingerprint_sha256: "a".repeat(64),
  financial_evidence: {
    policy_version: "hone-financial-verification-v5-valuation-input-preparation",
    status: "partially_measured",
    financial_as_of: "2026-08-18",
    current_free_cash_flow: 11_494,
    prior_free_cash_flow: -120,
    financial_value_unit: "USD_millions",
    source_claim_ids: ["claim-1"],
    source_urls: ["https://www.sec.gov/filing"],
    source_calculations: ["FCF = OCF - capex"],
    quality_warnings: [],
    missing_checks: [],
  },
  review_status: "sec_valuation_review_pending",
  valuation_authorized: false,
  blocking_reasons: ["尚未完成独立估值用途复核"],
  prepared_method_ids: [],
};

const completeDraft = {
  ...confirmations,
  rationale: "已逐项核对股本、净现金、前瞻输入和三种方法",
  input_as_of: "2026-08-21",
  diluted_shares_millions: "150",
  complete_net_cash_millions: "3000",
  forward_eps: "12",
  forward_revenue_millions: "20000",
  normalized_ebit_margin_percent: "30",
  annual_fcf_history_millions: "1000, 1500, 2000",
  source_urls: "https://www.sec.gov/filing\nhttps://investor.example.com/estimate",
  source_note: "稀释股本和一致预期均按 2026-08-21 口径核验",
};

describe("independent valuation input review", () => {
  it("requires universal inputs, two methods, sources and all confirmations", () => {
    expect(valuationPreparedMethods(completeDraft)).toEqual([
      "前瞻 P/E",
      "EV/EBIT",
      "周期调整 DCF",
    ]);
    expect(valuationReviewCanApprove(completeDraft)).toBe(true);
    expect(valuationReviewCanApprove({
      ...completeDraft,
      forward_revenue_millions: "",
      annual_fcf_history_millions: "1000",
    })).toBe(false);
    expect(valuationReviewCanApprove({
      ...completeDraft,
      complete_net_cash_or_debt_verified: false,
    })).toBe(false);
  });

  it("binds the exact SEC fingerprint and supplemental input packet", () => {
    expect(buildValuationInputReviewRequest(
      candidate,
      completeDraft,
      "approved_for_valuation",
    )).toEqual({
      expected_review_id: undefined,
      expected_financial_evidence_fingerprint_sha256: "a".repeat(64),
      verdict: "approved_for_valuation",
      rationale: completeDraft.rationale,
      confirmations,
      supplemental_inputs: {
        input_as_of: "2026-08-21",
        currency: "USD",
        diluted_shares_millions: 150,
        complete_net_cash_millions: 3000,
        forward_eps: 12,
        forward_revenue_millions: 20000,
        normalized_ebit_margin_percent: 30,
        annual_fcf_history_millions: [1000, 1500, 2000],
        source_urls: [
          "https://www.sec.gov/filing",
          "https://investor.example.com/estimate",
        ],
        source_note: completeDraft.source_note,
      },
    });
  });

  it("states expiry and keeps every downstream authority closed", () => {
    expect(source).toContain("独立估值用途门禁");
    expect(source).toContain("评级财务批准不能替代这里");
    expect(source).toContain("7 天有效 · 交易关闭");
    expect(source).toContain("至少两种方法");
    expect(source).toContain("批准进入估值模型");
    expect(source).toContain("训练、奖励、组合、影子组合或交易");
  });
});
