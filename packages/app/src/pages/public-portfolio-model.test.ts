import { describe, expect, it } from "bun:test"

import {
  canRefreshPublicMainline,
  formatPublicMainlineTimestamp,
  publicRefreshMessage,
} from "./public-portfolio-model"

describe("public-portfolio-model", () => {
  it("formats last mainline timestamps with an explicit clock", () => {
    const now = new Date("2026-05-14T08:00:00+08:00").getTime()

    expect(formatPublicMainlineTimestamp(null, now)).toBe("从未")
    expect(formatPublicMainlineTimestamp("not-a-date", now)).toBe("not-a-date")
    expect(formatPublicMainlineTimestamp("2026-05-14T07:30:00+08:00", now)).toMatch(
      /^今天 /,
    )
    expect(formatPublicMainlineTimestamp("2026-05-13T08:00:00+08:00", now)).toBe(
      "1 天前",
    )
    expect(formatPublicMainlineTimestamp("2026-05-11T08:00:00+08:00", now)).toBe(
      "3 天前",
    )
  })

  it("derives refresh state and messages without page state", () => {
    expect(canRefreshPublicMainline(0)).toBe(false)
    expect(canRefreshPublicMainline(1)).toBe(true)
    expect(
      publicRefreshMessage({
        mainline_count: 3,
        skipped_tickers: ["AAPL", "MSFT"],
      }),
    ).toBe("更新完成：3 条投资主线，2 只跳过")
  })

})
