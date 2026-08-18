import { describe, expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { readFileSync } from "node:fs";

const page = readFileSync(new URL("./public-valuation-lab.tsx", import.meta.url), "utf8");
const css = readFileSync(new URL("./public-valuation-lab.css", import.meta.url), "utf8");
const app = readFileSync(new URL("../app.tsx", import.meta.url), "utf8");
const chat = readFileSync(new URL("./chat.tsx", import.meta.url), "utf8");
const research = readFileSync(new URL("./public-research.tsx", import.meta.url), "utf8");

describe("daily valuation lab", () => {
  test("is reachable from the administrator tools menu", () => {
    expect(app).toContain('path="/valuation-lab"');
    // Still reachable from conversation, but only inside the admin group.
    expect(chat).toContain('href: "/valuation-lab"');
    expect(chat).toContain("tools_group_admin");
    // The publicly-visible group holds only the macro light for now, so every
    // other destination — this one included — must sit past the admin gate.
    const daily = chat.slice(
      chat.indexOf("tools_group_daily"),
      chat.indexOf("tools_group_admin"),
    );
    expect(daily).not.toContain('href: "/valuation-lab"');
    expect(daily).not.toContain("panel: \"daily-signal-ai\"");
    expect(daily).toContain("panel: \"daily-signal-macro\"");
  });

  test("is parked under the research-desk admin group until public rollout", () => {
    const block = research.slice(
      research.indexOf('key: "valuation-lab"'),
      research.indexOf('key: "portfolio-news"'),
    );
    expect(block).toContain('group: "admin"');
    expect(block).toContain("adminOnly: true");
    expect(page).toContain('setView("forbidden")');
    expect(page).toContain("暂未对全部用户开放");
  });

  test("lives in the research navigation section", () => {
    expect(page).toContain('active="research"');
    expect(page).not.toContain('active="insights"');
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

  test("merged the v2 patch layer into the main stylesheet", () => {
    expect(page).not.toContain("public-valuation-lab-v2.css");
    expect(
      existsSync(new URL("./public-valuation-lab-v2.css", import.meta.url)),
    ).toBe(false);
    expect(css).toContain("valuation-lab-method-results");
  });

  test("consumes hone design tokens instead of ad-hoc accents", () => {
    expect(css).toContain("var(--hone-coral-600)");
    expect(css).toContain("var(--hone-signal-green-soft)");
    expect(css).toContain("var(--hone-signal-yellow-soft)");
    expect(css).not.toContain("#d65f4a");
    expect(css).not.toContain("#fff2d8");
    expect(css).not.toContain("#fff9ed");
  });

  test("supports mobile and dark layouts", () => {
    expect(css).toContain("@media (max-width: 780px)");
    expect(css).toContain('[data-theme="dark"]');
  });
});
