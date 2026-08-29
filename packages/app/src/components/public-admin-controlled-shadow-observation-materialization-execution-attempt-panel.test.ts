import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-materialization-execution-attempt-panel.tsx", import.meta.url),
  "utf8",
);

const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 112 observation-materialization one-shot execution panel", () => {
  it("binds the exact claim, artifact, admitted input and specification", () => {
    expect(source).toContain("第 112 阶段");
    expect(source).toContain("expected_claim_sha256");
    expect(source).toContain("expected_runner_artifact_sha256");
    expect(source).toContain("expected_artifact_manifest_sha256");
    expect(source).toContain("expected_observation_materialization_specification_sha256");
    expect(source).toContain("expected_stage_104_admission_review_sha256");
    expect(source).toContain("expected_stage_102_output_sha256");
  });

  it("makes interruption terminal and keeps output untrusted", () => {
    expect(source).toContain("start_marker_consumes_claim_before_artifact_or_input_read_confirmed");
    expect(source).toContain("one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed");
    expect(source).toContain("output_create_once_content_addressed_untrusted_and_requires_independent_validation_confirmed");
    expect(source).toContain("等待 Stage 113");
    expect(source).toContain("不允许重试");
  });

  it("is mounted immediately after the Stage 111 claim panel", () => {
    expect(governanceSource).toContain("PublicAdminControlledShadowObservationMaterializationExecutionAttemptPanel");
    expect(governanceSource).toContain("<PublicAdminControlledShadowObservationMaterializationExecutionAttemptPanel />");
  });
});
