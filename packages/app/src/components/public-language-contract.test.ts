import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

import { setLocale } from "@/lib/i18n";
import { CONTENT } from "@/lib/public-content";

const prefs = readFileSync(
  new URL("./public-prefs-button.tsx", import.meta.url),
  "utf8",
);
const chat = readFileSync(new URL("../pages/chat.tsx", import.meta.url), "utf8");
const api = readFileSync(new URL("../lib/api.ts", import.meta.url), "utf8");
const workspace = readFileSync(
  new URL("./public-agent-workspace.tsx", import.meta.url),
  "utf8",
);

/** Chinese characters outside comments are what makes a surface half-translated. */
function chineseLiterals(source: string): string[] {
  const withoutComments = source
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/^\s*\/\/.*$/gm, "");
  const quoted = withoutComments.match(/"[^"\n]*[一-鿿][^"\n]*"/g) ?? [];
  const templated =
    withoutComments.match(/`[^`\n]*[一-鿿][^`\n]*`/g) ?? [];
  return [...new Set([...quoted, ...templated])];
}

describe("language is reachable and reported", () => {
  it("keeps the switcher with the other per-device reading preferences", () => {
    // It used to exist only in a drawer footer and the mobile menu, which is
    // why users reported not finding it. The prefs panel is one tap from the
    // chat header, beside font size and theme.
    expect(prefs).toContain("setLocale");
    expect(prefs).toContain("CONTENT.chat_page.prefs.language");
    const fontRow = prefs.indexOf("CONTENT.chat_page.prefs.font_size");
    const themeRow = prefs.indexOf("CONTENT.chat_page.prefs.theme}");
    const languageRow = prefs.indexOf("CONTENT.chat_page.prefs.language}");
    expect(fontRow).toBeGreaterThan(-1);
    expect(themeRow).toBeGreaterThan(fontRow);
    expect(languageRow).toBeGreaterThan(themeRow);
  });

  it("tells the server which language the user is reading", () => {
    // Without this the backend can only guess the answer language from the
    // conversation, which is how a Chinese answer reaches an English reader.
    expect(api).toContain("language: useLocale()");
  });

  it("leaves no untranslated string on the chat surface", () => {
    // The composer entry row mixed localized and hardcoded entries, so the
    // same row rendered half Chinese and half English.
    expect(chineseLiterals(chat)).toEqual([]);
  });

  it("leaves no untranslated string on the workspace shell", () => {
    // The shell around the chat (sidebar, top bar, drawer, mobile nav, quick
    // starts) was the largest untranslated surface on the product.
    expect(chineseLiterals(workspace)).toEqual([]);
  });

  it("reads locale-dependent data at render time", () => {
    // Quick starts and seed insights used to be module-level constants, so
    // they captured whichever language was active at import and never changed
    // when the user switched.
    expect(workspace).toContain("const quickStarts = (): QuickStart[] =>");
    expect(workspace).toContain("<For each={quickStarts()}>");
    expect(workspace).toContain(
      "const fallbackInsights = (): AgentWorkspaceInsight[] =>",
    );
  });

  it("translates the earnings entries in both locales", () => {
    setLocale("zh");
    expect(CONTENT.chat_page.earnings.preview_label).toBe("财报前瞻");
    expect(CONTENT.chat_page.prefs.language).toBe("语言");
    setLocale("en");
    expect(CONTENT.chat_page.earnings.preview_label).toBe("Earnings preview");
    expect(CONTENT.chat_page.prefs.language).toBe("Language");
    // Locale labels stay in their own language so either audience can find
    // the one they want.
    expect(CONTENT.chat_page.prefs.language_zh).toBe("中文");
    expect(CONTENT.chat_page.prefs.language_en).toBe("English");
    setLocale("zh");
  });
});
