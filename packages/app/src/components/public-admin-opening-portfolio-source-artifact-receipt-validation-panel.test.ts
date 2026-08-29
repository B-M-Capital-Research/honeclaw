import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(new URL("./public-admin-opening-portfolio-source-artifact-receipt-validation-panel.tsx", import.meta.url), "utf8");

describe("Stage 133 source artifact receipt validation panel", () => {
  test("makes the independent integrity boundary and closed financial authority explicit", () => {
    expect(source).toContain("第 133 阶段 · 加密 receipt 责任链外独立验证");
    expect(source).toContain("不证明文件里的持仓数字真实");
    expect(source).toContain("AES-256-GCM 认证解密");
    expect(source).toContain("下一步仅开放第 134 阶段零能力实现登记");
    expect(source).toContain("不解析金融行");
  });
});
