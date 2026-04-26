// 管理端 — 查看任意 actor 的系统蒸馏 thesis 与画像 inventory
//
// 数据源:
// - GET /api/event-engine/thesis-context?channel=&user_id=&channel_scope=
// - GET /api/event-engine/company-profile?...&ticker=
// - POST /api/event-engine/thesis-distill?... (立即触发蒸馏)
//
// 与 public 端 /portfolio 一致,但 actor 由 URL 决定而非 session。

import { createSignal, For, Show, createEffect } from "solid-js"
import { marked } from "marked"
import DOMPurify from "dompurify"
import {
  adminTriggerThesisDistill,
  getAdminCompanyProfile,
  getAdminThesisContext,
  type AdminThesisContext,
} from "@/lib/api"
import type { ActorRef } from "@/lib/actors"

function formatTimestamp(iso: string | null): string {
  if (!iso) return "尚未蒸馏"
  try {
    const dt = new Date(iso)
    const days = Math.floor((Date.now() - dt.getTime()) / (24 * 3600 * 1000))
    if (days === 0)
      return `今天 ${dt.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })}`
    if (days === 1) return "1 天前"
    if (days < 7) return `${days} 天前`
    return dt.toLocaleDateString("zh-CN", { year: "numeric", month: "short", day: "numeric" })
  } catch {
    return iso
  }
}

function ProfileMarkdownModal(props: {
  open: boolean
  actor: ActorRef
  ticker: string | null
  onClose: () => void
}) {
  const [markdown, setMarkdown] = createSignal<string | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  createEffect(() => {
    const t = props.ticker
    if (!props.open || !t) {
      setMarkdown(null)
      return
    }
    setLoading(true)
    setError(null)
    setMarkdown(null)
    getAdminCompanyProfile(props.actor, t)
      .then((d) => setMarkdown(d.markdown))
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false))
  })

  const renderedHtml = () => {
    const md = markdown()
    if (!md) return ""
    const raw = marked.parse(md, { gfm: true, breaks: false }) as string
    return DOMPurify.sanitize(raw)
  }

  return (
    <Show when={props.open}>
      <div
        class="fixed inset-0 z-50 flex items-center justify-center p-8"
        style={{ background: "rgba(0,0,0,0.55)" }}
        onClick={props.onClose}
      >
        <div
          class="flex max-h-[90vh] w-full max-w-3xl flex-col overflow-hidden rounded-xl bg-[color:var(--panel)]"
          onClick={(e) => e.stopPropagation()}
        >
          <div class="flex items-center justify-between border-b border-[color:var(--border)] px-5 py-3">
            <div class="text-sm font-semibold">{props.ticker} · 公司画像</div>
            <button
              type="button"
              class="text-lg text-[color:var(--text-muted)]"
              onClick={props.onClose}
            >
              ×
            </button>
          </div>
          <div class="overflow-auto px-6 py-5 text-sm leading-7 text-[color:var(--text-primary)]">
            <Show when={loading()}>
              <div class="text-[color:var(--text-muted)]">加载中…</div>
            </Show>
            <Show when={error()}>
              <div class="text-red-400">{error()}</div>
            </Show>
            <Show when={markdown() && !loading()}>
              <div class="prose prose-invert max-w-none" innerHTML={renderedHtml()}></div>
            </Show>
          </div>
          <div class="border-t border-[color:var(--border)] bg-[color:var(--surface-elevated,rgba(0,0,0,0.2))] px-5 py-2 text-xs text-[color:var(--text-muted)]">
            画像由用户在 chat 里通过 company_portrait skill 维护,read-only。
          </div>
        </div>
      </div>
    </Show>
  )
}

