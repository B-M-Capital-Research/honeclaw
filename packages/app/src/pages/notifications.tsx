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

// ── 状态映射(对齐 task-detail.tsx 的 sendStatusLabel/executionStatusLabel) ──

const SEND_STATUS_OPTIONS: Array<{ value: string; label: string }> = [
  { value: "", label: "全部发送状态" },
  { value: "sent", label: "已发送" },
  { value: "dryrun", label: "Dry run" },
  { value: "queued", label: "已排队" },
  { value: "quiet_held", label: "静音暂存" },
  { value: "filtered", label: "偏好过滤" },
  { value: "capped", label: "上限降级" },
  { value: "cooled_down", label: "冷却降级" },
  { value: "price_capped", label: "价格上限降级" },
  { value: "price_cooled_down", label: "价格冷却降级" },
  { value: "omitted", label: "摘要省略" },
  { value: "skipped_noop", label: "未发送(未命中/排队)" },
  { value: "skipped_error", label: "未发送(执行失败)" },
  { value: "send_failed", label: "发送失败" },
  { value: "failed", label: "发送失败(event)" },
  { value: "target_resolution_failed", label: "目标解析失败" },
  { value: "duplicate_suppressed", label: "已拦截重复发送" },
]

const EXEC_STATUS_OPTIONS: Array<{ value: string; label: string }> = [
  { value: "", label: "全部执行状态" },
  { value: "completed", label: "执行成功" },
  { value: "noop", label: "未命中" },
  { value: "execution_failed", label: "执行失败" },
]

const CHANNEL_OPTIONS: Array<{ value: string; label: string }> = [
  { value: "", label: "全部渠道" },
  { value: "telegram", label: "Telegram" },
  { value: "discord", label: "Discord" },
  { value: "feishu", label: "飞书" },
  { value: "imessage", label: "iMessage" },
]

function sendLabel(s: string): string {
  return SEND_STATUS_OPTIONS.find((o) => o.value === s)?.label ?? s ?? "—"
}
function execLabel(s: string): string {
  return EXEC_STATUS_OPTIONS.find((o) => o.value === s)?.label ?? s ?? "—"
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
  return d.toLocaleString("zh-CN", {
    timeZone: "Asia/Shanghai",
    hour: "2-digit",
    hour12: false,
  })
}

function recordSourceLabel(source: string): string {
  switch (source) {
    case "cron_job":
      return "cron"
    case "event_engine":
      return "event"
    default:
      return source || "—"
  }
}

