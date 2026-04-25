import { useSymbolDrawer } from "@/context/symbol-drawer"

export type SymbolLinkProps = {
  symbol: string
  /** 自定义显示文本(默认大写 symbol) */
  label?: string
  /** 紧凑模式(去掉 chip 外壳,直接显示带下划线的文本) */
  inline?: boolean
  class?: string
}

/**
 * 把纯文本 symbol 渲染为可点击的元素 — 点击打开 SymbolDrawer。
 * 用于持仓表、研究详情、画像标题等所有展示 symbol 的地方。
 */
export function SymbolLink(props: SymbolLinkProps) {
  const drawer = useSymbolDrawer()
  const label = () => (props.label ?? props.symbol).toUpperCase()

  const handleClick = (e: MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()
    drawer.openSymbol(props.symbol)
  }

  if (props.inline) {
    return (
      <button
        type="button"
        onClick={handleClick}
        class={[
          "inline font-mono font-medium text-[color:var(--accent)] underline-offset-2 hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]",
          props.class ?? "",
        ].join(" ")}
        title={`查看 ${label()} 详情`}
      >
        {label()}
      </button>
    )
  }

  return (
    <button
      type="button"
      onClick={handleClick}
      class={[
        "inline-flex items-center rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-2 py-0.5 font-mono text-xs font-medium text-[color:var(--text-primary)] transition hover:border-[color:var(--accent)] hover:bg-[color:var(--accent-soft)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]",
        props.class ?? "",
      ].join(" ")}
      title={`查看 ${label()} 详情`}
    >
      {label()}
    </button>
  )
}
