import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const app = readFileSync(new URL("../app.tsx", import.meta.url), "utf8");
const activation = readFileSync(
  new URL("./public-whop-activate.tsx", import.meta.url),
  "utf8",
);
const login = readFileSync(
  new URL("../components/public-login-form.tsx", import.meta.url),
  "utf8",
);
const account = readFileSync(new URL("./public-me.tsx", import.meta.url), "utf8");
const api = readFileSync(new URL("../lib/api.ts", import.meta.url), "utf8");
const publicSiteCss = readFileSync(
  new URL("./public-site.css", import.meta.url),
  "utf8",
);

describe("Whop to HONE activation contract", () => {
  it("exposes a dedicated purchase-email activation route from login", () => {
    expect(app).toContain('<Route path="/activate/whop"');
    expect(login).toContain('href="/activate/whop"');
    expect(activation).toContain("使用 Whop 付款时填写的邮箱验证身份");
  });

  it("loads its public surface styles directly and owns a responsive scrollable page", () => {
    expect(activation).toContain('import "./public-foundation.css"');
    expect(activation).toContain('import "./public-site.css"');
    expect(activation).toContain('class="public-login-screen public-whop-activate"');
    expect(activation).toContain('"overflow-y": "auto"');
    expect(activation).toContain('<PublicPrefsButton />');
    expect(publicSiteCss).toContain(".public-whop-activate-card");
    expect(publicSiteCss).toContain(".public-whop-code-row");
    expect(publicSiteCss).toContain("place-items: start center !important");
  });

  it("uses the shared light and dark form surfaces instead of hard-coded white", () => {
    expect(activation).toContain('background: "var(--hone-surface-raised)"');
    expect(activation).toContain('background: "var(--hone-control-surface)"');
    expect(activation).toContain('? "var(--hone-action-bg)"');
    expect(activation).not.toContain('background: "#fff"');
  });

  it("uses HONE-owned email verification endpoints and current terms", () => {
    expect(api).toContain('"/api/public/auth/email/send"');
    expect(api).toContain('"/api/public/auth/email/login"');
    expect(activation).toContain("tos_version: TOS_VERSION");
    expect(activation).toContain("navigate(\"/me?checkout=success\")");
  });

  it("renders server-authoritative membership state instead of equating login with payment", () => {
    expect(account).toContain("whopMembershipStatusLabel");
    expect(account).toContain("publicUserHasProductAccess");
    expect(account).not.toContain("能登录 HONE 即代表你是年度付费会员");
  });
});
