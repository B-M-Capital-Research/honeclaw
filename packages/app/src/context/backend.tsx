import { Show, createContext, createEffect, onCleanup, onMount, useContext, type ParentProps } from "solid-js"
import { createStore } from "solid-js/store"
import {
  connectDesktopBackend,
  detectTauriRuntime,
  defaultBackendConfig,
  isTauriRuntime,
  loadDesktopBackendStatus,
  loadDesktopChannelSettings,
  saveDesktopAgentSettings,
  probeBackendMeta,
  resolveBaseUrl,
  saveDesktopBackendConfig,
  saveDesktopChannelSettings,
  setBackendRuntime,
  stopDesktopBundledBackend,
  supportsApiVersion,
} from "@/lib/backend"
import { hasLocaleOverride, setLocale } from "@/lib/i18n"
import { putLanguage } from "@/lib/api"
import type {
  AgentSettings,
  AgentSettingsUpdateResult,
  BackendConfig,
  BackendStatusInfo,
  DesktopChannelSettingsInput,
  DesktopChannelSettingsUpdateResult,
  MetaInfo,
} from "@/lib/types"

let _localeBootstrapped = false

/**
 * Seed the global locale signal from the backend's configured `language`,
 * but only on the very first successful meta fetch and only if the operator
 * hasn't already pinned a per-device override via the sidebar switcher.
 */
function applyMetaLocaleBootstrap(meta?: MetaInfo): void {
  if (_localeBootstrapped) return
  if (!meta) return
  const lang = meta.language
  if (lang !== "zh" && lang !== "en") return
  _localeBootstrapped = true
  if (hasLocaleOverride()) return
  setLocale(lang)
}

type LegacyMetaInfo = MetaInfo & {
  api_version?: string
  deployment_mode?: MetaInfo["deploymentMode"]
  supports_imessage?: boolean
}

type LegacyBackendStatusInfo = BackendStatusInfo & {
  resolved_base_url?: string
  last_error?: string
  meta?: LegacyMetaInfo
  diagnostics?: BackendStatusInfo["diagnostics"] & {
    config_dir?: string
    data_dir?: string
    logs_dir?: string
    desktop_log?: string
    sidecar_log?: string
  }
}

type BackendContextValue = ReturnType<typeof createBackendState>

const BackendContext = createContext<BackendContextValue>()

const DESKTOP_RUNTIME_DETECT_TIMEOUT_MS = 3000
const DESKTOP_STATUS_TIMEOUT_MS = 4000
const DESKTOP_CONNECT_TIMEOUT_MS = 12000

async function withTimeout<T>(promise: Promise<T>, timeoutMs: number, label: string): Promise<T> {
  let timer: number | undefined
  try {
    return await Promise.race([
      promise,
      new Promise<T>((_, reject) => {
        timer = window.setTimeout(() => {
          reject(new Error(`${label} 超时（>${timeoutMs}ms）`))
        }, timeoutMs)
      }),
    ])
  } finally {
    if (timer) {
      window.clearTimeout(timer)
    }
  }
}

