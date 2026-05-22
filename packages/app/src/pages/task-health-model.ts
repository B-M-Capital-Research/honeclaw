import type { TaskSummary } from "@/lib/api"
import { TASK_HEALTH } from "@/lib/admin-content/task-health"
import { tpl, type Locale } from "@/lib/i18n"

export const TASK_HEALTH_DAYS_OPTIONS = [1, 3, 7, 14] as const

type TaskSummaryRow = TaskSummary & {
  task: string
}

export function shortTaskRunTime(
  iso: string | null | undefined,
  locale: Locale,
): string {
  if (!iso) return "—"
  const date = new Date(iso)
  if (isNaN(date.getTime())) return iso
  return date.toLocaleTimeString(locale === "zh" ? "zh-CN" : "en-US", {
    hour12: false,
  })
}

export function relativeTaskRunTime(
  iso: string | null | undefined,
  now: Date,
): string {
  if (!iso) return "—"
  const date = new Date(iso)
  if (isNaN(date.getTime())) return iso
  const secAgo = Math.max(0, Math.floor((now.getTime() - date.getTime()) / 1000))
  if (secAgo < 60) return tpl(TASK_HEALTH.relative.seconds_ago, { count: secAgo })
  if (secAgo < 3600) {
    return tpl(TASK_HEALTH.relative.minutes_ago, {
      count: Math.floor(secAgo / 60),
    })
  }
  if (secAgo < 86400) {
    return tpl(TASK_HEALTH.relative.hours_ago, {
      count: Math.floor(secAgo / 3600),
    })
  }
  return tpl(TASK_HEALTH.relative.days_ago, {
    count: Math.floor(secAgo / 86400),
  })
}

export function taskRunDurationMs(start: string, end: string): number {
  const startMs = new Date(start).getTime()
  const endMs = new Date(end).getTime()
  if (isNaN(startMs) || isNaN(endMs)) return 0
  return Math.max(0, endMs - startMs)
}

export function taskRunOutcomeClass(outcome: string): string {
  switch (outcome) {
    case "ok":
      return "text-emerald-300 bg-emerald-500/15"
    case "skipped":
      return "text-[color:var(--text-muted)] bg-white/5"
    case "failed":
      return "text-rose-300 bg-rose-500/15"
    default:
      return "text-[color:var(--text-muted)] bg-white/5"
  }
}

export function taskSuccessRate(summary: TaskSummary): string {
  const denominator = summary.ok_24h + summary.failed_24h
  if (denominator === 0) return "—"
  const pct = (summary.ok_24h / denominator) * 100
  return `${pct.toFixed(0)}%`
}

export function taskSummaryRows(
  summary: Record<string, TaskSummary>,
): TaskSummaryRow[] {
  return Object.entries(summary)
    .map(([task, taskSummary]) => ({ task, ...taskSummary }))
    .sort((a, b) => a.task.localeCompare(b.task))
}

export function taskFilterOptions(summary: Record<string, TaskSummary>): string[] {
  return Object.keys(summary).sort()
}
