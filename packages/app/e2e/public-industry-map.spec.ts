import { expect, test, type Page, type Route } from "@playwright/test";
import { readFileSync } from "node:fs";

// 行业分析页的阅读顺序、公司视角与底稿折叠。行业本体只对管理员开放，所以这里全部以管理员
// 身份跑；读者被拒的路径由 public-data-center.spec.ts 覆盖。
const industryMap = JSON.parse(readFileSync(
  new URL("../../../skills/industry-map/references/industry-map.json", import.meta.url), "utf8",
)) as { industries: { id: string; name: string; members: { symbol: string }[] }[] } & Record<string, unknown>;

async function json(route: Route, payload: unknown, status = 200) {
  await route.fulfill({ status, contentType: "application/json", body: JSON.stringify(payload) });
}

async function installAdminMocks(page: Page) {
  const user = {
    user_id: "industry-admin",
    created_at: "2026-09-01T00:00:00Z",
    daily_limit: 20,
    success_count: 0,
    in_flight: 0,
    remaining_today: 20,
    has_password: true,
    is_admin: true,
    tos_accepted_at: "2026-09-01T00:00:00Z",
    tos_version: "2.1",
  };
  const state = { industryReads: 0, edits: 0, chatSends: 0 };
  await page.addInitScript(() => {
    localStorage.clear();
    localStorage.setItem("hone-public-locale", "zh");
  });
  await page.route("**/api/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === "/api/meta") {
      return json(route, {
        name: "hone", version: "test", channel: "web", apiVersion: "desktop-v1",
        capabilities: ["public_chat"], deploymentMode: "remote", supportsImessage: false,
      });
    }
    if (path === "/api/public/auth/me") return json(route, { user });
    if (path === "/api/public/bootstrap") {
      return json(route, { user, messages: [], history_start: 0, has_more: false });
    }
    if (path === "/api/public/history") return json(route, { messages: [], history_start: 0, has_more: false });
    if (path === "/api/public/pushes") return json(route, { items: [], unread_count: 0, next_before: null });
    if (path === "/api/public/community") return json(route, { items: [], next_before: null, unread: false });
    if (path === "/api/public/research-overview") return json(route, { cards: [], report_today: "2026-09-06" });
    if (path === "/api/public/community/edge-session") return json(route, { enabled: false });
    if (path === "/api/public/events") return route.fulfill({ status: 204, body: "" });
    if (path === "/api/public/chat") {
      state.chatSends += 1;
      return json(route, { error: "no model in e2e" }, 503);
    }
    if (path === "/api/public/industry-map") {
      state.industryReads += 1;
      return json(route, {
        ...industryMap, is_admin: true,
        recent_edits: [], edit_count: 0, market_data_available: false,
      });
    }
    if (path === "/api/public/industry-map/edits") {
      state.edits += 1;
      return json(route, { error: "e2e never writes" }, 500);
    }
    return json(route, { error: `unmocked ${path}` }, 404);
  });
  return state;
}

async function top(page: Page, selector: string) {
  const box = await page.locator(selector).first().boundingBox();
  expect(box, selector).not.toBeNull();
  return box!.y;
}

test("an industry opens with the brief, then changes, companies, watch items and a folded dossier", async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const state = await installAdminMocks(page);
  await page.goto("/industry-map?industry=ai-chip");
  await expect(page.locator(".industry-detail > h2")).toHaveText("AI 芯片");
  await expect(page.getByText("研究底稿基线：")).toBeVisible();
  await expect(page.locator(".industry-detail-dates")).toContainText("内容截至 2026-09-02");

  const ys = await Promise.all(["#brief", "#changes", "#members", "#watch", "#dossier", "#sources"].map((id) => top(page, id)));
  expect([...ys].sort((a, b) => a - b)).toEqual(ys);
  // 底稿默认折起：方法论不可见。简报是管理员写的（底稿带 brief），所以不是自动归纳。
  await expect(page.locator("#method")).toBeHidden();
  await expect(page.locator("#brief .industry-brief-lead")).toContainText("CoWoS 与 HBM");
  await expect(page.locator("#brief")).not.toContainText("底稿自动归纳");
  await expect(page.locator("#brief")).toContainText("下一次先看");
  await expect(page.locator("#brief")).toContainText("截至 2026-09-02");
  // 最近变化里最新的一条是 2026-09-02 的 Broadcom 8-K，且带截至日。
  await expect(page.locator("#changes .industry-change").first()).toContainText("2026-09-02");
  await expect(page.locator("#changes .industry-change").first()).toContainText("Broadcom");
  // 关注点带数字截至日。
  await expect(page.locator("#watch")).toContainText("数字截至 2026-09-02");
  // 手机上不能横向溢出。
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  expect(overflow).toBeLessThanOrEqual(1);
  await page.screenshot({ path: testInfo.outputPath("industry-ai-chip-mobile.png"), fullPage: true });
  expect(state.edits).toBe(0);
});

