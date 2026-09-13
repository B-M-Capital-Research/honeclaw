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

async function installPdfConversation(page: Page, isAdmin = true) {
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
    is_admin: isAdmin,
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

  const pdfCard = page.getByRole("button", { name: `Download ${PDF_NAME}`, exact: true })
  await expect(pdfCard).toBeVisible()
  await expect(pdfCard).toContainText("PDF")
  const href = `/api/public/file?path=${encodeURIComponent(PDF_PATH)}`

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

for (const [kind, label] of [["preview", "财报前瞻"], ["analysis", "财报分析"]] as const) {
  test(`${label} starts from company alone without requesting uploads`, async ({ page }) => {
    await installPdfConversation(page)
    await page.addInitScript(() => localStorage.setItem("hone-public-locale", "zh"))
    let requestBody: Record<string, unknown> | undefined
    let uploadCalls = 0
    await page.route("**/api/public/upload", async (route) => {
      uploadCalls++
      await route.abort()
    })
    await page.route("**/api/public/chat", async (route) => {
      requestBody = route.request().postDataJSON()
      await route.fulfill({ status: 200, contentType: "text/event-stream", body: "event: done\ndata: {}\n\n" })
    })
    await page.goto("/chat")
    await page.getByRole("button", { name: "工具", exact: true }).click()
    await page.getByRole("menuitem", { name: label }).click()
    const dialog = page.getByRole("dialog", { name: label })
    await expect(dialog).toBeVisible()
    await expect(dialog.locator('input[type="file"]')).toHaveCount(0)
    await expect(dialog.getByText("财报材料（可选）")).toHaveCount(0)
    await expect(page.getByTestId(`earnings-${kind}-start`)).toBeDisabled()
    await page.getByTestId(`earnings-${kind}-company`).fill(" TEM ")
    await page.getByTestId(`earnings-${kind}-start`).click()
    await expect.poll(() => requestBody).toBeDefined()
    expect(requestBody!.earnings_workflow).toEqual({ kind, company: "TEM" })
    expect(requestBody!.attachments ?? []).toEqual([])
    expect(uploadCalls).toBe(0)
    await expect(dialog).toBeHidden()
  })
}

test("company analysis restores real task progress without repeating start and downloads its complete PDF", async ({page}) => {
  await installPdfConversation(page);
  await page.addInitScript(() => localStorage.setItem("hone-public-locale", "zh"));
  await page.setViewportSize({width:1440,height:1024});
  let starts=0;
  let task: Record<string,unknown> | undefined;
  await page.route("**/api/public/company-analysis/tasks**",async route=>{
    const url=new URL(route.request().url());
    if(url.pathname.endsWith("/pdf")) {await route.fulfill({status:200,contentType:"application/pdf",path:PDF_FIXTURE});return;}
    if(route.request().method()==="POST") {
      starts++;expect(route.request().postDataJSON()).toEqual({company:"TEM"});
      task={task_id:"11111111-1111-4111-8111-111111111111",company:"TEM",status:"running",progress:25,info:"公司基本信息搜索完成",created_at:new Date().toISOString(),updated_at:new Date().toISOString(),pdf_ready:false,completed_stages:["公司基本介绍"]};
      await fulfillJson(route,task);return;
    }
    await fulfillJson(route,url.pathname.endsWith("/tasks")?{tasks:task?[task]:[]}:task);
  });
  const open=async()=>{await page.getByRole("button",{name:"工具",exact:true}).click();await page.getByRole("menuitem",{name:"公司完整分析"}).click();};
  await page.goto("/chat");await open();
  const dialog=page.getByRole("dialog",{name:"公司完整分析"});
  await expect(dialog.locator('input[type="file"]')).toHaveCount(0);
  await page.getByTestId("company-analysis-company").fill(" TEM ");
  await page.getByTestId("company-analysis-start").click();
  await expect(dialog.getByText("25%",{exact:true})).toBeVisible();
  await expect(dialog.getByText(/任务 ID：11111111/)).toBeVisible();
  await dialog.getByRole("button",{name:"关闭",exact:true}).click();
  await page.reload();await open();
  await expect(dialog.getByText("25%",{exact:true})).toBeVisible();expect(starts).toBe(1);
  task={...task,progress:95,info:"PDF 已生成，正在保存"};
  await expect(dialog.getByText("95%",{exact:true})).toBeVisible();
  await expect(page.getByTestId("company-analysis-download")).toHaveCount(0);
  task={...task,status:"completed",progress:100,pdf_ready:true,info:"完整分析与 PDF 已完成"};
  await expect(page.getByTestId("company-analysis-download")).toBeVisible();
  const downloaded=page.waitForEvent("download");await page.getByTestId("company-analysis-download").click();
  expect((await downloaded).suggestedFilename()).toBe("TEM-公司完整分析.pdf");
  expect(starts).toBe(1);
});

test("company analysis shortcut is absent for non-administrators",async({page})=>{
  await installPdfConversation(page,false);
  await page.addInitScript(()=>localStorage.setItem("hone-public-locale","zh"));
  await page.goto("/chat");await page.getByRole("button",{name:"工具",exact:true}).click();
  await expect(page.getByRole("menuitem",{name:"公司完整分析"})).toHaveCount(0);
});
