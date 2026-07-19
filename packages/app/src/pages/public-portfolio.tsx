// public-portfolio.tsx — 「跟踪」工作台页：财经日历、持仓财报与持续任务。
// 投资主线与公司画像已迁往 /invest（public-invest.tsx）。

import { createMemo, createSignal, For, onMount, Show } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { PublicChatStartup } from "@/components/public-chat-startup"
import { PublicLoginForm } from "@/components/public-login-form"
import { PublicWorkspaceShell } from "@/components/public-workspace-shell"
import { getPublicAuthMe, getPublicFinanceCalendar } from "@/lib/api"
import {
  defaultFinanceCalendarMonth,
  financeCalendarMonthGrid,
  groupFinanceCalendarEvents,
  parseFinanceCalendarMonth,
} from "@/lib/finance-calendar"
import { workspaceUserName } from "@/lib/public-agent-workspace"
import type { FinanceCalendarPayload, PublicAuthUserInfo } from "@/lib/types"
import "./public-site.css"

const TRACKING_WEEKDAYS = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"]

type TrackingView = "today" | "calendar" | "tasks" | "history"

function TrackingCalendar(props: { view: TrackingView }) {
  const navigate = useNavigate()
  const [month, setMonth] = createSignal(defaultFinanceCalendarMonth())
  const [calendar, setCalendar] = createSignal<FinanceCalendarPayload | null>(null)
  const [loading, setLoading] = createSignal(true)
  const [error, setError] = createSignal("")
  const cells = createMemo(() => financeCalendarMonthGrid(month()))
  const eventsByDate = createMemo(() => groupFinanceCalendarEvents(calendar()?.events ?? []))
  const agenda = createMemo(() =>
    [...(calendar()?.events ?? [])].sort((left, right) => left.date.localeCompare(right.date)),
  )
  const visibleAgenda = createMemo(() => {
    const today = calendar()?.today ?? ""
    if (props.view === "today") return agenda().filter((event) => event.date === today)
    if (props.view === "history") return agenda().filter((event) => event.date < today).reverse()
    return agenda()
  })
  const monthTitle = createMemo(() => {
    const parsed = parseFinanceCalendarMonth(month())
    return parsed ? `${parsed.year} 年 ${parsed.month} 月` : month()
  })
  const load = async (value: string) => {
    setLoading(true)
    setError("")
    try {
      setCalendar(await getPublicFinanceCalendar(value))
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "财经日历暂时无法加载")
    } finally {
      setLoading(false)
    }
  }
  const shiftMonth = (delta: number) => {
    const parsed = parseFinanceCalendarMonth(month())
    if (!parsed) return
    const next = new Date(parsed.year, parsed.month - 1 + delta, 1)
    const value = `${next.getFullYear()}-${String(next.getMonth() + 1).padStart(2, "0")}`
    setMonth(value)
    void load(value)
  }
  onMount(() => void load(month()))
  return (
    <section class="public-workspace-panel public-tracking-calendar" aria-label="跟踪日历">
      <header class="public-tracking-calendar-head">
        <div><h2>{props.view === "tasks" ? "持续任务" : props.view === "today" ? "今日事件" : props.view === "history" ? "已发生事件" : monthTitle()}</h2><p>宏观事件、持仓财报与持续跟踪任务</p></div>
        <Show when={props.view === "calendar"}><div class="public-tracking-month-controls">
          <button type="button" aria-label="上一个月" onClick={() => shiftMonth(-1)}>‹</button>
          <button type="button" aria-label="下一个月" onClick={() => shiftMonth(1)}>›</button>
        </div></Show>
      </header>
      <Show when={!loading()} fallback={<div class="public-workspace-state">正在整理跟踪日历…</div>}>
        <Show when={!error()} fallback={<div class="public-workspace-state is-error">{error()}</div>}>
          <Show when={props.view === "calendar"}>
            <div class="public-tracking-weekdays"><For each={TRACKING_WEEKDAYS}>{(day) => <span>{day}</span>}</For></div>
            <div class="public-tracking-grid">
            <For each={cells()}>{(cell) => {
              const events = () => eventsByDate()[cell.date] ?? []
              return <div class="public-tracking-day" classList={{ "is-muted": !cell.inMonth, "is-today": calendar()?.today === cell.date }}>
                <strong>{cell.day}</strong>
                <For each={events().slice(0, 2)}>{(event) => <span class="public-tracking-event" classList={{ "is-earnings": event.kind === "earnings" }}>{event.title}</span>}</For>
              </div>
            }}</For>
            </div>
          </Show>
          <Show when={props.view !== "tasks"} fallback={<div class="public-tracking-task-empty"><strong>让 Agent 持续跟踪一条研究主线</strong><p>建立任务后，关键财报、宏观事件和验证节点会进入你的跟踪时间线。</p><button type="button" onClick={() => navigate("/chat")}>去建立跟踪</button></div>}>
          <div class="public-tracking-agenda" classList={{ "is-desktop-list": props.view !== "calendar" }}>
            <Show when={visibleAgenda().length > 0} fallback={<div class="public-workspace-state">{props.view === "today" ? "今天暂无重要事件" : props.view === "history" ? "暂无已发生事件" : "本月暂无重要事件"}</div>}>
              <For each={visibleAgenda()}>{(event) => <article><time>{event.date.slice(5).replace("-", "/")}</time><div><strong>{event.title}</strong><small>{event.subtitle || (event.kind === "earnings" ? "持仓相关财报" : event.source)}</small></div></article>}</For>
            </Show>
          </div>
          </Show>
        </Show>
      </Show>
    </section>
  )
}

