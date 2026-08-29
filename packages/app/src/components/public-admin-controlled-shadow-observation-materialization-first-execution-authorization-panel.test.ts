import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-materialization-first-execution-authorization-panel.tsx", import.meta.url),
  "utf8",
);

const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 110 observation-materialization first-execution authorization panel", () => {
  it("requires a server-verified content-addressed artifact instead of a typed digest", () => {
    expect(source).toContain("第 110 阶段");
    expect(source).toContain("artifact_inspection.artifact_verified");
    expect(source).toContain("expected_artifact_manifest_sha256");
    expect(source).toContain("手填 SHA 不能通过");
    expect(source).not.toContain("independently_reproduced_runner_artifact_sha256");
  });

  it("binds every Stage 101-109 identity and immutable observation contract", () => {
    expect(source).toContain("expected_isolated_runner_spec_sha256");
    expect(source).toContain("expected_runner_contract_sha256");
    expect(source).toContain("expected_implementation_contract_sha256");
    expect(source).toContain("expected_implementation_review_sha256");
    expect(source).toContain("expected_independent_audit_sha256");
    expect(source).toContain("expected_specification_review_sha256");
    expect(source).toContain("expected_specification_registration_sha256");
    expect(source).toContain("expected_stage_104_admission_review_sha256");
    expect(source).toContain("expected_stage_103_validation_sha256");
    expect(source).toContain("expected_stage_102_result_sha256");
    expect(source).toContain("expected_stage_102_output_sha256");
    expect(source).toContain("expected_stage_101_claim_sha256");
    expect(source).toContain("expected_stage_101_input_manifest_sha256");
    expect(source).toContain("expected_cycle_claim_sha256");
  });

  it("keeps approval single-use, time-bounded and non-executable", () => {
    expect(source).toContain("authorization_single_use_24_hour_expiry_and_stage_111_claim_separation_confirmed");
    expect(source).toContain("approval_only_opens_future_stage_111_claim_first_attempt_confirmed");
    expect(source).toContain("没有执行观察物化或读取 Stage 104 输入");
    expect(source).toContain("无 runtime/entrypoint/挂载/输入读取/观察物化执行或观察输出");
  });

  it("is mounted in the historical outcome governance surface", () => {
    expect(governanceSource).toContain("PublicAdminControlledShadowObservationMaterializationFirstExecutionAuthorizationPanel");
    expect(governanceSource).toContain("<PublicAdminControlledShadowObservationMaterializationFirstExecutionAuthorizationPanel />");
  });
});
