import { expect, test, type Page, type Route } from "@playwright/test"

const PHONE = "13800138000"
const STORED_PASSWORD = "secret123"

type ServerState = {
  hasPassword: boolean
  storedPassword: string | null
  loggedIn: boolean
  rememberLong: boolean
  setPasswordCalls: number
  passwordLoginCalls: number
  changeCalls: number
  lastSetBody: string
  lastLoginBody: string
}

async function fulfillJson(route: Route, payload: unknown, status = 200) {
  await route.fulfill({
    status,
    contentType: "application/json",
    body: JSON.stringify(payload),
  })
}

function buildUser(state: ServerState) {
  return {
    user_id: "test-user",
    created_at: "2026-04-20T00:00:00Z",
    last_login_at: "2026-04-25T08:00:00Z",
    daily_limit: 20,
    success_count: 3,
    in_flight: 0,
    remaining_today: 17,
    has_password: state.hasPassword,
    tos_accepted_at: state.hasPassword ? "2026-04-25T08:00:00Z" : undefined,
    tos_version: state.hasPassword ? "1.0" : undefined,
  }
}

async function installPublicAuthMocks(page: Page, initial: Partial<ServerState>) {
  const state: ServerState = {
    hasPassword: false,
    storedPassword: null,
    loggedIn: true,
    rememberLong: false,
    setPasswordCalls: 0,
    passwordLoginCalls: 0,
    changeCalls: 0,
    lastSetBody: "",
    lastLoginBody: "",
    ...initial,
  }

  await page.addInitScript(() => {
    localStorage.clear()
    localStorage.setItem("hone-public-locale", "zh")
  })

  await page.route("**/api/**", async (route) => {
    const request = route.request()
    const url = new URL(request.url())
    const path = url.pathname

    if (path === "/api/meta") {
      await fulfillJson(route, {
        name: "hone",
        version: "test",
        channel: "web",
        supportsImessage: false,
        apiVersion: "desktop-v1",
        capabilities: ["public_chat"],
        deploymentMode: "remote",
      })
      return
    }

    if (path === "/api/public/auth/me") {
      if (!state.loggedIn) {
        await fulfillJson(route, { error: "未登录" }, 401)
        return
      }
      await fulfillJson(route, { user: buildUser(state) })
      return
    }

    if (path === "/api/public/history") {
      await fulfillJson(route, { messages: [] })
      return
    }

    if (path === "/api/public/events") {
      await route.fulfill({ status: 204, body: "" })
      return
    }

    if (path === "/api/public/auth/set-password") {
      state.setPasswordCalls += 1
      state.lastSetBody = request.postData() ?? ""
      const body = JSON.parse(state.lastSetBody || "{}") as {
        new_password?: string
        tos_version?: string
      }
      if (
        !body.new_password ||
        body.new_password.length < 8 ||
        body.new_password.length > 128 ||
        !/[0-9]/.test(body.new_password) ||
        !/[A-Za-z]/.test(body.new_password)
      ) {
        await fulfillJson(route, { error: "密码强度不足" }, 400)
        return
      }
      if (state.hasPassword) {
        await fulfillJson(route, { error: "已设过密码" }, 409)
        return
      }
      state.hasPassword = true
      state.storedPassword = body.new_password
      await fulfillJson(route, { user: buildUser(state) })
      return
    }

    if (path === "/api/public/auth/password-login") {
      state.passwordLoginCalls += 1
      state.lastLoginBody = request.postData() ?? ""
      const body = JSON.parse(state.lastLoginBody || "{}") as {
        phone_number?: string
        password?: string
        remember?: boolean
      }
      if (!body.phone_number || body.phone_number !== PHONE) {
        await fulfillJson(route, { error: "手机号或密码不正确" }, 401)
        return
      }
      if (!body.password || body.password !== state.storedPassword) {
        await fulfillJson(route, { error: "手机号或密码不正确" }, 401)
        return
      }
      state.loggedIn = true
      state.rememberLong = !!body.remember
      await fulfillJson(route, { user: buildUser(state) })
      return
    }

    if (path === "/api/public/auth/change-password") {
      state.changeCalls += 1
      const body = JSON.parse(request.postData() ?? "{}") as {
        current_password?: string
        new_password?: string
      }
      if (!body.current_password || body.current_password !== state.storedPassword) {
        await fulfillJson(route, { error: "当前密码不正确" }, 401)
        return
      }
      if (
        !body.new_password ||
        body.new_password.length < 8 ||
        !/[0-9]/.test(body.new_password) ||
        !/[A-Za-z]/.test(body.new_password)
      ) {
        await fulfillJson(route, { error: "新密码强度不足" }, 400)
        return
      }
      if (body.new_password === body.current_password) {
        await fulfillJson(route, { error: "新密码不能与当前密码相同" }, 400)
        return
      }
      state.storedPassword = body.new_password
      await fulfillJson(route, { user: buildUser(state) })
      return
    }

    if (path === "/api/public/auth/logout") {
      state.loggedIn = false
      await fulfillJson(route, { ok: true })
      return
    }

    await route.fallback()
  })

  return state
}

