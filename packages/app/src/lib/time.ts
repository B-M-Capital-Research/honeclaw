const SHANGHAI_TIME_ZONE = "Asia/Shanghai"

export function formatShanghaiDateTime(
  iso?: string,
  options?: Intl.DateTimeFormatOptions,
) {
  if (!iso) return "未知"
  try {
    return new Date(iso).toLocaleString("zh-CN", {
      timeZone: SHANGHAI_TIME_ZONE,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
      ...options,
    })
  } catch {
    return iso
  }
}
