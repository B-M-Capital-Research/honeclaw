import {
  createMemo,
  createSignal,
  For,
  Show,
  onMount,
} from "solid-js"
import { getTaskRuns, type TaskRunRecord, type TaskSummary } from "@/lib/api"
import { TASK_HEALTH } from "@/lib/admin-content/task-health"
import { tpl, useLocale } from "@/lib/i18n"
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

// ── 主组件 ────────────────────────────────────────────────────────────────────

export default function TaskHealthPage() {
  const [days, setDays] = createSignal<number>(1)
  const [taskFilter, setTaskFilter] = createSignal<string>("")
  const [runs, setRuns] = createSignal<TaskRunRecord[]>([])
  const [summary, setSummary] = createSignal<Record<string, TaskSummary>>({})
  const [runtimeDir, setRuntimeDir] = createSignal<string>("")
  const [loading, setLoading] = createSignal(false)
  const [loadError, setLoadError] = createSignal<string | null>(null)

  async function refresh() {
    setLoading(true)
    setLoadError(null)
    try {
      const response = await getTaskRuns({
        days: days(),
        limit: 500,
        task: taskFilter() || undefined,
      })
      setRuns(response.runs)
      setSummary(response.summary_by_task)
      setRuntimeDir(response.runtime_dir)
    } catch (e) {
      setLoadError(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(() => {
    void refresh()
    // 30s 自动刷新——周期任务节奏最快也是分钟级,30s 足够及时。
    const timer = window.setInterval(refresh, 30_000)
    return () => window.clearInterval(timer)
  })

  const now = () => new Date()
  const summaryRows = createMemo(() => taskSummaryRows(summary()))
  const taskOptions = createMemo(() => taskFilterOptions(summary()))

  return (
    <div class="flex h-full min-h-0 flex-col gap-4 p-4 text-sm">
      {/* 顶栏 */}
      <div class="flex flex-wrap items-center gap-3">
        <h1 class="text-lg font-semibold text-[color:var(--text-primary)]">
          {TASK_HEALTH.page.title}
        </h1>
        <div class="flex items-center gap-1 text-xs text-[color:var(--text-muted)]">
          <span>{TASK_HEALTH.page.window_label}</span>
          <select
            value={days()}
            onChange={(e) => {
              setDays(Number(e.currentTarget.value))
              void refresh()
            }}
            class="rounded border border-[color:var(--border)] bg-transparent px-2 py-1 text-xs text-[color:var(--text-primary)]"
          >
            <For each={TASK_HEALTH_DAYS_OPTIONS}>{(optionDays) => <option value={optionDays}>{optionDays}d</option>}</For>
          </select>
        </div>
        <div class="flex items-center gap-1 text-xs text-[color:var(--text-muted)]">
          <span>{TASK_HEALTH.page.filter_task_label}</span>
          <select
            value={taskFilter()}
            onChange={(e) => {
              setTaskFilter(e.currentTarget.value)
              void refresh()
            }}
            class="rounded border border-[color:var(--border)] bg-transparent px-2 py-1 text-xs text-[color:var(--text-primary)]"
          >
            <option value="">{TASK_HEALTH.page.filter_all}</option>
            <For each={taskOptions()}>
              {(taskName) => <option value={taskName}>{taskName}</option>}
            </For>
          </select>
        </div>
        <button
          onClick={() => void refresh()}
          disabled={loading()}
          class="rounded border border-[color:var(--border)] px-3 py-1 text-xs hover:bg-white/5 disabled:opacity-50"
        >
          {loading() ? TASK_HEALTH.page.refreshing_button : TASK_HEALTH.page.refresh_button}
        </button>
        <Show when={runtimeDir()}>
          <span
            class="ml-auto text-[10px] text-[color:var(--text-muted)]"
            title={runtimeDir()}
          >
            {runtimeDir()}
          </span>
        </Show>
      </div>

      <Show when={loadError()}>
        <div class="rounded border border-rose-500/40 bg-rose-500/10 p-3 text-rose-300">
          {loadError()}
        </div>
      </Show>

      {/* 24h summary */}
      <section class="space-y-2">
        <div class="text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">
          {TASK_HEALTH.summary.eyebrow}
        </div>
        <div class="overflow-x-auto rounded border border-[color:var(--border)]">
          <table class="w-full text-xs">
            <thead class="bg-white/[0.03] text-[color:var(--text-muted)]">
              <tr>
                <th class="px-3 py-2 text-left font-normal">{TASK_HEALTH.summary.col_task}</th>
                <th class="px-3 py-2 text-right font-normal">{TASK_HEALTH.summary.col_last_seen}</th>
                <th class="px-3 py-2 text-right font-normal">{TASK_HEALTH.summary.col_runs_24h}</th>
                <th class="px-3 py-2 text-right font-normal">{TASK_HEALTH.summary.col_ok}</th>
                <th class="px-3 py-2 text-right font-normal">{TASK_HEALTH.summary.col_skipped}</th>
                <th class="px-3 py-2 text-right font-normal">{TASK_HEALTH.summary.col_failed}</th>
                <th class="px-3 py-2 text-right font-normal">{TASK_HEALTH.summary.col_success_rate}</th>
                <th class="px-3 py-2 text-left font-normal">{TASK_HEALTH.summary.col_last_error}</th>
              </tr>
            </thead>
            <tbody>
              <Show
                when={summaryRows().length > 0}
                fallback={
                  <tr>
                    <td
                      colspan={8}
                      class="px-3 py-6 text-center text-[color:var(--text-muted)]"
                    >
                      {TASK_HEALTH.summary.empty_no_records}
                    </td>
                  </tr>
                }
              >
                <For each={summaryRows()}>
                  {(row) => (
                    <tr class="border-t border-[color:var(--border)] hover:bg-white/[0.02]">
                      <td class="px-3 py-2 font-mono text-[11px]">{row.task}</td>
                      <td
                        class="px-3 py-2 text-right text-[color:var(--text-muted)]"
                        title={row.last_seen_at ?? ""}
                      >
                        {relativeTaskRunTime(row.last_seen_at, now())}
                      </td>
                      <td class="px-3 py-2 text-right">{row.runs_24h}</td>
                      <td class="px-3 py-2 text-right text-emerald-300">
                        {row.ok_24h}
                      </td>
                      <td class="px-3 py-2 text-right text-[color:var(--text-muted)]">
                        {row.skipped_24h}
                      </td>
                      <td class="px-3 py-2 text-right text-rose-300">
                        {row.failed_24h}
                      </td>
                      <td class="px-3 py-2 text-right">{taskSuccessRate(row)}</td>
                      <td
                        class="max-w-[24rem] px-3 py-2 text-left"
                        title={
                          row.last_failure_at
                            ? `${row.last_failure_at}\n${row.last_error ?? ""}`
                            : ""
                        }
                      >
                        <Show
                          when={row.last_error}
                          fallback={
                            <span class="text-[color:var(--text-muted)]">—</span>
                          }
                        >
                          <div class="flex items-center gap-2 text-[10px] uppercase tracking-wide">
                            <span class="text-[color:var(--text-muted)]">
                              {relativeTaskRunTime(row.last_failure_at, now())}
                            </span>
                            <Show
                              when={(row.runs_since_last_failure ?? 0) > 0}
                              fallback={
                                <span class="rounded bg-rose-500/15 px-1.5 py-0.5 text-rose-300">
                                  {TASK_HEALTH.summary.badge_latest_failure}
                                </span>
                              }
                            >
                              <span class="rounded bg-emerald-500/10 px-1.5 py-0.5 text-emerald-300/80">
                                {tpl(TASK_HEALTH.summary.badge_recovered, { count: row.runs_since_last_failure ?? 0 })}
                              </span>
                            </Show>
                          </div>
                          <div class="mt-0.5 truncate text-rose-300/80">
                            {row.last_error}
                          </div>
                        </Show>
                      </td>
                    </tr>
                  )}
                </For>
              </Show>
            </tbody>
          </table>
        </div>
      </section>

      {/* runs 时间线 */}
      <section class="flex min-h-0 flex-col gap-2">
        <div class="text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">
          {TASK_HEALTH.runs.eyebrow}
        </div>
        <div class="flex-1 overflow-auto rounded border border-[color:var(--border)]">
          <table class="w-full text-xs">
            <thead class="sticky top-0 bg-[color:var(--bg-elevated)] text-[color:var(--text-muted)]">
              <tr>
                <th class="px-3 py-2 text-left font-normal">{TASK_HEALTH.runs.col_started}</th>
                <th class="px-3 py-2 text-left font-normal">{TASK_HEALTH.runs.col_task}</th>
                <th class="px-3 py-2 text-left font-normal">{TASK_HEALTH.runs.col_outcome}</th>
                <th class="px-3 py-2 text-right font-normal">{TASK_HEALTH.runs.col_items}</th>
                <th class="px-3 py-2 text-right font-normal">{TASK_HEALTH.runs.col_duration}</th>
                <th class="px-3 py-2 text-left font-normal">{TASK_HEALTH.runs.col_error}</th>
              </tr>
            </thead>
            <tbody>
              <Show
                when={runs().length > 0}
                fallback={
                  <tr>
                    <td
                      colspan={6}
                      class="px-3 py-6 text-center text-[color:var(--text-muted)]"
                    >
                      {TASK_HEALTH.runs.empty_no_match}
                    </td>
                  </tr>
                }
              >
                <For each={runs()}>
                  {(run) => (
                    <tr class="border-t border-[color:var(--border)] hover:bg-white/[0.02]">
                      <td
                        class="px-3 py-1.5 font-mono text-[10px] text-[color:var(--text-muted)]"
                        title={run.started_at}
                      >
                        {shortTaskRunTime(run.started_at, useLocale())}
                      </td>
                      <td class="px-3 py-1.5 font-mono text-[11px]">{run.task}</td>
                      <td class="px-3 py-1.5">
                        <span
                          class={`inline-block rounded px-1.5 py-0.5 text-[10px] uppercase ${taskRunOutcomeClass(run.outcome)}`}
                        >
                          {run.outcome}
                        </span>
                      </td>
                      <td class="px-3 py-1.5 text-right">{run.items}</td>
                      <td class="px-3 py-1.5 text-right text-[color:var(--text-muted)]">
                        {taskRunDurationMs(run.started_at, run.ended_at)}ms
                      </td>
                      <td
                        class="max-w-[28rem] truncate px-3 py-1.5 text-rose-300/80"
                        title={run.error ?? ""}
                      >
                        {run.error ?? ""}
                      </td>
                    </tr>
                  )}
                </For>
              </Show>
            </tbody>
          </table>
        </div>
      </section>
    </div>
  )
}
