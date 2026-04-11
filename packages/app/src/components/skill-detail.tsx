import { Badge } from "@hone-financial/ui/badge"
import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Markdown } from "@hone-financial/ui/markdown"
import { For, Show, createEffect } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { useConsole } from "@/context/console"
import { useSessions } from "@/context/sessions"
import { useSkills } from "@/context/skills"

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
            <div class="text-2xl font-semibold">技能管理</div>
            <div class="mt-2 max-w-3xl text-sm leading-7 text-[color:var(--text-secondary)]">
              查看当前注册的 skills，并控制它们是否可被 Hone 在所有渠道与 runners 中调用。
            </div>
          </div>
          <Button onClick={() => void skills.resetRegistry()} disabled={skills.state.resetting}>
            {skills.state.resetting ? "恢复中..." : "恢复默认"}
          </Button>
        </div>
        <div class="mt-4 grid gap-3 md:grid-cols-4">
          <div class="rounded-md border border-[color:var(--border)] px-4 py-3 text-sm">总数 {counts().total}</div>
          <div class="rounded-md border border-[color:var(--border)] px-4 py-3 text-sm">启用 {counts().enabled}</div>
          <div class="rounded-md border border-[color:var(--border)] px-4 py-3 text-sm">禁用 {counts().disabled}</div>
          <div class="rounded-md border border-[color:var(--border)] px-4 py-3 text-sm">Slash {counts().invocable}</div>
        </div>
      </div>

      <Show
        when={skill()}
        fallback={
          <div class="flex min-h-0 flex-1 rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm">
            <EmptyState title="从左侧选择一个技能" description="这里会展示技能状态、运行时元信息和 Markdown 文档。" />
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
                  <Show when={current().summary.user_invocable}><Badge>slash</Badge></Show>
                  <Show when={current().summary.has_script}><Badge>script</Badge></Show>
                  <Show when={current().summary.has_path_gate}><Badge>path-gated</Badge></Show>
                  <For each={current().summary.allowed_tools}>{(tool) => <Badge tone="accent">{tool}</Badge>}</For>
                </div>
              </div>
              <div class="flex min-w-[220px] flex-col gap-3 rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
                <label class="flex items-center justify-between gap-4 text-sm">
                  <span>启用此技能</span>
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
                  在对话中触发
                </Button>
              </div>
            </div>

            <div class="mt-6 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
              <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-3 text-sm">
                来源：{current().summary.loaded_from}
              </div>
              <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-3 text-sm">
                执行上下文：{current().summary.context}
              </div>
              <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-3 text-sm">
                Slash：{current().summary.user_invocable ? "允许" : "不允许"}
              </div>
              <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-3 text-sm">
                文件：{current().detail_path}
              </div>
            </div>

            <Show when={current().summary.aliases.length > 0 || current().summary.paths.length > 0}>
              <div class="mt-4 grid gap-3 md:grid-cols-2">
                <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-3 text-sm">
                  <div class="text-xs uppercase tracking-wide text-[color:var(--text-muted)]">Aliases</div>
                  <div class="mt-2 break-words text-[color:var(--text-secondary)]">
                    {current().summary.aliases.length > 0 ? current().summary.aliases.join(", ") : "无"}
                  </div>
                </div>
                <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-4 py-3 text-sm">
                  <div class="text-xs uppercase tracking-wide text-[color:var(--text-muted)]">Path Gate</div>
                  <div class="mt-2 break-words text-[color:var(--text-secondary)]">
                    {current().summary.paths.length > 0 ? current().summary.paths.join(", ") : "无"}
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
