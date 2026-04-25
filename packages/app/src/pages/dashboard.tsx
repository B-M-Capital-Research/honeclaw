import { useNavigate } from "@solidjs/router"
import { For, Show, createMemo, createResource, createSignal } from "solid-js"
import { loadDesktopAgentSettings } from "@/lib/backend"
import { useBackend } from "@/context/backend"
import { useConsole } from "@/context/console"
import { useResearch } from "@/context/research"
import { useSessions, ME_SESSION_ID } from "@/context/sessions"
import type { AgentProvider } from "@/lib/types"

type ChannelDef = {
  runner: AgentProvider
  name: string
  desc: string
  icon: string
}

const CHANNELS: ChannelDef[] = [
  { runner: "multi-agent", name: "Multi-Agent", desc: "MiniMax 搜索 + Gemini 回答", icon: "∞" },
  { runner: "codex_acp", name: "Codex ACP", desc: "通过 codex-acp 驱动当前会话", icon: "⌘" },
  { runner: "opencode_acp", name: "自定义 OpenAI 协议", desc: "OpenAI compatible / 推荐 OpenRouter", icon: "⚡" },
  { runner: "gemini_cli", name: "Gemini CLI", desc: "复用本机 Gemini 命令行", icon: "✦" },
  { runner: "codex_cli", name: "Codex CLI", desc: "复用本机 Codex 命令行", icon: "◈" },
]

function formatTime(iso?: string): string {
  if (!iso) return ""
  try {
    return new Date(iso).toLocaleString("zh-CN", {
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    })
  } catch {
    return iso
  }
}

