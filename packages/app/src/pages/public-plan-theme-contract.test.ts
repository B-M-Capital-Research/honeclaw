import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const plan = readFileSync(new URL("./public-plan.tsx", import.meta.url), "utf8");

describe("public plan theme contract", () => {
  it("uses theme-aware surfaces for badges, stats, socials, and content cards", () => {
    for (const selector of [
      ".hone-hub-id h1 span",
      ".hone-hub-stats > div",
      ".hone-hub-social",
      ".hone-hub-card",
    ]) {
      expect(plan).toMatch(
        new RegExp(
          `${selector.replace(/[.*+?^${}()|[\\]\\]/g, "\\$&")} \\{[^}]*background: var\\(--hone-surface-raised\\)`,
          "s",
        ),
      );
    }
  });

  it("pairs dark-theme action backgrounds with their matching foregrounds", () => {
    expect(plan).toMatch(
      /\.hone-hub-solid-btn \{[^}]*background: var\(--hone-ink-950\)[^}]*color: var\(--hone-action-fg\)/s,
    );
    expect(plan).toMatch(
      /\.hone-share-buy \{[^}]*background: var\(--hone-action-bg\)[^}]*color: var\(--hone-action-fg\)/s,
    );
  });

  it("uses control surfaces for secondary actions and the modal close button", () => {
    expect(plan).toMatch(
      /\.hone-hub-ghost-btn \{[^}]*background: var\(--hone-control-surface\)/s,
    );
    expect(plan).toMatch(
      /\.hone-hub-trial-btn \{[^}]*background: var\(--hone-control-surface\)/s,
    );
    expect(plan).toMatch(
      /\.hone-share-pop figcaption button \{[^}]*background: var\(--hone-control-surface\)/s,
    );
  });
});
