import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-ledger-transition-execution-attempt-claim-panel.tsx", import.meta.url),
  "utf8",
);
const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 121 observation-ledger-transition claim-first panel", () => {
  it("binds the exact Stage 111-120 chain before any execution", () => {
    expect(source).toContain("第 121 阶段");
    expect(source).toContain("expected_authorization_review_sha256");
    expect(source).toContain("expected_artifact_manifest_sha256");
    expect(source).toContain("expected_stage_114_admission_review_sha256");
    expect(source).toContain("expected_stage_111_claim_sha256");
    expect(source).not.toContain("expected_cycle_claim_sha256");
  });

  it("permanently consumes authorization without execution, input read, or retry", () => {
    expect(source).toContain("no_retry_release_or_authorization_restoration_after_claim_confirmed");
    expect(source).toContain("永久消费授权");
    expect(source).toContain("不运行工件、不挂载或读取已准入输出，也不能撤销");
    expect(source).not.toContain("executeControlledShadowObservationLedgerTransition");
  });

  it("is mounted immediately after Stage 120 governance", () => {
    expect(governanceSource).toContain("PublicAdminControlledShadowObservationLedgerTransitionExecutionAttemptClaimPanel");
    expect(governanceSource).toContain("<PublicAdminControlledShadowObservationLedgerTransitionExecutionAttemptClaimPanel />");
  });
});
