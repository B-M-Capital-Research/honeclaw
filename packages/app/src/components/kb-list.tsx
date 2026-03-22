import { createEffect, createSignal, Show } from "solid-js"
import { For } from "solid-js"
import { useLocation, useNavigate, useParams } from "@solidjs/router"
import { useKb } from "@/context/kb"
import type { KbEntry } from "@/lib/types"

// ── 辅助函数 ──────────────────────────────────────────────────────────────────

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function kindLabel(kind: string): string {
  const map: Record<string, string> = {
    Pdf: "PDF",
    Image: "图片",
    Spreadsheet: "表格",
    Text: "文本",
    Audio: "音频",
    Video: "视频",
    Archive: "压缩包",
    Other: "其他",
  }
  return map[kind] ?? kind
}

function channelLabel(channel: string): string {
  const map: Record<string, string> = {
    feishu: "飞书",
    discord: "Discord",
    console: "控制台",
  }
  return map[channel] ?? channel
}

function parseStatusBadge(status: string) {
  const map: Record<string, { text: string; cls: string }> = {
    ok: { text: "已解析", cls: "bg-emerald-100 text-emerald-700" },
    failed: { text: "解析失败", cls: "bg-rose-100 text-rose-700" },
    empty: { text: "空文本", cls: "bg-amber-100 text-amber-700" },
    skipped: { text: "跳过", cls: "bg-gray-100 text-gray-500" },
  }
  return map[status] ?? { text: status, cls: "bg-gray-100 text-gray-500" }
}

// ── 三点菜单 ──────────────────────────────────────────────────────────────────

function KbItemMenu(props: {
  entry: KbEntry
  onDelete: () => void
}) {
  const [open, setOpen] = createSignal(false)
  const [confirming, setConfirming] = createSignal(false)
  const [deleting, setDeleting] = createSignal(false)
  const kb = useKb()

  const handleDelete = async (e: MouseEvent) => {
    e.stopPropagation()
    if (!confirming()) {
      setConfirming(true)
      return
    }
    setDeleting(true)
    try {
      await kb.deleteEntry(props.entry.id)
      props.onDelete()
    } catch (err) {
      console.error("删除失败", err)
    } finally {
      setDeleting(false)
      setConfirming(false)
      setOpen(false)
    }
  }

  const handleCancelDelete = (e: MouseEvent) => {
    e.stopPropagation()
    setConfirming(false)
  }

  const handleToggle = (e: MouseEvent) => {
    e.stopPropagation()
    setOpen((v) => !v)
    setConfirming(false)
  }

  // 点击外部关闭
  const handleBlur = () => {
    // 延迟以便内部 click 先执行
    setTimeout(() => {
      setOpen(false)
      setConfirming(false)
    }, 150)
  }

  return (
    <div class="relative shrink-0" onBlur={handleBlur}>
      <button
        type="button"
        tabIndex={0}
        onClick={handleToggle}
        class="flex h-6 w-6 items-center justify-center rounded text-[color:var(--text-muted)] opacity-0 transition hover:bg-[color:var(--border)] hover:text-[color:var(--text-primary)] group-hover:opacity-100"
        title="更多操作"
      >
        <svg class="h-4 w-4" viewBox="0 0 16 16" fill="currentColor">
          <circle cx="8" cy="3" r="1.2" />
          <circle cx="8" cy="8" r="1.2" />
          <circle cx="8" cy="13" r="1.2" />
        </svg>
      </button>

      <Show when={open()}>
        <div
          class="absolute right-0 top-7 z-50 min-w-[110px] rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] py-1 shadow-lg"
          onClick={(e) => e.stopPropagation()}
        >
          <Show
            when={!confirming()}
            fallback={
              <div class="px-3 py-2">
                <div class="mb-1.5 text-xs text-[color:var(--text-primary)]">确认删除？</div>
                <div class="flex gap-1.5">
                  <button
                    type="button"
                    disabled={deleting()}
                    onClick={handleDelete}
                    class="flex-1 rounded bg-rose-500 px-2 py-1 text-[11px] font-medium text-white hover:bg-rose-600 disabled:opacity-60"
                  >
                    {deleting() ? "删除中…" : "确认"}
                  </button>
                  <button
                    type="button"
                    onClick={handleCancelDelete}
                    class="flex-1 rounded border border-[color:var(--border)] px-2 py-1 text-[11px] text-[color:var(--text-muted)] hover:bg-[color:var(--accent-soft)]"
                  >
                    取消
                  </button>
                </div>
              </div>
            }
          >
            <button
              type="button"
              onClick={handleDelete}
              class="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-rose-600 hover:bg-rose-50"
            >
              <svg class="h-3.5 w-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M2 4h12M5 4V2h6v2M6 7v5M10 7v5M3 4l1 9h8l1-9" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
              删除
            </button>
          </Show>
        </div>
      </Show>
    </div>
  )
}

// ── 单条 item ──────────────────────────────────────────────────────────────────

