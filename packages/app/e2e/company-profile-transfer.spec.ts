import { expect, test, type Page, type Route } from "@playwright/test"

type RawProfileSummary = {
  profile_id: string
  title: string
  updated_at?: string
  event_count: number
}

type RawProfileDetail = {
  profile_id: string
  title: string
  updated_at?: string
  markdown: string
  events: Array<{
    id: string
    filename: string
    title: string
    updated_at?: string
    markdown: string
  }>
}

type MockApiState = {
  exportCalls: number
  previewCalls: number
  applyCalls: number
  lastPreviewBody: string
  lastApplyBody: string
  profiles: RawProfileSummary[]
  detailsById: Record<string, RawProfileDetail>
}

async function fulfillJson(route: Route, payload: unknown, status = 200) {
  await route.fulfill({
    status,
    contentType: "application/json",
    body: JSON.stringify(payload),
  })
}

async function installCompanyProfileApiMock(
  page: Page,
  options: {
    previewPayload: unknown
    applyPayload: unknown
    profiles: RawProfileSummary[]
    detailsById: Record<string, RawProfileDetail>
    onApply?: (state: MockApiState) => void
  },
) {
  const state: MockApiState = {
    exportCalls: 0,
    previewCalls: 0,
    applyCalls: 0,
    lastPreviewBody: "",
    lastApplyBody: "",
    profiles: [...options.profiles],
    detailsById: { ...options.detailsById },
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
        capabilities: ["company_profiles", "company_profile_transfer", "users"],
        deploymentMode: "local",
      })
      return
    }

    if (path === "/api/users") {
      await fulfillJson(route, [])
      return
    }

    if (path === "/api/company-profiles/actors") {
      await fulfillJson(route, {
        actors: [
          {
            channel: "discord",
            user_id: "alice",
            channel_scope: "watchlist",
            profile_count: state.profiles.length,
            updated_at: state.profiles[0]?.updated_at ?? "2026-04-19T08:00:00Z",
          },
        ],
      })
      return
    }

    if (path === "/api/company-profiles/export") {
      state.exportCalls += 1
      await route.fulfill({
        status: 200,
        contentType: "application/zip",
        headers: {
          "content-disposition":
            'attachment; filename="company-profiles-discord-watchlist-alice-20260419.zip"',
        },
        body: Buffer.from("PK\x03\x04mock-bundle"),
      })
      return
    }

    if (path === "/api/company-profiles/import/preview") {
      state.previewCalls += 1
      state.lastPreviewBody = request.postDataBuffer()?.toString("utf-8") ?? ""
      await fulfillJson(route, options.previewPayload)
      return
    }

    if (path === "/api/company-profiles/import/apply") {
      state.applyCalls += 1
      state.lastApplyBody = request.postDataBuffer()?.toString("utf-8") ?? ""
      options.onApply?.(state)
      await fulfillJson(route, options.applyPayload)
      return
    }

    if (path === "/api/company-profiles") {
      await fulfillJson(route, { profiles: state.profiles })
      return
    }

    if (path.startsWith("/api/company-profiles/")) {
      const profileId = decodeURIComponent(path.slice("/api/company-profiles/".length))
      const detail = state.detailsById[profileId]
      if (!detail) {
        await fulfillJson(route, { error: "company profile not found" }, 404)
        return
      }
      await fulfillJson(route, { profile: detail })
      return
    }

    await route.fallback()
  })

  return state
}

