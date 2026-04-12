import { EmptyState } from "@hone-financial/ui/empty-state"
import { Skeleton } from "@hone-financial/ui/skeleton"
import { For, Show } from "solid-js"
import { actorKey, actorLabel, type ActorRef } from "@/lib/actors"
import type { CompanyProfileSpaceSummary } from "@/lib/types"
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

function actorFromSummary(summary: CompanyProfileSpaceSummary): ActorRef {
  return {
    channel: summary.channel,
    user_id: summary.user_id,
    channel_scope: summary.channel_scope,
  }
}

export function CompanyProfileList() {
  const profiles = useCompanyProfiles()
  const currentActorKey = () => profiles.state.currentActorKey
  const currentProfileId = () => profiles.state.currentProfileId

  return (
    <div class="flex h-full min-h-0 w-[360px] flex-col border-r border-[color:var(--border)] bg-[color:var(--surface)]">
      <div class="border-b border-[color:var(--border)] px-4 py-3">
        <div>
          <div class="text-sm font-semibold tracking-tight">公司画像</div>
          <div class="text-xs text-[color:var(--text-muted)]">每个用户 x 渠道都是独立画像空间；页面只读，建档和更新请通过 agent 完成</div>
        </div>
      </div>

      <div class="border-b border-[color:var(--border)] px-3 py-3">
        <div class="mb-2 text-xs font-semibold uppercase tracking-wider text-[color:var(--text-muted)]">
          画像空间
        </div>
        <Show
          when={!profiles.actorsList.loading}
          fallback={
            <div class="space-y-2">
              <Skeleton class="h-14" />
              <Skeleton class="h-14" />
            </div>
          }
        >
          <Show
            when={(profiles.actorsList() ?? []).length > 0}
            fallback={
              <EmptyState
                title="暂无画像空间"
                description="还没有任何用户空间生成公司画像。让 agent 为某家公司建档后，这里会出现对应空间。"
              />
            }
          >
            <div class="max-h-52 space-y-2 overflow-y-auto pr-1">
              <For each={profiles.actorsList() ?? []}>
                {(summary) => {
                  const actor = actorFromSummary(summary)
                  const key = actorKey(actor)
                  const isActive = () => currentActorKey() === key
                  return (
                    <button
                      type="button"
                      class={[
                        "w-full rounded-md border p-3 text-left transition",
                        isActive()
                          ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                          : "border-[color:var(--border)] bg-[color:var(--panel)] hover:border-[color:var(--accent)]/50",
                      ].join(" ")}
                      onClick={() => profiles.selectActor(actor)}
                    >
                      <div class="flex items-start justify-between gap-3">
                        <div class="min-w-0 flex-1">
                          <div class="truncate text-sm font-medium text-[color:var(--text-primary)]">
                            {actorLabel(actor)}
                          </div>
                          <div class="mt-1 text-[11px] text-[color:var(--text-muted)]">
                            {summary.channel}
                          </div>
                          <div class="mt-1 text-[11px] text-[color:var(--text-muted)]">
                            {summary.profile_count} 份画像
                          </div>
                        </div>
                        <div class="text-[10px] text-[color:var(--text-muted)]">
                          {formatDate(summary.updated_at)}
                        </div>
                      </div>
                    </button>
                  )
                }}
              </For>
            </div>
          </Show>
        </Show>
      </div>

      <div class="min-h-0 flex-1 px-3 py-3">
        <div class="mb-2 flex items-center justify-between">
          <div class="text-xs font-semibold uppercase tracking-wider text-[color:var(--text-muted)]">
            当前空间画像
          </div>
          <Show when={profiles.currentActor()}>
            <div class="text-[11px] text-[color:var(--text-muted)]">
              {(profiles.profiles() ?? []).length} 家
            </div>
          </Show>
        </div>

        <Show
          when={profiles.currentActor()}
          fallback={
            <EmptyState
              title="先选择画像空间"
              description="公司画像按 actor 隔离展示。先选左侧空间，再查看这个空间里的公司画像。"
            />
          }
        >
          <Show
            when={!profiles.profiles.loading}
            fallback={
              <div class="space-y-3 px-2 py-2">
                <Skeleton class="h-16" />
                <Skeleton class="h-16" />
              </div>
            }
          >
            <Show
              when={(profiles.profiles() ?? []).length > 0}
              fallback={
                <EmptyState
                  title="当前空间暂无公司画像"
                  description="这个用户空间里还没有画像文档。请通过 agent 建立首份画像。"
                />
              }
            >
              <div class="hf-scrollbar h-full space-y-2 overflow-y-auto">
                <For each={profiles.profiles() ?? []}>
                  {(profile) => (
                    <button
                      type="button"
                      class={[
                        "w-full rounded-md border p-3 text-left transition",
                        currentProfileId() === profile.profile_id
                          ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                          : "border-[color:var(--border)] bg-[color:var(--panel)] hover:border-[color:var(--accent)]/50",
                      ].join(" ")}
                      onClick={() => profiles.selectProfile(profile.profile_id)}
                    >
                      <div class="flex items-start justify-between gap-3">
                        <div class="min-w-0 flex-1">
                          <div class="truncate text-sm font-medium text-[color:var(--text-primary)]">
                            {profile.company_name}
                          </div>
                          <div class="mt-1 flex flex-wrap items-center gap-2 text-[11px] text-[color:var(--text-muted)]">
                            <span>{profile.stock_code || "无代码"}</span>
                            <span>{profile.industry_template}</span>
                            <span>{profile.event_count} 条事件</span>
                          </div>
                        </div>
                        <div class="text-[10px] text-[color:var(--text-muted)]">
                          {formatDate(profile.updated_at)}
                        </div>
                      </div>
                    </button>
                  )}
                </For>
              </div>
            </Show>
          </Show>
        </Show>
      </div>
    </div>
  )
}
