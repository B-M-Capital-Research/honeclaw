import { describe, expect, it } from "bun:test"

import {
  DEFAULT_NOTIFICATION_PREFS,
  isValidDigestSlotTime,
  sameActor,
  sortDigestSlots,
  timeFallsInQuiet,
  toggleTag,
} from "./notification-preferences-model"

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
    expect(sortDigestSlots(slots).map((slot) => slot.id)).toEqual([
      "early",
      "late",
    ])
    expect(slots.map((slot) => slot.id)).toEqual(["late", "early"])
  })
})
