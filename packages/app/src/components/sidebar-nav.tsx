import { A, useLocation } from "@solidjs/router"
import { Show } from "solid-js"
import { Logo } from "@hone-financial/ui/logo"
import { useConsole } from "@/context/console"
import { useBackend } from "@/context/backend"
import { setLocale, useLocale } from "@/lib/i18n"
import { SHARED } from "@/lib/admin-content/shared"

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

function LocaleSwitch() {
  const opts: Array<{ code: "zh" | "en"; key: "zh" | "en" }> = [
    { code: "zh", key: "zh" },
    { code: "en", key: "en" },
  ]
  return (
    <div
      class="flex items-center gap-1 rounded-md border border-[color:var(--border)] p-1"
      role="group"
      aria-label={SHARED.locale.switch_label}
    >
      {opts.map((opt) => {
        const active = () => useLocale() === opt.code
        return (
          <button
            type="button"
            onClick={() => setLocale(opt.code)}
            aria-pressed={active()}
            class={[
              "flex-1 rounded px-2 py-1 text-xs font-medium transition",
              active()
                ? "bg-[color:var(--accent-soft)] text-[color:var(--text-primary)]"
                : "text-[color:var(--text-secondary)] hover:bg-black/5 hover:text-[color:var(--text-primary)]",
            ].join(" ")}
          >
            {SHARED.locale[opt.key]}
          </button>
        )
      })}
    </div>
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
          {SHARED.brand.eyebrow}
        </div>
      </div>

      <div class="mt-10 space-y-2">
        <div class="flex items-center gap-2">
          <div class="min-w-0 flex-1">
            <NavLink href="/dashboard" label={SHARED.nav.dashboard} also={["/start"]} />
          </div>
          <a
            href={publicHref()}
            target="_blank"
            rel="noreferrer"
            class="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-md border border-[color:var(--border)] text-[color:var(--text-secondary)] transition hover:border-[color:var(--accent)]/60 hover:bg-black/5 hover:text-[color:var(--text-primary)]"
            title={SHARED.brand.public_chat_aria}
          >
            <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
              <path d="M11.5 3a.75.75 0 000 1.5h2.94L8.22 10.72a.75.75 0 101.06 1.06L15.5 5.56V8.5a.75.75 0 001.5 0v-5A.75.75 0 0016.25 2h-4.75z" />
              <path d="M4.5 5A2.5 2.5 0 002 7.5v8A2.5 2.5 0 004.5 18h8a2.5 2.5 0 002.5-2.5V11a.75.75 0 00-1.5 0v4.5a1 1 0 01-1 1h-8a1 1 0 01-1-1v-8a1 1 0 011-1H9a.75.75 0 000-1.5H4.5z" />
            </svg>
          </a>
        </div>
      </div>

      <div class="mt-6 space-y-2">
        <div class="px-4 pb-1 text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">{SHARED.nav.section_user_view}</div>
        <NavLink href="/sessions" label={SHARED.nav.sessions} />
        <NavLink href="/users" label={SHARED.nav.users} also={["/memory", "/portfolio"]} />
        <Show when={backend.hasCapability("cron_jobs")}><NavLink href="/tasks" label={SHARED.nav.tasks} /></Show>
        <Show when={backend.hasCapability("cron_jobs")}><NavLink href="/notifications" label={SHARED.nav.notifications} /></Show>
        <Show when={backend.hasCapability("cron_jobs")}><NavLink href="/schedule" label={SHARED.nav.schedule} /></Show>
      </div>

      <Show when={backend.hasCapability("research")}>
        <div class="mt-6 space-y-2">
          <div class="px-4 pb-1 text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">{SHARED.nav.section_research}</div>
          <NavLink href="/research" label={SHARED.nav.research} />
        </div>
      </Show>

      <div class="mt-auto space-y-2 pb-3">
        <div class="px-4 pb-1 text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">{SHARED.nav.section_system}</div>
        <Show when={backend.hasCapability("skills")}><NavLink href="/skills" label={SHARED.nav.skills} /></Show>
        <Show when={backend.hasCapability("llm_audit")}><NavLink href="/llm-audit" label={SHARED.nav.llm_audit} /></Show>
        <Show when={backend.hasCapability("logs")}><NavLink href="/logs" label={SHARED.nav.logs} /></Show>
        <NavLink href="/task-health" label={SHARED.nav.task_health} />
        <NavLink href="/settings" label={SHARED.nav.settings} />
      </div>

      <div class="pt-1 pb-2">
        <LocaleSwitch />
      </div>

      <div class="pt-1 text-[10px] text-[color:var(--text-muted)]">
        {meta()?.name ?? "Hone"} {meta()?.version ?? ""}
      </div>
    </aside>
  )
}
