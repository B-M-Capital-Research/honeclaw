import { Button } from "@hone-financial/ui/button"
import { useNavigate } from "@solidjs/router"
import { Portal } from "solid-js/web"
import { For, Show, createMemo, createSignal } from "solid-js"
import { useCompanyProfiles } from "@/context/company-profiles"
import { usePortfolio } from "@/context/portfolio"
import { useResearch } from "@/context/research"
import { useSessions } from "@/context/sessions"
import { useSymbolDrawer } from "@/context/symbol-drawer"
import { actorKey, type ActorRef } from "@/lib/actors"

type DrawerTab = "profile" | "research" | "sessions" | "actions"

const TABS: { id: DrawerTab; label: string }[] = [
  { id: "profile", label: "公司画像" },
  { id: "research", label: "研究记录" },
  { id: "sessions", label: "相关会话" },
  { id: "actions", label: "操作" },
]

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

function ProfileTab(props: { symbol: string; actor?: ActorRef }) {
  const navigate = useNavigate()
  const profiles = useCompanyProfiles()

  const matched = createMemo(() => {
    const sym = props.symbol.toUpperCase()
    return (profiles.profiles() ?? []).filter((p) =>
      p.title.toUpperCase().includes(sym),
    )
  })

  return (
    <Show
      when={props.actor}
      fallback={
        <div class="rounded-md border border-dashed border-[color:var(--border)] p-6 text-center text-sm text-[color:var(--text-muted)]">
          先在用户档案页选定一个用户,这里才能列出该用户的相关画像。
        </div>
      }
    >
      {(actor) => (
        <Show
          when={matched().length > 0}
          fallback={
            <div class="rounded-md border border-dashed border-[color:var(--border)] p-6 text-center text-sm text-[color:var(--text-muted)]">
              该用户的画像里没有标题匹配 {props.symbol} 的公司。
            </div>
          }
        >
          <div class="space-y-2">
            <For each={matched()}>
              {(p) => (
                <button
                  type="button"
                  class="block w-full rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-3 text-left transition hover:border-[color:var(--accent)] hover:bg-[color:var(--accent-soft)]"
                  onClick={() => {
                    const k = encodeURIComponent(actorKey(actor()))
                    navigate(`/users/${k}/profiles?profile=${encodeURIComponent(p.profile_id)}`)
                  }}
                >
                  <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                    {p.title}
                  </div>
                  <div class="mt-1 text-[11px] text-[color:var(--text-muted)]">
                    {formatDate(p.updated_at)} · {p.event_count} 条事件
                  </div>
                </button>
              )}
            </For>
          </div>
        </Show>
      )}
    </Show>
  )
}

function ResearchTab(props: { symbol: string }) {
  const navigate = useNavigate()
  const research = useResearch()

  const matched = createMemo(() => {
    const sym = props.symbol.toUpperCase()
    return research.state.tasks.filter((t) =>
      t.company_name.toUpperCase().includes(sym),
    )
  })

  return (
    <Show
      when={matched().length > 0}
      fallback={
        <div class="rounded-md border border-dashed border-[color:var(--border)] p-6 text-center text-sm text-[color:var(--text-muted)]">
          没有 company_name 匹配 {props.symbol} 的研究任务。可在"操作"tab 启动一个。
        </div>
      }
    >
      <div class="space-y-2">
        <For each={matched()}>
          {(task) => (
            <button
              type="button"
              class="block w-full rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-3 text-left transition hover:border-[color:var(--accent)]"
              onClick={() => navigate(`/research/${encodeURIComponent(task.task_id)}`)}
            >
              <div class="flex items-center justify-between gap-2">
                <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                  {task.company_name}
                </div>
                <div class="text-[11px] text-[color:var(--text-muted)]">
                  {task.status} · {task.progress}
                </div>
              </div>
              <div class="mt-1 text-[11px] text-[color:var(--text-muted)]">
                创建于 {formatDate(task.created_at)}
              </div>
            </button>
          )}
        </For>
      </div>
    </Show>
  )
}

