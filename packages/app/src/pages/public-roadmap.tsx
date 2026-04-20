// public-roadmap.tsx — Hone Public Site Roadmap & Docs hub (dev-docs style)

import { createSignal, For, onCleanup, onMount, Show } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { PublicNav, PublicFooter } from "@/components/public-nav"
import { CONTENT } from "@/lib/public-content"
import "./public-site.css"

type Status = "stable" | "beta" | "planned" | "na"
type PhaseVariant = "now" | "next" | "later"

const STATUS_CHIP: Record<Status, { bg: string; bd: string; c: string; label: string }> = {
  stable: { bg: "rgba(34,197,94,0.10)", bd: "rgba(34,197,94,0.25)", c: "#16a34a", label: "STABLE" },
  beta: { bg: "rgba(245,158,11,0.10)", bd: "rgba(245,158,11,0.25)", c: "#d97706", label: "BETA" },
  planned: { bg: "rgba(99,102,241,0.08)", bd: "rgba(99,102,241,0.20)", c: "#6366f1", label: "PLANNED" },
  na: { bg: "rgba(0,0,0,0.04)", bd: "rgba(0,0,0,0.08)", c: "#94a3b8", label: "—" },
}

const PHASE_COLORS: Record<PhaseVariant, {
  border: string
  bg: string
  dot: string
  badge: string
}> = {
  now: { border: "rgba(245,158,11,0.30)", bg: "rgba(245,158,11,0.04)", dot: "#f59e0b", badge: "#d97706" },
  next: { border: "rgba(99,102,241,0.20)", bg: "rgba(99,102,241,0.04)", dot: "#818cf8", badge: "#6366f1" },
  later: { border: "rgba(100,116,139,0.15)", bg: "#f8fafc", dot: "#94a3b8", badge: "#64748b" },
}

const TOC = [
  { id: "quick-start", label: "快速开始", sub: "Quick Start" },
  { id: "capabilities", label: "能力矩阵", sub: "Capability Matrix" },
  { id: "channels", label: "渠道接入", sub: "Channels" },
  { id: "architecture", label: "架构", sub: "Architecture" },
  { id: "skills", label: "内置 Skill", sub: "Skills" },
  { id: "roadmap", label: "产品路线图", sub: "Roadmap" },
  { id: "boundary", label: "开源边界", sub: "Open Source" },
  { id: "docs", label: "文档入口", sub: "Docs" },
  { id: "contributing", label: "参与贡献", sub: "Contributing" },
  { id: "faq", label: "常见问题", sub: "FAQ" },
] as const

function StatusChip(props: { status: Status }) {
  const s = () => STATUS_CHIP[props.status] || STATUS_CHIP.na
  return (
    <span
      style={{
        padding: "2px 8px",
        "border-radius": "999px",
        background: s().bg,
        border: `1px solid ${s().bd}`,
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
        "font-size": "9px",
        "font-weight": "700",
        color: s().c,
        "letter-spacing": "0.15em",
        "white-space": "nowrap",
      }}
    >
      {s().label}
    </span>
  )
}

function CodeBlock(props: { lines: readonly string[]; lang?: string }) {
  return (
    <div
      style={{
        "border-radius": "8px",
        overflow: "hidden",
        border: "1px solid rgba(0,0,0,0.08)",
        background: "#0f172a",
        "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
        "font-size": "13px",
      }}
    >
      <div
        style={{
          padding: "8px 14px",
          "border-bottom": "1px solid rgba(255,255,255,0.06)",
          display: "flex",
          "justify-content": "space-between",
          "align-items": "center",
        }}
      >
        <span
          style={{
            "font-size": "10px",
            color: "#64748b",
            "letter-spacing": "0.15em",
            "text-transform": "uppercase",
          }}
        >
          {props.lang ?? "bash"}
        </span>
        <div style={{ display: "flex", gap: "5px" }}>
          <For each={["#ef4444", "#f59e0b", "#22c55e"]}>
            {(c) => (
              <span style={{ width: "8px", height: "8px", "border-radius": "50%", background: c, opacity: "0.35" }} />
            )}
          </For>
        </div>
      </div>
      <pre style={{ margin: "0", padding: "14px 18px", color: "#e2e8f0", "line-height": "1.7", overflow: "auto" }}>
        <For each={props.lines}>
          {(line) => (
            <Show
              when={!line.startsWith("#")}
              fallback={<div style={{ color: "#64748b" }}>{line}</div>}
            >
              <Show
                when={line.startsWith("$")}
                fallback={<div style={{ color: "#94a3b8" }}>{line}</div>}
              >
                <div>
                  <span style={{ color: "#f59e0b" }}>$</span>
                  <span style={{ color: "#e2e8f0" }}>{line.slice(1)}</span>
                </div>
              </Show>
            </Show>
          )}
        </For>
      </pre>
    </div>
  )
}