test("company profile transfer exports and backs up before conflict replace", async ({
  page,
}) => {
  const mock = await installCompanyProfileApiMock(page, {
    profiles: [
      {
        profile_id: "AAPL",
        title: "Apple Inc.",
        updated_at: "2026-04-18T09:00:00Z",
        event_count: 1,
      },
    ],
    detailsById: {
      AAPL: {
        profile_id: "AAPL",
        title: "Apple Inc.",
        updated_at: "2026-04-18T09:00:00Z",
        markdown: "# Apple Inc.\n\n## Thesis\ncurrent thesis\n",
        events: [
          {
            id: "2026-04-18-update",
            filename: "2026-04-18-update.md",
            title: "Current Event",
            updated_at: "2026-04-18T09:00:00Z",
            markdown: "# Current Event\n\nbody\n",
          },
        ],
      },
    },
    previewPayload: {
      preview: {
        manifest: {
          version: "company-profile-bundle-v1",
          exported_at: "2026-04-19T08:30:00Z",
          profile_count: 1,
          event_count: 0,
          profiles: [
            {
              profile_id: "AAPL",
              company_name: "Apple Inc.",
              stock_code: "AAPL",
              event_count: 0,
              updated_at: "2026-04-19T08:30:00Z",
            },
          ],
        },
        profiles: [
          {
            profile_id: "AAPL",
            company_name: "Apple Inc.",
            stock_code: "AAPL",
            updated_at: "2026-04-19T08:30:00Z",
            event_count: 0,
            thesis_excerpt: "imported thesis",
          },
        ],
        conflicts: [
          {
            imported: {
              profile_id: "AAPL",
              company_name: "Apple Inc.",
              stock_code: "AAPL",
              updated_at: "2026-04-19T08:30:00Z",
              event_count: 0,
              thesis_excerpt: "imported thesis",
            },
            existing: {
              profile_id: "AAPL",
              company_name: "Apple Inc.",
              stock_code: "AAPL",
              updated_at: "2026-04-18T09:00:00Z",
              event_count: 1,
              thesis_excerpt: "current thesis",
            },
            reasons: ["股票代码相同", "目录名相同"],
          },
        ],
        importable_count: 0,
        conflict_count: 1,
        suggested_mode: "interactive",
      },
    },
    applyPayload: {
      result: {
        imported_count: 0,
        replaced_count: 1,
        skipped_count: 0,
        imported_profile_ids: [],
        replaced_profile_ids: ["AAPL"],
        skipped_profile_ids: [],
        changed_profile_ids: ["AAPL"],
      },
    },
    onApply(state) {
      state.profiles = [
        {
          profile_id: "AAPL",
          title: "Apple Inc.",
          updated_at: "2026-04-19T08:31:00Z",
          event_count: 0,
        },
      ]
      state.detailsById.AAPL = {
        profile_id: "AAPL",
        title: "Apple Inc.",
        updated_at: "2026-04-19T08:31:00Z",
        markdown: "# Apple Inc.\n\n## Thesis\nimported thesis\n",
        events: [],
      }
    },
  })

  // 新 IA: /users/:actorKey/profiles 直接落到目标 actor 的画像 tab,
  // actorKey 格式:channel|channel_scope|user_id → discord|watchlist|alice
  await page.goto("/users/discord%7Cwatchlist%7Calice/profiles")

  await expect(page.getByRole("button", { name: "导出当前空间" })).toBeVisible()
  await page.getByRole("button", { name: "导出当前空间" }).click()
  await expect.poll(() => mock.exportCalls).toBe(1)

  await page
    .locator('input[type="file"]')
    .setInputFiles({
      name: "company-profile-bundle.zip",
      mimeType: "application/zip",
      buffer: Buffer.from("mock upload bundle"),
    })

  await expect(page.getByText("冲突审阅")).toBeVisible()
  expect(mock.lastPreviewBody).toContain('name="bundle"')

  await page.getByRole("button", { name: /^用导入版本替换$/ }).click()
  await page.getByRole("button", { name: "开始导入" }).click()

  await expect.poll(() => mock.exportCalls).toBe(2)
  await expect.poll(() => mock.applyCalls).toBe(1)
  expect(mock.lastApplyBody).toContain('name="mode"')
  expect(mock.lastApplyBody).toContain("interactive")
  expect(mock.lastApplyBody).toContain('"AAPL":"replace"')

  await expect(page.getByText("导入完成", { exact: true })).toBeVisible()
  await expect(page.getByRole("button", { name: "下载导入前备份" })).toBeVisible()
  // 导入完成后,被替换/新增的画像在顶部横向选择条上应有"更新"徽章
  await expect(page.getByText("更新", { exact: true }).first()).toBeVisible()
})

test("company profile transfer imports directly when preview has no conflicts", async ({
  page,
}) => {
  const mock = await installCompanyProfileApiMock(page, {
    profiles: [
      {
        profile_id: "TSLA",
        title: "Tesla",
        updated_at: "2026-04-18T09:00:00Z",
        event_count: 0,
      },
    ],
    detailsById: {
      TSLA: {
        profile_id: "TSLA",
        title: "Tesla",
        updated_at: "2026-04-18T09:00:00Z",
        markdown: "# Tesla\n\n## Thesis\nexisting thesis\n",
        events: [],
      },
      MSFT: {
        profile_id: "MSFT",
        title: "Microsoft",
        updated_at: "2026-04-19T08:31:00Z",
        markdown: "# Microsoft\n\n## Thesis\nimported thesis\n",
        events: [],
      },
    },
    previewPayload: {
      preview: {
        manifest: {
          version: "company-profile-bundle-v1",
          exported_at: "2026-04-19T08:30:00Z",
          profile_count: 1,
          event_count: 0,
          profiles: [
            {
              profile_id: "MSFT",
              company_name: "Microsoft",
              stock_code: "MSFT",
              event_count: 0,
              updated_at: "2026-04-19T08:30:00Z",
            },
          ],
        },
        profiles: [
          {
            profile_id: "MSFT",
            company_name: "Microsoft",
            stock_code: "MSFT",
            updated_at: "2026-04-19T08:30:00Z",
            event_count: 0,
            thesis_excerpt: "imported thesis",
          },
        ],
        conflicts: [],
        importable_count: 1,
        conflict_count: 0,
        suggested_mode: "keep_existing",
      },
    },
    applyPayload: {
      result: {
        imported_count: 1,
        replaced_count: 0,
        skipped_count: 0,
        imported_profile_ids: ["MSFT"],
        replaced_profile_ids: [],
        skipped_profile_ids: [],
        changed_profile_ids: ["MSFT"],
      },
    },
    onApply(state) {
      state.profiles = [
        ...state.profiles,
        {
          profile_id: "MSFT",
          title: "Microsoft",
          updated_at: "2026-04-19T08:31:00Z",
          event_count: 0,
        },
      ]
    },
  })

  // 新 IA: /users/:actorKey/profiles 直接落到目标 actor 的画像 tab,
  // actorKey 格式:channel|channel_scope|user_id → discord|watchlist|alice
  await page.goto("/users/discord%7Cwatchlist%7Calice/profiles")

  await page
    .locator('input[type="file"]')
    .setInputFiles({
      name: "company-profile-bundle.zip",
      mimeType: "application/zip",
      buffer: Buffer.from("mock upload bundle"),
    })

  await expect(page.getByText("当前空间没有冲突，可以直接导入。")).toBeVisible()
  await page.getByRole("button", { name: "开始导入" }).click()

  await expect.poll(() => mock.applyCalls).toBe(1)
  await expect.poll(() => mock.exportCalls).toBe(0)
  expect(mock.lastApplyBody).toContain("keep_existing")

  await expect(page.getByText("导入完成", { exact: true })).toBeVisible()
  await expect(page.getByRole("button", { name: "下载导入前备份" })).toHaveCount(0)
})
