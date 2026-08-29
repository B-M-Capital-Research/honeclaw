import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(new URL("./public-admin-controlled-shadow-observation-materialization-specification-review-panel.tsx", import.meta.url), "utf8");

describe("Stage 106 observation materialization specification review panel", () => {
  test("requires a chain-external second implementation rebuild", () => {
    expect(source).toContain("第二实现");
    expect(source).toContain("从当前 Stage 104 源完整重建规格");
    expect(source).toContain("rebuilt_specification_exactly_matches_registered_specification_confirmed");
  });

  test("reviews price bases, gaps, actions and initial allocation separately", () => {
    expect(source).toContain("官方交易日、标的、SPY 和三价格口径矩阵");
    expect(source).toContain("分红、拆股和三价格口径继续分开");
    expect(source).toContain("初始影子组合只绑定");
  });

  test("approval opens only the Stage 107 zero-capability gate", () => {
    expect(source).toContain("Stage 107 零能力实现登记");
    expect(source).toContain("观察仍未生成");
    expect(source).toContain("no_unconfirmed_hari_or_old_wang_logic_claimed");
  });
});
