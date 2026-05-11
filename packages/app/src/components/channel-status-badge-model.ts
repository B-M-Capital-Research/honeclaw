import type { ChannelStatusInfo } from "@/lib/types"

type ChannelStatusCountsInput = Pick<
  ChannelStatusInfo,
  "enabled" | "running" | "processes"
>

export type ChannelStatusCounts = {
  hasData: boolean
  successCount: number
  failCount: number
  duplicateProcessCount: number
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
