import { useLocation, useNavigate, useSearchParams } from "@solidjs/router"
import {
  createContext,
  createMemo,
  useContext,
  type ParentProps,
} from "solid-js"

/**
 * Symbol 上下文枢纽 — 统一管理"打开 symbol 详情抽屉"的 open/close 行为。
 *
 * URL 同步: ?symbol=AAPL — 抽屉的开启/关闭走 URL,所以可分享、可后退键关闭。
 */
type SymbolDrawerContextValue = {
  symbol: () => string | undefined
  isOpen: () => boolean
  openSymbol: (symbol: string) => void
  close: () => void
}

const SymbolDrawerContext = createContext<SymbolDrawerContextValue>()

function normalizeSymbol(value: string): string {
  return value.trim().toUpperCase().replace(/^\$/, "")
}

export function SymbolDrawerProvider(props: ParentProps) {
  const [searchParams, setSearchParams] = useSearchParams()
  const navigate = useNavigate()
  const location = useLocation()

  const symbol = createMemo<string | undefined>(() => {
    const raw = typeof searchParams.symbol === "string" ? searchParams.symbol : undefined
    if (!raw) return undefined
    const normalized = normalizeSymbol(raw)
    return normalized || undefined
  })

  const value: SymbolDrawerContextValue = {
    symbol,
    isOpen: () => !!symbol(),
    openSymbol(next) {
      const normalized = normalizeSymbol(next)
      if (!normalized) return
      // 通过 setSearchParams 保留当前路径,只更新 query
      setSearchParams({ symbol: normalized })
    },
    close() {
      // 清除 ?symbol= 参数,保持其它 query 与 hash
      const url = new URL(window.location.href)
      url.searchParams.delete("symbol")
      const next = url.pathname + (url.search ? url.search : "") + url.hash
      navigate(next || location.pathname, { replace: true })
    },
  }

  return (
    <SymbolDrawerContext.Provider value={value}>
      {props.children}
    </SymbolDrawerContext.Provider>
  )
}

export function useSymbolDrawer() {
  const ctx = useContext(SymbolDrawerContext)
  if (!ctx) throw new Error("SymbolDrawerProvider missing")
  return ctx
}
