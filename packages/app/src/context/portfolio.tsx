import { createContext, createEffect, createMemo, createResource, useContext, type ParentProps } from "solid-js"
import { createStore } from "solid-js/store"
import { getPortfolio, createHolding, updateHolding, deleteHolding, listPortfolioActors } from "@/lib/api"
import type { HoldingUpsertInput, PortfolioSummary } from "@/lib/types"
import { readStoredSelection, writeStoredSelection } from "@/lib/persist"
import { actorKey, parseActorKey, type ActorRef } from "@/lib/actors"
import { useBackend } from "./backend"

type PortfolioContextValue = ReturnType<typeof createPortfolioState>

const PortfolioContext = createContext<PortfolioContextValue>()

function createPortfolioState() {
    const backend = useBackend()
    const storedSelection = readStoredSelection()
    const [state, setState] = createStore({
        currentActorKey: storedSelection.portfolioActorKey || "",
        loading: false,
        submitting: false,
        editingSymbol: undefined as string | undefined,
        draft: {} as Partial<HoldingUpsertInput>,
    })

    const currentActor = createMemo(() => parseActorKey(state.currentActorKey))

    const [portfolioData, { refetch }] = createResource(
        () => (backend.state.connected && backend.hasCapability("portfolio") ? currentActor() : undefined),
        (actor) => actor ? getPortfolio(actor) : undefined
    )

    const allHoldings = createMemo(() => portfolioData()?.portfolio?.holdings ?? [])
    const holdingsList = createMemo(() => allHoldings().filter(h => !h.tracking_only))
    const watchlist = createMemo(() => allHoldings().filter(h => !!h.tracking_only))

    // 加载所有已有持仓的 actor 列表（从 data/portfolio/ 目录）
    const [actorsList, { refetch: refetchActors }] = createResource(
        () => backend.state.connected && backend.hasCapability("portfolio"),
        async (connected) => {
            if (!connected) return [] as PortfolioSummary[]
            try {
                return await listPortfolioActors()
            } catch {
                return [] as PortfolioSummary[]
            }
        }
    )

    createEffect(() => {
        writeStoredSelection({
            ...readStoredSelection(),
            portfolioActorKey: state.currentActorKey,
        })
    })

    const selectActor = (actor?: ActorRef) => {
        setState("currentActorKey", actor ? actorKey(actor) : "")
    }

    const openForm = (symbol?: string) => {
        if (symbol) {
            setState("editingSymbol", symbol)
            const existing = portfolioData()?.portfolio?.holdings.find(h => h.symbol === symbol)
            if (existing) {
                setState("draft", {
                    symbol: existing.symbol,
                    shares: existing.shares,
                    avg_cost: existing.avg_cost,
                    holding_horizon: existing.holding_horizon || "",
                    strategy_notes: existing.strategy_notes || "",
                    notes: existing.notes || "",
                    tracking_only: !!existing.tracking_only,
                })
            }
        } else {
            setState("editingSymbol", "new")
            setState("draft", {
                symbol: "",
                shares: 0,
                avg_cost: 0,
                holding_horizon: "",
                strategy_notes: "",
                notes: "",
                tracking_only: false,
            })
        }
    }

    const closeForm = () => {
        setState("editingSymbol", undefined)
        setState("draft", {})
    }

    const saveHolding = async (input: HoldingUpsertInput) => {
        const actor = currentActor()
        if (!actor) return

        setState("submitting", true)
        try {
            const finalInput = {
                ...input,
                channel: actor.channel,
                user_id: actor.user_id,
                channel_scope: actor.channel_scope,
            }
            if (state.editingSymbol === "new") {
                await createHolding(finalInput)
            } else if (state.editingSymbol) {
                await updateHolding(state.editingSymbol, finalInput)
            }
            await refetch()
            closeForm()
        } finally {
            setState("submitting", false)
        }
    }

    const removeHolding = async (symbol: string) => {
        const actor = currentActor()
        if (!actor) return
        await deleteHolding(symbol, actor)
        await refetch()
    }

    return {
        state,
        currentActor,
        portfolioData,
        holdingsList,
        watchlist,
        refetch,
        actorsList,
        refetchActors,
        selectActor,
        openForm,
        closeForm,
        saveHolding,
        removeHolding,
        setDraft: (key: keyof HoldingUpsertInput, value: any) => {
            setState("draft", key as any, value)
        },
    }
}

export function PortfolioProvider(props: ParentProps) {
    const value = createPortfolioState()
    return <PortfolioContext.Provider value={value}>{props.children}</PortfolioContext.Provider>
}

export function usePortfolio() {
    const value = useContext(PortfolioContext)
    if (!value) throw new Error("PortfolioProvider missing")
    return value
}
