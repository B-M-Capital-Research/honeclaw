import { describe, expect, it } from "bun:test";

const source = await Bun.file(new URL("./public-admin-opening-portfolio-source-artifact-receipt-first-execution-authorization-panel.tsx", import.meta.url)).text();
const backend = await Bun.file(new URL("../../../../crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_source_artifact_receipt_first_execution_authorizations.rs", import.meta.url)).text();
const host = await Bun.file(new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url)).text();

describe("Stage 130 source artifact receiver first execution authorization", () => {
  it("requires a server-rehashed artifact and exposes no upload or execution control", () => {
    expect(source).toContain("artifact_inspection.artifact_verified");
    expect(source).toContain("服务端重哈希");
    expect(source).toContain("24 小时");
    expect(source).not.toContain('type="file"');
    expect(source).not.toContain("source_artifact_bytes");
    expect(backend).toContain("safe_read_only_regular_file");
    expect(backend).toContain("source_artifact_was_not_received_or_read");
    expect(backend).toContain("stage_131_claim_first_source_artifact_receipt_attempt");
  });

  it("is mounted after Stage 129 and keeps claim separate", () => {
    const stage129 = host.indexOf("PublicAdminOpeningPortfolioSourceArtifactReceiptIsolatedReceiverPanel />");
    const stage130 = host.indexOf("PublicAdminOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationPanel />");
    expect(stage129).toBeGreaterThan(-1);
    expect(stage130).toBeGreaterThan(stage129);
    expect(source).toContain("approval_only_opens_future_stage_131_claim_first_attempt_confirmed");
  });
});
