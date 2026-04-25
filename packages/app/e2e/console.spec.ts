import { expect, test } from "@playwright/test"

test("admin shell renders new IA groups", async ({ page }) => {
  await page.goto("/")
  // 默认重定向到 /dashboard
  await expect(page).toHaveURL(/\/dashboard$/)

  // Sidebar 三组关键标题(用 sidebar 内的精确 link/section 定位避免 dashboard 卡片重名)
  await expect(page.getByRole("link", { name: "概览" })).toBeVisible()
  await expect(page.getByRole("link", { name: "用户档案" })).toBeVisible()
  await expect(page.getByRole("link", { name: "设置" })).toBeVisible()

  // sidebar 分组标题(uppercase tracking 的 label)
  await expect(page.getByText("用户视角", { exact: true })).toBeVisible()
  await expect(page.getByText("系统", { exact: true })).toBeVisible()

  // Dashboard 着陆页核心区
  await expect(page.getByText("快速发起对话")).toBeVisible()
  await expect(page.getByText("最近会话")).toBeVisible()
})

test("legacy paths redirect to new IA", async ({ page }) => {
  await page.goto("/start")
  await expect(page).toHaveURL(/\/dashboard$/)

  await page.goto("/memory")
  await expect(page).toHaveURL(/\/users$/)

  await page.goto("/portfolio/imessage%7C%7Calice")
  await expect(page).toHaveURL(/\/users\/imessage%7C%7Calice\/portfolio$/)
})

test("symbol drawer opens via ?symbol= query", async ({ page }) => {
  await page.goto("/dashboard?symbol=AAPL")
  // 抽屉头部的 Symbol 标签 + 大写 symbol
  await expect(page.getByText("Symbol", { exact: true })).toBeVisible()
  await expect(page.getByText("AAPL", { exact: true })).toBeVisible()
  // 4 个 tab 都能定位到
  await expect(page.getByRole("button", { name: "公司画像" })).toBeVisible()
  await expect(page.getByRole("button", { name: "研究记录" })).toBeVisible()
  await expect(page.getByRole("button", { name: "相关会话" })).toBeVisible()
  await expect(page.getByRole("button", { name: "操作" })).toBeVisible()
})
