import { expect, test } from "@playwright/test";

test("switches from Stripe to Whop on the same activation route", async ({ page }) => {
  await page.route("**/api/public/billing/config", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        primary_provider: "stripe",
        stripe_checkout_enabled: true,
        whop_new_purchases_enabled: true,
        purchases_allowed_on_this_client: true,
        management_allowed_on_this_client: true,
      }),
    });
  });

  await page.goto("/activate");
  await expect(page.getByRole("heading", { name: "验证邮箱并安全结账" })).toBeVisible();

  await page.getByRole("link", { name: "连接 Whop" }).click();

  await expect(page).toHaveURL(/\/activate\?provider=whop$/);
  await expect(page.getByRole("heading", { name: "连接已有 Whop 会员" })).toBeVisible();
  await expect(page.getByText("WHOP 会员连接", { exact: true })).toBeVisible();
});
