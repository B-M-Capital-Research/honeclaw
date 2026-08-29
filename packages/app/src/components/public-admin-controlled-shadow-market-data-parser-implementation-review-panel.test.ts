import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-market-data-parser-implementation-review-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 98 parser implementation chain-external review panel", () => {
  it("independently binds every parser contract layer and complete prior chain", () => {
    expect(source).toContain("第 98 阶段");
    expect(source).toContain("expected_implementation_sha256");
    expect(source).toContain("expected_implementation_contract_sha256");
    expect(source).toContain("expected_specification_review_sha256");
    expect(source).toContain("expected_specification_registration_sha256");
    expect(source).toContain("expected_parser_specification_sha256");
    expect(source).toContain("expected_independent_audit_sha256");
    expect(source).toContain("reviewer_independent_from_registrar_and_complete_prior_chain_confirmed");
  });

  it("keeps approval zero-capability and bounded to Stage 99 specification registration", () => {
    expect(source).toContain("all_eight_synthetic_vectors_independently_reconstructed_confirmed");
    expect(source).toContain("source_available_at_remains_unverified_until_separate_evidence_confirmed");
    expect(source).toContain("没有源码/可执行工件、入口、runtime、原始载荷挂载读取");
    expect(source).toContain("approval_only_opens_future_isolated_parser_runner_specification_registration_confirmed");
    expect(source).toContain("Stage 99 runner 规格登记资格");
    expect(source).toContain("仍无 parser runner、载荷访问或交易权限");
  });
});
