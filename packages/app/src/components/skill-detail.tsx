import { Badge } from "@hone-financial/ui/badge"
import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Markdown } from "@hone-financial/ui/markdown"
import { For, Show, createEffect } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { useConsole } from "@/context/console"
import { useSessions } from "@/context/sessions"
import { useSkills } from "@/context/skills"
import { SKILLS } from "@/lib/admin-content/skills"
import { tpl } from "@/lib/i18n"

export function SkillDetail() {
  const navigate = useNavigate()
  const consoleState = useConsole()
  const sessions = useSessions()
  const skills = useSkills()
  const skill = () => skills.currentSkill()
  const counts = () => skills.counts()

  createEffect(() => {
    void skills.ensureSkillDetail(skills.state.currentSkillId)
  })

  return (
    <div class="flex h-full min-h-0 flex-col gap-4">
      <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm">
        <div class="flex flex-wrap items-start justify-between gap-4">
          <div>
            <div class="text-2xl font-semibold">{SKILLS.detail.header_title}</div>
            <div class="mt-2 max-w-3xl text-sm leading-7 text-[color:var(--text-secondary)]">
              {SKILLS.detail.header_subtitle}
            </div>
          </div>
          <Button onClick={() => void skills.resetRegistry()} disabled={skills.state.resetting}>
            {skills.state.resetting ? SKILLS.detail.resetting_button : SKILLS.detail.reset_button}
          </Button>
        </div>
        <div class="mt-4 grid gap-3 md:grid-cols-4">
          <div class="rounded-md border border-[color:var(--border)] px-4 py-3 text-sm">{tpl(SKILLS.detail.counts_total, { count: counts().total })}</div>
          <div class="rounded-md border border-[color:var(--border)] px-4 py-3 text-sm">{tpl(SKILLS.detail.counts_enabled, { count: counts().enabled })}</div>
          <div class="rounded-md border border-[color:var(--border)] px-4 py-3 text-sm">{tpl(SKILLS.detail.counts_disabled, { count: counts().disabled })}</div>
          <div class="rounded-md border border-[color:var(--border)] px-4 py-3 text-sm">{tpl(SKILLS.detail.counts_invocable, { count: counts().invocable })}</div>
        </div>
      </div>

      <Show
        when={skill()}
        fallback={
          <div class="flex min-h-0 flex-1 rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm">
            <EmptyState title={SKILLS.detail.empty_title} description={SKILLS.detail.empty_description} />
          </div>
        }
      >
        {(current) => (
          <div class="flex min-h-0 flex-1 flex-col rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm">
            <div class="flex flex-wrap items-start justify-between gap-4">
              <div>
                <div class="text-3xl font-semibold">{current().summary.display_name}</div>
                <div class="mt-2 text-sm text-[color:var(--text-muted)]">{current().summary.id}</div>
                <div class="mt-3 max-w-3xl text-sm leading-7 text-[color:var(--text-secondary)]">{current().summary.description}</div>
                <Show when={current().summary.when_to_use}>
                  <div class="mt-2 max-w-3xl text-sm leading-7 text-[color:var(--text-muted)]">
                    {current().summary.when_to_use}
                  </div>
                </Show>
                <Show when={current().summary.disabled_reason}>
                  <div class="mt-3 rounded-md border border-amber-300/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-200">
                    {current().summary.disabled_reason}
                  </div>
                </Show>
                <div class="mt-4 flex flex-wrap gap-2">
                  <Badge>{current().summary.loaded_from}</Badge>
                  <Badge>{current().summary.context}</Badge>
                  <Show when={current().summary.user_invocable}><Badge>{SKILLS.detail.badge_slash}</Badge></Show>
                  <Show when={current().summary.has_script}><Badge>{SKILLS.detail.badge_script}</Badge></Show>
                  <Show when={current().summary.has_path_gate}><Badge>{SKILLS.detail.badge_path_gated}</Badge></Show>
                  <For each={current().summary.allowed_tools}>{(tool) => <Badge tone="accent">{tool}</Badge>}</For>
                </div>
              </div>
              <div class="flex min-w-[220px] flex-col gap-3 rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
                <label class="flex items-center justify-between gap-4 text-sm">
                  <span>{SKILLS.detail.enable_label}</span>
                  <input
                    type="checkbox"
                    checked={current().summary.enabled}
                    disabled={skills.state.updatingSkillId === current().summary.id}
                    onChange={(event) => void skills.toggleSkill(current().summary.id, event.currentTarget.checked)}
                  />
                </label>
                <Button
                  disabled={!current().summary.enabled}
                  onClick={() => {
                    sessions.prefillDraft(`/${current().summary.id}`)
                    const target = consoleState.state.lastUserId
                    navigate(target ? `/sessions/${encodeURIComponent(target)}` : "/sessions")
                  }}
                >
                  {SKILLS.detail.invoke_button}
                </Button>
              </div>
            </div>

            <div class="mt-6 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
              <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-3 text-sm">
                {tpl(SKILLS.detail.meta_loaded_from, { value: current().summary.loaded_from })}
              </div>
              <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-3 text-sm">
                {tpl(SKILLS.detail.meta_context, { value: current().summary.context })}
              </div>
              <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-3 text-sm">
                {tpl(SKILLS.detail.meta_invocable, { value: current().summary.user_invocable ? SKILLS.detail.invocable_yes : SKILLS.detail.invocable_no })}
              </div>
              <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-3 text-sm">
                {tpl(SKILLS.detail.meta_file, { value: current().detail_path })}
              </div>
            </div>

            <Show when={current().summary.aliases.length > 0 || current().summary.paths.length > 0}>
              <div class="mt-4 grid gap-3 md:grid-cols-2">
                <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-3 text-sm">
                  <div class="text-xs uppercase tracking-wide text-[color:var(--text-muted)]">{SKILLS.detail.aliases_label}</div>
                  <div class="mt-2 break-words text-[color:var(--text-secondary)]">
                    {current().summary.aliases.length > 0 ? current().summary.aliases.join(", ") : SKILLS.detail.none}
                  </div>
                </div>
                <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-3 text-sm">
                  <div class="text-xs uppercase tracking-wide text-[color:var(--text-muted)]">{SKILLS.detail.paths_label}</div>
                  <div class="mt-2 break-words text-[color:var(--text-secondary)]">
                    {current().summary.paths.length > 0 ? current().summary.paths.join(", ") : SKILLS.detail.none}
                  </div>
                </div>
              </div>
            </Show>

            <div class="hf-scrollbar mt-6 min-h-0 flex-1 overflow-y-auto rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-6">
              <Markdown text={current().markdown} />
            </div>
          </div>
        )}
      </Show>
    </div>
  )
}
