import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";
import { loginCodeLooksValid, looksLikeEmail } from "./public-login-form";

const source = readFileSync(new URL("./public-login-form.tsx", import.meta.url), "utf8");

describe("public login form", () => {
  it("validates codes per method: SMS 4–8 alphanumerics, email digits only", () => {
    expect(loginCodeLooksValid("phone", "a1b2")).toBe(true);
    expect(loginCodeLooksValid("phone", "123")).toBe(false);
    expect(loginCodeLooksValid("email", "12345678")).toBe(true);
    expect(loginCodeLooksValid("email", "1234abcd")).toBe(false);
    expect(loginCodeLooksValid("email", "12345")).toBe(false);
  });

  it("does a loose email shape check and leaves normalization to the server", () => {
    expect(looksLikeEmail(" member@example.com ")).toBe(true);
    expect(looksLikeEmail("member@example")).toBe(false);
    expect(looksLikeEmail("member example.com")).toBe(false);
  });

  it("offers both allowlisted methods without a Checkout intent on sign-in", () => {
    expect(source).toContain('role="tablist"');
    expect(source).toContain("await publicSendEmailCode(emailAddress().trim());");
    expect(source).toContain("await publicEmailLogin({");
    expect(source).toContain("await publicSmsLogin({");
    // Captcha stays on the SMS path; the email path is rate limited server-side.
    expect(source.indexOf("verifyCaptchaIfEnabled()")).toBeGreaterThan(
      source.indexOf("await publicSendEmailCode"),
    );
  });
});
