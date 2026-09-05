import { expect, test, type Page, type Route } from "@playwright/test";
import { readFileSync } from "node:fs";
import { DATA_CENTER_ZONES, industryHref } from "../src/lib/data-center-model";

const industryMap = JSON.parse(readFileSync(
  new URL("../../../skills/industry-map/references/industry-map.json", import.meta.url), "utf8",
)) as { industries: { id: string; name: string }[] } & Record<string, unknown>;

async function json(route: Route, payload: unknown, status = 200) {
  await route.fulfill({ status, contentType: "application/json", body: JSON.stringify(payload) });
}

async function installPublicMocks(
  page: Page,
  options: { admin?: boolean; authStatus?: number; industryStatus?: number; theme?: "dark" | "light" } = {},
) {
  const user = {
    user_id: "data-center-reader",
    created_at: "2026-09-01T00:00:00Z",
    daily_limit: 20,
    success_count: 0,
    in_flight: 0,
    remaining_today: 20,
    has_password: true,
    is_admin: options.admin ?? false,
    tos_accepted_at: "2026-09-01T00:00:00Z",
    tos_version: "2.1",
  };
  const state = { industryReads: 0, edits: 0 };
  await page.addInitScript(({ theme }) => {
    localStorage.clear();
    localStorage.setItem("hone-public-locale", "zh");
    if (theme) localStorage.setItem("hone.public.theme", theme);
  }, { theme: options.theme });
  await page.route("**/api/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === "/api/meta") {
      return json(route, {
        name: "hone", version: "test", channel: "web", apiVersion: "desktop-v1",
        capabilities: ["public_chat"], deploymentMode: "remote", supportsImessage: false,
      });
    }
    if (path === "/api/public/auth/me") {
      return options.authStatus === 401
        ? json(route, { error: "未登录" }, 401)
        : json(route, { user });
    }
    if (path === "/api/public/bootstrap") {
      return options.authStatus === 401
        ? json(route, { error: "未登录" }, 401)
        : json(route, { user, messages: [], history_start: 0, has_more: false });
    }
    if (path === "/api/public/history") return json(route, { messages: [], history_start: 0, has_more: false });
    if (path === "/api/public/pushes") return json(route, { items: [], unread_count: 0, next_before: null });
    if (path === "/api/public/community") return json(route, { items: [], next_before: null, unread: false });
    if (path === "/api/public/research-overview") return json(route, { cards: [], report_today: "2026-09-05" });
    if (path === "/api/public/community/edge-session") return json(route, { enabled: false });
    if (path === "/api/public/finance-calendar") {
      return json(route, { today: "2026-09-05", month: "2026-09", months: [], holdings: [], events: [], earnings_status: "ok", errors: [] });
    }
    if (path === "/api/public/events") return route.fulfill({ status: 204, body: "" });
    if (path === "/api/public/industry-map") {
      state.industryReads += 1;
      if (options.industryStatus === 401) return json(route, { error: "登录已过期" }, 401);
      return json(route, {
        ...industryMap, is_admin: user.is_admin,
        // A rolling frontend deploy must hide internal metadata even if an older
        // server still sends it to ordinary readers.
        recent_edits: [{
          at: "2026-09-05T09:00:00Z", by: "internal-admin",
          industry: "optical", industry_name: "光通信",
          summary: "更新研究说明", note: "internal-note",
        }], edit_count: 1, market_data_available: false,
      });
    }
    if (path === "/api/public/industry-map/edits") {
      state.edits += 1;
      return json(route, { error: "仅管理员可编辑" }, 403);
    }
    if (path === "/api/public/auth/dev-login/config") return json(route, { enabled: false });
    return json(route, {});
  });
  return state;
}

async function expectNoHorizontalOverflow(page: Page) {
  const dimensions = await page.evaluate(() => ({
    viewport: document.documentElement.clientWidth,
    document: document.documentElement.scrollWidth,
    body: document.body.scrollWidth,
  }));
  expect(dimensions.document).toBeLessThanOrEqual(dimensions.viewport + 1);
  expect(dimensions.body).toBeLessThanOrEqual(dimensions.viewport + 1);
}

