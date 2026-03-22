import { createResource, createSignal, createEffect, For, Show, Switch, Match } from "solid-js"
import { A } from "@solidjs/router"
import { getKbStockTable, updateStockKnowledge } from "@/lib/api"
import type { StockRow } from "@/lib/types"

// ── 辅助 ─────────────────────────────────────────────────────────────────────

function formatDate(iso: string): string {
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

// ── 重点知识编辑单元格 ────────────────────────────────────────────────────────

function KnowledgeCell(props: {
  row: StockRow
  onSave: (items: string[]) => Promise<void>
}) {
  const [editing, setEditing] = createSignal(false)
  const [items, setItems] = createSignal<string[]>([...(props.row.key_knowledge ?? [])])
  const [saving, setSaving] = createSignal(false)

  // 退出编辑时同步外部变更（行数据刷新）
  createEffect(() => {
    if (!editing()) {
      setItems([...(props.row.key_knowledge ?? [])])
    }
  })

  const addItem = () => {
    setItems((prev) => [...prev, ""])
  }

  const removeItem = (i: number) => {
    setItems((prev) => prev.filter((_, idx) => idx !== i))
  }

  const updateItem = (i: number, val: string) => {
    setItems((prev) => prev.map((item, idx) => (idx === i ? val : item)))
  }

  const handleSave = async () => {
    const clean = items()
      .map((s) => s.trim())
      .filter((s) => s.length > 0)
    setSaving(true)
    try {
      await props.onSave(clean)
      setEditing(false)
    } finally {
      setSaving(false)
    }
  }

  const handleCancel = () => {
    setItems([...(props.row.key_knowledge ?? [])])
    setEditing(false)
  }

  return (
    <td class="px-3 py-2.5 align-top">
      <Show
        when={editing()}
        fallback={
          /* ── 查看模式 ── */
          <div class="group flex items-start gap-2">
            <div class="min-w-0 flex-1">
              <Show
                when={(props.row.key_knowledge ?? []).length > 0}
                fallback={
                  <span class="text-xs italic text-[color:var(--text-muted)]">暂无重点知识</span>
                }
              >
                <ul class="space-y-1">
                  <For each={props.row.key_knowledge ?? []}>
                    {(item) => (
                      <li class="flex items-start gap-1.5 text-xs text-[color:var(--text-secondary)]">
                        <span class="mt-[5px] h-1.5 w-1.5 shrink-0 rounded-full bg-[color:var(--accent)]" />
                        <span class="leading-snug">{item}</span>
                      </li>
                    )}
                  </For>
                </ul>
              </Show>
            </div>
            {/* 悬停显示编辑按钮 */}
            <button
              class="shrink-0 rounded p-0.5 text-[11px] text-[color:var(--text-muted)] opacity-0 transition-opacity hover:bg-[color:var(--accent-soft)] hover:text-[color:var(--accent)] group-hover:opacity-100"
              onClick={() => setEditing(true)}
              title="编辑重点知识"
            >
              ✎
            </button>
          </div>
        }
      >
        {/* ── 编辑模式 ── */}
        <div class="flex flex-col gap-1.5">
          <For each={items()}>
            {(item, i) => (
              <div class="flex items-center gap-1">
                <input
                  class="min-w-0 flex-1 rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-xs text-[color:var(--text-primary)] focus:border-[color:var(--accent)] focus:outline-none"
                  value={item}
                  placeholder="输入知识条目…"
                  onInput={(e) => updateItem(i(), e.currentTarget.value)}
                  onKeyDown={(e) => {
                    if (e.isComposing) return
                    if (e.key === "Enter") {
                      e.preventDefault()
                      addItem()
                    }
                    if (e.key === "Escape") handleCancel()
                  }}
                />
                <button
                  class="shrink-0 text-sm text-[color:var(--text-muted)] hover:text-rose-400"
                  onClick={() => removeItem(i())}
                  title="删除此条"
                >
                  ×
                </button>
              </div>
            )}
          </For>

          <button
            class="self-start text-xs text-[color:var(--accent)] hover:opacity-70"
            onClick={addItem}
          >
            + 添加条目
          </button>

          <div class="mt-0.5 flex gap-2">
            <button
              class="rounded bg-[color:var(--accent)] px-2.5 py-1 text-xs text-white disabled:opacity-50"
              onClick={handleSave}
              disabled={saving()}
            >
              {saving() ? "保存中…" : "保存"}
            </button>
            <button
              class="text-xs text-[color:var(--text-muted)] hover:text-[color:var(--text-primary)]"
              onClick={handleCancel}
            >
              取消
            </button>
          </div>
        </div>
      </Show>
    </td>
  )
}

// ── 单行 ─────────────────────────────────────────────────────────────────────

function StockRowItem(props: {
  row: StockRow
  index: number
  onSaveKnowledge: (items: string[]) => Promise<void>
}) {
  return (
    <tr class={props.index % 2 === 0 ? "bg-[color:var(--surface)]" : "bg-[color:var(--panel)]"}>
      {/* 公司名称 + 更新时间 */}
      <td class="px-3 py-2.5 align-top">
        <div class="text-sm font-medium text-[color:var(--text-primary)]">
          {props.row.company_name}
        </div>
        <div class="mt-0.5 text-[10px] text-[color:var(--text-muted)]">
          {formatDate(props.row.updated_at)}
        </div>
      </td>

      {/* 股票代码 */}
      <td class="px-3 py-2.5 align-top">
        <Show
          when={props.row.stock_code}
          fallback={<span class="text-xs text-[color:var(--text-muted)]">—</span>}
        >
          <span class="rounded bg-[color:var(--accent-soft)] px-2 py-0.5 font-mono text-xs font-semibold text-[color:var(--accent)]">
            {props.row.stock_code}
          </span>
        </Show>
      </td>

      {/* 相关文件 */}
      <td class="px-3 py-2.5 align-top">
        <div class="flex flex-col gap-2">
          <For each={props.row.related_files}>
            {(file) => (
              <A
                href={`/kb/${encodeURIComponent(file.kb_id)}`}
                class="block rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1.5 transition hover:border-[color:var(--accent)] hover:bg-[color:var(--accent-soft)]"
              >
                <div class="flex items-center gap-1.5">
                  <span class="max-w-xs truncate text-xs font-medium text-[color:var(--accent)]">
                    {file.filename}
                  </span>
                  <span class="shrink-0 text-[9px] text-[color:var(--text-muted)]">↗</span>
                </div>
                <Show when={file.summary}>
                  <div class="mt-0.5 text-[11px] leading-snug text-[color:var(--text-secondary)]">
                    {file.summary}
                  </div>
                </Show>
              </A>
            )}
          </For>
          <Show when={props.row.related_files.length === 0}>
            <span class="text-xs text-[color:var(--text-muted)]">—</span>
          </Show>
        </div>
      </td>

      {/* 重点知识（可编辑） */}
      <KnowledgeCell row={props.row} onSave={props.onSaveKnowledge} />
    </tr>
  )
}

// ── 主组件 ────────────────────────────────────────────────────────────────────

export function KbStockTable() {
  const [rows, { refetch }] = createResource<StockRow[]>(getKbStockTable)

  const handleSaveKnowledge = async (row: StockRow, items: string[]) => {
    await updateStockKnowledge({
      company_name: row.company_name,
      stock_code: row.stock_code,
      key_knowledge: items,
    })
    await refetch()
  }

  return (
    <div class="flex flex-col gap-3">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-semibold text-[color:var(--text-primary)]">股票信息表</h3>
        <Show when={rows()}>
          <span class="text-xs text-[color:var(--text-muted)]">{rows()!.length} 家公司</span>
        </Show>
      </div>

      <Show
        when={!rows.loading}
        fallback={
          <div class="py-4 text-center text-sm text-[color:var(--text-muted)]">加载中…</div>
        }
      >
        <Switch>
          {/* 先检查错误，避免 rows() 在错误状态下抛出异常 */}
          <Match when={rows.error}>
            <div class="rounded-lg border border-rose-500/30 bg-rose-500/10 px-3 py-2 text-sm text-rose-300">
              加载失败：{String(rows.error)}
            </div>
          </Match>

          <Match when={(rows() ?? []).length > 0}>
            <div class="overflow-x-auto rounded-lg border border-[color:var(--border)]">
              <table class="w-full border-collapse text-left">
                <thead>
                  <tr class="border-b border-[color:var(--border)] bg-[color:var(--panel)]">
                    <th class="px-3 py-2 text-xs font-semibold uppercase tracking-wider text-[color:var(--text-muted)]">
                      公司名称
                    </th>
                    <th class="px-3 py-2 text-xs font-semibold uppercase tracking-wider text-[color:var(--text-muted)]">
                      股票代码
                    </th>
                    <th class="px-3 py-2 text-xs font-semibold uppercase tracking-wider text-[color:var(--text-muted)]">
                      相关文件
                    </th>
                    <th class="px-3 py-2 text-xs font-semibold uppercase tracking-wider text-[color:var(--text-muted)]">
                      重点知识
                    </th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-[color:var(--border)]">
                  <For each={rows() ?? []}>
                    {(row, i) => (
                      <StockRowItem
                        row={row}
                        index={i()}
                        onSaveKnowledge={(items) => handleSaveKnowledge(row, items)}
                      />
                    )}
                  </For>
                </tbody>
              </table>
            </div>
          </Match>

          <Match when={true}>
            <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] py-6 text-center text-sm text-[color:var(--text-muted)]">
              尚未提取到任何公司/股票信息。上传 PDF 文档后自动分析。
            </div>
          </Match>
        </Switch>
      </Show>
    </div>
  )
}