function eventKindLabel(kind?: string | null): string {
  switch (kind) {
    case "earnings_upcoming":
      return "财报预告"
    case "earnings_released":
      return "财报发布"
    case "earnings_call_transcript":
      return "财报电话会"
    case "news_critical":
      return "重点新闻"
    case "price_alert":
      return "价格异动"
    case "weekly52_high":
      return "52周新高"
    case "weekly52_low":
      return "52周新低"
    case "dividend":
      return "分红"
    case "split":
      return "拆股"
    case "sec_filing":
      return "SEC 文件"
    case "analyst_grade":
      return "分析师评级"
    case "macro_event":
      return "宏观事件"
    case "social_post":
      return "社媒动态"
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
          推送日志
        </h1>
        <div class="flex items-center gap-1 text-xs text-[color:var(--text-muted)]">
          <span>窗口</span>
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
          <span>渠道</span>
          <select
            value={channel()}
            disabled={!!selectedActor()}
            onChange={(e) => {
              setChannel(e.currentTarget.value)
              void refresh()
            }}
            class="rounded border border-[color:var(--border)] bg-transparent px-2 py-1 text-xs text-[color:var(--text-primary)] disabled:opacity-50"
          >
            <For each={CHANNEL_OPTIONS}>
              {(o) => <option value={o.value}>{o.label}</option>}
            </For>
          </select>
        </div>
        <div class="flex items-center gap-1 text-xs text-[color:var(--text-muted)]">
          <span>用户</span>
          <ActorSelect
            allowAll
            allLabel="全部用户"
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
            <For each={SEND_STATUS_OPTIONS}>
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
            <For each={EXEC_STATUS_OPTIONS}>
              {(o) => <option value={o.value}>{o.label}</option>}
            </For>
          </select>
        </div>
        <button
          onClick={() => void refresh()}
          disabled={loading()}
          class="rounded border border-[color:var(--border)] px-3 py-1 text-xs hover:bg-white/5 disabled:opacity-50"
        >
          {loading() ? "刷新中…" : "刷新"}
        </button>
      </div>

      <Show when={err()}>
        <div class="rounded border border-rose-500/40 bg-rose-500/10 p-3 text-rose-300">
          {err()}
        </div>
      </Show>

      {/* 24h 汇总数字 */}
      <section class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
        <SummaryCard label="24h 总数" value={summary().total} />
        <SummaryCard label="已发送" value={summary().sent} tone="ok" />
        <SummaryCard label="发送失败" value={summary().failed} tone="bad" />
        <SummaryCard label="主动跳过" value={summary().skipped} tone="muted" />
        <SummaryCard
          label="重复拦截"
          value={summary().duplicate_suppressed}
          tone="warn"
        />
        <SummaryCard label="覆盖用户" value={summary().distinct_users} />
      </section>

      {/* 24h 直方图 */}
      <section class="space-y-2">
        <div class="text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">
          24h 推送频率(每小时一桶,左→右 = 旧→新)
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
                    title={`${formatShanghaiDateTime(b.bucket_start)}\n总 ${b.total} · 发送 ${b.sent} · 失败 ${b.failed} · 跳过 ${b.skipped}`}
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
                {bucketHourLabel(histogram()[0].bucket_start)}时
              </Show>
            </span>
            <span class="flex items-center gap-3">
              <Legend color="bg-emerald-500/70" label="发送" />
              <Legend color="bg-rose-500/70" label="失败" />
              <Legend color="bg-[color:var(--text-muted)]/30" label="跳过" />
            </span>
            <span>
              <Show when={histogram().length > 0}>
                {bucketHourLabel(
                  histogram()[histogram().length - 1].bucket_start,
                )}
                时
              </Show>
            </span>
          </div>
        </div>
      </section>

      {/* 推送列表 */}
      <section class="flex min-h-0 flex-col gap-2">
        <div class="flex items-center justify-between text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">
          <span>推送记录(最多 {limit()} 条,倒序)</span>
          <span>共 {records().length} 条</span>
        </div>
        <div class="flex-1 overflow-auto rounded border border-[color:var(--border)]">
          <table class="w-full text-xs">
            <thead class="sticky top-0 bg-[color:var(--panel)] text-[color:var(--text-muted)]">
              <tr>
                <th class="px-3 py-2 text-left font-normal">时间</th>
                <th class="px-3 py-2 text-left font-normal">用户</th>
                <th class="px-3 py-2 text-left font-normal">渠道</th>
                <th class="px-3 py-2 text-left font-normal">事件类型</th>
                <th class="px-3 py-2 text-left font-normal">任务</th>
                <th class="px-3 py-2 text-left font-normal">发送状态</th>
                <th class="px-3 py-2 text-left font-normal">摘要</th>
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
                      该窗口内没有匹配的推送记录。
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
            关闭
          </button>
        </div>

        <dl class="mt-4 grid grid-cols-3 gap-x-3 gap-y-2 text-[12px]">
          <DetailItem label="来源" value={recordSourceLabel(props.record.record_source)} />
          <DetailItem label="事件类型" value={eventKindLabel(props.record.event_kind)} />
          <DetailItem label="用户" value={props.record.user_id} />
          <DetailItem label="渠道" value={props.record.channel} />
          <DetailItem
            label="Channel Scope"
            value={props.record.channel_scope ?? "—"}
          />
          <DetailItem label="目标" value={props.record.channel_target} />
          <DetailItem
            label="执行状态"
            value={execLabel(props.record.execution_status)}
          />
          <DetailItem
            label="发送状态"
            value={sendLabel(props.record.message_send_status)}
          />
          <DetailItem
            label="should_deliver"
            value={String(props.record.should_deliver)}
          />
          <DetailItem
            label="delivered"
            value={String(props.record.delivered)}
          />
          <DetailItem label="job_id" value={props.record.job_id} />
        </dl>

        <Show when={props.record.response_preview}>
          <div class="mt-4">
            <div class="text-[10px] uppercase tracking-widest text-[color:var(--text-muted)]">
              响应预览
            </div>
            <pre class="mt-1 whitespace-pre-wrap rounded border border-[color:var(--border)] bg-black/20 p-2 text-[12px]">
              {props.record.response_preview}
            </pre>
          </div>
        </Show>

        <Show when={props.record.error_message}>
          <div class="mt-4">
            <div class="text-[10px] uppercase tracking-widest text-rose-300/80">
              错误
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
              detail
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
