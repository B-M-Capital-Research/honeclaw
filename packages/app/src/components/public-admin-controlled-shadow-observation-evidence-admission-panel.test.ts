import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-evidence-admission-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 114 observation evidence admission review panel", () => {
  it("shows a chain-external admission review rather than a ledger action", () => {
    expect(source).toContain("第 114 阶段");
    expect(source).toContain("分离证据 · 不建账");
    expect(source).toContain("reviewer_independent_from_validator_executor_and_complete_prior_chain_confirmed");
  });

  it("binds the exact Stage 111 through Stage 113 immutable chain", () => {
    expect(source).toContain("expected_stage_113_validation_sha256");
    expect(source).toContain("expected_stage_112_output_sha256");
    expect(source).toContain("expected_stage_111_claim_sha256");
    expect(source).toContain("stage_112_envelope_reopened_rehashed_and_reprojected_confirmed");
  });

  it("keeps the provider-time limitation and opens only Stage 115 specification registration", () => {
    expect(source).toContain("供应商发布时间仍未验证");
    expect(source).toContain("批准只开放 Stage 115 账本转换规格登记");
    expect(source).toContain("no_ledger_position_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed");
  });
});