function KbItem(props: { entry: KbEntry; selected: boolean; onClick: () => void; onDeleted: () => void }) {
  const badge = () => parseStatusBadge(props.entry.parse_status)
  return (
    <div
      class={[
        "group relative w-full rounded-lg px-3 py-2.5 text-left transition hover:bg-[color:var(--accent-soft)] cursor-pointer",
        props.selected
          ? "bg-[color:var(--accent-soft)] ring-1 ring-[color:var(--accent)]"
          : "",
      ].join(" ")}
      onClick={props.onClick}
    >
      <div class="flex items-start justify-between gap-2">
        <div class="min-w-0 flex-1">
          <div class="truncate text-sm font-medium text-[color:var(--text-primary)]">
            {props.entry.filename}
          </div>
          <div class="mt-1 flex flex-wrap items-center gap-1.5 text-[11px] text-[color:var(--text-muted)]">
            <span>{channelLabel(props.entry.channel)}</span>
            <span>·</span>
            <span>{kindLabel(props.entry.kind)}</span>
            <span>·</span>
            <span>{formatSize(props.entry.size)}</span>
          </div>
        </div>
        <div class="flex shrink-0 items-center gap-1.5">
          <span
            class={[
              "rounded px-1.5 py-0.5 text-[10px] font-medium",
              badge().cls,
            ].join(" ")}
          >
            {badge().text}
          </span>
          <KbItemMenu entry={props.entry} onDelete={props.onDeleted} />
        </div>
      </div>
    </div>
  )
}

// ── 主组件 ────────────────────────────────────────────────────────────────────

export function KbList() {
  const kb = useKb()
  const navigate = useNavigate()
  const params = useParams()
  const location = useLocation()

  let fileInputRef!: HTMLInputElement
  const [uploading, setUploading] = createSignal(false)
  const [uploadError, setUploadError] = createSignal<string | null>(null)

  // 首次挂载时加载列表
  createEffect(() => {
    if (location.pathname.startsWith("/kb")) {
      void kb.loadEntries()
    }
  })

  const selectedId = () => params.entryId ? decodeURIComponent(params.entryId) : null

  const handleUploadClick = () => {
    setUploadError(null)
    fileInputRef.click()
  }

  const handleFileChange = async (e: Event) => {
    const input = e.currentTarget as HTMLInputElement
    const file = input.files?.[0]
    input.value = ""
    if (!file) return
    setUploading(true)
    setUploadError(null)
    try {
      const entry = await kb.uploadFile(file)
      navigate(`/kb/${encodeURIComponent(entry.id)}`)
    } catch (err) {
      setUploadError(String(err))
    } finally {
      setUploading(false)
    }
  }

  const handleDeleted = (id: string) => {
    // 如果删除的是当前选中项，返回列表根
    if (selectedId() === id) {
      navigate("/kb")
    }
  }

  return (
    <aside class="flex h-full w-[300px] min-w-0 flex-col border-r border-[color:var(--border)] bg-[color:var(--panel)]">
      {/* 顶部标题 + 上传按钮 */}
      <div class="border-b border-[color:var(--border)] px-4 py-3">
        <div class="flex items-center justify-between gap-2">
          <div>
            <div class="text-sm font-semibold text-[color:var(--text-primary)]">知识库</div>
            <div class="mt-0.5 text-[11px] text-[color:var(--text-muted)]">
              {kb.state.entries.length} 个文件
            </div>
          </div>
          <button
            type="button"
            disabled={uploading()}
            onClick={handleUploadClick}
            class={[
              "flex items-center gap-1.5 rounded-md border px-2.5 py-1.5 text-xs font-medium transition",
              uploading()
                ? "cursor-not-allowed border-[color:var(--border)] bg-[color:var(--surface)] text-[color:var(--text-muted)]"
                : "border-[color:var(--accent)] bg-[color:var(--accent-soft)] text-[color:var(--accent)] hover:opacity-80",
            ].join(" ")}
          >
            <Show
              when={!uploading()}
              fallback={
                <>
                  <svg class="h-3.5 w-3.5 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" stroke-linecap="round"/>
                  </svg>
                  解析中…
                </>
              }
            >
              <svg class="h-3.5 w-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M8 2v9M4 7l4-5 4 5" stroke-linecap="round" stroke-linejoin="round" />
                <path d="M2 13h12" stroke-linecap="round" />
              </svg>
              上传并解析
            </Show>
          </button>
        </div>
        <Show when={uploadError()}>
          <div class="mt-2 rounded border border-rose-200 bg-rose-50 px-2 py-1 text-[11px] text-rose-700">
            {uploadError()}
          </div>
        </Show>
      </div>

      {/* 隐藏文件输入 */}
      <input
        ref={fileInputRef}
        type="file"
        class="hidden"
        onChange={handleFileChange}
      />

      {/* 文件列表 */}
      <div class="min-h-0 flex-1 overflow-y-auto p-2">
        <Show
          when={!kb.state.loading}
          fallback={
            <div class="py-8 text-center text-sm text-[color:var(--text-muted)]">加载中…</div>
          }
        >
          <Show
            when={kb.state.entries.length > 0}
            fallback={
              <div class="py-8 text-center text-sm text-[color:var(--text-muted)]">
                暂无文件。飞书或 Discord 渠道发送附件后将自动归档，或点击右上角上传。
              </div>
            }
          >
            <For each={kb.state.entries}>
              {(entry) => (
                <KbItem
                  entry={entry}
                  selected={selectedId() === entry.id}
                  onClick={() => navigate(`/kb/${encodeURIComponent(entry.id)}`)}
                  onDeleted={() => handleDeleted(entry.id)}
                />
              )}
            </For>
          </Show>
        </Show>
        <Show when={kb.state.error}>
          <div class="mt-3 rounded-lg border border-rose-200 bg-rose-50 px-3 py-2 text-sm text-rose-700">
            {kb.state.error}
          </div>
        </Show>
      </div>
    </aside>
  )
}
