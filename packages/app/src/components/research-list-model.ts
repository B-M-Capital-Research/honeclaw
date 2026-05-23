import { RESEARCH } from "@/lib/admin-content/research"
import type { Locale } from "@/lib/i18n"
import type { ResearchTask } from "@/lib/types"

type ResearchStatusInput = Pick<
  ResearchTask,
  "status"
> &
  Partial<Pick<ResearchTask, "answer_markdown" | "progress">>

type ResearchStatusBadgeConfig = {
  label: string
  dot: string
  text: string
}

export function researchStatusBadgeConfig(
  task: ResearchStatusInput,
): ResearchStatusBadgeConfig {
  switch (task.status) {
    case "completed":
      return {
        label: task.answer_markdown
          ? RESEARCH.list.status.report_ready
          : RESEARCH.list.status.completed,
        dot: "bg-[color:var(--success)]",
        text: "text-[color:var(--success)]",
      }
    case "running":
      return {
        label: task.progress || RESEARCH.list.status.running,
        dot: "bg-blue-400 animate-pulse",
        text: "text-blue-500",
      }
    case "pending":
      return {
        label: RESEARCH.list.status.pending,
        dot: "bg-black/15",
        text: "text-[color:var(--text-muted)]",
      }
    case "error":
      return {
        label: RESEARCH.list.status.error,
        dot: "bg-rose-500",
        text: "text-rose-500",
      }
    default:
      return {
        label: RESEARCH.list.status.unknown,
        dot: "bg-black/15",
        text: "text-[color:var(--text-muted)]",
      }
  }
}

export function researchSymbolFromSearchParam(raw: unknown): string {
  return typeof raw === "string" ? raw.toUpperCase() : ""
}

export function confirmableResearchName(
  companyInput: string,
  starting: boolean,
): string | null {
  const name = companyInput.trim()
  if (!name || starting) return null
  return name
}

export function formatResearchTaskTime(
  iso: string | undefined,
  locale: Locale,
): string {
  if (!iso) return ""
  const date = new Date(iso)
  if (isNaN(date.getTime())) return iso
  return date.toLocaleString(locale === "zh" ? "zh-CN" : "en-US", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  })
}
