import { describe, expect, it } from "bun:test"

import {
  firstProfileTicker,
  mainlineHoldingCardState,
  profileInventoryRowState,
  profileTickerSet,
} from "./mainline-context-model"

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

  it("derives holding card state from shared mainline context", () => {
    const context = {
      holdings: ["AAPL", "MSFT"],
      mainline_by_ticker: {
        AAPL: "Own for platform durability.",
      },
      mainline_distill_skipped: ["MSFT"],
      profile_list: [{ tickers: ["AAPL"] }],
    }

    expect(mainlineHoldingCardState(context, "AAPL")).toEqual({
      ticker: "AAPL",
      mainline: "Own for platform durability.",
      hasProfile: true,
      isSkipped: false,
    })
    expect(mainlineHoldingCardState(context, "MSFT")).toEqual({
      ticker: "MSFT",
      mainline: undefined,
      hasProfile: false,
      isSkipped: true,
    })
  })

  it("derives profile inventory display rows", () => {
    expect(
      profileInventoryRowState({
        title: "Apple",
        dir: "apple",
        bytes: 1536,
        tickers: ["AAPL", "MSFT"],
      }),
    ).toEqual({
      title: "Apple",
      tickerLabel: "AAPL / MSFT",
      sizeLabel: "1.5 KB",
      dir: "apple",
      viewTicker: "AAPL",
    })

    expect(
      profileInventoryRowState({
        title: "",
        dir: "empty-ticker-profile",
        bytes: 0,
        tickers: [],
      }).viewTicker,
    ).toBeNull()
  })
})
