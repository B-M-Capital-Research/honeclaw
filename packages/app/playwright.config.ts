import { defineConfig } from "@playwright/test"

const ADMIN_PORT = Number(process.env.HONE_E2E_ADMIN_PORT ?? 4173)
const PUBLIC_PORT = Number(process.env.HONE_E2E_PUBLIC_PORT ?? 4174)

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  webServer: [
    {
      command: `bun run dev -- --host 127.0.0.1 --port ${ADMIN_PORT}`,
      url: `http://127.0.0.1:${ADMIN_PORT}`,
      reuseExistingServer: !process.env.CI,
      timeout: 120_000,
    },
    {
      command: `bun run dev -- --host 127.0.0.1 --port ${PUBLIC_PORT}`,
      url: `http://127.0.0.1:${PUBLIC_PORT}`,
      env: { VITE_HONE_APP_SURFACE: "public" },
      reuseExistingServer: !process.env.CI,
      timeout: 120_000,
    },
  ],
  projects: [
    {
      name: "admin",
      testIgnore: /public-(chat-upload|password-flow)\.spec\.ts$/,
      use: {
        baseURL:
          process.env.HONE_E2E_BASE_URL ?? `http://127.0.0.1:${ADMIN_PORT}`,
        headless: true,
      },
    },
    {
      name: "public",
      testMatch: /public-(chat-upload|password-flow)\.spec\.ts$/,
      use: {
        baseURL: `http://127.0.0.1:${PUBLIC_PORT}`,
        headless: true,
      },
    },
  ],
})
