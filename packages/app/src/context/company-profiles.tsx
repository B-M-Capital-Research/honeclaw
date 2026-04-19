import { useToast } from "@hone-financial/ui/context/toast"
import {
  createContext,
  createEffect,
  createMemo,
  createResource,
  useContext,
  type ParentProps,
} from "solid-js"
import { createStore } from "solid-js/store"
import {
  applyImportCompanyProfiles,
  deleteCompanyProfile,
  exportCompanyProfiles,
  getCompanyProfile,
  getUsers,
  listCompanyProfileActors,
  listCompanyProfiles,
  previewImportCompanyProfiles,
} from "@/lib/api"
import {
  buildCompanyProfileImportApplyRequest,
  isCompanyProfileImportReady,
  shouldCreateCompanyProfileBackup,
} from "@/lib/company-profile-transfer"
import { actorFromUser, actorKey, parseActorKey, type ActorRef } from "@/lib/actors"
import { readStoredSelection, writeStoredSelection } from "@/lib/persist"
import type {
  CompanyProfileConflictDecision,
  CompanyProfileImportApplyResult,
  CompanyProfileImportPreview,
  CompanyProfileSpaceSummary,
  CompanyProfileSummary,
  UserInfo,
} from "@/lib/types"
import { useBackend } from "./backend"

type CompanyProfilesContextValue = ReturnType<typeof createCompanyProfilesState>

type TargetOption = {
  actor: ActorRef
  key: string
  label: string
  description: string
  source: "space" | "session" | "manual"
  profileCount?: number
  updatedAt?: string
  sessionLastTime?: string
}

const CompanyProfilesContext = createContext<CompanyProfilesContextValue>()

function triggerBlobDownload(blob: Blob, fileName: string) {
  const objectUrl = URL.createObjectURL(blob)
  const anchor = document.createElement("a")
  anchor.href = objectUrl
  anchor.download = fileName
  anchor.click()
  window.setTimeout(() => URL.revokeObjectURL(objectUrl), 0)
}

function actorFromSummary(summary: CompanyProfileSpaceSummary): ActorRef {
  return {
    channel: summary.channel,
    user_id: summary.user_id,
    channel_scope: summary.channel_scope,
  }
}

function formatTargetLabel(actor: ActorRef) {
  return actor.channel_scope
    ? `${actor.channel} / ${actor.user_id} · ${actor.channel_scope}`
    : `${actor.channel} / ${actor.user_id}`
}

