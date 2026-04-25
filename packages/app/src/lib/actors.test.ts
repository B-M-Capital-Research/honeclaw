import { describe, expect, it } from "bun:test"
import {
  actorFromSessionId,
  actorKey,
  mergeActorSummaries,
  parseActorKey,
} from "./actors"
import type {
  CompanyProfileSpaceSummary,
  PortfolioSummary,
  UserInfo,
} from "./types"

describe("actors", () => {
  it("roundtrips actor keys with optional scope", () => {
    const actor = { channel: "discord", user_id: "alice", channel_scope: "g:1:c:2" }
    expect(parseActorKey(actorKey(actor))).toEqual(actor)
  })

  it("parses direct actor keys without scope", () => {
    expect(parseActorKey("imessage||alice")).toEqual({
      channel: "imessage",
      user_id: "alice",
      channel_scope: undefined,
    })
  })
})

describe("actorFromSessionId", () => {
  it("decodes ME default session", () => {
    expect(actorFromSessionId("Actor_web__direct__ME")).toEqual({
      channel: "web",
      user_id: "ME",
      channel_scope: undefined,
    })
  })

  it("decodes scoped session ids with hex-encoded special chars", () => {
    // scope "g:1:c:2" → ":" 编码为 _3a
    expect(actorFromSessionId("Actor_discord__g_3a1_3ac_3a2__alice")).toEqual({
      channel: "discord",
      user_id: "alice",
      channel_scope: "g:1:c:2",
    })
  })

  it("returns undefined for invalid prefix or shape", () => {
    expect(actorFromSessionId(undefined)).toBeUndefined()
    expect(actorFromSessionId("")).toBeUndefined()
    expect(actorFromSessionId("not_actor_session")).toBeUndefined()
    expect(actorFromSessionId("Actor_web__direct")).toBeUndefined()
  })
})

describe("mergeActorSummaries", () => {
  const portfolios: PortfolioSummary[] = [
    {
      channel: "imessage",
      user_id: "alice",
      holdings_count: 3,
      watchlist_count: 2,
      total_shares: 100,
      updated_at: "2026-04-20T10:00:00Z",
    },
  ]
  const profiles: CompanyProfileSpaceSummary[] = [
    {
      channel: "imessage",
      user_id: "alice",
      profile_count: 5,
      updated_at: "2026-04-22T10:00:00Z",
    },
    {
      channel: "discord",
      user_id: "bob",
      profile_count: 1,
    },
  ]
  const sessions: UserInfo[] = [
    {
      channel: "discord",
      user_id: "bob",
      session_id: "Actor_discord__direct__bob",
      session_kind: "direct",
      session_label: "bob",
      last_message: "hi",
      last_role: "user",
      last_time: "2026-04-25T12:00:00Z",
      message_count: 7,
    },
  ]

  it("merges by actor key and aggregates per-source fields", () => {
    const result = mergeActorSummaries({ portfolios, profiles, sessions })
    expect(result).toHaveLength(2)
    const alice = result.find((r) => r.actor.user_id === "alice")
    const bob = result.find((r) => r.actor.user_id === "bob")
    expect(alice?.holdingsCount).toBe(3)
    expect(alice?.watchlistCount).toBe(2)
    expect(alice?.profileCount).toBe(5)
    expect(alice?.updatedAt).toBe("2026-04-22T10:00:00Z")
    expect(bob?.profileCount).toBe(1)
    expect(bob?.lastSessionTime).toBe("2026-04-25T12:00:00Z")
    expect(bob?.sessionLabel).toBe("bob")
  })

  it("sorts by lastSessionTime first, falls back to updatedAt", () => {
    const result = mergeActorSummaries({ portfolios, profiles, sessions })
    // bob has session timestamp 2026-04-25 > alice updatedAt 2026-04-22 → bob first
    expect(result[0].actor.user_id).toBe("bob")
    expect(result[1].actor.user_id).toBe("alice")
  })

  it("handles empty input", () => {
    expect(mergeActorSummaries({})).toEqual([])
  })
})
