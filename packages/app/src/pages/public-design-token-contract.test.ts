import { describe, expect, it } from "bun:test";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

const SRC = new URL("..", import.meta.url).pathname;

function walk(dir: string, out: string[] = []): string[] {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) walk(path, out);
    else if (/\.(css|tsx|ts)$/.test(entry) && !entry.includes(".test.")) out.push(path);
  }
  return out;
}

const files = walk(SRC).map((path) => ({ path, text: readFileSync(path, "utf8") }));

const defined = new Set<string>();
for (const { text } of files) {
  for (const match of text.matchAll(/(--hone-[a-z0-9-]+)\s*:/g)) defined.add(match[1]);
}

describe("one design-token vocabulary, one theme switch", () => {
  it("never references a token nothing defines", () => {
    // The survey page was written against `--hone-surface` / `--hone-text` /
    // `--hone-border` / `--hone-accent`, none of which exist. Every reference
    // silently fell through to a hardcoded fallback written for the opposite
    // theme, so in dark mode its heading rendered at a contrast ratio of 1.01
    // — invisible — and its option chips stayed bright white. A phantom token
    // fails silently; only a check like this one surfaces it.
    const missing: string[] = [];
    for (const { path, text } of files) {
      for (const match of text.matchAll(/var\(\s*(--hone-[a-z0-9-]+)/g)) {
        if (!defined.has(match[1])) {
          missing.push(`${match[1]} in ${path.replace(SRC, "")}`);
        }
      }
    }
    expect(missing).toEqual([]);
  });

  it("keeps the theme on the app's own attribute, never on the OS query", () => {
    // The theme is a user preference with an explicit `auto` option. A surface
    // that keys off `prefers-color-scheme` ignores that choice, so it could
    // render dark inside a page the user had set to light.
    const offenders = files
      .filter(({ text }) => text.includes("prefers-color-scheme"))
      .map(({ path }) => path.replace(SRC, ""))
      // The preference resolver is the one place allowed to read the OS.
      .filter((path) => !path.endsWith("lib/public-prefs.ts"));
    expect(offenders).toEqual([]);
  });

  it("resolves the theme from the system until the user picks one", () => {
    const prefs = readFileSync(join(SRC, "lib/public-prefs.ts"), "utf8");
    // A first visit on a dark machine used to get a light page with nothing
    // on screen explaining why, while the interface language already followed
    // the system. The two preferences now behave the same way.
    expect(prefs).toContain('return "auto"');
    expect(prefs).not.toMatch(/normalizeStoredPublicTheme[\s\S]{0,200}return "light"/);
  });

  it("flips the palette at the token layer so surfaces inherit it", () => {
    // Tokens are redefined under the dark attribute; a surface built from
    // them themes itself. This is what makes the phantom-token check above
    // worth enforcing rather than merely tidy.
    const foundation = readFileSync(join(SRC, "pages/public-foundation.css"), "utf8");
    expect(foundation).toContain('[data-theme="dark"]');
    for (const token of ["--hone-ink-950", "--hone-paper-50", "--hone-surface-raised"]) {
      const darkBlock = foundation.slice(foundation.indexOf('[data-theme="dark"]'));
      expect(darkBlock).toContain(token);
    }
  });
});
