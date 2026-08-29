import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-materialization-execution-attempt-claim-panel.tsx", import.meta.url),
  "utf8",
);

const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 111 observation-materialization claim-first panel", () => {
  it("binds the exact Stage 101-110 chain before execution", () => {
    expect(source).toContain("第 111 阶段");
    expect(source).toContain("expected_authorization_review_sha256");
    expect(source).toContain("expected_runner_contract_sha256");
    expect(source).toContain("expected_artifact_manifest_sha256");
    expect(source).toContain("expected_stage_104_admission_review_sha256");
    expect(source).toContain("expected_stage_101_input_manifest_sha256");
    expect(source).toContain("expected_cycle_claim_sha256");
  });

  it("permanently consumes authorization without execution or retry", () => {
    expect(source).toContain("no_retry_release_or_authorization_restoration_after_claim_confirmed");
    expect(source).toContain("永久消费授权");
    expect(source).toContain("不运行工件、不挂载或读取输入，也不能撤销");
    expect(source).not.toContain("executeControlledShadowObservationMaterialization");
  });

  it("is mounted immediately after Stage 110 governance", () => {
    expect(governanceSource).toContain("PublicAdminControlledShadowObservationMaterializationExecutionAttemptClaimPanel");
    expect(governanceSource).toContain("<PublicAdminControlledShadowObservationMaterializationExecutionAttemptClaimPanel />");
  });
});
