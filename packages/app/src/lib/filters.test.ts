import { describe, expect, it } from "bun:test"
import { filterUsers, hasUnread } from "./filters"
import type { UserInfo } from "./types"

function userInfoFixture(patch: Partial<UserInfo>): UserInfo {
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

function filteredUserIds(
  users: UserInfo[],
  query: string,
  channel = "all",
): string[] {
  return filterUsers(users, query, channel).map((user) => user.user_id)
}

type UnreadDecisionInput = {
  userId?: string
  lastTime?: string
  lastRole?: string
  readAt?: Record<string, string>
  activeUserId?: string
}

function expectUnreadDecision(
  input: UnreadDecisionInput,
  expected: boolean,
): void {
  expect(
    hasUnread(
      input.userId ?? "alice",
      input.lastTime ?? "2026-03-07T12:00:00Z",
      input.lastRole ?? "user",
      input.readAt ?? {},
      input.activeUserId,
    ),
  ).toBe(expected)
}

describe("filters", () => {
  it("matches users by query and channel without losing row identity", () => {
    const users = [
      userInfoFixture({ user_id: "alice@example.com", channel: "direct" }),
      userInfoFixture({
        user_id: "bob@test.com",
        channel: "discord",
        session_id: "Actor_discord__direct__bob",
        session_label: "bob@test.com",
      }),
    ]

    expect(filteredUserIds(users, "ali")).toEqual(["alice@example.com"])
    expect(filteredUserIds(users, "  ")).toEqual([
      "alice@example.com",
      "bob@test.com",
    ])
    expect(filteredUserIds(users, "", "discord")).toEqual(["bob@test.com"])
    expect(filteredUserIds(users, "bob", "direct")).toEqual([])
  })

  it("marks unread rows only for other-user messages newer than the read stamp", () => {
    expectUnreadDecision({}, true)
    expectUnreadDecision({ lastRole: "assistant" }, false)
    expectUnreadDecision({ activeUserId: "alice" }, false)
    expectUnreadDecision(
      {
        lastTime: "2026-03-07T12:30:01Z",
        lastRole: "assistant",
        readAt: { alice: "2026-03-07T12:30:00Z" },
      },
      true,
    )
    expectUnreadDecision(
      {
        lastRole: "assistant",
        readAt: { alice: "2026-03-07T12:30:00Z" },
      },
      false,
    )
  })
})
