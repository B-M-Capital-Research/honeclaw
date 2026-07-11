// public-portfolio.tsx — 用户的"投资上下文"页:展示并刷新系统蒸馏的投资主线、
// 整体投资风格和 sandbox 里的只读公司画像列表。编辑画像走 /chat 与 agent 对话(company_portrait skill)。

import { createEffect, createMemo, createSignal, For, onMount, Show } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { marked } from "marked"
import DOMPurify from "dompurify"
import { PublicNav, PublicFooter } from "@/components/public-nav"
import { PublicLoginForm } from "@/components/public-login-form"
import {
  getDigestContext,
  getCompanyProfileMarkdown,
  refreshDigestContext,
  getPublicAuthMe,
  type DigestContext,
} from "@/lib/api"
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
    <div
      style={{
        padding: "20px 22px",
        "border-radius": "12px",
        border: props.mainline
          ? "1px solid rgba(0,0,0,0.08)"
          : "1px dashed rgba(245,158,11,0.30)",
        background: props.mainline ? "#fff" : "rgba(245,158,11,0.04)",
        display: "flex",
        "flex-direction": "column",
        gap: "10px",
      }}
    >
      <div style={{ display: "flex", "align-items": "baseline", "justify-content": "space-between", gap: "10px" }}>
        <div
          style={{
            "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
            "font-size": "16px",
            "font-weight": "700",
            color: "#0f172a",
          }}
        >
          {props.ticker}
        </div>
        <Show when={props.hasProfile}>
          <button
            type="button"
            onClick={props.onView}
            style={{
              "font-size": "12px",
              padding: "4px 10px",
              "border-radius": "6px",
              border: "1px solid rgba(0,0,0,0.10)",
              background: "#fff",
              color: "#475569",
              cursor: "pointer",
              "font-family": "inherit",
            }}
          >
            查看画像
          </button>
        </Show>
      </div>
      <Show
        when={props.mainline}
        fallback={
          <div style={{ "font-size": "13px", color: "#94a3b8", "line-height": "1.6" }}>
            <Show
              when={props.hasProfile}
              fallback={
                <>
                  <strong style={{ color: "#d97706" }}>暂无公司画像</strong> —— 跟 HONE 说
                  “建立 {props.ticker} 的公司画像”，立即更新或下一次自动检查后就会带上它。
                </>
              }
            >
              <strong style={{ color: "#d97706" }}>画像存在，但投资主线生成失败 / 跳过</strong>
              {props.isSkipped ? "（上次跳过）" : ""}—— 可立即更新重试，或等下一次自动检查。
            </Show>
          </div>
        }
      >
        <div
          style={{
            "font-size": "14px",
            color: "#0f172a",
            "line-height": "1.7",
          }}
        >
          {props.mainline}
        </div>
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
      <div
        style={{
          position: "fixed",
          inset: "0",
          background: "rgba(15,23,42,0.5)",
          display: "flex",
          "align-items": "center",
          "justify-content": "center",
          "z-index": "1000",
          padding: "32px",
        }}
        onClick={props.onClose}
      >
        <div
          style={{
            "max-width": "780px",
            "max-height": "90vh",
            width: "100%",
            background: "#fff",
            "border-radius": "12px",
            display: "flex",
            "flex-direction": "column",
            overflow: "hidden",
          }}
          onClick={(e) => e.stopPropagation()}
        >
          <div
            style={{
              padding: "16px 20px",
              "border-bottom": "1px solid rgba(0,0,0,0.08)",
              display: "flex",
              "align-items": "center",
              "justify-content": "space-between",
            }}
          >
            <div style={{ "font-weight": "700", color: "#0f172a" }}>
              {props.ticker} · 公司画像（只读）
            </div>
            <button
              type="button"
              onClick={props.onClose}
              style={{
                background: "transparent",
                border: "none",
                cursor: "pointer",
                "font-size": "20px",
                color: "#94a3b8",
              }}
            >
              ×
            </button>
          </div>
          <div
            style={{
              padding: "20px 28px",
              overflow: "auto",
              "font-size": "14px",
              "line-height": "1.7",
              color: "#0f172a",
            }}
          >
            <Show when={loading()}>
              <div style={{ color: "#94a3b8" }}>加载中…</div>
            </Show>
            <Show when={error()}>
              <div style={{ color: "#dc2626" }}>{error()}</div>
            </Show>
            <Show when={markdown() && !loading()}>
              <div class="profile-md" innerHTML={renderedHtml()}></div>
            </Show>
          </div>
          <div
            style={{
              padding: "12px 20px",
              "border-top": "1px solid rgba(0,0,0,0.06)",
              "background": "#f8fafc",
              "font-size": "12px",
              color: "#64748b",
            }}
          >
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
    <div style={{ "padding-top": "56px", "min-height": "100vh", background: "#f8fafc" }}>
      <div style={{ "max-width": "920px", margin: "0 auto", padding: "48px 32px" }}>
        <div style={{ "margin-bottom": "32px" }}>
          <h1
            style={{
              "font-size": "28px",
              "font-weight": "700",
              color: "#0f172a",
              margin: "0",
              "letter-spacing": "-0.01em",
            }}
          >
            投资上下文
          </h1>
          <p style={{ "font-size": "13px", color: "#64748b", "margin-top": "8px", "line-height": "1.7" }}>
            HONE 会从你的公司画像里整理投资主线；新增画像或持仓后通常会在下一次自动检查里尝试更新，覆盖完整后约每周刷新一次。要修改画像，直接跟 HONE 对话即可。
          </p>
        </div>

        <Show when={loading()}>
          <div style={{ color: "#94a3b8", "padding": "40px 0", "text-align": "center" }}>加载中…</div>
        </Show>
        <Show when={error()}>
          <div
            style={{
              "background": "rgba(220,38,38,0.06)",
              "border": "1px solid rgba(220,38,38,0.20)",
              "border-radius": "10px",
              padding: "16px 20px",
              color: "#b91c1c",
              "font-size": "13px",
              "margin-bottom": "16px",
            }}
          >
            加载失败：{error()}
          </div>
        </Show>

        <Show when={digestContext()}>
          {(context) => (
            <>
              {/* Meta + 操作 */}
              <div
                style={{
                  display: "flex",
                  "align-items": "center",
                  "justify-content": "space-between",
                  "margin-bottom": "24px",
                  "flex-wrap": "wrap",
                  gap: "12px",
                }}
              >
                <div style={{ "font-size": "13px", color: "#64748b" }}>
                  上次更新：<strong style={{ color: "#0f172a" }}>{formatPublicMainlineTimestamp(context().last_mainline_distilled_at)}</strong>
                  <Show when={context().mainline_distill_skipped.length > 0}>
                    <span style={{ "margin-left": "16px" }}>
                      跳过 {context().mainline_distill_skipped.length} 只：
                      <span style={{ color: "#d97706", "font-family": "monospace" }}>
                        {context().mainline_distill_skipped.join(", ")}
                      </span>
                    </span>
                  </Show>
                </div>
                <button
                  type="button"
                  onClick={handleRefresh}
                  disabled={refreshing() || !canRefreshPublicMainline(context().profile_list.length)}
                  style={{
                    padding: "8px 16px",
                    "border-radius": "8px",
                    border: "1px solid #f59e0b",
                    background:
                      refreshing() || !canRefreshPublicMainline(context().profile_list.length)
                        ? "rgba(245,158,11,0.4)"
                        : "#f59e0b",
                    color: "#fff",
                    cursor:
                      refreshing() || !canRefreshPublicMainline(context().profile_list.length)
                        ? "not-allowed"
                        : "pointer",
                    "font-family": "inherit",
                    "font-size": "13px",
                    "font-weight": "600",
                  }}
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
                <div
                  style={{
                    "background": "rgba(34,197,94,0.06)",
                    "border": "1px solid rgba(34,197,94,0.20)",
                    "border-radius": "8px",
                    padding: "10px 14px",
                    color: "#15803d",
                    "font-size": "13px",
                    "margin-bottom": "16px",
                  }}
                >
                  {refreshMsg()}
                </div>
              </Show>

              {/* 整体投资风格 */}
              <div
                style={{
                  padding: "20px 24px",
                  "border-radius": "12px",
                  border: "1px solid rgba(0,0,0,0.08)",
                  background: "#fff",
                  "margin-bottom": "24px",
                }}
              >
                <div
                  style={{
                    "font-size": "11px",
                    "font-weight": "700",
                    "letter-spacing": "0.15em",
                    "text-transform": "uppercase",
                    color: "#94a3b8",
                    "margin-bottom": "8px",
                  }}
                >
                  整体投资风格
                </div>
                <div style={{ "font-size": "14px", color: "#0f172a", "line-height": "1.7" }}>
                  <Show
                    when={context().mainline_style}
                    fallback={
                      <span style={{ color: "#94a3b8" }}>
                        暂无数据 —— 需要先建立至少 1 个公司画像。
                      </span>
                    }
                  >
                    {context().mainline_style}
                  </Show>
                </div>
              </div>

              {/* Per-ticker mainline */}
              <h2
                style={{
                  "font-size": "16px",
                  "font-weight": "700",
                  color: "#0f172a",
                  margin: "24px 0 12px",
                }}
              >
                各持仓投资主线（{context().holdings.length} 只）
              </h2>
              <Show
                when={context().holdings.length > 0}
                fallback={
                  <div
                    style={{
                      padding: "32px",
                      "border-radius": "10px",
                      background: "#fff",
                      "text-align": "center",
                      color: "#94a3b8",
                      "font-size": "13px",
                      display: "flex",
                      "flex-direction": "column",
                      "align-items": "center",
                      gap: "14px",
                    }}
                  >
                    <span>暂无持仓。跟 HONE 说一声你持有什么就行。</span>
                    <button
                      type="button"
                      onClick={() => navigate("/chat")}
                      style={{
                        padding: "8px 18px",
                        "border-radius": "999px",
                        background: "#0f172a",
                        color: "#fff",
                        border: "none",
                        "font-family": "inherit",
                        "font-size": "13px",
                        "font-weight": "600",
                        cursor: "pointer",
                      }}
                    >
                      去对话 →
                    </button>
                  </div>
                }
              >
                <div
                  style={{
                    display: "grid",
                    "grid-template-columns": "repeat(auto-fill, minmax(280px, 1fr))",
                    gap: "14px",
                    "margin-bottom": "32px",
                  }}
                >
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
              <h2
                style={{
                  "font-size": "16px",
                  "font-weight": "700",
                  color: "#0f172a",
                  margin: "32px 0 12px",
                }}
              >
                公司画像 ({context().profile_list.length})
              </h2>
              <Show
                when={context().profile_list.length > 0}
                fallback={
                  <div
                    style={{
                      padding: "32px",
                      "border-radius": "10px",
                      background: "#fff",
                      "text-align": "center",
                      color: "#94a3b8",
                      "font-size": "13px",
                      display: "flex",
                      "flex-direction": "column",
                      "align-items": "center",
                      gap: "14px",
                    }}
                  >
                    <span>还没有公司画像。跟 HONE 说「建立 X 的公司画像」就能开始。</span>
                    <button
                      type="button"
                      onClick={() => navigate("/chat")}
                      style={{
                        padding: "8px 18px",
                        "border-radius": "999px",
                        background: "#0f172a",
                        color: "#fff",
                        border: "none",
                        "font-family": "inherit",
                        "font-size": "13px",
                        "font-weight": "600",
                        cursor: "pointer",
                      }}
                    >
                      去对话 →
                    </button>
                  </div>
                }
              >
                <div style={{ display: "flex", "flex-direction": "column", gap: "10px" }}>
                  <For each={context().profile_list}>
                    {(profile) => {
                      const row = createMemo(() => profileInventoryRowState(profile))
                      return (
                        <div
                          style={{
                            padding: "14px 18px",
                            "border-radius": "10px",
                            background: "#fff",
                            border: "1px solid rgba(0,0,0,0.06)",
                            display: "flex",
                            "align-items": "center",
                            "justify-content": "space-between",
                            gap: "12px",
                          }}
                        >
                          <div style={{ flex: "1" }}>
                            <div style={{ "font-size": "14px", "font-weight": "600", color: "#0f172a" }}>
                              {row().title}
                              <span
                                style={{
                                  "margin-left": "8px",
                                  "font-family": "monospace",
                                  "font-size": "12px",
                                  color: "#64748b",
                                }}
                              >
                                {row().tickerLabel}
                              </span>
                            </div>
                            <div
                              style={{
                                "font-size": "12px",
                                color: "#94a3b8",
                                "margin-top": "4px",
                              }}
                            >
                              {row().sizeLabel} · {row().dir}
                            </div>
                          </div>
                          <Show when={row().viewTicker}>
                            {(ticker) => (
                              <button
                                type="button"
                                onClick={() => openProfile(ticker())}
                                style={{
                                  padding: "6px 12px",
                                  "border-radius": "6px",
                                  border: "1px solid rgba(0,0,0,0.10)",
                                  background: "#fff",
                                  color: "#475569",
                                  cursor: "pointer",
                                  "font-family": "inherit",
                                  "font-size": "12px",
                                }}
                              >
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
    </div>
  )
}

export default function PublicPortfolioPage() {
  const navigate = useNavigate()
  const [loggedIn, setLoggedIn] = createSignal<boolean | null>(null)

  onMount(async () => {
    try {
      await getPublicAuthMe()
      setLoggedIn(true)
    } catch {
      setLoggedIn(false)
    }
  })

  return (
    <div
      class="pub-page"
      style={{
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
        "-webkit-font-smoothing": "antialiased",
      }}
    >
      <PublicNav />
      <Show
        when={loggedIn() !== null}
        fallback={
          <div
            style={{
              "padding-top": "56px",
              "min-height": "100vh",
              background: "#f8fafc",
              display: "flex",
              "align-items": "center",
              "justify-content": "center",
            }}
          >
            <div style={{ "font-size": "13px", color: "#94a3b8" }}>检查登录…</div>
          </div>
        }
      >
        <Show when={loggedIn()} fallback={<PublicLoginForm onLogin={() => navigate("/portfolio")} />}>
          <PortfolioContextView />
        </Show>
      </Show>
      <PublicFooter />
    </div>
  )
}
