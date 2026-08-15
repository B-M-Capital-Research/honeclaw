import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const page = readFileSync(new URL("./public-community.tsx", import.meta.url), "utf8");
const css = readFileSync(new URL("./public-community.css", import.meta.url), "utf8");

describe("public community interaction contract", () => {
  it("separates official material from member discussion", () => {
    expect(page).toContain("官方动态");
    expect(page).toContain("讨论区");
    expect(page).toContain("未经官方核验");
    expect(page).toContain("CommunityForum");
  });

  it("keeps image preview zoomable by touch, pointer, wheel, and explicit controls", () => {
    expect(page).toContain('addEventListener("touchmove"');
    expect(page).toContain('addEventListener("pointermove"');
    expect(page).toContain('addEventListener("wheel"');
    expect(page).toContain('addEventListener("dblclick"');
    expect(page).toContain("CONTENT.chat_page.community_page.image_zoom");
    expect(page).toContain("CONTENT.chat_page.community_page.fit_screen");
    expect(css).toContain("touch-action: none");
  });

  it("renders the modal through a focus-managed accessible portal", () => {
    expect(page).toContain("<Portal>");
    expect(page).toContain('aria-modal="true"');
    expect(page).toContain('event.key === "Escape"');
    expect(page).toContain('event.key !== "Tab"');
    expect(page).toContain('setAttribute("inert"');
    expect(page).toContain("previousFocus?.focus()");
  });

  it("sandboxes inline files and uses authenticated blob downloads", () => {
    expect(page).toContain('sandbox="allow-downloads allow-same-origin"');
    expect(page).toContain("getPublicCommunityResourceBlob");
    expect(page).toContain("CONTENT.chat_page.community_page.pdf_slow");
    expect(page).toContain("CONTENT.chat_page.community_page.pdf_unsupported");
    expect(page).not.toContain('target="_blank"');
  });

  it("keeps pagination failures inline and lays out multiple images as a grid", () => {
    expect(page).toContain("setLoadMoreError");
    expect(page).toContain('class="public-community-image-grid"');
    expect(css).toContain("grid-template-columns: repeat(2, minmax(0, 1fr))");
  });

  it("keeps the final timeline card clear of the mobile tab bar", () => {
    expect(css).toContain(
      "padding-bottom: calc(94px + env(safe-area-inset-bottom))",
    );
  });
});