function Eyebrow(props: { children: any; color?: string }) {
  return (
    <div
      style={{
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
        "font-size": "11px",
        "font-weight": "700",
        "letter-spacing": "0.30em",
        "text-transform": "uppercase",
        color: props.color ?? "#f59e0b",
        "margin-bottom": "12px",
      }}
    >
      {props.children}
    </div>
  )
}

function DocSection(props: { id: string; eyebrow: string; title: string; divider?: boolean; children: any }) {
  return (
    <section
      id={props.id}
      class="pub-doc-section"
      style={{
        "padding-top": "64px",
        "padding-bottom": "64px",
        "border-bottom": (props.divider ?? true) ? "1px solid rgba(0,0,0,0.06)" : "none",
        "scroll-margin-top": "72px",
      }}
    >
      <Eyebrow>{props.eyebrow}</Eyebrow>
      <h2
        style={{
          "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
          "font-size": "28px",
          "font-weight": "700",
          color: "#0f172a",
          margin: "0 0 28px",
          "letter-spacing": "-0.02em",
        }}
      >
        {props.title}
      </h2>
      {props.children}
    </section>
  )
}

function PhaseCard(props: { phase: string; label: string; items: readonly string[]; variant: PhaseVariant }) {
  const c = PHASE_COLORS[props.variant]
  return (
    <div
      style={{
        "border-radius": "12px",
        border: `1px solid ${c.border}`,
        background: c.bg,
        padding: "28px 24px",
        display: "flex",
        "flex-direction": "column",
      }}
    >
      <div style={{ display: "flex", "align-items": "center", gap: "10px", "margin-bottom": "20px" }}>
        <span
          style={{
            "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
            "font-size": "11px",
            "font-weight": "700",
            "letter-spacing": "0.20em",
            color: c.badge,
            padding: "3px 8px",
            background: "rgba(255,255,255,0.6)",
            "border-radius": "4px",
            border: `1px solid ${c.border}`,
          }}
        >
          {props.phase}
        </span>
        <span style={{ "font-size": "13px", "font-weight": "600", color: "#0f172a" }}>{props.label}</span>
      </div>
      <ul style={{ "list-style": "none", padding: "0", margin: "0", display: "flex", "flex-direction": "column", gap: "8px" }}>
        <For each={props.items}>
          {(item) => (
            <li style={{ display: "flex", "align-items": "flex-start", gap: "10px" }}>
              <span
                style={{
                  width: "5px",
                  height: "5px",
                  "border-radius": "50%",
                  background: c.dot,
                  "flex-shrink": "0",
                  "margin-top": "8px",
                }}
              />
              <span style={{ "font-size": "13px", color: "#475569", "line-height": "1.6" }}>{item}</span>
            </li>
          )}
        </For>
      </ul>
    </div>
  )
}

function FAQItem(props: { q: string; a: string; defaultOpen?: boolean }) {
  const [open, setOpen] = createSignal(!!props.defaultOpen)
  return (
    <div
      style={{
        "border-radius": "10px",
        border: "1px solid rgba(0,0,0,0.08)",
        background: open() ? "rgba(245,158,11,0.03)" : "#fff",
        transition: "background 0.15s",
      }}
    >
      <button
        onClick={() => setOpen(!open())}
        style={{
          width: "100%",
          padding: "18px 22px",
          background: "none",
          border: "none",
          cursor: "pointer",
          display: "flex",
          "align-items": "center",
          "justify-content": "space-between",
          gap: "12px",
          "text-align": "left",
        }}
      >
        <span
          style={{
            "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
            "font-size": "14px",
            "font-weight": "600",
            color: "#0f172a",
          }}
        >
          {props.q}
        </span>
        <span
          style={{
            "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
            "font-size": "16px",
            color: "#f59e0b",
            transform: open() ? "rotate(45deg)" : "rotate(0deg)",
            transition: "transform 0.2s",
          }}
        >
          +
        </span>
      </button>
      <Show when={open()}>
        <div
          style={{
            padding: "0 22px 18px",
            "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
            "font-size": "13px",
            color: "#475569",
            "line-height": "1.75",
          }}
        >
          {props.a}
        </div>
      </Show>
    </div>
  )
}

