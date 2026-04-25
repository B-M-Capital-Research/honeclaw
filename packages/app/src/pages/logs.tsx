import {
  createEffect,
  createMemo,
  createSignal,
  For,
  on,
  onCleanup,
  onMount,
  Show,
} from "solid-js"
import { getLogs, connectLogStream } from "@/lib/api"
import type { LogEntry } from "@/lib/types"
import { useBackend } from "@/context/backend"
import { EntityRefLink } from "@/components/entity-ref-link"
import { extractLogRefs, logMatchesUser } from "@/lib/log-refs"

// ── 常量 ─────────────────────────────────────────────────────────────────────

const MAX_ENTRIES = 2000
const LEVEL_ORDER = ["ALL", "DEBUG", "INFO", "WARN", "ERROR"] as const
type LevelFilter = (typeof LEVEL_ORDER)[number]

// ── 工具 ─────────────────────────────────────────────────────────────────────

function levelColor(level: string): string {
  switch (level.toUpperCase()) {
    case "ERROR": return "text-rose-400 bg-rose-500/20"
    case "WARN": return "text-amber-300 bg-amber-500/15"
    case "INFO": return "text-blue-400 bg-blue-500/15"
    case "DEBUG": return "text-[color:var(--text-muted)] bg-white/5"
    default: return "text-[color:var(--text-muted)] bg-white/5"
  }
}

function levelRowBg(level: string): string {
  switch (level.toUpperCase()) {
    case "ERROR": return "bg-rose-500/5 hover:bg-rose-500/10"
    case "WARN": return "hover:bg-amber-500/5"
    default: return "hover:bg-white/[0.02]"
  }
}

function shortTime(ts: string): string {
  // "2026-03-07 13:45:09.123" → "13:45:09.123"
  const parts = ts.split(" ")
  return parts[1] ?? ts
}

// ── 主组件 ────────────────────────────────────────────────────────────────────

