import { For, Show, createMemo, createSignal } from "solid-js"
import { useConsole } from "@/context/console"
import { useBackend } from "@/context/backend"
import { cleanupDesktopChannelProcesses } from "@/lib/backend"

function statusLabel(status: string): string {
  switch (status) {
    case "running":     return "运行中"
    case "degraded":    return "部分异常"
    case "disabled":    return "已禁用"
    case "stopped":     return "已停止"
    case "unsupported": return "不支持"
    default:            return status
  }
}

function statusDotClass(status: string): string {
  switch (status) {
    case "running":     return "bg-[color:var(--success)]"
    case "degraded":    return "bg-amber-400"
    case "disabled":    return "bg-[color:var(--text-muted)] opacity-40"
    case "unsupported": return "bg-amber-400"
    default:            return "bg-rose-500"
  }
}

function statusTextClass(status: string): string {
  switch (status) {
    case "running":     return "text-[color:var(--success)]"
    case "degraded":    return "text-amber-300"
    case "disabled":
    case "unsupported": return "text-[color:var(--text-muted)]"
    default:            return "text-rose-400"
  }
}

export function ChannelStatusBadge() {
  const consoleState = useConsole()
  const backend = useBackend()
  const [open, setOpen] = createSignal(false)
  const [cleanupBusy, setCleanupBusy] = createSignal(false)
  const [cleanupMessage, setCleanupMessage] = createSignal("")

  const channels    = () => consoleState.channels() ?? []
  const channelError = () => consoleState.channelError()
  const hasData     = () => channels().length > 0
  const duplicateProcessCount = createMemo(() =>
    channels().filter((channel) => (channel.processes?.length ?? 0) > 1).length,
  )

  const successCount = createMemo(() => channels().filter((c) => c.running).length)
  const failCount    = createMemo(() => channels().filter((c) => c.enabled && !c.running).length)
  const totalListeningCount = createMemo(() => successCount())
  const backendConnected = createMemo(() => backend.state.connected)
  const frontendConnected = createMemo(() => true)
  const backendLabel = createMemo(() => {
    if (backend.state.initializing) return "后端连接中"
    if (backendConnected()) return "后端正常连接中"
    return "后端未连接"
  })
  const frontendLabel = createMemo(() => frontendConnected() ? "前端正常连接中" : "前端未连接")

  const dotColor = createMemo(() => {
    if (!backendConnected() && !backend.state.initializing) return "bg-rose-500"
    if (channelError())         return "bg-amber-400"
    if (!hasData())             return "bg-[color:var(--text-muted)]"
    if (failCount() > 0)        return "bg-rose-500"
    if (successCount() > 0)     return "bg-[color:var(--success)]"
    return "bg-[color:var(--text-muted)]"
  })

  const summaryText = createMemo(() => {
    const channelText = hasData() ? `${totalListeningCount()} 个渠道监听中` : "渠道加载中"
    return [channelText, backendLabel(), frontendLabel()].join("，")
  })

  const backendStatus = createMemo(() => {
    if (backend.state.initializing) {
      return {
        label: "后端",
        detail: "正在建立连接…",
        status: "degraded",
      }
    }
    if (backend.state.connected) {
      const target =
        backend.state.resolvedBaseUrl ||
        (backend.isRemote() ? backend.state.config.baseUrl : "bundled")
      return {
        label: "后端",
        detail: backend.isRemote() ? `remote · ${target}` : `bundled · ${target}`,
        status: "running",
      }
    }
    return {
      label: "后端",
      detail: backend.state.error || "未连接",
      status: "stopped",
    }
  })

  const frontendStatus = createMemo(() => {
    const target =
      typeof window !== "undefined" && window.location?.origin && window.location.origin !== "null"
        ? window.location.origin
        : "desktop shell"
    return {
      label: "前端",
      detail: backend.state.isDesktop ? `desktop · ${target}` : `browser · ${target}`,
      status: frontendConnected() ? "running" : "stopped",
    }
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

  const cleanupDuplicates = async () => {
    if (!backend.state.isDesktop || cleanupBusy()) return
    setCleanupBusy(true)
    setCleanupMessage("")
    try {
      const result = await cleanupDesktopChannelProcesses()
      setCleanupMessage(result.message)
      await consoleState.refreshChannels()
    } catch (error) {
      setCleanupMessage(error instanceof Error ? error.message : String(error))
    } finally {
      setCleanupBusy(false)
    }
  }

  return (
    <div ref={containerRef} class="relative z-50">
      {/* 主 Badge 按钮 */}
      <button
        type="button"
        onClick={toggle}
        class="flex max-w-[min(70vw,680px)] items-center gap-1.5 rounded-full border border-[color:var(--border)] bg-[color:var(--panel)] px-3 py-1.5 text-xs font-medium text-[color:var(--text-secondary)] shadow-sm transition hover:bg-[color:var(--surface)] hover:text-[color:var(--text-primary)]"
      >
        <span class={["h-2 w-2 shrink-0 rounded-full", dotColor()].join(" ")} />
        <span class="truncate">{summaryText()}</span>
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
        <div class="absolute right-0 top-full mt-1.5 min-w-[320px] max-w-[420px] rounded-lg border border-[color:var(--border)] bg-[color:var(--panel)] p-3 shadow-xl">
          <Show when={backend.state.isDesktop}>
            <div class="mb-2.5 flex items-center justify-between gap-2">
              <div class="text-[10px] text-[color:var(--text-muted)]">
                {duplicateProcessCount() > 0 ? `${duplicateProcessCount()} 个渠道存在重复进程` : "每个渠道当前最多保留 1 个主进程"}
              </div>
              <button
                type="button"
                disabled={cleanupBusy()}
                onClick={() => void cleanupDuplicates()}
                class="rounded-md border border-[color:var(--border)] px-2 py-1 text-[10px] font-semibold text-[color:var(--text-secondary)] transition hover:bg-[color:var(--surface)] disabled:cursor-not-allowed disabled:opacity-50"
              >
                {cleanupBusy() ? "清理中…" : "清理多余进程"}
              </button>
            </div>
          </Show>

          {/* 错误提示 */}
          <Show when={channelError()}>
            <div class="mb-2.5 rounded-md border border-amber-300/30 bg-amber-400/10 px-2.5 py-1.5 text-[11px] text-amber-200">
              {channelError()}
            </div>
          </Show>
          <Show when={cleanupMessage()}>
            <div class="mb-2.5 rounded-md border border-sky-300/20 bg-sky-400/10 px-2.5 py-1.5 text-[11px] text-sky-100">
              {cleanupMessage()}
            </div>
          </Show>

          <div class="mb-3 flex flex-col gap-2">
            <div class="text-[10px] font-semibold uppercase tracking-wide text-[color:var(--text-muted)]">
              系统连接
            </div>

            <For each={[backendStatus(), frontendStatus()]}>
              {(item) => (
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0">
                    <div class="flex min-w-0 items-center gap-1.5">
                      <span class={["h-1.5 w-1.5 shrink-0 rounded-full", statusDotClass(item.status)].join(" ")} />
                      <span class="truncate text-[12px] font-medium text-[color:var(--text-primary)]">
                        {item.label}
                      </span>
                    </div>
                    <div class="mt-1 break-all text-[10px] leading-4 text-[color:var(--text-muted)]">
                      {item.detail}
                    </div>
                  </div>
                  <span
                    class={["shrink-0 text-[10px] font-semibold uppercase tracking-wide", statusTextClass(item.status)].join(" ")}
                  >
                    {statusLabel(item.status)}
                  </span>
                </div>
              )}
            </For>
          </div>

          {/* 渠道列表 */}
          <div class="flex flex-col gap-2">
            <div class="text-[10px] font-semibold uppercase tracking-wide text-[color:var(--text-muted)]">
              渠道监听
            </div>
            <For
              each={channels()}
              fallback={
                <div class="py-1 text-center text-[11px] text-[color:var(--text-muted)]">暂无渠道数据</div>
              }
            >
              {(channel) => (
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0">
                    <div class="flex min-w-0 items-center gap-1.5">
                      <span
                        class={["h-1.5 w-1.5 shrink-0 rounded-full", statusDotClass(channel.status)].join(" ")}
                      />
                      <span class="truncate text-[12px] font-medium text-[color:var(--text-primary)]">
                        {channel.label}
                      </span>
                    </div>
                    <div class="mt-1 text-[10px] leading-4 text-[color:var(--text-muted)]">
                      {channel.detail}
                    </div>
                    <Show when={channel.processes?.length}>
                      <div class="mt-1 flex flex-wrap gap-1">
                        <For each={channel.processes}>
                          {(process) => (
                            <span
                              class={[
                                "rounded-full border px-1.5 py-0.5 text-[10px] leading-none",
                                process.running
                                  ? "border-emerald-400/30 bg-emerald-400/10 text-emerald-200"
                                  : "border-rose-400/30 bg-rose-400/10 text-rose-200",
                              ].join(" ")}
                              title={process.last_heartbeat_at ? `last_seen=${process.last_heartbeat_at}` : "当前仅检测到进程，未收到该实例心跳"}
                            >
                              pid {process.pid}
                            </span>
                          )}
                        </For>
                      </div>
                    </Show>
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
