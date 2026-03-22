import { createContext, useContext, type ParentProps } from "solid-js"
import { createStore } from "solid-js/store"
import { getKbEntries, getKbEntry, deleteKbEntry, uploadKbFile } from "@/lib/api"
import type { KbEntry } from "@/lib/types"

// ── 类型 ─────────────────────────────────────────────────────────────────────

type KbState = {
  entries: KbEntry[]
  selectedId: string | null
  selectedEntry: KbEntry | null
  parsedText: string | null
  loading: boolean
  detailLoading: boolean
  error: string | null
}

type KbContextValue = ReturnType<typeof createKbState>

// ── Context ───────────────────────────────────────────────────────────────────

const KbContext = createContext<KbContextValue>()

// ── 状态工厂 ──────────────────────────────────────────────────────────────────

function createKbState() {
  const [state, setState] = createStore<KbState>({
    entries: [],
    selectedId: null,
    selectedEntry: null,
    parsedText: null,
    loading: false,
    detailLoading: false,
    error: null,
  })

  const loadEntries = async () => {
    setState("loading", true)
    setState("error", null)
    try {
      const entries = await getKbEntries()
      setState("entries", entries)
    } catch (err) {
      setState("error", String(err))
    } finally {
      setState("loading", false)
    }
  }

  const selectEntry = async (id: string | null) => {
    setState("selectedId", id)
    setState("selectedEntry", null)
    setState("parsedText", null)
    if (!id) return
    setState("detailLoading", true)
    try {
      const { entry, parsed_text } = await getKbEntry(id)
      setState("selectedEntry", entry)
      setState("parsedText", parsed_text ?? null)
    } catch (err) {
      setState("error", String(err))
    } finally {
      setState("detailLoading", false)
    }
  }

  const deleteEntry = async (id: string) => {
    await deleteKbEntry(id)
    setState("entries", (prev) => prev.filter((e) => e.id !== id))
    if (state.selectedId === id) {
      setState("selectedId", null)
      setState("selectedEntry", null)
      setState("parsedText", null)
    }
  }

  const uploadFile = async (file: File): Promise<KbEntry> => {
    const { entry } = await uploadKbFile(file)
    setState("entries", (prev) => [entry, ...prev])
    return entry
  }

  return { state, loadEntries, selectEntry, deleteEntry, uploadFile }
}

// ── Provider ──────────────────────────────────────────────────────────────────

export function KbProvider(props: ParentProps) {
  const value = createKbState()
  return <KbContext.Provider value={value}>{props.children}</KbContext.Provider>
}

export function useKb() {
  const ctx = useContext(KbContext)
  if (!ctx) throw new Error("useKb must be used within KbProvider")
  return ctx
}
