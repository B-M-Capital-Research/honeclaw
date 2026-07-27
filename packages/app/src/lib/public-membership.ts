import type { PublicAuthUserInfo, WhopMembershipInfo } from "@/lib/types";

const WHOP_ACCESS_STATUSES = new Set([
  "active",
  "trialing",
  "past_due",
  "canceling",
]);

export function whopMembershipGrantsAccess(
  membership?: WhopMembershipInfo,
): boolean {
  return membership
    ? WHOP_ACCESS_STATUSES.has(membership.status.toLowerCase())
    : false;
}

export function publicUserHasProductAccess(user: PublicAuthUserInfo): boolean {
  if (user.registration_policy !== "whop_international") return true;
  return whopMembershipGrantsAccess(user.whop_membership);
}

export function whopMembershipStatusLabel(
  membership: WhopMembershipInfo,
): string {
  if (membership.cancel_at_period_end || membership.status === "canceling") {
    return "本周期结束后停止续费";
  }
  switch (membership.status.toLowerCase()) {
    case "active":
      return "生效中";
    case "trialing":
      return "试用中";
    case "past_due":
      return "付款待处理";
    case "completed":
      return "已完成";
    case "canceled":
      return "已取消";
    case "expired":
      return "已到期";
    case "unresolved":
      return "待处理";
    default:
      return membership.status || "未知";
  }
}
