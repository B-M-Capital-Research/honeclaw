import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-ledger-transition-output-validation-panel.tsx", import.meta.url),
  "utf8",
);

const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 123 observation-ledger-transition output validation panel", () => {
  it("binds the exact Stage 121-122 candidate and Stage 114 source", () => {
    expect(source).toContain("第 123 阶段");
    expect(source).toContain("expected_claim_sha256");
    expect(source).toContain("expected_result_sha256");
    expect(source).toContain("expected_candidate_sha256");
    expect(source).toContain("expected_stage_114_review_sha256");
    expect(source).toContain("expected_stage_112_output_sha256");
  });

  it("requires a second projection and keeps every financial authority closed", () => {
    expect(source).toContain("second_projection_does_not_call_stage_122_projector_helpers_confirmed");
    expect(source).toContain("every_notice_identity_decimal_hash_sort_and_complete_candidate_exactly_compared_confirmed");
    expect(source).toContain("opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed");
    expect(source).toContain("候选仍未受信");
    expect(source).toContain("没有财务账本或交易权限");
  });

  it("is mounted immediately after the Stage 122 execution panel", () => {
    expect(governanceSource).toContain("PublicAdminControlledShadowObservationLedgerTransitionOutputValidationPanel");
    const stage122 = governanceSource.indexOf("<PublicAdminControlledShadowObservationLedgerTransitionExecutionAttemptPanel />");
    const stage123 = governanceSource.indexOf("<PublicAdminControlledShadowObservationLedgerTransitionOutputValidationPanel />");
    expect(stage122).toBeGreaterThan(-1);
    expect(stage123).toBeGreaterThan(stage122);
  });
});
