import { EmptyState } from "@hone-financial/ui/empty-state"
import { Skeleton } from "@hone-financial/ui/skeleton"
import { For, Show, createSignal } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { usePortfolio } from "@/context/portfolio"
import { Input } from "@hone-financial/ui/input"
import { Button } from "@hone-financial/ui/button"
import { actorKey, actorLabel, type ActorRef } from "@/lib/actors"
import type { PortfolioSummary } from "@/lib/types"

function actorFromSummary(s: PortfolioSummary): ActorRef {
    return {
        channel: s.channel,
        user_id: s.user_id,
        channel_scope: s.channel_scope,
    }
}

export function PortfolioList() {
    const navigate = useNavigate()
    const portfolio = usePortfolio()
    const [showManual, setShowManual] = createSignal(false)
    const [draft, setDraft] = createSignal<ActorRef>({
        channel: "imessage",
        user_id: "",
        channel_scope: "",
    })

    const openActor = (actor: ActorRef) => {
        portfolio.selectActor(actor)
        navigate(`/portfolio/${encodeURIComponent(actorKey(actor))}`)
    }

    const currentKey = () => portfolio.state.currentActorKey

    return (
        <div class="flex h-full min-h-0 w-[300px] flex-col border-r border-[color:var(--border)] bg-[color:var(--panel)]">
            <div class="border-b border-[color:var(--border)] px-4 py-3">
                <div class="flex items-center justify-between">
                    <div>
                        <div class="text-sm font-semibold tracking-tight">持仓管理</div>
                        <div class="text-xs text-[color:var(--text-muted)]">从 IM 渠道自动同步</div>
                    </div>
                    <button
                        type="button"
                        class="rounded-md border border-[color:var(--border)] px-2 py-1 text-[11px] text-[color:var(--text-secondary)] transition hover:border-[color:var(--accent)] hover:text-[color:var(--text-primary)] bg-[color:var(--surface)]"
                        onClick={() => setShowManual((v) => !v)}
                    >
                        {showManual() ? "收起" : "手动输入"}
                    </button>
                </div>

                <Show when={showManual()}>
                    <div class="mt-3 space-y-2">
                        <Input
                            class="h-8 text-xs bg-[color:var(--surface)]"
                            value={draft().channel}
                            onInput={(e) => setDraft((prev) => ({ ...prev, channel: e.currentTarget.value.trim() }))}
                            placeholder="渠道，如 imessage"
                        />
                        <Input
                            class="h-8 text-xs bg-[color:var(--surface)]"
                            value={draft().user_id}
                            onInput={(e) => setDraft((prev) => ({ ...prev, user_id: e.currentTarget.value.trim() }))}
                            placeholder="用户 ID"
                        />
                        <Input
                            class="h-8 text-xs bg-[color:var(--surface)]"
                            value={draft().channel_scope ?? ""}
                            onInput={(e) => setDraft((prev) => ({ ...prev, channel_scope: e.currentTarget.value.trim() }))}
                            placeholder="范围，可选"
                        />
                        <Button
                            class="h-8 w-full text-xs"
                            onClick={() => {
                                const actor = draft()
                                if (!actor.channel || !actor.user_id) return
                                openActor({
                                    channel: actor.channel,
                                    user_id: actor.user_id,
                                    channel_scope: actor.channel_scope || undefined,
                                })
                                setShowManual(false)
                            }}
                        >
                            打开
                        </Button>
                    </div>
                </Show>
            </div>

            <div class="hf-scrollbar min-h-0 flex-1 overflow-y-auto px-3 py-3">
                <Show
                    when={!portfolio.actorsList.loading}
                    fallback={
                        <div class="space-y-3 px-2 py-2">
                            <Skeleton class="h-16" />
                            <Skeleton class="h-16" />
                        </div>
                    }
                >
                    <Show
                        when={(portfolio.actorsList() ?? []).length > 0}
                        fallback={
                            <EmptyState
                                title="暂无持仓数据"
                                description="暂无持仓数据，可通过手动输入或 IM 渠道添加。"
                            />
                        }
                    >
                        <div class="space-y-2">
                            <For each={portfolio.actorsList() ?? []}>
                                {(summary) => {
                                    const actor = actorFromSummary(summary)
                                    const key = actorKey(actor)
                                    const isActive = () => currentKey() === key
                                    return (
                                        <button
                                            type="button"
                                            class={[
                                                "w-full rounded-md border p-3 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]",
                                                isActive()
                                                    ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                                                    : "border-[color:var(--border)] bg-[color:var(--surface)] hover:border-[color:var(--accent)]/50",
                                            ].join(" ")}
                                            onClick={() => openActor(actor)}
                                        >
                                            <div class="flex items-start gap-3">
                                                <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-[color:var(--panel-strong)] text-xs font-semibold text-[color:var(--text-secondary)]">
                                                    {summary.user_id.slice(0, 1).toUpperCase()}
                                                </div>
                                                <div class="min-w-0 flex-1">
                                                    <div class="truncate text-sm font-medium text-[color:var(--text-primary)]">
                                                        {actorLabel(actor)}
                                                    </div>
                                                    <div class="mt-0.5 text-[11px] text-[color:var(--text-muted)]">
                                                        {summary.channel}
                                                    </div>
                                                    <div class="mt-1.5 text-[11px] text-[color:var(--text-muted)]">
                                                        {summary.holdings_count} 个标的 · 共 {summary.total_shares.toFixed(2)} 单位
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