export default function LogsPage() {
  const backend = useBackend()
  const [entries, setEntries] = createSignal<LogEntry[]>([])
  const [filterLevel, setFilterLevel] = createSignal<LevelFilter>("ALL")
  const [search, setSearch] = createSignal("")
  const [userFilter, setUserFilter] = createSignal("")
  const [paused, setPaused] = createSignal(false)
  const [connected, setConnected] = createSignal(false)
  let listRef: HTMLDivElement | undefined
  let autoScroll = true
  // 批量缓冲：将高频 SSE 日志先放入队列，每 100ms 统一 flush 到 signal，
  // 防止每条日志单独触发过滤计算和 DOM 更新导致主线程卡顿
  let pendingBatch: LogEntry[] = []
  let flushTimer: ReturnType<typeof setTimeout> | null = null

  // 用 createMemo 缓存过滤结果，避免每次渲染都重新扫描全部条目
  const filtered = createMemo(() => {
    const lvl = filterLevel()
    const q = search().trim().toLowerCase()
    const u = userFilter().trim()
    return entries().filter((e) => {
      if (lvl !== "ALL" && e.level.toUpperCase() !== lvl) return false
      if (q) {
        const haystack = [
          e.message,
          e.target,
          e.level,
          e.timestamp,
          e.message_id ?? "",
          e.state ?? ""
        ].join(" ").toLowerCase()
        if (!haystack.includes(q)) return false
      }
      if (u && !logMatchesUser(e, u)) return false
      return true
    })
  })

  function scrollToBottom() {
    if (listRef && autoScroll) {
      listRef.scrollTop = listRef.scrollHeight
    }
  }

  // 将缓冲区一次性写入 signal（最多保留 MAX_ENTRIES 条）
  function flushPending() {
    flushTimer = null
    if (pendingBatch.length === 0) return
    const toAdd = pendingBatch.splice(0)
    setEntries((prev) => {
      const next = [...prev, ...toAdd]
      return next.length > MAX_ENTRIES ? next.slice(next.length - MAX_ENTRIES) : next
    })
    requestAnimationFrame(scrollToBottom)
  }

  // 追加新条目：先写入缓冲，100ms 内批量 flush 一次
  function appendEntry(entry: LogEntry) {
    if (paused()) return
    pendingBatch.push(entry)
    if (flushTimer === null) {
      flushTimer = setTimeout(flushPending, 100)
    }
  }

  // 加载历史
  onMount(async () => {
    if (!backend.hasCapability("logs")) return
    try {
      const logs = await getLogs()
      setEntries(logs)
      requestAnimationFrame(scrollToBottom)
    } catch (e) {
      console.warn("getLogs failed", e)
    }
  })

  // 连接 SSE 流
  onMount(() => {
    if (!backend.hasCapability("logs")) return
    let es: EventSource | null = null
    let retryTimer: ReturnType<typeof setTimeout> | null = null

    async function connect() {
      es = await connectLogStream()

      es.addEventListener("connected", () => setConnected(true))

      es.addEventListener("log", (e: MessageEvent) => {
        try {
          const entry: LogEntry = JSON.parse(e.data as string)
          appendEntry(entry)
        } catch (_) { }
      })

      es.onerror = () => {
        setConnected(false)
        es?.close()
        retryTimer = setTimeout(() => void connect(), 5000)
      }
    }

    void connect()

    onCleanup(() => {
      es?.close()
      if (retryTimer != null) clearTimeout(retryTimer)
      if (flushTimer !== null) {
        clearTimeout(flushTimer)
        flushTimer = null
      }
    })
  })

  // 过滤器变化时滚到底
  createEffect(on(filterLevel, () => requestAnimationFrame(scrollToBottom)))

  // 滚动检测：用户上滚时关闭自动滚动
  function onScroll() {
    if (!listRef) return
    const atBottom = listRef.scrollHeight - listRef.scrollTop - listRef.clientHeight < 80
    autoScroll = atBottom
  }

  function togglePause() {
    const next = !paused()
    setPaused(next)
    if (!next) {
      autoScroll = true
      // 恢复时立即将暂停期间积累的缓冲条目写入
      flushPending()
    }
  }

  function clearAll() {
    setEntries([])
  }

  return (
    <div class="flex h-full flex-col overflow-hidden">
      <Show
        when={backend.hasCapability("logs")}
        fallback={
          <div class="flex h-full items-center justify-center rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] text-sm text-[color:var(--text-secondary)]">
            当前 backend 未开放日志能力。
          </div>
        }
      >
        {/* 工具栏 */}
        <div class="flex flex-shrink-0 flex-wrap items-center gap-2 border-b border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-2.5">
          <span class="text-sm font-semibold text-[color:var(--text-primary)] mr-1">日志</span>

          {/* 级别过滤 */}
          <div class="flex gap-1.5 flex-wrap">
            <For each={LEVEL_ORDER}>
              {(lvl) => (
                <button
                  class={[
                    "rounded-full px-2.5 py-0.5 text-[11px] font-semibold tracking-wide transition border",
                    filterLevel() === lvl
                      ? lvl === "ALL"
                        ? "border-[color:var(--border)] bg-[color:var(--surface)] text-[color:var(--text-primary)]"
                        : lvl === "ERROR"
                          ? "border-rose-400/60 bg-rose-400/10 text-rose-400"
                          : lvl === "WARN"
                            ? "border-amber-300/60 bg-amber-300/10 text-amber-300"
                            : lvl === "INFO"
                              ? "border-blue-400/60 bg-blue-400/10 text-blue-400"
                              : "border-white/20 bg-white/5 text-[color:var(--text-muted)]"
                      : "border-transparent text-[color:var(--text-muted)] hover:text-[color:var(--text-secondary)]",
                  ].join(" ")}
                  onClick={() => setFilterLevel(lvl)}
                >
                  {lvl}
                </button>
              )}
            </For>
          </div>

          {/* 搜索 */}
          <input
            type="text"
            placeholder="搜索日志…"
            class="w-40 rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1 text-xs text-[color:var(--text-primary)] placeholder:text-[color:var(--text-muted)] outline-none focus:border-[color:var(--accent)] transition"
            value={search()}
            onInput={(e) => setSearch(e.currentTarget.value)}
          />

          {/* 按用户筛选 */}
          <input
            type="text"
            placeholder="按 user_id 筛选…"
            class="w-36 rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1 text-xs text-[color:var(--text-primary)] placeholder:text-[color:var(--text-muted)] outline-none focus:border-[color:var(--accent)] transition"
            value={userFilter()}
            onInput={(e) => setUserFilter(e.currentTarget.value)}
            title="只显示与该用户相关的日志(匹配结构化 actor 或 message 文本)"
          />

          {/* 操作按钮 */}
          <div class="ml-auto flex items-center gap-2">
            <button
              class={[
                "flex items-center gap-1.5 rounded px-2.5 py-1 text-xs font-medium transition border",
                paused()
                  ? "border-[color:var(--accent)]/50 text-[color:var(--accent)]"
                  : "border-transparent bg-[color:var(--surface)] text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]",
              ].join(" ")}
              onClick={togglePause}
            >
              <Show when={paused()} fallback={
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <rect x="6" y="4" width="4" height="16" /><rect x="14" y="4" width="4" height="16" />
                </svg>
              }>
                <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
                  <polygon points="5 3 19 12 5 21 5 3" />
                </svg>
              </Show>
              {paused() ? "继续" : "暂停"}
            </button>

            <button
              class="flex items-center gap-1.5 rounded border border-transparent bg-[color:var(--surface)] px-2.5 py-1 text-xs font-medium text-[color:var(--text-secondary)] transition hover:text-[color:var(--text-primary)]"
              onClick={clearAll}
            >
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="3 6 5 6 21 6" />
                <path d="M19 6l-1 14H6L5 6" />
                <path d="M9 6V4h6v2" />
              </svg>
              清空
            </button>

            <span class="text-[11px] text-[color:var(--text-muted)]">
              {filtered().length} 条
            </span>

            {/* 实时连接指示器 */}
            <div class="flex items-center gap-1.5">
              <span
                class={[
                  "h-1.5 w-1.5 rounded-full",
                  connected() ? "bg-[color:var(--success)] animate-pulse" : "bg-[color:var(--text-muted)]",
                ].join(" ")}
              />
              <span class="text-[11px] text-[color:var(--text-muted)]">
                {connected() ? "实时" : "断开"}
              </span>
            </div>
          </div>
        </div>

        {/* 日志列表 */}
        <div
          ref={listRef}
          class="min-h-0 flex-1 overflow-y-auto font-mono text-[12px] leading-relaxed"
          onScroll={onScroll}
        >
          <Show
            when={filtered().length > 0}
            fallback={
              <div class="flex h-full flex-col items-center justify-center gap-3 text-[color:var(--text-muted)]">
                <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="opacity-20">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                  <polyline points="14 2 14 8 20 8" />
                  <line x1="16" y1="13" x2="8" y2="13" />
                  <line x1="16" y1="17" x2="8" y2="17" />
                </svg>
                <span class="font-sans text-sm">暂无匹配日志</span>
              </div>
            }
          >
            <For each={filtered()}>
              {(entry) => (
                <div
                  class={[
                    "flex items-start gap-2 border-b border-white/[0.03] px-4 py-[3px] transition-colors",
                    levelRowBg(entry.level),
                  ].join(" ")}
                >
                  {/* 时间戳 */}
                  <span class="w-[100px] shrink-0 text-[11px] text-[color:var(--text-muted)] pt-px">
                    {shortTime(entry.timestamp)}
                  </span>

                  {/* Level Badge */}
                  <span
                    class={[
                      "w-[38px] shrink-0 rounded px-1 py-px text-[10px] font-bold tracking-wide text-center",
                      levelColor(entry.level),
                    ].join(" ")}
                  >
                    {entry.level.toUpperCase()}
                  </span>

                  {/* Target */}
                  <span class="w-[160px] shrink-0 truncate text-[11px] text-amber-300/70 pt-px" title={entry.target}>
                    {entry.target}
                  </span>

                  {/* 状态 (如果存在) */}
                  <Show when={entry.state}>
                    <span class="w-[80px] shrink-0 truncate text-[10px] text-sky-400 font-bold border border-sky-500/30 rounded px-1.5 py-px bg-sky-500/5 text-center" title={entry.state}>
                      {entry.state}
                    </span>
                  </Show>

                  {/* 消息 */}
                  <div class="min-w-0 flex-1 flex flex-col pt-0.5">
                    <span class={[
                      "break-all whitespace-pre-wrap",
                      entry.level.toUpperCase() === "ERROR"
                        ? "text-rose-300"
                        : entry.level.toUpperCase() === "WARN"
                          ? "text-amber-200/80"
                          : "text-[color:var(--text-secondary)]",
                    ].join(" ")}>
                      {entry.message}
                    </span>
                    {(() => {
                      const refs = extractLogRefs(entry)
                      return (
                        <Show when={refs.length > 0}>
                          <div class="mt-1 flex flex-wrap gap-1 font-sans">
                            <For each={refs}>
                              {(ref) => {
                                if (ref.kind === "actor") {
                                  return (
                                    <EntityRefLink
                                      kind="actor"
                                      id={ref.actor.user_id}
                                      channel={ref.actor.channel}
                                      scope={ref.actor.channel_scope}
                                    />
                                  )
                                }
                                if (ref.kind === "session") {
                                  return (
                                    <EntityRefLink
                                      kind="session"
                                      id={ref.sessionId}
                                      label={ref.actor?.user_id ?? ref.sessionId.slice(0, 16) + "…"}
                                    />
                                  )
                                }
                                return <EntityRefLink kind="task" id={ref.taskId} />
                              }}
                            </For>
                          </div>
                        </Show>
                      )
                    })()}
                    <Show when={entry.message_id}>
                      <span class="text-[9px] text-[color:var(--text-muted)] font-mono opacity-60 mt-0.5">
                        MSG_ID: {entry.message_id}
                      </span>
                    </Show>
                  </div>
                </div>
              )}
            </For>
          </Show>
        </div>
      </Show>
    </div>
  )
}
