import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const site = readFileSync(new URL("./public-site.css", import.meta.url), "utf8");
const mobileStart = site.indexOf("@media (max-width: 768px)");
const mobileEnd = site.indexOf('[data-theme="dark"]', mobileStart);
const mobile = site.slice(mobileStart, mobileEnd);

describe("public push detail mobile style contract", () => {
  it("keeps the detail dialog inset, centered, rounded, and safe-area aware", () => {
    expect(mobile).toMatch(
      /\.public-push-detail-backdrop\s*\{[^}]*align-items:\s*flex-end;[^}]*padding:\s*12px;[^}]*padding-bottom:\s*max\(12px, env\(safe-area-inset-bottom\)\);/s,
    );
    expect(mobile).toMatch(
      /\.public-push-detail\s*\{[^}]*position:\s*relative;[^}]*width:\s*100%;[^}]*max-width:\s*680px;[^}]*border-radius:\s*20px;/s,
    );
    expect(mobile).not.toMatch(/\.public-push-detail\s*\{[^}]*left:\s*0;[^}]*right:\s*0;/s);
  });
});
