import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-ledger-transition-candidate-admission-panel.tsx", import.meta.url),
  "utf8",
);

const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 124 non-financial observation candidate admission panel", () => {
  it("binds the exact Stage 121-123 candidate and Stage 114 evidence", () => {
    expect(source).toContain("第 124 阶段");
    expect(source).toContain("expected_stage_123_validation_sha256");
    expect(source).toContain("expected_stage_122_candidate_sha256");
    expect(source).toContain("expected_stage_121_claim_sha256");
    expect(source).toContain("expected_stage_114_review_sha256");
  });

  it("creates only formal non-financial evidence and opens snapshot governance", () => {
    expect(source).toContain("admission_creates_separate_formal_non_financial_evidence_record_without_mutating_candidate_confirmed");
    expect(source).toContain("opening_portfolio_absent_financial_allowlist_empty_and_no_authoritative_ledger_event_confirmed");
    expect(source).toContain("approval_only_opens_stage_125_opening_portfolio_snapshot_governance_specification_confirmed");
    expect(source).toContain("没有持仓、现金、净值或交易状态");
  });

  it("is mounted immediately after Stage 123", () => {
    const stage123 = governanceSource.indexOf("<PublicAdminControlledShadowObservationLedgerTransitionOutputValidationPanel />");
    const stage124 = governanceSource.indexOf("<PublicAdminControlledShadowObservationLedgerTransitionCandidateAdmissionPanel />");
    expect(stage123).toBeGreaterThan(-1);
    expect(stage124).toBeGreaterThan(stage123);
  });
});
