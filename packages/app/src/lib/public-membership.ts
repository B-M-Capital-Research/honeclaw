import type {
  PublicAuthUserInfo,
  PublicBillingEntitlement,
} from "@/lib/types";

export function billingEntitlementGrantsAccess(
  entitlement: PublicBillingEntitlement,
): boolean {
  if (entitlement.access_state === "active") return true;
  if (entitlement.access_state !== "grace" || !entitlement.grace_expires_at) return false;
  const deadline = Date.parse(entitlement.grace_expires_at);
  return Number.isFinite(deadline) && deadline >= Date.now();
}

export function publicUserHasProductAccess(user: PublicAuthUserInfo): boolean {
  return user.billing.access_granted;
}

export function billingEntitlementStatusLabel(
  entitlement: PublicBillingEntitlement,
): string {
  if (entitlement.cancel_at_period_end && entitlement.access_state === "active") {
    return "本周期结束后停止续费";
  }
  switch (entitlement.access_state) {
    case "active":
      return "生效中";
    case "grace":
      return billingEntitlementGrantsAccess(entitlement)
        ? "付款待恢复（宽限期）"
        : "宽限期已结束";
    case "pending":
      return "待处理";
    case "inactive":
      return entitlement.raw_status === "canceled" ? "已取消" : "已失效";
    default:
      return entitlement.raw_status || "未知";
  }
}

export function billingProviderLabel(provider: string): string {
  switch (provider) {
    case "stripe":
      return "Stripe";
    case "domestic_invite":
      return "国内邀请";
    default:
      return provider;
  }
}
