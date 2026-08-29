import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";
import {
  buildFinancialEvidenceReviewRequest,
  financialReviewCanApprove,
} from "./public-admin-financial-evidence-review";
import type {
  InvestmentFinancialEvidenceReviewCandidate,
  InvestmentFinancialEvidenceReviewConfirmations,
} from "@/lib/types";

const source = readFileSync(
  new URL("./public-admin-financial-evidence-review.tsx", import.meta.url),
  "utf8",
);

const confirmations: InvestmentFinancialEvidenceReviewConfirmations = {
  official_filings_opened: true,
  identity_periods_and_units_verified: true,
  calculations_recomputed: true,
  corporate_actions_and_restatements_checked: true,
  quality_warnings_resolved: true,
  no_unresolved_material_issue: true,
};

const candidate: InvestmentFinancialEvidenceReviewCandidate = {
  symbol: "MSFT",
  evidence_fingerprint_sha256: "a".repeat(64),
  evidence: {
    policy_version: "financial-v3",
    status: "partially_measured",
    financial_as_of: "2026-07-30",
    source_claim_ids: ["claim-1"],
    source_urls: ["https://www.sec.gov/filing"],
    source_calculations: ["收入同比 17.8%"],
    quality_warnings: [],
    missing_checks: [],
  },
  review_status: "sec_structured_pending_human_review",
  score_eligible: false,
  blocking_reasons: [],
  review_priority_rank: 3,
  review_priority_reasons: ["尚未完成人工财务质量审核"],
};

describe("SEC financial evidence review", () => {
  it("requires every confirmation before rating admission", () => {
    const complete = { ...confirmations, rationale: "已逐项核对官方原文与公式" };
    expect(financialReviewCanApprove(complete)).toBe(true);
    expect(financialReviewCanApprove({ ...complete, calculations_recomputed: false })).toBe(false);
    expect(financialReviewCanApprove({ ...complete, rationale: "太短" })).toBe(false);
  });

  it("binds the optimistic review id and exact evidence fingerprint", () => {
    const reviewed = {
      ...candidate,
      latest_review: {
        schema_version: "review-v1",
        policy_version: "admission-v1",
        review_id: "review-1",
        symbol: "MSFT",
        submitted_at: "2026-08-13T12:00:00Z",
        reviewer_id: "admin",
        evidence_fingerprint_sha256: candidate.evidence_fingerprint_sha256,
        evidence_snapshot: candidate.evidence,
        verdict: "changes_requested" as const,
        rationale: "期间口径需要修正",
        confirmations,
        rating_factor_authorized: false,
        valuation_authorized: false as const,
        training_authorized: false as const,
        reward_authorized: false as const,
        portfolio_action_authorized: false as const,
        shadow_portfolio_authorized: false as const,
        trade_authorized: false as const,
        old_wang_logic_confirmed: false as const,
      },
    };
    expect(buildFinancialEvidenceReviewRequest(
      reviewed,
      { ...confirmations, rationale: " 已重算并核对所有异常 " },
      "approved_for_rating",
    )).toEqual({
      expected_review_id: "review-1",
      expected_evidence_fingerprint_sha256: "a".repeat(64),
      verdict: "approved_for_rating",
      rationale: "已重算并核对所有异常",
      confirmations,
    });
  });

  it("states the narrow authority and keeps training, RL and trading closed", () => {
    expect(source).toContain("只审核数字、期间、单位和公式");
    expect(source).toContain("批准进入每日评级");
    expect(source).toContain("训练、RL、交易关闭");
    expect(source).toContain("证据变化，旧复核失效");
    expect(source).toContain("打开 SEC 原文");
    expect(source).toContain("会计口径");
    expect(source).toContain("原始单位");
    expect(source).toContain("对应 SEC 原文");
    expect(source).toContain("优先 5 家");
    expect(source).toContain("selection_scope");
    expect(source).toContain("查看全部");
  });
});
