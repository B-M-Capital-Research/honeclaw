import { describe, expect, it } from "bun:test";

const source = await Bun.file(new URL(
  "./public-admin-opening-portfolio-snapshot-governance-specification-review-panel.tsx",
  import.meta.url,
)).text();
const governanceSource = await Bun.file(new URL(
  "./public-admin-historical-outcome-governance-panel.tsx",
  import.meta.url,
)).text();

describe("Stage 126 opening portfolio governance specification review panel", () => {
  it("requires second-implementation review and preserves zero financial state", () => {
    expect(source).toContain("complete_specification_rebuilt_without_stage_125_builder_confirmed");
    expect(source).toContain("statement_values_informational_and_independent_marks_fx_derivatives_required_confirmed");
    expect(source).toContain("no_artifact_upload_read_parser_runtime_snapshot_or_financial_state_confirmed");
    expect(source).toContain("Stage 127 零能力");
    expect(source).not.toContain("uploadSourceArtifact");
  });

  it("is mounted strictly after Stage 125", () => {
    const stage125 = governanceSource.indexOf("<PublicAdminOpeningPortfolioSnapshotGovernanceSpecificationPanel />");
    const stage126 = governanceSource.indexOf("<PublicAdminOpeningPortfolioSnapshotGovernanceSpecificationReviewPanel />");
    expect(stage125).toBeGreaterThanOrEqual(0);
    expect(stage126).toBeGreaterThan(stage125);
  });
});
