import { describe, expect, it } from "bun:test"

import {
  NOTIFICATION_QUERY_LIMIT,
  buildNotificationsQuery,
  bucketHourLabel,
  eventKindLabel,
  execLabel,
  notificationBucketSegments,
  notificationPeakBucket,
  recordSourceLabel,
  sendBadgeClass,
  sendLabel,
} from "./notifications-model"
import { setLocale } from "@/lib/i18n"

describe("notifications-model", () => {
  it("builds notification queries from filter state", () => {
    const referenceNow = new Date("2026-05-13T12:00:00.000Z")

    expect(
      buildNotificationsQuery({
        now: referenceNow,
        hours: 6,
        selectedActor: null,
        channel: "discord",
        execStatus: "completed",
        sendStatus: "sent",
      }),
    ).toEqual({
      since: "2026-05-13T06:00:00.000Z",
      channel: "discord",
      user_id: undefined,
      channel_scope: undefined,
      execution_status: "completed",
      message_send_status: "sent",
      limit: NOTIFICATION_QUERY_LIMIT,
    })

    expect(
      buildNotificationsQuery({
        now: referenceNow,
        hours: 24,
        selectedActor: {
          channel: "telegram",
          user_id: "u1",
          channel_scope: "scope-a",
        },
        channel: "discord",
        execStatus: "",
        sendStatus: "",
      }),
    ).toEqual({
      since: "2026-05-12T12:00:00.000Z",
      channel: "telegram",
      user_id: "u1",
      channel_scope: "scope-a",
      execution_status: undefined,
      message_send_status: undefined,
      limit: NOTIFICATION_QUERY_LIMIT,
    })
  })

  it("keeps status labels and badge tone mapping outside the page", () => {
    setLocale("zh")

    expect(sendLabel("sent")).toBe("已发送")
    expect(sendLabel("unknown_status")).toBe("unknown_status")
    expect(execLabel("execution_failed")).toBe("执行失败")
    expect(sendBadgeClass("sent")).toBe("text-emerald-300 bg-emerald-500/15")
    expect(sendBadgeClass("send_failed")).toBe("text-rose-300 bg-rose-500/15")
    expect(sendBadgeClass("quiet_held")).toBe("text-amber-300 bg-amber-500/15")
    expect(sendBadgeClass("filtered")).toBe(
      "text-[color:var(--text-muted)] bg-white/5",
    )
  })

  it("maps record source and event kind fallbacks", () => {
    setLocale("zh")

    expect(recordSourceLabel("cron_job")).toBe("定时任务")
    expect(recordSourceLabel("event_engine")).toBe("事件")
    expect(recordSourceLabel("custom")).toBe("custom")
    expect(eventKindLabel("price_alert")).toBe("价格异动")
    expect(eventKindLabel("custom_kind")).toBe("custom_kind")
    expect(eventKindLabel(null)).toBe("—")
  })

  it("derives chart helper values from histogram data", () => {
    const peakHistogramBucket = {
      bucket_start: "2026-05-13T01:00:00.000Z",
      total: 8,
      sent: 7,
      failed: 0,
      skipped: 1,
    }

    expect(
      notificationPeakBucket([
        {
          bucket_start: "2026-05-13T00:00:00.000Z",
          total: 3,
          sent: 2,
          failed: 1,
          skipped: 0,
        },
        peakHistogramBucket,
      ]),
    ).toBe(8)

    expect(notificationBucketSegments(peakHistogramBucket, 8)).toEqual({
      heightPct: 100,
      sentPct: 87.5,
      failedPct: 0,
      minHeight: "2px",
    })
    expect(
      notificationBucketSegments(
        {
          bucket_start: "2026-05-13T02:00:00.000Z",
          total: 0,
          sent: 0,
          failed: 0,
          skipped: 0,
        },
        8,
      ),
    ).toEqual({
      heightPct: 0,
      sentPct: 0,
      failedPct: 0,
      minHeight: "0",
    })

    expect(bucketHourLabel("not-a-date", "zh")).toBe("not-a-date")
    expect(bucketHourLabel("2026-05-13T00:00:00.000Z", "zh")).toMatch(/08/)
  })
})
