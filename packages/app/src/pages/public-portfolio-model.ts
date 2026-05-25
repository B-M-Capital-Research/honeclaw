type PublicRefreshResult = {
  mainline_count: number
  skipped_tickers: string[]
}

export function formatPublicMainlineTimestamp(
  iso: string | null,
  nowMs = Date.now(),
): string {
  if (!iso) return "从未"
  const dt = new Date(iso)
  if (Number.isNaN(dt.getTime())) return iso

  const days = Math.floor((nowMs - dt.getTime()) / (24 * 3600 * 1000))
  if (days === 0) {
    return `今天 ${dt.toLocaleTimeString("zh-CN", {
      hour: "2-digit",
      minute: "2-digit",
    })}`
  }
  if (days === 1) return "1 天前"
  if (days < 7) return `${days} 天前`
  return dt.toLocaleDateString("zh-CN", {
    year: "numeric",
    month: "short",
    day: "numeric",
  })
}

export function publicRefreshMessage(result: PublicRefreshResult): string {
  return `更新完成：${result.mainline_count} 条投资主线，${result.skipped_tickers.length} 只跳过`
}

export function canRefreshPublicMainline(profileCount: number): boolean {
  return profileCount > 0
}