function createBackendState() {
  const [state, setState] = createStore({
    isDesktop: isTauriRuntime(),
    initializing: true,
    connected: false,
    compatible: true,
    saving: false,
    resolvedBaseUrl: "",
    config: defaultBackendConfig() as BackendConfig,
    meta: undefined as MetaInfo | undefined,
    diagnostics: undefined as BackendStatusInfo["diagnostics"],
    error: "",
  })

  const applyConnection = (input: {
    config: BackendConfig
    resolvedBaseUrl: string
    meta?: MetaInfo
    diagnostics?: BackendStatusInfo["diagnostics"]
    connected: boolean
    error?: string
  }) => {
    applyMetaLocaleBootstrap(input.meta)
    const compatible = input.meta ? supportsApiVersion(input.meta.apiVersion) : true
    const error =
      input.error ||
      (input.meta && !compatible
        ? `不支持的后端 API 版本：${input.meta.apiVersion}（当前仅支持 ${SUPPORTED_VERSION_LABEL}）`
        : "")

    setBackendRuntime({
      mode: state.isDesktop ? input.config.mode : "browser",
      baseUrl: input.resolvedBaseUrl,
      bearerToken: input.connected && compatible ? input.config.bearerToken : "",
      meta: compatible ? input.meta : undefined,
      isDesktop: state.isDesktop,
    })

    setState({
      config: input.config,
      resolvedBaseUrl: input.resolvedBaseUrl,
      meta: compatible ? input.meta : undefined,
      diagnostics: input.diagnostics,
      connected: input.connected && compatible,
      compatible,
      error,
    })
  }

  const normalizeMeta = (meta?: LegacyMetaInfo): MetaInfo | undefined => {
    if (!meta) return undefined
    return {
      ...meta,
      apiVersion: meta.apiVersion ?? meta.api_version ?? "",
      deploymentMode: meta.deploymentMode ?? meta.deployment_mode ?? "local",
      supportsImessage: meta.supportsImessage ?? meta.supports_imessage ?? false,
    }
  }

  const normalizeDiagnostics = (diagnostics?: LegacyBackendStatusInfo["diagnostics"]) => {
    if (!diagnostics) return undefined
    return {
      ...diagnostics,
      configDir: diagnostics.configDir ?? diagnostics.config_dir ?? "",
      dataDir: diagnostics.dataDir ?? diagnostics.data_dir ?? "",
      logsDir: diagnostics.logsDir ?? diagnostics.logs_dir ?? "",
      desktopLog: diagnostics.desktopLog ?? diagnostics.desktop_log ?? "",
      sidecarLog: diagnostics.sidecarLog ?? diagnostics.sidecar_log ?? "",
    }
  }

  const applyDesktopStatus = (status: LegacyBackendStatusInfo) => {
    const meta = normalizeMeta(status.meta)
    applyConnection({
      config: status.config,
      resolvedBaseUrl: status.resolvedBaseUrl ?? status.resolved_base_url ?? resolveBaseUrl(status.config),
      meta,
      diagnostics: normalizeDiagnostics(status.diagnostics),
      connected: status.connected,
      error: status.lastError ?? status.last_error,
    })
  }

  const applyDesktopStatusWithRemoteFallback = async (status: LegacyBackendStatusInfo) => {
    const resolvedBaseUrl =
      status.resolvedBaseUrl ?? status.resolved_base_url ?? resolveBaseUrl(status.config)

    if (status.connected || status.config.mode !== "remote" || !resolvedBaseUrl) {
      applyDesktopStatus(status)
      return
    }

    try {
      const meta = normalizeMeta(await probeBackendMeta(status.config, resolvedBaseUrl) as LegacyMetaInfo)
      applyConnection({
        config: status.config,
        resolvedBaseUrl,
        meta,
        diagnostics: normalizeDiagnostics(status.diagnostics),
        connected: true,
      })
    } catch {
      applyDesktopStatus(status)
    }
  }

  const initBrowser = async () => {
    const config = defaultBackendConfig()
    try {
      const meta = normalizeMeta(await probeBackendMeta(config) as LegacyMetaInfo)
      applyConnection({
        config,
        resolvedBaseUrl: "",
        meta,
        connected: true,
      })
    } catch (error) {
      applyConnection({
        config,
        resolvedBaseUrl: "",
        connected: false,
        error: error instanceof Error ? error.message : String(error),
      })
    } finally {
      setState("initializing", false)
    }
  }

  const loadDesktopBackendStatusSafe = async () =>
    withTimeout(loadDesktopBackendStatus(), DESKTOP_STATUS_TIMEOUT_MS, "读取 desktop backend 状态")

  const connectDesktopBackendSafe = async () =>
    withTimeout(connectDesktopBackend(), DESKTOP_CONNECT_TIMEOUT_MS, "连接 desktop backend")

  const initDesktop = async () => {
    try {
      const initial = await loadDesktopBackendStatusSafe()
      if (initial.connected) {
        await applyDesktopStatusWithRemoteFallback(initial)
      } else {
        try {
          const connected = await connectDesktopBackendSafe()
          await applyDesktopStatusWithRemoteFallback(connected)
        } catch {
          const refreshed = await loadDesktopBackendStatusSafe()
          await applyDesktopStatusWithRemoteFallback(refreshed)
        }
      }
    } catch (error) {
      applyConnection({
        config: state.config,
        resolvedBaseUrl: "",
        connected: false,
        error: error instanceof Error ? error.message : String(error),
      })
    } finally {
      setState("initializing", false)
    }
  }

  onMount(() => {
    void (async () => {
      let desktop = false
      try {
        desktop = await withTimeout(
          detectTauriRuntime(),
          DESKTOP_RUNTIME_DETECT_TIMEOUT_MS,
          "检测 desktop runtime",
        )
      } catch {
        desktop = isTauriRuntime()
      }
      setState("isDesktop", desktop)
      if (desktop) {
        await initDesktop()
        return
      }
      await initBrowser()
    })()
  })

  createEffect(() => {
    if (!state.isDesktop || state.initializing || state.connected || state.saving) return
    const timer = window.setInterval(() => {
      void (async () => {
        try {
          const status = await connectDesktopBackendSafe()
          await applyDesktopStatusWithRemoteFallback(status)
        } catch {
          // Keep the existing error message; this retry is only for transient startup races.
        }
      })()
    }, 1200)
    onCleanup(() => window.clearInterval(timer))
  })

  return {
    state,
    hasCapability(capability: string) {
      return state.meta?.capabilities.includes(capability) ?? false
    },
    isRemote() {
      return state.config.mode === "remote"
    },
    async reconnect() {
      setState("saving", true)
      try {
        if (state.isDesktop) {
          const status = await connectDesktopBackendSafe()
          await applyDesktopStatusWithRemoteFallback(status)
        } else {
          await initBrowser()
        }
      } finally {
        setState("saving", false)
      }
    },
    async saveConfig(config: BackendConfig) {
      if (!state.isDesktop) return
      setState("saving", true)
      try {
        await saveDesktopBackendConfig(config)
        const status = await connectDesktopBackendSafe()
        await applyDesktopStatusWithRemoteFallback(status)
      } finally {
        setState("saving", false)
      }
    },
    async loadChannelSettings() {
      if (!state.isDesktop) {
        throw new Error("desktop runtime unavailable")
      }
      return loadDesktopChannelSettings()
    },
    async saveAgentSettings(settings: AgentSettings): Promise<AgentSettingsUpdateResult> {
      if (!state.isDesktop) {
        throw new Error("desktop runtime unavailable")
      }
      setState("saving", true)
      try {
        const result = await saveDesktopAgentSettings(settings)
        if (result.backendStatus) {
          await applyDesktopStatusWithRemoteFallback(result.backendStatus)
        }
        return result
      } finally {
        setState("saving", false)
      }
    },
    async saveLanguage(next: "zh" | "en"): Promise<"zh" | "en"> {
      const stored = await putLanguage(next)
      setLocale(stored)
      setState("meta", (prev) => (prev ? { ...prev, language: stored } : prev))
      return stored
    },
    async saveChannelSettings(settings: DesktopChannelSettingsInput): Promise<DesktopChannelSettingsUpdateResult> {
      if (!state.isDesktop) {
        throw new Error("desktop runtime unavailable")
      }
      setState("saving", true)
      try {
        const result = await saveDesktopChannelSettings(settings)
        if (result.backendStatus) {
          await applyDesktopStatusWithRemoteFallback(result.backendStatus)
        }
        return result
      } finally {
        setState("saving", false)
      }
    },
    async stopBundledBackend() {
      if (!state.isDesktop) return
      const status = await stopDesktopBundledBackend()
      applyDesktopStatus(status)
    },
  }
}

const SUPPORTED_VERSION_LABEL = "desktop-v1"

function InitializingScreen() {
  return (
    <div class="flex h-screen items-center justify-center gap-3 text-sm text-[color:var(--text-secondary)]">
      <svg
        class="h-4 w-4 animate-spin"
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
      >
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
        <path
          class="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 22 6.477 22 12h-4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
        />
      </svg>
      正在连接后端…
    </div>
  )
}

export function BackendProvider(props: ParentProps) {
  const value = createBackendState()
  return (
    <BackendContext.Provider value={value}>
      <Show when={!value.state.initializing} fallback={<InitializingScreen />}>
        {props.children}
      </Show>
    </BackendContext.Provider>
  )
}

export function useBackend() {
  const value = useContext(BackendContext)
  if (!value) {
    throw new Error("BackendProvider missing")
  }
  return value
}
