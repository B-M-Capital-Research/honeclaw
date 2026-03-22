import { A, useLocation } from "@solidjs/router"
import { Show } from "solid-js"
import { Logo } from "@hone-financial/ui/logo"
import { useConsole } from "@/context/console"
import { useBackend } from "@/context/backend"

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

  return (
    <aside class="flex h-full min-h-0 w-[220px] flex-col border-r border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-5">
      {/* Logo */}
      <div>
        <Logo class="w-28" />
        <div class="mt-3 text-[11px] uppercase tracking-[0.3em] text-[color:var(--text-muted)]">
          Open Financial Console
        </div>
      </div>

      {/* 主导航 */}
      <div class="mt-10 space-y-2">
        <NavLink href="/start" label="开始" />
        <NavLink href="/sessions" label="会话" />
        <Show when={backend.hasCapability("skills")}><NavLink href="/skills" label="技能库" /></Show>
        <Show when={backend.hasCapability("cron_jobs")}><NavLink href="/tasks" label="任务中心" /></Show>
        <NavLink href="/memory" label="记忆" also={["/portfolio"]} />
        <Show when={backend.hasCapability("research")}><NavLink href="/research" label="个股研究" /></Show>
        <NavLink href="/kb" label="知识库" />
      </div>

      {/* 系统分组（紧靠底部状态卡之上） */}
      <div class="mt-auto space-y-2 pb-3">
        <div class="px-4 pb-1 text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">系统</div>
        <Show when={backend.hasCapability("llm_audit")}><NavLink href="/llm-audit" label="LLM 审计" /></Show>
        <Show when={backend.hasCapability("logs")}><NavLink href="/logs" label="日志" /></Show>
        <NavLink href="/settings" label="设置" />
      </div>

      {/* 版本信息 */}
      <div class="pt-1 text-[10px] text-[color:var(--text-muted)]">
        {meta()?.name ?? "Hone"} {meta()?.version ?? ""}
      </div>
    </aside>
  )
}
