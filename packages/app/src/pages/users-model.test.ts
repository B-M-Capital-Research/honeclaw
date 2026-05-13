import { describe, expect, it } from "bun:test"

import {
  availableUsersTabs,
  resolveUsersTab,
  uniqueSortedSymbols,
  USER_TAB_CONFIG,
} from "./users-model"

describe("users-model", () => {
  it("resolves route tab params with portfolio fallback", () => {
    expect(resolveUsersTab("profiles")).toBe("profiles")
    expect(resolveUsersTab("sessions")).toBe("sessions")
    expect(resolveUsersTab("unknown")).toBe("portfolio")
    expect(resolveUsersTab(undefined)).toBe("portfolio")
  })

  it("keeps user tabs in route order and filters capability-gated tabs", () => {
    expect(USER_TAB_CONFIG.map((tab) => tab.id)).toEqual([
      "portfolio",
      "profiles",
      "mainline",
      "sessions",
      "research",
    ])

    expect(availableUsersTabs(() => false).map((tab) => tab.id)).toEqual([
      "portfolio",
      "mainline",
      "sessions",
    ])
    expect(
      availableUsersTabs((capability) => capability === "research").map(
        (tab) => tab.id,
      ),
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
