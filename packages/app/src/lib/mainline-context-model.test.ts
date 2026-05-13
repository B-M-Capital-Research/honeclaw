import { describe, expect, it } from "bun:test"

import { firstProfileTicker, profileTickerSet } from "./mainline-context-model"

describe("mainline-context-model", () => {
  it("deduplicates profile tickers while preserving the ticker values", () => {
    const tickers = profileTickerSet({
      profile_list: [
        { tickers: ["AAPL", "MSFT"] },
        { tickers: ["MSFT", "TSLA"] },
        { tickers: [] },
      ],
    })

    expect([...tickers].sort()).toEqual(["AAPL", "MSFT", "TSLA"])
    expect(tickers.has("AAPL")).toBe(true)
  })

  it("returns an empty set for missing context", () => {
    expect(profileTickerSet(null).size).toBe(0)
    expect(profileTickerSet(undefined).size).toBe(0)
  })

  it("selects the first ticker available for profile view actions", () => {
    expect(firstProfileTicker({ tickers: ["AAPL", "MSFT"] })).toBe("AAPL")
    expect(firstProfileTicker({ tickers: [] })).toBeNull()
  })
})
