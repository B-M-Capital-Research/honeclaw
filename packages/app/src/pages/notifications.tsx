import {
  For,
  Show,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js"
import { ActorSelect } from "@/components/actor-select"
import {
  getNotifications,
  type NotificationHistogramBucket,
  type NotificationRecord,
  type NotificationsQuery,
  type NotificationsSummary,
} from "@/lib/api"
import { actorKey, type ActorRef } from "@/lib/actors"
import { formatShanghaiDateTime } from "@/lib/time"
import { NOTIFICATIONS } from "@/lib/admin-content/notifications"
import { tpl, useLocale } from "@/lib/i18n"

// ── 状态映射(对齐 task-detail.tsx 的 sendStatusLabel/executionStatusLabel) ──

function sendStatusOptions(): Array<{ value: string; label: string }> {
  const t = NOTIFICATIONS.send_status
  return [
    { value: "", label: t.all },
    { value: "sent", label: t.sent },
    { value: "dryrun", label: t.dryrun },
    { value: "queued", label: t.queued },
    { value: "quiet_held", label: t.quiet_held },
    { value: "filtered", label: t.filtered },
    { value: "capped", label: t.capped },
    { value: "cooled_down", label: t.cooled_down },
    { value: "price_capped", label: t.price_capped },
    { value: "price_cooled_down", label: t.price_cooled_down },
    { value: "omitted", label: t.omitted },
    { value: "skipped_noop", label: t.skipped_noop },
    { value: "skipped_error", label: t.skipped_error },
    { value: "send_failed", label: t.send_failed },
    { value: "failed", label: t.failed },
    { value: "target_resolution_failed", label: t.target_resolution_failed },
    { value: "duplicate_suppressed", label: t.duplicate_suppressed },
  ]
}

function execStatusOptions(): Array<{ value: string; label: string }> {
  const t = NOTIFICATIONS.exec_status
  return [
    { value: "", label: t.all },
    { value: "completed", label: t.completed },
    { value: "noop", label: t.noop },
    { value: "execution_failed", label: t.execution_failed },
  ]
}

function channelOptions(): Array<{ value: string; label: string }> {
  return [
    { value: "", label: NOTIFICATIONS.channel.all },
    { value: "telegram", label: "Telegram" },
    { value: "discord", label: "Discord" },
    { value: "feishu", label: NOTIFICATIONS.channel.feishu },
    { value: "imessage", label: "iMessage" },
  ]
}

function sendLabel(s: string): string {
  return sendStatusOptions().find((o) => o.value === s)?.label ?? s ?? "—"
}
function execLabel(s: string): string {
  return execStatusOptions().find((o) => o.value === s)?.label ?? s ?? "—"
}

function sendBadgeClass(s: string): string {
  switch (s) {
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

function bucketHourLabel(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return iso
  const loc = useLocale() === "zh" ? "zh-CN" : "en-US"
  return d.toLocaleString(loc, {
    timeZone: "Asia/Shanghai",
    hour: "2-digit",
    hour12: false,
  })
}

function recordSourceLabel(source: string): string {
  switch (source) {
    case "cron_job":
      return NOTIFICATIONS.source.cron
    case "event_engine":
      return NOTIFICATIONS.source.event
    default:
      return source || "—"
  }
}

function eventKindLabel(kind?: string | null): string {
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
      return kind || "—"
  }
}

// ── 组件 ─────────────────────────────────────────────────────────────────────

export default function NotificationsPage() {
  const [channel, setChannel] = createSignal("")
  const [selectedActor, setSelectedActor] = createSignal<ActorRef | null>(null)
  const [execStatus, setExecStatus] = createSignal("")
  const [sendStatus, setSendStatus] = createSignal("")
  const [hours, setHours] = createSignal<number>(24)
  const [limit, setLimit] = createSignal<number>(200)

  const [records, setRecords] = createSignal<NotificationRecord[]>([])
  const [histogram, setHistogram] = createSignal<NotificationHistogramBucket[]>(
    [],
  )
  const [summary, setSummary] = createSignal<NotificationsSummary>({
    total: 0,
    sent: 0,
    failed: 0,
    skipped: 0,
    duplicate_suppressed: 0,
    distinct_users: 0,
  })
  const [loading, setLoading] = createSignal(false)
  const [err, setErr] = createSignal<string | null>(null)
  const [openRecord, setOpenRecord] = createSignal<NotificationRecord | null>(
    null,
  )

  async function refresh() {
    setLoading(true)
    setErr(null)
    try {
      const sinceDate = new Date(Date.now() - hours() * 3600 * 1000)
      const actor = selectedActor()
      const q: NotificationsQuery = {
        since: sinceDate.toISOString(),
        channel: actor?.channel ?? (channel() || undefined),
        user_id: actor?.user_id,
        channel_scope: actor?.channel_scope,
        execution_status: execStatus() || undefined,
        message_send_status: sendStatus() || undefined,
        limit: limit(),
      }
      const resp = await getNotifications(q)
      setRecords(resp.records)
      setHistogram(resp.histogram_24h)
      setSummary(resp.summary_24h)
    } catch (e) {
      setErr(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(() => {
    void refresh()
    const timer = window.setInterval(refresh, 30_000)
    onCleanup(() => window.clearInterval(timer))
  })

  const peakBucket = createMemo(() => {
    let max = 0
    for (const b of histogram()) {
      if (b.total > max) max = b.total
    }
    return max
  })

  return (
    <div class="flex h-full min-h-0 flex-col gap-4 p-4 text-sm">
      {/* 顶栏 + 过滤器 */}
      <div class="flex flex-wrap items-center gap-3">
        <h1 class="text-lg font-semibold text-[color:var(--text-primary)]">
          {NOTIFICATIONS.page.title}
        </h1>
        <div class="flex items-center gap-1 text-xs text-[color:var(--text-muted)]">
          <span>{NOTIFICATIONS.page.window_label}</span>
          <select
            value={hours()}
            onChange={(e) => {
              setHours(Number(e.currentTarget.value))
              void refresh()
            }}
            class="rounded border border-[color:var(--border)] bg-transparent px-2 py-1 text-xs text-[color:var(--text-primary)]"
          >
            <option value={1}>1h</option>
            <option value={6}>6h</option>
            <option value={24}>24h</option>
            <option value={72}>3d</option>
            <option value={168}>7d</option>
          </select>
        </div>
        <div class="flex items-center gap-1 text-xs text-[color:var(--text-muted)]">
          <span>{NOTIFICATIONS.page.channel_label}</span>
          <select
            value={channel()}
            disabled={!!selectedActor()}
            onChange={(e) => {
              setChannel(e.currentTarget.value)
              void refresh()
            }}
            class="rounded border border-[color:var(--border)] bg-transparent px-2 py-1 text-xs text-[color:var(--text-primary)] disabled:opacity-50"
          >
            <For each={channelOptions()}>
              {(o) => <option value={o.value}>{o.label}</option>}
            </For>
          </select>
        </div>
        <div class="flex items-center gap-1 text-xs text-[color:var(--text-muted)]">
          <span>{NOTIFICATIONS.page.user_label}</span>
          <ActorSelect
            allowAll
            allLabel={NOTIFICATIONS.page.all_users}
            value={selectedActor() ? actorKey(selectedActor()!) : ""}
            onChange={(actor) => {
              setSelectedActor(actor)
              void refresh()
            }}
          />
        </div>
        <div class="flex items-center gap-1 text-xs text-[color:var(--text-muted)]">
          <select
            value={sendStatus()}
            onChange={(e) => {
              setSendStatus(e.currentTarget.value)
              void refresh()
            }}
            class="rounded border border-[color:var(--border)] bg-transparent px-2 py-1 text-xs text-[color:var(--text-primary)]"
          >
            <For each={sendStatusOptions()}>
              {(o) => <option value={o.value}>{o.label}</option>}
            </For>
          </select>
        </div>
        <div class="flex items-center gap-1 text-xs text-[color:var(--text-muted)]">
          <select
            value={execStatus()}
            onChange={(e) => {
              setExecStatus(e.currentTarget.value)
              void refresh()
            }}
            class="rounded border border-[color:var(--border)] bg-transparent px-2 py-1 text-xs text-[color:var(--text-primary)]"
          >
            <For each={execStatusOptions()}>
              {(o) => <option value={o.value}>{o.label}</option>}
            </For>
          </select>
        </div>
        <button
          onClick={() => void refresh()}
          disabled={loading()}
          class="rounded border border-[color:var(--border)] px-3 py-1 text-xs hover:bg-white/5 disabled:opacity-50"
        >
          {loading() ? NOTIFICATIONS.page.refreshing_button : NOTIFICATIONS.page.refresh_button}
        </button>
      </div>

      <Show when={err()}>
        <div class="rounded border border-rose-500/40 bg-rose-500/10 p-3 text-rose-300">
          {err()}
        </div>
      </Show>

      {/* 24h 汇总数字 */}
      <section class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
        <SummaryCard label={NOTIFICATIONS.page.summary_total} value={summary().total} />
        <SummaryCard label={NOTIFICATIONS.page.summary_sent} value={summary().sent} tone="ok" />
        <SummaryCard label={NOTIFICATIONS.page.summary_failed} value={summary().failed} tone="bad" />
        <SummaryCard label={NOTIFICATIONS.page.summary_skipped} value={summary().skipped} tone="muted" />
        <SummaryCard
          label={NOTIFICATIONS.page.summary_duplicate}
          value={summary().duplicate_suppressed}
          tone="warn"
        />
        <SummaryCard label={NOTIFICATIONS.page.summary_users} value={summary().distinct_users} />
      </section>

      {/* 24h 直方图 */}
      <section class="space-y-2">
        <div class="text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">
          {NOTIFICATIONS.page.histogram_title}
        </div>
        <div class="rounded border border-[color:var(--border)] p-3">
          <div class="flex h-24 items-end gap-[2px]">
            <For each={histogram()}>
              {(b) => {
                const peak = peakBucket()
                const heightPct = peak > 0 ? (b.total / peak) * 100 : 0
                const sentPct = b.total > 0 ? (b.sent / b.total) * 100 : 0
                const failedPct =
                  b.total > 0 ? (b.failed / b.total) * 100 : 0
                return (
                  <div
                    class="group relative flex flex-1 flex-col justify-end"
                    title={tpl(NOTIFICATIONS.page.histogram_tooltip, {
                      ts: formatShanghaiDateTime(b.bucket_start),
                      total: b.total,
                      sent: b.sent,
                      failed: b.failed,
                      skipped: b.skipped,
                    })}
                  >
                    <div
                      class="flex w-full flex-col-reverse overflow-hidden rounded-sm bg-white/[0.04]"
                      style={{ height: `${heightPct}%`, "min-height": b.total > 0 ? "2px" : "0" }}
                    >
                      <div
                        class="bg-emerald-500/70"
                        style={{ height: `${sentPct}%` }}
                      />
                      <div
                        class="bg-rose-500/70"
                        style={{ height: `${failedPct}%` }}
                      />
                      <div class="flex-1 bg-[color:var(--text-muted)]/30" />
                    </div>
                  </div>
                )
              }}
            </For>
          </div>
          <div class="mt-1 flex items-center justify-between text-[9px] text-[color:var(--text-muted)]">
            <span>
              <Show when={histogram().length > 0}>
                {bucketHourLabel(histogram()[0].bucket_start)}{NOTIFICATIONS.page.histogram_hour_suffix}
              </Show>
            </span>
            <span class="flex items-center gap-3">
              <Legend color="bg-emerald-500/70" label={NOTIFICATIONS.page.legend_sent} />
              <Legend color="bg-rose-500/70" label={NOTIFICATIONS.page.legend_failed} />
              <Legend color="bg-[color:var(--text-muted)]/30" label={NOTIFICATIONS.page.legend_skipped} />
            </span>
            <span>
              <Show when={histogram().length > 0}>
                {bucketHourLabel(
                  histogram()[histogram().length - 1].bucket_start,
                )}
                {NOTIFICATIONS.page.histogram_hour_suffix}
              </Show>
            </span>
          </div>
        </div>
      </section>

      {/* 推送列表 */}
      <section class="flex min-h-0 flex-col gap-2">
        <div class="flex items-center justify-between text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">
          <span>{tpl(NOTIFICATIONS.page.list_caption, { limit: limit() })}</span>
          <span>{tpl(NOTIFICATIONS.page.list_count, { count: records().length })}</span>
        </div>
        <div class="flex-1 overflow-auto rounded border border-[color:var(--border)]">
          <table class="w-full text-xs">
            <thead class="sticky top-0 bg-[color:var(--panel)] text-[color:var(--text-muted)]">
              <tr>
                <th class="px-3 py-2 text-left font-normal">{NOTIFICATIONS.page.col_time}</th>
                <th class="px-3 py-2 text-left font-normal">{NOTIFICATIONS.page.col_user}</th>
                <th class="px-3 py-2 text-left font-normal">{NOTIFICATIONS.page.col_channel}</th>
                <th class="px-3 py-2 text-left font-normal">{NOTIFICATIONS.page.col_event}</th>
                <th class="px-3 py-2 text-left font-normal">{NOTIFICATIONS.page.col_job}</th>
                <th class="px-3 py-2 text-left font-normal">{NOTIFICATIONS.page.col_send_status}</th>
                <th class="px-3 py-2 text-left font-normal">{NOTIFICATIONS.page.col_summary}</th>
              </tr>
            </thead>
            <tbody>
              <Show
                when={records().length > 0}
                fallback={
                  <tr>
                    <td
                      colspan={7}
                      class="px-3 py-8 text-center text-[color:var(--text-muted)]"
                    >
                      {NOTIFICATIONS.page.empty_records}
                    </td>
                  </tr>
                }
              >
                <For each={records()}>
                  {(r) => (
                    <tr
                      class="cursor-pointer border-t border-[color:var(--border)] hover:bg-white/[0.03]"
                      onClick={() => setOpenRecord(r)}
                    >
                      <td
                        class="whitespace-nowrap px-3 py-2 font-mono text-[11px] text-[color:var(--text-muted)]"
                        title={r.executed_at}
                      >
                        {formatShanghaiDateTime(r.executed_at)}
                      </td>
                      <td class="px-3 py-2 font-mono text-[11px]">
                        {r.user_id}
                        <Show when={r.channel_scope}>
                          <span class="ml-1 text-[10px] text-[color:var(--text-muted)]">
                            {r.channel_scope}
                          </span>
                        </Show>
                      </td>
                      <td class="px-3 py-2 text-[11px] text-[color:var(--text-secondary)]">
                        {r.channel}
                        <div class="font-mono text-[10px] text-[color:var(--text-muted)]">
                          {r.channel_target}
                        </div>
                      </td>
                      <td class="px-3 py-2">
                        <span class="inline-block rounded bg-white/5 px-1.5 py-0.5 text-[10px] text-[color:var(--text-secondary)]">
                          {eventKindLabel(r.event_kind)}
                        </span>
                      </td>
                      <td class="px-3 py-2">
                        <div class="font-medium text-[color:var(--text-primary)]">
                          {r.job_name}
                        </div>
                        <div class="text-[10px] text-[color:var(--text-muted)]">
                          {recordSourceLabel(r.record_source)} ·{" "}
                          {execLabel(r.execution_status)}
                          <Show when={r.heartbeat}>
                            <span class="ml-1 rounded bg-white/5 px-1 py-[1px] text-[9px] uppercase">
                              heartbeat
                            </span>
                          </Show>
                        </div>
                      </td>
                      <td class="px-3 py-2">
                        <span
                          class={`inline-block rounded px-1.5 py-0.5 text-[10px] ${sendBadgeClass(r.message_send_status)}`}
                        >
                          {sendLabel(r.message_send_status)}
                        </span>
                      </td>
                      <td class="max-w-[28rem] px-3 py-2">
                        <Show when={r.response_preview}>
                          <div class="line-clamp-2 break-words text-[color:var(--text-secondary)]">
                            {r.response_preview}
                          </div>
                        </Show>
                        <Show when={r.error_message}>
                          <div class="line-clamp-2 break-words text-rose-300/80">
                            {r.error_message}
                          </div>
                        </Show>
                      </td>
                    </tr>
                  )}
                </For>
              </Show>
            </tbody>
          </table>
        </div>
      </section>

      <Show when={openRecord()}>
        {(rec) => (
          <RecordDrawer record={rec()} onClose={() => setOpenRecord(null)} />
        )}
      </Show>
    </div>
  )
}

function SummaryCard(props: {
  label: string
  value: number
  tone?: "ok" | "bad" | "warn" | "muted"
}) {
  const valueColor =
    props.tone === "ok"
      ? "text-emerald-300"
      : props.tone === "bad"
        ? "text-rose-300"
        : props.tone === "warn"
          ? "text-amber-300"
          : props.tone === "muted"
            ? "text-[color:var(--text-muted)]"
            : "text-[color:var(--text-primary)]"
  return (
    <div class="rounded border border-[color:var(--border)] p-3">
      <div class="text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">
        {props.label}
      </div>
      <div class={`mt-1 text-2xl font-semibold ${valueColor}`}>
        {props.value}
      </div>
    </div>
  )
}

function Legend(props: { color: string; label: string }) {
  return (
    <span class="flex items-center gap-1">
      <span class={`inline-block h-2 w-2 rounded-sm ${props.color}`} />
      {props.label}
    </span>
  )
}

function RecordDrawer(props: {
  record: NotificationRecord
  onClose: () => void
}) {
  return (
    <div
      class="fixed inset-0 z-40 flex justify-end bg-black/30"
      onClick={props.onClose}
    >
      <div
        class="hf-scrollbar h-full w-full max-w-xl overflow-y-auto border-l border-[color:var(--border)] bg-[color:var(--panel)] p-5 text-sm"
        onClick={(e) => e.stopPropagation()}
      >
        <div class="flex items-start justify-between">
          <div>
            <div class="text-base font-semibold">{props.record.job_name}</div>
            <div class="text-[11px] text-[color:var(--text-muted)]">
              {formatShanghaiDateTime(props.record.executed_at)}
            </div>
          </div>
          <button
            onClick={props.onClose}
            class="rounded border border-[color:var(--border)] px-2 py-1 text-xs hover:bg-white/5"
          >
            {NOTIFICATIONS.page.drawer_close}
          </button>
        </div>

        <dl class="mt-4 grid grid-cols-3 gap-x-3 gap-y-2 text-[12px]">
          <DetailItem label={NOTIFICATIONS.page.drawer_label_source} value={recordSourceLabel(props.record.record_source)} />
          <DetailItem label={NOTIFICATIONS.page.drawer_label_event_kind} value={eventKindLabel(props.record.event_kind)} />
          <DetailItem label={NOTIFICATIONS.page.drawer_label_user} value={props.record.user_id} />
          <DetailItem label={NOTIFICATIONS.page.drawer_label_channel} value={props.record.channel} />
          <DetailItem
            label={NOTIFICATIONS.page.drawer_label_channel_scope}
            value={props.record.channel_scope ?? "—"}
          />
          <DetailItem label={NOTIFICATIONS.page.drawer_label_target} value={props.record.channel_target} />
          <DetailItem
            label={NOTIFICATIONS.page.drawer_label_exec_status}
            value={execLabel(props.record.execution_status)}
          />
          <DetailItem
            label={NOTIFICATIONS.page.drawer_label_send_status}
            value={sendLabel(props.record.message_send_status)}
          />
          <DetailItem
            label={NOTIFICATIONS.page.drawer_label_should_deliver}
            value={String(props.record.should_deliver)}
          />
          <DetailItem
            label={NOTIFICATIONS.page.drawer_label_delivered}
            value={String(props.record.delivered)}
          />
          <DetailItem label={NOTIFICATIONS.page.drawer_label_job_id} value={props.record.job_id} />
        </dl>

        <Show when={props.record.response_preview}>
          <div class="mt-4">
            <div class="text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">
              {NOTIFICATIONS.page.drawer_response_preview}
            </div>
            <pre class="mt-1 whitespace-pre-wrap rounded border border-[color:var(--border)] bg-black/20 p-2 text-[12px]">
              {props.record.response_preview}
            </pre>
          </div>
        </Show>

        <Show when={props.record.error_message}>
          <div class="mt-4">
            <div class="text-[10px] uppercase tracking-widest text-rose-300/80">
              {NOTIFICATIONS.page.drawer_error}
            </div>
            <pre class="mt-1 whitespace-pre-wrap rounded border border-rose-500/40 bg-rose-500/10 p-2 text-[12px] text-rose-200">
              {props.record.error_message}
            </pre>
          </div>
        </Show>

        <Show
          when={
            props.record.detail &&
            JSON.stringify(props.record.detail) !== "null" &&
            JSON.stringify(props.record.detail) !== "{}"
          }
        >
          <div class="mt-4">
            <div class="text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">
              {NOTIFICATIONS.page.drawer_detail}
            </div>
            <pre class="mt-1 whitespace-pre-wrap rounded border border-[color:var(--border)] bg-black/20 p-2 text-[11px] text-[color:var(--text-secondary)]">
              {JSON.stringify(props.record.detail, null, 2)}
            </pre>
          </div>
        </Show>
      </div>
    </div>
  )
}

function DetailItem(props: { label: string; value: string }) {
  return (
    <>
      <dt class="col-span-1 text-[color:var(--text-muted)]">{props.label}</dt>
      <dd class="col-span-2 break-words font-mono text-[11px]">
        {props.value}
      </dd>
    </>
  )
}