function SessionsTab(props: { actor?: ActorRef }) {
  const navigate = useNavigate()
  const sessions = useSessions()

  const userSessions = createMemo(() => {
    const a = props.actor
    if (!a) return []
    return sessions.state.users
      .filter((u) => {
        return (
          u.channel === a.channel &&
          u.user_id === a.user_id &&
          (u.channel_scope ?? "") === (a.channel_scope ?? "")
        )
      })
      .slice()
      .sort((x, y) => (y.last_time ?? "").localeCompare(x.last_time ?? ""))
      .slice(0, 6)
  })

  return (
    <Show
      when={props.actor}
      fallback={
        <div class="rounded-md border border-dashed border-[color:var(--border)] p-6 text-center text-sm text-[color:var(--text-muted)]">
          先选定用户。
        </div>
      }
    >
      {(actor) => (
        <Show
          when={userSessions().length > 0}
          fallback={
            <div class="rounded-md border border-dashed border-[color:var(--border)] p-6 text-center text-sm text-[color:var(--text-muted)]">
              该用户暂无会话。
            </div>
          }
        >
          <div class="mb-3 text-[11px] text-[color:var(--text-muted)]">
            后端尚无消息全文搜索 — 这里列出该用户最近 6 个会话,可点入查找上下文。
          </div>
          <div class="space-y-2">
            <For each={userSessions()}>
              {(u) => (
                <button
                  type="button"
                  class="block w-full rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-3 text-left transition hover:border-[color:var(--accent)]"
                  onClick={() =>
                    navigate(`/sessions/${encodeURIComponent(u.session_id)}`)
                  }
                >
                  <div class="flex items-center justify-between gap-2">
                    <div class="text-sm font-medium text-[color:var(--text-primary)]">
                      {u.session_label || u.user_id}
                    </div>
                    <div class="text-[11px] text-[color:var(--text-muted)]">
                      {formatDate(u.last_time)}
                    </div>
                  </div>
                  <div class="mt-1 line-clamp-2 text-xs text-[color:var(--text-secondary)]">
                    {u.last_message || "(空)"}
                  </div>
                </button>
              )}
            </For>
            <button
              type="button"
              class="block w-full rounded-md border border-dashed border-[color:var(--border)] p-2 text-center text-xs text-[color:var(--text-muted)] transition hover:border-[color:var(--accent)] hover:text-[color:var(--text-primary)]"
              onClick={() => {
                const k = encodeURIComponent(actorKey(actor()))
                navigate(`/users/${k}/sessions`)
              }}
            >
              查看该用户的所有会话 →
            </button>
          </div>
        </Show>
      )}
    </Show>
  )
}

function ActionsTab(props: { symbol: string; actor?: ActorRef; onDone: () => void }) {
  const navigate = useNavigate()
  const portfolio = usePortfolio()
  const [adding, setAdding] = createSignal(false)
  const [feedback, setFeedback] = createSignal<string>("")

  const inWatchlist = createMemo(() => {
    const sym = props.symbol.toUpperCase()
    return portfolio.watchlist().some((h) => h.symbol.toUpperCase() === sym)
  })
  const inHolding = createMemo(() => {
    const sym = props.symbol.toUpperCase()
    return portfolio.holdingsList().some((h) => h.symbol.toUpperCase() === sym)
  })

  const handleAddWatchlist = async () => {
    if (!props.actor) {
      setFeedback("先选定用户才能加 watchlist")
      return
    }
    setAdding(true)
    try {
      await portfolio.saveHolding({
        symbol: props.symbol,
        shares: 0,
        avg_cost: 0,
        holding_horizon: "",
        strategy_notes: "",
        notes: "",
        tracking_only: true,
      })
      setFeedback(`已加入 ${props.actor.user_id} 的 watchlist`)
    } catch (err) {
      setFeedback(err instanceof Error ? err.message : String(err))
    } finally {
      setAdding(false)
    }
  }

  const handleStartResearch = () => {
    navigate(`/research?symbol=${encodeURIComponent(props.symbol)}`)
    props.onDone()
  }

  return (
    <div class="space-y-4">
      <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
        <div class="text-sm font-semibold mb-2">加到 watchlist</div>
        <div class="text-[11px] text-[color:var(--text-muted)] mb-3">
          <Show when={props.actor} fallback={<span>当前未选定用户。</span>}>
            {(a) => <span>目标用户: {a().user_id} · {a().channel}</span>}
          </Show>
        </div>
        <Show when={!inWatchlist() && !inHolding()} fallback={
          <div class="rounded-md border border-emerald-300/40 bg-emerald-500/10 p-3 text-xs text-emerald-300">
            {inHolding() ? `${props.symbol} 已在该用户持仓里` : `${props.symbol} 已在该用户 watchlist 里`}
          </div>
        }>
          <Button
            class="h-9 w-full text-sm"
            onClick={() => void handleAddWatchlist()}
            disabled={adding() || !props.actor}
          >
            {adding() ? "添加中…" : `加 ${props.symbol} 到 watchlist`}
          </Button>
        </Show>
      </div>

      <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--panel)] p-4">
        <div class="text-sm font-semibold mb-2">启动深度研究</div>
        <div class="text-[11px] text-[color:var(--text-muted)] mb-3">
          预填 {props.symbol} 跳到个股研究模块,你需要确认才会真正启动。
        </div>
        <Button
          variant="ghost"
          class="h-9 w-full text-sm"
          onClick={handleStartResearch}
        >
          去启动 {props.symbol} 研究 →
        </Button>
      </div>

      <Show when={feedback()}>
        <div class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] p-3 text-xs text-[color:var(--text-secondary)]">
          {feedback()}
        </div>
      </Show>
    </div>
  )
}

