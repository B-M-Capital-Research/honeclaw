import { describe, expect, it } from "bun:test"
import { extractLogRefs, logMatchesUser } from "./log-refs"
import type { LogEntry } from "./types"

function entry(partial: Partial<LogEntry>): LogEntry {
  return {
    timestamp: "2026-04-25 10:00:00.000",
    level: "ERROR",
    target: "test::module",
    message: "",
    ...partial,
  }
}

function requireValue<T>(value: T | null | undefined, label: string): T {
  if (value == null) {
    throw new Error(`${label} was not found`)
  }
  return value
}

describe("extractLogRefs", () => {
  it("returns empty list when nothing structured or matched", () => {
    expect(extractLogRefs(entry({ message: "boom" }))).toEqual([])
  })

  it("reads structured actor from extra.actor", () => {
    const refs = extractLogRefs(
      entry({
        extra: { actor: { channel: "imessage", user_id: "alice", channel_scope: "g:1" } },
      }),
    )
    expect(refs).toContainEqual({
      kind: "actor",
      actor: { channel: "imessage", user_id: "alice", channel_scope: "g:1" },
    })
  })

  it("reads top-level actor_channel/actor_user_id and emits actor ref", () => {
    const refs = extractLogRefs(
      entry({ extra: { actor_channel: "discord", actor_user_id: "bob" } }),
    )
    expect(refs).toContainEqual({
      kind: "actor",
      actor: { channel: "discord", user_id: "bob", channel_scope: undefined },
    })
  })

  it("emits session ref from extra.session_id and derives actor", () => {
    const refs = extractLogRefs(
      entry({ extra: { session_id: "Actor_web__direct__ME" } }),
    )
    const sessionRef = requireValue(
      refs.find((r) => r.kind === "session"),
      "session ref",
    )
    expect(sessionRef.kind).toBe("session")
    if (sessionRef.kind !== "session") {
      throw new Error(`expected session ref, got ${sessionRef.kind}`)
    }
    expect(sessionRef.sessionId).toBe("Actor_web__direct__ME")
    expect(sessionRef.actor).toEqual({
      channel: "web",
      user_id: "ME",
      channel_scope: undefined,
    })
    expect(refs).toContainEqual({
      kind: "actor",
      actor: { channel: "web", user_id: "ME", channel_scope: undefined },
    })
  })

  it("extracts session ids from free-text message", () => {
    const refs = extractLogRefs(
      entry({ message: "failed to push notice for Actor_imessage__direct__alice" }),
    )
    const sessionRef = requireValue(
      refs.find((r) => r.kind === "session"),
      "session ref",
    )
    expect(sessionRef).toEqual({
      kind: "session",
      sessionId: "Actor_imessage__direct__alice",
      actor: {
        channel: "imessage",
        user_id: "alice",
        channel_scope: undefined,
      },
    })
  })

  it("dedupes when extra and message reference the same session", () => {
    const refs = extractLogRefs(
      entry({
        message: "Actor_web__direct__ME failed",
        extra: { session_id: "Actor_web__direct__ME" },
      }),
    )
    expect(refs.filter((r) => r.kind === "session")).toEqual([
      {
        kind: "session",
        sessionId: "Actor_web__direct__ME",
        actor: {
          channel: "web",
          user_id: "ME",
          channel_scope: undefined,
        },
      },
    ])
  })

  it("emits task ref from extra.task_id", () => {
    const refs = extractLogRefs(entry({ extra: { task_id: "cron_42" } }))
    expect(refs).toContainEqual({ kind: "task", taskId: "cron_42" })
  })
})

describe("logMatchesUser", () => {
  it("matches by structured actor user_id", () => {
    const e = entry({ extra: { actor: { channel: "imessage", user_id: "alice" } } })
    expect(logMatchesUser(e, "alice")).toBe(true)
    expect(logMatchesUser(e, "bob")).toBe(false)
  })

  it("matches via session_id-derived actor", () => {
    const e = entry({ extra: { session_id: "Actor_web__direct__ME" } })
    expect(logMatchesUser(e, "ME")).toBe(true)
  })

  it("falls back to message substring", () => {
    expect(logMatchesUser(entry({ message: "user=charlie failed" }), "charlie")).toBe(true)
  })
})
