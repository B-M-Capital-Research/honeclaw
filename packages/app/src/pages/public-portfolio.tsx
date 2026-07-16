// public-portfolio.tsx — 用户的"投资上下文"页:展示并刷新系统蒸馏的投资主线、
// 整体投资风格和 sandbox 里的只读公司画像列表。编辑画像走 /chat 与 agent 对话(company_portrait skill)。

import { createEffect, createMemo, createSignal, For, onMount, Show } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { marked } from "marked"
import DOMPurify from "dompurify"
import { PublicChatStartup } from "@/components/public-chat-startup"
import { PublicLoginForm } from "@/components/public-login-form"
import { PublicWorkspaceShell } from "@/components/public-workspace-shell"
import {
  getDigestContext,
  getCompanyProfileMarkdown,
  refreshDigestContext,
  getPublicAuthMe,
  getPublicFinanceCalendar,
  type DigestContext,
} from "@/lib/api"
import {
  defaultFinanceCalendarMonth,
  financeCalendarMonthGrid,
  groupFinanceCalendarEvents,
  parseFinanceCalendarMonth,
} from "@/lib/finance-calendar"
import { workspaceUserName } from "@/lib/public-agent-workspace"
import type { FinanceCalendarPayload, PublicAuthUserInfo } from "@/lib/types"
import {
  mainlineHoldingCardState,
  profileInventoryRowState,
  profileTickerSet,
} from "@/lib/mainline-context-model"
import {
  canRefreshPublicMainline,
  formatPublicMainlineTimestamp,
  publicRefreshMessage,
} from "./public-portfolio-model"
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

function MainlineCard(props: {
  ticker: string
  mainline: string | undefined
  hasProfile: boolean
  onView: () => void
  isSkipped: boolean
}) {
  return (
    <div class="public-mainline-card" classList={{ "is-pending": !props.mainline }}>
      <div class="public-mainline-card-head">
        <div class="public-mainline-ticker">{props.ticker}</div>
        <Show when={props.hasProfile}>
          <button type="button" class="public-profile-view-btn" onClick={props.onView}>
            查看画像
          </button>
        </Show>
      </div>
      <Show
        when={props.mainline}
        fallback={
          <div class="public-mainline-fallback">
            <Show
              when={props.hasProfile}
              fallback={
                <>
                  <strong>暂无公司画像</strong> —— 跟 HONE 说
                  “建立 {props.ticker} 的公司画像”，立即更新或下一次自动检查后就会带上它。
                </>
              }
            >
              <strong>画像存在，但投资主线生成失败 / 跳过</strong>
              {props.isSkipped ? "（上次跳过）" : ""}—— 可立即更新重试，或等下一次自动检查。
            </Show>
          </div>
        }
      >
        <div class="public-mainline-text">{props.mainline}</div>
      </Show>
    </div>
  )
}