test("search enters the company lens, the URL keeps it through reload, and exit clears it", async ({ page }, testInfo) => {
  const state = await installAdminMocks(page);
  await page.goto("/industry-map?industry=ai-chip");
  await page.getByRole("searchbox", { name: "找公司" }).fill("SNDK");
  await expect(page).toHaveURL(/industry=storage&symbol=SNDK$/);
  const card = page.getByRole("region", { name: "公司视角", exact: true });
  await expect(card).toBeVisible();
  await expect(card).toContainText("SNDK");
  await expect(card.getByRole("button", { name: /问 HONE：给 SNDK 做前瞻估值/ })).toBeVisible();
  await page.screenshot({ path: testInfo.outputPath("industry-lens-sndk.png"), fullPage: true });

  await page.reload();
  await expect(page).toHaveURL(/industry=storage&symbol=SNDK$/);
  await expect(page.getByRole("region", { name: "公司视角", exact: true })).toContainText("SNDK");

  await page.getByRole("button", { name: "退出公司视角", exact: true }).click();
  await expect(page).toHaveURL(/industry=storage$/);
  await expect(page.getByRole("region", { name: "公司视角", exact: true })).toHaveCount(0);

  // 公司表每行的「查看」进入视角；切行业自动退出。
  await page.getByRole("button", { name: "查看 MU", exact: true }).click();
  await expect(page).toHaveURL(/industry=storage&symbol=MU$/);
  await expect(page.getByRole("region", { name: "公司视角", exact: true })).toContainText("MU");
  await page.getByRole("navigation", { name: "行业树" }).getByRole("button", { name: /^光通信/ }).click();
  await expect(page).toHaveURL(/industry=optical$/);
  await expect(page.getByRole("region", { name: "公司视角", exact: true })).toHaveCount(0);
  expect(state.edits).toBe(0);
});

test("a symbol that is not a member of the industry is dropped from the URL", async ({ page }) => {
  await installAdminMocks(page);
  await page.goto("/industry-map?industry=optical&symbol=NOPE");
  await expect(page.locator(".industry-detail > h2")).toHaveText("光通信");
  await expect(page).toHaveURL(/industry=optical$/);
  await expect(page.getByRole("region", { name: "公司视角", exact: true })).toHaveCount(0);
});

test("問 HONE from a change hands the question to chat and sends it on arrival", async ({ page }) => {
  const state = await installAdminMocks(page);
  await page.goto("/industry-map?industry=ai-chip");
  await page.locator("#changes .industry-change").first().getByRole("button", { name: "问 HONE" }).click();
  await expect(page).toHaveURL(/\/chat(\?|$)/);
  // 到达即发送：chat 页把 q 清出 URL 并调用一次 /api/public/chat。
  await expect.poll(() => state.chatSends, { timeout: 15_000 }).toBeGreaterThan(0);
});

test("the administrator's editing mode unfolds the dossier and exposes the brief and signal editors", async ({ page }) => {
  const state = await installAdminMocks(page);
  await page.goto("/industry-map?industry=power");
  await expect(page.locator("#dossier")).not.toHaveAttribute("open", "");
  const editing = page.getByRole("switch", { name: "编辑本体", exact: true });
  await editing.click();
  await expect(editing).toHaveAttribute("aria-checked", "true");
  await expect(page.locator("#dossier")).toHaveAttribute("open", "");
  await expect(page.locator("#dossier textarea").first()).toBeVisible();
  await expect(page.getByRole("textbox", { name: "简报问题" })).toBeVisible();
  await expect(page.getByRole("textbox", { name: /最近动作$/ }).first()).toBeVisible();
  await expect(page.getByRole("button", { name: "移除此行业", exact: true })).toBeVisible();
  // 编辑态改不了就不会发请求。
  expect(state.edits).toBe(0);
});

test("the dossier table of contents unfolds and scrolls to the valuation card", async ({ page }) => {
  await installAdminMocks(page);
  await page.goto("/industry-map?industry=storage");
  await expect(page.locator("#valuation-card")).toBeHidden();
  await page.locator("#dossier > summary").click();
  await page.getByRole("navigation", { name: "底稿目录" }).getByRole("button", { name: "这类公司怎么估值" }).click();
  await expect(page.locator("#valuation-card")).toBeVisible();
  // 公司表里的子类型标签也会打开底稿并定位到执行卡。
  await page.reload();
  await expect(page.locator("#valuation-card")).toBeHidden();
  await page.locator(".industry-members .industry-subtype-tag").first().click();
  await expect(page.locator("#valuation-card")).toBeVisible();
});
