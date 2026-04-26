import {
  createMemo,
  createSignal,
  For,
  Show,
  onMount,
} from "solid-js"
import { getTaskRuns, type TaskRunRecord, type TaskSummary } from "@/lib/api"

// ── 常量 ─────────────────────────────────────────────────────────────────────

const DAYS_OPTIONS = [1, 3, 7, 14] as const

// ── 工具 ─────────────────────────────────────────────────────────────────────

function shortTime(iso: string | null | undefined): string {
  if (!iso) return "—"
  // "2026-04-26T12:34:56.789Z" → "12:34:56"
  const d = new Date(iso)
  if (isNaN(d.getTime())) return iso
  return d.toLocaleTimeString("zh-CN", { hour12: false })
}

function relativeTime(iso: string | null | undefined): string {
  if (!iso) return "—"
  const d = new Date(iso)
  if (isNaN(d.getTime())) return iso
  const secAgo = Math.max(0, Math.floor((Date.now() - d.getTime()) / 1000))
  if (secAgo < 60) return `${secAgo}s ago`
  if (secAgo < 3600) return `${Math.floor(secAgo / 60)}m ago`
  if (secAgo < 86400) return `${Math.floor(secAgo / 3600)}h ago`
  return `${Math.floor(secAgo / 86400)}d ago`
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
          任务健康
        </h1>
        <div class="flex items-center gap-1 text-xs text-[color:var(--text-muted)]">
          <span>窗口</span>
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
          <span>过滤 task</span>
          <select
            value={taskFilter()}
            onChange={(e) => {
              setTaskFilter(e.currentTarget.value)
              void refresh()
            }}
            class="rounded border border-[color:var(--border)] bg-transparent px-2 py-1 text-xs text-[color:var(--text-primary)]"
          >
            <option value="">全部</option>
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
          {loading() ? "刷新中…" : "刷新"}
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
          24h 汇总(每个 task 一行)
        </div>
        <div class="overflow-x-auto rounded border border-[color:var(--border)]">
          <table class="w-full text-xs">
            <thead class="bg-white/[0.03] text-[color:var(--text-muted)]">
              <tr>
                <th class="px-3 py-2 text-left font-normal">Task</th>
                <th class="px-3 py-2 text-right font-normal">最近一次</th>
                <th class="px-3 py-2 text-right font-normal">24h 总</th>
                <th class="px-3 py-2 text-right font-normal">ok</th>
                <th class="px-3 py-2 text-right font-normal">skipped</th>
                <th class="px-3 py-2 text-right font-normal">failed</th>
                <th class="px-3 py-2 text-right font-normal">成功率</th>
                <th class="px-3 py-2 text-left font-normal">最近错误</th>
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
                      过去 24h 没有 task_runs.jsonl 记录。检查 web-api 是否
                      已经启动且 with_task_runs_dir 已配置。
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
                        class="max-w-[24rem] truncate px-3 py-2 text-left text-rose-300/80"
                        title={row.last_error ?? ""}
                      >
                        {row.last_error ?? "—"}
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
          最近运行(最多 500 条,倒序)
        </div>
        <div class="flex-1 overflow-auto rounded border border-[color:var(--border)]">
          <table class="w-full text-xs">
            <thead class="sticky top-0 bg-[color:var(--bg-elevated)] text-[color:var(--text-muted)]">
              <tr>
                <th class="px-3 py-2 text-left font-normal">起始</th>
                <th class="px-3 py-2 text-left font-normal">Task</th>
                <th class="px-3 py-2 text-left font-normal">Outcome</th>
                <th class="px-3 py-2 text-right font-normal">Items</th>
                <th class="px-3 py-2 text-right font-normal">耗时</th>
                <th class="px-3 py-2 text-left font-normal">错误</th>
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
                      没有匹配的记录。
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
