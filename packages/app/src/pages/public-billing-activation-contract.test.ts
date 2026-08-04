import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");
const app = read("../app.tsx");
const activation = read("./public-activate.tsx");
const login = read("../components/public-login-form.tsx");
const account = read("./public-me.tsx");
const api = read("../lib/api.ts");
const css = read("./public-site.css");

describe("Stripe-only billing activation contract", () => {
  it("uses one Stripe-only activation route", () => {
    expect(app).toContain('<Route path="/activate"');
    expect(app).not.toContain('<Route path="/activate/whop"');
    expect(login).toContain('href="/activate"');
    expect(activation).not.toContain("useSearchParams");
    expect(activation).not.toContain("billingActivationProvider");
    expect(activation).not.toContain("whop");
    expect(activation).not.toContain("new URLSearchParams(window.location.search)");
  });

  it("follows the server Stripe availability and fails closed when Checkout is disabled", () => {
    expect(activation).toContain("getPublicBillingConfig");
    expect(activation).toContain("purchaseAvailable()");
    expect(activation).toContain("stripe_checkout_enabled");
    expect(activation).toContain("configReady()");
    expect(activation).toContain("正在确认会员渠道");
    expect(activation).toContain("Stripe 结账暂不可用");
  });

  it("creates Checkout only after HONE email verification", () => {
    expect(api).toContain('"/api/public/auth/email/send"');
    expect(api).toContain('"/api/public/auth/email/login"');
    expect(api).toContain('"/api/public/billing/checkout/stripe"');
    expect(activation).toContain("tos_version: TOS_VERSION");
    expect(activation).toContain("await publicEmailLogin");
    expect(activation).toContain("await createStripeCheckout");
    expect(activation).toContain("window.location.assign(checkout_url)");
  });

  it("removes purchase language from the restore-only client flow", () => {
    expect(activation).toContain('if (!purchaseAvailable()) return ["验证邮箱", "登录账户", "恢复权益"]');
    expect(activation).toContain("restoreOnly() || user.billing.access_granted");
    expect(activation).toContain('purchaseAvailable() ? "stripe_checkout" : undefined');
  });

  it("never equates a success redirect with paid access", () => {
    expect(account).toContain("billing.access_granted");
    expect(account).toContain("正在等待付款平台确认");
    expect(account).toContain("getPublicAuthMe");
    expect(account).not.toContain("checkout=success");
  });

  it("owns responsive Stripe activation styles", () => {
    expect(activation).toContain('class="public-login-screen public-activate"');
    expect(activation).toContain('import "./public-site.css"');
    expect(css).toContain(".public-activate-card");
    expect(css).toContain(".public-activate-code-row");
    expect(css).not.toContain(".public-whop-activate");
  });

  it("uses server-owned account state and gates Stripe management by server client policy", () => {
    expect(account).toContain("billingEntitlementStatusLabel");
    expect(account).toContain("createStripePortal");
    expect(account).toContain("getPublicBillingConfig");
    expect(account).toContain("config.management_allowed_on_this_client");
    expect(account).toContain("managementAllowed={managementAllowed()}");
    expect(account).toContain('props.managementAllowed && props.entitlement.provider === "stripe"');
    expect(account).not.toContain('props.entitlement.provider === "whop"');
    expect(account).not.toContain("Whop");
    expect(account).toContain("has_duplicate_active_subscriptions");
    expect(account).not.toContain("whop_membership");
    expect(account).not.toContain("registration_policy");
  });
});
