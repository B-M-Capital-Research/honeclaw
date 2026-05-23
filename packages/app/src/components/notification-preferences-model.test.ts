import { describe, expect, it } from "bun:test"

import {
  DEFAULT_NOTIFICATION_PREFS,
  addDigestSlot,
  digestSlotsOverlapQuiet,
  enableQuietHours,
  isValidDigestSlotTime,
  priceHighOverrideFromInput,
  quietHoursHasEmptyWindow,
  removeDigestSlot,
  sameActor,
  sortDigestSlots,
  setQuietHourEnd,
  setQuietHourStart,
  timeFallsInQuiet,
  timezonePrefFromInput,
  toggleAllowedKind,
  toggleBlockedKind,
  toggleImmediateKind,
  toggleQuietExemptKind,
  toggleTag,
} from "./notification-preferences-model"

function slotIds(slots: Array<{ id: string }>): string[] {
  return slots.map((slot) => slot.id)
}

describe("notification-preferences-model", () => {
  it("keeps the default notification prefs in one reusable value", () => {
    expect(DEFAULT_NOTIFICATION_PREFS.enabled).toBe(true)
    expect(DEFAULT_NOTIFICATION_PREFS.min_severity).toBe("low")
    expect(DEFAULT_NOTIFICATION_PREFS.digest_slots).toBeNull()
    expect(DEFAULT_NOTIFICATION_PREFS.quiet_hours).toBeNull()
  })

  it("matches quiet-hour boundaries including overnight windows", () => {
    expect(timeFallsInQuiet("01:00", null)).toBe(false)
    expect(
      timeFallsInQuiet("12:00", {
        from: "09:00",
        to: "17:00",
        exempt_kinds: [],
      }),
    ).toBe(true)
    expect(
      timeFallsInQuiet("17:00", {
        from: "09:00",
        to: "17:00",
        exempt_kinds: [],
      }),
    ).toBe(false)
    expect(
      timeFallsInQuiet("23:30", {
        from: "22:00",
        to: "08:00",
        exempt_kinds: [],
      }),
    ).toBe(true)
    expect(
      timeFallsInQuiet("08:00", {
        from: "22:00",
        to: "08:00",
        exempt_kinds: [],
      }),
    ).toBe(false)
    expect(
      timeFallsInQuiet("10:00", {
        from: "08:00",
        to: "08:00",
        exempt_kinds: [],
      }),
    ).toBe(false)
  })

  it("compares actors by channel, user, and optional scope", () => {
    const base = { channel: "discord", user_id: "u1" }
    expect(sameActor(base, { channel: "discord", user_id: "u1" })).toBe(true)
    expect(
      sameActor(
        { ...base, channel_scope: "dm" },
        { ...base, channel_scope: "group" },
      ),
    ).toBe(false)
    expect(sameActor(undefined, base)).toBe(false)
  })

  it("toggles tags immutably", () => {
    const tags = ["price", "filing"]
    expect(toggleTag(tags, "price")).toEqual(["filing"])
    expect(toggleTag(tags, "news")).toEqual(["price", "filing", "news"])
    expect(tags).toEqual(["price", "filing"])
  })

  it("validates and sorts digest slot times", () => {
    expect(isValidDigestSlotTime("00:00")).toBe(true)
    expect(isValidDigestSlotTime("23:59")).toBe(true)
    expect(isValidDigestSlotTime("24:00")).toBe(false)
    expect(isValidDigestSlotTime("8:00")).toBe(false)

    const slots = [
      { id: "late", time: "23:00" },
      { id: "early", time: "08:00" },
    ]
    expect(slotIds(sortDigestSlots(slots))).toEqual(["early", "late"])
    expect(slotIds(slots)).toEqual(["late", "early"])
  })

  it("keeps nullable tag preferences normalized while toggling", () => {
    const withAllowed = toggleAllowedKind(DEFAULT_NOTIFICATION_PREFS, "filing")
    expect(withAllowed.allow_kinds).toEqual(["filing"])
    expect(toggleAllowedKind(withAllowed, "filing").allow_kinds).toBeNull()

    expect(
      toggleBlockedKind(DEFAULT_NOTIFICATION_PREFS, "price").blocked_kinds,
    ).toEqual(["price"])

    const withImmediate = toggleImmediateKind(
      { ...DEFAULT_NOTIFICATION_PREFS, immediate_kinds: ["news"] },
      "news",
    )
    expect(withImmediate.immediate_kinds).toBeNull()
  })

  it("updates digest slots without mutating or duplicating times", () => {
    const withLateSlot = addDigestSlot(DEFAULT_NOTIFICATION_PREFS, "23:00")
    const withSortedSlots = addDigestSlot(withLateSlot, "08:00")
    expect(slotIds(withSortedSlots.digest_slots ?? [])).toEqual([
      "slot_1",
      "slot_0",
    ])
    expect(
      (addDigestSlot(withSortedSlots, "08:00").digest_slots ?? []).length,
    ).toBe(2)
    expect(removeDigestSlot(withSortedSlots, "slot_1").digest_slots).toEqual([
      { id: "slot_0", time: "23:00" },
    ])
    expect(DEFAULT_NOTIFICATION_PREFS.digest_slots).toBeNull()
  })

  it("updates quiet hours through reusable state transforms", () => {
    const enabled = enableQuietHours(DEFAULT_NOTIFICATION_PREFS)
    expect(enabled.quiet_hours).toEqual({
      from: "00:00",
      to: "08:00",
      exempt_kinds: [],
    })

    const movedStart = setQuietHourStart(enabled, "22:00")
    expect(movedStart.quiet_hours?.from).toBe("22:00")
    expect(movedStart.quiet_hours?.to).toBe("08:00")

    const movedEnd = setQuietHourEnd(DEFAULT_NOTIFICATION_PREFS, "07:30")
    expect(movedEnd.quiet_hours).toEqual({
      from: "00:00",
      to: "07:30",
      exempt_kinds: [],
    })

    const withExempt = toggleQuietExemptKind(movedStart, "earnings")
    expect(withExempt.quiet_hours?.exempt_kinds).toEqual(["earnings"])
    expect(toggleQuietExemptKind(DEFAULT_NOTIFICATION_PREFS, "earnings")).toBe(
      DEFAULT_NOTIFICATION_PREFS,
    )
  })

  it("summarizes quiet-hour derived warning state", () => {
    const prefs = {
      ...DEFAULT_NOTIFICATION_PREFS,
      digest_slots: [{ id: "late", time: "23:30" }],
      quiet_hours: { from: "22:00", to: "08:00", exempt_kinds: [] },
    }
    expect(digestSlotsOverlapQuiet(prefs)).toBe(true)
    expect(
      digestSlotsOverlapQuiet({
        ...prefs,
        digest_slots: [{ id: "midday", time: "12:00" }],
      }),
    ).toBe(false)
    expect(quietHoursHasEmptyWindow(prefs)).toBe(false)
    expect(
      quietHoursHasEmptyWindow({
        ...prefs,
        quiet_hours: { from: "08:00", to: "08:00", exempt_kinds: [] },
      }),
    ).toBe(true)
  })

  it("normalizes scalar preference inputs", () => {
    expect(timezonePrefFromInput(" Asia/Shanghai ")).toBe("Asia/Shanghai")
    expect(timezonePrefFromInput("   ")).toBeNull()
    expect(priceHighOverrideFromInput("", 3)).toBeNull()
    expect(priceHighOverrideFromInput("4.5", null)).toBe(4.5)
    expect(priceHighOverrideFromInput("bad", 3)).toBe(3)
  })
})
