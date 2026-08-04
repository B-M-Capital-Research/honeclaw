import { expect, test, type Page } from "@playwright/test";

async function routeBillingConfig(page: Page, stripeCheckoutEnabled: boolean) {
  await page.route("**/api/public/billing/config", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        stripe_checkout_enabled: stripeCheckoutEnabled,
        purchases_allowed_on_this_client: true,
        management_allowed_on_this_client: true,
      }),
    });
  });
}

test("offers Stripe Checkout on the single activation route", async ({ page }) => {
  await routeBillingConfig(page, true);

  await page.goto("/activate");

  await expect(page.getByRole("heading", { name: "验证邮箱并安全结账" })).toBeVisible();
  await expect(page.getByText("STRIPE 安全结账", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "验证并前往 Stripe" })).toBeVisible();
});

test("fails closed to entitlement recovery while Stripe Checkout is disabled", async ({ page }) => {
  await routeBillingConfig(page, false);

  await page.goto("/activate?provider=legacy");

  await expect(page.getByRole("heading", { name: "恢复你的 HONE 会员权益" })).toBeVisible();
  await expect(page.getByText("会员恢复", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "验证并恢复权益" })).toBeVisible();
  await expect(page.getByRole("button", { name: "验证并前往 Stripe" })).toHaveCount(0);
});

test("waits for server billing policy before collecting account data", async ({ page }) => {
  let releaseConfig = () => {};
  let markConfigRequested = () => {};
  const configGate = new Promise<void>((resolve) => {
    releaseConfig = resolve;
  });
  const configRequested = new Promise<void>((resolve) => {
    markConfigRequested = resolve;
  });
  await page.route("**/api/public/billing/config", async (route) => {
    markConfigRequested();
    await configGate;
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        stripe_checkout_enabled: false,
        purchases_allowed_on_this_client: true,
        management_allowed_on_this_client: true,
      }),
    });
  });

  const navigation = page.goto("/activate");
  await configRequested;
  try {
    await expect(page.getByRole("heading", { name: "正在确认会员渠道" })).toBeVisible();
    await expect(page.getByRole("textbox")).toHaveCount(0);
  } finally {
    releaseConfig();
  }
  await navigation;
  await expect(page.getByRole("heading", { name: "恢复你的 HONE 会员权益" })).toBeVisible();
});
