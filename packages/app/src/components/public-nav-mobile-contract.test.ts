import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(new URL("./public-nav.tsx", import.meta.url), "utf8");
const polish = readFileSync(
  new URL("../pages/public-polish.css", import.meta.url),
  "utf8",
);

describe("public mobile navigation contract", () => {
  it("uses four first-class mobile tabs without duplicating them in the utility menu", () => {
    expect(source).not.toContain("PublicContactCards");
    expect(source).not.toContain("pub-mobile-menu-index");
    expect(source).not.toContain("pub-mobile-menu-kicker");
    expect(source).not.toContain('class="pub-mobile-menu-chat"');
    expect(source).toContain('class="pub-mobile-tabs"');
    expect(source).toContain('{ labelKey: "community", path: "/community"');
    expect(source).toContain('{ labelKey: "me", path: "/me", icon: "me" }');
    expect(source).toContain('aria-current={isActive(tab.path) ? "page" : undefined}');
    expect(polish).toContain("grid-template-columns: repeat(4, minmax(0, 1fr))");
  });

  it("anchors the flat header, its drawer, and the tabs above the safe area", () => {
    // 通栏扁平导航：吸顶、细分割线、无圆角；抽屉紧贴导航条下方展开。
    expect(polish).toContain("height: calc(56px + env(safe-area-inset-top))");
    expect(polish).toContain("top: calc(56px + env(safe-area-inset-top))");
    expect(polish).toContain("border-bottom: 1px solid var(--hone-line)");
    expect(polish).toContain("width: auto");
    expect(polish).toContain("bottom: max(7px, env(safe-area-inset-bottom))");
    expect(polish).not.toContain(".pub-mobile-menu {\n    top: 0;");
  });

  it("keeps the desktop bar focused and moves secondary destinations into More", () => {
    expect(source).toContain('{ labelKey: "community", path: "/community" }');
    expect(source).toContain('class="pub-nav-more-trigger"');
    expect(source).toContain('href="/roadmap"');
    expect(source).toContain('href="/me"');
    expect(source).toContain("communityUnread?: boolean");
  });
});
