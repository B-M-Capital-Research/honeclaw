import { Show, createSignal } from "solid-js"
import { useKb } from "@/context/kb"
import { analyzeKbEntry } from "@/lib/api"

function formatDate(iso: string): string {
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

function MetaRow(props: { label: string; value: string }) {
  return (
    <div class="flex gap-3 py-1.5 text-sm">
      <span class="w-20 shrink-0 text-[color:var(--text-muted)]">{props.label}</span>
      <span class="min-w-0 break-all text-[color:var(--text-primary)]">{props.value}</span>
    </div>
  )
}

export function KbDetail() {
  const kb = useKb()
  const [syncState, setSyncState] = createSignal<"idle" | "loading" | "error">("idle")

  const handleSync = async (id: string) => {
    if (syncState() === "loading") return
    setSyncState("loading")
    try {
      await analyzeKbEntry(id)
      // 重新加载详情，获取最新的 analyzed_at
      await kb.selectEntry(id)
      setSyncState("idle")
    } catch {
      setSyncState("error")
      setTimeout(() => setSyncState("idle"), 3000)
    }
  }

  const syncButtonClass = (analyzedAt: string | undefined) => {
    if (syncState() === "loading") {
      return "cursor-not-allowed border-[color:var(--border)] bg-[color:var(--surface)] text-[color:var(--text-muted)]"
    }
    if (syncState() === "error") {
      return "border-rose-300 bg-rose-50 text-rose-700"
    }
    if (analyzedAt) {
      return "border-emerald-300 bg-emerald-50 text-emerald-700 hover:opacity-80"
    }
    return "border-[color:var(--accent)] bg-[color:var(--accent-soft)] text-[color:var(--accent)] hover:opacity-80"
  }

  const syncButtonLabel = (analyzedAt: string | undefined) => {
    if (syncState() === "loading") return "分析中…"
    if (syncState() === "error") return "✗ 失败，点击重试"
    if (analyzedAt) return `✓ 已同步 · ${formatDate(analyzedAt)}`
    return "同步到知识"
  }

  return (
    <div class="flex h-full flex-col overflow-hidden">
      <Show when={!kb.state.selectedId}>
        <div class="flex flex-1 items-center justify-center text-sm text-[color:var(--text-muted)]">
          从左侧列表选择一个文件查看详情
        </div>
      </Show>

      <Show when={kb.state.detailLoading}>
        <div class="flex flex-1 items-center justify-center text-sm text-[color:var(--text-muted)]">
          加载中…
        </div>
      </Show>

      <Show when={!kb.state.detailLoading && kb.state.selectedEntry}>
        {(entry) => (
          <div class="flex h-full flex-col gap-4 overflow-y-auto p-4">
            {/* 文件名 + 同步按钮 */}
            <div class="flex items-start justify-between gap-3">
              <h2 class="break-all text-lg font-semibold text-[color:var(--text-primary)]">
                {entry().filename}
              </h2>
              <button
                type="button"
                disabled={syncState() === "loading"}
                onClick={() => void handleSync(entry().id)}
                class={[
                  "shrink-0 rounded-md border px-3 py-1.5 text-xs font-medium transition",
                  syncButtonClass(entry().analyzed_at),
                ].join(" ")}
              >
                {syncButtonLabel(entry().analyzed_at)}
              </button>
            </div>

            {/* 元数据区 */}
            <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] px-4 py-2 divide-y divide-[color:var(--border)]">
              <MetaRow label="渠道" value={entry().channel} />
              <MetaRow label="用户" value={entry().user_id} />
              <MetaRow label="类型" value={entry().kind} />
              <MetaRow label="大小" value={`${(entry().size / 1024).toFixed(1)} KB`} />
              <MetaRow label="上传时间" value={formatDate(entry().uploaded_at)} />
              <MetaRow label="解析状态" value={entry().parse_status} />
              {entry().parse_error && (
                <MetaRow label="解析错误" value={entry().parse_error!} />
              )}
              {entry().analyzed_at && (
                <MetaRow label="同步时间" value={formatDate(entry().analyzed_at!)} />
              )}
            </div>

            {/* 解析文本 */}
            <Show
              when={kb.state.parsedText}
              fallback={
                <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] px-4 py-6 text-center text-sm text-[color:var(--text-muted)]">
                  {entry().parse_status === "skipped"
                    ? "该文件类型不支持文本提取"
                    : "无可用解析文本"}
                </div>
              }
            >
              <div class="flex flex-col gap-2">
                <div class="text-xs font-medium uppercase tracking-wider text-[color:var(--text-muted)]">
                  提取文本
                </div>
                <pre class="min-h-0 flex-1 overflow-y-auto whitespace-pre-wrap break-words rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4 font-mono text-xs leading-relaxed text-[color:var(--text-primary)]">
                  {kb.state.parsedText}
                </pre>
              </div>
            </Show>
          </div>
        )}
      </Show>
    </div>
  )
}
