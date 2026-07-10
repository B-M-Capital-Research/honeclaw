import { expect, test, type Route } from "@playwright/test";

async function fulfillJson(route: Route, payload: unknown, status = 200) {
  await route.fulfill({
    status,
    contentType: "application/json",
    body: JSON.stringify(payload),
  });
}

test.use({ viewport: { width: 390, height: 844 } });

test("mobile push center clears unread state and calendar opens a zoomable preview", async ({
  page,
}) => {
  const pushes = [
    {
      push_id: "latest",
      job_id: "daily",
      title: "移动端每日简报",
      summary: "验证红点、紧凑卡片和已读状态的最新摘要。",
      created_at: "2026-07-10T20:00:00+08:00",
    },
    {
      push_id: "older",
      job_id: "daily",
      title: "移动端昨日简报",
      summary: "验证列表间距和层级不会折叠。",
      created_at: "2026-07-09T20:00:00+08:00",
    },
  ];
  let unreadCount = 2;

  await page.addInitScript(() => {
    localStorage.clear();
    localStorage.setItem("hone-public-locale", "zh");
  });
  await page.route("**/api/**", async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;
    if (path === "/api/meta") {
      await fulfillJson(route, {
        name: "hone",
        version: "test",
        channel: "web",
        supportsImessage: false,
        apiVersion: "desktop-v1",
        capabilities: ["public_chat"],
        deploymentMode: "remote",
      });
      return;
    }
    if (path === "/api/public/auth/me") {
      await fulfillJson(route, {
        user: {
          user_id: "mobile-test-user",
          created_at: "2026-07-01T08:00:00+08:00",
          last_login_at: "2026-07-10T08:00:00+08:00",
          daily_limit: 20,
          success_count: 0,
          in_flight: 0,
          remaining_today: 20,
          has_password: false,
          tos_accepted_at: "2026-07-01T08:00:00+08:00",
          tos_version: "2.1",
        },
      });
      return;
    }
    if (path === "/api/public/history") {
      await fulfillJson(route, { messages: [] });
      return;
    }
    if (path === "/api/public/events") {
      await route.fulfill({ status: 204, body: "" });
      return;
    }
    if (path === "/api/public/pushes" && request.method() === "GET") {
      await fulfillJson(route, {
        items: pushes,
        unread_count: unreadCount,
        next_before: null,
      });
      return;
    }
    if (path.endsWith("/open") && request.method() === "POST") {
      const push = pushes.find((item) => path.includes(item.push_id)) ?? pushes[0];
      unreadCount = 0;
      await fulfillJson(route, {
        push: {
          ...push,
          content: "# 移动端每日简报\n\n这是完整通知内容。",
        },
        unread_count: unreadCount,
      });
      return;
    }
    if (path === "/api/public/finance-calendar") {
      await fulfillJson(route, {
        today: "2026-07-10",
        month: "2026-07",
        months: [{ value: "2026-07", label: "2026年7月" }],
        holdings: ["NVDA"],
        events: [
          {
            date: "2026-07-14",
            title: "美国 CPI",
            kind: "macro",
            source: "bls.gov",
          },
          {
            date: "2026-07-30",
            title: "NVDA 财报",
            kind: "earnings",
            ticker: "NVDA",
            source: "fmp",
          },
        ],
        earnings_status: "ok",
        errors: [],
      });
      return;
    }
    await route.fallback();
  });

  await page.goto("/chat");
  const mobileControls = page.locator(".pub-nav-mobile-controls");
  const pushButton = mobileControls.getByRole("button", {
    name: "打开推送中心",
  });
  await expect(pushButton).toBeVisible();
  await expect(mobileControls.locator(".public-push-unread-dot")).toBeVisible();

  await pushButton.click();
  await expect(mobileControls.locator(".public-push-unread-dot")).toHaveCount(0);
  const pushCenter = page.getByRole("dialog", { name: "推送列表" });
  await expect(pushCenter).toBeVisible();
  await expect
    .poll(async () => Math.round((await pushCenter.boundingBox())?.x ?? -1))
    .toBe(0);
  const centerBox = await pushCenter.boundingBox();
  expect(centerBox?.x).toBe(0);
  expect(centerBox?.width).toBe(390);
  expect(centerBox?.height).toBe(844);
  const pushCards = pushCenter.locator(".public-push-list-item");
  await expect(pushCards).toHaveCount(2);
  expect((await pushCards.first().boundingBox())?.height).toBeLessThan(100);

  await pushCards.first().click();
  const pushDetail = page.getByRole("dialog", { name: "推送完整内容" });
  await expect(pushDetail).toBeVisible();
  await expect
    .poll(async () => Math.round((await pushDetail.boundingBox())?.width ?? -1))
    .toBe(390);
  const detailBox = await pushDetail.boundingBox();
  expect(detailBox?.width).toBe(390);
  expect(detailBox?.height).toBeLessThan(744);
  await pushDetail.getByRole("button", { name: "关闭完整内容" }).click();
  await pushCenter.getByRole("button", { name: "关闭推送中心" }).click();

  await page.getByRole("button", { name: "我的财经日历" }).click();
  const calendarPreview = page.getByRole("button", { name: "查看日历大图" });
  await expect(calendarPreview).toBeVisible();
  expect((await calendarPreview.boundingBox())?.width).toBeLessThanOrEqual(310);
  await calendarPreview.click();

  const largePreview = page.getByRole("dialog", {
    name: "财经日历图片预览",
  });
  await expect(largePreview).toBeVisible();
  await expect(largePreview.getByRole("button", { name: "放大" })).toBeVisible();
  const initialScale = await largePreview
    .locator(".public-chat-calendar-zoom-controls > span")
    .innerText();
  await largePreview.getByRole("button", { name: "放大" }).click();
  await expect(
    largePreview.locator(".public-chat-calendar-zoom-controls > span"),
  ).not.toHaveText(initialScale);
});
