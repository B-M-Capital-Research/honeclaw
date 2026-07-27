import { describe, expect, it } from "bun:test";
import type { PublicAuthUserInfo, WhopMembershipInfo } from "@/lib/types";
import {
  publicUserHasProductAccess,
  whopMembershipGrantsAccess,
  whopMembershipStatusLabel,
} from "./public-membership";

function membership(
  status: string,
  cancelAtPeriodEnd = false,
): WhopMembershipInfo {
  return {
    membership_id: "mem_1",
    whop_user_id: "user_1",
    company_id: "biz_1",
    product_id: "prod_1",
    plan_id: "plan_1",
    status,
    cancel_at_period_end: cancelAtPeriodEnd,
    last_event_id: "event_1",
    last_event_at: "2026-07-26T00:00:00Z",
    updated_at: "2026-07-26T00:00:00Z",
  };
}

function user(
  policy: string,
  whopMembership?: WhopMembershipInfo,
): PublicAuthUserInfo {
  return {
    user_id: "web-user",
    created_at: "2026-07-26T00:00:00Z",
    daily_limit: 20,
    success_count: 0,
    in_flight: 0,
    remaining_today: 20,
    has_password: false,
    registration_policy: policy,
    whop_membership: whopMembership,
  };
}

describe("public membership policy", () => {
  it("matches the backend Whop access-granting statuses", () => {
    for (const status of ["active", "trialing", "past_due", "canceling"]) {
      expect(whopMembershipGrantsAccess(membership(status))).toBe(true);
    }
    for (const status of ["canceled", "expired", "completed", "unresolved"]) {
      expect(whopMembershipGrantsAccess(membership(status))).toBe(false);
    }
  });

  it("keeps the domestic phone path independent from Whop", () => {
    expect(publicUserHasProductAccess(user("cn_domestic"))).toBe(true);
    expect(
      publicUserHasProductAccess(
        user("whop_international", membership("canceled")),
      ),
    ).toBe(false);
  });

  it("explains cancel-at-period-end without revoking the current period", () => {
    const canceling = membership("active", true);
    expect(whopMembershipGrantsAccess(canceling)).toBe(true);
    expect(whopMembershipStatusLabel(canceling)).toBe(
      "本周期结束后停止续费",
    );
  });
});