function SidebarTOC(props: { active: string; onJump: (id: string) => void }) {
  return (
    <aside class="pub-doc-aside">
      <div
        style={{
          "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
          "font-size": "10px",
          "font-weight": "700",
          "letter-spacing": "0.25em",
          "text-transform": "uppercase",
          color: "#94a3b8",
          "margin-bottom": "16px",
        }}
      >
        ON THIS PAGE
      </div>
      <ul style={{ "list-style": "none", padding: "0", margin: "0", display: "flex", "flex-direction": "column", gap: "2px" }}>
        <For each={TOC}>
          {(item) => (
            <li>
              <a
                href={`#${item.id}`}
                onClick={(e) => {
                  e.preventDefault()
                  props.onJump(item.id)
                }}
                style={{
                  display: "flex",
                  "flex-direction": "column",
                  gap: "2px",
                  padding: "8px 12px",
                  "border-radius": "6px",
                  "text-decoration": "none",
                  background: props.active === item.id ? "rgba(245,158,11,0.08)" : "transparent",
                  "border-left": `2px solid ${props.active === item.id ? "#f59e0b" : "transparent"}`,
                  transition: "all 0.15s",
                }}
              >
                <span
                  style={{
                    "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                    "font-size": "13px",
                    "font-weight": "500",
                    color: props.active === item.id ? "#0f172a" : "#475569",
                  }}
                >
                  {item.label}
                </span>
                <span
                  style={{
                    "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                    "font-size": "10px",
                    color: props.active === item.id ? "#d97706" : "#94a3b8",
                    "letter-spacing": "0.05em",
                  }}
                >
                  {item.sub}
                </span>
              </a>
            </li>
          )}
        </For>
      </ul>
      <div style={{ "margin-top": "24px", "padding-top": "20px", "border-top": "1px solid rgba(0,0,0,0.06)" }}>
        <a
          href={CONTENT.nav.github_url}
          target="_blank"
          rel="noopener noreferrer"
          style={{
            display: "flex",
            "align-items": "center",
            "justify-content": "space-between",
            padding: "10px 12px",
            "border-radius": "6px",
            background: "#0f172a",
            color: "#f59e0b",
            "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
            "font-size": "12px",
            "font-weight": "600",
            "text-decoration": "none",
            "letter-spacing": "0.03em",
          }}
        >
          <span>GitHub</span>
          <span>↗</span>
        </a>
      </div>
    </aside>
  )
}

const ARCH_DIAGRAM = `            ┌────────────────────── CHANNELS ──────────────────────┐
            │  Web   iMessage   Lark   Discord   Telegram   CLI    │
            │                       MCP server                     │
            └──────────────────────────┬───────────────────────────┘
                                       │ SSE / Bot API / stdio
            ┌──────────────────────────▼───────────────────────────┐
            │             hone-web-api  (Rust · Tokio · axum)      │
            │  ┌─────────┐  ┌──────────┐  ┌─────────┐  ┌────────┐  │
            │  │ Session │  │  Skill   │  │  Cron   │  │ Router │  │
            │  │  Mgr    │  │ Registry │  │  Jobs   │  │        │  │
            │  └────┬────┘  └────┬─────┘  └────┬────┘  └───┬────┘  │
            └───────┼────────────┼─────────────┼───────────┼───────┘
                    │            │             │           │
            ┌───────▼────────────▼─────────────▼───────────▼───────┐
            │                    RUNNER LAYER                      │
            │  OpenAI · Gemini CLI/ACP · Codex CLI/ACP · OpenCode  │
            └──────────────────────────┬───────────────────────────┘
                                       │
            ┌──────────────────────────▼───────────────────────────┐
            │  Company Profile · Portfolio · Memory · Audit · Logs │
            └──────────────────────────────────────────────────────┘`

