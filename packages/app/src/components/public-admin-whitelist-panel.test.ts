import { describe, expect, it } from "bun:test";
import {
  normalizeDomesticAdminPhone,
  publicAdminCanCreate,
} from "./public-admin-whitelist-panel";

describe("public admin whitelist panel", () => {
  it("normalizes local and +86 domestic phone input", () => {
    expect(normalizeDomesticAdminPhone("138 7139 6421")).toBe("13871396421");
    expect(normalizeDomesticAdminPhone("+86 138-7139-6421")).toBe("13871396421");
  });

  it("stops creating when the server-authoritative daily allowance is exhausted", () => {
    expect(publicAdminCanCreate({ remaining_today: 1 }, false)).toBe(true);
    expect(publicAdminCanCreate({ remaining_today: 0 }, false)).toBe(false);
    expect(publicAdminCanCreate({ remaining_today: 3 }, true)).toBe(false);
    expect(publicAdminCanCreate(null, false)).toBe(false);
  });
});