function ProfileModal(props: { open: boolean; ticker: string | null; onClose: () => void }) {
  const [markdown, setMarkdown] = createSignal<string | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  const fetchProfile = async (ticker: string) => {
    setLoading(true)
    setError(null)
    setMarkdown(null)
    try {
      const profile = await getCompanyProfileMarkdown(ticker)
      setMarkdown(profile.markdown)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  let lastTicker = ""
  createEffect(() => {
    const selectedTicker = props.ticker
    if (!props.open) {
      lastTicker = ""
      return
    }
    if (selectedTicker && selectedTicker !== lastTicker) {
      lastTicker = selectedTicker
      void fetchProfile(selectedTicker)
    }
  })

  const renderedHtml = () => {
    const md = markdown()
    if (!md) return ""
    const raw = marked.parse(md, { gfm: true, breaks: false }) as string
    return DOMPurify.sanitize(raw)
  }

  return (
    <Show when={props.open}>
      <div class="public-profile-modal-overlay" onClick={props.onClose}>
        <div class="public-profile-modal" onClick={(e) => e.stopPropagation()}>
          <div class="public-profile-modal-head">
            <div class="public-profile-modal-title">{props.ticker} · 公司画像（只读）</div>
            <button type="button" class="public-profile-modal-close" onClick={props.onClose}>
              ×
            </button>
          </div>
          <div class="public-profile-modal-body">
            <Show when={loading()}>
              <div class="public-profile-modal-muted">加载中…</div>
            </Show>
            <Show when={error()}>
              <div class="public-profile-modal-error">{error()}</div>
            </Show>
            <Show when={markdown() && !loading()}>
              <div class="profile-md" innerHTML={renderedHtml()}></div>
            </Show>
          </div>
          <div class="public-profile-modal-foot">
            画像由 HONE 维护。如需修改，请回到对话页跟 HONE 说一声。
          </div>
        </div>
      </div>
    </Show>
  )
}

function PortfolioContextView() {
  const navigate = useNavigate()
  const [digestContext, setDigestContext] = createSignal<DigestContext | null>(null)
  const [loading, setLoading] = createSignal(true)
  const [error, setError] = createSignal<string | null>(null)
  const [refreshing, setRefreshing] = createSignal(false)
  const [refreshMsg, setRefreshMsg] = createSignal<string | null>(null)
  const [modalOpen, setModalOpen] = createSignal(false)
  const [modalTicker, setModalTicker] = createSignal<string | null>(null)
  const [trackingView, setTrackingView] = createSignal<TrackingView>("calendar")

  const load = async () => {
    setLoading(true)
    setError(null)
    try {
      const context = await getDigestContext()
      setDigestContext(context)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(load)

  const handleRefresh = async () => {
    setRefreshing(true)
    setRefreshMsg(null)
    try {
      const refreshResult = await refreshDigestContext()
      setRefreshMsg(publicRefreshMessage(refreshResult))
      await load()
    } catch (e) {
      setRefreshMsg(`更新失败：${e instanceof Error ? e.message : String(e)}`)
    } finally {
      setRefreshing(false)
    }
  }

  const openProfile = (ticker: string) => {
    setModalTicker(ticker)
    setModalOpen(true)
  }

  const profileTickers = createMemo(() => profileTickerSet(digestContext()))

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
        <section class="public-tracking-context">
          <h2>投资主线与公司画像</h2>

        <Show when={loading()}>
          <div class="public-workspace-state">加载中…</div>
        </Show>
        <Show when={error()}>
          <div class="public-mainline-notice is-error">加载失败：{error()}</div>
        </Show>

        <Show when={digestContext()}>
          {(context) => (
            <>
              {/* Meta + 操作 */}
              <div class="public-mainline-meta">
                <div class="public-mainline-meta-info">
                  上次更新：<strong>{formatPublicMainlineTimestamp(context().last_mainline_distilled_at)}</strong>
                  <Show when={context().mainline_distill_skipped.length > 0}>
                    <span class="public-mainline-skipped">
                      跳过 {context().mainline_distill_skipped.length} 只：
                      <span>{context().mainline_distill_skipped.join(", ")}</span>
                    </span>
                  </Show>
                </div>
                <button
                  type="button"
                  class="public-mainline-refresh"
                  onClick={handleRefresh}
                  disabled={refreshing() || !canRefreshPublicMainline(context().profile_list.length)}
                  title={
                    !canRefreshPublicMainline(context().profile_list.length)
                      ? "先建立至少 1 个公司画像才能更新"
                      : ""
                  }
                >
                  {refreshing() ? "更新中…" : "立即更新"}
                </button>
              </div>
              <Show when={refreshMsg()}>
                <div class="public-mainline-notice is-success">{refreshMsg()}</div>
              </Show>

              {/* 整体投资风格 */}
              <div class="public-mainline-style-card">
                <div class="public-mainline-style-label">整体投资风格</div>
                <div class="public-mainline-style-body">
                  <Show
                    when={context().mainline_style}
                    fallback={<span class="public-mainline-empty-text">暂无数据 —— 需要先建立至少 1 个公司画像。</span>}
                  >
                    {context().mainline_style}
                  </Show>
                </div>
              </div>

              {/* Per-ticker mainline */}
              <h2 class="public-mainline-heading">各持仓投资主线（{context().holdings.length} 只）</h2>
              <Show
                when={context().holdings.length > 0}
                fallback={
                  <div class="public-mainline-empty">
                    <span>暂无持仓。跟 HONE 说一声你持有什么就行。</span>
                    <button type="button" onClick={() => navigate("/chat")}>去对话 →</button>
                  </div>
                }
              >
                <div class="public-mainline-grid">
                  <For each={context().holdings}>
                    {(ticker) => {
                      const card = createMemo(() =>
                        mainlineHoldingCardState(context(), ticker, profileTickers()),
                      )
                      return (
                        <MainlineCard
                          ticker={card().ticker}
                          mainline={card().mainline}
                          hasProfile={card().hasProfile}
                          isSkipped={card().isSkipped}
                          onView={() => openProfile(ticker)}
                        />
                      )
                    }}
                  </For>
                </div>
              </Show>

              {/* 公司画像列表 */}
              <h2 class="public-mainline-heading">公司画像 ({context().profile_list.length})</h2>
              <Show
                when={context().profile_list.length > 0}
                fallback={
                  <div class="public-mainline-empty">
                    <span>还没有公司画像。跟 HONE 说「建立 X 的公司画像」就能开始。</span>
                    <button type="button" onClick={() => navigate("/chat")}>去对话 →</button>
                  </div>
                }
              >
                <div class="public-profile-list">
                  <For each={context().profile_list}>
                    {(profile) => {
                      const row = createMemo(() => profileInventoryRowState(profile))
                      return (
                        <div class="public-profile-row">
                          <div class="public-profile-row-main">
                            <div class="public-profile-title">
                              {row().title}
                              <span class="public-profile-ticker">{row().tickerLabel}</span>
                            </div>
                            <div class="public-profile-sub">
                              {row().sizeLabel} · {row().dir}
                            </div>
                          </div>
                          <Show when={row().viewTicker}>
                            {(ticker) => (
                              <button type="button" class="public-profile-view-btn" onClick={() => openProfile(ticker())}>
                                查看
                              </button>
                            )}
                          </Show>
                        </div>
                      )
                    }}
                  </For>
                </div>
              </Show>

              <ProfileModal
                open={modalOpen()}
                ticker={modalTicker()}
                onClose={() => {
                  setModalOpen(false)
                  setModalTicker(null)
                }}
              />
            </>
          )}
        </Show>
        </section>
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
