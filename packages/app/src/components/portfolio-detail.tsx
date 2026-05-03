import { Button } from "@hone-financial/ui/button"
import { EmptyState } from "@hone-financial/ui/empty-state"
import { Input } from "@hone-financial/ui/input"
import { Show, For } from "solid-js"
import { usePortfolio } from "@/context/portfolio"
import { actorLabel } from "@/lib/actors"
import type { HoldingInfo } from "@/lib/types"
import { SymbolLink } from "./symbol-link"
import { PORTFOLIO } from "@/lib/admin-content/portfolio"
import { tpl, useLocale } from "@/lib/i18n"

export function PortfolioDetail() {
    const portfolio = usePortfolio()
    const data = () => portfolio.portfolioData()

    const isEditing = () => !!portfolio.state.editingSymbol
    const isNew = () => portfolio.state.editingSymbol === "new"
    const isWatchDraft = () => !!portfolio.state.draft.tracking_only

    const handleSubmit = async (e: Event) => {
        e.preventDefault()
        const draft = portfolio.state.draft
        const tracking = !!draft.tracking_only
        await portfolio.saveHolding({
            symbol: draft.symbol,
            shares: tracking ? 0 : Number(draft.shares),
            avg_cost: tracking ? 0 : Number(draft.avg_cost),
            holding_horizon: draft.holding_horizon || "",
            strategy_notes: draft.strategy_notes,
            notes: draft.notes,
            tracking_only: tracking,
        })
    }

    const formatMoney = (val: number) => {
        const loc = useLocale() === "zh" ? "zh-CN" : "en-US"
        return new Intl.NumberFormat(loc, { style: 'currency', currency: 'USD' }).format(val)
    }

    const horizonLabel = (value?: string) => {
        if (value === "long_term") return PORTFOLIO.detail.horizon_long
        if (value === "short_term") return PORTFOLIO.detail.horizon_short
        return PORTFOLIO.detail.horizon_unmarked
    }

    const renderActions = (holding: HoldingInfo) => (
        <>
            <button
                class="text-[color:var(--accent)] hover:underline text-xs mr-3"
                onClick={() => portfolio.openForm(holding.symbol)}
            >
                {PORTFOLIO.detail.edit_button}
            </button>
            <button
                class="text-rose-500 hover:underline text-xs"
                onClick={async () => {
                    const label = holding.tracking_only
                        ? PORTFOLIO.detail.delete_label_watch
                        : PORTFOLIO.detail.delete_label_holding
                    if (confirm(tpl(PORTFOLIO.detail.delete_confirm, { symbol: holding.symbol, label }))) {
                        await portfolio.removeHolding(holding.symbol)
                    }
                }}
            >
                {PORTFOLIO.detail.delete_button}
            </button>
        </>
    )

    const totalRecords = () =>
        (portfolio.holdingsList()?.length || 0) + (portfolio.watchlist()?.length || 0)

    return (
        <Show
            when={portfolio.currentActor()}
            fallback={<EmptyState title={PORTFOLIO.detail.empty_title} description={PORTFOLIO.detail.empty_description} />}
        >
            <div class="flex h-full min-h-0 flex-col bg-[color:var(--surface)]">
                <div class="flex items-center justify-between border-b border-[color:var(--border)] px-6 py-4">
                    <div>
                        <div class="text-xl font-semibold">{PORTFOLIO.detail.header_title}</div>
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
                        {PORTFOLIO.detail.add_button}
                    </Button>
                </div>

                <div class="min-h-0 flex-1 flex flex-col md:flex-row overflow-hidden">
                    {/* Main Table View */}
                    <div class="min-h-0 flex-1 overflow-y-auto hf-scrollbar p-6">
                        <Show
                            when={totalRecords() > 0}
                            fallback={<EmptyState title={PORTFOLIO.detail.empty_records_title} description={PORTFOLIO.detail.empty_records_description} />}
                        >
                            <Show when={portfolio.holdingsList().length > 0}>
                                <div class="mb-3 text-sm font-semibold">{PORTFOLIO.detail.section_holdings}</div>
                                <table class="w-full border-collapse text-left text-sm mb-8">
                                    <thead>
                                        <tr class="border-b border-[color:var(--border)]">
                                            <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)]">{PORTFOLIO.detail.col_symbol}</th>
                                            <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)] text-right">{PORTFOLIO.detail.col_shares}</th>
                                            <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)] text-right">{PORTFOLIO.detail.col_avg_cost}</th>
                                            <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)] text-right">{PORTFOLIO.detail.col_total_cost}</th>
                                            <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)] text-center">{PORTFOLIO.detail.col_actions}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For each={portfolio.holdingsList()}>
                                            {(holding) => (
                                                <tr class="border-b border-[color:var(--border)] hover:bg-black/5 transition-colors">
                                                    <td class="py-3 px-4">
                                                        <SymbolLink symbol={holding.symbol} />
                                                        <div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-[color:var(--text-muted)]">
                                                            <span class="rounded-full border border-[color:var(--border)] px-2 py-0.5">
                                                                {horizonLabel(holding.holding_horizon)}
                                                            </span>
                                                            <Show when={holding.strategy_notes}>
                                                                <span>{PORTFOLIO.detail.strategy_label}{holding.strategy_notes}</span>
                                                            </Show>
                                                            <Show when={holding.notes}>
                                                                <span>{PORTFOLIO.detail.notes_label}{holding.notes}</span>
                                                            </Show>
                                                        </div>
                                                    </td>
                                                    <td class="py-3 px-4 text-right">{holding.shares}</td>
                                                    <td class="py-3 px-4 text-right">{formatMoney(holding.avg_cost)}</td>
                                                    <td class="py-3 px-4 text-right font-medium">{formatMoney(holding.shares * holding.avg_cost)}</td>
                                                    <td class="py-3 px-4 text-center">{renderActions(holding)}</td>
                                                </tr>
                                            )}
                                        </For>
                                    </tbody>
                                </table>
                            </Show>

                            <Show when={portfolio.watchlist().length > 0}>
                                <div class="mb-3 text-sm font-semibold flex items-center gap-2">
                                    {PORTFOLIO.detail.section_watchlist}
                                    <span class="rounded-full bg-[color:var(--panel-strong)] border border-[color:var(--border)] px-2 py-0.5 text-xs font-normal text-[color:var(--text-muted)]">
                                        {PORTFOLIO.detail.watchlist_badge}
                                    </span>
                                </div>
                                <table class="w-full border-collapse text-left text-sm mb-8">
                                    <thead>
                                        <tr class="border-b border-[color:var(--border)]">
                                            <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)]">{PORTFOLIO.detail.col_symbol}</th>
                                            <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)]">{PORTFOLIO.detail.col_notes}</th>
                                            <th class="py-3 px-4 font-semibold text-[color:var(--text-secondary)] text-center">{PORTFOLIO.detail.col_actions}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For each={portfolio.watchlist()}>
                                            {(holding) => (
                                                <tr class="border-b border-[color:var(--border)] hover:bg-black/5 transition-colors">
                                                    <td class="py-3 px-4">
                                                        <div class="flex items-center gap-2">
                                                            <SymbolLink symbol={holding.symbol} />
                                                            <span class="rounded-full border border-amber-400 bg-amber-50 px-2 py-0.5 text-[10px] font-normal text-amber-700">
                                                                {PORTFOLIO.detail.watch_chip}
                                                            </span>
                                                        </div>
                                                    </td>
                                                    <td class="py-3 px-4 text-xs text-[color:var(--text-muted)]">
                                                        <Show when={holding.strategy_notes}>
                                                            <span class="mr-3">{PORTFOLIO.detail.strategy_label}{holding.strategy_notes}</span>
                                                        </Show>
                                                        <Show when={holding.notes}>
                                                            <span>{PORTFOLIO.detail.notes_label}{holding.notes}</span>
                                                        </Show>
                                                    </td>
                                                    <td class="py-3 px-4 text-center">{renderActions(holding)}</td>
                                                </tr>
                                            )}
                                        </For>
                                    </tbody>
                                </table>
                            </Show>

                            <div class="mt-2 rounded-lg bg-[color:var(--panel-strong)] p-4 border border-[color:var(--border)]">
                                <div class="text-sm font-semibold mb-2">{PORTFOLIO.detail.summary_title}</div>
                                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                                    <div>
                                        <div class="text-xs text-[color:var(--text-muted)]">{PORTFOLIO.detail.summary_holdings_count}</div>
                                        <div class="text-lg font-medium">{data()?.summary.holdings_count ?? 0}</div>
                                    </div>
                                    <div>
                                        <div class="text-xs text-[color:var(--text-muted)]">{PORTFOLIO.detail.summary_watchlist_count}</div>
                                        <div class="text-lg font-medium">{data()?.summary.watchlist_count ?? 0}</div>
                                    </div>
                                    <div>
                                        <div class="text-xs text-[color:var(--text-muted)]">{PORTFOLIO.detail.summary_total_shares}</div>
                                        <div class="text-lg font-medium">{data()?.summary.total_shares ?? 0}</div>
                                    </div>
                                    <div>
                                        <div class="text-xs text-[color:var(--text-muted)]">{PORTFOLIO.detail.summary_updated_at}</div>
                                        <div class="text-sm font-medium mt-1 truncate">
                                            {data()?.summary.updated_at
                                                ? new Date(data()!.summary.updated_at!).toLocaleString(useLocale() === "zh" ? "zh-CN" : "en-US")
                                                : PORTFOLIO.detail.summary_unknown}
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
                                <div class="font-semibold">{isNew() ? PORTFOLIO.detail.form_title_new : tpl(PORTFOLIO.detail.form_title_edit, { symbol: portfolio.state.draft.symbol ?? "" })}</div>
                                <button
                                    onClick={() => portfolio.closeForm()}
                                    class="text-xs text-[color:var(--text-muted)] hover:text-black"
                                >
                                    {PORTFOLIO.detail.form_cancel}
                                </button>
                            </div>

                            <form class="space-y-4" onSubmit={handleSubmit}>
                                <label class="flex items-center gap-2 rounded-md border border-[color:var(--border)] bg-[color:var(--panel-strong)] px-3 py-2 text-xs">
                                    <input
                                        type="checkbox"
                                        checked={isWatchDraft()}
                                        onChange={(e) => portfolio.setDraft("tracking_only", e.currentTarget.checked)}
                                    />
                                    <span>{PORTFOLIO.detail.form_watch_only}</span>
                                </label>

                                <div class="space-y-2">
                                    <label class="text-xs font-medium">{PORTFOLIO.detail.field_symbol}</label>
                                    <Input
                                        required
                                        disabled={!isNew()}
                                        class="h-9 uppercase"
                                        value={portfolio.state.draft.symbol || ""}
                                        onInput={(e) => portfolio.setDraft("symbol", e.currentTarget.value.toUpperCase())}
                                        placeholder={PORTFOLIO.detail.field_symbol_placeholder}
                                    />
                                </div>
                                <Show when={!isWatchDraft()}>
                                    <div class="space-y-2">
                                        <label class="text-xs font-medium">{PORTFOLIO.detail.field_shares}</label>
                                        <Input
                                            required
                                            type="number"
                                            step="0.0001"
                                            min="0"
                                            class="h-9"
                                            value={portfolio.state.draft.shares ?? ""}
                                            onInput={(e) => portfolio.setDraft("shares", parseFloat(e.currentTarget.value))}
                                            placeholder={PORTFOLIO.detail.field_shares_placeholder}
                                        />
                                    </div>
                                    <div class="space-y-2">
                                        <label class="text-xs font-medium">{PORTFOLIO.detail.field_avg_cost}</label>
                                        <Input
                                            required
                                            type="number"
                                            step="0.01"
                                            class="h-9"
                                            value={portfolio.state.draft.avg_cost ?? ""}
                                            onInput={(e) => portfolio.setDraft("avg_cost", parseFloat(e.currentTarget.value))}
                                            placeholder={PORTFOLIO.detail.field_avg_cost_placeholder}
                                        />
                                    </div>
                                </Show>
                                <div class="space-y-2">
                                    <label class="text-xs font-medium">{PORTFOLIO.detail.field_horizon}</label>
                                    <select
                                        class="h-9 w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 text-sm"
                                        value={portfolio.state.draft.holding_horizon || ""}
                                        onChange={(e) =>
                                            portfolio.setDraft("holding_horizon", e.currentTarget.value as "long_term" | "short_term" | "")
                                        }
                                    >
                                        <option value="">{PORTFOLIO.detail.horizon_unmarked}</option>
                                        <option value="long_term">{PORTFOLIO.detail.horizon_long}</option>
                                        <option value="short_term">{PORTFOLIO.detail.horizon_short}</option>
                                    </select>
                                </div>
                                <div class="space-y-2">
                                    <label class="text-xs font-medium">{PORTFOLIO.detail.field_strategy}</label>
                                    <Input
                                        class="h-9"
                                        value={portfolio.state.draft.strategy_notes || ""}
                                        onInput={(e) => portfolio.setDraft("strategy_notes", e.currentTarget.value)}
                                        placeholder={PORTFOLIO.detail.field_strategy_placeholder}
                                    />
                                </div>
                                <div class="space-y-2">
                                    <label class="text-xs font-medium">{PORTFOLIO.detail.field_notes}</label>
                                    <Input
                                        class="h-9"
                                        value={portfolio.state.draft.notes || ""}
                                        onInput={(e) => portfolio.setDraft("notes", e.currentTarget.value)}
                                        placeholder={PORTFOLIO.detail.field_notes_placeholder}
                                    />
                                </div>

                                <div class="pt-4">
                                    <Button type="submit" class="w-full" disabled={portfolio.state.submitting}>
                                        {portfolio.state.submitting ? PORTFOLIO.detail.saving_button : PORTFOLIO.detail.save_button}
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
