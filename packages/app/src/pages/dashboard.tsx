import { useNavigate } from "@solidjs/router"
import { For, Show, createMemo, createResource, createSignal } from "solid-js"
import { loadDesktopAgentSettings } from "@/lib/backend"
import { useBackend } from "@/context/backend"
import { useConsole } from "@/context/console"
import { useResearch } from "@/context/research"
import { useSessions, ME_SESSION_ID } from "@/context/sessions"
import { tpl, useLocale } from "@/lib/i18n"
import { DASH } from "@/lib/admin-content/dashboard"
import type { AgentProvider } from "@/lib/types"

type ChannelDef = {
  runner: AgentProvider
  name: string
  desc: () => string
  icon: string
}

const CHANNELS: ChannelDef[] = [
  { runner: "multi-agent", name: "Multi-Agent", desc: () => DASH.channels.multi_agent_desc, icon: "∞" },
  { runner: "codex_acp", name: "Codex ACP", desc: () => DASH.channels.codex_acp_desc, icon: "⌘" },
  { runner: "opencode_acp", name: "", desc: () => DASH.channels.opencode_acp_desc, icon: "⚡" },
  { runner: "gemini_cli", name: "Gemini CLI", desc: () => DASH.channels.gemini_cli_desc, icon: "✦" },
  { runner: "codex_cli", name: "Codex CLI", desc: () => DASH.channels.codex_cli_desc, icon: "◈" },
]

function formatTime(iso?: string): string {
  if (!iso) return ""
  try {
    const localeTag = useLocale() === "zh" ? "zh-CN" : "en-US"
    return new Date(iso).toLocaleString(localeTag, {
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

  const channelLabel = (ch: ChannelDef) => ch.name || DASH.channels.opencode_acp_name

  return (
    <div class="hf-scrollbar h-full overflow-y-auto px-6 py-6">
      <div class="mx-auto flex w-full max-w-5xl flex-col gap-6">
        <div class="grid gap-3 md:grid-cols-3">
          <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
            <div class="text-[11px] uppercase tracking-widest text-[color:var(--text-muted)]">
              {DASH.status_panel.backend_label}
            </div>
            <div class="mt-1.5 flex items-center gap-2">
              <span
                class={[
                  "h-2 w-2 rounded-full",
                  backend.state.connected ? "bg-emerald-400" : "bg-rose-400",
                ].join(" ")}
              />
              <span class="text-sm font-medium text-[color:var(--text-primary)]">
                {backend.state.connected
                  ? DASH.status_panel.backend_connected
                  : DASH.status_panel.backend_disconnected}
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
              {DASH.status_panel.channels_label}
            </div>
            <div class="mt-1.5 text-sm font-medium text-[color:var(--text-primary)]">
              {tpl(DASH.status_panel.channels_summary, {
                live: channelStatus().live,
                total: channelStatus().total,
              })}
            </div>
            <Show when={consoleState.channelError()}>
              <div class="mt-1 line-clamp-2 text-[11px] text-rose-400">
                {consoleState.channelError()}
              </div>
            </Show>
          </div>

          <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
            <div class="text-[11px] uppercase tracking-widest text-[color:var(--text-muted)]">
              {DASH.status_panel.active_research_label}
            </div>
            <div class="mt-1.5 text-sm font-medium text-[color:var(--text-primary)]">
              {tpl(DASH.status_panel.active_research_summary, {
                count: research.state.tasks.filter((t) => t.status === "running" || t.status === "pending").length,
              })}
            </div>
            <button
              type="button"
              class="mt-1 text-[11px] text-[color:var(--accent)] hover:underline"
              onClick={() => navigate("/research")}
            >
              {DASH.status_panel.research_open_link}
            </button>
          </div>
        </div>

        <div class="rounded-xl border border-[color:var(--border)] bg-[color:var(--surface)] p-5 shadow-sm">
          <div class="mb-3 flex items-center justify-between">
            <div>
              <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                {DASH.quick_chat.title}
              </div>
              <div class="text-[11px] text-[color:var(--text-muted)]">
                {DASH.quick_chat.subtitle}
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
                      title={tpl(DASH.quick_chat.runner_chip_title, { desc: ch.desc() })}
                    >
                      <span>{ch.icon}</span>
                      <span class="font-medium">{channelLabel(ch)}</span>
                    </button>
                  )
                }}
              </For>
            </div>
          </div>

          <div class="relative overflow-hidden rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] focus-within:border-[color:var(--accent)] transition-all">
            <textarea
              rows={3}
              placeholder={DASH.quick_chat.placeholder}
              class="min-h-[96px] w-full resize-none bg-transparent px-4 pb-12 pt-3 text-sm leading-relaxed text-[color:var(--text-primary)] outline-none placeholder:text-[color:var(--text-muted)]/70"
              value={input()}
              onInput={(e) => setInput(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
            />
            <div class="absolute bottom-0 left-0 right-0 flex items-center justify-between bg-gradient-to-t from-[color:var(--panel)] to-transparent px-3 py-2">
              <div class="text-[11px] text-[color:var(--text-muted)]/70">
                <Show when={agentSettings.loading} fallback={<span>{DASH.quick_chat.shift_enter_hint}</span>}>
                  <span class="animate-pulse">{DASH.quick_chat.loading_settings}</span>
                </Show>
              </div>
              <button
                type="button"
                onClick={handleSend}
                disabled={!input().trim()}
                class="flex h-8 w-8 items-center justify-center rounded-lg bg-[color:var(--accent)] text-white transition hover:scale-105 disabled:cursor-not-allowed disabled:opacity-30 disabled:hover:scale-100"
                aria-label={DASH.quick_chat.send_aria}
              >
                <svg viewBox="0 0 20 20" fill="currentColor" class="h-4 w-4">
                  <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
                </svg>
              </button>
            </div>
          </div>
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
            <div class="mb-3 flex items-center justify-between">
              <div class="text-sm font-semibold">{DASH.recent.sessions_title}</div>
              <button
                type="button"
                class="text-[11px] text-[color:var(--accent)] hover:underline"
                onClick={() => navigate("/sessions")}
              >
                {DASH.recent.sessions_view_all}
              </button>
            </div>
            <Show
              when={recentSessions().length > 0}
              fallback={
                <div class="rounded-md border border-dashed border-[color:var(--border)] p-4 text-center text-[11px] text-[color:var(--text-muted)]">
                  {DASH.recent.sessions_empty}
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
                        {u.last_message || DASH.recent.last_message_empty}
                      </div>
                    </button>
                  )}
                </For>
              </div>
            </Show>
          </div>

          <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
            <div class="mb-3 flex items-center justify-between">
              <div class="text-sm font-semibold">{DASH.recent.research_title}</div>
              <button
                type="button"
                class="text-[11px] text-[color:var(--accent)] hover:underline"
                onClick={() => navigate("/research")}
              >
                {DASH.recent.research_view_all}
              </button>
            </div>
            <Show
              when={activeResearch().length > 0}
              fallback={
                <div class="rounded-md border border-dashed border-[color:var(--border)] p-4 text-center text-[11px] text-[color:var(--text-muted)]">
                  {DASH.recent.research_empty}
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
                        {tpl(DASH.recent.created_prefix, { time: formatTime(t.created_at) })}
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
