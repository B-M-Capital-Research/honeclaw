import { describe, expect, it } from "bun:test";
import { normalizeStoredPublicTheme } from "./public-prefs";

describe("public theme preference", () => {
  it("defaults a new or invalid preference to the light theme", () => {
    // A first visit follows the system, the same way the language does.
    expect(normalizeStoredPublicTheme(null)).toBe("auto");
    expect(normalizeStoredPublicTheme("unexpected")).toBe("auto");
  });

  it("preserves every explicit supported preference", () => {
    expect(normalizeStoredPublicTheme("light")).toBe("light");
    expect(normalizeStoredPublicTheme("dark")).toBe("dark");
    expect(normalizeStoredPublicTheme("auto")).toBe("auto");
  });
});
