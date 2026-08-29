import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(new URL("./public-admin-opening-portfolio-source-artifact-receipt-execution-attempt-panel.tsx", import.meta.url), "utf8");

describe("Stage 132 source artifact receipt panel", () => {
  test("exposes encrypted one-shot upload without implying a portfolio import", () => {
    expect(source).toContain("第 132 阶段 · 来源工件单次加密接收");
    expect(source).toContain("失败不可重试");
    expect(source).toContain("它还不是期初持仓");
    expect(source).toContain(".pdf,.csv,.json");
    expect(source).toContain("encryption_key_configured");
  });
});
