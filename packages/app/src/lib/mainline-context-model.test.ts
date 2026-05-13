import { describe, expect, it } from "bun:test"

import { profileTickerSet } from "./mainline-context-model"

describe("mainline-context-model", () => {
  it("derives unique profile tickers from context profile summaries", () => {
    const tickers = profileTickerSet({
      profile_list: [
        { tickers: ["AAPL", "MSFT"] },
        { tickers: ["MSFT", "TSLA"] },
        { tickers: [] },
      ],
    })

    expect([...tickers].sort()).toEqual(["AAPL", "MSFT", "TSLA"])
  })

  it("returns an empty set for missing context", () => {
    expect(profileTickerSet(null).size).toBe(0)
    expect(profileTickerSet(undefined).size).toBe(0)
  })
})
