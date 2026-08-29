import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(new URL("./public-admin-controlled-shadow-observation-materialization-specification-panel.tsx", import.meta.url), "utf8");

describe("Stage 105 observation materialization specification panel", () => {
  test("preserves the three price bases and explicit gaps", () => {
    expect(source).toContain("股票、SPY 和三种价格口径矩阵");
    expect(source).toContain("显式 gap");
    expect(source).toContain("不填充、插值或跨口径替代");
  });

  test("shows specification-only zero capability boundary", () => {
    expect(source).toContain("零执行能力");
    expect(source).toContain("没有实现、工件、入口、runtime 或输入挂载");
    expect(source).toContain("观察尚未生成");
  });

  test("requires an independent Stage 106 review", () => {
    expect(source).toContain("Stage 106");
    expect(source).toContain("no_unconfirmed_hari_or_old_wang_logic_claimed");
  });
});
