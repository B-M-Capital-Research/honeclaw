import { describe, expect, it } from "bun:test"
import { readFileSync } from "node:fs"
import { join } from "node:path"

const pagesDir = import.meta.dir

describe("desktop runner visibility contract", () => {
  it("exposes codex_acp on the dashboard page", () => {
    const source = readFileSync(join(pagesDir, "dashboard.tsx"), "utf8")
    expect(source).toContain('runner: "codex_acp"')
    expect(source).toContain('name: "Codex ACP"')
  })

  it("exposes hone_cloud and hides legacy runner cards", () => {
    const dashboard = readFileSync(join(pagesDir, "dashboard.tsx"), "utf8")
    const settings = readFileSync(join(pagesDir, "settings.tsx"), "utf8")
    expect(dashboard).toContain('runner: "hone_cloud"')
    expect(dashboard).not.toContain('runner: "multi-agent"')
    expect(dashboard).not.toContain('runner: "codex_cli"')
    expect(settings).toContain('selectRunner("hone_cloud")')
    expect(settings).not.toContain('selectRunner("multi-agent")')
    expect(settings).not.toContain('selectRunner("codex_cli")')
  })

  it("exposes codex_acp on the desktop settings page", () => {
    const source = readFileSync(join(pagesDir, "settings.tsx"), "utf8")
    expect(source).toContain('selectRunner("codex_acp")')
    expect(source).toContain('checkDesktopAgentCli("codex_acp")')
    expect(source).toContain("Codex ACP")
  })
})
