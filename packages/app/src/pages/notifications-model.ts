import type {
  NotificationHistogramBucket,
  NotificationsQuery,
} from "@/lib/api"
import type { ActorRef } from "@/lib/actors"
import { NOTIFICATIONS } from "@/lib/admin-content/notifications"

export const NOTIFICATION_QUERY_LIMIT = 200
const EMPTY_LABEL = "—"

export function sendStatusOptions(): Array<{ value: string; label: string }> {
  const labels = NOTIFICATIONS.send_status
  return [
    { value: "", label: labels.all },
    { value: "sent", label: labels.sent },
    { value: "dryrun", label: labels.dryrun },
    { value: "queued", label: labels.queued },
    { value: "quiet_held", label: labels.quiet_held },
    { value: "filtered", label: labels.filtered },
    { value: "capped", label: labels.capped },
    { value: "cooled_down", label: labels.cooled_down },
    { value: "price_capped", label: labels.price_capped },
    { value: "price_cooled_down", label: labels.price_cooled_down },
    { value: "omitted", label: labels.omitted },
    { value: "skipped_noop", label: labels.skipped_noop },
    { value: "skipped_error", label: labels.skipped_error },
    { value: "send_failed", label: labels.send_failed },
    { value: "failed", label: labels.failed },
    { value: "target_resolution_failed", label: labels.target_resolution_failed },
    { value: "duplicate_suppressed", label: labels.duplicate_suppressed },
  ]
}

export function execStatusOptions(): Array<{ value: string; label: string }> {
  const labels = NOTIFICATIONS.exec_status
  return [
    { value: "", label: labels.all },
    { value: "completed", label: labels.completed },
    { value: "noop", label: labels.noop },
    { value: "execution_failed", label: labels.execution_failed },
  ]
}

export function channelOptions(): Array<{ value: string; label: string }> {
  return [
    { value: "", label: NOTIFICATIONS.channel.all },
    { value: "telegram", label: "Telegram" },
    { value: "discord", label: "Discord" },
    { value: "feishu", label: NOTIFICATIONS.channel.feishu },
    { value: "imessage", label: "iMessage" },
  ]
}

export function sendLabel(status: string): string {
  return (
    sendStatusOptions().find((option) => option.value === status)?.label ??
    status ??
    EMPTY_LABEL
  )
}

export function execLabel(status: string): string {
  return (
    execStatusOptions().find((option) => option.value === status)?.label ??
    status ??
    EMPTY_LABEL
  )
}

export function sendBadgeClass(status: string): string {
  switch (status) {
    case "sent":
    case "dryrun":
      return "text-emerald-300 bg-emerald-500/15"
    case "send_failed":
    case "failed":
    case "target_resolution_failed":
    case "skipped_error":
      return "text-rose-300 bg-rose-500/15"
    case "duplicate_suppressed":
    case "quiet_held":
    case "queued":
    case "capped":
    case "cooled_down":
    case "price_capped":
    case "price_cooled_down":
      return "text-amber-300 bg-amber-500/15"
    case "filtered":
    case "omitted":
    case "skipped_noop":
      return "text-[color:var(--text-muted)] bg-white/5"
    default:
      return "text-[color:var(--text-muted)] bg-white/5"
  }
}

export function bucketHourLabel(iso: string, locale: "zh" | "en"): string {
  const date = new Date(iso)
  if (isNaN(date.getTime())) return iso
  return date.toLocaleString(locale === "zh" ? "zh-CN" : "en-US", {
    timeZone: "Asia/Shanghai",
    hour: "2-digit",
    hour12: false,
  })
}

export function recordSourceLabel(source: string): string {
  switch (source) {
    case "cron_job":
      return NOTIFICATIONS.source.cron
    case "event_engine":
      return NOTIFICATIONS.source.event
    default:
      return source || EMPTY_LABEL
  }
}

export function eventKindLabel(kind?: string | null): string {
  switch (kind) {
    case "earnings_upcoming":
      return NOTIFICATIONS.event_kind.earnings_upcoming
    case "earnings_released":
      return NOTIFICATIONS.event_kind.earnings_released
    case "earnings_call_transcript":
      return NOTIFICATIONS.event_kind.earnings_call_transcript
    case "news_critical":
      return NOTIFICATIONS.event_kind.news_critical
    case "price_alert":
      return NOTIFICATIONS.event_kind.price_alert
    case "weekly52_high":
      return NOTIFICATIONS.event_kind.weekly52_high
    case "weekly52_low":
      return NOTIFICATIONS.event_kind.weekly52_low
    case "dividend":
      return NOTIFICATIONS.event_kind.dividend
    case "split":
      return NOTIFICATIONS.event_kind.split
    case "sec_filing":
      return NOTIFICATIONS.event_kind.sec_filing
    case "analyst_grade":
      return NOTIFICATIONS.event_kind.analyst_grade
    case "macro_event":
      return NOTIFICATIONS.event_kind.macro_event
    case "social_post":
      return NOTIFICATIONS.event_kind.social_post
    default:
      return kind || EMPTY_LABEL
  }
}

export function notificationPeakBucket(
  histogram: NotificationHistogramBucket[],
): number {
  let max = 0
  for (const bucket of histogram) {
    if (bucket.total > max) max = bucket.total
  }
  return max
}

export function buildNotificationsQuery(options: {
  now: Date
  hours: number
  selectedActor: ActorRef | null
  channel: string
  execStatus: string
  sendStatus: string
}): NotificationsQuery {
  const sinceDate = new Date(
    options.now.getTime() - options.hours * 3600 * 1000,
  )
  const actor = options.selectedActor
  return {
    since: sinceDate.toISOString(),
    channel: actor?.channel ?? (options.channel || undefined),
    user_id: actor?.user_id,
    channel_scope: actor?.channel_scope,
    execution_status: options.execStatus || undefined,
    message_send_status: options.sendStatus || undefined,
    limit: NOTIFICATION_QUERY_LIMIT,
  }
}
