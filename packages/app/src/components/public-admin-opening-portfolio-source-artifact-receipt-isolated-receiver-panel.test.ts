import { describe, expect, it } from "bun:test";

const source = await Bun.file(new URL("./public-admin-opening-portfolio-source-artifact-receipt-isolated-receiver-panel.tsx", import.meta.url)).text();
const backend = await Bun.file(new URL("../../../../crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_source_artifact_receipt_isolated_receivers.rs", import.meta.url)).text();
const host = await Bun.file(new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url)).text();

describe("Stage 129 isolated source artifact receiver specification panel", () => {
  it("registers a proposed artifact without upload or execution controls", () => {
    expect(source).toContain("proposed_receiver_artifact_sha256");
    expect(source).toContain("Stage 130");
    expect(source).not.toContain('type="file"');
    expect(source).not.toContain("source_artifact_bytes");
    expect(backend).toContain("create-once registration");
    expect(backend).toContain("remote_url_fetch_allowed: false");
  });

  it("is mounted after the independent Stage 128 review", () => {
    const stage128 = host.indexOf("PublicAdminOpeningPortfolioSourceArtifactReceiptImplementationReviewPanel />");
    const stage129 = host.indexOf("PublicAdminOpeningPortfolioSourceArtifactReceiptIsolatedReceiverPanel />");
    expect(stage128).toBeGreaterThan(-1);
    expect(stage129).toBeGreaterThan(stage128);
  });
});
