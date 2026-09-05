import { expect, test, type Page, type Route } from "@playwright/test";
import { fileURLToPath } from "node:url";

const pdfFixture = fileURLToPath(new URL("./fixtures/sample-report.pdf", import.meta.url));
const item = (content_id: number) => ({
  content_id,
  author_name: "HONE",
  published_at: "2026-09-05T12:00:00+08:00",
  content_type: "text",
  body_text: `社区动态 ${content_id}`,
  resources: [],
});

async function json(route: Route, payload: unknown, status = 200) {
  await route.fulfill({ status, contentType: "application/json", body: JSON.stringify(payload) });
}

async function installShell(page: Page) {
  await page.addInitScript(() => {
    localStorage.clear();
    localStorage.setItem("hone-public-locale", "zh");
  });
  await page.route("**/api/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === "/api/public/community/seen") return json(route, { ok: true });
    if (path === "/api/public/pushes") return json(route, { items: [], unread_count: 0 });
    if (path === "/api/public/bootstrap") return json(route, { messages: [], history_start: 0 });
    if (path === "/api/meta") {
      return json(route, { name: "hone", version: "test", capabilities: ["public_chat"], deploymentMode: "remote" });
    }
    await route.fallback();
  });
}

test("official feed refreshes in place, retains older pages and reports failed refreshes", async ({ page }) => {
  await installShell(page);
  await page.clock.install();
  let latest = [item(42), item(41)];
  let failed = false;
  await page.route("**/api/public/community", async (route) => {
    if (failed) return json(route, { error: "refresh failed" }, 400);
    return json(route, { items: latest, next_before: latest.at(-1)?.content_id, unread: true });
  });
  await page.route("**/api/public/community?before=41", (route) => json(route, {
    items: [item(40)], next_before: null, unread: false,
  }));

  await page.goto("/community");
  const cards = page.locator(".public-community-card");
  await expect(cards).toHaveCount(2);
  await page.getByRole("button", { name: "加载更早动态", exact: true }).click();
  await expect(cards).toHaveCount(3);

  latest = [item(43), item(42)];
  await page.getByRole("button", { name: "刷新动态", exact: true }).click();
  await expect(cards).toHaveCount(4);
  await expect(cards.first()).toContainText("社区动态 43");
  await expect(cards.last()).toContainText("社区动态 40");

  latest = [item(44), item(43)];
  await page.clock.fastForward(60_000);
  await expect(cards.first()).toContainText("社区动态 44");
  await expect(cards).toHaveCount(5);

  failed = true;
  await page.getByRole("button", { name: "刷新动态", exact: true }).click();
  await expect(page.getByRole("alert")).toHaveText("暂未获取到最新动态，请重试刷新。");
  await expect(cards).toHaveCount(5);
});

test("stored images open and a prepared PDF downloads without a second resource transfer", async ({ page }) => {
  await installShell(page);
  let pdfRequests = 0;
  await page.route("**/api/public/community", (route) => json(route, {
    items: [{
      ...item(42),
      resources: [
        { resource_id: 1, ordinal: 0, resource_kind: "image", display_name: "社区图片", content_type: "image/png", access_state: "stored" },
        { resource_id: 2, ordinal: 1, resource_kind: "file", display_name: "社区报告.pdf", content_type: "application/pdf", access_state: "stored" },
      ],
    }], next_before: null, unread: true,
  }));
  await page.route("**/api/public/community/resources/1", (route) => route.fulfill({
    contentType: "image/png",
    body: Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aZ3kAAAAASUVORK5CYII=", "base64"),
  }));
  await page.route("**/api/public/community/resources/2", async (route) => {
    pdfRequests += 1;
    await route.fulfill({ contentType: "application/pdf", path: pdfFixture });
  });

  await page.goto("/community");
  await expect(page.locator(".public-community-image img")).toBeVisible();
  await expect.poll(() => page.locator(".public-community-image img").evaluate((image: HTMLImageElement) => image.naturalWidth)).toBe(1);
  await page.getByRole("button", { name: "预览社区图片", exact: true }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.getByRole("button", { name: "关闭预览", exact: true }).click();

  await page.getByRole("button", { name: "社区报告.pdf" }).click();
  const preview = page.getByRole("dialog");
  await expect(preview.locator("iframe")).toHaveAttribute("src", /^blob:/);
  const downloaded = page.waitForEvent("download");
  await preview.getByRole("button", { name: "下载资源", exact: true }).click();
  expect((await downloaded).suggestedFilename()).toBe("社区报告.pdf");
  expect(pdfRequests).toBe(1);
});
