import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const app = readFileSync(new URL("../app.tsx", import.meta.url), "utf8");
const login = readFileSync(
  new URL("../components/public-login-form.tsx", import.meta.url),
  "utf8",
);
const checkbox = readFileSync(
  new URL("../components/public-checkbox.tsx", import.meta.url),
  "utf8",
);
const foundation = readFileSync(
  new URL("./public-foundation.css", import.meta.url),
  "utf8",
);
const site = readFileSync(new URL("./public-site.css", import.meta.url), "utf8");

describe("public logged-out theme contract", () => {
  it("applies preferences for every public route and exposes the control while logged out", () => {
    expect(app).toContain('if (APP_SURFACE === "public") initPublicPrefs()');
    expect(login).toContain('<div class="public-login-preferences">');
    expect(login).toContain("<PublicPrefsButton />");
  });

  it("uses paired theme surfaces instead of white-only form controls", () => {
    expect(login).toContain('background: "var(--hone-surface-raised)"');
    expect(login).toContain('background: "var(--hone-control-surface)"');
    expect(login).toContain(': "var(--hone-action-bg)"');
    expect(login).not.toContain('background: "#fff"');
    expect(checkbox).toContain('"var(--hone-control-surface)"');
    expect(checkbox).toContain('stroke="currentColor"');
  });

  it("defines dark theme contrast tokens for brand, controls, feedback, and actions", () => {
    const darkTokens = foundation.slice(foundation.indexOf('[data-theme="dark"]'));
    expect(darkTokens).toContain("--hone-surface-raised: #242825");
    expect(darkTokens).toContain("--hone-control-surface: #171a18");
    expect(darkTokens).toContain("--hone-action-fg: #17201f");
    expect(darkTokens).toContain("--hone-error-600: #ffb3a8");
    expect(darkTokens).toContain("--hone-success-600: #85dca4");
    expect(site).toContain(".public-login-brand");
    expect(site).toContain("color: var(--hone-ink-950)");
    expect(site).toContain(".public-login-input::placeholder");
  });
});
