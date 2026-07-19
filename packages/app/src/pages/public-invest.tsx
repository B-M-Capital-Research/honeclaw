// public-invest.tsx — 「投资」工作台页：持仓、投资主线、整体风格、公司画像与
// 即将到来的持仓财报。数据来自 digest-context 与财经日历；编辑一律回到 /chat 与
// Agent 对话完成，本页保持只读 + 一键刷新。

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
import { defaultFinanceCalendarMonth } from "@/lib/finance-calendar"
import { workspaceUserName } from "@/lib/public-agent-workspace"
import type { FinanceCalendarEvent, PublicAuthUserInfo } from "@/lib/types"
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

/* 即将到来的持仓财报：取当月日历中今天之后的 earnings 事件，联动跟踪页看全量。 */
function UpcomingEarnings() {
  const navigate = useNavigate()
  const [events, setEvents] = createSignal<FinanceCalendarEvent[]>([])
  const [today, setToday] = createSignal("")
  const [loading, setLoading] = createSignal(true)

  onMount(async () => {
    try {
      const calendar = await getPublicFinanceCalendar(defaultFinanceCalendarMonth())
      setToday(calendar.today ?? "")
      setEvents(
        calendar.events
          .filter((event) => event.kind === "earnings" && event.date >= (calendar.today ?? ""))
          .sort((left, right) => left.date.localeCompare(right.date))
          .slice(0, 4),
      )
    } catch {
      setEvents([])
    } finally {
      setLoading(false)
    }
  })

  return (
    <section aria-label="即将到来的持仓财报">
      <div class="public-invest-section-head">
        <h2 class="public-mainline-heading">即将到来的持仓财报</h2>
        <button type="button" onClick={() => navigate("/portfolio")}>在跟踪中查看全部 →</button>
      </div>
      <Show when={!loading()} fallback={<div class="public-workspace-state">正在同步财经日历…</div>}>
        <Show
          when={events().length > 0}
          fallback={<div class="public-invest-earnings-empty">本月暂无持仓相关财报。把持仓告诉 HONE 后，财报会自动进入你的跟踪日历。</div>}
        >
          <div class="public-invest-earnings">
            <For each={events()}>
              {(event) => (
                <button type="button" onClick={() => navigate("/portfolio")}>
                  <time classList={{ "is-today": event.date === today() }}>
                    {event.date.slice(5).replace("-", "/")}
                  </time>
                  <div>
                    <strong>{event.title}</strong>
                    <small>{event.subtitle || "持仓相关财报"}</small>
                  </div>
                </button>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </section>
  )
}

function InvestContextView() {
  const navigate = useNavigate()
  const [digestContext, setDigestContext] = createSignal<DigestContext | null>(null)
  const [loading, setLoading] = createSignal(true)
  const [error, setError] = createSignal<string | null>(null)
  const [refreshing, setRefreshing] = createSignal(false)
  const [refreshMsg, setRefreshMsg] = createSignal<string | null>(null)
  const [modalOpen, setModalOpen] = createSignal(false)
  const [modalTicker, setModalTicker] = createSignal<string | null>(null)

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
  const mainlineCount = createMemo(() => {
    const context = digestContext()
    if (!context) return 0
    return context.holdings.filter((ticker) => mainlineHoldingCardState(context, ticker, profileTickers()).mainline).length
  })

  return (
    <div class="public-workspace-inner">
      <header class="public-workspace-page-heading">
        <div>
          <time>{new Date().toLocaleDateString("zh-CN", { year: "numeric", month: "long", day: "numeric", weekday: "short" })}</time>
          <h1>投资</h1>
          <p>持仓、投资主线与公司画像——HONE 为你持续蒸馏的研究上下文。</p>
        </div>
        <button type="button" class="public-workspace-primary-action" onClick={() => navigate("/chat")}>＋ 添加持仓</button>
      </header>

      <Show when={loading()}>
        <div class="public-workspace-state">正在整理投资上下文…</div>
      </Show>
      <Show when={error()}>
        <div class="public-mainline-notice is-error">加载失败：{error()}</div>
      </Show>

      <Show when={digestContext()}>
        {(context) => (
          <>
            {/* 概览统计 */}
            <div class="public-invest-stats" role="list">
              <div role="listitem">
                <strong>{context().holdings.length}</strong>
                <small>持仓</small>
              </div>
              <div role="listitem">
                <strong>{context().profile_list.length}</strong>
                <small>公司画像</small>
              </div>
              <div role="listitem">
                <strong>{mainlineCount()}<em>/{context().holdings.length}</em></strong>
                <small>主线覆盖</small>
              </div>
              <div role="listitem">
                <strong class="is-text">{formatPublicMainlineTimestamp(context().last_mainline_distilled_at)}</strong>
                <small>上次蒸馏</small>
              </div>
            </div>

            {/* 整体投资风格 + 刷新 */}
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

            {/* 各持仓投资主线 */}
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

            {/* 即将到来的持仓财报 */}
            <UpcomingEarnings />

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
    </div>
  )
}

export default function PublicInvestPage() {
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
          <PublicChatStartup title="正在加载投资" description="正在同步持仓、投资主线与公司画像。" />
        }
      >
        <Show when={loggedIn()} fallback={<PublicLoginForm onLogin={() => navigate("/invest")} />}>
          <PublicWorkspaceShell active="invest" userName={workspaceUserName(user()?.user_id ?? "")} searchPlaceholder="搜索持仓、主线或画像"><InvestContextView /></PublicWorkspaceShell>
        </Show>
      </Show>
    </div>
  )
}
