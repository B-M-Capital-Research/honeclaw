import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const publicPlan = readFileSync(
  new URL("./public-plan.tsx", import.meta.url),
  "utf8",
);

describe("public plan purchase contract", () => {
  it("loads server billing policy and keeps Stripe primary with canonical Whop secondary", () => {
    expect(publicPlan).toContain("getPublicBillingConfig");
    expect(publicPlan).toContain('window.location.assign("/activate")');
    expect(publicPlan).toContain("stripePrimary()");
    expect(publicPlan).toContain(
      "https://whop.com/edda1183-b297-4502-811f-339ae5e773be/bm-research-membership/",
    );
    expect(publicPlan).toContain('href="/activate?provider=whop"');
    expect(publicPlan).toContain('href="/activate?provider=stripe"');
    expect(publicPlan).not.toContain("/vip-copy-18/");
  });

  it("fails closed for the HONE iOS purchase surface", () => {
    expect(publicPlan).toContain("purchases_allowed_on_this_client");
    expect(publicPlan).toContain("App 内仅支持登录并恢复");
  });
});
