import { EmptyState } from "@hone-financial/ui/empty-state"
import { Skeleton } from "@hone-financial/ui/skeleton"
import { For, Show } from "solid-js"
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
  const currentId = () => profiles.state.currentProfileId

  return (
    <div class="flex h-full min-h-0 w-[340px] flex-col border-r border-[color:var(--border)] bg-[color:var(--surface)]">
      <div class="border-b border-[color:var(--border)] px-4 py-3">
        <div>
          <div class="text-sm font-semibold tracking-tight">公司画像</div>
          <div class="text-xs text-[color:var(--text-muted)]">长期基本面与事件时间线，仅展示；建档与更新请通过 agent 完成</div>
        </div>
      </div>

      <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-3 py-3">
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
            fallback={<EmptyState title="暂无公司画像" description="页面只展示画像；如需建立首份画像，请直接让 agent 为公司建档。" />}
          >
            <div class="space-y-2">
              <For each={profiles.profiles() ?? []}>
                {(profile) => (
                  <button
                    type="button"
                    class={[
                      "w-full rounded-md border p-3 text-left transition",
                      currentId() === profile.profile_id
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
      </div>
    </div>
  )
}
