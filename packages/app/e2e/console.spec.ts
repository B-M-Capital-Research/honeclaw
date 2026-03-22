import { expect, test } from "@playwright/test"

test("console shell renders", async ({ page }) => {
  await page.goto("/")
  await expect(page.getByText("会话")).toBeVisible()
  await expect(page.getByText("技能库")).toBeVisible()
})
