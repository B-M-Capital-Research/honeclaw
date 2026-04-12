import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Markdown } from "@hone-financial/ui/markdown"
import { For, Show } from "solid-js"
import { actorLabel } from "@/lib/actors"
import { useCompanyProfiles } from "@/context/company-profiles"

function formatDate(iso?: string) {
  if (!iso) return "—"
  try {
    return new Date(iso).toLocaleString("zh-CN", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    })
  } catch {
    return iso
  }
}

export function CompanyProfileDetail() {
  const profiles = useCompanyProfiles()
  const profile = () => profiles.currentProfile()
  const currentActor = () => profiles.currentActor()

  return (
    <Show
      when={profile()}
      fallback={
        <EmptyState
          title={currentActor() ? "从左侧选择公司画像" : "先选择画像空间"}
          description={currentActor()
            ? "页面只负责展示公司画像；建档、更新和事件追加请通过 agent 完成。"
            : "公司画像按 actor 用户空间隔离展示，请先在左侧选择空间。"}
        />
      }
    >
      {(current) => (
        <div class="flex h-full min-h-0 flex-col rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] shadow-sm">
          <div class="flex items-start justify-between gap-4 border-b border-[color:var(--border)] px-6 py-4">
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-2">
                <div class="text-xl font-semibold">{current().metadata.company_name}</div>
                <Show when={current().metadata.stock_code}>
                  <span class="rounded bg-[color:var(--accent-soft)] px-2 py-0.5 font-mono text-xs font-semibold text-[color:var(--accent)]">
                    {current().metadata.stock_code}
                  </span>
                </Show>
              </div>
              <div class="mt-1 flex flex-wrap gap-3 text-sm text-[color:var(--text-muted)]">
                <span>模板：{current().metadata.industry_template}</span>
                <span>Sector：{current().metadata.sector || "未设置"}</span>
                <Show when={currentActor()}>
                  <span>空间：{currentActor() ? actorLabel(currentActor()!) : "—"}</span>
                </Show>
                <span>更新于：{formatDate(current().metadata.updated_at)}</span>
              </div>
            </div>
            <Button
              variant="ghost"
              class="h-9 px-3 text-sm text-rose-500 hover:text-rose-600"
              disabled={profiles.state.deleting}
              onClick={async () => {
                if (!confirm(`确定彻底删除 ${current().metadata.company_name} 的公司画像吗？`)) {
                  return
                }
                await profiles.removeProfile(current().profile_id)
              }}
            >
              {profiles.state.deleting ? "删除中…" : "删除画像"}
            </Button>
          </div>

          <div class="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_320px] overflow-hidden">
            <div class="hf-scrollbar min-h-0 overflow-y-auto px-6 py-5">
              <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
                <div class="mb-3 text-sm font-semibold">画像主文件</div>
                <Markdown text={current().markdown} />
              </div>

              <div class="mt-5 rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
                <div class="mb-3 flex items-center justify-between">
                  <div class="text-sm font-semibold">事件时间线</div>
                  <div class="text-xs text-[color:var(--text-muted)]">{current().events.length} 条事件</div>
                </div>

                <Show
                  when={current().events.length > 0}
                  fallback={<div class="text-sm text-[color:var(--text-muted)]">当前还没有时间线事件。</div>}
                >
                  <div class="space-y-4">
                    <For each={current().events}>
                      {(event) => (
                        <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
                          <div class="flex flex-wrap items-center gap-2">
                            <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                              {event.title}
                            </div>
                            <span class="rounded bg-[color:var(--accent-soft)] px-2 py-0.5 text-[11px] text-[color:var(--accent)]">
                              {event.metadata.event_type}
                            </span>
                            <span class="text-[11px] text-[color:var(--text-muted)]">
                              {formatDate(event.metadata.occurred_at)}
                            </span>
                          </div>
                          <div class="mt-2 text-[11px] text-[color:var(--text-muted)]">
                            thesis 影响：{event.metadata.thesis_impact}
                          </div>
                          <div class="prose prose-sm mt-3 max-w-none text-[color:var(--text-primary)]">
                            <Markdown text={event.markdown} />
                          </div>
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
              </div>
            </div>

            <div class="hf-scrollbar min-h-0 overflow-y-auto border-l border-[color:var(--border)] bg-[color:var(--panel)] p-5">
              <div class="text-sm font-semibold">追踪状态</div>
              <div class="mt-3 space-y-3">
                <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-3 text-sm">
                  <div class="flex items-center justify-between">
                    <span class="text-[color:var(--text-muted)]">长期追踪</span>
                    <span class={current().metadata.tracking.enabled ? "text-emerald-500" : "text-[color:var(--text-muted)]"}>
                      {current().metadata.tracking.enabled ? "已开启" : "未开启"}
                    </span>
                  </div>
                  <div class="mt-2 flex items-center justify-between">
                    <span class="text-[color:var(--text-muted)]">Cadence</span>
                    <span>{current().metadata.tracking.cadence || "weekly"}</span>
                  </div>
                </div>

                <div class="space-y-1">
                  <div class="text-xs text-[color:var(--text-muted)]">Focus Metrics</div>
                  <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-3 text-sm">
                    <Show
                      when={current().metadata.tracking.focus_metrics.length > 0}
                      fallback={<div class="text-[color:var(--text-muted)]">当前未设置重点跟踪指标，请通过 agent 更新画像。</div>}
                    >
                      <ul class="space-y-1">
                        <For each={current().metadata.tracking.focus_metrics}>
                          {(metric) => <li>{metric}</li>}
                        </For>
                      </ul>
                    </Show>
                  </div>
                </div>

                <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-3 text-xs text-[color:var(--text-muted)]">
                  <div>创建时间：{formatDate(current().metadata.created_at)}</div>
                  <div class="mt-1">最近 review：{formatDate(current().metadata.last_reviewed_at)}</div>
                  <Show when={current().metadata.aliases.length > 0}>
                    <div class="mt-2">
                      别名：{current().metadata.aliases.join(" / ")}
                    </div>
                  </Show>
                </div>

                <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-3 text-xs text-[color:var(--text-muted)]">
                  页面为只读视图。若要建档、修改 section、追加事件或调整追踪参数，请直接让 agent 操作公司画像。
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </Show>
  )
}
