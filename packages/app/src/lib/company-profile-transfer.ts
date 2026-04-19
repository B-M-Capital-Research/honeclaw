import type {
  CompanyProfileConflictDecision,
  CompanyProfileImportApplyRequest,
  CompanyProfileImportPreview,
} from "./types"

export type CompanyProfileDecisionMap = Record<
  string,
  CompanyProfileConflictDecision | undefined
>

export function isCompanyProfileImportReady(
  preview?: CompanyProfileImportPreview,
  decisions: CompanyProfileDecisionMap = {},
) {
  if (!preview) return false
  if (preview.conflict_count === 0) return preview.profiles.length > 0
  return preview.conflicts.every((conflict) => {
    const decision = decisions[conflict.imported.profile_id]
    return decision === "skip" || decision === "replace"
  })
}

export function buildCompanyProfileImportApplyRequest(
  preview: CompanyProfileImportPreview,
  decisions: CompanyProfileDecisionMap = {},
): CompanyProfileImportApplyRequest {
  if (preview.conflict_count === 0) {
    return {
      mode: "keep_existing",
      decisions: {},
    }
  }

  const filteredDecisions: Record<string, CompanyProfileConflictDecision> = {}
  for (const conflict of preview.conflicts) {
    const decision = decisions[conflict.imported.profile_id]
    if (decision !== "skip" && decision !== "replace") {
      throw new Error(`missing decision for ${conflict.imported.profile_id}`)
    }
    filteredDecisions[conflict.imported.profile_id] = decision
  }

  return {
    mode: "interactive",
    decisions: filteredDecisions,
  }
}

export function shouldCreateCompanyProfileBackup(
  preview?: CompanyProfileImportPreview,
  decisions: CompanyProfileDecisionMap = {},
) {
  if (!preview || preview.conflict_count === 0) return false
  return preview.conflicts.some(
    (conflict) => decisions[conflict.imported.profile_id] === "replace",
  )
}
