import { Show, createContext, onMount, useContext, type ParentProps } from "solid-js"
import { createStore } from "solid-js/store"
import {
  connectDesktopBackend,
  defaultBackendConfig,
  isTauriRuntime,
  loadDesktopChannelSettings,
  loadDesktopBackendStatus,
  probeBackendMeta,
  resolveBaseUrl,
  saveDesktopBackendConfig,
  saveDesktopChannelSettings,
  setBackendRuntime,
  stopDesktopBundledBackend,
  supportsApiVersion,
} from "@/lib/backend"
import type {
  BackendConfig,
  BackendStatusInfo,
  DesktopChannelSettingsInput,
  DesktopChannelSettingsUpdateResult,
  MetaInfo,
} from "@/lib/types"

type BackendContextValue = ReturnType<typeof createBackendState>

const BackendContext = createContext<BackendContextValue>()

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
    const compatible = input.meta ? supportsApiVersion(input.meta.api_version) : true
    const error =
      input.error ||
      (input.meta && !compatible
        ? `不支持的后端 API 版本：${input.meta.api_version}（当前仅支持 ${SUPPORTED_VERSION_LABEL}）`
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

  const applyDesktopStatus = (status: BackendStatusInfo) => {
    applyConnection({
      config: status.config,
      resolvedBaseUrl: status.resolved_base_url || resolveBaseUrl(status.config),
      meta: status.meta,
      diagnostics: status.diagnostics,
      connected: status.connected,
      error: status.last_error,
    })
  }

  const initBrowser = async () => {
    const config = defaultBackendConfig()
    try {
      const meta = await probeBackendMeta(config)
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

  const initDesktop = async () => {
    try {
      const initial = await loadDesktopBackendStatus()
      if (initial.connected) {
        applyDesktopStatus(initial)
      } else {
        const connected = await connectDesktopBackend()
        applyDesktopStatus(connected)
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
    if (state.isDesktop) {
      void initDesktop()
      return
    }
    void initBrowser()
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
          const status = await connectDesktopBackend()
          applyDesktopStatus(status)
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
        const status = await connectDesktopBackend()
        applyDesktopStatus(status)
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
    async saveChannelSettings(settings: DesktopChannelSettingsInput): Promise<DesktopChannelSettingsUpdateResult> {
      if (!state.isDesktop) {
        throw new Error("desktop runtime unavailable")
      }
      setState("saving", true)
      try {
        const result = await saveDesktopChannelSettings(settings)
        if (result.backendStatus) {
          applyDesktopStatus(result.backendStatus)
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
