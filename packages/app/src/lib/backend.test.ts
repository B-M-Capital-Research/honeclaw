import { beforeEach, describe, expect, test } from "bun:test"
import {
  buildApiUrl,
  buildAuthHeaders,
  defaultBackendConfig,
  hasRuntimeCapability,
  normalizeBaseUrl,
  resolveBaseUrl,
  setBackendRuntime,
  supportsApiVersion,
} from "./backend"

describe("backend runtime helpers", () => {
  beforeEach(() => {
    setBackendRuntime({
      mode: "browser",
      baseUrl: "",
      bearerToken: "",
      meta: undefined,
      isDesktop: false,
    })
  })

  test("defaultBackendConfig uses bundled mode with empty connection details", () => {
    expect(defaultBackendConfig()).toEqual({
      mode: "bundled",
      baseUrl: "",
      bearerToken: "",
    })
  })

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

  test("resolveBaseUrl ignores fallback in remote mode", () => {
    expect(
      resolveBaseUrl(
        { mode: "remote", baseUrl: "https://api.example.com///" },
        "http://127.0.0.1:8077/",
      ),
    ).toBe("https://api.example.com")
  })

  test("buildApiUrl resolves relative paths against current origin when no backend base URL exists", () => {
    expect(buildApiUrl("/api/cron-jobs", "")).toBe("http://localhost/api/cron-jobs")
  })

  test("buildAuthHeaders injects bearer token", () => {
    const headers = buildAuthHeaders(undefined, "secret-token")
    expect(headers.get("Authorization")).toBe("Bearer secret-token")
  })

  test("buildAuthHeaders preserves existing headers and skips empty tokens", () => {
    const headers = buildAuthHeaders({ "X-Test": "1" }, "")
    expect(headers.get("X-Test")).toBe("1")
    expect(headers.has("Authorization")).toBe(false)
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

  test("runtime capabilities come from backend meta after handshake", () => {
    setBackendRuntime({
      mode: "bundled",
      meta: {
        name: "hone",
        version: "1.0.0",
        channel: "desktop",
        supportsImessage: false,
        apiVersion: "desktop-v1",
        capabilities: ["logs", "channels"],
        deploymentMode: "local",
      },
    })
    expect(hasRuntimeCapability("logs")).toBe(true)
    expect(hasRuntimeCapability("local_file_proxy")).toBe(false)
  })
})
