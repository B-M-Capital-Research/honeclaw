import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Skeleton } from "@hone-financial/ui/skeleton"
import { For, Show } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { useTasks } from "@/context/tasks"
import type { CronJobInfo } from "@/lib/types"
import { formatShanghaiDateTime } from "@/lib/time"
import { TASKS } from "@/lib/admin-content/tasks"

export function TaskList() {
    const navigate = useNavigate()
    const tasks = useTasks()

    const isHeartbeatJob = (job: CronJobInfo) =>
        job.schedule.repeat === "heartbeat" || (job.tags || []).includes("heartbeat")

    const formatNextRunAt = (dateString?: string) => {
        if (!dateString) return TASKS.list.not_scheduled
        return formatShanghaiDateTime(dateString, {
            month: "2-digit",
            day: "2-digit",
            hour: "2-digit",
            minute: "2-digit",
            second: undefined,
        })
    }

    return (
        <div class="flex h-full min-h-0 w-[320px] flex-col border-r border-[color:var(--border)] bg-[color:var(--surface)]">
            <div class="border-b border-[color:var(--border)] px-4 py-3">
                <div class="flex items-center justify-between">
                    <div>
                        <div class="text-sm font-semibold tracking-tight">{TASKS.list.title}</div>
                        <div class="text-xs text-[color:var(--text-muted)]">{TASKS.list.subtitle}</div>
                    </div>
                    <Button
                        variant="ghost"
                        class="h-7 px-2 text-xs hover:bg-black/5"
                        onClick={() => {
                            navigate("/tasks/new")
                        }}
                    >
                        {TASKS.list.new_button}
                    </Button>
                </div>
            </div>

            <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-3 py-3">
                <Show
                    when={!tasks.jobs.loading}
                    fallback={
                        <div class="space-y-3 px-2 py-2">
                            <Skeleton class="h-20" />
                            <Skeleton class="h-20" />
                            <Skeleton class="h-20" />
                        </div>
                    }
                >
                    <Show
                        when={tasks.jobs() && tasks.jobs()!.length > 0}
                        fallback={<EmptyState title={TASKS.list.empty_title} description={TASKS.list.empty_description} />}
                    >
                        <div class="space-y-2">
                            <For each={tasks.jobs()}>
                                {(job) => {
                                    const active = () => tasks.state.currentTaskId === job.id

                                    return (
                                        <button
                                            type="button"
                                            onClick={() => navigate(`/tasks/${encodeURIComponent(job.id)}`)}
                                            class={[
                                                "w-full rounded-md border p-3 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]",
                                                active()
                                                    ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                                                    : "border-transparent bg-transparent hover:border-[color:var(--border-strong)] hover:bg-black/5",
                                            ].join(" ")}
                                        >
                                            <div class="flex items-start gap-3">
                                                <div
                                                    class={[
                                                        "mt-1 h-2 w-2 shrink-0 rounded-full",
                                                        job.enabled ? "bg-[color:var(--success)]" : "bg-black/20",
                                                    ].join(" ")}
                                                />
                                                <div class="min-w-0 flex-1">
                                                    <div class="flex items-center justify-between gap-2">
                                                        <div class="truncate text-sm font-medium text-[color:var(--text-primary)]">
                                                            {job.name}
                                                        </div>
                                                        <div class="text-[11px] text-[color:var(--text-muted)]">
                                                            {isHeartbeatJob(job)
                                                                ? TASKS.list.every_30_minutes
                                                                : `${job.schedule.hour.toString().padStart(2, "0")}:${job.schedule.minute.toString().padStart(2, "0")}`}
                                                        </div>
                                                    </div>
                                                    <Show when={isHeartbeatJob(job)}>
                                                        <div class="mt-1 inline-flex rounded-full border border-[color:var(--accent)]/30 bg-[color:var(--accent-soft)] px-2 py-0.5 text-[10px] font-medium text-[color:var(--accent)]">
                                                            {TASKS.list.heartbeat_badge}
                                                        </div>
                                                    </Show>
                                                    <div class="mt-0.5 line-clamp-1 text-xs leading-5 text-[color:var(--text-secondary)]">
                                                        {job.task_prompt}
                                                    </div>
                                                    <div class="mt-2 text-[11px] text-[color:var(--text-muted)]">
                                                        {TASKS.list.next_run_label} {isHeartbeatJob(job) ? TASKS.list.next_run_heartbeat : formatNextRunAt(job.next_run_at)}
                                                    </div>
                                                </div>
                                            </div>
                                        </button>
                                    )
                                }}
                            </For>
                        </div>
                    </Show>
                </Show>
            </div>
        </div>
    )
}
