import { describe, expect, it } from "bun:test"

import {
  availableUsersTabs,
  resolveUsersTab,
  uniqueSortedSymbols,
  USER_TAB_CONFIG,
} from "./users-model"

function tabIds(tabs: Array<{ id: string }>): string[] {
  return tabs.map((tab) => tab.id)
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
})