async function expectReachableHotspots(page: Page) {
  const scene = page.locator(".dc-scene-wrap");
  await scene.scrollIntoViewIfNeeded();
  const bounds = await scene.boundingBox();
  expect(bounds).not.toBeNull();
  for (const zone of DATA_CENTER_ZONES) {
    const hotspot = page.locator(".dc-hotspot").filter({ hasText: zone.title });
    const box = await hotspot.boundingBox();
    expect(box, zone.title).not.toBeNull();
    expect(box!.width, `${zone.title} touch width`).toBeGreaterThanOrEqual(44);
    expect(box!.height, `${zone.title} touch height`).toBeGreaterThanOrEqual(44);
    // Fractional projection and borders can differ by a subpixel in Chromium.
    expect(box!.x, `${zone.title} left edge`).toBeGreaterThanOrEqual(bounds!.x - 1);
    expect(box!.x + box!.width, `${zone.title} right edge`).toBeLessThanOrEqual(bounds!.x + bounds!.width + 1);
    expect(box!.y, `${zone.title} top edge`).toBeGreaterThanOrEqual(bounds!.y - 1);
    expect(box!.y + box!.height, `${zone.title} bottom edge`).toBeLessThanOrEqual(bounds!.y + bounds!.height + 1);
    await hotspot.click({ trial: true });
  }
  await expectNoHorizontalOverflow(page);
}

async function geometryDigest(page: Page) {
  return page.locator(".dc-scene-svg polygon").evaluateAll((polygons) => {
    let hash = 2166136261;
    for (const polygon of polygons) {
      const properties = `${polygon.getAttribute("points")}|${polygon.getAttribute("fill")}`;
      for (let index = 0; index < properties.length; index += 1) {
        hash = Math.imul(hash ^ properties.charCodeAt(index), 16777619);
      }
    }
    return hash >>> 0;
  });
}

for (const viewport of [{ width: 390, height: 844 }, { width: 1440, height: 1000 }]) {
  test(`data center is reachable and all zones work at ${viewport.width}px`, async ({ page }, testInfo) => {
    await page.setViewportSize(viewport);
    const state = await installPublicMocks(page);
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));

    await page.goto("/chat");
    const entry = page.getByRole("link", { name: "3D 数据中心", exact: true });
    await expect(entry).toBeVisible();
    await entry.click();
    await expect(page).toHaveURL(/\/data-center$/);
    await expect(page.getByRole("heading", { name: "3D 数据中心", level: 1 })).toBeVisible();
    await expect(page.locator(".dc-scene-svg polygon").first()).toBeVisible();
    await expectNoHorizontalOverflow(page);
    await page.screenshot({ path: testInfo.outputPath(`data-center-${viewport.width}.png`), fullPage: true });

    for (const zone of DATA_CENTER_ZONES) {
      const hotspot = page.locator(".dc-hotspot").filter({ hasText: zone.title });
      await expect(hotspot).toHaveAccessibleName(`查看${zone.title}`);
      await hotspot.click();
      const dialog = page.getByRole("dialog", { name: zone.title, exact: true });
      await expect(dialog).toBeVisible();
      await expect(dialog).toContainText(zone.location);
      for (const industry of zone.industries) {
        await expect(dialog.locator(`a[href="${industryHref(industry.id)}"]`)).toBeVisible();
      }
      await expectNoHorizontalOverflow(page);
      if (zone.id === "optical") {
        await page.screenshot({ path: testInfo.outputPath(`data-center-dialog-${viewport.width}.png`), fullPage: true });
      }
      await page.keyboard.press("Escape");
      await expect(dialog).toBeHidden();
      await expect(hotspot).toBeFocused();

      const pill = page.locator(".dc-zone-card").filter({ hasText: zone.title });
      await pill.click();
      await expect(dialog).toBeVisible();
      await dialog.getByRole("button", { name: "关闭行业浮窗", exact: true }).click();
      await expect(dialog).toBeHidden();
      await expect(pill).toBeFocused();
    }

    const firstFace = page.locator(".dc-scene-svg polygon").first();
    const initialProjection = await firstFace.getAttribute("points");
    await page.getByRole("button", { name: "向左旋转", exact: true }).click();
    await expect(firstFace).not.toHaveAttribute("points", initialProjection!);
    await page.getByRole("button", { name: "向右旋转", exact: true }).click();
    await expect(firstFace).toHaveAttribute("points", initialProjection!);
    const zoom = page.getByLabel("模型缩放", { exact: true });
    const initialZoom = await zoom.innerText();
    await page.getByRole("button", { name: "放大模型", exact: true }).click();
    await expect(zoom).not.toHaveText(initialZoom);
    await expectNoHorizontalOverflow(page);
    await page.getByRole("button", { name: "缩小模型", exact: true }).click();
    await expect(zoom).toHaveText(initialZoom);
    await page.getByRole("button", { name: "向右旋转", exact: true }).click();
    await page.getByRole("button", { name: "复位", exact: true }).click();
    await expect(firstFace).toHaveAttribute("points", initialProjection!);
    expect(state.edits).toBe(0);
    expect(errors).toEqual([]);
  });
}

