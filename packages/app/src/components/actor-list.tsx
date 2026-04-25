import { EmptyState } from "@hone-financial/ui/empty-state"
import { Skeleton } from "@hone-financial/ui/skeleton"
import { For, Show, createMemo, createSignal } from "solid-js"
import { Input } from "@hone-financial/ui/input"
import { Button } from "@hone-financial/ui/button"
import { useCompanyProfiles } from "@/context/company-profiles"
import { usePortfolio } from "@/context/portfolio"
import { useSessions } from "@/context/sessions"
import {
  actorKey,
  actorLabel,
  mergeActorSummaries,
  type ActorListItem,
  type ActorRef,
} from "@/lib/actors"

type ActorListProps = {
  currentKey: string
  onSelect: (actor: ActorRef) => void
}

/**
 * 用户中心(/users)左栏:统一的 actor 列表,合并持仓/画像/会话三处来源。
 * 替代了原 portfolio-list 和 company-profile-list 的"各自一栏"模式。
 */
export function ActorList(props: ActorListProps) {
  const portfolio = usePortfolio()
  const companyProfiles = useCompanyProfiles()
  const sessions = useSessions()

  const [search, setSearch] = createSignal("")
  const [showManual, setShowManual] = createSignal(false)
  const [draft, setDraft] = createSignal<ActorRef>({
    channel: "imessage",
    user_id: "",
    channel_scope: "",
  })

  const merged = createMemo<ActorListItem[]>(() =>
    mergeActorSummaries({
      portfolios: portfolio.actorsList() ?? [],
      profiles: companyProfiles.actorsList() ?? [],
      sessions: sessions.state.users,
    }),
  )

  const filtered = createMemo(() => {
    const q = search().trim().toLowerCase()
    if (!q) return merged()
    return merged().filter((item) => {
      const haystack = [
        item.actor.user_id,
        item.actor.channel,
        item.actor.channel_scope ?? "",
        item.sessionLabel ?? "",
      ]
        .join(" ")
        .toLowerCase()
      return haystack.includes(q)
    })
  })

  const loading = () =>
    portfolio.actorsList.loading || companyProfiles.actorsList.loading

  const submitManual = () => {
    const a = draft()
    if (!a.channel || !a.user_id) return
    props.onSelect({
      channel: a.channel,
      user_id: a.user_id,
      channel_scope: a.channel_scope || undefined,
    })
    setShowManual(false)
  }

  return (
    <div class="flex h-full min-h-0 w-[300px] flex-col border-r border-[color:var(--border)] bg-[color:var(--panel)]">
      <div class="border-b border-[color:var(--border)] px-4 py-3">
        <div class="flex items-center justify-between">
          <div>
            <div class="text-sm font-semibold tracking-tight">用户档案</div>
            <div class="text-xs text-[color:var(--text-muted)]">
              合并持仓 / 画像 / 会话三处来源
            </div>
          </div>
          <button
            type="button"
            class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-1 text-[11px] text-[color:var(--text-secondary)] transition hover:border-[color:var(--accent)] hover:text-[color:var(--text-primary)]"
            onClick={() => setShowManual((v) => !v)}
          >
            {showManual() ? "收起" : "手动输入"}
          </button>
        </div>

        <Input
          class="mt-3 h-8 text-xs bg-[color:var(--surface)]"
          value={search()}
          onInput={(e) => setSearch(e.currentTarget.value)}
          placeholder="搜索 user_id / 渠道 / scope"
        />

        <Show when={showManual()}>
          <div class="mt-3 space-y-2">
            <Input
              class="h-8 text-xs bg-[color:var(--surface)]"
              value={draft().channel}
              onInput={(e) =>
                setDraft((prev) => ({ ...prev, channel: e.currentTarget.value.trim() }))
              }
              placeholder="渠道,如 imessage"
            />
            <Input
              class="h-8 text-xs bg-[color:var(--surface)]"
              value={draft().user_id}
              onInput={(e) =>
                setDraft((prev) => ({ ...prev, user_id: e.currentTarget.value.trim() }))
              }
              placeholder="用户 ID"
            />
            <Input
              class="h-8 text-xs bg-[color:var(--surface)]"
              value={draft().channel_scope ?? ""}
              onInput={(e) =>
                setDraft((prev) => ({
                  ...prev,
                  channel_scope: e.currentTarget.value.trim(),
                }))
              }
              placeholder="范围,可选"
            />
            <Button class="h-8 w-full text-xs" onClick={submitManual}>
              打开
            </Button>
          </div>
        </Show>
      </div>

      <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-3 py-3">
        <Show
          when={!loading()}
          fallback={
            <div class="space-y-3 px-2 py-2">
              <Skeleton class="h-16" />
              <Skeleton class="h-16" />
              <Skeleton class="h-16" />
            </div>
          }
        >
          <Show
            when={filtered().length > 0}
            fallback={
              <EmptyState
                title={search() ? "没有匹配的用户" : "暂无用户"}
                description={
                  search()
                    ? "试试切换搜索词,或用上方手动输入定位特定 actor。"
                    : "还没有人产生持仓 / 画像 / 会话,先让 IM 渠道接收消息或手动添加。"
                }
              />
            }
          >
            <div class="space-y-2">
              <For each={filtered()}>
                {(item) => {
                  const key = item.key
                  const isActive = () => props.currentKey === key
                  const stats = () => {
                    const parts: string[] = []
                    if (item.holdingsCount != null && item.holdingsCount > 0) {
                      parts.push(`${item.holdingsCount} 持仓`)
                    }
                    if (item.watchlistCount != null && item.watchlistCount > 0) {
                      parts.push(`${item.watchlistCount} 关注`)
                    }
                    if (item.profileCount != null && item.profileCount > 0) {
                      parts.push(`${item.profileCount} 画像`)
                    }
                    if (item.lastSessionTime) parts.push("会话")
                    return parts.length > 0 ? parts.join(" · ") : "暂无数据"
                  }
                  return (
                    <button
                      type="button"
                      class={[
                        "w-full rounded-md border p-3 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]",
                        isActive()
                          ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                          : "border-[color:var(--border)] bg-[color:var(--surface)] hover:border-[color:var(--accent)]/50",
                      ].join(" ")}
                      onClick={() => props.onSelect(item.actor)}
                    >
                      <div class="flex items-start gap-3">
                        <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-[color:var(--panel-strong)] text-xs font-semibold text-[color:var(--text-secondary)]">
                          {item.actor.user_id.slice(0, 1).toUpperCase()}
                        </div>
                        <div class="min-w-0 flex-1">
                          <div class="truncate text-sm font-medium text-[color:var(--text-primary)]">
                            {actorLabel(item.actor)}
                          </div>
                          <div class="mt-0.5 text-[11px] text-[color:var(--text-muted)]">
                            {item.actor.channel}
                          </div>
                          <div class="mt-1.5 text-[11px] text-[color:var(--text-muted)]">
                            {stats()}
                          </div>
                        </div>
                      </div>
                    </button>
                  )
                }}
              </For>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  )
}

export { actorKey }
