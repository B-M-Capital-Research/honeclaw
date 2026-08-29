import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(new URL("./public-admin-controlled-shadow-observation-input-admission-panel.tsx", import.meta.url), "utf8");

describe("Stage 104 observation-input admission panel", () => {
  test("states that provider publication time is unverified", () => {
    expect(source).toContain("供应商发布时间：未验证");
    expect(source).toContain("保管取得时间");
  });

  test("approval only opens Stage 105", () => {
    expect(source).toContain("准入，仅开放 Stage 105");
    expect(source).toContain("观察尚未开始");
  });

  test("requires the complete no-authority review checklist", () => {
    expect(source).toContain("no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed");
    expect(source).toContain("no_unconfirmed_hari_or_old_wang_logic_claimed");
  });
});
