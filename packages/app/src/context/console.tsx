import { createContext, createEffect, createResource, createSignal, onCleanup, useContext, type ParentProps } from "solid-js"
import { createStore } from "solid-js/store"
import { getChannels } from "@/lib/api"
import {
  readStoredModule,
  readStoredReadAt,
  readStoredSelection,
  writeStoredModule,
  writeStoredReadAt,
  writeStoredSelection,
  type StoredModule,
} from "@/lib/persist"
import { useBackend } from "./backend"

type ConsoleContextValue = ReturnType<typeof createConsoleState>

const ConsoleContext = createContext<ConsoleContextValue>()

function createConsoleState() {
  const backend = useBackend()
  const selection = readStoredSelection()
  const [state, setState] = createStore({
    module: readStoredModule() as StoredModule,
    readAt: readStoredReadAt(),
    lastUserId: selection.userId,
    lastSkillId: selection.skillId,
  })

  const [channels, setChannels] = createSignal<Awaited<ReturnType<typeof getChannels>>>([])
  const [channelError, setChannelError] = createSignal("")

  const refreshChannels = async () => {
    if (!backend.state.connected || !backend.hasCapability("channels")) {
      setChannels([])
      setChannelError("")
      return
    }
    if (backend.state.isDesktop && !backend.state.resolvedBaseUrl) {
      setChannelError("desktop backend base URL 未就绪")
      return
    }
    try {
      setChannels(await getChannels())
      setChannelError("")
    } catch (error) {
      console.warn("refreshChannels failed", error)
      setChannelError(error instanceof Error ? error.message : String(error))
    }
  }

  createEffect(() => {
    if (!backend.state.connected) {
      setChannels([])
      setChannelError(backend.state.initializing ? "" : backend.state.error || "后端未连接")
      return
    }
    void refreshChannels()
  })

  const channelPoll = window.setInterval(() => {
    void refreshChannels()
  }, 5000)

  onCleanup(() => window.clearInterval(channelPoll))

  createEffect(() => writeStoredModule(state.module))
  createEffect(() => writeStoredReadAt(state.readAt))
  createEffect(() => writeStoredSelection({ userId: state.lastUserId, skillId: state.lastSkillId }))

  return {
    state,
    meta() {
      return backend.state.meta
    },
    channels,
    channelError,
    refreshChannels,
    setModule(module: StoredModule) {
      setState("module", module)
    },
    markRead(userId: string) {
      setState("readAt", userId, new Date().toISOString())
    },
    setLastUserId(userId?: string) {
      setState("lastUserId", userId)
    },
    setLastSkillId(skillId?: string) {
      setState("lastSkillId", skillId)
    },
  }
}

export function ConsoleProvider(props: ParentProps) {
  const value = createConsoleState()
  return <ConsoleContext.Provider value={value}>{props.children}</ConsoleContext.Provider>
}

export function useConsole() {
  const value = useContext(ConsoleContext)
  if (!value) {
    throw new Error("ConsoleProvider missing")
  }
  return value
}
