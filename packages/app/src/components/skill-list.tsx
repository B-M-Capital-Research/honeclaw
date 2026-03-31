import { Badge } from "@hone-financial/ui/badge"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Skeleton } from "@hone-financial/ui/skeleton"
import { useNavigate } from "@solidjs/router"
import { For, Show } from "solid-js"
import { useSkills } from "@/context/skills"

export function SkillList() {
  const navigate = useNavigate()
  const skills = useSkills()

  return (
    <div class="flex h-full min-h-0 w-[320px] flex-col border-r border-[color:var(--border)] bg-[color:var(--surface)]">
      <div class="border-b border-[color:var(--border)] px-5 py-5">
        <div class="text-lg font-semibold">技能库</div>
        <div class="mt-1 text-sm text-[color:var(--text-muted)]">开源 skill 定义与说明文档</div>
      </div>
      <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-3 py-3">
        <Show when={!skills.state.loading} fallback={<div class="space-y-3"><Skeleton class="h-24" /><Skeleton class="h-24" /><Skeleton class="h-24" /></div>}>
          <Show
            when={skills.state.skills.length > 0}
            fallback={<EmptyState title="没有找到技能" description="请检查 skills 目录和 SKILL.md 文件。" />}
          >
            <div class="space-y-2">
              <For each={skills.state.skills}>
                {(skill) => {
                  const active = () => skill.id === skills.state.currentSkillId
                  return (
                    <button
                      type="button"
                      onClick={() => {
                        skills.selectSkill(skill.id)
                        navigate(`/skills/${encodeURIComponent(skill.id)}`)
                      }}
                      class={[
                        "w-full rounded-md border p-4 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]",
                        active()
                          ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                          : "border-transparent bg-black/5 hover:border-[color:var(--border-strong)] hover:bg-black/10",
                      ].join(" ")}
                    >
                      <div class="font-semibold text-[color:var(--text-primary)]">{skill.display_name}</div>
                      <div class="mt-2 text-sm leading-6 text-[color:var(--text-secondary)]">{skill.description}</div>
                      <div class="mt-3 flex flex-wrap gap-2">
                        <For each={skill.allowed_tools}>{(tool) => <Badge>{tool}</Badge>}</For>
                      </div>
                    </button>
                  )
                }}
              </For>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  )
}
