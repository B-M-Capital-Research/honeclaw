import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-materialization-implementation-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 107 observation materialization zero-capability implementation panel", () => {
  it("keeps the contract non-executable and routes it to Stage 108 independent review", () => {
    expect(source).toContain("第 107 阶段");
    expect(source).toContain("没有入口、runtime、输入挂载/读取、环境、秘密、网络、工具、子进程或生产 I/O");
    expect(source).toContain("future_independent_implementation_review_required_before_isolated_runner_registration_confirmed");
    expect(source).toContain("Stage 108");
  });

  it("binds Stage 106 and the exact Stage 105 specification hashes", () => {
    expect(source).toContain("expected_specification_review_sha256");
    expect(source).toContain("expected_independent_audit_sha256");
    expect(source).toContain("expected_registration_sha256");
    expect(source).toContain("expected_specification_sha256");
    expect(source).toContain("exact_stage_104_admitted_output_is_only_future_input_confirmed");
  });
});
