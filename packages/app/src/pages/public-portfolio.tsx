// public-portfolio.tsx — 用户的"投资上下文"页:展示系统蒸馏的 thesis、整体投资风格、
// sandbox 里的公司画像列表(read-only)。编辑画像走 /chat 与 agent 对话(company_portrait skill)。

import { createSignal, For, onMount, Show } from "solid-js"
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
import "./public-site.css"

function formatTimestamp(iso: string | null): string {
  if (!iso) return "尚未蒸馏"
  try {
    const dt = new Date(iso)
    const days = Math.floor((Date.now() - dt.getTime()) / (24 * 3600 * 1000))
    if (days === 0) return `今天 ${dt.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })}`
    if (days === 1) return "1 天前"
    if (days < 7) return `${days} 天前`
    return dt.toLocaleDateString("zh-CN", { year: "numeric", month: "short", day: "numeric" })
  } catch {
    return iso
  }
}

function ThesisCard(props: {
  ticker: string
  thesis: string | undefined
  hasProfile: boolean
  onView: () => void
  isSkipped: boolean
}) {
  return (
    <div
      style={{
        padding: "20px 22px",
        "border-radius": "12px",
        border: props.thesis
          ? "1px solid rgba(0,0,0,0.08)"
          : "1px dashed rgba(245,158,11,0.30)",
        background: props.thesis ? "#fff" : "rgba(245,158,11,0.04)",
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
        when={props.thesis}
        fallback={
          <div style={{ "font-size": "13px", color: "#94a3b8", "line-height": "1.6" }}>
            <Show
              when={props.hasProfile}
              fallback={
                <>
                  <strong style={{ color: "#d97706" }}>没有公司画像</strong> —— 在 chat 里对 agent 说
                  "建立 {props.ticker} 的公司画像",蒸馏会在下次 cron 自动跑。
                </>
              }
            >
              <strong style={{ color: "#d97706" }}>画像存在但 thesis 蒸馏失败 / 跳过</strong>
              {props.isSkipped ? "(上次跳过)" : ""}—— 点"立即刷新"重试。
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
          {props.thesis}
        </div>
      </Show>
    </div>
  )
}

function ProfileModal(props: { open: boolean; ticker: string | null; onClose: () => void }) {
  const [markdown, setMarkdown] = createSignal<string | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  // 当 ticker 改变时拉数据
  const fetchProfile = async (ticker: string) => {
    setLoading(true)
    setError(null)
    setMarkdown(null)
    try {
      const data = await getCompanyProfileMarkdown(ticker)
      setMarkdown(data.markdown)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  // 监听 ticker 变化(使用 createEffect 风格)
  let lastTicker = ""
  const maybeFetch = () => {
    const t = props.ticker
    if (props.open && t && t !== lastTicker) {
      lastTicker = t
      fetchProfile(t)
    }
    if (!props.open) {
      lastTicker = ""
    }
  }
  // SolidJS reactivity:每次 render 都会调用,且 props 是 reactive
  maybeFetch()

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
              {props.ticker} · 公司画像 (read-only)
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
            画像由 chat 里 company_portrait skill 维护。如需修改,请在 /chat 与 agent 对话。
          </div>
        </div>
      </div>
    </Show>
  )
}

function PortfolioContextView() {
  const [ctx, setCtx] = createSignal<DigestContext | null>(null)
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
      const data = await getDigestContext()
      setCtx(data)
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
      const r = await refreshDigestContext()
      setRefreshMsg(
        `蒸馏完成:${r.theses_count} 条 thesis,跳过 ${r.skipped_tickers.length} 只`,
      )
      await load()
    } catch (e) {
      setRefreshMsg(`蒸馏失败:${e instanceof Error ? e.message : String(e)}`)
    } finally {
      setRefreshing(false)
    }
  }

  const openProfile = (ticker: string) => {
    setModalTicker(ticker)
    setModalOpen(true)
  }

  const profileTickers = () => {
    const c = ctx()
    if (!c) return new Set<string>()
    const set = new Set<string>()
    for (const p of c.profile_list) {
      for (const t of p.tickers) set.add(t)
    }
    return set
  }

  return (
    <div style={{ "padding-top": "56px", "min-height": "100vh", background: "#f8fafc" }}>
      <div style={{ "max-width": "920px", margin: "0 auto", padding: "48px 32px" }}>
        <div style={{ "margin-bottom": "32px" }}>
          <div
            style={{
              "font-size": "11px",
              "font-weight": "700",
              "letter-spacing": "0.30em",
              "text-transform": "uppercase",
              color: "#f59e0b",
              "margin-bottom": "8px",
            }}
          >
            DIGEST CONTEXT
          </div>
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
            系统每周自动从你的公司画像蒸馏 thesis,用于过滤全球 digest 的相关性。画像编辑请通过 /chat。
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
            加载失败:{error()}
          </div>
        </Show>

        <Show when={ctx()}>
          {(c) => (
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
                  上次蒸馏:<strong style={{ color: "#0f172a" }}>{formatTimestamp(c().last_thesis_distilled_at)}</strong>
                  <Show when={c().thesis_distill_skipped.length > 0}>
                    <span style={{ "margin-left": "16px" }}>
                      跳过 {c().thesis_distill_skipped.length} 只:
                      <span style={{ color: "#d97706", "font-family": "monospace" }}>
                        {c().thesis_distill_skipped.join(", ")}
                      </span>
                    </span>
                  </Show>
                </div>
                <button
                  type="button"
                  onClick={handleRefresh}
                  disabled={refreshing()}
                  style={{
                    padding: "8px 16px",
                    "border-radius": "8px",
                    border: "1px solid #f59e0b",
                    background: refreshing() ? "rgba(245,158,11,0.5)" : "#f59e0b",
                    color: "#fff",
                    cursor: refreshing() ? "not-allowed" : "pointer",
                    "font-family": "inherit",
                    "font-size": "13px",
                    "font-weight": "600",
                  }}
                >
                  {refreshing() ? "蒸馏中…" : "立即刷新"}
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
                    when={c().investment_global_style}
                    fallback={
                      <span style={{ color: "#94a3b8" }}>
                        尚未蒸馏 —— 至少要有 1 个公司画像才能产出整体风格。
                      </span>
                    }
                  >
                    {c().investment_global_style}
                  </Show>
                </div>
              </div>

              {/* Per-ticker thesis */}
              <h2
                style={{
                  "font-size": "16px",
                  "font-weight": "700",
                  color: "#0f172a",
                  margin: "24px 0 12px",
                }}
              >
                各持仓 Thesis ({c().holdings.length} 只)
              </h2>
              <Show
                when={c().holdings.length > 0}
                fallback={
                  <div
                    style={{
                      padding: "32px",
                      "border-radius": "10px",
                      background: "#fff",
                      "text-align": "center",
                      color: "#94a3b8",
                      "font-size": "13px",
                    }}
                  >
                    持仓为空 —— 请在 /chat 里告诉 agent 你持有什么。
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
                  <For each={c().holdings}>
                    {(ticker) => (
                      <ThesisCard
                        ticker={ticker}
                        thesis={c().investment_theses[ticker]}
                        hasProfile={profileTickers().has(ticker)}
                        isSkipped={c().thesis_distill_skipped.includes(ticker)}
                        onView={() => openProfile(ticker)}
                      />
                    )}
                  </For>
                </div>
              </Show>

              {/* 画像 inventory */}
              <h2
                style={{
                  "font-size": "16px",
                  "font-weight": "700",
                  color: "#0f172a",
                  margin: "32px 0 12px",
                }}
              >
                公司画像 inventory ({c().profile_list.length})
              </h2>
              <Show
                when={c().profile_list.length > 0}
                fallback={
                  <div
                    style={{
                      padding: "32px",
                      "border-radius": "10px",
                      background: "#fff",
                      "text-align": "center",
                      color: "#94a3b8",
                      "font-size": "13px",
                    }}
                  >
                    sandbox 里还没有任何公司画像。在 /chat 里说"建立 X 的公司画像"开始。
                  </div>
                }
              >
                <div style={{ display: "flex", "flex-direction": "column", gap: "10px" }}>
                  <For each={c().profile_list}>
                    {(p) => (
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
                            {p.title || p.dir}
                            <span
                              style={{
                                "margin-left": "8px",
                                "font-family": "monospace",
                                "font-size": "12px",
                                color: "#64748b",
                              }}
                            >
                              {p.tickers.join(" / ")}
                            </span>
                          </div>
                          <div
                            style={{
                              "font-size": "12px",
                              color: "#94a3b8",
                              "margin-top": "4px",
                            }}
                          >
                            {(p.bytes / 1024).toFixed(1)} KB · {p.dir}
                          </div>
                        </div>
                        <Show when={p.tickers.length > 0}>
                          <button
                            type="button"
                            onClick={() => openProfile(p.tickers[0])}
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
                        </Show>
                      </div>
                    )}
                  </For>
                </div>
              </Show>

              <ProfileModal
                open={modalOpen()}
                ticker={modalTicker()}
                onClose={() => setModalOpen(false)}
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
