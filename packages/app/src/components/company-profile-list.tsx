import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Input } from "@hone-financial/ui/input"
import { Skeleton } from "@hone-financial/ui/skeleton"
import { createMemo, createSignal, For, Show } from "solid-js"
import { useCompanyProfiles } from "@/context/company-profiles"

function formatDate(iso?: string) {
  if (!iso) return "—"
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

export function CompanyProfileList() {
  const profiles = useCompanyProfiles()
  const [query, setQuery] = createSignal("")

  const allTargets = createMemo(() => {
    const items = [...profiles.profileSpaceTargets(), ...profiles.sessionTargets()]
    const manual = profiles.manualTarget()
    if (manual) {
      items.unshift(manual)
    }
    return items
  })

  const filteredTargets = createMemo(() => {
    const keyword = query().trim().toLowerCase()
    if (!keyword) return allTargets()
    return allTargets().filter((target) => {
      const haystack = [
        target.label,
        target.description,
        target.actor.channel,
        target.actor.user_id,
        target.actor.channel_scope ?? "",
      ]
        .join(" ")
        .toLowerCase()
      return haystack.includes(keyword)
    })
  })

  const currentActorKey = () => profiles.state.currentActorKey

  const badgeClass = (kind: "space" | "session" | "manual") => {
    if (kind === "space") {
      return "bg-emerald-100 text-emerald-700"
    }
    if (kind === "session") {
      return "bg-sky-100 text-sky-700"
    }
    return "bg-amber-100 text-amber-700"
  }

  return (
    <div class="flex h-full min-h-0 w-[300px] flex-col border-r border-[color:var(--border)] bg-[color:var(--panel)]">
      <div class="border-b border-[color:var(--border)] px-4 py-3">
        <div>
          <div class="text-sm font-semibold tracking-tight">目标用户空间</div>
          <div class="text-xs text-[color:var(--text-muted)]">
            左侧只负责选人。选中后，右侧再看这个人的公司画像和导入结果。
          </div>
        </div>

        <Input
          class="mt-3 h-9 text-sm"
          value={query()}
          onInput={(event) => setQuery(event.currentTarget.value)}
          placeholder="搜索 channel / user_id / scope"
        />

        <Button
          variant="ghost"
          class="mt-3 h-8 px-3 text-xs"
          onClick={() => profiles.setManualTargetOpen(!profiles.state.manualTargetOpen)}
        >
          {profiles.state.manualTargetOpen ? "收起手动指定" : "手动指定一个新目标"}
        </Button>

        <Show when={profiles.state.manualTargetOpen}>
          <div class="mt-3 space-y-2 rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-3">
            <Input
              class="h-8 text-xs"
              value={profiles.state.manualTargetChannel}
              onInput={(event) =>
                profiles.setManualTargetField("channel", event.currentTarget.value)
              }
              placeholder="channel，例如 discord / telegram / web"
            />
            <Input
              class="h-8 text-xs"
              value={profiles.state.manualTargetUserId}
              onInput={(event) =>
                profiles.setManualTargetField("user_id", event.currentTarget.value)
              }
              placeholder="user_id"
            />
            <Input
              class="h-8 text-xs"
              value={profiles.state.manualTargetScope}
              onInput={(event) =>
                profiles.setManualTargetField("channel_scope", event.currentTarget.value)
              }
              placeholder="channel_scope，可留空"
            />
            <Button
              class="h-8 w-full text-xs"
              onClick={() => void profiles.selectManualTarget()}
            >
              选择这个目标
            </Button>
          </div>
        </Show>
      </div>

      <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-3 py-3">
        <Show
          when={!profiles.actorsList.loading && !profiles.usersList.loading}
          fallback={
            <div class="space-y-3 px-1 py-1">
              <Skeleton class="h-16" />
              <Skeleton class="h-16" />
              <Skeleton class="h-16" />
            </div>
          }
        >
          <Show
            when={allTargets().length > 0}
            fallback={
              <EmptyState
                title="还没有可选目标"
                description="你可以先打开一个用户会话，或直接手动指定要导入到哪个用户空间。"
              />
            }
          >
            <div class="mb-3 flex items-center justify-between px-1">
              <div class="text-xs font-semibold uppercase tracking-wider text-[color:var(--text-muted)]">
                目标用户
              </div>
              <div class="text-[11px] text-[color:var(--text-muted)]">
                {filteredTargets().length} / {allTargets().length}
              </div>
            </div>

            <Show
              when={filteredTargets().length > 0}
              fallback={
                <EmptyState
                  title="没有匹配的目标"
                  description="换个关键词，直接手动指定。"
                />
              }
            >
              <div class="space-y-2">
                <For each={filteredTargets()}>
                  {(target) => {
                    const isSelected = () => currentActorKey() === target.key
                    return (
                      <button
                        type="button"
                        class={[
                          "w-full rounded-md border p-3 text-left transition",
                          isSelected()
                            ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                            : "border-[color:var(--border)] bg-[color:var(--surface)] hover:border-[color:var(--accent)]/50",
                        ].join(" ")}
                        onClick={() => profiles.selectActor(target.actor)}
                      >
                        <div class="flex items-start justify-between gap-3">
                          <div class="min-w-0 flex-1">
                            <div class="flex flex-wrap items-center gap-2">
                              <div class="truncate text-sm font-medium text-[color:var(--text-primary)]">
                                {target.label}
                              </div>
                              <span
                                class={[
                                  "rounded-full px-2 py-0.5 text-[10px] font-medium",
                                  badgeClass(target.source),
                                ].join(" ")}
                              >
                                {target.source === "space"
                                  ? "已有画像"
                                  : target.source === "session"
                                    ? "最近会话"
                                    : "手动指定"}
                              </span>
                            </div>
                            <div class="mt-1 text-[11px] text-[color:var(--text-muted)]">
                              {target.description}
                            </div>
                          </div>
                          <div class="text-[10px] text-[color:var(--text-muted)]">
                            {formatDate(
                              target.source === "session"
                                ? target.sessionLastTime
                                : target.source === "space"
                                  ? target.updatedAt
                                  : undefined,
                            )}
                          </div>
                        </div>
                      </button>
                    )
                  }}
                </For>
              </div>
            </Show>
          </Show>
        </Show>
      </div>
    </div>
  )
}
