import { afterEach, beforeEach, describe, expect, test } from "bun:test"
import {
  FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE,
  apiFetch,
  buildApiUrl,
  buildAuthHeaders,
  defaultBackendConfig,
  friendlyBackendErrorMessage,
  hasRuntimeCapability,
  resetApiFetchRetryDelayForTests,
  normalizeBaseUrl,
  resolveBaseUrl,
  setApiFetchRetryDelayForTests,
  setBackendRuntime,
  supportsApiVersion,
} from "./backend"

function requireValue<T>(value: T | null | undefined, label: string): T {
  if (value == null) {
    throw new Error(`${label} was not captured`)
  }
  return value
}

async function withWindowOrigin<T>(
  origin: string,
  testBody: () => T | Promise<T>,
): Promise<T> {
  const originalWindow = globalThis.window
  Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: { location: { origin } },
  })
  try {
    return await testBody()
  } finally {
    if (originalWindow === undefined) {
      delete (globalThis as { window?: Window }).window
    } else {
      Object.defineProperty(globalThis, "window", {
        configurable: true,
        value: originalWindow,
      })
    }
  }
}

async function captureApiFetchInit(path: string): Promise<RequestInit> {
  const originalFetch = globalThis.fetch
  let captured: RequestInit | undefined
  globalThis.fetch = ((_: RequestInfo | URL, init?: RequestInit) => {
    captured = init
    return Promise.resolve(new Response("{}", { status: 200 }))
  }) as typeof fetch
  try {
    await apiFetch(path)
    return requireValue(captured, "fetch init")
  } finally {
    globalThis.fetch = originalFetch
  }
}

describe("backend runtime helpers", () => {
  const originalFetch = globalThis.fetch

  beforeEach(() => {
    setBackendRuntime({
      mode: "browser",
      baseUrl: "",
      bearerToken: "",
      meta: undefined,
      isDesktop: false,
    })
    setApiFetchRetryDelayForTests(0)
  })

  afterEach(() => {
    globalThis.fetch = originalFetch
    resetApiFetchRetryDelayForTests()
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
    return withWindowOrigin("http://localhost", () => {
      expect(buildApiUrl("/api/cron-jobs", "")).toBe(
        "http://localhost/api/cron-jobs",
      )
    })
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

  test("apiFetch includes cookies for public auth refreshes", async () => {
    const init = await captureApiFetchInit("/api/public/auth/me")
    expect(init.credentials).toBe("include")
  })

  test("apiFetch retries transient backend statuses once", async () => {
    let calls = 0
    globalThis.fetch = ((_: RequestInfo | URL, __?: RequestInit) => {
      calls += 1
      const status = calls === 1 ? 502 : 200
      return Promise.resolve(new Response("{}", { status }))
    }) as typeof fetch

    const response = await apiFetch("/api/meta")

    expect(response.status).toBe(200)
    expect(calls).toBe(2)
  })

  test("apiFetch retries transport failures before showing friendly error", async () => {
    let calls = 0
    globalThis.fetch = ((_: RequestInfo | URL, __?: RequestInit) => {
      calls += 1
      return Promise.reject(new TypeError("Failed to fetch"))
    }) as typeof fetch

    await expect(apiFetch("/api/meta")).rejects.toThrow(
      FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE,
    )
    expect(calls).toBe(2)
  })

  test("friendlyBackendErrorMessage only rewrites temporary backend failures", () => {
    expect(friendlyBackendErrorMessage(502)).toBe(
      FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE,
    )
    expect(friendlyBackendErrorMessage(400)).toBe(null)
  })
})
