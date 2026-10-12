import { describe, expect, it } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { setLocale, type Locale } from "@/lib/i18n";
import { CONTENT } from "@/lib/public-content";

function landingFor(locale: Locale) {
  setLocale(locale);
  return JSON.parse(JSON.stringify(CONTENT.landing)) as typeof CONTENT.landing;
}

const page = readFileSync(new URL("./public-home.tsx", import.meta.url), "utf8");
const css = readFileSync(new URL("./public-home.css", import.meta.url), "utf8");
const publicDir = new URL("../../public/", import.meta.url);

describe("public home page", () => {
  it("presents the current product surfaces and links to each of them", () => {
    for (const href of ["/chat", "/research", "/community", "/me", "/plan"]) {
      expect(landingFor("zh").products.map((p) => p.href).concat(["/plan"])).toContain(href);
    }
    expect(page).toContain('id="quant"');
    expect(page).toContain("CONTENT.landing");
    expect(page).toContain("<Title>{L.page_title}</Title>");
  });

  it("only uses current product captures that exist in public/", () => {
    const images = [...page.matchAll(/["'](\/[\w-]+\.(?:jpg|png|webp))["']/g)].map((match) => match[1]);
    expect(images).toEqual(
      expect.arrayContaining(["/home-assistant.jpg", "/home-research.jpg", "/home-influencer-mobile.jpg"]),
    );
    for (const image of images) {
      expect(existsSync(new URL(`.${image}`, publicDir))).toBe(true);
    }
    // Retired captures of the April infographics and the 0.1.x admin console.
    for (const retired of ["hone_page", "hone_channels", "hone_solution", "hone_work", "hone_introduction", "company_profile"]) {
      expect(page).not.toContain(retired);
    }
  });

  it("keeps the copy current and claims only what the product ships", () => {
    for (const landing of [landingFor("zh"), landingFor("en")]) {
      const text = JSON.stringify(landing);
      // Self-hosted chat channels are not the hosted product; never advertise them here.
      expect(text).not.toMatch(/iMessage|Telegram|飞书|Lark|7 接入渠道/);
      // Sample-data screenshots are labelled as examples.
      expect(landing.research.caption.length).toBeGreaterThan(0);
      expect(landing.hero_caption.length).toBeGreaterThan(0);
      expect(landing.products).toHaveLength(4);
    }
    setLocale("zh");
    // The hone-quant backtest always travels with its disclaimer.
    expect(page).toContain("{Q.disclaimer}");
    expect(page).toContain("{L.disclaimer}");
  });

  it("styles with tokens so the dark theme follows", () => {
    expect(css).toContain("var(--hone-surface-raised)");
    expect(css).not.toContain("!important");
  });
});
