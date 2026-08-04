import { expect, test, type Page, type Route } from "@playwright/test"
import path from "node:path"
import { fileURLToPath } from "node:url"

const PDF_NAME = "SNDK（闪迪）_财报前瞻.pdf"
const PDF_PATH = "/tmp/hone/earnings-reports/SNDK-workflow-preview.pdf"
const TEST_DIR = path.dirname(fileURLToPath(import.meta.url))
const PDF_FIXTURE = path.join(TEST_DIR, "fixtures/sample-report.pdf")

async function fulfillJson(route: Route, payload: unknown, status = 200) {
  await route.fulfill({
    status,
    contentType: "application/json",
    body: JSON.stringify(payload),
  })
}

async function installPdfConversation(page: Page) {
  const user = {
    user_id: "admin-user",
    created_at: "2026-08-04T00:00:00Z",
    daily_limit: 20,
    success_count: 1,
    in_flight: 0,
    remaining_today: 19,
    has_password: true,
    identity_kind: "international_email",
    email_hint: "bm@vsource.club",
    billing: {
      access_granted: true,
      entitlements: [],
      has_duplicate_active_subscriptions: false,
    },
    is_admin: true,
  }
  const messages = [
    {
      role: "user",
      content:
        "请为 SNDK（闪迪）生成财报前瞻，并完成证据核验和可分享 PDF。",
      attachments: [],
    },
    {
      role: "assistant",
      content:
        "财报前瞻已完成：基准判断为超出分析师预期。完整正文、近期新闻与证据链见下方 PDF。",
      attachments: [
        {
          path: PDF_PATH,
          name: PDF_NAME,
          kind: "pdf",
          size: 680_659,
        },
      ],
    },
  ]
  await page.addInitScript(() => localStorage.clear())
  await page.route("**/api/**", async (route) => {
    const url = new URL(route.request().url())
    if (url.pathname === "/api/meta") {
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
    if (url.pathname === "/api/public/auth/me") {
      await fulfillJson(route, { user })
      return
    }
    if (url.pathname === "/api/public/bootstrap") {
      await fulfillJson(route, {
        user,
        messages,
        history_start: 0,
        next_before: null,
        active_run: null,
        interrupted_run: false,
      })
      return
    }
    if (url.pathname === "/api/public/history") {
      await fulfillJson(route, {
        messages,
        history_start: 0,
        next_before: null,
      })
      return
    }
    if (url.pathname === "/api/public/events") {
      await route.fulfill({ status: 204, body: "" })
      return
    }
    if (url.pathname === "/api/public/file") {
      expect(url.searchParams.get("path")).toBe(PDF_PATH)
      await route.fulfill({
        status: 200,
        contentType: "application/pdf",
        path: PDF_FIXTURE,
      })
      return
    }
    await route.fallback()
  })
}

test("assistant PDF card resolves PDF bytes and triggers a named download", async ({
  page,
}) => {
  await installPdfConversation(page)
  await page.setViewportSize({ width: 1440, height: 1024 })
  await page.goto("/chat")

  const pdfCard = page.getByRole("link", { name: PDF_NAME })
  await expect(pdfCard).toBeVisible()
  await expect(pdfCard).toContainText("PDF")
  const href = await pdfCard.getAttribute("href")
  expect(href).toContain("/api/public/file?path=")

  const fetched = await page.evaluate(async (url) => {
    const response = await fetch(url)
    const bytes = new Uint8Array(await response.arrayBuffer())
    return {
      ok: response.ok,
      contentType: response.headers.get("content-type"),
      magic: new TextDecoder().decode(bytes.slice(0, 5)),
    }
  }, href!)
  expect(fetched).toEqual({
    ok: true,
    contentType: "application/pdf",
    magic: "%PDF-",
  })

  await page.screenshot({
    path: path.resolve(
      TEST_DIR,
      "../../../output/pdf/screenshots/user-chat-pdf-download.png",
    ),
    fullPage: true,
  })

  const downloadPromise = page.waitForEvent("download")
  await pdfCard.click()
  const download = await downloadPromise
  expect(download.suggestedFilename()).toBe(PDF_NAME)
})
