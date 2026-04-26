import type {
  AgentSettings,
  AgentSettingsUpdateResult,
  ChannelProcessCleanupResult,
  CliCheckResult,
  BackendConfig,
  BackendStatusInfo,
  DesktopChannelSettings,
  DesktopChannelSettingsInput,
  DesktopChannelSettingsUpdateResult,
  FmpSettings,
  MetaInfo,
  OpenRouterSettings,
  TavilySettings,
} from "./types"

export const SUPPORTED_API_VERSION = "desktop-v1"

type RuntimeMode = BackendConfig["mode"] | "browser"

type RuntimeState = {
  mode: RuntimeMode
  baseUrl: string
  bearerToken: string
  meta?: MetaInfo
  isDesktop: boolean
}

let runtimeState: RuntimeState = {
  mode: "browser",
  baseUrl: "",
  bearerToken: "",
  isDesktop: false,
}

export function defaultBackendConfig(): BackendConfig {
  return {
    mode: "bundled",
    baseUrl: "",
    bearerToken: "",
  }
}

export function isTauriRuntime() {
  if (typeof window === "undefined") return false
  const globalObject = globalThis as typeof globalThis & {
    isTauri?: boolean
    __TAURI__?: unknown
  }
  const windowObject = window as typeof window & {
    __TAURI_INTERNALS__?: unknown
    __TAURI__?: unknown
  }
  return Boolean(
    globalObject.isTauri ||
      windowObject.__TAURI_INTERNALS__ ||
      windowObject.__TAURI__ ||
      globalObject.__TAURI__,
  )
}

export async function detectTauriRuntime() {
  if (isTauriRuntime()) return true
  if (typeof window === "undefined") return false
  try {
    const { isTauri } = await import("@tauri-apps/api/core")
    return isTauri()
  } catch {
    return false
  }
}

export function supportsApiVersion(version: string) {
  return version === SUPPORTED_API_VERSION
}

export function normalizeBaseUrl(raw: string) {
  const trimmed = raw.trim()
  return trimmed.replace(/\/+$/, "")
}

export function resolveBaseUrl(config: Pick<BackendConfig, "mode" | "baseUrl">, fallbackBaseUrl?: string) {
  if (config.mode === "bundled") {
    return normalizeBaseUrl(fallbackBaseUrl ?? config.baseUrl)
  }
  return normalizeBaseUrl(config.baseUrl)
}

export function setBackendRuntime(next: Partial<RuntimeState>) {
  runtimeState = {
    ...runtimeState,
    ...next,
  }
}

export function getBackendRuntime() {
  return runtimeState
}

export function hasRuntimeCapability(capability: string) {
  if (!runtimeState.meta) {
    return runtimeState.mode === "browser" && capability === "local_file_proxy"
  }
  return runtimeState.meta?.capabilities.includes(capability) ?? false
}

export function buildApiUrl(path: string, baseUrl = runtimeState.baseUrl) {
  const normalizedBaseUrl = normalizeBaseUrl(baseUrl)
  if (normalizedBaseUrl) {
    return new URL(path, `${normalizedBaseUrl}/`).toString()
  }
  if (typeof window !== "undefined") {
    const origin = window.location.origin
    const fallbackOrigin =
      origin && origin !== "null" && origin !== "undefined" ? origin : "http://localhost"
    return new URL(path, fallbackOrigin).toString()
  }
  return path
}

export function buildAuthHeaders(headers?: HeadersInit, bearerToken = runtimeState.bearerToken) {
  const next = new Headers(headers)
  if (bearerToken) {
    next.set("Authorization", `Bearer ${bearerToken}`)
  }
  return next
}

export async function apiFetch(path: string, init: RequestInit = {}) {
  return fetch(buildApiUrl(path), {
    credentials: "include",
    ...init,
    headers: buildAuthHeaders(init.headers),
  })
}

async function parseJson<T>(response: Response): Promise<T> {
  if (!response.ok) {
    throw new Error(await response.text())
  }
  return response.json() as Promise<T>
}

export async function probeBackendMeta(config: Pick<BackendConfig, "mode" | "baseUrl" | "bearerToken">, fallbackBaseUrl?: string) {
  const baseUrl = resolveBaseUrl(config, fallbackBaseUrl)
  const response = await fetch(buildApiUrl("/api/meta", baseUrl), {
    headers: buildAuthHeaders(undefined, config.bearerToken),
  })
  return parseJson<MetaInfo>(response)
}

