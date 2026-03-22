import { describe, expect, test } from "bun:test"
import {
  buildApiUrl,
  buildAuthHeaders,
  hasRuntimeCapability,
  normalizeBaseUrl,
  resolveBaseUrl,
  setBackendRuntime,
  supportsApiVersion,
} from "./backend"

describe("backend runtime helpers", () => {
  test("normalizeBaseUrl trims trailing slash", () => {
    expect(normalizeBaseUrl("https://example.com///")).toBe("https://example.com")
  })

  test("resolveBaseUrl prefers bundled fallback", () => {
    expect(
      resolveBaseUrl(
        { mode: "bundled", baseUrl: "" },
        "http://127.0.0.1:8077/",
      ),
    ).toBe("http://127.0.0.1:8077")
  })

  test("buildApiUrl resolves relative paths against current origin when no backend base URL exists", () => {
    expect(buildApiUrl("/api/cron-jobs", "")).toBe("http://localhost/api/cron-jobs")
  })

  test("buildAuthHeaders injects bearer token", () => {
    const headers = buildAuthHeaders(undefined, "secret-token")
    expect(headers.get("Authorization")).toBe("Bearer secret-token")
  })

  test("supportsApiVersion only accepts desktop-v1", () => {
    expect(supportsApiVersion("desktop-v1")).toBe(true)
    expect(supportsApiVersion("desktop-v2")).toBe(false)
  })

  test("browser runtime keeps local file proxy enabled before meta handshake", () => {
    setBackendRuntime({
      mode: "browser",
      baseUrl: "",
      bearerToken: "",
      meta: undefined,
      isDesktop: false,
    })
    expect(hasRuntimeCapability("local_file_proxy")).toBe(true)
    expect(hasRuntimeCapability("logs")).toBe(false)
  })
})
