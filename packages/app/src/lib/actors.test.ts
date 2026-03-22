import { describe, expect, it } from "bun:test"
import { actorKey, parseActorKey } from "./actors"

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
