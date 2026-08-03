import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");

const app = read("../app.tsx");
const home = read("./public-home.tsx");
const roadmap = read("./public-roadmap.tsx");
const community = read("./public-community.css");
const foundation = read("./public-foundation.css");
const site = read("./public-site.css");
const polish = read("./public-polish.css");

describe("public theme surface contract", () => {
  it("keeps every public route in the whole-site theme audit inventory", () => {
    for (const route of [
      "/",
      "/roadmap",
      "/plan",
      "/blog",
      "/blog/:slug",
      "/me",
      "/portfolio",
      "/invest",
      "/activate",
      "/community",
      "/terms",
      "/privacy",
      "/chat",
      "/__share-preview",
    ]) {
      expect(app).toContain(`<Route path="${route}"`);
    }
  });

  it("uses semantic surfaces and action foregrounds on the home page", () => {
    expect(home).toMatch(/\.hone-home-window \{[^}]*background: var\(--hone-surface-raised\)/s);
    expect(home).toMatch(/\.hone-home-trust article \{[^}]*background: var\(--hone-surface-raised\)/s);
    expect(home).toMatch(/\.hone-home-cases \{[^}]*background: var\(--hone-surface-raised\)/s);
    expect(home).toMatch(/\.hone-home-blog \{[^}]*background: var\(--hone-surface-raised\)/s);
    expect(home).toMatch(/\.hone-home-cta \{[^}]*color: var\(--hone-action-fg\)/s);
  });

  it("uses semantic surfaces and stable inverse colors on the roadmap", () => {
    expect(roadmap).toMatch(/\.roadmap-card \{[^}]*background: var\(--hone-surface-raised\)/s);
    expect(roadmap).toMatch(/\.roadmap-card\.dark \{[^}]*background: var\(--hone-inverse-bg\)[^}]*color: var\(--hone-inverse-fg\)/s);
    expect(roadmap).toMatch(/\.version-pill \{[^}]*background: var\(--hone-surface-raised\)/s);
    expect(roadmap).toMatch(/\.btn-primary\.large \{[^}]*color: var\(--hone-action-fg\)/s);
  });

  it("keeps signed-in community content theme-aware", () => {
    expect(community).toMatch(/\.public-community-page \{[^}]*background: var\(--hone-paper-50\)[^}]*color: var\(--hone-ink-950\)/s);
    expect(community).toMatch(/\.public-community-card \{[^}]*var\(--hone-surface-raised\)/s);
    expect(community).toMatch(/\.public-community-body \{[^}]*color: var\(--hone-ink-800\)/s);
    expect(community).toMatch(/\.public-community-file \{[^}]*background: var\(--hone-control-surface\)[^}]*color: var\(--hone-ink-950\)/s);
  });

  it("keeps blog, footer, and navigation overlays readable in both themes", () => {
    expect(site).toMatch(/\.public-blog-page,[\s\S]*?background:[\s\S]*?var\(--hone-paper-50\)/);
    expect(site).toMatch(/\.public-blog-card \{[^}]*var\(--hone-surface-raised\)/s);
    expect(site).toMatch(/\.public-blog-markdown \{[^}]*color: var\(--hone-ink-800\)/s);
    expect(site).toMatch(/\.public-blog-markdown \{[^}]*--text-primary: var\(--hone-ink-950\)[^}]*--text-secondary: var\(--hone-ink-800\)/s);
    expect(site).toMatch(/\.pub-footer-bottom \{[^}]*color: #879398/s);
    expect(polish).toMatch(/\.pub-nav-more-panel \{[^}]*var\(--hone-surface-raised\)/s);
    expect(polish).toMatch(/\.pub-mobile-tabs \{[^}]*var\(--hone-surface-raised\)/s);
  });

  it("keeps small accent text above the AA contrast floor in the light theme", () => {
    expect(foundation).toContain("--hone-coral-600: #b94432;");
    expect(roadmap).toContain(".status-badge.stable { background: #ecfdf5; color: #047857; }");
  });
});
