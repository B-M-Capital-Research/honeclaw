import type { ChannelStatusInfo } from "@/lib/types"

type ChannelStatusCountsInput = Pick<
  ChannelStatusInfo,
  "enabled" | "running" | "processes"
>

type ChannelStatusCounts = {
  hasData: boolean
  successCount: number
  failCount: number
  duplicateProcessCount: number
}

type BackendConnectionStatusInput = {
  initializing: boolean
  connected: boolean
  error?: string
  isRemote: boolean
  baseUrl: string
  resolvedBaseUrl?: string
}

type FrontendConnectionStatusInput = {
  isDesktop: boolean
  origin: string
}

type ConnectionStatusItem = {
  label: string
  detail: string
  status: "degraded" | "running" | "stopped"
}

export function statusLabel(status: string): string {
  switch (status) {
    case "running":
      return "运行中"
    case "degraded":
      return "部分异常"
    case "disabled":
      return "已禁用"
    case "stopped":
      return "已停止"
    case "unsupported":
      return "不支持"
    default:
      return status
  }
}

export function statusDotClass(status: string): string {
  switch (status) {
    case "running":
      return "bg-[color:var(--success)]"
    case "degraded":
      return "bg-amber-400"
    case "disabled":
      return "bg-[color:var(--text-muted)] opacity-40"
    case "unsupported":
      return "bg-amber-400"
    default:
      return "bg-rose-500"
  }
}

export function statusTextClass(status: string): string {
  switch (status) {
    case "running":
      return "text-[color:var(--success)]"
    case "degraded":
      return "text-amber-300"
    case "disabled":
    case "unsupported":
      return "text-[color:var(--text-muted)]"
    default:
      return "text-rose-400"
  }
}

export function summarizeChannelStatuses(
  channels: ChannelStatusCountsInput[],
): ChannelStatusCounts {
  return channels.reduce<ChannelStatusCounts>(
    (summary, channel) => ({
      hasData: true,
      successCount: summary.successCount + (channel.running ? 1 : 0),
      failCount:
        summary.failCount + (channel.enabled && !channel.running ? 1 : 0),
      duplicateProcessCount:
        summary.duplicateProcessCount +
        ((channel.processes?.length ?? 0) > 1 ? 1 : 0),
    }),
    {
      hasData: false,
      successCount: 0,
      failCount: 0,
      duplicateProcessCount: 0,
    },
  )
}

export function channelBadgeDotClass(options: {
  backendConnected: boolean
  backendInitializing: boolean
  channelError: string
  counts: ChannelStatusCounts
}): string {
  if (!options.backendConnected && !options.backendInitializing) return "bg-rose-500"
  if (options.channelError) return "bg-amber-400"
  if (!options.counts.hasData) return "bg-[color:var(--text-muted)]"
  if (options.counts.failCount > 0) return "bg-rose-500"
  if (options.counts.successCount > 0) return "bg-[color:var(--success)]"
  return "bg-[color:var(--text-muted)]"
}

export function backendConnectionLabel(options: {
  initializing: boolean
  connected: boolean
}): string {
  if (options.initializing) return "后端连接中"
  if (options.connected) return "管理端后端正常连接中"
  return "管理端后端未连接"
}

export function channelSummaryText(
  counts: ChannelStatusCounts,
  backendLabel: string,
): string {
  const channelText = counts.hasData
    ? `${counts.successCount} 个渠道监听中`
    : "渠道加载中"
  return [channelText, backendLabel, "管理端前端正常连接中"].join("，")
}

export function backendConnectionStatus(
  options: BackendConnectionStatusInput,
): ConnectionStatusItem {
  if (options.initializing) {
    return {
      label: "管理端后端",
      detail: "正在建立连接…",
      status: "degraded",
    }
  }
  if (options.connected) {
    const target =
      options.resolvedBaseUrl || (options.isRemote ? options.baseUrl : "bundled")
    return {
      label: "管理端后端",
      detail: options.isRemote
        ? `remote · ${target}（管理端端口 8077）`
        : `bundled · ${target}（管理端端口 8077）`,
      status: "running",
    }
  }
  return {
    label: "管理端后端",
    detail: options.error || "未连接",
    status: "stopped",
  }
}

export function frontendConnectionStatus(
  options: FrontendConnectionStatusInput,
): ConnectionStatusItem {
  return {
    label: "管理端前端",
    detail: options.isDesktop
      ? `desktop · ${options.origin}`
      : `browser · ${options.origin}（管理端页面）`,
    status: "running",
  }
}
