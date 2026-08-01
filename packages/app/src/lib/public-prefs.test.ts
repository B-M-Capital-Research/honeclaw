import { describe, expect, it } from "bun:test";
import { normalizeStoredPublicTheme } from "./public-prefs";

describe("public theme preference", () => {
  it("defaults a new or invalid preference to the light theme", () => {
    expect(normalizeStoredPublicTheme(null)).toBe("light");
    expect(normalizeStoredPublicTheme("unexpected")).toBe("light");
  });

  it("preserves every explicit supported preference", () => {
    expect(normalizeStoredPublicTheme("light")).toBe("light");
    expect(normalizeStoredPublicTheme("dark")).toBe("dark");
    expect(normalizeStoredPublicTheme("auto")).toBe("auto");
  });
});
