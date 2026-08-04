import { expect, test, type Page } from "@playwright/test";

async function routeBillingConfig(
  page: Page,
  config: { primary_provider: "stripe" | "whop"; stripe_checkout_enabled: boolean },
) {
  await page.route("**/api/public/billing/config", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        primary_provider: config.primary_provider,
        stripe_checkout_enabled: config.stripe_checkout_enabled,
        whop_new_purchases_enabled: true,
        purchases_allowed_on_this_client: true,
        management_allowed_on_this_client: true,
      }),
    });
  });
}

test("switches from Stripe to Whop on the same activation route", async ({ page }) => {
  await routeBillingConfig(page, {
    primary_provider: "stripe",
    stripe_checkout_enabled: true,
  });

  await page.goto("/activate");
  await expect(page.getByRole("heading", { name: "验证邮箱并安全结账" })).toBeVisible();

  await page.getByRole("link", { name: "连接 Whop" }).click();

  await expect(page).toHaveURL(/\/activate\?provider=whop$/);
  await expect(page.getByRole("heading", { name: "连接已有 Whop 会员" })).toBeVisible();
  await expect(page.getByText("WHOP 会员连接", { exact: true })).toBeVisible();
});

test("defaults to Whop while Stripe Checkout is disabled", async ({ page }) => {
  await routeBillingConfig(page, {
    primary_provider: "whop",
    stripe_checkout_enabled: false,
  });

  await page.goto("/activate?provider=stripe");

  await expect(page.getByRole("heading", { name: "连接已有 Whop 会员" })).toBeVisible();
  await expect(page.getByText("WHOP 会员连接", { exact: true })).toBeVisible();
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
        primary_provider: "whop",
        stripe_checkout_enabled: false,
        whop_new_purchases_enabled: true,
        purchases_allowed_on_this_client: true,
        management_allowed_on_this_client: true,
      }),
    });
  });

  const navigation = page.goto("/activate?provider=stripe");
  await configRequested;
  try {
    await expect(page.getByRole("heading", { name: "正在确认会员渠道" })).toBeVisible();
    await expect(page.getByRole("textbox")).toHaveCount(0);
  } finally {
    releaseConfig();
  }
  await navigation;
  await expect(page.getByRole("heading", { name: "连接已有 Whop 会员" })).toBeVisible();
});
