import { Button } from "@hone-financial/ui/button";
import { EmptyState } from "@hone-financial/ui/empty-state";
import { Markdown } from "@hone-financial/ui/markdown";
import { For, Show } from "solid-js";
import { actorLabel } from "@/lib/actors";
import { useCompanyProfiles } from "@/context/company-profiles";

function formatDate(iso?: string) {
  if (!iso) return "—";
  try {
    return new Date(iso).toLocaleString("zh-CN", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return iso;
  }
}

export function CompanyProfileDetail() {
  const profiles = useCompanyProfiles();
  const profile = () => profiles.currentProfile();
  const currentActor = () => profiles.currentActor();

  return (
    <Show
      when={profile()}
      fallback={
        <EmptyState
          title={currentActor() ? "从左侧选择公司画像" : "先选择渠道和用户"}
          description={
            currentActor()
              ? "页面只负责展示公司画像；建档、更新和事件追加请通过 agent 完成。"
              : "公司画像按 actor 用户空间隔离展示，请先在左侧选择渠道 + 用户。"
          }
        />
      }
    >
      {(current) => (
        <div class="flex h-full min-h-0 flex-col rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] shadow-sm">
          <div class="flex items-start justify-between gap-4 border-b border-[color:var(--border)] px-6 py-4">
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-2">
                <div class="text-xl font-semibold">{current().title}</div>
              </div>
              <div class="mt-1 flex flex-wrap gap-3 text-sm text-[color:var(--text-muted)]">
                <Show when={currentActor()}>
                  <span>
                    空间：
                    {currentActor()
                      ? `${currentActor()!.channel} / ${actorLabel(currentActor()!)}`
                      : "—"}
                  </span>
                </Show>
                <span>目录：{current().profile_id}</span>
                <span>更新于：{formatDate(current().updated_at)}</span>
              </div>
            </div>
            <Button
              variant="ghost"
              class="h-9 px-3 text-sm text-rose-500 hover:text-rose-600"
              disabled={profiles.state.deleting}
              onClick={async () => {
                if (
                  !confirm(`确定彻底删除 ${current().title} 的公司画像吗？`)
                ) {
                  return;
                }
                await profiles.removeProfile(current().profile_id);
              }}
            >
              {profiles.state.deleting ? "删除中…" : "删除画像"}
            </Button>
          </div>

          <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-6 py-5">
            <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
              <div class="mb-3 flex items-center justify-between gap-3">
                <div class="text-sm font-semibold">画像主文件</div>
                <div class="text-xs text-[color:var(--text-muted)]">
                  直接展示当前目录中的 `profile.md`
                </div>
              </div>
              <Markdown text={current().markdown} />
            </div>

            <div class="mt-5 rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
              <div class="mb-3 flex items-center justify-between">
                <div class="text-sm font-semibold">事件文件</div>
                <div class="text-xs text-[color:var(--text-muted)]">
                  {current().events.length} 条事件
                </div>
              </div>

              <Show
                when={current().events.length > 0}
                fallback={
                  <div class="text-sm text-[color:var(--text-muted)]">
                    当前目录下还没有事件文件。
                  </div>
                }
              >
                <div class="space-y-4">
                  <For each={current().events}>
                    {(event) => (
                      <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
                        <div class="flex flex-wrap items-center gap-3">
                          <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                            {event.title}
                          </div>
                          <div class="text-[11px] text-[color:var(--text-muted)]">
                            {event.filename}
                          </div>
                          <div class="text-[11px] text-[color:var(--text-muted)]">
                            {formatDate(event.updated_at)}
                          </div>
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

            <div class="mt-5 rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4 text-xs text-[color:var(--text-muted)]">
              页面按文件系统直显当前 actor 空间里的公司画像，不要求结构化
              frontmatter。
            </div>
          </div>
        </div>
      )}
    </Show>
  );
}