export function UserThesisView(props: { actor: ActorRef }) {
  const [ctx, setCtx] = createSignal<AdminThesisContext | null>(null)
  const [loading, setLoading] = createSignal(true)
  const [error, setError] = createSignal<string | null>(null)
  const [refreshing, setRefreshing] = createSignal(false)
  const [refreshMsg, setRefreshMsg] = createSignal<string | null>(null)
  const [modalOpen, setModalOpen] = createSignal(false)
  const [modalTicker, setModalTicker] = createSignal<string | null>(null)

  const load = async () => {
    setLoading(true)
    setError(null)
    try {
      const data = await getAdminThesisContext(props.actor)
      setCtx(data)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  createEffect(() => {
    // actor 切换时重拉
    void props.actor.user_id
    void props.actor.channel
    void props.actor.channel_scope
    load()
  })

  const handleRefresh = async () => {
    setRefreshing(true)
    setRefreshMsg(null)
    try {
      const r = await adminTriggerThesisDistill(props.actor)
      setRefreshMsg(
        `蒸馏完成:${r.theses_count} 条 thesis,跳过 ${r.skipped_tickers.length} 只`,
      )
      await load()
    } catch (e) {
      setRefreshMsg(`蒸馏失败:${e instanceof Error ? e.message : String(e)}`)
    } finally {
      setRefreshing(false)
    }
  }

  const profileTickers = () => {
    const c = ctx()
    if (!c) return new Set<string>()
    const set = new Set<string>()
    for (const p of c.profile_list) {
      for (const t of p.tickers) set.add(t)
    }
    return set
  }

  const openProfile = (ticker: string) => {
    setModalTicker(ticker)
    setModalOpen(true)
  }

  return (
    <div class="h-full overflow-auto p-6">
      <Show when={loading()}>
        <div class="text-sm text-[color:var(--text-muted)]">加载中…</div>
      </Show>
      <Show when={error()}>
        <div class="rounded-md border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
          加载失败:{error()}
        </div>
      </Show>

      <Show when={ctx()}>
        {(c) => (
          <>
            {/* meta + 刷新 */}
            <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
              <div class="text-xs text-[color:var(--text-muted)]">
                上次蒸馏:
                <span class="text-[color:var(--text-primary)]">
                  {formatTimestamp(c().last_thesis_distilled_at)}
                </span>
                <Show when={c().thesis_distill_skipped.length > 0}>
                  <span class="ml-4">
                    跳过 {c().thesis_distill_skipped.length} 只:
                    <span class="ml-1 font-mono text-amber-400">
                      {c().thesis_distill_skipped.join(", ")}
                    </span>
                  </span>
                </Show>
              </div>
              <button
                type="button"
                onClick={handleRefresh}
                disabled={refreshing()}
                class="rounded-md bg-amber-500 px-3 py-1.5 text-xs font-medium text-white disabled:opacity-50"
              >
                {refreshing() ? "蒸馏中…" : "立即触发蒸馏"}
              </button>
            </div>
            <Show when={refreshMsg()}>
              <div class="mb-4 rounded-md border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-300">
                {refreshMsg()}
              </div>
            </Show>

            {/* 整体投资风格 */}
            <div class="mb-4 rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
              <div class="mb-2 text-[11px] font-semibold uppercase tracking-wider text-[color:var(--text-muted)]">
                整体投资风格
              </div>
              <div class="text-sm leading-7 text-[color:var(--text-primary)]">
                <Show
                  when={c().investment_global_style}
                  fallback={
                    <span class="text-[color:var(--text-muted)]">
                      尚未蒸馏 — 至少需要一个公司画像。
                    </span>
                  }
                >
                  {c().investment_global_style}
                </Show>
              </div>
            </div>

            {/* 持仓 thesis 卡片 */}
            <div class="mb-2 text-sm font-semibold">
              各持仓 Thesis ({c().holdings.length})
            </div>
            <Show
              when={c().holdings.length > 0}
              fallback={
                <div class="rounded-md border border-dashed border-[color:var(--border)] p-6 text-center text-sm text-[color:var(--text-muted)]">
                  该 actor 持仓为空。
                </div>
              }
            >
              <div class="mb-6 grid gap-3" style={{ "grid-template-columns": "repeat(auto-fill, minmax(280px, 1fr))" }}>
                <For each={c().holdings}>
                  {(ticker) => {
                    const thesis = c().investment_theses[ticker]
                    const hasProfile = profileTickers().has(ticker)
                    const isSkipped = c().thesis_distill_skipped.includes(ticker)
                    return (
                      <div
                        class="flex flex-col gap-2 rounded-md border p-4"
                        classList={{
                          "border-[color:var(--border)] bg-[color:var(--panel)]": !!thesis,
                          "border-amber-500/30 bg-amber-500/5": !thesis,
                        }}
                      >
                        <div class="flex items-baseline justify-between gap-2">
                          <div class="font-mono text-base font-bold">{ticker}</div>
                          <Show when={hasProfile}>
                            <button
                              type="button"
                              class="rounded border border-[color:var(--border)] px-2 py-0.5 text-[11px] hover:border-[color:var(--accent)]"
                              onClick={() => openProfile(ticker)}
                            >
                              查看画像
                            </button>
                          </Show>
                        </div>
                        <Show
                          when={thesis}
                          fallback={
                            <div class="text-xs leading-6 text-[color:var(--text-muted)]">
                              <Show
                                when={hasProfile}
                                fallback={
                                  <>
                                    <span class="font-semibold text-amber-400">没有公司画像</span>
                                    {" — "}用户需在 chat 里建档。
                                  </>
                                }
                              >
                                <span class="font-semibold text-amber-400">
                                  画像存在但 thesis 蒸馏失败
                                </span>
                                {isSkipped ? "(上次跳过)" : ""},点"立即触发蒸馏"重试。
                              </Show>
                            </div>
                          }
                        >
                          <div class="text-sm leading-6 text-[color:var(--text-primary)]">
                            {thesis}
                          </div>
                        </Show>
                      </div>
                    )
                  }}
                </For>
              </div>
            </Show>

            {/* 画像 inventory */}
            <div class="mb-2 text-sm font-semibold">
              公司画像 inventory ({c().profile_list.length})
            </div>
            <Show
              when={c().profile_list.length > 0}
              fallback={
                <div class="rounded-md border border-dashed border-[color:var(--border)] p-6 text-center text-sm text-[color:var(--text-muted)]">
                  该 actor sandbox 里还没有任何公司画像。
                </div>
              }
            >
              <div class="space-y-2">
                <For each={c().profile_list}>
                  {(p) => (
                    <div class="flex items-center justify-between gap-3 rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-3">
                      <div class="flex-1 min-w-0">
                        <div class="text-sm font-medium">
                          {p.title || p.dir}
                          <span class="ml-2 font-mono text-xs text-[color:var(--text-muted)]">
                            {p.tickers.join(" / ")}
                          </span>
                        </div>
                        <div class="mt-0.5 text-[11px] text-[color:var(--text-muted)]">
                          {(p.bytes / 1024).toFixed(1)} KB · {p.dir}
                        </div>
                      </div>
                      <Show when={p.tickers.length > 0}>
                        <button
                          type="button"
                          class="rounded border border-[color:var(--border)] px-2 py-1 text-[11px]"
                          onClick={() => openProfile(p.tickers[0])}
                        >
                          查看
                        </button>
                      </Show>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            <ProfileMarkdownModal
              open={modalOpen()}
              actor={props.actor}
              ticker={modalTicker()}
              onClose={() => setModalOpen(false)}
            />
          </>
        )}
      </Show>
    </div>
  )
}
