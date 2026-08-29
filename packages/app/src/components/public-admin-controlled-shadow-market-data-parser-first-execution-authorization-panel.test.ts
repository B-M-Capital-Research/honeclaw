import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-market-data-parser-first-execution-authorization-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 100 market-data parser first-execution authorization panel", () => {
  it("requires a server-verified content-addressed artifact instead of a typed digest", () => {
    expect(source).toContain("第 100 阶段");
    expect(source).toContain("artifact_inspection.artifact_verified");
    expect(source).toContain("expected_artifact_manifest_sha256");
    expect(source).toContain("手填 SHA 不能通过");
    expect(source).not.toContain("independently_reproduced_runner_artifact_sha256");
  });

  it("binds every Stage 93-99 identity and the immutable runner contract", () => {
    expect(source).toContain("expected_isolated_runner_spec_sha256");
    expect(source).toContain("expected_runner_contract_sha256");
    expect(source).toContain("expected_implementation_contract_sha256");
    expect(source).toContain("expected_implementation_review_sha256");
    expect(source).toContain("expected_independent_audit_sha256");
    expect(source).toContain("expected_specification_review_sha256");
    expect(source).toContain("expected_specification_registration_sha256");
    expect(source).toContain("expected_validation_sha256");
    expect(source).toContain("expected_receipt_sha256");
    expect(source).toContain("expected_claim_sha256");
    expect(source).toContain("expected_result_sha256");
  });

  it("keeps approval single-use, time-bounded and non-executable", () => {
    expect(source).toContain("authorization_single_use_24_hour_expiry_and_stage_101_claim_separation_confirmed");
    expect(source).toContain("approval_only_opens_future_stage_101_claim_first_attempt_confirmed");
    expect(source).toContain("没有执行 parser 或读取行情");
    expect(source).toContain("无 runtime/entrypoint/挂载/载荷读取/parser 执行或解析行");
  });
});
