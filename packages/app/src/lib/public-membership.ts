import type {
  PublicAuthUserInfo,
  PublicBillingEntitlement,
} from "@/lib/types";

export function billingEntitlementGrantsAccess(
  entitlement: PublicBillingEntitlement,
): boolean {
  return entitlement.grants_access;
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

export function billingEntitlementKindLabel(
  entitlementKind: string,
): string {
  switch (entitlementKind) {
    case "recurring_subscription":
      return "自动续费年订阅";
    case "fixed_term_purchase":
      return "一次性年费（不自动续费）";
    case "domestic_invite":
      return "国内邀请权益";
    default:
      return entitlementKind;
  }
}
