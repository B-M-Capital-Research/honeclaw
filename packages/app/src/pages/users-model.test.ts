import { describe, expect, it } from "bun:test"

import {
  actorFromManualDraft,
  actorListStatsText,
  availableUsersTabs,
  filterActorList,
  patchActorDraft,
  resolveUsersTab,
  uniqueSortedSymbols,
  USER_TAB_CONFIG,
} from "./users-model"
import type { ActorListItem } from "@/lib/actors"

function tabIds(tabs: Array<{ id: string }>): string[] {
  return tabs.map((tab) => tab.id)
}

function actorItem(patch: Partial<ActorListItem>): ActorListItem {
  return {
    actor: { channel: "imessage", user_id: "alice" },
    key: "imessage||alice",
    ...patch,
  }
}

describe("users-model", () => {
  it("resolves route tab params with portfolio fallback", () => {
    expect(resolveUsersTab("profiles")).toBe("profiles")
    expect(resolveUsersTab("sessions")).toBe("sessions")
    expect(resolveUsersTab("unknown")).toBe("portfolio")
    expect(resolveUsersTab(undefined)).toBe("portfolio")
  })

  it("keeps user tabs in route order and filters capability-gated tabs", () => {
    expect(tabIds(USER_TAB_CONFIG)).toEqual([
      "portfolio",
      "profiles",
      "mainline",
      "sessions",
      "research",
    ])

    expect(tabIds(availableUsersTabs(() => false))).toEqual([
      "portfolio",
      "mainline",
      "sessions",
    ])
    expect(
      tabIds(availableUsersTabs((capability) => capability === "research")),
    ).toEqual(["portfolio", "mainline", "sessions", "research"])
  })

  it("derives a sorted unique symbol list for research shortcuts", () => {
    expect(
      uniqueSortedSymbols(
        [{ symbol: " aapl " }, { symbol: "MSFT" }],
        [{ symbol: "msft" }, { symbol: "" }, { symbol: "tsla" }],
      ),
    ).toEqual(["AAPL", "MSFT", "TSLA"])
  })

  it("keeps actor-list filtering in the model layer", () => {
    const items = [
      actorItem({ actor: { channel: "imessage", user_id: "alice" } }),
      actorItem({
        actor: { channel: "telegram", user_id: "bob", channel_scope: "desk" },
        key: "telegram|desk|bob",
        sessionLabel: "Daily research",
      }),
    ]

    expect(filterActorList(items, "")).toBe(items)
    expect(filterActorList(items, " TELEGRAM ")).toEqual([items[1]])
    expect(filterActorList(items, "desk")).toEqual([items[1]])
    expect(filterActorList(items, "daily")).toEqual([items[1]])
    expect(filterActorList(items, "missing")).toEqual([])
  })

  it("normalizes manual actor draft state before selection", () => {
    const draft = { channel: " imessage ", user_id: " alice ", channel_scope: " " }
    expect(actorFromManualDraft(draft)).toEqual({
      channel: "imessage",
      user_id: "alice",
      channel_scope: undefined,
    })
    expect(actorFromManualDraft({ ...draft, user_id: "" })).toBeNull()
    expect(patchActorDraft(draft, { channel_scope: "family" })).toEqual({
      channel: " imessage ",
      user_id: " alice ",
      channel_scope: "family",
    })
  })

  it("derives actor-list stat text outside the component", () => {
    expect(actorListStatsText(actorItem({}))).toBe("暂无数据")
    expect(
      actorListStatsText(
        actorItem({
          holdingsCount: 2,
          watchlistCount: 1,
          profileCount: 3,
          lastSessionTime: "2026-05-23T00:00:00Z",
        }),
      ),
    ).toBe("2 持仓 · 1 关注 · 3 画像 · 会话")
  })
})
