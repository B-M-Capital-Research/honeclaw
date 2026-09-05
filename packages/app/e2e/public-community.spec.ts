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

test("unarchived attachments explain availability, copy their name and link the source group", async ({ page, context }) => {
  await installShell(page);
  await page.addInitScript(() => {
    Object.defineProperty(navigator.clipboard, "writeText", { configurable: true, value: async (text: string) => {
      (window as Window & { copiedFilename?: string }).copiedFilename = text;
    } });
  });
  let archived = false;
  let resourceRequests = 0;
  const sourceGroup = "https://wx.zsxq.com/group/51115212285814";
  await context.route(sourceGroup, (route) => route.fulfill({ contentType: "text/html", body: "<h1>Source group test page</h1>" }));
  await page.route("**/api/public/community", (route) => json(route, {
    items: [{ ...item(42), resources: [
      { resource_id: 2, ordinal: 0, resource_kind: "file", display_name: "ASML研究.pdf", content_type: "application/pdf", access_state: archived ? "stored" : "metadata_only" },
      { resource_id: 3, ordinal: 1, resource_kind: "file", display_name: "历史附件.pdf", content_type: "application/pdf", access_state: "protected_in_app" },
      { resource_id: 4, ordinal: 2, resource_kind: "image", display_name: "未知状态图片.png", content_type: "image/png", access_state: "unknown_future_state" },
    ] }], next_before: null, unread: false,
  }));
  await page.route("**/api/public/community/resources/*", (route) => {
    resourceRequests++;
    return route.fulfill({ contentType: "application/pdf", path: pdfFixture });
  });
  await page.goto("/community");
  await page.setViewportSize({ width: 390, height: 844 });
  const unarchived = page.getByRole("button", { name: "ASML研究.pdf 附件尚未归档", exact: true });
  await expect(unarchived).toBeEnabled();
  await unarchived.click();
  let dialog = page.getByRole("dialog", { name: "附件尚未归档", exact: true });
  await expect(dialog).toContainText("目前仅收录了附件名称");
  await expect(dialog).not.toContainText("来源保护");
  await expect(dialog).not.toContainText("同步中");
  await expect(dialog.getByRole("button", { name: "下载资源", exact: true })).toHaveCount(0);
  await dialog.getByRole("button", { name: "复制文件名", exact: true }).click();
  await expect(dialog.getByRole("status")).toHaveText("文件名已复制");
  expect(await page.evaluate(() => (window as Window & { copiedFilename?: string }).copiedFilename)).toBe("ASML研究.pdf");
  const link = dialog.getByRole("link", { name: "打开知识星球群组", exact: true });
  await expect(link).toHaveAttribute("href", sourceGroup);
  await expect(link).toHaveAttribute("rel", "noopener noreferrer");
  const popupReady = page.waitForEvent("popup");
  await link.click();
  const popup = await popupReady;
  await expect(popup).toHaveURL(sourceGroup);
  await expect(popup.getByRole("heading")).toHaveText("Source group test page");
  await popup.close();
  await page.screenshot({ path: test.info().outputPath("community-unarchived-mobile.png") });
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
  await expect(unarchived).toBeFocused();

  await page.getByRole("button", { name: "历史附件.pdf 原采集时记录访问受限", exact: true }).click();
  dialog = page.getByRole("dialog", { name: "原采集时记录访问受限", exact: true });
  await expect(dialog).toContainText("当前来源可用性尚未确认");
  await page.evaluate(() => Object.defineProperty(navigator.clipboard, "writeText", { configurable: true, value: async () => { throw new Error("Clipboard unavailable"); } }));
  await dialog.getByRole("button", { name: "复制文件名", exact: true }).click();
  await expect(dialog.getByRole("alert")).toHaveText("复制失败，请选中文件名后复制。");
  await expect(dialog.getByRole("textbox", { name: "文件名", exact: true })).toHaveValue("历史附件.pdf");
  await dialog.getByRole("button", { name: "关闭附件说明", exact: true }).click();
  await page.getByRole("button", { name: "查看附件说明：未知状态图片.png", exact: true }).click();
  dialog = page.getByRole("dialog", { name: "附件暂不可用", exact: true });
  await expect(dialog).toBeVisible();
  await expect(dialog).not.toContainText("访问受限");
  await dialog.getByRole("button", { name: "关闭附件说明", exact: true }).click();
  expect(resourceRequests).toBe(0);

  archived = true;
  await page.getByRole("button", { name: "刷新动态", exact: true }).click();
  await page.getByRole("button", { name: "ASML研究.pdf 点击预览", exact: true }).click();
  await expect(page.getByTestId("community-pdf-canvas")).toBeVisible();
  const downloaded = page.waitForEvent("download");
  await page.getByRole("button", { name: "下载资源", exact: true }).click();
  expect((await downloaded).suggestedFilename()).toBe("ASML研究.pdf");
  expect(resourceRequests).toBe(1);
});
