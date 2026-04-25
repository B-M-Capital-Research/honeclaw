import { expect, test, type Page, type Route } from "@playwright/test"

// Smallest valid PNG (1x1 transparent). Used to respond to /api/public/image.
const TINY_PNG_BASE64 =
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="

const UPLOAD_PATH =
  "/tmp/hone/public-uploads/test-user/2026-04-25/cat.jpg"
const ASSISTANT_IMAGE_PATH =
  "/tmp/hone/public-uploads/test-user/2026-04-25/reply.png"

async function fulfillJson(route: Route, payload: unknown, status = 200) {
  await route.fulfill({
    status,
    contentType: "application/json",
    body: JSON.stringify(payload),
  })
}

function buildSseBody(
  events: Array<{ event: string; data: Record<string, unknown> }>,
) {
  return (
    events
      .map(
        ({ event, data }) => `event: ${event}\ndata: ${JSON.stringify(data)}`,
      )
      .join("\n\n") + "\n\n"
  )
}

type MockState = {
  historyCalls: number
  uploadCalls: number
  chatCalls: number
  lastChatBody: string
}

async function installPublicChatMocks(page: Page) {
  const state: MockState = {
    historyCalls: 0,
    uploadCalls: 0,
    chatCalls: 0,
    lastChatBody: "",
  }

  await page.addInitScript(() => {
    localStorage.clear()
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
        capabilities: ["public_chat", "local_file_proxy"],
        deploymentMode: "remote",
      })
      return
    }

    if (path === "/api/public/auth/me") {
      await fulfillJson(route, {
        user: {
          user_id: "test-user",
          created_at: "2026-04-20T00:00:00Z",
          daily_limit: 20,
          success_count: 0,
          in_flight: 0,
          remaining_today: 20,
          has_password: true,
        },
      })
      return
    }

    if (path === "/api/public/history") {
      state.historyCalls += 1
      // First call = empty history; subsequent calls (after send) return the
      // full exchange with server-persisted attachments.
      if (state.historyCalls === 1) {
        await fulfillJson(route, { messages: [] })
        return
      }
      await fulfillJson(route, {
        messages: [
          {
            role: "user",
            content: `你看看这张图\n[附件: ${UPLOAD_PATH}]`,
            attachments: [
              { path: UPLOAD_PATH, name: "cat.jpg", kind: "image" },
            ],
          },
          {
            role: "assistant",
            content: `收到！参见下图：\n\n![](file://${ASSISTANT_IMAGE_PATH})`,
            attachments: [],
          },
        ],
      })
      return
    }

    if (path === "/api/public/events") {
      // 204 = EventSource should not reconnect; connection closes cleanly.
      await route.fulfill({ status: 204, body: "" })
      return
    }

    if (path === "/api/public/upload") {
      state.uploadCalls += 1
      await fulfillJson(route, {
        attachments: [
          {
            path: UPLOAD_PATH,
            name: "cat.jpg",
            kind: "image",
            size: 1234,
          },
        ],
      })
      return
    }

    if (path === "/api/public/chat") {
      state.chatCalls += 1
      state.lastChatBody = request.postData() ?? ""
      const body = buildSseBody([
        { event: "run_started", data: {} },
        {
          event: "assistant_delta",
          data: {
            content: `收到！参见下图：\n\n![](file://${ASSISTANT_IMAGE_PATH})`,
          },
        },
        { event: "run_finished", data: { success: true } },
      ])
      await route.fulfill({
        status: 200,
        headers: {
          "content-type": "text/event-stream",
          "cache-control": "no-cache",
        },
        body,
      })
      return
    }

    if (path === "/api/public/image" || path === "/api/image") {
      await route.fulfill({
        status: 200,
        contentType: "image/png",
        body: Buffer.from(TINY_PNG_BASE64, "base64"),
      })
      return
    }

    await route.fallback()
  })

  return state
}

test("public chat uploads an image and renders assistant image reply", async ({
  page,
}) => {
  const mock = await installPublicChatMocks(page)

  await page.goto("/chat")

  const attachButton = page.getByTestId("composer-attach-button")
  await expect(attachButton).toBeVisible()

  await page
    .getByTestId("composer-image-input")
    .setInputFiles({
      name: "cat.jpg",
      mimeType: "image/jpeg",
      buffer: Buffer.from("mock image bytes"),
    })

  // Preview strip should surface the just-picked image.
  await expect(page.getByTestId("composer-attach-preview")).toBeVisible()
  await expect.poll(() => mock.uploadCalls).toBe(1)

  await page
    .getByRole("textbox")
    .first()
    .fill("你看看这张图")

  await page.getByTestId("composer-send-button").click()

  await expect.poll(() => mock.chatCalls).toBe(1)
  expect(mock.lastChatBody).toContain(UPLOAD_PATH)
  expect(mock.lastChatBody).toContain('"message":"你看看这张图"')

  // User bubble shows the uploaded image (mosaic thumbnail).
  await expect(page.getByTestId("user-attachment-image").first()).toBeVisible()

  // Assistant inline image (from `file://…png` in the streamed reply).
  await expect(
    page.getByTestId("assistant-inline-image").first(),
  ).toBeVisible({ timeout: 10_000 })
})

test("public chat accepts pasted clipboard image", async ({ page }) => {
  const mock = await installPublicChatMocks(page)

  await page.goto("/chat")
  const textarea = page.getByRole("textbox").first()
  await expect(textarea).toBeVisible()

  // Dispatch a synthetic paste carrying an image file on the focused textarea.
  await textarea.focus()
  await textarea.evaluate((el) => {
    const bytes = new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10])
    const file = new File([bytes], "", { type: "image/png" })
    const dt = new DataTransfer()
    dt.items.add(file)
    el.dispatchEvent(
      new ClipboardEvent("paste", {
        clipboardData: dt,
        bubbles: true,
        cancelable: true,
      }),
    )
  })

  await expect(page.getByTestId("composer-attach-preview")).toBeVisible()
  await expect.poll(() => mock.uploadCalls).toBe(1)
})
