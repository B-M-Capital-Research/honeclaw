import { Badge } from "@hone-financial/ui/badge"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Skeleton } from "@hone-financial/ui/skeleton"
import { useNavigate } from "@solidjs/router"
import { For, Show } from "solid-js"
import { useSkills } from "@/context/skills"

export function SkillList() {
  const navigate = useNavigate()
  const skills = useSkills()
  const statusFilters = [
    { value: "all", label: "全部" },
    { value: "enabled", label: "已启用" },
    { value: "disabled", label: "已禁用" },
  ] as const
  const sourceFilters = [
    { value: "all", label: "全部来源" },
    { value: "system", label: "System" },
    { value: "custom", label: "Custom" },
    { value: "dynamic", label: "Dynamic" },
  ] as const
  const counts = () => skills.counts()

  return (
    <div class="flex h-full min-h-0 w-[320px] flex-col border-r border-[color:var(--border)] bg-[color:var(--surface)]">
      <div class="border-b border-[color:var(--border)] px-5 py-5">
        <div class="text-lg font-semibold">技能管理</div>
        <div class="mt-1 text-sm text-[color:var(--text-muted)]">注册表、启停状态与技能文档</div>
        <div class="mt-3 grid grid-cols-2 gap-2 text-xs text-[color:var(--text-muted)]">
          <div class="rounded-md border border-[color:var(--border)] px-3 py-2">总数 {counts().total}</div>
          <div class="rounded-md border border-[color:var(--border)] px-3 py-2">启用 {counts().enabled}</div>
          <div class="rounded-md border border-[color:var(--border)] px-3 py-2">禁用 {counts().disabled}</div>
          <div class="rounded-md border border-[color:var(--border)] px-3 py-2">Slash {counts().invocable}</div>
        </div>
        <input
          value={skills.state.query}
          onInput={(event) => skills.setQuery(event.currentTarget.value)}
          placeholder="搜索 id、名称、alias"
          class="mt-3 w-full rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] px-3 py-2 text-sm outline-none transition focus:border-[color:var(--accent)]"
        />
        <div class="mt-3 flex flex-wrap gap-2">
          <For each={statusFilters}>
            {(filter) => (
              <button
                type="button"
                onClick={() => skills.setStatusFilter(filter.value)}
                class={[
                  "rounded-full border px-3 py-1 text-xs transition",
                  skills.state.statusFilter === filter.value
                    ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)] text-[color:var(--text-primary)]"
                    : "border-[color:var(--border)] text-[color:var(--text-secondary)] hover:border-[color:var(--border-strong)]",
                ].join(" ")}
              >
                {filter.label}
              </button>
            )}
          </For>
        </div>
        <div class="mt-2 flex flex-wrap gap-2">
          <For each={sourceFilters}>
            {(filter) => (
              <button
                type="button"
                onClick={() => skills.setSourceFilter(filter.value)}
                class={[
                  "rounded-full border px-3 py-1 text-xs transition",
                  skills.state.sourceFilter === filter.value
                    ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)] text-[color:var(--text-primary)]"
                    : "border-[color:var(--border)] text-[color:var(--text-secondary)] hover:border-[color:var(--border-strong)]",
                ].join(" ")}
              >
                {filter.label}
              </button>
            )}
          </For>
        </div>
        <Show when={skills.state.error}>
          <div class="mt-3 rounded-md border border-rose-300/30 bg-rose-500/10 px-3 py-2 text-xs text-rose-300">
            {skills.state.error}
          </div>
        </Show>
      </div>
      <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-3 py-3">
        <Show when={!skills.state.loading} fallback={<div class="space-y-3"><Skeleton class="h-24" /><Skeleton class="h-24" /><Skeleton class="h-24" /></div>}>
          <Show
            when={skills.filteredSkills().length > 0}
            fallback={<EmptyState title="没有找到技能" description="请调整筛选条件，或检查 skills 目录与 SKILL.md。" />}
          >
            <div class="space-y-2">
              <For each={skills.filteredSkills()}>
                {(skill) => {
                  const active = () => skill.id === skills.state.currentSkillId
                  const updating = () => skills.state.updatingSkillId === skill.id
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
                        skill.enabled ? "" : "opacity-70",
                      ].join(" ")}
                    >
                      <div class="flex items-start justify-between gap-3">
                        <div>
                          <div class="font-semibold text-[color:var(--text-primary)]">{skill.display_name}</div>
                          <div class="mt-1 text-xs text-[color:var(--text-muted)]">{skill.id}</div>
                        </div>
                        <label
                          class="inline-flex cursor-pointer items-center gap-2 text-xs text-[color:var(--text-secondary)]"
                          onClick={(event) => event.stopPropagation()}
                        >
                          <input
                            type="checkbox"
                            checked={skill.enabled}
                            disabled={updating()}
                            onChange={(event) => void skills.toggleSkill(skill.id, event.currentTarget.checked)}
                          />
                          {skill.enabled ? "启用" : "禁用"}
                        </label>
                      </div>
                      <div class="mt-2 text-sm leading-6 text-[color:var(--text-secondary)]">{skill.description}</div>
                      <div class="mt-3 flex flex-wrap gap-2">
                        <Badge>{skill.loaded_from}</Badge>
                        <Badge>{skill.context}</Badge>
                        <Show when={skill.enabled} fallback={<Badge>disabled</Badge>}>
                          <Show when={skill.user_invocable}><Badge>slash</Badge></Show>
                        </Show>
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
