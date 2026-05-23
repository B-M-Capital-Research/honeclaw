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

function emptyTagsAsNull(tags: string[]): string[] | null {
  return tags.length === 0 ? null : tags
}

export function toggleAllowedKind(
  prefs: NotificationPrefs,
  tag: string,
): NotificationPrefs {
  return {
    ...prefs,
    allow_kinds: emptyTagsAsNull(toggleTag(prefs.allow_kinds ?? [], tag)),
  }
}

export function toggleBlockedKind(
  prefs: NotificationPrefs,
  tag: string,
): NotificationPrefs {
  return {
    ...prefs,
    blocked_kinds: toggleTag(prefs.blocked_kinds ?? [], tag),
  }
}

export function toggleImmediateKind(
  prefs: NotificationPrefs,
  tag: string,
): NotificationPrefs {
  return {
    ...prefs,
    immediate_kinds: emptyTagsAsNull(
      toggleTag(prefs.immediate_kinds ?? [], tag),
    ),
  }
}

export function addDigestSlot(
  prefs: NotificationPrefs,
  slotTime: string,
): NotificationPrefs {
  if (!isValidDigestSlotTime(slotTime)) return prefs
  const existingSlots = prefs.digest_slots ?? []
  if (existingSlots.some((slot) => slot.time === slotTime)) return prefs
  return {
    ...prefs,
    digest_slots: sortDigestSlots([
      ...existingSlots,
      { id: `slot_${existingSlots.length}`, time: slotTime },
    ]),
  }
}

export function removeDigestSlot(
  prefs: NotificationPrefs,
  slotId: string,
): NotificationPrefs {
  return {
    ...prefs,
    digest_slots: (prefs.digest_slots ?? []).filter(
      (slot) => slot.id !== slotId,
    ),
  }
}

export function digestSlotsOverlapQuiet(prefs: NotificationPrefs): boolean {
  const quietHours = prefs.quiet_hours
  if (!quietHours) return false
  return (prefs.digest_slots ?? []).some((digestSlot) =>
    timeFallsInQuiet(digestSlot.time, quietHours),
  )
}

export function setQuietHourStart(
  prefs: NotificationPrefs,
  quietStart: string,
): NotificationPrefs {
  if (!isValidDigestSlotTime(quietStart)) return prefs
  return {
    ...prefs,
    quiet_hours: {
      from: quietStart,
      to: prefs.quiet_hours?.to ?? "08:00",
      exempt_kinds: prefs.quiet_hours?.exempt_kinds ?? [],
    },
  }
}

export function setQuietHourEnd(
  prefs: NotificationPrefs,
  quietEnd: string,
): NotificationPrefs {
  if (!isValidDigestSlotTime(quietEnd)) return prefs
  return {
    ...prefs,
    quiet_hours: {
      from: prefs.quiet_hours?.from ?? "00:00",
      to: quietEnd,
      exempt_kinds: prefs.quiet_hours?.exempt_kinds ?? [],
    },
  }
}

export function enableQuietHours(prefs: NotificationPrefs): NotificationPrefs {
  return prefs.quiet_hours
    ? prefs
    : {
        ...prefs,
        quiet_hours: { from: "00:00", to: "08:00", exempt_kinds: [] },
      }
}

export function toggleQuietExemptKind(
  prefs: NotificationPrefs,
  tag: string,
): NotificationPrefs {
  if (!prefs.quiet_hours) return prefs
  return {
    ...prefs,
    quiet_hours: {
      ...prefs.quiet_hours,
      exempt_kinds: toggleTag(prefs.quiet_hours.exempt_kinds, tag),
    },
  }
}

export function quietHoursHasEmptyWindow(prefs: NotificationPrefs): boolean {
  return Boolean(
    prefs.quiet_hours && prefs.quiet_hours.from === prefs.quiet_hours.to,
  )
}

export function timezonePrefFromInput(raw: string): string | null {
  const timezone = raw.trim()
  return timezone === "" ? null : timezone
}

export function priceHighOverrideFromInput(
  raw: string,
  currentValue: number | null | undefined,
): number | null {
  const priceThreshold = raw.trim()
  if (priceThreshold === "") return null
  const parsedThreshold = Number(priceThreshold)
  return Number.isFinite(parsedThreshold)
    ? parsedThreshold
    : currentValue ?? null
}
