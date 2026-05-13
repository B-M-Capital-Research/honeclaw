import { describe, expect, it } from "bun:test"

import { setLocale } from "@/lib/i18n"
import {
  TASK_HEALTH_DAYS_OPTIONS,
  relativeTaskRunTime,
  shortTaskRunTime,
  taskFilterOptions,
  taskRunDurationMs,
  taskRunOutcomeClass,
  taskSuccessRate,
  taskSummaryRows,
} from "./task-health-model"
import type { TaskSummary } from "@/lib/api"

function taskSummaryFixture(patch: Partial<TaskSummary> = {}): TaskSummary {
  return {
    runs_24h: 0,
    ok_24h: 0,
    skipped_24h: 0,
    failed_24h: 0,
    last_seen_at: null,
    last_failure_at: null,
    last_error: null,
    runs_since_last_failure: null,
    ...patch,
  }
}

describe("task-health-model", () => {
  it("keeps day filter options stable", () => {
    expect([...TASK_HEALTH_DAYS_OPTIONS]).toEqual([1, 3, 7, 14])
  })

  it("formats run times and durations without page state", () => {
    expect(shortTaskRunTime(null, "zh")).toBe("—")
    expect(shortTaskRunTime("not-a-date", "zh")).toBe("not-a-date")
    expect(shortTaskRunTime("2026-05-13T00:01:02", "zh")).toMatch(/00:01:02/)

    expect(taskRunDurationMs("2026-05-13T00:00:00Z", "2026-05-13T00:00:03Z")).toBe(3000)
    expect(taskRunDurationMs("bad", "2026-05-13T00:00:03Z")).toBe(0)
    expect(taskRunDurationMs("2026-05-13T00:00:03Z", "2026-05-13T00:00:00Z")).toBe(0)
  })

  it("derives relative labels using an explicit clock", () => {
    setLocale("zh")
    const now = new Date("2026-05-13T00:00:00Z")

    expect(relativeTaskRunTime(null, now)).toBe("—")
    expect(relativeTaskRunTime("not-a-date", now)).toBe("not-a-date")
    expect(relativeTaskRunTime("2026-05-12T23:59:30Z", now)).toBe("30 秒前")
    expect(relativeTaskRunTime("2026-05-12T23:30:00Z", now)).toBe("30 分钟前")
    expect(relativeTaskRunTime("2026-05-12T21:00:00Z", now)).toBe("3 小时前")
    expect(relativeTaskRunTime("2026-05-11T00:00:00Z", now)).toBe("2 天前")
  })

  it("maps outcome tone and success rate", () => {
    expect(taskRunOutcomeClass("ok")).toBe("text-emerald-300 bg-emerald-500/15")
    expect(taskRunOutcomeClass("failed")).toBe("text-rose-300 bg-rose-500/15")
    expect(taskRunOutcomeClass("skipped")).toBe(
      "text-[color:var(--text-muted)] bg-white/5",
    )
    expect(taskRunOutcomeClass("custom")).toBe(
      "text-[color:var(--text-muted)] bg-white/5",
    )

    expect(taskSuccessRate(taskSummaryFixture())).toBe("—")
    expect(
      taskSuccessRate(
        taskSummaryFixture({ ok_24h: 3, failed_24h: 1, skipped_24h: 8 }),
      ),
    ).toBe("75%")
  })

  it("sorts summary rows and task filter options consistently", () => {
    const input = {
      zeta: taskSummaryFixture({ runs_24h: 2 }),
      alpha: taskSummaryFixture({ runs_24h: 1 }),
    }

    expect(taskFilterOptions(input)).toEqual(["alpha", "zeta"])
    expect(taskSummaryRows(input).map((row) => [row.task, row.runs_24h])).toEqual([
      ["alpha", 1],
      ["zeta", 2],
    ])
  })
})
