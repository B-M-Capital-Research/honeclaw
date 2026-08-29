import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-market-data-parser-implementation-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 97 zero-capability parser implementation contract panel", () => {
  it("keeps the contract non-executable and routes it to Stage 98 independent review", () => {
    expect(source).toContain("第 97 阶段");
    expect(source).toContain("没有入口、runtime、载荷挂载/读取、环境、秘密、网络、工具或子进程");
    expect(source).toContain("future_independent_implementation_review_required_before_isolated_runner_confirmed");
    expect(source).toContain("Stage 98");
  });

  it("binds the full Stage 96 and upstream fingerprint chain", () => {
    expect(source).toContain("expected_specification_review_sha256");
    expect(source).toContain("expected_parser_specification_sha256");
    expect(source).toContain("expected_validation_sha256");
    expect(source).toContain("expected_canonical_request_set_sha256");
    expect(source).toContain("all_eight_synthetic_vector_hashes_bound_confirmed");
  });
});
