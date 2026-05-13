import { describe, expect, it } from "bun:test"
import { filterUsers, hasUnread } from "./filters"
import type { UserInfo } from "./types"

function user(patch: Partial<UserInfo>): UserInfo {
  return {
    user_id: "alice@example.com",
    channel: "direct",
    session_id: "Actor_direct__direct__alice",
    session_kind: "direct",
    session_label: "alice@example.com",
    last_message: "",
    last_role: "assistant",
    last_time: "2026-03-07T12:00:00Z",
    message_count: 0,
    ...patch,
  }
}

describe("filters", () => {
  it("filters users by query", () => {
    const users = [
      user({ user_id: "alice@example.com", channel: "direct" }),
      user({
        user_id: "bob@test.com",
        channel: "discord",
        session_id: "Actor_discord__direct__bob",
        session_label: "bob@test.com",
      }),
    ]

    expect(filterUsers(users, "ali")).toHaveLength(1)
    expect(filterUsers(users, "  ")).toHaveLength(2)
    expect(filterUsers(users, "", "discord")).toHaveLength(1)
    expect(filterUsers(users, "bob", "direct")).toHaveLength(0)
  })

  it("detects unread state", () => {
    expect(hasUnread("alice", "2026-03-07T12:00:00Z", "user", {}, undefined)).toBe(true)
    expect(
      hasUnread(
        "alice",
        "2026-03-07T12:00:00Z",
        "assistant",
        { alice: "2026-03-07T12:30:00Z" },
        undefined,
      ),
    ).toBe(false)
  })
})