function createCompanyProfilesState() {
  const backend = useBackend()
  const toast = useToast()
  const storedSelection = readStoredSelection()
  const [state, setState] = createStore({
    currentActorKey: storedSelection.companyProfileActorKey || "",
    currentProfileId: storedSelection.companyProfileId || "",
    deleting: false,
    exporting: false,
    highlightedProfileIds: [] as string[],
    manualTargetOpen: false,
    manualTargetChannel: "",
    manualTargetUserId: "",
    manualTargetScope: "",
    transfer: {
      importFile: undefined as File | undefined,
      preview: undefined as CompanyProfileImportPreview | undefined,
      decisions: {} as Record<string, CompanyProfileConflictDecision>,
      previewing: false,
      applying: false,
      backupBlob: undefined as Blob | undefined,
      backupFileName: "",
      lastResult: undefined as CompanyProfileImportApplyResult | undefined,
      error: "",
    },
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

  const [usersList, { refetch: refetchUsers }] = createResource(
    () => backend.state.connected && backend.hasCapability("users"),
    async (enabled) => {
      if (!enabled) return [] as UserInfo[]
      try {
        return await getUsers()
      } catch {
        return [] as UserInfo[]
      }
    },
  )

  const [profiles, { refetch: refetchProfiles }] = createResource(
    () =>
      backend.state.connected && backend.hasCapability("company_profiles")
        ? currentActor()
        : undefined,
    async (actor) => (actor ? listCompanyProfiles(actor) : [] as CompanyProfileSummary[]),
  )

  const [currentProfile, { refetch: refetchCurrent }] = createResource(
    () => {
      const actor = currentActor()
      const profileId = currentProfileId()
      if (
        !backend.state.connected ||
        !backend.hasCapability("company_profiles") ||
        !actor ||
        !profileId
      ) {
        return undefined
      }
      return {
        actor,
        profileId,
      }
    },
    async (source) => getCompanyProfile(source.profileId, source.actor),
  )

  const profileSpaceTargets = createMemo(() =>
    (actorsList() ?? []).map((summary) => {
      const actor = actorFromSummary(summary)
      return {
        actor,
        key: actorKey(actor),
        label: formatTargetLabel(actor),
        description: `${summary.profile_count} 份画像`,
        source: "space" as const,
        profileCount: summary.profile_count,
        updatedAt: summary.updated_at,
      }
    }),
  )

  const sessionTargets = createMemo(() => {
    const seen = new Set(profileSpaceTargets().map((item) => item.key))
    const entries = new Map<string, TargetOption>()
    for (const user of usersList() ?? []) {
      const actor = actorFromUser(user)
      const key = actorKey(actor)
      if (seen.has(key) || entries.has(key)) continue
      entries.set(key, {
        actor,
        key,
        label: formatTargetLabel(actor),
        description: user.session_kind === "group" ? user.session_label : "已有会话",
        source: "session",
        sessionLastTime: user.last_time,
      })
    }
    return Array.from(entries.values()).sort((left, right) =>
      (right.sessionLastTime ?? "").localeCompare(left.sessionLastTime ?? ""),
    )
  })

  const manualTarget = createMemo(() => {
    const actor = currentActor()
    if (!actor) return undefined
    const key = actorKey(actor)
    const exists =
      profileSpaceTargets().some((target) => target.key === key) ||
      sessionTargets().some((target) => target.key === key)
    if (exists) return undefined
    return {
      actor,
      key,
      label: formatTargetLabel(actor),
      description: "手动指定目标",
      source: "manual" as const,
    }
  })

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

  createEffect(() => {
    const actor = currentActor()
    const availableProfiles = profiles()
    if (!actor || !availableProfiles || availableProfiles.length === 0) return
    if (state.currentProfileId) return
    setState("currentProfileId", availableProfiles[0]?.profile_id ?? "")
  })

  const resetTransfer = () => {
    setState("transfer", {
      importFile: undefined,
      preview: undefined,
      decisions: {},
      previewing: false,
      applying: false,
      backupBlob: undefined,
      backupFileName: "",
      lastResult: undefined,
      error: "",
    })
  }

  const selectActor = (actor?: ActorRef | null) => {
    const nextKey = actor ? actorKey(actor) : ""
    if (nextKey !== state.currentActorKey) {
      setState("currentActorKey", nextKey)
      setState("currentProfileId", "")
      setState("highlightedProfileIds", [])
      resetTransfer()
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
      toast.show("公司画像已删除")
    } finally {
      setState("deleting", false)
    }
  }

  const exportCurrentSpace = async () => {
    const actor = currentActor()
    if (!actor) return
    setState("exporting", true)
    try {
      const payload = await exportCompanyProfiles(actor)
      triggerBlobDownload(payload.blob, payload.fileName)
      toast.show("画像包已导出", formatTargetLabel(actor))
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setState("transfer", "error", message)
      throw error
    } finally {
      setState("exporting", false)
    }
  }

  const previewImport = async (file: File) => {
    const actor = currentActor()
    if (!actor) return
    setState("transfer", "previewing", true)
    setState("transfer", "importFile", file)
    setState("transfer", "preview", undefined)
    setState("transfer", "decisions", {})
    setState("transfer", "lastResult", undefined)
    setState("transfer", "backupBlob", undefined)
    setState("transfer", "backupFileName", "")
    setState("transfer", "error", "")
    try {
      const preview = await previewImportCompanyProfiles(actor, file)
      setState("transfer", "preview", preview)
      toast.show(
        "画像包已扫描",
        preview.conflict_count > 0
          ? `发现 ${preview.conflict_count} 家冲突公司，需要你确认`
          : `共 ${preview.profiles.length} 家公司，可直接导入`,
      )
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setState("transfer", "error", message)
      throw error
    } finally {
      setState("transfer", "previewing", false)
    }
  }

  const setConflictDecision = (
    profileId: string,
    decision: CompanyProfileConflictDecision,
  ) => {
    setState("transfer", "decisions", profileId, decision)
  }

  const applyDecisionToAll = (decision: CompanyProfileConflictDecision) => {
    const preview = state.transfer.preview
    if (!preview) return
    const next: Record<string, CompanyProfileConflictDecision> = {}
    for (const conflict of preview.conflicts) {
      next[conflict.imported.profile_id] = decision
    }
    setState("transfer", "decisions", next)
  }

  const applyImport = async () => {
    const actor = currentActor()
    const preview = state.transfer.preview
    const file = state.transfer.importFile
    if (!actor || !preview || !file) return

    const request = buildCompanyProfileImportApplyRequest(
      preview,
      state.transfer.decisions,
    )

    setState("transfer", "applying", true)
    setState("transfer", "error", "")

    try {
      if (shouldCreateCompanyProfileBackup(preview, state.transfer.decisions)) {
        const backup = await exportCompanyProfiles(actor)
        setState("transfer", "backupBlob", backup.blob)
        setState("transfer", "backupFileName", backup.fileName)
      }

      const result = await applyImportCompanyProfiles(actor, file, request)
      setState("transfer", "lastResult", result)
      setState("transfer", "preview", undefined)
      setState("transfer", "importFile", undefined)
      setState("transfer", "decisions", {})
      setState("highlightedProfileIds", result.changed_profile_ids)

      await refetchProfiles()
      await refetchActors()

      if (result.changed_profile_ids.length > 0) {
        setState("currentProfileId", result.changed_profile_ids[0])
      }

      toast.show(
        "画像导入完成",
        `新增 ${result.imported_count} 家，替换 ${result.replaced_count} 家，跳过 ${result.skipped_count} 家`,
      )
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setState("transfer", "error", message)
      throw error
    } finally {
      setState("transfer", "applying", false)
    }
  }

  const downloadBackup = () => {
    const blob = state.transfer.backupBlob
    const fileName = state.transfer.backupFileName
    if (!blob || !fileName) return
    triggerBlobDownload(blob, fileName)
  }

  const setManualTargetField = (
    field: "channel" | "user_id" | "channel_scope",
    value: string,
  ) => {
    if (field === "channel") {
      setState("manualTargetChannel", value)
      return
    }
    if (field === "user_id") {
      setState("manualTargetUserId", value)
      return
    }
    setState("manualTargetScope", value)
  }

  const selectManualTarget = () => {
    const channel = state.manualTargetChannel.trim()
    const userId = state.manualTargetUserId.trim()
    const scope = state.manualTargetScope.trim()
    if (!channel || !userId) return false
    selectActor({
      channel,
      user_id: userId,
      channel_scope: scope || undefined,
    })
    setState("manualTargetOpen", false)
    return true
  }

  return {
    state,
    actorsList,
    usersList,
    profileSpaceTargets,
    sessionTargets,
    manualTarget,
    currentActor,
    currentProfileId,
    profiles,
    currentProfile,
    transferReady() {
      return isCompanyProfileImportReady(
        state.transfer.preview,
        state.transfer.decisions,
      )
    },
    selectActor,
    selectProfile,
    refetchActors,
    refetchProfiles,
    refetchCurrent,
    refetchUsers,
    removeProfile,
    exportCurrentSpace,
    previewImport,
    setConflictDecision,
    applyDecisionToAll,
    applyImport,
    downloadBackup,
    resetTransfer,
    setManualTargetField,
    selectManualTarget,
    setManualTargetOpen(open: boolean) {
      setState("manualTargetOpen", open)
      if (open) {
        const actor = currentActor()
        setState("manualTargetChannel", actor?.channel ?? "")
        setState("manualTargetUserId", actor?.user_id ?? "")
        setState("manualTargetScope", actor?.channel_scope ?? "")
      }
    },
  }
}

export function CompanyProfilesProvider(props: ParentProps) {
  const value = createCompanyProfilesState()
  return (
    <CompanyProfilesContext.Provider value={value}>
      {props.children}
    </CompanyProfilesContext.Provider>
  )
}

export function useCompanyProfiles() {
  const value = useContext(CompanyProfilesContext)
  if (!value) throw new Error("CompanyProfilesProvider missing")
  return value
}