test.describe("narrow touch viewport", () => {
  test.use({ viewport: { width: 320, height: 740 }, isMobile: true, hasTouch: true });

test("320px and tablet keep touch targets reachable at camera and zoom limits", async ({ page }, testInfo) => {
  await installPublicMocks(page);
  await page.goto("/data-center");
  await expect(page.getByRole("heading", { name: "3D 数据中心", level: 1 })).toBeVisible();
  await expectReachableHotspots(page);
  for (const zone of DATA_CENTER_ZONES) {
    await page.locator(".dc-hotspot").filter({ hasText: zone.title }).tap();
    const dialog = page.getByRole("dialog", { name: zone.title, exact: true });
    await expect(dialog).toBeVisible();
    await expectNoHorizontalOverflow(page);
    await dialog.getByRole("button", { name: "关闭行业浮窗" }).click();
  }
  const adjustToLimit = async (name: string) => {
    const control = page.getByRole("button", { name, exact: true });
    for (let step = 0; step < 10 && await control.isEnabled(); step += 1) await control.click();
    await expect(control).toBeDisabled();
  };
  for (const rotation of ["向左旋转", "向右旋转"]) {
    await adjustToLimit(rotation);
    for (const [control, value] of [["缩小模型", "80%"], ["放大模型", "135%"]]) {
      await adjustToLimit(control);
      await expect(page.getByLabel("模型缩放", { exact: true })).toHaveText(value);
      await expectReachableHotspots(page);
    }
  }
  await page.screenshot({ path: testInfo.outputPath("data-center-320-camera-limit.png"), fullPage: true });
  await page.getByRole("button", { name: "复位", exact: true }).click();
  await expect(page.getByLabel("模型缩放", { exact: true })).toHaveText("100%");
  await page.screenshot({ path: testInfo.outputPath("data-center-320.png"), fullPage: true });

  // The visible desktop sidebar leaves a narrow scene at this tablet width.
  await page.setViewportSize({ width: 900, height: 1000 });
  await expectReachableHotspots(page);
  await page.screenshot({ path: testInfo.outputPath("data-center-tablet-900.png"), fullPage: true });
  await adjustToLimit("缩小模型");
  await expect(page.getByLabel("模型缩放", { exact: true })).toHaveText("80%");
  await expectReachableHotspots(page);
});
});

test("drag rotates the geometry while dark and reduced-motion preferences remain usable", async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.emulateMedia({ colorScheme: "dark", reducedMotion: "reduce" });
  await installPublicMocks(page, { theme: "dark" });
  await page.goto("/data-center");
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  const scene = page.locator(".dc-scene-svg");
  await scene.scrollIntoViewIfNeeded();
  const firstFace = scene.locator("polygon").first();
  const initialProjection = await firstFace.getAttribute("points");
  const initialGeometry = await geometryDigest(page);
  const bounds = await scene.boundingBox();
  expect(bounds).not.toBeNull();
  // Use open floor below the software layer, away from the overlaid buttons.
  await page.mouse.move(bounds!.x + bounds!.width * 0.48, bounds!.y + bounds!.height * 0.42);
  await page.mouse.down();
  await page.mouse.move(bounds!.x + bounds!.width * 0.62, bounds!.y + bounds!.height * 0.42, { steps: 8 });
  await expect(firstFace).not.toHaveAttribute("points", initialProjection!);
  await page.mouse.up();
  await expect(page.locator(".dc-scene")).not.toHaveClass(/is-dragging/);
  expect(await scene.evaluate((element) => getComputedStyle(element).touchAction.split(" "))).toContain("pan-y");
  await page.getByRole("button", { name: "复位", exact: true }).click();
  await expect(firstFace).toHaveAttribute("points", initialProjection!);
  await expect.poll(() => geometryDigest(page)).toBe(initialGeometry);
  await page.screenshot({ path: testInfo.outputPath("data-center-dark.png"), fullPage: true });

  const optical = page.locator(".dc-hotspot").filter({ hasText: "光通信与互联" });
  await optical.click();
  const activeFlow = page.locator(".dc-flow.is-active");
  await expect(activeFlow).toHaveCount(1);
  expect(await activeFlow.evaluate((element) => getComputedStyle(element).animationName)).toBe("none");
  const transitionDurations = await optical.evaluate((element) =>
    getComputedStyle(element).transitionDuration.split(",").map((duration) => Number.parseFloat(duration)),
  );
  // The shared foundation may use 0.01ms instead of removing transitions.
  expect(Math.max(...transitionDurations)).toBeLessThanOrEqual(0.001);
  await expect(page.getByRole("dialog", { name: "光通信与互联", exact: true })).toBeVisible();
  await expectNoHorizontalOverflow(page);
  await page.screenshot({ path: testInfo.outputPath("data-center-dark-dialog.png"), fullPage: true });
  await page.keyboard.press("Escape");
  await expect(optical).toBeFocused();
});

