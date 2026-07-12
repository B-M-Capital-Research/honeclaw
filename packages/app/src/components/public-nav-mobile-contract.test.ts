import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(new URL("./public-nav.tsx", import.meta.url), "utf8");
const polish = readFileSync(
  new URL("../pages/public-polish.css", import.meta.url),
  "utf8",
);

describe("public mobile navigation contract", () => {
  it("keeps the dropdown compact and free of drawer-only content", () => {
    expect(source).not.toContain("PublicContactCards");
    expect(source).not.toContain("pub-mobile-menu-index");
    expect(source).not.toContain("pub-mobile-menu-kicker");
    expect(source).toContain('class="pub-mobile-menu-chat"');
  });

  it("anchors the dropdown below the floating navigation bar", () => {
    expect(polish).toContain(
      "top: calc(max(8px, env(safe-area-inset-top)) + 66px)",
    );
    expect(polish).toContain("width: auto");
    expect(polish).not.toContain(".pub-mobile-menu {\n    top: 0;");
  });
});
