import { A, useLocation } from "@solidjs/router"
import { Show } from "solid-js"
import { Logo } from "@hone-financial/ui/logo"
import { useConsole } from "@/context/console"
import { useBackend } from "@/context/backend"

function publicChatUrl() {
  if (typeof window === "undefined") return "/chat"
  try {
    const url = new URL(window.location.href)
    if (url.port === "8077") {
      url.port = "8088"
    }
    url.pathname = "/chat"
    url.search = ""
    url.hash = ""
    return url.toString()
  } catch {
    return "/chat"
  }
}

function NavLink(props: { href: string; label: string; also?: string[] }) {
  const location = useLocation()
  const active = () =>
    location.pathname.startsWith(props.href) ||
    (props.also ?? []).some((p) => location.pathname.startsWith(p))
  return (
    <A
      href={props.href}
      class={[
        "flex items-center rounded-md px-4 py-3 text-sm font-medium transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]",
        active()
          ? "bg-[color:var(--accent-soft)] text-[color:var(--text-primary)]"
          : "text-[color:var(--text-secondary)] hover:bg-black/5 hover:text-[color:var(--text-primary)]",
      ].join(" ")}
    >
      {props.label}
    </A>
  )
}

export function SidebarNav() {
  const consoleState = useConsole()
  const backend = useBackend()
  const meta = () => consoleState.meta()
  const publicHref = () => publicChatUrl()

  return (
    <aside class="flex h-full min-h-0 w-[220px] flex-col border-r border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-5">
      {/* Logo */}
      <div>
        <Logo class="w-28" />
        <div class="mt-3 text-[11px] uppercase tracking-[0.3em] text-[color:var(--text-muted)]">
          Open Financial Console
        </div>
      </div>

      {/* 工作台 */}
      <div class="mt-10 space-y-2">
        <div class="flex items-center gap-2">
          <div class="min-w-0 flex-1">
            <NavLink href="/dashboard" label="概览" also={["/start"]} />
          </div>
          <a
            href={publicHref()}
            target="_blank"
            rel="noreferrer"
            class="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-md border border-[color:var(--border)] text-[color:var(--text-secondary)] transition hover:border-[color:var(--accent)]/60 hover:bg-black/5 hover:text-[color:var(--text-primary)]"
            title="打开用户端（端口 8088）"
          >
            <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
              <path d="M11.5 3a.75.75 0 000 1.5h2.94L8.22 10.72a.75.75 0 101.06 1.06L15.5 5.56V8.5a.75.75 0 001.5 0v-5A.75.75 0 0016.25 2h-4.75z" />
              <path d="M4.5 5A2.5 2.5 0 002 7.5v8A2.5 2.5 0 004.5 18h8a2.5 2.5 0 002.5-2.5V11a.75.75 0 00-1.5 0v4.5a1 1 0 01-1 1h-8a1 1 0 01-1-1v-8a1 1 0 011-1H9a.75.75 0 000-1.5H4.5z" />
            </svg>
          </a>
        </div>
      </div>

      {/* 用户视角 */}
      <div class="mt-6 space-y-2">
        <div class="px-4 pb-1 text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">用户视角</div>
        <NavLink href="/sessions" label="会话" />
        <NavLink href="/users" label="用户档案" also={["/memory", "/portfolio"]} />
        <Show when={backend.hasCapability("cron_jobs")}><NavLink href="/tasks" label="推送任务" /></Show>
        <Show when={backend.hasCapability("cron_jobs")}><NavLink href="/notifications" label="推送日志" /></Show>
        <Show when={backend.hasCapability("cron_jobs")}><NavLink href="/schedule" label="推送日程" /></Show>
      </div>

      {/* 研究 */}
      <Show when={backend.hasCapability("research")}>
        <div class="mt-6 space-y-2">
          <div class="px-4 pb-1 text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">研究</div>
          <NavLink href="/research" label="个股研究" />
        </div>
      </Show>

      {/* 系统分组(紧靠底部状态卡之上) */}
      <div class="mt-auto space-y-2 pb-3">
        <div class="px-4 pb-1 text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">系统</div>
        <Show when={backend.hasCapability("skills")}><NavLink href="/skills" label="技能管理" /></Show>
        <Show when={backend.hasCapability("llm_audit")}><NavLink href="/llm-audit" label="LLM 审计" /></Show>
        <Show when={backend.hasCapability("logs")}><NavLink href="/logs" label="日志" /></Show>
        <NavLink href="/task-health" label="任务健康" />
        <NavLink href="/settings" label="设置" />
      </div>

      {/* 版本信息 */}
      <div class="pt-1 text-[10px] text-[color:var(--text-muted)]">
        {meta()?.name ?? "Hone"} {meta()?.version ?? ""}
      </div>
    </aside>
  )
}
