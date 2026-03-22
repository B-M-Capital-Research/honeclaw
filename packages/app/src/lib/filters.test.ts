import { describe, expect, it } from "bun:test"
import { filterUsers, hasUnread } from "./filters"

describe("filters", () => {
  it("filters users by query", () => {
    const users = [
      { user_id: "alice@example.com", channel: "direct" },
      { user_id: "bob@test.com", channel: "discord" },
    ] as any

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
