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

// ── 常量 ─────────────────────────────────────────────────────────────────────

const DAYS_OPTIONS = [1, 3, 7, 14] as const

// ── 工具 ─────────────────────────────────────────────────────────────────────

function shortTime(iso: string | null | undefined): string {
  if (!iso) return "—"
  // "2026-04-26T12:34:56.789Z" → "12:34:56"
  const d = new Date(iso)
  if (isNaN(d.getTime())) return iso
  const loc = useLocale() === "zh" ? "zh-CN" : "en-US"
  return d.toLocaleTimeString(loc, { hour12: false })
}

function relativeTime(iso: string | null | undefined): string {
  if (!iso) return "—"
  const d = new Date(iso)
  if (isNaN(d.getTime())) return iso
  const secAgo = Math.max(0, Math.floor((Date.now() - d.getTime()) / 1000))
  if (secAgo < 60) return tpl(TASK_HEALTH.relative.seconds_ago, { count: secAgo })
  if (secAgo < 3600) return tpl(TASK_HEALTH.relative.minutes_ago, { count: Math.floor(secAgo / 60) })
  if (secAgo < 86400) return tpl(TASK_HEALTH.relative.hours_ago, { count: Math.floor(secAgo / 3600) })
  return tpl(TASK_HEALTH.relative.days_ago, { count: Math.floor(secAgo / 86400) })
}

function durationMs(start: string, end: string): number {
  const a = new Date(start).getTime()
  const b = new Date(end).getTime()
  if (isNaN(a) || isNaN(b)) return 0
  return Math.max(0, b - a)
}

function outcomeColor(outcome: string): string {
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

function successRate(s: TaskSummary): string {
  // skipped 是"主动跳过",不算分母里的失败,只算 ok / (ok+failed)。
  const denom = s.ok_24h + s.failed_24h
  if (denom === 0) return "—"
  const pct = (s.ok_24h / denom) * 100
  return `${pct.toFixed(0)}%`
}

// ── 主组件 ────────────────────────────────────────────────────────────────────

export default function TaskHealthPage() {
  const [days, setDays] = createSignal<number>(1)
  const [taskFilter, setTaskFilter] = createSignal<string>("")
  const [runs, setRuns] = createSignal<TaskRunRecord[]>([])
  const [summary, setSummary] = createSignal<Record<string, TaskSummary>>({})
  const [runtimeDir, setRuntimeDir] = createSignal<string>("")
  const [loading, setLoading] = createSignal(false)
  const [err, setErr] = createSignal<string | null>(null)

  async function refresh() {
    setLoading(true)
    setErr(null)
    try {
      const resp = await getTaskRuns({
        days: days(),
        limit: 500,
        task: taskFilter() || undefined,
      })
      setRuns(resp.runs)
      setSummary(resp.summary_by_task)
      setRuntimeDir(resp.runtime_dir)
    } catch (e) {
      setErr(String(e))
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

  // 把 summary map 排成数组按 task 字典序
  const summaryRows = createMemo(() => {
    return Object.entries(summary())
      .map(([task, s]) => ({ task, ...s }))
      .sort((a, b) => a.task.localeCompare(b.task))
  })

  // task 下拉选项 = summary 里出现过的所有 task
  const taskOptions = createMemo(() => {
    return Object.keys(summary()).sort()
  })

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
            <For each={DAYS_OPTIONS}>{(d) => <option value={d}>{d}d</option>}</For>
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
              {(t) => <option value={t}>{t}</option>}
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

      <Show when={err()}>
        <div class="rounded border border-rose-500/40 bg-rose-500/10 p-3 text-rose-300">
          {err()}
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
                        {relativeTime(row.last_seen_at)}
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
                      <td class="px-3 py-2 text-right">{successRate(row)}</td>
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
                              {relativeTime(row.last_failure_at)}
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
                  {(r) => (
                    <tr class="border-t border-[color:var(--border)] hover:bg-white/[0.02]">
                      <td
                        class="px-3 py-1.5 font-mono text-[10px] text-[color:var(--text-muted)]"
                        title={r.started_at}
                      >
                        {shortTime(r.started_at)}
                      </td>
                      <td class="px-3 py-1.5 font-mono text-[11px]">{r.task}</td>
                      <td class="px-3 py-1.5">
                        <span
                          class={`inline-block rounded px-1.5 py-0.5 text-[10px] uppercase ${outcomeColor(r.outcome)}`}
                        >
                          {r.outcome}
                        </span>
                      </td>
                      <td class="px-3 py-1.5 text-right">{r.items}</td>
                      <td class="px-3 py-1.5 text-right text-[color:var(--text-muted)]">
                        {durationMs(r.started_at, r.ended_at)}ms
                      </td>
                      <td
                        class="max-w-[28rem] truncate px-3 py-1.5 text-rose-300/80"
                        title={r.error ?? ""}
                      >
                        {r.error ?? ""}
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