export default function PublicRoadmapPage() {
  const navigate = useNavigate()
  const C = CONTENT.roadmap
  const [activeToc, setActiveToc] = createSignal<string>(TOC[0].id)
  const [installTab, setInstallTab] = createSignal<"curl" | "brew" | "source">("curl")

  const jumpTo = (id: string) => {
    const el = document.getElementById(id)
    if (el) el.scrollIntoView({ behavior: "smooth", block: "start" })
  }

  onMount(() => {
    const obs = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) setActiveToc(e.target.id)
        })
      },
      { rootMargin: "-80px 0px -60% 0px" },
    )
    TOC.forEach((t) => {
      const el = document.getElementById(t.id)
      if (el) obs.observe(el)
    })
    onCleanup(() => obs.disconnect())
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

      <div style={{ "padding-top": "56px", background: "#fff" }}>
        {/* Header band */}
        <div
          style={{
            background: "linear-gradient(180deg, #0f172a 0%, #0b1120 100%)",
            padding: "72px 32px 56px",
            position: "relative",
            overflow: "hidden",
            "border-bottom": "1px solid rgba(245,158,11,0.10)",
          }}
        >
          <div
            style={{
              position: "absolute",
              top: "50%",
              right: "8%",
              transform: "translateY(-50%)",
              width: "500px",
              height: "500px",
              "border-radius": "50%",
              background: "radial-gradient(ellipse, rgba(245,158,11,0.10) 0%, transparent 70%)",
              "pointer-events": "none",
            }}
          />
          <div
            style={{
              position: "absolute",
              inset: "0",
              "pointer-events": "none",
              "background-image":
                "linear-gradient(rgba(255,255,255,0.02) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.02) 1px, transparent 1px)",
              "background-size": "80px 80px",
            }}
          />
          <div style={{ "max-width": "1200px", margin: "0 auto", position: "relative" }}>
            <div style={{ display: "flex", "align-items": "center", gap: "10px", "margin-bottom": "16px" }}>
              <span
                style={{
                  "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                  "font-size": "11px",
                  "font-weight": "700",
                  "letter-spacing": "0.30em",
                  color: "rgba(245,158,11,0.70)",
                  "text-transform": "uppercase",
                }}
              >
                {C.version}
              </span>
              <span
                style={{
                  "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                  "font-size": "11px",
                  color: "#334155",
                }}
              >
                ·
              </span>
              <span
                style={{
                  "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                  "font-size": "11px",
                  "font-weight": "600",
                  "letter-spacing": "0.15em",
                  color: "#64748b",
                  "text-transform": "uppercase",
                }}
              >
                ROADMAP · DOCS · API
              </span>
            </div>
            <h1
              style={{
                "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                "font-size": "clamp(32px,4vw,48px)",
                "font-weight": "700",
                color: "#f1f5f9",
                margin: "0 0 14px",
                "letter-spacing": "-0.02em",
              }}
            >
              {C.hero_title}
            </h1>
            <p
              style={{
                "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                "font-size": "16px",
                color: "#94a3b8",
                margin: "0 0 28px",
                "max-width": "640px",
                "line-height": "1.7",
              }}
            >
              {C.hero_sub}
            </p>
            <div style={{ display: "flex", "flex-wrap": "wrap", gap: "8px" }}>
              <For each={["quick-start", "capabilities", "channels", "roadmap", "faq"] as const}>
                {(id) => {
                  const t = TOC.find((x) => x.id === id)
                  return (
                    <a
                      href={`#${id}`}
                      onClick={(e) => {
                        e.preventDefault()
                        jumpTo(id)
                      }}
                      style={{
                        padding: "6px 14px",
                        "border-radius": "999px",
                        border: "1px solid rgba(255,255,255,0.10)",
                        background: "rgba(255,255,255,0.03)",
                        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                        "font-size": "12px",
                        "font-weight": "500",
                        color: "#94a3b8",
                        "text-decoration": "none",
                        "letter-spacing": "0.02em",
                      }}
                    >
                      → {t?.label}
                    </a>
                  )
                }}
              </For>
            </div>
          </div>
        </div>

        {/* Content + Sidebar */}
        <div class="pub-doc-shell">
          <SidebarTOC active={activeToc()} onJump={jumpTo} />

          <main style={{ flex: "1", "min-width": "0" }}>
            {/* QUICK START */}
            <DocSection id="quick-start" eyebrow="§ 01 · QUICK START" title="快速开始">
              <p
                style={{
                  "font-size": "15px",
                  color: "#475569",
                  "line-height": "1.75",
                  margin: "0 0 24px",
                  "max-width": "640px",
                }}
              >
                三种方式接入 Hone：一键安装脚本、Homebrew、或源码开发。任选其一即可开始。
              </p>
              <div
                class="pub-install-tabs"
                style={{
                  display: "flex",
                  gap: "2px",
                  "margin-bottom": "0",
                  "border-bottom": "1px solid rgba(0,0,0,0.06)",
                }}
              >
                <For each={[
                  { k: "curl" as const, label: "curl | bash", badge: "推荐" },
                  { k: "brew" as const, label: "Homebrew", badge: null },
                  { k: "source" as const, label: "源码 / launch.sh", badge: null },
                ]}>
                  {(tab) => (
                    <button
                      onClick={() => setInstallTab(tab.k)}
                      style={{
                        padding: "10px 18px",
                        background: "transparent",
                        border: "none",
                        cursor: "pointer",
                        "border-bottom": `2px solid ${installTab() === tab.k ? "#f59e0b" : "transparent"}`,
                        "font-family": "inherit",
                        "font-size": "13px",
                        "font-weight": "600",
                        color: installTab() === tab.k ? "#0f172a" : "#64748b",
                        display: "flex",
                        "align-items": "center",
                        gap: "8px",
                        "margin-bottom": "-1px",
                      }}
                    >
                      {tab.label}
                      <Show when={tab.badge}>
                        <span
                          style={{
                            padding: "1px 6px",
                            "border-radius": "3px",
                            background: "rgba(245,158,11,0.12)",
                            "font-size": "9px",
                            "font-weight": "700",
                            color: "#d97706",
                            "letter-spacing": "0.1em",
                          }}
                        >
                          {tab.badge}
                        </span>
                      </Show>
                    </button>
                  )}
                </For>
              </div>
              <div style={{ "margin-top": "20px" }}>
                <CodeBlock lines={C.install[installTab()]} lang="bash" />
              </div>
              <div
                style={{
                  "margin-top": "16px",
                  display: "flex",
                  gap: "12px",
                  "flex-wrap": "wrap",
                  "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                  "font-size": "12px",
                  color: "#94a3b8",
                }}
              >
                <span>系统要求：{C.requirements}</span>
              </div>
            </DocSection>

            {/* CAPABILITIES */}
            <DocSection id="capabilities" eyebrow="§ 02 · CAPABILITY MATRIX" title="能力矩阵">
              <p
                style={{
                  "font-size": "15px",
                  color: "#475569",
                  "line-height": "1.75",
                  margin: "0 0 24px",
                  "max-width": "640px",
                }}
              >
                <StatusChip status="stable" /> 生产可用 · <StatusChip status="beta" /> 预览 · <StatusChip status="planned" /> 规划中
              </p>
              <div style={{ "border-radius": "10px", border: "1px solid rgba(0,0,0,0.08)", overflow: "hidden" }}>
                <For each={C.capability_matrix}>
                  {(grp, gi) => (
                    <div>
                      <div
                        style={{
                          padding: "12px 20px",
                          background: "#f8fafc",
                          "font-size": "11px",
                          "font-weight": "700",
                          "letter-spacing": "0.20em",
                          "text-transform": "uppercase",
                          color: "#64748b",
                          "border-top": gi() > 0 ? "1px solid rgba(0,0,0,0.06)" : "none",
                          "border-bottom": "1px solid rgba(0,0,0,0.06)",
                        }}
                      >
                        {grp.group}
                      </div>
                      <For each={grp.rows}>
                        {(row, ri) => (
                          <div
                            class="pub-cap-row"
                            style={{
                              display: "grid",
                              "grid-template-columns": "1fr 110px 1fr",
                              gap: "20px",
                              "align-items": "center",
                              padding: "14px 20px",
                              "border-bottom": ri() < grp.rows.length - 1 ? "1px solid rgba(0,0,0,0.04)" : "none",
                            }}
                          >
                            <span style={{ "font-size": "14px", "font-weight": "500", color: "#0f172a" }}>
                              {row.name}
                            </span>
                            <StatusChip status={row.status as Status} />
                            <span
                              style={{
                                "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                                "font-size": "12px",
                                color: "#94a3b8",
                              }}
                            >
                              {row.note}
                            </span>
                          </div>
                        )}
                      </For>
                    </div>
                  )}
                </For>
              </div>
            </DocSection>

            {/* CHANNELS */}
            <DocSection id="channels" eyebrow="§ 03 · CHANNELS" title="渠道接入">
              <p
                style={{
                  "font-size": "15px",
                  color: "#475569",
                  "line-height": "1.75",
                  margin: "0 0 24px",
                  "max-width": "640px",
                }}
              >
                Hone 是多端接入的投研 agent。每个渠道都是独立进程，可独立启停、独立配置。
              </p>
              <div class="pub-channels-grid">
                <For each={C.channels}>
                  {(ch) => (
                    <div
                      style={{
                        padding: "20px 24px",
                        background: "#fff",
                        display: "flex",
                        "align-items": "center",
                        gap: "16px",
                      }}
                    >
                      <div
                        style={{
                          width: "40px",
                          height: "40px",
                          "border-radius": "8px",
                          background: "rgba(245,158,11,0.08)",
                          border: "1px solid rgba(245,158,11,0.20)",
                          display: "flex",
                          "align-items": "center",
                          "justify-content": "center",
                          "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                          "font-size": "18px",
                          color: "#f59e0b",
                          "flex-shrink": "0",
                        }}
                      >
                        {ch.icon}
                      </div>
                      <div style={{ flex: "1", "min-width": "0" }}>
                        <div style={{ display: "flex", "align-items": "center", gap: "8px", "margin-bottom": "4px" }}>
                          <span style={{ "font-size": "14px", "font-weight": "700", color: "#0f172a" }}>{ch.name}</span>
                          <StatusChip status={ch.status as Status} />
                        </div>
                        <span style={{ "font-size": "12px", color: "#64748b" }}>{ch.desc}</span>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </DocSection>

            {/* ARCHITECTURE */}
            <DocSection id="architecture" eyebrow="§ 04 · ARCHITECTURE" title="系统架构">
              <p
                style={{
                  "font-size": "15px",
                  color: "#475569",
                  "line-height": "1.75",
                  margin: "0 0 24px",
                  "max-width": "640px",
                }}
              >
                Rust 核心引擎 · 多 Runner 抽象 · SolidJS 前端。设计目标：长时间运行不掉线、多渠道状态隔离、Skill 可热插拔。
              </p>
              <pre
                style={{
                  background: "#0f172a",
                  "border-radius": "10px",
                  padding: "28px 32px",
                  "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                  "font-size": "12px",
                  color: "#94a3b8",
                  "line-height": "1.9",
                  overflow: "auto",
                  margin: "0",
                }}
              >
                {ARCH_DIAGRAM}
              </pre>
              <div
                style={{
                  "margin-top": "20px",
                  padding: "16px 20px",
                  "border-radius": "8px",
                  background: "rgba(245,158,11,0.04)",
                  border: "1px solid rgba(245,158,11,0.15)",
                  display: "flex",
                  gap: "12px",
                  "align-items": "flex-start",
                }}
              >
                <span
                  style={{
                    "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                    color: "#f59e0b",
                    "font-size": "16px",
                    "margin-top": "-2px",
                  }}
                >
                  ℹ
                </span>
                <span style={{ "font-size": "13px", color: "#475569", "line-height": "1.65" }}>
                  完整模块说明见{" "}
                  <a
                    href="https://github.com/B-M-Capital-Research/honeclaw/blob/main/AGENTS.md"
                    target="_blank"
                    rel="noopener noreferrer"
                    style={{ color: "#d97706", "text-decoration": "none", "font-weight": "600" }}
                  >
                    AGENTS.md ↗
                  </a>
                  。
                </span>
              </div>
            </DocSection>

            {/* SKILLS */}
            <DocSection id="skills" eyebrow="§ 05 · BUILT-IN SKILLS" title="内置 Skill">
              <p
                style={{
                  "font-size": "15px",
                  color: "#475569",
                  "line-height": "1.75",
                  margin: "0 0 24px",
                  "max-width": "640px",
                }}
              >
                Hone 的 Skill 由模型根据上下文自动调用。下面是仓库 <code style={{ "font-family": "var(--font-mono, 'JetBrains Mono', monospace)", "font-size": "13px", padding: "1px 6px", background: "rgba(245,158,11,0.10)", "border-radius": "3px", color: "#d97706" }}>skills/</code> 目录下的 18 个公开 Skill。
              </p>
              <div style={{ "border-radius": "10px", border: "1px solid rgba(0,0,0,0.08)", overflow: "hidden" }}>
                <For each={C.skills}>
                  {(s, i) => (
                    <div
                      class="pub-skill-row"
                      style={{
                        display: "grid",
                        "grid-template-columns": "260px 1fr",
                        padding: "14px 20px",
                        "border-bottom": i() < C.skills.length - 1 ? "1px solid rgba(0,0,0,0.04)" : "none",
                        gap: "20px",
                        "align-items": "center",
                      }}
                    >
                      <code
                        style={{
                          "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                          "font-size": "13px",
                          color: "#d97706",
                          "font-weight": "600",
                        }}
                      >
                        {s.name}
                      </code>
                      <span style={{ "font-size": "13px", color: "#475569" }}>{s.desc}</span>
                    </div>
                  )}
                </For>
              </div>
            </DocSection>

            {/* ROADMAP */}
            <DocSection id="roadmap" eyebrow="§ 06 · ROADMAP" title="产品路线图">
              <p
                style={{
                  "font-size": "15px",
                  color: "#475569",
                  "line-height": "1.75",
                  margin: "0 0 24px",
                  "max-width": "640px",
                }}
              >
                我们按 <strong style={{ color: "#0f172a" }}>Now / Next / Later</strong> 三阶段推进，具体发布节奏见 GitHub Releases。
              </p>
              <div class="pub-roadmap-phases">
                <PhaseCard phase="NOW" label={C.now.label} items={C.now.items} variant="now" />
                <PhaseCard phase="NEXT" label={C.next.label} items={C.next.items} variant="next" />
                <PhaseCard phase="LATER" label={C.later.label} items={C.later.items} variant="later" />
              </div>
            </DocSection>

            {/* BOUNDARY */}
            <DocSection id="boundary" eyebrow="§ 07 · OPEN SOURCE BOUNDARY" title="开源边界">
              <p
                style={{
                  "font-size": "15px",
                  color: "#475569",
                  "line-height": "1.75",
                  margin: "0 0 24px",
                  "max-width": "640px",
                }}
              >
                MIT 协议开源。开源仓库包含完整可运行的核心系统，私域增强能力不公开但不影响主流程可用性。
              </p>
              <div class="pub-roadmap-boundary">
                <div
                  style={{
                    padding: "28px 24px",
                    "border-radius": "10px",
                    border: "1px solid rgba(34,197,94,0.20)",
                    background: "rgba(34,197,94,0.03)",
                  }}
                >
                  <div style={{ display: "flex", "align-items": "center", gap: "8px", "margin-bottom": "18px" }}>
                    <div style={{ width: "6px", height: "6px", "border-radius": "50%", background: "#22c55e" }} />
                    <span
                      style={{
                        "font-size": "12px",
                        "font-weight": "700",
                        color: "#16a34a",
                        "letter-spacing": "0.12em",
                        "text-transform": "uppercase",
                      }}
                    >
                      开源公开
                    </span>
                  </div>
                  <ul style={{ "list-style": "none", padding: "0", margin: "0", display: "flex", "flex-direction": "column", gap: "8px" }}>
                    <For each={C.boundary.open}>
                      {(item) => (
                        <li style={{ display: "flex", "align-items": "center", gap: "10px" }}>
                          <span style={{ "font-family": "var(--font-mono, 'JetBrains Mono', monospace)", "font-size": "11px", color: "#22c55e" }}>
                            ✓
                          </span>
                          <span style={{ "font-size": "13px", color: "#475569" }}>{item}</span>
                        </li>
                      )}
                    </For>
                  </ul>
                </div>
                <div
                  style={{
                    padding: "28px 24px",
                    "border-radius": "10px",
                    border: "1px solid rgba(0,0,0,0.08)",
                    background: "#f8fafc",
                  }}
                >
                  <div style={{ display: "flex", "align-items": "center", gap: "8px", "margin-bottom": "18px" }}>
                    <div style={{ width: "6px", height: "6px", "border-radius": "50%", background: "#94a3b8" }} />
                    <span
                      style={{
                        "font-size": "12px",
                        "font-weight": "700",
                        color: "#64748b",
                        "letter-spacing": "0.12em",
                        "text-transform": "uppercase",
                      }}
                    >
                      私域 / 付费
                    </span>
                  </div>
                  <ul style={{ "list-style": "none", padding: "0", margin: "0", display: "flex", "flex-direction": "column", gap: "8px" }}>
                    <For each={C.boundary.closed}>
                      {(item) => (
                        <li style={{ display: "flex", "align-items": "center", gap: "10px" }}>
                          <span style={{ "font-family": "var(--font-mono, 'JetBrains Mono', monospace)", "font-size": "11px", color: "#94a3b8" }}>
                            —
                          </span>
                          <span style={{ "font-size": "13px", color: "#94a3b8" }}>{item}</span>
                        </li>
                      )}
                    </For>
                  </ul>
                </div>
              </div>
            </DocSection>

            {/* DOCS */}
            <DocSection id="docs" eyebrow="§ 08 · DOCUMENTATION" title="文档入口">
              <div class="pub-docs-grid">
                <For each={C.docs}>
                  {(doc) => (
                    <a
                      href={doc.url}
                      target={doc.url.startsWith("http") ? "_blank" : "_self"}
                      rel="noopener noreferrer"
                      class="pub-docs-card"
                      style={{
                        display: "flex",
                        "align-items": "center",
                        "justify-content": "space-between",
                        gap: "16px",
                        padding: "18px 20px",
                        "border-radius": "10px",
                        border: "1px solid rgba(0,0,0,0.08)",
                        background: "#fff",
                        "text-decoration": "none",
                        transition: "all 0.15s",
                      }}
                    >
                      <div>
                        <div style={{ "font-size": "14px", "font-weight": "700", color: "#0f172a", "margin-bottom": "4px" }}>
                          {doc.title}
                        </div>
                        <div style={{ "font-size": "12px", color: "#64748b" }}>{doc.desc}</div>
                      </div>
                      <span style={{ "font-family": "var(--font-mono, 'JetBrains Mono', monospace)", "font-size": "14px", color: "#f59e0b" }}>
                        ↗
                      </span>
                    </a>
                  )}
                </For>
              </div>
            </DocSection>

            {/* CONTRIBUTING */}
            <DocSection id="contributing" eyebrow="§ 09 · CONTRIBUTING" title="参与贡献">
              <p
                style={{
                  "font-size": "15px",
                  color: "#475569",
                  "line-height": "1.75",
                  margin: "0 0 24px",
                  "max-width": "640px",
                }}
              >
                Hone 是开源项目，欢迎所有形式的参与——不只是代码。
              </p>
              <div class="pub-contrib-grid">
                <For each={[
                  {
                    icon: "◈",
                    title: "提交 Issue",
                    desc: "报告 bug、提功能建议、讨论设计",
                    href: "https://github.com/B-M-Capital-Research/honeclaw/issues/new/choose",
                  },
                  {
                    icon: "⚡",
                    title: "发 Pull Request",
                    desc: "修 bug、加功能、优化文档",
                    href: "https://github.com/B-M-Capital-Research/honeclaw/pulls",
                  },
                  {
                    icon: "∞",
                    title: "贡献 Skill",
                    desc: "用 skills/skill_manager/create_skill.sh 起一个新 Skill",
                    href: "https://github.com/B-M-Capital-Research/honeclaw/tree/main/skills",
                  },
                ]}>
                  {(c) => (
                    <a
                      href={c.href}
                      target="_blank"
                      rel="noopener noreferrer"
                      style={{
                        padding: "24px 22px",
                        "border-radius": "10px",
                        border: "1px solid rgba(0,0,0,0.08)",
                        background: "#fff",
                        "text-decoration": "none",
                        display: "block",
                      }}
                    >
                      <div
                        style={{
                          "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                          "font-size": "18px",
                          color: "#f59e0b",
                          "margin-bottom": "12px",
                        }}
                      >
                        {c.icon}
                      </div>
                      <div style={{ "font-size": "14px", "font-weight": "700", color: "#0f172a", "margin-bottom": "6px" }}>
                        {c.title}
                      </div>
                      <div style={{ "font-size": "12px", color: "#64748b", "line-height": "1.6" }}>{c.desc}</div>
                    </a>
                  )}
                </For>
              </div>
            </DocSection>

            {/* FAQ */}
            <DocSection id="faq" eyebrow="§ 10 · FAQ" title="常见问题" divider={false}>
              <div style={{ display: "flex", "flex-direction": "column", gap: "10px" }}>
                <For each={C.faqs}>
                  {(f, i) => <FAQItem q={f.q} a={f.a} defaultOpen={i() === 0} />}
                </For>
              </div>
            </DocSection>
          </main>
        </div>

        {/* Bottom CTA */}
        <div style={{ background: "#0f172a", padding: "64px 32px", "text-align": "center" }}>
          <div style={{ "max-width": "600px", margin: "0 auto" }}>
            <h2
              style={{
                "font-size": "28px",
                "font-weight": "700",
                color: "#f1f5f9",
                margin: "0 0 14px",
                "letter-spacing": "-0.02em",
              }}
            >
              准备好开始了吗？
            </h2>
            <p
              style={{
                "font-size": "15px",
                color: "#64748b",
                margin: "0 0 32px",
                "line-height": "1.7",
              }}
            >
              进入对话，或直接 clone 仓库开始本地运行。
            </p>
            <div style={{ display: "flex", gap: "12px", "justify-content": "center", "flex-wrap": "wrap" }}>
              <button
                onClick={() => navigate("/chat")}
                style={{
                  padding: "12px 28px",
                  "border-radius": "8px",
                  background: "#f59e0b",
                  border: "none",
                  cursor: "pointer",
                  "font-family": "inherit",
                  "font-size": "14px",
                  "font-weight": "700",
                  color: "#fff",
                  "box-shadow": "0 4px 20px rgba(245,158,11,0.30)",
                }}
              >
                进入对话 →
              </button>
              <a
                href={CONTENT.nav.github_url}
                target="_blank"
                rel="noopener noreferrer"
                style={{
                  padding: "12px 24px",
                  "border-radius": "8px",
                  border: "1px solid rgba(255,255,255,0.15)",
                  "font-size": "14px",
                  "font-weight": "600",
                  color: "#94a3b8",
                  "text-decoration": "none",
                }}
              >
                GitHub ↗
              </a>
            </div>
          </div>
        </div>
      </div>

      <PublicFooter />
    </div>
  )
}
