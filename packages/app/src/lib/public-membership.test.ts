import { describe, expect, it } from "bun:test";
import type {
  PublicAuthUserInfo,
  PublicBillingConfig,
  PublicBillingEntitlement,
} from "@/lib/types";
import {
  billingActivationProvider,
  billingEntitlementGrantsAccess,
  billingEntitlementStatusLabel,
  publicUserHasProductAccess,
} from "./public-membership";

function billingConfig(
  primaryProvider: "stripe" | "whop",
  stripeCheckoutEnabled: boolean,
): PublicBillingConfig {
  return {
    primary_provider: primaryProvider,
    stripe_checkout_enabled: stripeCheckoutEnabled,
    whop_new_purchases_enabled: true,
    purchases_allowed_on_this_client: true,
    management_allowed_on_this_client: true,
  };
}

function entitlement(
  accessState: string,
  cancelAtPeriodEnd = false,
  provider = "stripe",
): PublicBillingEntitlement {
  return {
    entitlement_id: "ent_1",
    provider,
    raw_status: accessState,
    access_state: accessState,
    cancel_at_period_end: cancelAtPeriodEnd,
    grace_expires_at: accessState === "grace" ? "2099-08-03T00:00:00Z" : undefined,
  };
}

function user(accessGranted: boolean): PublicAuthUserInfo {
  return {
    user_id: "web-user",
    created_at: "2026-07-26T00:00:00Z",
    daily_limit: 20,
    success_count: 0,
    in_flight: 0,
    remaining_today: 20,
    has_password: false,
    identity_kind: "international_email",
    billing: {
      access_granted: accessGranted,
      entitlements: [],
      has_duplicate_active_subscriptions: false,
    },
    is_admin: false,
  };
}

describe("public membership policy", () => {
  it("routes activation through the server-selected available provider", () => {
    expect(billingActivationProvider(undefined, billingConfig("whop", false))).toBe("whop");
    expect(billingActivationProvider(undefined, billingConfig("stripe", true))).toBe("stripe");
    expect(billingActivationProvider("stripe", billingConfig("whop", true))).toBe("stripe");
    expect(billingActivationProvider("stripe", billingConfig("whop", false))).toBe("whop");
    expect(billingActivationProvider("whop", billingConfig("stripe", true))).toBe("whop");
  });

  it("treats any provider active or grace entitlement as access granting", () => {
    for (const provider of ["stripe", "whop"]) {
      expect(billingEntitlementGrantsAccess(entitlement("active", false, provider))).toBe(true);
      expect(billingEntitlementGrantsAccess(entitlement("grace", false, provider))).toBe(true);
      expect(billingEntitlementGrantsAccess(entitlement("pending", false, provider))).toBe(false);
      expect(billingEntitlementGrantsAccess(entitlement("inactive", false, provider))).toBe(false);
    }
  });

  it("fails closed when grace has no valid future deadline", () => {
    const missing = entitlement("grace");
    missing.grace_expires_at = undefined;
    expect(billingEntitlementGrantsAccess(missing)).toBe(false);

    const expired = entitlement("grace");
    expired.grace_expires_at = "2020-08-03T00:00:00Z";
    expect(billingEntitlementGrantsAccess(expired)).toBe(false);
    expect(billingEntitlementStatusLabel(expired)).toBe("宽限期已结束");
  });

  it("uses the backend policy result and fails closed", () => {
    expect(publicUserHasProductAccess(user(true))).toBe(true);
    expect(publicUserHasProductAccess(user(false))).toBe(false);
  });

  it("explains cancel-at-period-end without revoking current access", () => {
    const canceling = entitlement("active", true);
    expect(billingEntitlementGrantsAccess(canceling)).toBe(true);
    expect(billingEntitlementStatusLabel(canceling)).toBe(
      "本周期结束后停止续费",
    );

    const ended = entitlement("inactive", true);
    ended.raw_status = "canceled";
    expect(billingEntitlementStatusLabel(ended)).toBe("已取消");
  });
});