test("forced first-login: set password → logout → login with phone + password", async ({
  page,
}) => {
  const mock = await installPublicAuthMocks(page, {
    hasPassword: false,
    loggedIn: true,
  })

  await page.goto("/me")

  // The setup guard pops a non-closable modal because user.has_password=false.
  const guardTitle = page.getByText("首次登录：请设置密码")
  await expect(guardTitle).toBeVisible()

  // No close button (blockClose is honored).
  await expect(
    page.getByRole("button", { name: "关闭" }),
  ).toHaveCount(0)

  // Fill new password + confirm. Use an explicit getByLabel to avoid clashing
  // with any other password input that might exist.
  await page.getByLabel("新密码").fill("abcd1234")
  await page.getByLabel("确认密码").fill("abcd1234")

  // Tick the ToS checkbox — the label spans the whole row so click on the role.
  const tosBox = page.getByRole("checkbox").first()
  await tosBox.click()
  await expect(tosBox).toHaveAttribute("aria-checked", "true")

  await page.getByRole("button", { name: "保存并继续" }).click()

  // Modal disappears once /set-password resolves.
  await expect(guardTitle).toBeHidden()
  await expect.poll(() => mock.setPasswordCalls).toBe(1)
  expect(mock.lastSetBody).toContain('"new_password":"abcd1234"')
  expect(mock.lastSetBody).toContain('"tos_version":"1.0"')

  // Logged-in account view should now be visible.
  await expect(page.getByText("账号信息")).toBeVisible()

  // Logout — wait for the menu to reflect logged-out state.
  await page.getByRole("button", { name: "退出登录" }).click()
  await expect.poll(() => mock.loggedIn).toBe(false)

  // Reload to land on the LoggedOutView.
  await page.goto("/me")

  // Default tab is "密码登录".
  await expect(page.getByTestId("tab-password")).toBeVisible()
  await page.getByLabel("手机号").fill(PHONE)
  await page.getByLabel("密码", { exact: true }).fill("abcd1234")

  // ToS checkbox required even on login.
  await page.getByRole("checkbox").nth(1).click() // 0 = remember (already checked), 1 = ToS

  await page.getByRole("button", { name: "登录", exact: true }).click()
  await expect.poll(() => mock.passwordLoginCalls).toBe(1)
  expect(mock.lastLoginBody).toContain('"remember":true')

  await expect(page.getByText("账号信息")).toBeVisible()
})

test("password-rules live hints + submit button gating", async ({ page }) => {
  await installPublicAuthMocks(page, {
    hasPassword: false,
    loggedIn: true,
  })

  await page.goto("/me")
  await expect(page.getByText("首次登录：请设置密码")).toBeVisible()

  const newField = page.getByLabel("新密码")
  const submit = page.getByRole("button", { name: "保存并继续" })

  // Too short — submit disabled even without ToS check.
  await newField.fill("abc")
  await expect(submit).toBeDisabled()

  // Long enough but digits-only → still missing letter.
  await newField.fill("12345678")
  await expect(submit).toBeDisabled()

  // Strong password but ToS still unchecked + no confirm → still disabled.
  await newField.fill("abcd1234")
  await expect(submit).toBeDisabled()

  // Confirm matches but ToS still unchecked.
  await page.getByLabel("确认密码").fill("abcd1234")
  await expect(submit).toBeDisabled()

  // Tick ToS → enabled.
  await page.getByRole("checkbox").first().click()
  await expect(submit).toBeEnabled()
})

test("login submit gated on ToS checkbox", async ({ page }) => {
  await installPublicAuthMocks(page, {
    hasPassword: true,
    storedPassword: STORED_PASSWORD,
    loggedIn: false,
  })

  await page.goto("/me")
  await page.getByLabel("手机号").fill(PHONE)
  await page.getByLabel("密码", { exact: true }).fill(STORED_PASSWORD)

  const submit = page.getByRole("button", { name: "登录", exact: true })
  await expect(submit).toBeDisabled()

  // Tick ToS (second checkbox; first is "保持登录" — already checked by default).
  await page.getByRole("checkbox").nth(1).click()
  await expect(submit).toBeEnabled()
})

test("change password from logged-in view: wrong current → error, correct → success", async ({
  page,
}) => {
  const mock = await installPublicAuthMocks(page, {
    hasPassword: true,
    storedPassword: STORED_PASSWORD,
    loggedIn: true,
  })

  await page.goto("/me")
  await expect(page.getByText("账号信息")).toBeVisible()

  await page.getByRole("button", { name: "修改密码" }).click()

  await page.getByLabel("当前密码").fill("wrong-pass")
  await page.getByLabel("新密码", { exact: true }).fill("newpass123")
  await page.getByLabel("确认新密码").fill("newpass123")
  await page.getByRole("button", { name: "保存", exact: true }).click()

  await expect(page.getByText("当前密码不正确")).toBeVisible()
  await expect.poll(() => mock.changeCalls).toBe(1)

  // Retry with the correct current password.
  await page.getByLabel("当前密码").fill(STORED_PASSWORD)
  await page.getByRole("button", { name: "保存", exact: true }).click()

  await expect(page.getByText("✓ 密码已更新。下次登录请使用新密码。")).toBeVisible()
  await expect.poll(() => mock.changeCalls).toBe(2)
})
