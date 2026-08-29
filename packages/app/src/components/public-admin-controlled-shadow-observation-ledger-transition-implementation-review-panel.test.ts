import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL(
    "./public-admin-controlled-shadow-observation-ledger-transition-implementation-review-panel.tsx",
    import.meta.url,
  ),
  "utf8",
);
const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 118 observation ledger transition implementation chain-external review panel", () => {
  it("independently binds implementation, contract, audit, review, registration and specification", () => {
    expect(source).toContain("第 118 阶段");
    expect(source).toContain("expected_implementation_sha256");
    expect(source).toContain("expected_implementation_contract_sha256");
    expect(source).toContain("expected_specification_review_sha256");
    expect(source).toContain("expected_specification_independent_audit_sha256");
    expect(source).toContain("expected_specification_registration_sha256");
    expect(source).toContain("expected_observation_ledger_transition_specification_sha256");
    expect(source).toContain("expected_independent_audit_sha256");
  });

  it("keeps approval zero-capability and bounded to Stage 119 specification registration", () => {
    expect(source).toContain("all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed");
    expect(source).toContain("provider_publication_time_remains_unverified_confirmed");
    expect(source).toContain("没有源码/可执行工件、入口、runtime、输入挂载读取");
    expect(source).toContain("approval_only_opens_future_isolated_observation_ledger_transition_runner_specification_registration_confirmed");
    expect(source).toContain("Stage 119 runner 规格登记资格");
    expect(source).toContain("仍无 runner、输入读取、opening snapshot、账本、NAV 或交易权限");
  });

  it("is mounted strictly after the Stage 117 implementation contract", () => {
    expect(
      governanceSource.indexOf(
        "<PublicAdminControlledShadowObservationLedgerTransitionImplementationReviewPanel />",
      ),
    ).toBeGreaterThan(
      governanceSource.indexOf(
        "<PublicAdminControlledShadowObservationLedgerTransitionImplementationPanel />",
      ),
    );
  });
});
