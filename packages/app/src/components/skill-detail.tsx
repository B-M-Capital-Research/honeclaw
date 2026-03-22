import { Badge } from "@hone-financial/ui/badge"
import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Markdown } from "@hone-financial/ui/markdown"
import { For, Show } from "solid-js"
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

  return (
    <Show
      when={skill()}
      fallback={<EmptyState title="从左侧选择一个技能" description="技能说明会以 Markdown 渲染，并可以直接预填到会话输入框。" />}
    >
      {(current) => (
        <div class="flex h-full min-h-0 flex-col rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm">
          <div class="flex flex-wrap items-start justify-between gap-4">
            <div>
              <div class="text-3xl font-semibold">{current().display_name}</div>
              <div class="mt-3 max-w-3xl text-sm leading-7 text-[color:var(--text-secondary)]">{current().description}</div>
              <div class="mt-4 flex flex-wrap gap-2">
                <For each={current().tools}>{(tool) => <Badge tone="accent">{tool}</Badge>}</For>
              </div>
            </div>
            <Button
              onClick={() => {
                sessions.prefillDraft(`load_skill("${current().id}")`)
                const target = consoleState.state.lastUserId
                navigate(target ? `/sessions/${encodeURIComponent(target)}` : "/sessions")
              }}
            >
              在对话中加载
            </Button>
          </div>

          <div class="hf-scrollbar mt-8 min-h-0 flex-1 overflow-y-auto rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-6">
            <Markdown text={current().guide} />
          </div>
        </div>
      )}
    </Show>
  )
}
