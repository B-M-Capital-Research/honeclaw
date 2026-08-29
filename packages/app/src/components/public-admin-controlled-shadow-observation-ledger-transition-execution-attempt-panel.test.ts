import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-ledger-transition-execution-attempt-panel.tsx", import.meta.url),
  "utf8",
);

const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 122 observation-ledger-transition one-shot execution panel", () => {
  it("binds the exact claim, artifact and Stage 111-114 evidence chain", () => {
    expect(source).toContain("第 122 阶段");
    expect(source).toContain("expected_claim_sha256");
    expect(source).toContain("expected_runner_contract_sha256");
    expect(source).toContain("expected_stage_114_admission_review_sha256");
    expect(source).toContain("expected_stage_112_output_sha256");
    expect(source).toContain("expected_stage_111_claim_sha256");
  });

  it("makes interruption terminal and forbids financial state", () => {
    expect(source).toContain("start_marker_consumes_claim_before_artifact_or_input_read_confirmed");
    expect(source).toContain("one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed");
    expect(source).toContain("opening_portfolio_snapshot_absent_no_default_notional_cash_positions_or_shares_confirmed");
    expect(source).toContain("non_financial_notice_allowlist_only_and_no_ledger_event_or_financial_posting_confirmed");
    expect(source).toContain("等待 Stage 123");
    expect(source).toContain("不允许重试");
  });

  it("is mounted immediately after the Stage 121 claim panel", () => {
    expect(governanceSource).toContain("PublicAdminControlledShadowObservationLedgerTransitionExecutionAttemptPanel");
    expect(governanceSource).toContain("<PublicAdminControlledShadowObservationLedgerTransitionExecutionAttemptPanel />");
  });
});
