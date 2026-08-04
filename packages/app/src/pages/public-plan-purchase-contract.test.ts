import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const publicPlan = readFileSync(
  new URL("./public-plan.tsx", import.meta.url),
  "utf8",
);

describe("public plan purchase contract", () => {
  it("loads server billing policy and exposes only Stripe Checkout", () => {
    expect(publicPlan).toContain("getPublicBillingConfig");
    expect(publicPlan).toContain('window.location.assign("/activate")');
    expect(publicPlan).toContain("stripeAvailable()");
    expect(publicPlan).toContain('href="/activate"');
    expect(publicPlan).not.toContain("whop");
    expect(publicPlan).not.toContain("provider=");
    expect(publicPlan).not.toContain("/vip-copy-18/");
  });

  it("fails closed for the HONE iOS purchase surface", () => {
    expect(publicPlan).toContain("purchases_allowed_on_this_client");
    expect(publicPlan).toContain("App 内仅支持登录并恢复");
  });
});
