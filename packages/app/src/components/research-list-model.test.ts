import { afterAll, describe, expect, it } from "bun:test";

import {
  confirmableResearchName,
  formatResearchTaskTime,
  researchStatusBadgeConfig,
  researchSymbolFromSearchParam,
} from "./research-list-model"
import { setLocale } from "@/lib/i18n"

describe("research-list-model", () => {
  it("maps task status into stable badge config", () => {
    setLocale("zh")
    expect(
      researchStatusBadgeConfig({
        status: "completed",
        answer_markdown: "# report",
      }),
    ).toMatchObject({
      label: "报告就绪",
      dot: "bg-[color:var(--success)]",
      text: "text-[color:var(--success)]",
    })
    expect(
      researchStatusBadgeConfig({
        status: "running",
        progress: "正在抓取资料",
      }),
    ).toMatchObject({
      label: "正在抓取资料",
      dot: "bg-blue-400 animate-pulse",
      text: "text-blue-500",
    })
    expect(researchStatusBadgeConfig({ status: "error" }).text).toBe(
      "text-rose-500",
    )
  })

  it("normalizes search-param symbols and confirm names", () => {
    expect(researchSymbolFromSearchParam("aapl")).toBe("AAPL")
    expect(researchSymbolFromSearchParam(["aapl"])).toBe("")
    expect(confirmableResearchName(" Apple ", false)).toBe("Apple")
    expect(confirmableResearchName(" Apple ", true)).toBeNull()
    expect(confirmableResearchName(" ", false)).toBeNull()
  })

  it("formats task timestamps with an explicit locale", () => {
    expect(formatResearchTaskTime(undefined, "zh")).toBe("")
    expect(formatResearchTaskTime("not-a-date", "en")).toBe("not-a-date")
    expect(formatResearchTaskTime("2026-05-23T01:02:00Z", "zh")).toContain(
      "05",
    )
    expect(formatResearchTaskTime("2026-05-23T01:02:00Z", "en")).toContain(
      "05",
    )
  })
})

// `locale` 是模块级 signal，整个 bun test 进程共用；不还原会泄漏到之后运行的
// 测试文件（本地/CI 文件顺序不同，表现为随机的跨文件失败）。
afterAll(() => {
  setLocale("en");
});
