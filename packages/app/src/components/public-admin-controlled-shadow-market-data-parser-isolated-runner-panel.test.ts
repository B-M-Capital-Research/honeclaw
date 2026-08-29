import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-market-data-parser-isolated-runner-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 99 isolated market-data parser runner specification panel", () => {
  it("binds Stage 98 approval and every Stage 93-97 identity", () => {
    expect(source).toContain("第 99 阶段");
    expect(source).toContain("expected_implementation_review_sha256");
    expect(source).toContain("expected_independent_audit_sha256");
    expect(source).toContain("expected_specification_review_sha256");
    expect(source).toContain("expected_specification_registration_sha256");
    expect(source).toContain("expected_validation_sha256");
    expect(source).toContain("expected_receipt_sha256");
    expect(source).toContain("expected_claim_sha256");
    expect(source).toContain("expected_result_sha256");
  });

  it("freezes a proposed artifact identity without claiming an artifact or runtime exists", () => {
    expect(source).toContain("proposed_runner_artifact_sha256");
    expect(source).toContain("artifact_reproduction_procedure");
    expect(source).toContain("future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed");
    expect(source).toContain("没有源码、可执行工件、入口、runtime、挂载、读取");
    expect(source).toContain("registration_only_opens_chain_external_first_execution_authorization_review_confirmed");
    expect(source).toContain("工件存在：否 · runtime 实例化：否 · 载荷读取：否");
  });

  it("makes resource ceilings and zero downstream authority visible", () => {
    expect(source).toContain("maximum_memory_mib");
    expect(source).toContain("maximum_wall_clock_seconds");
    expect(source).toContain("maximum_process_count");
    expect(source).toContain("下单/券商/交易：否");
  });
});