test("ordinary readers follow a zone into its industry and retain selection through browser history", async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const state = await installPublicMocks(page);
  await page.goto("/data-center");
  await page.locator(".dc-hotspot").filter({ hasText: "光通信与互联" }).click();
  await page.getByRole("dialog", { name: "光通信与互联", exact: true })
    .locator('a[href="/industry-map?industry=optical"]').click();
  await expect(page).toHaveURL(/\/industry-map\?industry=optical$/);
  const selectedHeading = page.locator(".industry-detail > h2");
  await expect(selectedHeading).toHaveText("光通信");
  await expect(page.getByRole("switch", { name: "编辑本体" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "移除此行业" })).toHaveCount(0);
  await expect(page.locator(".industry-detail textarea")).toHaveCount(0);
  await expect(page.getByText("internal-note", { exact: true })).toHaveCount(0);
  await expect(page.getByText("internal-admin", { exact: true })).toHaveCount(0);
  await page.screenshot({ path: testInfo.outputPath("industry-optical-reader-mobile.png"), fullPage: true });

  await page.getByRole("navigation", { name: "行业树" }).getByRole("button", { name: /^存储/ }).click();
  await expect(page).toHaveURL(/industry=storage$/);
  await expect(selectedHeading).toHaveText("存储");
  await page.reload();
  await expect(selectedHeading).toHaveText("存储");
  await page.goBack();
  await expect(page).toHaveURL(/industry=optical$/);
  await expect(selectedHeading).toHaveText("光通信");
  await page.goForward();
  await expect(selectedHeading).toHaveText("存储");
  expect(state.edits).toBe(0);

  await page.goto("/industry-map?industry=missing-industry");
  await expect(selectedHeading).toHaveText(industryMap.industries[0].name);
  await expect(page).toHaveURL(new RegExp(`industry=${industryMap.industries[0].id}$`));

  await page.goto("/research?group=industry");
  const industryEntries = page.getByRole("region", { name: "产业研究入口", exact: true });
  await expect(industryEntries.getByRole("button", { name: /3D 数据中心/ })).toBeVisible();
  await expect(industryEntries.getByRole("button", { name: /行业分析/ })).toBeVisible();
  await industryEntries.getByRole("button", { name: /3D 数据中心/ }).click();
  await expect(page).toHaveURL(/\/data-center$/);
  await expect(page.getByRole("heading", { name: "3D 数据中心", level: 1 })).toBeVisible();
});

test("the existing administrator editor remains available", async ({ page }) => {
  await installPublicMocks(page, { admin: true });
  await page.goto("/industry-map?industry=power");
  await expect(page.locator(".industry-detail > h2")).toHaveText("电力");
  const editing = page.getByRole("switch", { name: "编辑本体", exact: true });
  await expect(editing).toBeVisible();
  await expect(page.getByText("internal-note", { exact: true })).toBeVisible();
  await expect(page.getByText("internal-admin", { exact: true })).toBeVisible();
  await editing.click();
  await expect(editing).toHaveAttribute("aria-checked", "true");
  await expect(page.getByRole("button", { name: "移除此行业", exact: true })).toBeVisible();
  await expect(page.getByPlaceholder("为什么改（必填，展示给其它管理员）")).toBeVisible();
});

for (const expiredAt of ["auth", "industry"] as const) {
  test(`industry detail requires login when ${expiredAt} returns 401`, async ({ page }) => {
    const state = await installPublicMocks(page, expiredAt === "auth"
      ? { authStatus: 401 }
      : { industryStatus: 401 });
    await page.goto("/industry-map?industry=optical");
    await expect(page.getByRole("heading", { name: "登录后查看行业分析", exact: true })).toBeVisible();
    await expect(page.locator(".industry-detail")).toHaveCount(0);
    await expect(page.getByRole("switch", { name: "编辑本体" })).toHaveCount(0);
    await expect(page).toHaveURL(/industry=optical$/);
    expect(state.industryReads).toBe(expiredAt === "auth" ? 0 : 1);
    expect(state.edits).toBe(0);
  });
}
