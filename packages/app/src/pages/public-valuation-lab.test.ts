import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const page = readFileSync(new URL("./public-valuation-lab.tsx", import.meta.url), "utf8");
const css = readFileSync(new URL("./public-valuation-lab.css", import.meta.url), "utf8");
const v2Css = readFileSync(new URL("./public-valuation-lab-v2.css", import.meta.url), "utf8");
const app = readFileSync(new URL("../app.tsx", import.meta.url), "utf8");
const chat = readFileSync(new URL("./chat.tsx", import.meta.url), "utf8");

describe("daily valuation lab", () => {
  test("is reachable from signed-in HONE", () => {
    expect(app).toContain('path="/valuation-lab"');
    expect(chat).toContain('navigate("/valuation-lab")');
  });

  test("keeps methods, evidence and failure states visible", () => {
    expect(page).toContain("前瞻 P/E · EV/EBIT · 周期 DCF");
    expect(page).toContain("当前股价反向估值");
    expect(page).toContain("方法交叉验证");
    expect(page).toContain("概率加权价值");
    expect(page).toContain("scenario.methods");
    expect(page).toContain("method.weight");
    expect(page).toContain("数据与来源");
    expect(page).toContain("本日不生成估值");
    expect(page).toContain("methodology_note");
    expect(page).toContain("公司研究卡建议：");
  });

  test("supports mobile and dark layouts", () => {
    expect(css).toContain('@media (max-width: 780px)');
    expect(css).toContain('[data-theme="dark"]');
    expect(v2Css).toContain("valuation-lab-method-results");
    expect(v2Css).toContain('[data-theme="dark"]');
  });
});
