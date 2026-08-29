import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-market-data-parser-output-validation-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 103 market-data parser independent output validation panel", () => {
  it("makes the second implementation and terminal validation visible", () => {
    expect(source).toContain("第 103 阶段");
    expect(source).toContain("第二实现");
    expect(source).toContain("失败也会形成不可覆盖终态");
    expect(source).toContain("second_implementation_does_not_call_stage_102_parser_helpers_confirmed");
  });

  it("binds the exact claim result output and frozen raw inputs", () => {
    expect(source).toContain("expected_claim_sha256");
    expect(source).toContain("expected_result_sha256");
    expect(source).toContain("expected_output_sha256");
    expect(source).toContain("expected_input_manifest_sha256");
    expect(source).toContain("fixed_stage_94_raw_payloads_rehashed_and_independently_reparsed_confirmed");
  });

  it("keeps source time observation and trading closed", () => {
    expect(source).toContain("source_available_at_remains_unverified_confirmed");
    expect(source).toContain("只进入 Stage 104 候选");
    expect(source).toContain("仍未开始观察");
    expect(source).toContain("no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed");
  });
});
