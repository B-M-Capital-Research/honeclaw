import { For, Show, createMemo, createSignal } from "solid-js"
import { useConsole } from "@/context/console"

function statusLabel(status: string): string {
  switch (status) {
    case "running":     return "运行中"
    case "disabled":    return "已禁用"
    case "stopped":     return "已停止"
    case "unsupported": return "不支持"
    default:            return status
  }
}

function statusDotClass(status: string): string {
  switch (status) {
    case "running":     return "bg-[color:var(--success)]"
    case "disabled":    return "bg-[color:var(--text-muted)] opacity-40"
    case "unsupported": return "bg-amber-400"
    default:            return "bg-rose-500"
  }
}

function statusTextClass(status: string): string {
  switch (status) {
    case "running":     return "text-[color:var(--success)]"
    case "disabled":
    case "unsupported": return "text-[color:var(--text-muted)]"
    default:            return "text-rose-400"
  }
}

export function ChannelStatusBadge() {
  const consoleState = useConsole()
  const [open, setOpen] = createSignal(false)

  const channels    = () => consoleState.channels() ?? []
  const channelError = () => consoleState.channelError()
  const hasData     = () => channels().length > 0

  const successCount = createMemo(() => channels().filter((c) => c.running).length)
  const failCount    = createMemo(() => channels().filter((c) => c.enabled && !c.running).length)

  const dotColor = createMemo(() => {
    if (channelError())         return "bg-amber-400"
    if (!hasData())             return "bg-[color:var(--text-muted)]"
    if (failCount() > 0)        return "bg-rose-500"
    if (successCount() > 0)     return "bg-[color:var(--success)]"
    return "bg-[color:var(--text-muted)]"
  })

  const summaryText = createMemo(() => {
    if (channelError()) return "状态获取失败"
    if (!hasData())     return "渠道加载中…"
    const parts: string[] = []
    if (successCount() > 0) parts.push(`${successCount()} 个渠道监听成功`)
    if (failCount() > 0)    parts.push(`${failCount()} 个渠道监听失败`)
    if (parts.length === 0) return "无活跃渠道"
    return parts.join("，")
  })

  let containerRef: HTMLDivElement | undefined

  const onClickOutside = (e: MouseEvent) => {
    if (containerRef && !containerRef.contains(e.target as Node)) {
      setOpen(false)
      document.removeEventListener("click", onClickOutside)
    }
  }

  const toggle = (e: MouseEvent) => {
    e.stopPropagation()
    if (open()) {
      setOpen(false)
      document.removeEventListener("click", onClickOutside)
    } else {
      setOpen(true)
      // 延迟一个 tick，避免当前点击立即触发关闭
      setTimeout(() => document.addEventListener("click", onClickOutside), 0)
    }
  }

  return (
    <div ref={containerRef} class="relative z-50">
      {/* 主 Badge 按钮 */}
      <button
        type="button"
        onClick={toggle}
        class="flex items-center gap-1.5 rounded-full border border-[color:var(--border)] bg-[color:var(--panel)] px-3 py-1.5 text-xs font-medium text-[color:var(--text-secondary)] shadow-sm transition hover:bg-[color:var(--surface)] hover:text-[color:var(--text-primary)]"
      >
        <span class={["h-2 w-2 shrink-0 rounded-full", dotColor()].join(" ")} />
        <span>{summaryText()}</span>
        {/* 下箭头 */}
        <svg
          class={["h-3 w-3 shrink-0 transition-transform duration-150", open() ? "rotate-180" : ""].join(" ")}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </button>

      {/* 下拉面板 */}
      <Show when={open()}>
        <div class="absolute right-0 top-full mt-1.5 min-w-[200px] rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-3 shadow-xl">
          {/* 错误提示 */}
          <Show when={channelError()}>
            <div class="mb-2.5 rounded-md border border-amber-300/30 bg-amber-400/10 px-2.5 py-1.5 text-[11px] text-amber-200">
              {channelError()}
            </div>
          </Show>

          {/* 渠道列表 */}
          <div class="flex flex-col gap-2">
            <For
              each={channels()}
              fallback={
                <div class="py-1 text-center text-[11px] text-[color:var(--text-muted)]">暂无渠道数据</div>
              }
            >
              {(channel) => (
                <div class="flex items-center justify-between gap-3">
                  <div class="flex min-w-0 items-center gap-1.5">
                    <span
                      class={["h-1.5 w-1.5 shrink-0 rounded-full", statusDotClass(channel.status)].join(" ")}
                    />
                    <span class="truncate text-[12px] font-medium text-[color:var(--text-primary)]">
                      {channel.label}
                    </span>
                  </div>
                  <span
                    class={["shrink-0 text-[10px] font-semibold uppercase tracking-wide", statusTextClass(channel.status)].join(" ")}
                  >
                    {statusLabel(channel.status)}
                  </span>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>
    </div>
  )
}
