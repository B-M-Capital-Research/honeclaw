import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-materialization-output-validation-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 113 observation materialization independent output validation panel", () => {
  it("makes the chain-external second projection and terminal validation visible", () => {
    expect(source).toContain("第 113 阶段");
    expect(source).toContain("第二投影");
    expect(source).toContain("失败也会形成不可覆盖终态");
    expect(source).toContain("second_projection_does_not_call_stage_112_materializer_helpers_confirmed");
  });

  it("binds the exact claim result output and admitted Stage 104 input", () => {
    expect(source).toContain("expected_claim_sha256");
    expect(source).toContain("expected_result_sha256");
    expect(source).toContain("expected_output_sha256");
    expect(source).toContain("expected_stage_104_review_sha256");
    expect(source).toContain("exact_stage_104_admitted_stage_102_input_reopened_and_rehashed_confirmed");
  });

  it("opens only Stage 114 and keeps all portfolio authority closed", () => {
    expect(source).toContain("只进入 Stage 114 候选");
    expect(source).toContain("仍未形成账本、持仓、绩效或训练反馈");
    expect(source).toContain("no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed");
  });
});
