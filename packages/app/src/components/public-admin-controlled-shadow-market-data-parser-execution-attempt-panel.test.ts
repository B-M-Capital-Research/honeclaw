import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-market-data-parser-execution-attempt-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 102 market-data parser one-shot execution panel", () => {
  it("makes one-shot consumption and terminal failure visible", () => {
    expect(source).toContain("第 102 阶段");
    expect(source).toContain("失败不可重试");
    expect(source).toContain("one_shot_failure_consumes_claim_and_no_retry_confirmed");
    expect(source).toContain("失败已消费");
  });

  it("binds the exact claim, artifact and fixed input", () => {
    expect(source).toContain("expected_claim_sha256");
    expect(source).toContain("expected_runner_artifact_sha256");
    expect(source).toContain("expected_input_manifest_sha256");
    expect(source).toContain("artifact_is_declarative_not_spawned_or_executed_confirmed");
  });

  it("keeps successful output untrusted and trading closed", () => {
    expect(source).toContain("输出仍为非可信");
    expect(source).toContain("等待 Stage 103");
    expect(source).toContain("no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed");
  });
});
