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
import { setLocale } from "@/lib/i18n"
import type { ActorListItem } from "@/lib/actors"

function userTabIds(tabs: Array<{ id: string }>): string[] {
  return tabs.map((tab) => tab.id)
}

function actorListItemFixture(patch: Partial<ActorListItem>): ActorListItem {
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
    expect(userTabIds(USER_TAB_CONFIG)).toEqual([
      "portfolio",
      "profiles",
      "mainline",
      "sessions",
      "research",
    ])

    expect(userTabIds(availableUsersTabs(() => false))).toEqual([
      "portfolio",
      "mainline",
      "sessions",
    ])
    expect(
      userTabIds(availableUsersTabs((capability) => capability === "research")),
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
    const actorItems = [
      actorListItemFixture({ actor: { channel: "imessage", user_id: "alice" } }),
      actorListItemFixture({
        actor: { channel: "telegram", user_id: "bob", channel_scope: "desk" },
        key: "telegram|desk|bob",
        sessionLabel: "Daily research",
      }),
    ]

    expect(filterActorList(actorItems, "")).toBe(actorItems)
    expect(filterActorList(actorItems, " TELEGRAM ")).toEqual([actorItems[1]])
    expect(filterActorList(actorItems, "desk")).toEqual([actorItems[1]])
    expect(filterActorList(actorItems, "daily")).toEqual([actorItems[1]])
    expect(filterActorList(actorItems, "missing")).toEqual([])
  })

  it("normalizes manual actor draft state before selection", () => {
    const manualActorDraft = {
      channel: " imessage ",
      user_id: " alice ",
      channel_scope: " ",
    }
    expect(actorFromManualDraft(manualActorDraft)).toEqual({
      channel: "imessage",
      user_id: "alice",
      channel_scope: undefined,
    })
    expect(actorFromManualDraft({ ...manualActorDraft, user_id: "" })).toBeNull()
    expect(patchActorDraft(manualActorDraft, { channel_scope: "family" })).toEqual({
      channel: " imessage ",
      user_id: " alice ",
      channel_scope: "family",
    })
  })

  it("derives actor-list stat text outside the component", () => {
    setLocale("zh")

    expect(actorListStatsText(actorListItemFixture({}))).toBe("暂无数据")
    expect(
      actorListStatsText(
        actorListItemFixture({
          holdingsCount: 2,
          watchlistCount: 1,
          profileCount: 3,
          lastSessionTime: "2026-05-23T00:00:00Z",
        }),
      ),
    ).toBe("2 持仓 · 1 关注 · 3 画像 · 会话")
  })
})
