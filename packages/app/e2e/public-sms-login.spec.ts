import { expect, test, type Page, type Route } from "@playwright/test";

const PHONE = "13800138000";
const CODE = "123456";

type ServerState = {
  loggedIn: boolean;
  sendCalls: number;
  loginCalls: number;
  rememberLong: boolean;
  lastSendBody: string;
  lastLoginBody: string;
};

async function fulfillJson(route: Route, payload: unknown, status = 200) {
  await route.fulfill({
    status,
    contentType: "application/json",
    body: JSON.stringify(payload),
  });
}

function buildUser() {
  return {
    user_id: "test-user",
    created_at: "2026-04-20T00:00:00Z",
    last_login_at: "2026-04-25T08:00:00Z",
    daily_limit: 20,
    success_count: 3,
    in_flight: 0,
    remaining_today: 17,
    has_password: false,
    tos_accepted_at: "2026-05-12T08:00:00Z",
    tos_version: "2.1",
  };
}

async function installPublicAuthMocks(
  page: Page,
  initial: Partial<ServerState>,
) {
  const state: ServerState = {
    loggedIn: false,
    sendCalls: 0,
    loginCalls: 0,
    rememberLong: false,
    lastSendBody: "",
    lastLoginBody: "",
    ...initial,
  };

  await page.addInitScript(() => {
    localStorage.clear();
    localStorage.setItem("hone-public-locale", "zh");
  });

  await page.route("**/api/**", async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    const path = url.pathname;

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
      if (!state.loggedIn) {
        await fulfillJson(route, { error: "未登录" }, 401);
        return;
      }
      await fulfillJson(route, { user: buildUser() });
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

    if (path === "/api/public/auth/captcha/config") {
      await fulfillJson(route, {
        enabled: false,
        region: "cn",
        prefix: "",
        scene_id: "",
        script_url:
          "https://o.alicdn.com/captcha-frontend/aliyunCaptcha/AliyunCaptcha.js",
      });
      return;
    }

    if (path === "/api/public/auth/sms/send") {
      state.sendCalls += 1;
      state.lastSendBody = request.postData() ?? "";
      const body = JSON.parse(state.lastSendBody || "{}") as {
        phone_number?: string;
      };
      if (body.phone_number !== PHONE) {
        await fulfillJson(
          route,
          { error: "目前是邀请制，请联系 bm@hone-claw.com 加入白名单" },
          403,
        );
        return;
      }
      await fulfillJson(route, { ok: true });
      return;
    }

    if (path === "/api/public/auth/sms/login") {
      state.loginCalls += 1;
      state.lastLoginBody = request.postData() ?? "";
      const body = JSON.parse(state.lastLoginBody || "{}") as {
        phone_number?: string;
        verify_code?: string;
        remember?: boolean;
        tos_version?: string;
      };
      if (body.phone_number !== PHONE || body.verify_code !== CODE) {
        await fulfillJson(route, { error: "验证码不正确或已过期" }, 401);
        return;
      }
      if (body.tos_version !== "2.1") {
        await fulfillJson(route, { error: "需同意用户协议与隐私政策" }, 400);
        return;
      }
      state.loggedIn = true;
      state.rememberLong = !!body.remember;
      await fulfillJson(route, { user: buildUser() });
      return;
    }

    if (path === "/api/public/auth/logout") {
      state.loggedIn = false;
      await fulfillJson(route, { ok: true });
      return;
    }

    await route.fallback();
  });

  return state;
}

test("SMS login sends code, requires ToS, and signs in", async ({ page }) => {
  const mock = await installPublicAuthMocks(page, { loggedIn: false });

  await page.goto("/me");
  await expect(
    page.getByText("目前是邀请制，请联系 bm@hone-claw.com 加入白名单。"),
  ).toBeVisible();

  await page.getByLabel("手机号").fill(PHONE);
  await page.getByRole("button", { name: "获取验证码" }).click();

  await expect.poll(() => mock.sendCalls).toBe(1);
  expect(mock.lastSendBody).toContain(`"phone_number":"${PHONE}"`);
  await expect(page.getByText("验证码已发送，请查看短信。")).toBeVisible();

  await page.getByLabel("短信验证码").fill(CODE);
  const submit = page.getByRole("button", { name: "登录", exact: true });
  await expect(submit).toBeDisabled();

  await page.getByRole("checkbox").nth(1).click();
  await expect(submit).toBeEnabled();
  await submit.click();

  await expect.poll(() => mock.loginCalls).toBe(1);
  expect(mock.lastLoginBody).toContain('"remember":true');
  expect(mock.lastLoginBody).toContain('"tos_version":"2.1"');
  await expect(page.getByText("账号信息")).toBeVisible();
});

test("clicking ToS / Privacy links inside checkbox row does not toggle the box", async ({
  page,
}) => {
  await installPublicAuthMocks(page, { loggedIn: false });

  await page.goto("/me");

  const tosBox = page.getByRole("checkbox").nth(1);
  await expect(tosBox).toHaveAttribute("aria-checked", "false");

  const termsLink = page.getByRole("link", { name: "用户协议" });
  const privacyLink = page.getByRole("link", { name: "隐私政策" });
  await expect(termsLink).toHaveAttribute("target", "_blank");
  await expect(termsLink).toHaveAttribute("href", "/terms");
  await expect(privacyLink).toHaveAttribute("target", "_blank");
  await expect(privacyLink).toHaveAttribute("href", "/privacy");

  await termsLink.click({ modifiers: ["Meta"] });
  await expect(tosBox).toHaveAttribute("aria-checked", "false");

  await privacyLink.click({ modifiers: ["Meta"] });
  await expect(tosBox).toHaveAttribute("aria-checked", "false");

  await tosBox.click();
  await expect(tosBox).toHaveAttribute("aria-checked", "true");
});
