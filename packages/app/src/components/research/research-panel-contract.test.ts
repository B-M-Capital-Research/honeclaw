import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

/**
 * One geometry, one type scale, for every research panel.
 *
 * Each panel passes its own `backdropClass` / `dialogClass` into the shared
 * `ResearchPanel`, so `.X-dialog` and `.research-panel` land on the *same* DOM
 * node. When both sides declared width / height / radius / background the
 * winner was decided by CSS import order alone — single-class selectors tie on
 * specificity — and the loser showed up in production as a modal collapsed to
 * the max-content width of its subtitle.
 *
 * Geometry therefore lives in research.css and nowhere else, and every panel
 * draws its type and color from the foundation tokens.
 */

const PANELS = [
  { file: "company-rating", backdrop: "company-rating-backdrop", dialog: "company-rating-dialog" },
  { file: "daily-signal", backdrop: "daily-signal-backdrop", dialog: "daily-signal-dialog" },
  { file: "influencer-digest", backdrop: "influencer-digest-backdrop", dialog: "influencer-digest-dialog" },
  { file: "key-event-chain", backdrop: "key-chain-backdrop", dialog: "key-chain-dialog" },
  { file: "portfolio-news", backdrop: "portfolio-news-backdrop", dialog: "portfolio-news-dialog" },
  { file: "position-management", backdrop: "position-management-backdrop", dialog: "position-management-dialog" },
  { file: "weekly-brief", backdrop: "weekly-brief-backdrop", dialog: "weekly-brief-dialog" },
] as const;

/** Comments explain the rules; they are not subject to them. */
const stripComments = (css: string) => css.replace(/\/\*[\s\S]*?\*\//g, "");

const sheet = (name: string) =>
  stripComments(readFileSync(new URL(`../${name}-dashboard.css`, import.meta.url), "utf8"));

const foundation = readFileSync(
  new URL("../../pages/public-foundation.css", import.meta.url),
  "utf8",
);
const shell = stripComments(readFileSync(new URL("./research.css", import.meta.url), "utf8"));

/** Every `.selector { … }` block for a class, including inside media queries. */
function blocksFor(css: string, selector: string) {
  return [...css.matchAll(new RegExp(`\\.${selector}\\s*\\{([^}]*)\\}`, "g"))].map((m) => m[1]);
}

/** Declarations that size or paint the sheet itself — the shell's job alone. */
const GEOMETRY =
  /(?:^|\s)(position|inset|top|right|bottom|left|z-index|display|place-items|place-content|align-items|justify-content|flex-direction|width|min-width|max-width|height|min-height|max-height|padding|margin|border|border-radius|background|background-color|box-shadow|backdrop-filter|overflow|overflow-x|overflow-y)\s*:/;

describe("research panel contract", () => {
  it("keeps sheet geometry in the shared shell, never in a panel skin", () => {
    // The shell is where it does belong.
    expect(shell).toContain(".research-panel {");
    expect(shell).toMatch(/\.research-panel\s*\{[^}]*width: min\(/);

    for (const panel of PANELS) {
      const css = sheet(panel.file);
      for (const selector of [panel.backdrop, panel.dialog]) {
        for (const body of blocksFor(css, selector)) {
          const offender = body.match(GEOMETRY);
          expect(
            offender ? `${panel.file}: .${selector} 仍在声明 ${offender[1]}` : "",
          ).toBe("");
        }
      }
    }
  });

  it("paints every panel from tokens — no literal colors", () => {
    for (const panel of PANELS) {
      const css = sheet(panel.file);
      expect(`${panel.file}: ${css.match(/#[0-9a-fA-F]{3,8}\b/)?.[0] ?? ""}`).toBe(`${panel.file}: `);
      expect(`${panel.file}: ${css.match(/rgba?\([0-9]/)?.[0] ?? ""}`).toBe(`${panel.file}: `);
      expect(css).not.toContain("!important");
    }
  });

  it("sizes every panel from the type scale, with 11px as the floor", () => {
    // Panels shipped 8–10px CJK, which is unreadable at any density. The scale
    // has no step below 11px, so going through it is what enforces the floor.
    for (const step of ["2xs: 11px", "xs: 12px", "sm: 13px", "md: 14px", "lg: 16px"]) {
      expect(foundation).toContain(`--hone-text-${step}`);
    }
    for (const panel of PANELS) {
      const css = sheet(panel.file);
      expect(`${panel.file}: ${css.match(/font-size: *[0-9.]+px/)?.[0] ?? ""}`).toBe(`${panel.file}: `);
      expect(`${panel.file}: ${css.match(/border-radius: *[0-9.]+px/)?.[0] ?? ""}`).toBe(`${panel.file}: `);
      expect(`${panel.file}: ${css.match(/font-weight: *(?:750|800)/)?.[0] ?? ""}`).toBe(`${panel.file}: `);
    }
  });

  it("gives signal text its own AA-passing ink, separate from the dot color", () => {
    // The dot colors read as dots but failed as type on their own soft chip:
    // yellow 3.39:1, orange 3.94:1, red 4.43:1 against a 4.5:1 requirement.
    for (const hue of ["green", "yellow", "orange", "red", "neutral"]) {
      expect(foundation).toContain(`--hone-signal-${hue}-ink:`);
    }
    // Wherever a `-soft` background carries text, the ink variant is used.
    const grid = stripComments(
      readFileSync(new URL("../../pages/public-research.css", import.meta.url), "utf8"),
    );
    for (const css of [shell, grid]) {
      for (const hue of ["green", "yellow", "orange", "red"]) {
        const rule = new RegExp(
          `background: var\\(--hone-signal-${hue}-soft\\);\\s*\\n\\s*color: var\\(--hone-signal-${hue}\\);`,
        );
        expect(css).not.toMatch(rule);
      }
    }
  });
});
