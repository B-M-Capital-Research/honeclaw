import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-ledger-transition-first-execution-authorization-panel.tsx", import.meta.url),
  "utf8",
);

const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 120 observation-ledger-transition first-execution authorization panel", () => {
  it("requires a server-verified content-addressed artifact instead of a typed digest", () => {
    expect(source).toContain("第 120 阶段");
    expect(source).toContain("artifact_inspection.artifact_verified");
    expect(source).toContain("expected_artifact_manifest_sha256");
    expect(source).toContain("手填 SHA 不能通过");
    expect(source).not.toContain("independently_reproduced_runner_artifact_sha256");
  });

  it("binds the Stage 51-119 chain without inventing obsolete materialization fields", () => {
    expect(source).toContain("expected_isolated_runner_spec_sha256");
    expect(source).toContain("expected_runner_contract_sha256");
    expect(source).toContain("expected_implementation_contract_sha256");
    expect(source).toContain("expected_implementation_review_sha256");
    expect(source).toContain("expected_independent_audit_sha256");
    expect(source).toContain("expected_specification_review_sha256");
    expect(source).toContain("expected_specification_registration_sha256");
    expect(source).toContain("expected_stage_114_admission_review_sha256");
    expect(source).toContain("expected_stage_113_validation_sha256");
    expect(source).toContain("expected_stage_112_result_sha256");
    expect(source).toContain("expected_stage_111_claim_sha256");
    expect(source).not.toContain("expected_stage_112_output_sha256");
    expect(source).not.toContain("expected_stage_111_input_manifest_sha256");
    expect(source).not.toContain("expected_cycle_claim_sha256");
  });

  it("keeps approval single-use, time-bounded and non-executable", () => {
    expect(source).toContain("authorization_single_use_24_hour_expiry_and_stage_121_claim_separation_confirmed");
    expect(source).toContain("approval_only_opens_future_stage_121_claim_first_attempt_confirmed");
    expect(source).toContain("没有执行观察到账本转换或读取 Stage 114 输入");
    expect(source).toContain("无 runtime/entrypoint/挂载/输入读取/观察到账本转换执行或候选输出");
  });

  it("keeps the missing opening snapshot and empty financial allowlist fail closed", () => {
    expect(source).toContain("opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed");
    expect(source).toContain("future_financial_events_require_separately_admitted_opening_snapshot_confirmed");
    expect(source).toContain("future_attempt_limited_to_non_financial_notice_candidate_without_authoritative_state_confirmed");
    expect(source).toContain("期初组合仍不存在、金融事件白名单仍为空");
  });

  it("is mounted in the historical outcome governance surface", () => {
    expect(governanceSource).toContain("PublicAdminControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationPanel");
    expect(governanceSource).toContain("<PublicAdminControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationPanel />");
  });
});
