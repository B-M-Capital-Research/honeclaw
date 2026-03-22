import { createContext, createEffect, useContext, type ParentProps } from "solid-js"
import { createStore } from "solid-js/store"
import { getSkills } from "@/lib/api"
import type { SkillInfo } from "@/lib/types"
import { useConsole } from "./console"
import { useBackend } from "./backend"

type SkillsContextValue = ReturnType<typeof createSkillsState>

const SkillsContext = createContext<SkillsContextValue>()

function createSkillsState() {
  const backend = useBackend()
  const consoleState = useConsole()
  const [state, setState] = createStore({
    skills: [] as SkillInfo[],
    loading: false,
    error: "",
    currentSkillId: consoleState.state.lastSkillId ?? "",
  })

  const refresh = async () => {
    if (!backend.state.connected || !backend.hasCapability("skills")) {
      setState("skills", [])
      return
    }
    setState("loading", true)
    try {
      setState("skills", await getSkills())
      setState("error", "")
    } catch (error) {
      setState("error", error instanceof Error ? error.message : String(error))
    } finally {
      setState("loading", false)
    }
  }

  createEffect(() => {
    if (backend.state.connected) {
      void refresh()
    }
  })

  return {
    state,
    async refresh() {
      await refresh()
    },
    selectSkill(skillId?: string) {
      setState("currentSkillId", skillId ?? "")
      consoleState.setLastSkillId(skillId)
    },
    currentSkill() {
      return state.skills.find((item) => item.id === state.currentSkillId)
    },
  }
}

export function SkillsProvider(props: ParentProps) {
  const value = createSkillsState()
  return <SkillsContext.Provider value={value}>{props.children}</SkillsContext.Provider>
}

export function useSkills() {
  const value = useContext(SkillsContext)
  if (!value) {
    throw new Error("SkillsProvider missing")
  }
  return value
}
