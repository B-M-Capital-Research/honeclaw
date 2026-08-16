import { readFileSync } from "node:fs";
import { describe, expect, test } from "bun:test";

const page = readFileSync(
  new URL("./public-research-library.tsx", import.meta.url),
  "utf8",
);
const styles = readFileSync(
  new URL("./public-research-library.css", import.meta.url),
  "utf8",
);
const app = readFileSync(new URL("../app.tsx", import.meta.url), "utf8");
const api = readFileSync(new URL("../lib/api.ts", import.meta.url), "utf8");
const chat = readFileSync(new URL("./chat.tsx", import.meta.url), "utf8");
const research = readFileSync(new URL("./public-research.tsx", import.meta.url), "utf8");
const me = readFileSync(new URL("./public-me.tsx", import.meta.url), "utf8");

describe("unified research library", () => {
  test("has a discoverable signed-in route and chat entry", () => {
    expect(app).toContain('path="/research-library"');
    // Reachable from the conversation through the composer tools menu.
    expect(chat).toContain('href: "/research-library"');
    expect(page).toContain("我的知识源");
    // 知识源现在是管理员能力，入口在研究台「管理」分类；/me 只留指路卡。
    expect(research).toContain('key: "research-library"');
    expect(research).toContain("adminOnly: true");
    expect(me).toContain('navigate("/research?group=admin")');
  });

  test("keeps provenance, scope and downstream authorization explicit", () => {
    expect(page).toContain("知识星球导出");
    expect(page).toContain("IMA 导出");
    expect(page).toContain("HONE 官方");
    expect(page).toContain("关键事件链");
    expect(page).toContain("个人 → 候选 → 官方");
    expect(api).toContain("/api/public/research-library");
  });

  test("keeps community submissions isolated until an admin review", () => {
    expect(page).toContain("投稿给 HONE");
    expect(page).toContain("核验并采纳");
    expect(page).toContain('item.scope === "community_candidate"');
    expect(api).toContain("/submit");
    expect(api).toContain("/review");
  });

  test("replaces native dialogs with inline, cancellable confirmations", () => {
    expect(page).not.toContain("window.prompt");
    expect(page).not.toContain("window.confirm");
    expect(page).toContain("确认删除");
    expect(page).toContain("确认投稿");
    expect(page).toContain("确认采纳");
    expect(page).toContain("确认驳回");
    expect(page).toContain("research-library-review-form");
    expect(page).toContain("<textarea");
    expect(page).toContain("取消");
    expect(styles).toContain("var(--hone-error-600)");
  });

  test("updates the bundle in place instead of refetching everything", () => {
    expect(page).toContain("mergeItems(");
    expect(page).toContain("result.promoted_item");
    expect(page.match(/await load\(\)/g)?.length ?? 0).toBe(0);
  });

  test("consumes hone design tokens for statuses and review notes", () => {
    expect(styles).toContain("var(--hone-signal-green)");
    expect(styles).toContain("var(--hone-signal-yellow-soft)");
    expect(styles).toContain("var(--hone-coral-600)");
    expect(styles).not.toContain("#d8a329");
    expect(styles).not.toContain("#fff4d9");
  });

  test("supports responsive and dark layouts", () => {
    expect(styles).toContain('[data-theme="dark"]');
    expect(styles).toContain("@media (max-width: 760px)");
  });
});
