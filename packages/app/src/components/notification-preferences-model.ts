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
  const [h, m] = value.split(":").map(Number)
  if (Number.isNaN(h) || Number.isNaN(m)) return -1
  return h * 60 + m
}

/** Matches backend schedule_view::time_in_quiet semantics. */
export function timeFallsInQuiet(
  hhmm: string,
  qh: QuietHoursPrefs | null,
): boolean {
  if (!qh) return false
  const t = timeToMinutes(hhmm)
  const f = timeToMinutes(qh.from)
  const o = timeToMinutes(qh.to)
  if (t < 0 || f < 0 || o < 0) return false
  if (f === o) return false
  return f < o ? t >= f && t < o : t >= f || t < o
}

export function sameActor(a?: ActorRef, b?: ActorRef): boolean {
  if (!a || !b) return false
  return (
    a.channel === b.channel &&
    a.user_id === b.user_id &&
    (a.channel_scope ?? "") === (b.channel_scope ?? "")
  )
}

export function toggleTag(list: string[], tag: string): string[] {
  return list.includes(tag) ? list.filter((t) => t !== tag) : [...list, tag]
}

export function isValidDigestSlotTime(value: string): boolean {
  return HHMM_RE.test(value)
}

export function sortDigestSlots(list: DigestSlot[]): DigestSlot[] {
  return [...list].sort((a, b) => a.time.localeCompare(b.time))
}
