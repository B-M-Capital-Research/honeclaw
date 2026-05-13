import type {
  DigestSlot,
  NotificationPrefs,
  QuietHoursPrefs,
} from "@/lib/api"
import type { ActorRef } from "@/lib/actors"

export const DEFAULT_NOTIFICATION_PREFS: NotificationPrefs = {
  enabled: true,
  portfolio_only: false,
  min_severity: "low",
  allow_kinds: null,
  blocked_kinds: [],
  timezone: null,
  digest_slots: null,
  price_high_pct_override: null,
  immediate_kinds: null,
  quiet_hours: null,
}

const HHMM_RE = /^([01]\d|2[0-3]):[0-5]\d$/

function timeToMinutes(value: string): number {
  const [hours, minutes] = value.split(":").map(Number)
  if (Number.isNaN(hours) || Number.isNaN(minutes)) return -1
  return hours * 60 + minutes
}

/** Matches backend schedule_view::time_in_quiet semantics. */
export function timeFallsInQuiet(
  hhmm: string,
  quietHours: QuietHoursPrefs | null,
): boolean {
  if (!quietHours) return false
  const currentMinutes = timeToMinutes(hhmm)
  const quietStartMinutes = timeToMinutes(quietHours.from)
  const quietEndMinutes = timeToMinutes(quietHours.to)
  if (currentMinutes < 0 || quietStartMinutes < 0 || quietEndMinutes < 0) {
    return false
  }
  if (quietStartMinutes === quietEndMinutes) return false
  return quietStartMinutes < quietEndMinutes
    ? currentMinutes >= quietStartMinutes && currentMinutes < quietEndMinutes
    : currentMinutes >= quietStartMinutes || currentMinutes < quietEndMinutes
}

export function sameActor(leftActor?: ActorRef, rightActor?: ActorRef): boolean {
  if (!leftActor || !rightActor) return false
  return (
    leftActor.channel === rightActor.channel &&
    leftActor.user_id === rightActor.user_id &&
    (leftActor.channel_scope ?? "") === (rightActor.channel_scope ?? "")
  )
}

export function toggleTag(currentTags: string[], tag: string): string[] {
  return currentTags.includes(tag)
    ? currentTags.filter((currentTag) => currentTag !== tag)
    : [...currentTags, tag]
}

export function isValidDigestSlotTime(value: string): boolean {
  return HHMM_RE.test(value)
}

export function sortDigestSlots(digestSlots: DigestSlot[]): DigestSlot[] {
  return [...digestSlots].sort((leftSlot, rightSlot) =>
    leftSlot.time.localeCompare(rightSlot.time),
  )
}
