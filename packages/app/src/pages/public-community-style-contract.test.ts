import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const page = readFileSync(new URL("./public-community.tsx", import.meta.url), "utf8");
const css = readFileSync(new URL("./public-community.css", import.meta.url), "utf8");

describe("public community interaction contract", () => {
  it("keeps image preview zoomable by touch, pointer, wheel, and explicit controls", () => {
    expect(page).toContain('addEventListener("touchmove"');
    expect(page).toContain('addEventListener("pointermove"');
    expect(page).toContain('addEventListener("wheel"');
    expect(page).toContain('addEventListener("dblclick"');
    expect(page).toContain('aria-label="图片缩放"');
    expect(page).toContain("适应屏幕");
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
    expect(page).toContain('sandbox="allow-downloads"');
    expect(page).toContain("getPublicCommunityResourceBlob");
    expect(page).not.toContain('target="_blank"');
  });

  it("keeps pagination failures inline and lays out multiple images as a grid", () => {
    expect(page).toContain("setLoadMoreError");
    expect(page).toContain('class="public-community-image-grid"');
    expect(css).toContain("grid-template-columns: repeat(2, minmax(0, 1fr))");
  });
});