export async function createEventSource(path: string) {
  if (!runtimeState.bearerToken) {
    return new EventSource(buildApiUrl(path))
  }

  const ticketResponse = await apiFetch("/api/auth/sse-ticket", { method: "POST" })
  const ticket = await parseJson<{ ticket: string }>(ticketResponse)
  const url = new URL(buildApiUrl(path), window.location.origin)
  url.searchParams.set("sse_ticket", ticket.ticket)
  return new EventSource(url.toString())
}

export async function invokeDesktop<T>(command: string, args?: Record<string, unknown>) {
  const { invoke } = await import("@tauri-apps/api/core")
  return invoke<T>(command, args)
}

export async function loadDesktopBackendStatus() {
  return invokeDesktop<BackendStatusInfo>("backend_status")
}

export async function saveDesktopBackendConfig(config: BackendConfig) {
  return invokeDesktop<void>("set_backend_config", { config })
}

export async function connectDesktopBackend() {
  return invokeDesktop<BackendStatusInfo>("connect_backend")
}

export async function stopDesktopBundledBackend() {
  return invokeDesktop<BackendStatusInfo>("stop_bundled_backend")
}

export async function loadDesktopChannelSettings() {
  return invokeDesktop<DesktopChannelSettings>("get_channel_settings")
}

export async function saveDesktopChannelSettings(settings: DesktopChannelSettingsInput) {
  return invokeDesktop<DesktopChannelSettingsUpdateResult>("set_channel_settings", { settings })
}

export async function cleanupDesktopChannelProcesses() {
  return invokeDesktop<ChannelProcessCleanupResult>("cleanup_channel_processes")
}

// ── Agent 基础设置 ─────────────────────────────────────────────────────────

export async function loadDesktopAgentSettings() {
  return invokeDesktop<AgentSettings>("get_agent_settings")
}

export async function saveDesktopAgentSettings(settings: AgentSettings) {
  return invokeDesktop<AgentSettingsUpdateResult>("set_agent_settings", { settings })
}

/** 检测本地 CLI/ACP runner 是否可用（运行 --version，超时 8s） */
export async function checkDesktopAgentCli(
  runner: "gemini_cli" | "gemini_acp" | "codex_cli" | "codex_acp" | "opencode_acp" | "multi-agent",
) {
  return invokeDesktop<CliCheckResult>("check_agent_cli", { runner })
}

/** 测试 OpenAI 协议渠道连通性（发送最小请求到 {url}/chat/completions） */
export async function testDesktopOpenAiChannel(url: string, model: string, apiKey: string) {
  return invokeDesktop<CliCheckResult>("test_openai_channel", { url, model, apiKey })
}

// ── OpenRouter API Key 设置 ─────────────────────────────────────────────────

/** 读取运行时覆盖层中的 OpenRouter API Key */
export async function loadDesktopOpenRouterSettings() {
  return invokeDesktop<OpenRouterSettings>("get_openrouter_settings")
}

/** 保存 OpenRouter API Keys 到运行时覆盖层，内置后端模式下立即重启生效 */
export async function saveDesktopOpenRouterSettings(settings: OpenRouterSettings) {
  return invokeDesktop<void>("set_openrouter_settings", { settings })
}

// ── FMP API Key 设置 ────────────────────────────────────────────────────────

/** 读取运行时覆盖层中的 FMP API Keys */
export async function loadDesktopFmpSettings() {
  return invokeDesktop<FmpSettings>("get_fmp_settings")
}

/** 保存 FMP API Keys 到运行时覆盖层，内置后端模式下立即重启生效 */
export async function saveDesktopFmpSettings(settings: FmpSettings) {
  return invokeDesktop<void>("set_fmp_settings", { settings })
}

// ── Tavily API Key 设置 ─────────────────────────────────────────────────────

/** 读取运行时覆盖层中的 Tavily API Keys */
export async function loadDesktopTavilySettings() {
  return invokeDesktop<TavilySettings>("get_tavily_settings")
}

/** 保存 Tavily API Keys 到运行时覆盖层，内置后端模式下立即重启生效 */
export async function saveDesktopTavilySettings(settings: TavilySettings) {
  return invokeDesktop<void>("set_tavily_settings", { settings })
}