export default function DashboardPage() {
  const navigate = useNavigate()
  const backend = useBackend()
  const consoleState = useConsole()
  const sessions = useSessions()
  const research = useResearch()

  const [input, setInput] = createSignal("")

  const [agentSettings] = createResource(
    () => backend.state.isDesktop,
    async (isDesktop) => {
      if (!isDesktop) return undefined
      return loadDesktopAgentSettings()
    },
  )

  const activeRunner = () => agentSettings()?.runner ?? "opencode_acp"

  const handleSend = () => {
    const text = input().trim()
    if (!text) return
    sessions.setPendingPrefill(text)
    navigate(`/sessions/${encodeURIComponent(ME_SESSION_ID)}`)
  }

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.isComposing) return
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const recentSessions = createMemo(() =>
    sessions.state.users
      .slice()
      .sort((a, b) => (b.last_time ?? "").localeCompare(a.last_time ?? ""))
      .filter((u) => u.session_id !== ME_SESSION_ID)
      .slice(0, 5),
  )

  const activeResearch = createMemo(() =>
    research.state.tasks
      .filter((t) => t.status !== "error")
      .slice()
      .sort((a, b) => (b.created_at ?? "").localeCompare(a.created_at ?? ""))
      .slice(0, 5),
  )

  const channelStatus = createMemo(() => {
    const channels = consoleState.channels()
    const live = channels.filter((c) => c.running).length
    return { total: channels.length, live }
  })

  return (
    <div class="hf-scrollbar h-full overflow-y-auto px-6 py-6">
      <div class="mx-auto flex w-full max-w-5xl flex-col gap-6">
        {/* 状态面板 */}
        <div class="grid gap-3 md:grid-cols-3">
          <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
            <div class="text-[11px] uppercase tracking-widest text-[color:var(--text-muted)]">
              后端连接
            </div>
            <div class="mt-1.5 flex items-center gap-2">
              <span
                class={[
                  "h-2 w-2 rounded-full",
                  backend.state.connected ? "bg-emerald-400" : "bg-rose-400",
                ].join(" ")}
              />
              <span class="text-sm font-medium text-[color:var(--text-primary)]">
                {backend.state.connected ? "已连接" : "未连接"}
              </span>
            </div>
            <Show when={!backend.state.connected && backend.state.error}>
              <div class="mt-1 line-clamp-2 text-[11px] text-rose-400">
                {backend.state.error}
              </div>
            </Show>
          </div>

          <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
            <div class="text-[11px] uppercase tracking-widest text-[color:var(--text-muted)]">
              渠道
            </div>
            <div class="mt-1.5 text-sm font-medium text-[color:var(--text-primary)]">
              {channelStatus().live} / {channelStatus().total} 在线
            </div>
            <Show when={consoleState.channelError()}>
              <div class="mt-1 line-clamp-2 text-[11px] text-rose-400">
                {consoleState.channelError()}
              </div>
            </Show>
          </div>

          <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
            <div class="text-[11px] uppercase tracking-widest text-[color:var(--text-muted)]">
              进行中研究
            </div>
            <div class="mt-1.5 text-sm font-medium text-[color:var(--text-primary)]">
              {research.state.tasks.filter((t) => t.status === "running" || t.status === "pending").length} 个任务
            </div>
            <button
              type="button"
              class="mt-1 text-[11px] text-[color:var(--accent)] hover:underline"
              onClick={() => navigate("/research")}
            >
              打开研究模块 →
            </button>
          </div>
        </div>

        {/* 快速发起 */}
        <div class="rounded-xl border border-[color:var(--border)] bg-[color:var(--surface)] p-5 shadow-sm">
          <div class="mb-3 flex items-center justify-between">
            <div>
              <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                快速发起对话
              </div>
              <div class="text-[11px] text-[color:var(--text-muted)]">
                输入问题直接发给 ME 渠道,Enter 发送
              </div>
            </div>
            <div class="flex flex-wrap justify-end gap-2">
              <For each={CHANNELS}>
                {(ch) => {
                  const isActive = () => activeRunner() === ch.runner
                  return (
                    <button
                      type="button"
                      class={[
                        "group flex items-center gap-1 rounded-full border px-2.5 py-1 text-[11px] transition-all",
                        isActive()
                          ? "border-[color:var(--accent)] bg-[color:var(--accent)]/10 text-[color:var(--accent)]"
                          : "border-[color:var(--border)] bg-[color:var(--panel)] text-[color:var(--text-secondary)] hover:border-[color:var(--accent)]/50",
                        !backend.state.isDesktop ? "cursor-not-allowed opacity-40" : "cursor-pointer",
                      ].join(" ")}
                      disabled={!backend.state.isDesktop}
                      onClick={() => navigate("/settings#agent-settings")}
                      title={`${ch.desc} · 点击前往配置`}
                    >
                      <span>{ch.icon}</span>
                      <span class="font-medium">{ch.name}</span>
                    </button>
                  )
                }}
              </For>
            </div>
          </div>

          <div class="relative overflow-hidden rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] focus-within:border-[color:var(--accent)] transition-all">
            <textarea
              rows={3}
              placeholder="输入你想探索的投研问题…"
              class="min-h-[96px] w-full resize-none bg-transparent px-4 pb-12 pt-3 text-sm leading-relaxed text-[color:var(--text-primary)] outline-none placeholder:text-[color:var(--text-muted)]/70"
              value={input()}
              onInput={(e) => setInput(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
            />
            <div class="absolute bottom-0 left-0 right-0 flex items-center justify-between bg-gradient-to-t from-[color:var(--panel)] to-transparent px-3 py-2">
              <div class="text-[11px] text-[color:var(--text-muted)]/70">
                <Show when={agentSettings.loading} fallback={<span>Shift + Enter 换行</span>}>
                  <span class="animate-pulse">加载配置中…</span>
                </Show>
              </div>
              <button
                type="button"
                onClick={handleSend}
                disabled={!input().trim()}
                class="flex h-8 w-8 items-center justify-center rounded-lg bg-[color:var(--accent)] text-white transition hover:scale-105 disabled:cursor-not-allowed disabled:opacity-30 disabled:hover:scale-100"
                aria-label="发送"
              >
                <svg viewBox="0 0 20 20" fill="currentColor" class="h-4 w-4">
                  <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
                </svg>
              </button>
            </div>
          </div>
        </div>

        {/* 两列:最近会话 + 进行中研究 */}
        <div class="grid gap-4 md:grid-cols-2">
          <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
            <div class="mb-3 flex items-center justify-between">
              <div class="text-sm font-semibold">最近会话</div>
              <button
                type="button"
                class="text-[11px] text-[color:var(--accent)] hover:underline"
                onClick={() => navigate("/sessions")}
              >
                全部 →
              </button>
            </div>
            <Show
              when={recentSessions().length > 0}
              fallback={
                <div class="rounded-md border border-dashed border-[color:var(--border)] p-4 text-center text-[11px] text-[color:var(--text-muted)]">
                  暂无最近会话
                </div>
              }
            >
              <div class="space-y-1.5">
                <For each={recentSessions()}>
                  {(u) => (
                    <button
                      type="button"
                      class="block w-full rounded-md border border-transparent bg-[color:var(--panel)] px-3 py-2 text-left transition hover:border-[color:var(--accent)] hover:bg-[color:var(--accent-soft)]"
                      onClick={() =>
                        navigate(`/sessions/${encodeURIComponent(u.session_id)}`)
                      }
                    >
                      <div class="flex items-center justify-between gap-2">
                        <div class="truncate text-xs font-medium text-[color:var(--text-primary)]">
                          {u.session_label || u.user_id}
                        </div>
                        <div class="shrink-0 text-[10px] text-[color:var(--text-muted)]">
                          {formatTime(u.last_time)}
                        </div>
                      </div>
                      <div class="mt-1 line-clamp-1 text-[11px] text-[color:var(--text-secondary)]">
                        {u.last_message || "(空)"}
                      </div>
                    </button>
                  )}
                </For>
              </div>
            </Show>
          </div>

          <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
            <div class="mb-3 flex items-center justify-between">
              <div class="text-sm font-semibold">进行中研究</div>
              <button
                type="button"
                class="text-[11px] text-[color:var(--accent)] hover:underline"
                onClick={() => navigate("/research")}
              >
                全部 →
              </button>
            </div>
            <Show
              when={activeResearch().length > 0}
              fallback={
                <div class="rounded-md border border-dashed border-[color:var(--border)] p-4 text-center text-[11px] text-[color:var(--text-muted)]">
                  暂无研究任务
                </div>
              }
            >
              <div class="space-y-1.5">
                <For each={activeResearch()}>
                  {(t) => (
                    <button
                      type="button"
                      class="block w-full rounded-md border border-transparent bg-[color:var(--panel)] px-3 py-2 text-left transition hover:border-[color:var(--accent)] hover:bg-[color:var(--accent-soft)]"
                      onClick={() => navigate(`/research/${encodeURIComponent(t.id)}`)}
                    >
                      <div class="flex items-center justify-between gap-2">
                        <div class="truncate text-xs font-medium text-[color:var(--text-primary)]">
                          {t.company_name}
                        </div>
                        <div class="shrink-0 text-[10px] text-[color:var(--text-muted)]">
                          {t.status} · {t.progress}
                        </div>
                      </div>
                      <div class="mt-1 text-[10px] text-[color:var(--text-muted)]">
                        创建于 {formatTime(t.created_at)}
                      </div>
                    </button>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </div>
      </div>
    </div>
  )
}