function PortfolioContextView() {
  const navigate = useNavigate()
  const [trackingView, setTrackingView] = createSignal<TrackingView>("calendar")

  return (
    <div class="public-workspace-inner">
        <header class="public-workspace-page-heading">
          <div>
            <time>{new Date().toLocaleDateString("zh-CN", { year: "numeric", month: "long", day: "numeric", weekday: "short" })}</time>
          <h1>跟踪</h1>
          <p>把即将发生的事件、持续验证的主线和 Agent 任务放在一起。</p>
          </div>
          <button type="button" class="public-workspace-primary-action" onClick={() => navigate("/chat")}>＋ 新建跟踪</button>
        </header>
        <nav class="public-workspace-tabs" aria-label="跟踪视图"><button type="button" classList={{ "is-active": trackingView() === "today" }} onClick={() => setTrackingView("today")}>今日</button><button type="button" classList={{ "is-active": trackingView() === "calendar" }} onClick={() => setTrackingView("calendar")}>日历</button><button type="button" classList={{ "is-active": trackingView() === "tasks" }} onClick={() => setTrackingView("tasks")}>任务</button><button type="button" classList={{ "is-active": trackingView() === "history" }} onClick={() => setTrackingView("history")}>历史</button></nav>
        <TrackingCalendar view={trackingView()} />
        <div class="public-tracking-invest-link">
          <span>投资主线与公司画像已有独立页面。</span>
          <button type="button" onClick={() => navigate("/invest")}>前往投资 →</button>
        </div>
    </div>
  )
}

export default function PublicPortfolioPage() {
  const navigate = useNavigate()
  const [loggedIn, setLoggedIn] = createSignal<boolean | null>(null)
  const [user, setUser] = createSignal<PublicAuthUserInfo | null>(null)

  onMount(async () => {
    try {
      const currentUser = await getPublicAuthMe()
      setUser(currentUser)
      setLoggedIn(true)
    } catch {
      setLoggedIn(false)
    }
  })

  return (
    <div class="pub-page">
      <Show
        when={loggedIn() !== null}
        fallback={
          <PublicChatStartup title="正在加载跟踪" description="正在同步财经日历、持仓主线与持续任务。" />
        }
      >
        <Show when={loggedIn()} fallback={<PublicLoginForm onLogin={() => navigate("/portfolio")} />}>
          <PublicWorkspaceShell active="tracking" userName={workspaceUserName(user()?.user_id ?? "")} searchPlaceholder="搜索公司、事件或跟踪"><PortfolioContextView /></PublicWorkspaceShell>
        </Show>
      </Show>
    </div>
  )
}
