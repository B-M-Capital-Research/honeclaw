import { createContext, createEffect, useContext, type ParentProps } from "solid-js"
import { createStore } from "solid-js/store"
import { getSkill, getSkills, resetSkillRegistry, updateSkillState } from "@/lib/api"
import type { SkillDetailInfo, SkillInfo } from "@/lib/types"
import { useConsole } from "./console"
import { useBackend } from "./backend"

type SkillsContextValue = ReturnType<typeof createSkillsState>

const SkillsContext = createContext<SkillsContextValue>()

function createSkillsState() {
  const backend = useBackend()
  const consoleState = useConsole()
  const [state, setState] = createStore({
    skills: [] as SkillInfo[],
    detailById: {} as Record<string, SkillDetailInfo>,
    loading: false,
    error: "",
    query: "",
    sourceFilter: "all" as "all" | "system" | "custom" | "dynamic",
    statusFilter: "all" as "all" | "enabled" | "disabled",
    currentSkillId: consoleState.state.lastSkillId ?? "",
    updatingSkillId: "",
    resetting: false,
  })

  const applySkillSummary = (skill: SkillInfo) => {
    const index = state.skills.findIndex((item) => item.id === skill.id)
    if (index >= 0) {
      setState("skills", index, skill)
    }
    if (state.detailById[skill.id]) {
      setState("detailById", skill.id, "summary", skill)
    }
  }

  const refresh = async () => {
    if (!backend.state.connected || !backend.hasCapability("skills")) {
      setState("skills", [])
      return
    }
    setState("loading", true)
    try {
      const skills = await getSkills()
      setState("skills", skills)
      for (const skill of skills) {
        if (state.detailById[skill.id]) {
          setState("detailById", skill.id, "summary", skill)
        }
      }
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

  createEffect(() => {
    const skillId = state.currentSkillId
    if (skillId) {
      void getSkill(skillId)
        .then((detail) => setState("detailById", skillId, detail))
        .catch((error) =>
          setState("error", error instanceof Error ? error.message : String(error)),
        )
    }
  })

    return {
      state,
    filteredSkills() {
      const query = state.query.trim().toLowerCase()
      return state.skills.filter((skill) => {
        if (state.sourceFilter !== "all" && skill.loaded_from !== state.sourceFilter) {
          return false
        }
        if (state.statusFilter === "enabled" && !skill.enabled) {
          return false
        }
        if (state.statusFilter === "disabled" && skill.enabled) {
          return false
        }
        if (!query) {
          return true
        }
        const haystack = [
          skill.id,
          skill.display_name,
          skill.description,
          skill.when_to_use ?? "",
          ...skill.aliases,
        ]
          .join("\n")
          .toLowerCase()
        return haystack.includes(query)
      })
    },
    counts() {
      return {
        total: state.skills.length,
        enabled: state.skills.filter((skill) => skill.enabled).length,
        disabled: state.skills.filter((skill) => !skill.enabled).length,
        invocable: state.skills.filter((skill) => skill.user_invocable && skill.enabled).length,
      }
    },
    async refresh() {
      await refresh()
    },
    setQuery(query: string) {
      setState("query", query)
    },
    setSourceFilter(value: "all" | "system" | "custom" | "dynamic") {
      setState("sourceFilter", value)
    },
    setStatusFilter(value: "all" | "enabled" | "disabled") {
      setState("statusFilter", value)
    },
    selectSkill(skillId?: string) {
      setState("currentSkillId", skillId ?? "")
      consoleState.setLastSkillId(skillId)
    },
    currentSkill() {
      return state.detailById[state.currentSkillId]
    },
    async ensureSkillDetail(skillId?: string) {
      const id = skillId ?? state.currentSkillId
      if (!id || state.detailById[id]) {
        return state.detailById[id]
      }
      const detail = await getSkill(id)
      setState("detailById", id, detail)
      return detail
    },
    async toggleSkill(skillId: string, enabled: boolean) {
      setState("updatingSkillId", skillId)
      try {
        const skill = await updateSkillState(skillId, enabled)
        applySkillSummary(skill)
        setState("error", "")
        return skill
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error)
        setState("error", message)
        throw error
      } finally {
        setState("updatingSkillId", "")
      }
    },
    async resetRegistry() {
      setState("resetting", true)
      try {
        const skills = await resetSkillRegistry()
        setState("skills", skills)
        for (const skill of skills) {
          if (state.detailById[skill.id]) {
            setState("detailById", skill.id, "summary", skill)
          }
        }
        setState("error", "")
      } catch (error) {
        setState("error", error instanceof Error ? error.message : String(error))
        throw error
      } finally {
        setState("resetting", false)
      }
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
