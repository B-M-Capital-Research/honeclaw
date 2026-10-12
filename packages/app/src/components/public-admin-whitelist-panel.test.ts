import { describe, expect, it } from "bun:test";
import {
  adminInviteLabel,
  normalizeDomesticAdminPhone,
  publicAdminCanCreate,
  validateAdminInviteInput,
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

  it("accepts phone-only, email-only and phone + email entries", () => {
    expect(validateAdminInviteInput("+86 138-7139-6421", "")).toEqual({
      ok: true,
      phone_number: "13871396421",
      email_address: undefined,
    });
    expect(validateAdminInviteInput("", " Member@Example.COM ")).toEqual({
      ok: true,
      phone_number: undefined,
      email_address: "member@example.com",
    });
    expect(validateAdminInviteInput("13871396421", "m@example.com")).toEqual({
      ok: true,
      phone_number: "13871396421",
      email_address: "m@example.com",
    });
  });

  it("requires at least one login method and validates what is filled in", () => {
    expect(validateAdminInviteInput("", "  ")).toEqual({ ok: false, reason: "need_one" });
    expect(validateAdminInviteInput("12345", "")).toEqual({ ok: false, reason: "bad_phone" });
    expect(validateAdminInviteInput("", "not-an-email")).toEqual({ ok: false, reason: "bad_email" });
  });

  it("labels email-only entries by their email in confirmations", () => {
    expect(adminInviteLabel({ phone_number: "13871396421", email_address: "m@example.com" })).toBe("13871396421");
    expect(adminInviteLabel({ phone_number: "", email_address: "m@example.com" })).toBe("m@example.com");
  });
});