export function SymbolDrawer() {
  const drawer = useSymbolDrawer()
  const portfolio = usePortfolio()
  const [tab, setTab] = createSignal<DrawerTab>("profile")

  const sourceActor = () => portfolio.currentActor()

  return (
    <Show when={drawer.isOpen()}>
      <Portal>
        {/* 遮罩 — z-30,允许 toast(z-50)和 modal(z-50)在抽屉之上 */}
        <div
          class="fixed inset-0 z-30 bg-black/30 backdrop-blur-[1px]"
          onClick={() => drawer.close()}
        />
        {/* 抽屉本体 — z-40 */}
        <aside
          class="fixed right-0 top-0 z-40 flex h-full w-[420px] flex-col border-l border-[color:var(--border)] bg-[color:var(--surface)] shadow-2xl"
          onClick={(e) => e.stopPropagation()}
        >
          <div class="flex shrink-0 items-center justify-between border-b border-[color:var(--border)] px-5 py-3">
            <div>
              <div class="text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">
                Symbol
              </div>
              <div class="font-mono text-lg font-semibold text-[color:var(--text-primary)]">
                {drawer.symbol() ?? ""}
              </div>
            </div>
            <button
              type="button"
              class="rounded-md p-1.5 text-[color:var(--text-muted)] hover:bg-black/5 hover:text-[color:var(--text-primary)]"
              onClick={() => drawer.close()}
              aria-label="关闭"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div class="flex shrink-0 border-b border-[color:var(--border)] px-2">
            <For each={TABS}>
              {(t) => (
                <button
                  type="button"
                  class={[
                    "px-3 py-2 text-xs font-medium transition border-b-2 -mb-px",
                    tab() === t.id
                      ? "border-[color:var(--accent)] text-[color:var(--text-primary)]"
                      : "border-transparent text-[color:var(--text-muted)] hover:text-[color:var(--text-primary)]",
                  ].join(" ")}
                  onClick={() => setTab(t.id)}
                >
                  {t.label}
                </button>
              )}
            </For>
          </div>

          <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto p-4">
            <Show when={drawer.symbol()}>
              {(sym) => (
                <>
                  <Show when={tab() === "profile"}>
                    <ProfileTab symbol={sym()} actor={sourceActor()} />
                  </Show>
                  <Show when={tab() === "research"}>
                    <ResearchTab symbol={sym()} />
                  </Show>
                  <Show when={tab() === "sessions"}>
                    <SessionsTab actor={sourceActor()} />
                  </Show>
                  <Show when={tab() === "actions"}>
                    <ActionsTab
                      symbol={sym()}
                      actor={sourceActor()}
                      onDone={() => drawer.close()}
                    />
                  </Show>
                </>
              )}
            </Show>
          </div>
        </aside>
      </Portal>
    </Show>
  )
}
