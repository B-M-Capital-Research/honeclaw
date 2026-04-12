import { createContext, createEffect, createMemo, createResource, useContext, type ParentProps } from "solid-js"
import { createStore } from "solid-js/store"
import {
  deleteCompanyProfile,
  getCompanyProfile,
  listCompanyProfiles,
} from "@/lib/api"
import { readStoredSelection, writeStoredSelection } from "@/lib/persist"
import type { CompanyProfileSummary } from "@/lib/types"
import { useBackend } from "./backend"

type CompanyProfilesContextValue = ReturnType<typeof createCompanyProfilesState>

const CompanyProfilesContext = createContext<CompanyProfilesContextValue>()

function createCompanyProfilesState() {
  const backend = useBackend()
  const storedSelection = readStoredSelection()
  const [state, setState] = createStore({
    currentProfileId: storedSelection.companyProfileId || "",
    deleting: false,
  })

  const [profiles, { refetch: refetchProfiles }] = createResource(
    () => backend.state.connected && backend.hasCapability("company_profiles"),
    async (enabled) => {
      if (!enabled) return [] as CompanyProfileSummary[]
      return listCompanyProfiles()
    },
  )

  const currentProfileId = createMemo(() => state.currentProfileId || undefined)

  const [currentProfile, { refetch: refetchCurrent }] = createResource(
    () => (backend.state.connected && backend.hasCapability("company_profiles") ? currentProfileId() : undefined),
    async (profileId) => (profileId ? getCompanyProfile(profileId) : undefined),
  )

  createEffect(() => {
    writeStoredSelection({
      ...readStoredSelection(),
      companyProfileId: state.currentProfileId,
    })
  })

  const selectProfile = (profileId?: string | null) => {
    setState("currentProfileId", profileId || "")
  }

  const removeProfile = async (profileId: string) => {
    setState("deleting", true)
    try {
      await deleteCompanyProfile(profileId)
      await refetchProfiles()
      if (state.currentProfileId === profileId) {
        setState("currentProfileId", "")
      }
    } finally {
      setState("deleting", false)
    }
  }

  return {
    state,
    profiles,
    currentProfile,
    currentProfileId,
    selectProfile,
    refetchProfiles,
    refetchCurrent,
    removeProfile,
  }
}

export function CompanyProfilesProvider(props: ParentProps) {
  const value = createCompanyProfilesState()
  return <CompanyProfilesContext.Provider value={value}>{props.children}</CompanyProfilesContext.Provider>
}

export function useCompanyProfiles() {
  const value = useContext(CompanyProfilesContext)
  if (!value) throw new Error("CompanyProfilesProvider missing")
  return value
}
