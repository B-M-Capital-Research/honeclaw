import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Input } from "@hone-financial/ui/input"
import { Show, For } from "solid-js"
import { usePortfolio } from "@/context/portfolio"
import { actorLabel } from "@/lib/actors"

export function PortfolioDetail() {
    const portfolio = usePortfolio()
    const data = () => portfolio.portfolioData()

    const isEditing = () => !!portfolio.state.editingSymbol
    const isNew = () => portfolio.state.editingSymbol === "new"

    const handleSubmit = async (e: Event) => {
        e.preventDefault()
        const draft = portfolio.state.draft
        await portfolio.saveHolding({
            symbol: draft.symbol,
            shares: Number(draft.shares),
            avg_cost: Number(draft.avg_cost),
            holding_horizon: draft.holding_horizon || "",
            strategy_notes: draft.strategy_notes,
            notes: draft.notes,
        })
    }

    const formatMoney = (val: number) => {
        return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(val)
    }

    const horizonLabel = (value?: string) => {
        if (value === "long_term") return "长持"
        if (value === "short_term") return "短持"
        return "未标记"
    }

    return (
        <Show
            when={portfolio.currentActor()}
            fallback={<EmptyState title="从左侧定位主体持仓" description="你可以查看特定渠道主体的投资概况，或者为其手动调整持仓记录。" />}
        >
            <div class="flex h-full min-h-0 flex-col bg-[color:var(--surface)]">
                <div class="flex items-center justify-between border-b border-[color:var(--border)] px-6 py-4">
                    <div>
                        <div class="text-xl font-semibold">持仓概览</div>
                        <div class="mt-1 text-sm text-[color:var(--text-muted)]">
                            {actorLabel(portfolio.currentActor()!)} · {portfolio.currentActor()!.channel}
                        </div>
                    </div>
                    <Button
                        variant="primary"
                        class="h-9 px-4 text-sm"
                        onClick={() => {
                            portfolio.openForm()
                        }}
                        disabled={isEditing()}
                    >
                        添加新持仓
                    </Button>
                </div>

                <div class="min-h-0 flex-1 flex flex-col md:flex-row overflow-hidden">
                    {/* Main Table View */}
                    <div class="min-h-0 flex-1 overflow-y-auto hf-scrollbar p-6">
                        <Show
                            when={data()?.portfolio && data()?.portfolio?.holdings.length !== undefined && data()!.portfolio!.holdings.length > 0}
                            fallback={<EmptyState title="暂无持仓数据" description="该用户当前没有任何资产记录。" />}
                        >
                            <table class="w-full border-collapse text-left text-sm">
                                <thead>
                                    <tr class="border-b border-[color:var(--border)]">
                                        <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)]">标的</th>
                                        <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)] text-right">数量</th>
                                        <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)] text-right">平均成本</th>
                                        <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)] text-right">总成本基准</th>
                                        <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)] text-center">操作</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <For each={data()?.portfolio?.holdings || []}>
                                        {(holding) => (
                                            <tr class="border-b border-[color:var(--border)] hover:bg-black/5 transition-colors">
                                                <td class="py-3 px-4">
                                                    <div class="font-medium uppercase">{holding.symbol}</div>
                                                    <div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-[color:var(--text-muted)]">
                                                        <span class="rounded-full border border-[color:var(--border)] px-2 py-0.5">
                                                            {horizonLabel(holding.holding_horizon)}
                                                        </span>
                                                        <Show when={holding.strategy_notes}>
                                                            <span>策略：{holding.strategy_notes}</span>
                                                        </Show>
                                                        <Show when={holding.notes}>
                                                            <span>备注：{holding.notes}</span>
                                                        </Show>
                                                    </div>
                                                </td>
                                                <td class="py-3 px-4 text-right">{holding.shares}</td>
                                                <td class="py-3 px-4 text-right">{formatMoney(holding.avg_cost)}</td>
                                                <td class="py-3 px-4 text-right font-medium">{formatMoney(holding.shares * holding.avg_cost)}</td>
                                                <td class="py-3 px-4 text-center">
                                                    <button
                                                        class="text-[color:var(--accent)] hover:underline text-xs mr-3"
                                                        onClick={() => portfolio.openForm(holding.symbol)}
                                                    >
                                                        编辑
                                                    </button>
                                                    <button
                                                        class="text-rose-500 hover:underline text-xs"
                                                        onClick={async () => {
                                                            if (confirm(`确定删除 ${holding.symbol} 配置吗？`)) {
                                                                await portfolio.removeHolding(holding.symbol)
                                                            }
                                                        }}
                                                    >
                                                        删除
                                                    </button>
                                                </td>
                                            </tr>
                                        )}
                                    </For>
                                </tbody>
                            </table>

                            <div class="mt-8 rounded-lg bg-[color:var(--panel-strong)] p-4 border border-[color:var(--border)]">
                                <div class="text-sm font-semibold mb-2">组合统计概况</div>
                                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                                    <div>
                                        <div class="text-xs text-[color:var(--text-muted)]">持有标的数</div>
                                        <div class="text-lg font-medium">{data()?.summary.holdings_count}</div>
                                    </div>
                                    <div>
                                        <div class="text-xs text-[color:var(--text-muted)]">总数量</div>
                                        <div class="text-lg font-medium">{data()?.summary.total_shares}</div>
                                    </div>
                                    <div>
                                        <div class="text-xs text-[color:var(--text-muted)]">上次更新时间</div>
                                        <div class="text-sm font-medium mt-1 truncate">
                                            {data()?.summary.updated_at ? new Date(data()!.summary.updated_at!).toLocaleString() : "未知"}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </Show>
                    </div>

                    {/* Edit Form Panel */}
                    <Show when={isEditing()}>
                        <div class="w-full md:w-[320px] md:border-l border-t md:border-t-0 border-[color:var(--border)] bg-[color:var(--panel)] p-6 shrink-0 hf-scrollbar overflow-y-auto">
                            <div class="flex items-center justify-between mb-6">
                                <div class="font-semibold">{isNew() ? "新增持仓" : `编辑 ${portfolio.state.draft.symbol}`}</div>
                                <button
                                    onClick={() => portfolio.closeForm()}
                                    class="text-xs text-[color:var(--text-muted)] hover:text-black"
                                >
                                    取消
                                </button>
                            </div>

                            <form class="space-y-4" onSubmit={handleSubmit}>
                                <div class="space-y-2">
                                    <label class="text-xs font-medium">股票代码 (Symbol)</label>
                                    <Input
                                        required
                                        disabled={!isNew()}
                                        class="h-9 uppercase"
                                        value={portfolio.state.draft.symbol || ""}
                                        onInput={(e) => portfolio.setDraft("symbol", e.currentTarget.value.toUpperCase())}
                                        placeholder="AAPL"
                                    />
                                </div>
                                <div class="space-y-2">
                                    <label class="text-xs font-medium">持有数量 (Shares)</label>
                                    <Input
                                        required
                                        type="number"
                                        step="0.0001"
                                        min="0"
                                        class="h-9"
                                        value={portfolio.state.draft.shares ?? ""}
                                        onInput={(e) => portfolio.setDraft("shares", parseFloat(e.currentTarget.value))}
                                        placeholder="10.5"
                                    />
                                </div>
                                <div class="space-y-2">
                                    <label class="text-xs font-medium">平均成本 (Avg Cost)</label>
                                    <Input
                                        required
                                        type="number"
                                        step="0.01"
                                        class="h-9"
                                        value={portfolio.state.draft.avg_cost ?? ""}
                                        onInput={(e) => portfolio.setDraft("avg_cost", parseFloat(e.currentTarget.value))}
                                        placeholder="-2.35 / 150.25"
                                    />
                                </div>
                                <div class="space-y-2">
                                    <label class="text-xs font-medium">持有期限倾向 (Horizon)</label>
                                    <select
                                        class="h-9 w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 text-sm"
                                        value={portfolio.state.draft.holding_horizon || ""}
                                        onChange={(e) =>
                                            portfolio.setDraft("holding_horizon", e.currentTarget.value as "long_term" | "short_term" | "")
                                        }
                                    >
                                        <option value="">未标记</option>
                                        <option value="long_term">长持</option>
                                        <option value="short_term">短持</option>
                                    </select>
                                </div>
                                <div class="space-y-2">
                                    <label class="text-xs font-medium">特殊策略 (Strategy)</label>
                                    <Input
                                        class="h-9"
                                        value={portfolio.state.draft.strategy_notes || ""}
                                        onInput={(e) => portfolio.setDraft("strategy_notes", e.currentTarget.value)}
                                        placeholder="例如：现金担保卖沽 / 财报事件驱动 / 核心长期仓位"
                                    />
                                </div>
                                <div class="space-y-2">
                                    <label class="text-xs font-medium">备注 (Notes)</label>
                                    <Input
                                        class="h-9"
                                        value={portfolio.state.draft.notes || ""}
                                        onInput={(e) => portfolio.setDraft("notes", e.currentTarget.value)}
                                        placeholder="选填"
                                    />
                                </div>

                                <div class="pt-4">
                                    <Button type="submit" class="w-full" disabled={portfolio.state.submitting}>
                                        {portfolio.state.submitting ? "保存中..." : "保存"}
                                    </Button>
                                </div>
                            </form>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    )
}
