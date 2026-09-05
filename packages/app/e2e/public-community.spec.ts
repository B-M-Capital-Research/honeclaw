import { expect, test, type Page, type Route, type Worker } from "@playwright/test";
import { fileURLToPath } from "node:url";

const pdfFixture = fileURLToPath(new URL("./fixtures/community-report-zh.pdf", import.meta.url));
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

test("stored images open and Chinese PDFs actually render, paginate and reuse their download", async ({ page }) => {
  await installShell(page);
  let pdfRequests = 0;
  const workers = new Set<Worker>();
  page.on("worker", (worker) => {
    workers.add(worker);
    worker.on("close", () => workers.delete(worker));
  });
  let releasePdf: () => void = () => {};
  const pdfResponse = new Promise<void>((resolve) => { releasePdf = resolve; });
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
    await pdfResponse;
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
  await expect.poll(() => pdfRequests).toBe(1);
  // Clicking download before the preview response completes must join that GET.
  const downloaded = page.waitForEvent("download");
  await preview.getByRole("button", { name: "下载资源", exact: true }).click();
  await expect(preview.getByRole("button", { name: "正在下载…", exact: true })).toBeDisabled();
  releasePdf();
  expect((await downloaded).suggestedFilename()).toBe("社区报告.pdf");
  expect(pdfRequests).toBe(1);

  const canvas = preview.getByTestId("community-pdf-canvas");
  const ink = () => canvas.evaluate((element: HTMLCanvasElement) => {
    const { data } = element.getContext("2d")!.getImageData(0, 0, element.width, element.height);
    let dark = 0, blue = 0, orange = 0;
    for (let i = 0; i < data.length; i += 4) {
      if (data[i + 3]! < 200) continue;
      if (Math.max(data[i]!, data[i + 1]!, data[i + 2]!) < 120) dark++;
      if (data[i + 2]! > data[i]! + 50) blue++;
      if (data[i]! > data[i + 2]! + 50) orange++;
    }
    return { dark, blue, orange, pixels: element.width * element.height };
  });
  await expect(canvas).toBeVisible();
  await expect(preview.getByTestId("pdf-page-number")).toHaveText("第 1 页 / 共 2 页");
  await expect(preview.locator(".public-community-pdf-text")).toContainText("研究数据与图表正常显示");
  const first = await ink();
  expect(first.dark).toBeGreaterThan(100);
  expect(first.blue).toBeGreaterThan(10_000);
  expect(first.pixels).toBeLessThanOrEqual(8 * 1024 * 1024);
  await expect(preview.getByRole("button", { name: "上一页", exact: true })).toBeDisabled();

  await preview.getByRole("button", { name: "下一页", exact: true }).click();
  await expect(preview.getByTestId("pdf-page-number")).toHaveText("第 2 页 / 共 2 页");
  await expect(canvas).toBeVisible();
  await expect(preview.locator(".public-community-pdf-text")).toContainText("展示不同的内容");
  const second = await ink();
  expect(second.dark).toBeGreaterThan(100);
  expect(second.orange).toBeGreaterThan(10_000);
  expect(second.blue).toBeLessThan(first.blue / 10);
  await expect(preview.getByRole("button", { name: "下一页", exact: true })).toBeDisabled();
  await preview.getByRole("button", { name: "放大", exact: true }).click();
  await expect(canvas).toBeVisible();
  await expect.poll(async () => (await ink()).pixels).toBeGreaterThan(second.pixels);
  await preview.getByRole("button", { name: "适应屏幕", exact: true }).click();
  await expect.poll(async () => (await ink()).pixels).toBe(second.pixels);
  await expect.poll(() => workers.size).toBe(1);
  expect(pdfRequests).toBe(1);
  await page.screenshot({ path: test.info().outputPath("community-pdf-rendered.png") });
  await page.getByRole("button", { name: "关闭预览", exact: true }).click();
  await expect.poll(() => workers.size).toBe(0);
  await expect(canvas).toHaveCount(0);
});

test("invalid stored PDF shows a readable error and still downloads the original bytes", async ({ page }) => {
  await installShell(page);
  let requests = 0;
  await page.route("**/api/public/community", (route) => json(route, {
    items: [{ ...item(42), resources: [{ resource_id: 2, ordinal: 0, resource_kind: "file", display_name: "损坏报告.pdf", content_type: "application/pdf", access_state: "stored" }] }],
    next_before: null, unread: true,
  }));
  await page.route("**/api/public/community/resources/2", (route) => {
    requests++;
    return route.fulfill({ contentType: "application/pdf", body: "%PDF-1.4\ninvalid" });
  });
  await page.goto("/community");
  await page.getByRole("button", { name: "损坏报告.pdf" }).click();
  await expect(page.getByRole("dialog").getByRole("alert")).toContainText("暂时无法预览这份 PDF");
  const downloaded = page.waitForEvent("download");
  await page.getByRole("button", { name: "下载资源", exact: true }).click();
  expect((await downloaded).suggestedFilename()).toBe("损坏报告.pdf");
  expect(requests).toBe(1);
});

test("closing a pending PDF cancels its read and a fresh preview still renders", async ({ page }) => {
  await installShell(page);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  let requests = 0;
  let release: () => void = () => {};
  const responseReady = new Promise<void>((resolve) => { release = resolve; });
  await page.route("**/api/public/community", (route) => json(route, {
    items: [{ ...item(42), resources: [{ resource_id: 2, ordinal: 0, resource_kind: "file", display_name: "社区报告.pdf", content_type: "application/pdf", access_state: "stored" }] }],
    next_before: null, unread: true,
  }));
  await page.route("**/api/public/community/resources/2", async (route) => {
    requests++;
    await responseReady;
    await route.fulfill({ contentType: "application/pdf", path: pdfFixture });
  });
  await page.goto("/community");
  await page.getByRole("button", { name: "社区报告.pdf" }).click();
  await expect.poll(() => requests).toBe(1);
  const aborted = page.waitForEvent("requestfailed", (request) => request.url().endsWith("/community/resources/2"));
  await page.getByRole("button", { name: "关闭预览", exact: true }).click();
  await aborted;
  release();
  await expect(page.getByRole("dialog")).toHaveCount(0);

  // Reopen on a narrow screen after the old async task has been cancelled.
  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByRole("button", { name: "社区报告.pdf" }).click();
  const preview = page.getByRole("dialog");
  await expect(preview.getByTestId("community-pdf-canvas")).toBeVisible();
  await expect(preview.getByTestId("pdf-page-number")).toHaveText("第 1 页 / 共 2 页");
  await page.screenshot({ path: test.info().outputPath("community-pdf-mobile.png") });
  // Close during the next render, without waiting for its promise to settle.
  await preview.getByRole("button", { name: "下一页", exact: true }).click();
  await preview.getByRole("button", { name: "关闭预览", exact: true }).click();
  await expect(preview).toHaveCount(0);
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  expect(requests).toBe(2);
  expect(errors).toEqual([]);
});
