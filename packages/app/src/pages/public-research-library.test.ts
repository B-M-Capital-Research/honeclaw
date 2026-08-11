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
const me = readFileSync(new URL("./public-me.tsx", import.meta.url), "utf8");

describe("unified research library", () => {
  test("has a discoverable signed-in route and chat entry", () => {
    expect(app).toContain('path="/research-library"');
    expect(chat).toContain('navigate("/research-library")');
    expect(page).toContain("我的知识源");
    expect(me).toContain('navigate("/research-library")');
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

  test("supports responsive and dark layouts", () => {
    expect(styles).toContain('[data-theme="dark"]');
    expect(styles).toContain("@media (max-width: 760px)");
  });
});
