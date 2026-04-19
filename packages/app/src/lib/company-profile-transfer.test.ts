import { describe, expect, it } from "bun:test"
import {
  buildCompanyProfileImportApplyRequest,
  isCompanyProfileImportReady,
  shouldCreateCompanyProfileBackup,
} from "./company-profile-transfer"
import type { CompanyProfileImportPreview } from "./types"

const emptyPreview: CompanyProfileImportPreview = {
  manifest: {
    version: "company-profile-bundle-v1",
    exported_at: "2026-04-19T00:00:00Z",
    profile_count: 1,
    event_count: 0,
    profiles: [],
  },
  profiles: [
    {
      profile_id: "SNOW",
      company_name: "Snowflake",
      stock_code: "SNOW",
      updated_at: "2026-04-19T00:00:00Z",
      event_count: 0,
      thesis_excerpt: "SaaS thesis",
    },
  ],
  conflicts: [],
  importable_count: 1,
  conflict_count: 0,
  suggested_mode: "keep_existing",
}

const conflictPreview: CompanyProfileImportPreview = {
  ...emptyPreview,
  conflicts: [
    {
      imported: {
        profile_id: "AAPL",
        company_name: "Apple Inc.",
        stock_code: "AAPL",
        updated_at: "2026-04-19T00:00:00Z",
        event_count: 1,
        thesis_excerpt: "Import thesis",
      },
      existing: {
        profile_id: "AAPL",
        company_name: "Apple",
        stock_code: "AAPL",
        updated_at: "2026-04-18T00:00:00Z",
        event_count: 2,
        thesis_excerpt: "Existing thesis",
      },
      reasons: ["股票代码相同"],
    },
  ],
  conflict_count: 1,
  suggested_mode: "interactive",
}

describe("company profile transfer helpers", () => {
  it("allows direct submit when preview has no conflicts", () => {
    expect(isCompanyProfileImportReady(emptyPreview, {})).toBe(true)
    expect(buildCompanyProfileImportApplyRequest(emptyPreview, {})).toEqual({
      mode: "keep_existing",
      decisions: {},
    })
  })

  it("requires all conflict decisions before submit", () => {
    expect(isCompanyProfileImportReady(conflictPreview, {})).toBe(false)
    expect(() =>
      buildCompanyProfileImportApplyRequest(conflictPreview, {}),
    ).toThrow("missing decision")
  })

  it("builds interactive payload once conflicts are fully decided", () => {
    const decisions = { AAPL: "replace" as const }
    expect(isCompanyProfileImportReady(conflictPreview, decisions)).toBe(true)
    expect(
      buildCompanyProfileImportApplyRequest(conflictPreview, decisions),
    ).toEqual({
      mode: "interactive",
      decisions: { AAPL: "replace" },
    })
  })

  it("only requests a backup when at least one conflict will be replaced", () => {
    expect(shouldCreateCompanyProfileBackup(conflictPreview, {})).toBe(false)
    expect(
      shouldCreateCompanyProfileBackup(conflictPreview, { AAPL: "skip" }),
    ).toBe(false)
    expect(
      shouldCreateCompanyProfileBackup(conflictPreview, { AAPL: "replace" }),
    ).toBe(true)
  })
})
