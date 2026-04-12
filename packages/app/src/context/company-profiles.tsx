import { createContext, createEffect, createMemo, createResource, useContext, type ParentProps } from "solid-js"
import { createStore } from "solid-js/store"
import {
  deleteCompanyProfile,
  getCompanyProfile,
  listCompanyProfileActors,
  listCompanyProfiles,
} from "@/lib/api"
import { actorKey, parseActorKey, type ActorRef } from "@/lib/actors"
import { readStoredSelection, writeStoredSelection } from "@/lib/persist"
import type { CompanyProfileSpaceSummary, CompanyProfileSummary } from "@/lib/types"
import { useBackend } from "./backend"

type CompanyProfilesContextValue = ReturnType<typeof createCompanyProfilesState>

const CompanyProfilesContext = createContext<CompanyProfilesContextValue>()

function createCompanyProfilesState() {
  const backend = useBackend()
  const storedSelection = readStoredSelection()
  const [state, setState] = createStore({
    currentActorKey: storedSelection.companyProfileActorKey || "",
    currentProfileId: storedSelection.companyProfileId || "",
    deleting: false,
  })

  const currentActor = createMemo(() => parseActorKey(state.currentActorKey))
  const currentProfileId = createMemo(() => state.currentProfileId || undefined)

  const [actorsList, { refetch: refetchActors }] = createResource(
    () => backend.state.connected && backend.hasCapability("company_profiles"),
    async (enabled) => {
      if (!enabled) return [] as CompanyProfileSpaceSummary[]
      try {
        return await listCompanyProfileActors()
      } catch {
        return [] as CompanyProfileSpaceSummary[]
      }
    },
  )

  const [profiles, { refetch: refetchProfiles }] = createResource(
    () => (backend.state.connected && backend.hasCapability("company_profiles") ? currentActor() : undefined),
    async (actor) => (actor ? listCompanyProfiles(actor) : [] as CompanyProfileSummary[]),
  )

  const [currentProfile, { refetch: refetchCurrent }] = createResource(
    () => {
      const actor = currentActor()
      const profileId = currentProfileId()
      if (!backend.state.connected || !backend.hasCapability("company_profiles") || !actor || !profileId) {
        return undefined
      }
      return {
        actor,
        profileId,
      }
    },
    async (source) => getCompanyProfile(source.profileId, source.actor),
  )

  createEffect(() => {
    writeStoredSelection({
      ...readStoredSelection(),
      companyProfileActorKey: state.currentActorKey,
      companyProfileId: state.currentProfileId,
    })
  })

  createEffect(() => {
    const actor = currentActor()
    const currentId = state.currentProfileId
    const availableProfiles = profiles()
    if (!actor || !currentId || !availableProfiles) return
    if (!availableProfiles.some((profile) => profile.profile_id === currentId)) {
      setState("currentProfileId", "")
    }
  })

  const selectActor = (actor?: ActorRef | null) => {
    const nextKey = actor ? actorKey(actor) : ""
    if (nextKey !== state.currentActorKey) {
      setState("currentActorKey", nextKey)
      setState("currentProfileId", "")
      return
    }
    setState("currentActorKey", nextKey)
  }

  const selectProfile = (profileId?: string | null) => {
    setState("currentProfileId", profileId || "")
  }

  const removeProfile = async (profileId: string) => {
    const actor = currentActor()
    if (!actor) return

    setState("deleting", true)
    try {
      await deleteCompanyProfile(profileId, actor)
      await refetchProfiles()
      await refetchActors()
      if (state.currentProfileId === profileId) {
        setState("currentProfileId", "")
      }
    } finally {
      setState("deleting", false)
    }
  }

  return {
    state,
    actorsList,
    currentActor,
    profiles,
    currentProfile,
    currentProfileId,
    selectActor,
    selectProfile,
    refetchActors,
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
