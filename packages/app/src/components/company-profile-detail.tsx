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

function SummaryBlock(props: {
  title: string
  updatedAt: string
  eventCount: number
  thesisExcerpt: string
}) {
  return (
    <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
      <div class="text-sm font-semibold text-[color:var(--text-primary)]">
        {props.title}
      </div>
      <div class="mt-1 flex flex-wrap gap-3 text-[11px] text-[color:var(--text-muted)]">
        <span>更新于：{formatDate(props.updatedAt)}</span>
        <span>{props.eventCount} 条事件</span>
      </div>
      <div class="mt-3 text-sm leading-6 text-[color:var(--text-secondary)]">
        {props.thesisExcerpt || "未提取到 Thesis 摘要"}
      </div>
    </div>
  )
}

export function CompanyProfileDetail() {
  const profiles = useCompanyProfiles()
  const profile = () => profiles.currentProfile()
  const currentActor = () => profiles.currentActor()
  const preview = () => profiles.state.transfer.preview
  const result = () => profiles.state.transfer.lastResult
  const transferError = () => profiles.state.transfer.error
  let fileInputRef: HTMLInputElement | undefined

  const openFilePicker = () => fileInputRef?.click()
  const onChooseFile = async (file?: File | null) => {
    if (!file) return
    await profiles.previewImport(file)
  }

  return (
    <Show
      when={currentActor()}
      fallback={
        <EmptyState
          title="先选择目标用户"
          description="左侧先选一个用户空间，右侧才能查看、导出或导入公司画像。"
        />
      }
    >
      {(actor) => (
        <div class="flex h-full min-h-0 flex-col bg-[color:var(--surface)]">
          <input
            ref={fileInputRef}
            type="file"
            accept=".zip,application/zip"
            class="hidden"
            onChange={(event) => {
              const file = event.currentTarget.files?.[0]
              event.currentTarget.value = ""
              void onChooseFile(file).catch(() => undefined)
            }}
          />

          <div class="flex items-start justify-between gap-4 border-b border-[color:var(--border)] px-6 py-4">
            <div class="min-w-0">
              <div class="text-xl font-semibold">
                {actor().channel} / {actorLabel(actor())}
              </div>
              <div class="mt-1 flex flex-wrap gap-3 text-sm text-[color:var(--text-muted)]">
                <span>画像空间</span>
                <span>{(profiles.profiles() ?? []).length} 家公司</span>
                <Show when={profile()}>
                  <span>当前查看：{profile()!.title}</span>
                </Show>
              </div>
            </div>

            <div class="flex shrink-0 items-center gap-2">
              <Button
                variant="ghost"
                class="h-9 px-3 text-sm"
                disabled={profiles.state.exporting}
                onClick={() => void profiles.exportCurrentSpace().catch(() => undefined)}
              >
                {profiles.state.exporting ? "导出中…" : "导出当前空间"}
              </Button>
              <Button class="h-9 px-3 text-sm" onClick={openFilePicker}>
                导入画像包
              </Button>
              <Show when={profile()}>
                <Button
                  variant="ghost"
                  class="h-9 px-3 text-sm text-rose-500 hover:text-rose-600"
                  disabled={profiles.state.deleting}
                  onClick={async () => {
                    if (!confirm(`确定彻底删除 ${profile()!.title} 的公司画像吗？`)) {
                      return
                    }
                    await profiles.removeProfile(profile()!.profile_id)
                  }}
                >
                  {profiles.state.deleting ? "删除中…" : "删除画像"}
                </Button>
              </Show>
            </div>
          </div>

          <div class="flex min-h-0 flex-1 overflow-hidden">
            {/* 左侧：垂直列出当前空间的画像 */}
            <Show when={(profiles.profiles() ?? []).length > 0}>
              <div class="flex w-[240px] shrink-0 flex-col border-r border-[color:var(--border)] bg-[color:var(--panel)]">
                <div class="shrink-0 border-b border-[color:var(--border)] px-4 py-3">
                  <div class="flex items-center justify-between">
                    <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                      当前空间画像
                    </div>
                    <div class="text-xs text-[color:var(--text-muted)]">
                      {(profiles.profiles() ?? []).length} 家
                    </div>
                  </div>
                </div>

                <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto p-2">
                  <div class="flex flex-col gap-1">
                    <For each={profiles.profiles() ?? []}>
                      {(item) => (
                        <button
                          type="button"
                          class={[
                            "w-full rounded-md border px-3 py-2 text-left transition",
                            profiles.state.currentProfileId === item.profile_id
                              ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                              : "border-transparent bg-transparent hover:bg-[color:var(--surface)] hover:border-[color:var(--border)]",
                          ].join(" ")}
                          onClick={() => profiles.selectProfile(item.profile_id)}
                        >
                          <div class="truncate text-sm font-medium text-[color:var(--text-primary)]">
                            {item.title}
                          </div>
                          <div class="mt-1 flex items-center gap-2 text-[11px] text-[color:var(--text-muted)]">
                            <span class="flex-1 truncate">{formatDate(item.updated_at)}</span>
                            <Show
                              when={profiles.state.highlightedProfileIds.includes(
                                item.profile_id,
                              )}
                            >
                              <span class="shrink-0 rounded-full bg-emerald-100 px-1.5 py-0.5 font-medium text-[10px] text-emerald-700">
                                更新
                              </span>
                            </Show>
                          </div>
                        </button>
                      )}
                    </For>
                  </div>
                </div>
              </div>
            </Show>

            {/* 右侧：主视图（预览、文档等） */}
            <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-6 py-5 bg-[color:var(--surface)]">

            <Show when={transferError()}>
              <div class="mb-4 rounded-lg border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
                {transferError()}
              </div>
            </Show>

            <Show when={result()}>
              <div class="mb-4 rounded-lg border border-emerald-200 bg-emerald-50 p-4">
                <div class="text-sm font-semibold text-emerald-800">导入完成</div>
                <div class="mt-1 text-sm text-emerald-700">
                  新增 {result()!.imported_count} 家，替换 {result()!.replaced_count} 家，跳过{" "}
                  {result()!.skipped_count} 家。
                </div>
                <Show when={profiles.state.transfer.backupBlob}>
                  <div class="mt-3">
                    <Button
                      variant="ghost"
                      class="h-8 px-3 text-xs"
                      onClick={() => profiles.downloadBackup()}
                    >
                      下载导入前备份
                    </Button>
                  </div>
                </Show>
              </div>
            </Show>

            <Show when={preview()}>
              {(currentPreview) => (
                <div class="space-y-5">
                  <div
                    class="rounded-xl border border-dashed border-[color:var(--accent)]/40 bg-[color:var(--accent-soft)]/40 p-5 text-center"
                    onDragOver={(event) => event.preventDefault()}
                    onDrop={(event) => {
                      event.preventDefault()
                      const file = event.dataTransfer?.files?.[0]
                      void onChooseFile(file).catch(() => undefined)
                    }}
                  >
                    <div class="text-base font-semibold text-[color:var(--text-primary)]">
                      画像包已载入
                    </div>
                    <div class="mt-1 text-sm text-[color:var(--text-secondary)]">
                      共 {currentPreview().profiles.length} 家公司，发现{" "}
                      {currentPreview().conflict_count} 家冲突公司。
                    </div>
                    <div class="mt-3 flex items-center justify-center gap-2">
                      <Button
                        variant="ghost"
                        class="h-8 px-3 text-xs"
                        onClick={openFilePicker}
                      >
                        重新选择画像包
                      </Button>
                      <Button
                        variant="ghost"
                        class="h-8 px-3 text-xs"
                        onClick={() => profiles.resetTransfer()}
                      >
                        取消本次导入
                      </Button>
                    </div>
                  </div>

                  <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
                    <div class="text-sm font-semibold">导入扫描结果</div>
                    <div class="mt-2 grid gap-3 text-sm text-[color:var(--text-secondary)] md:grid-cols-3">
                      <div>画像总数：{currentPreview().profiles.length}</div>
                      <div>可直接导入：{currentPreview().importable_count}</div>
                      <div>需要确认：{currentPreview().conflict_count}</div>
                    </div>
                  </div>

                  <Show
                    when={currentPreview().conflict_count > 0}
                    fallback={
                      <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
                        <div class="text-base font-semibold text-[color:var(--text-primary)]">
                          当前空间没有冲突，可以直接导入。
                        </div>
                        <div class="mt-2 text-sm text-[color:var(--text-secondary)]">
                          这次会导入 {currentPreview().profiles.length} 家公司画像。
                        </div>
                      </div>
                    }
                  >
                    <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
                      <div class="flex flex-wrap items-center justify-between gap-3">
                        <div>
                          <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                            冲突审阅
                          </div>
                          <div class="mt-1 text-sm text-[color:var(--text-secondary)]">
                            只需要决定这些已存在的公司要保留当前版本，还是改成导入版本。
                          </div>
                        </div>
                        <div class="flex items-center gap-2">
                          <Button
                            variant="ghost"
                            class="h-8 px-3 text-xs"
                            onClick={() => profiles.applyDecisionToAll("skip")}
                          >
                            全部保留当前
                          </Button>
                          <Button
                            variant="ghost"
                            class="h-8 px-3 text-xs"
                            onClick={() => profiles.applyDecisionToAll("replace")}
                          >
                            全部用导入版本替换
                          </Button>
                        </div>
                      </div>
                    </div>

                    <div class="space-y-4">
                      <For each={currentPreview().conflicts}>
                        {(conflict) => {
                          const decision =
                            profiles.state.transfer.decisions[conflict.imported.profile_id]
                          return (
                            <div class="rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
                              <div class="flex flex-wrap items-center gap-2">
                                <div class="text-base font-semibold text-[color:var(--text-primary)]">
                                  {conflict.imported.company_name}
                                </div>
                                <For each={conflict.reasons}>
                                  {(reason) => (
                                    <span class="rounded-full bg-amber-100 px-2 py-0.5 text-[11px] font-medium text-amber-700">
                                      {reason}
                                    </span>
                                  )}
                                </For>
                              </div>

                              <div class="mt-4 grid gap-4 md:grid-cols-2">
                                <SummaryBlock
                                  title="你当前的版本"
                                  updatedAt={conflict.existing.updated_at}
                                  eventCount={conflict.existing.event_count}
                                  thesisExcerpt={conflict.existing.thesis_excerpt}
                                />
                                <SummaryBlock
                                  title="导入版本"
                                  updatedAt={conflict.imported.updated_at}
                                  eventCount={conflict.imported.event_count}
                                  thesisExcerpt={conflict.imported.thesis_excerpt}
                                />
                              </div>

                              <div class="mt-4 flex flex-wrap gap-2">
                                <Button
                                  variant="ghost"
                                  class={[
                                    "h-9 px-4 text-sm",
                                    decision === "skip"
                                      ? "border border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                                      : "",
                                  ].join(" ")}
                                  onClick={() =>
                                    profiles.setConflictDecision(
                                      conflict.imported.profile_id,
                                      "skip",
                                    )
                                  }
                                >
                                  保留我当前的
                                </Button>
                                <Button
                                  class={[
                                    "h-9 px-4 text-sm",
                                    decision === "replace"
                                      ? "ring-2 ring-[color:var(--accent)]"
                                      : "",
                                  ].join(" ")}
                                  onClick={() =>
                                    profiles.setConflictDecision(
                                      conflict.imported.profile_id,
                                      "replace",
                                    )
                                  }
                                >
                                  用导入版本替换
                                </Button>
                              </div>
                            </div>
                          )
                        }}
                      </For>
                    </div>
                  </Show>

                  <div class="flex items-center justify-end">
                    <Button
                      class="h-10 px-5 text-sm"
                      disabled={!profiles.transferReady() || profiles.state.transfer.applying}
                      onClick={() => void profiles.applyImport().catch(() => undefined)}
                    >
                      {profiles.state.transfer.applying ? "导入中…" : "开始导入"}
                    </Button>
                  </div>
                </div>
              )}
            </Show>

            <Show when={!preview()}>
              <Show
                when={profile()}
                fallback={
                  <EmptyState
                    title={
                      (profiles.profiles() ?? []).length > 0
                        ? "先选择一家公司"
                        : "这个空间还没有公司画像"
                    }
                    description={
                      (profiles.profiles() ?? []).length > 0
                        ? "上面已经列出这个空间里的公司，点一家公司就能查看详情。"
                        : "可以直接导入别人整理好的画像包，或者让 agent 为这个用户新建首份公司画像。"
                    }
                    action={
                      <Button class="h-9 px-4 text-sm" onClick={openFilePicker}>
                        选择画像包
                      </Button>
                    }
                  />
                }
              >
                {(current) => (
                  <div class="space-y-5">
                    <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
                      <div class="mb-3 flex items-center justify-between gap-3">
                        <div class="text-sm font-semibold">画像主文件</div>
                        <div class="text-xs text-[color:var(--text-muted)]">
                          直接展示当前目录中的 `profile.md`
                        </div>
                      </div>
                      <Markdown text={current().markdown} />
                    </div>

                    <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
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
                  </div>
                )}
              </Show>
            </Show>
            </div>
          </div>
        </div>
      )}
    </Show>
  )
}
