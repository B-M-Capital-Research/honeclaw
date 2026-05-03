import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Markdown } from "@hone-financial/ui/markdown"
import { For, Show } from "solid-js"
import { actorLabel } from "@/lib/actors"
import { useCompanyProfiles } from "@/context/company-profiles"
import { COMPANY_PROFILES } from "@/lib/admin-content/company-profiles"
import { tpl, useLocale } from "@/lib/i18n"

function formatDate(iso?: string) {
  if (!iso) return COMPANY_PROFILES.detail.date_unknown
  try {
    return new Date(iso).toLocaleString(useLocale() === "zh" ? "zh-CN" : "en-US", {
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
  mainlineExcerpt: string
}) {
  return (
    <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
      <div class="text-sm font-semibold text-[color:var(--text-primary)]">
        {props.title}
      </div>
      <div class="mt-1 flex flex-wrap gap-3 text-[11px] text-[color:var(--text-muted)]">
        <span>{tpl(COMPANY_PROFILES.detail.summary_updated_at, { date: formatDate(props.updatedAt) })}</span>
        <span>{tpl(COMPANY_PROFILES.detail.summary_event_count, { count: props.eventCount })}</span>
      </div>
      <div class="mt-3 text-sm leading-6 text-[color:var(--text-secondary)]">
        {props.mainlineExcerpt || COMPANY_PROFILES.detail.summary_no_mainline}
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
          title={COMPANY_PROFILES.detail.empty_title}
          description={COMPANY_PROFILES.detail.empty_description}
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
                <span>{COMPANY_PROFILES.detail.space_label}</span>
                <span>{tpl(COMPANY_PROFILES.detail.company_count, { count: (profiles.profiles() ?? []).length })}</span>
                <Show when={profile()}>
                  <span>{tpl(COMPANY_PROFILES.detail.current_viewing, { title: profile()!.title })}</span>
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
                {profiles.state.exporting ? COMPANY_PROFILES.detail.exporting_button : COMPANY_PROFILES.detail.export_button}
              </Button>
              <Button class="h-9 px-3 text-sm" onClick={openFilePicker}>
                {COMPANY_PROFILES.detail.import_button}
              </Button>
              <Show when={profile()}>
                <Button
                  variant="ghost"
                  class="h-9 px-3 text-sm text-rose-500 hover:text-rose-600"
                  disabled={profiles.state.deleting}
                  onClick={async () => {
                    if (!confirm(tpl(COMPANY_PROFILES.detail.delete_confirm, { title: profile()!.title }))) {
                      return
                    }
                    await profiles.removeProfile(profile()!.profile_id)
                  }}
                >
                  {profiles.state.deleting ? COMPANY_PROFILES.detail.deleting_button : COMPANY_PROFILES.detail.delete_button}
                </Button>
              </Show>
            </div>
          </div>

          {/* 顶部横向公司选择条 — 取代原来的 240px 内嵌列,避免在 /users 下三层嵌套 */}
          <Show when={(profiles.profiles() ?? []).length > 0}>
            <div class="hf-scrollbar shrink-0 overflow-x-auto border-b border-[color:var(--border)] bg-[color:var(--panel)] px-6 py-2">
              <div class="flex min-w-max items-center gap-2">
                <span class="shrink-0 text-[11px] uppercase tracking-wider text-[color:var(--text-muted)]">
                  {tpl(COMPANY_PROFILES.detail.company_list_label, { count: (profiles.profiles() ?? []).length })}
                </span>
                <For each={profiles.profiles() ?? []}>
                  {(item) => (
                    <button
                      type="button"
                      class={[
                        "shrink-0 rounded-md border px-3 py-1 text-xs transition",
                        profiles.state.currentProfileId === item.profile_id
                          ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)] text-[color:var(--text-primary)]"
                          : "border-[color:var(--border)] bg-[color:var(--surface)] text-[color:var(--text-secondary)] hover:border-[color:var(--accent)]/50 hover:text-[color:var(--text-primary)]",
                      ].join(" ")}
                      onClick={() => profiles.selectProfile(item.profile_id)}
                      title={`${item.title} · ${formatDate(item.updated_at)}`}
                    >
                      <span class="font-medium">{item.title}</span>
                      <Show
                        when={profiles.state.highlightedProfileIds.includes(
                          item.profile_id,
                        )}
                      >
                        <span class="ml-1.5 rounded-full bg-emerald-100 px-1.5 py-0.5 text-[9px] font-medium text-emerald-700">
                          {COMPANY_PROFILES.detail.updated_badge}
                        </span>
                      </Show>
                    </button>
                  )}
                </For>
              </div>
            </div>
          </Show>

          <div class="flex min-h-0 flex-1 overflow-hidden">
            {/* 主视图（预览、文档等） */}
            <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-6 py-5 bg-[color:var(--surface)]">

            <Show when={transferError()}>
              <div class="mb-4 rounded-lg border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
                {transferError()}
              </div>
            </Show>

            <Show when={result()}>
              <div class="mb-4 rounded-lg border border-emerald-200 bg-emerald-50 p-4">
                <div class="text-sm font-semibold text-emerald-800">{COMPANY_PROFILES.detail.import_done_title}</div>
                <div class="mt-1 text-sm text-emerald-700">
                  {tpl(COMPANY_PROFILES.detail.import_done_summary, {
                    imported: result()!.imported_count,
                    replaced: result()!.replaced_count,
                    skipped: result()!.skipped_count,
                  })}
                </div>
                <Show when={profiles.state.transfer.backupBlob}>
                  <div class="mt-3">
                    <Button
                      variant="ghost"
                      class="h-8 px-3 text-xs"
                      onClick={() => profiles.downloadBackup()}
                    >
                      {COMPANY_PROFILES.detail.download_backup_button}
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
                      {COMPANY_PROFILES.detail.package_loaded_title}
                    </div>
                    <div class="mt-1 text-sm text-[color:var(--text-secondary)]">
                      {tpl(COMPANY_PROFILES.detail.package_loaded_summary, {
                        profiles: currentPreview().profiles.length,
                        conflicts: currentPreview().conflict_count,
                      })}
                    </div>
                    <div class="mt-3 flex items-center justify-center gap-2">
                      <Button
                        variant="ghost"
                        class="h-8 px-3 text-xs"
                        onClick={openFilePicker}
                      >
                        {COMPANY_PROFILES.detail.reselect_package_button}
                      </Button>
                      <Button
                        variant="ghost"
                        class="h-8 px-3 text-xs"
                        onClick={() => profiles.resetTransfer()}
                      >
                        {COMPANY_PROFILES.detail.cancel_import_button}
                      </Button>
                    </div>
                  </div>

                  <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
                    <div class="text-sm font-semibold">{COMPANY_PROFILES.detail.scan_result_title}</div>
                    <div class="mt-2 grid gap-3 text-sm text-[color:var(--text-secondary)] md:grid-cols-3">
                      <div>{tpl(COMPANY_PROFILES.detail.scan_total, { count: currentPreview().profiles.length })}</div>
                      <div>{tpl(COMPANY_PROFILES.detail.scan_importable, { count: currentPreview().importable_count })}</div>
                      <div>{tpl(COMPANY_PROFILES.detail.scan_need_confirm, { count: currentPreview().conflict_count })}</div>
                    </div>
                  </div>

                  <Show
                    when={currentPreview().conflict_count > 0}
                    fallback={
                      <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
                        <div class="text-base font-semibold text-[color:var(--text-primary)]">
                          {COMPANY_PROFILES.detail.no_conflicts_title}
                        </div>
                        <div class="mt-2 text-sm text-[color:var(--text-secondary)]">
                          {tpl(COMPANY_PROFILES.detail.no_conflicts_description, { count: currentPreview().profiles.length })}
                        </div>
                      </div>
                    }
                  >
                    <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
                      <div class="flex flex-wrap items-center justify-between gap-3">
                        <div>
                          <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                            {COMPANY_PROFILES.detail.conflicts_title}
                          </div>
                          <div class="mt-1 text-sm text-[color:var(--text-secondary)]">
                            {COMPANY_PROFILES.detail.conflicts_description}
                          </div>
                        </div>
                        <div class="flex items-center gap-2">
                          <Button
                            variant="ghost"
                            class="h-8 px-3 text-xs"
                            onClick={() => profiles.applyDecisionToAll("skip")}
                          >
                            {COMPANY_PROFILES.detail.keep_all_button}
                          </Button>
                          <Button
                            variant="ghost"
                            class="h-8 px-3 text-xs"
                            onClick={() => profiles.applyDecisionToAll("replace")}
                          >
                            {COMPANY_PROFILES.detail.replace_all_button}
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
                                  title={COMPANY_PROFILES.detail.side_existing_title}
                                  updatedAt={conflict.existing.updated_at}
                                  eventCount={conflict.existing.event_count}
                                  mainlineExcerpt={conflict.existing.mainline_excerpt}
                                />
                                <SummaryBlock
                                  title={COMPANY_PROFILES.detail.side_imported_title}
                                  updatedAt={conflict.imported.updated_at}
                                  eventCount={conflict.imported.event_count}
                                  mainlineExcerpt={conflict.imported.mainline_excerpt}
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
                                  {COMPANY_PROFILES.detail.keep_button}
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
                                  {COMPANY_PROFILES.detail.replace_button}
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
                      {profiles.state.transfer.applying ? COMPANY_PROFILES.detail.importing_button : COMPANY_PROFILES.detail.start_import_button}
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
                        ? COMPANY_PROFILES.detail.pick_company_title
                        : COMPANY_PROFILES.detail.empty_space_title
                    }
                    description={
                      (profiles.profiles() ?? []).length > 0
                        ? COMPANY_PROFILES.detail.pick_company_description
                        : COMPANY_PROFILES.detail.empty_space_description
                    }
                    action={
                      <Button class="h-9 px-4 text-sm" onClick={openFilePicker}>
                        {COMPANY_PROFILES.detail.pick_package_button}
                      </Button>
                    }
                  />
                }
              >
                {(current) => (
                  <div class="space-y-5">
                    <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
                      <div class="mb-3 flex items-center justify-between gap-3">
                        <div class="text-sm font-semibold">{COMPANY_PROFILES.detail.main_file_title}</div>
                        <div class="text-xs text-[color:var(--text-muted)]">
                          {COMPANY_PROFILES.detail.main_file_subtitle}
                        </div>
                      </div>
                      <Markdown text={current().markdown} />
                    </div>

                    <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
                      <div class="mb-3 flex items-center justify-between">
                        <div class="text-sm font-semibold">{COMPANY_PROFILES.detail.events_title}</div>
                        <div class="text-xs text-[color:var(--text-muted)]">
                          {tpl(COMPANY_PROFILES.detail.events_count, { count: current().events.length })}
                        </div>
                      </div>

                      <Show
                        when={current().events.length > 0}
                        fallback={
                          <div class="text-sm text-[color:var(--text-muted)]">
                            {COMPANY_PROFILES.detail.events_empty}
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
