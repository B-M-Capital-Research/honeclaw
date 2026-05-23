import type { CronJobUpsertInput } from "@/lib/types"

type CronJobDraftTiming = Pick<Partial<CronJobUpsertInput>, "repeat" | "tags">

export function isHeartbeatCronDraft(draft: CronJobDraftTiming): boolean {
  return draft.repeat === "heartbeat" || (draft.tags ?? []).includes("heartbeat")
}

export function tagsForRepeatDraft(
  repeat: string,
  currentTags: string[] | undefined,
): string[] {
  return repeat === "heartbeat"
    ? ["heartbeat"]
    : (currentTags ?? []).filter((tag) => tag !== "heartbeat")
}

export function cronJobUpsertInputFromDraft(
  draft: Partial<CronJobUpsertInput>,
): CronJobUpsertInput {
  const isHeartbeat = isHeartbeatCronDraft(draft)
  return {
    channel: draft.channel,
    name: draft.name,
    task_prompt: draft.task_prompt,
    user_id: draft.user_id,
    channel_scope: draft.channel_scope,
    hour: isHeartbeat ? undefined : Number(draft.hour),
    minute: isHeartbeat ? undefined : Number(draft.minute),
    repeat: draft.repeat,
    weekday:
      draft.weekday !== undefined && !Number.isNaN(Number(draft.weekday))
        ? Number(draft.weekday)
        : undefined,
    enabled: draft.enabled,
    channel_target: draft.channel_target,
    tags: draft.tags,
  }
}
