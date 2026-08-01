import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const terms = readFileSync(new URL("./public-terms.tsx", import.meta.url), "utf8");
const privacy = readFileSync(
  new URL("./public-privacy.tsx", import.meta.url),
  "utf8",
);
const legalToc = readFileSync(
  new URL("../components/public-legal-toc.tsx", import.meta.url),
  "utf8",
);
const site = readFileSync(new URL("./public-site.css", import.meta.url), "utf8");
const polish = readFileSync(
  new URL("./public-polish.css", import.meta.url),
  "utf8",
);

describe("public legal page theme contract", () => {
  it("uses the active theme surface on both legal pages", () => {
    for (const page of [terms, privacy]) {
      expect(page).toContain('background: "var(--hone-paper-50)"');
      expect(page).not.toContain('background: "#fff"');
    }
  });

  it("keeps the shared navigation readable in light and dark themes", () => {
    expect(site).toMatch(
      /\.hone-brand \{[^}]*color: var\(--hone-ink-950\)/s,
    );
    expect(polish).toMatch(
      /\.pub-nav-cta \{[^}]*background: var\(--hone-ink-950\)[^}]*color: var\(--hone-action-fg\)/s,
    );
    expect(polish).toMatch(
      /\.pub-nav-buy \{[^}]*background: var\(--hone-action-bg\)[^}]*color: var\(--hone-action-fg\)/s,
    );
  });

  it("keeps the floating back-to-top glyph contrasted against its button", () => {
    expect(legalToc).toContain('background: "var(--hone-ink-950)"');
    expect(legalToc).toContain('color: "var(--hone-action-fg)"');
  });
});
