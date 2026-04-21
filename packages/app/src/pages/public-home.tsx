// public-home.tsx — Hone Public Site Homepage

import {
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
  type ParentProps,
} from "solid-js"
import { useNavigate } from "@solidjs/router"
import { PublicNav, PublicFooter } from "@/components/public-nav"
import { CONTENT } from "@/lib/public-content"
import "./public-site.css"

// ── Fade-in section wrapper ───────────────────────────────────────────────────
function FadeSection(props: ParentProps<{ style?: Record<string, string> }>) {
  let ref!: HTMLDivElement
  const [visible, setVisible] = createSignal(false)

  onMount(() => {
    const obs = new IntersectionObserver(
      ([e]) => { if (e.isIntersecting) setVisible(true) },
      { threshold: 0.12 },
    )
    obs.observe(ref)
    onCleanup(() => obs.disconnect())
  })

  return (
    <div
      ref={ref}
      style={{
        opacity: visible() ? "1" : "0",
        transform: visible() ? "translateY(0)" : "translateY(28px)",
        transition: "opacity 0.7s ease, transform 0.7s ease",
        ...props.style,
      }}
    >
      {props.children}
    </div>
  )
}

function SectionLabel(props: ParentProps<{ light?: boolean }>) {
  return (
    <div
      style={{
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
        "font-size": "11px",
        "font-weight": "700",
        "letter-spacing": "0.30em",
        "text-transform": "uppercase",
        color: props.light ? "rgba(245,158,11,0.7)" : "#f59e0b",
        "margin-bottom": "16px",
      }}
    >
      {props.children}
    </div>
  )
}

// ── Hero ──────────────────────────────────────────────────────────────────────
function HeroSection() {
  const navigate = useNavigate()
  const C = CONTENT.hero

  return (
    <section
      style={{
        "min-height": "100vh",
        background: "#0f172a",
        display: "flex",
        "align-items": "center",
        position: "relative",
        overflow: "hidden",
        padding: "0 32px",
      }}
    >
      {/* Amber glow */}
      <div
        style={{
          position: "absolute",
          top: "30%",
          right: "10%",
          width: "600px",
          height: "600px",
          "border-radius": "50%",
          background: "radial-gradient(ellipse, rgba(245,158,11,0.12) 0%, transparent 70%)",
          "pointer-events": "none",
        }}
      />
      <div
        style={{
          position: "absolute",
          bottom: "-100px",
          left: "-100px",
          width: "400px",
          height: "400px",
          "border-radius": "50%",
          background: "radial-gradient(ellipse, rgba(245,158,11,0.06) 0%, transparent 70%)",
          "pointer-events": "none",
        }}
      />

      {/* Grid lines */}
      <div
        style={{
          position: "absolute",
          inset: "0",
          "pointer-events": "none",
          "background-image":
            "linear-gradient(rgba(255,255,255,0.025) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.025) 1px, transparent 1px)",
          "background-size": "80px 80px",
        }}
      />

      <div
        class="pub-hero-grid"
        style={{ "max-width": "1100px", margin: "0 auto", width: "100%" }}
      >
        {/* Left: copy */}
        <div>
          <div
            style={{
              "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
              "font-size": "11px",
              "font-weight": "700",
              "letter-spacing": "0.30em",
              "text-transform": "uppercase",
              color: "rgba(245,158,11,0.65)",
              "margin-bottom": "32px",
            }}
          >
            {C.eyebrow}
          </div>

          <h1
            style={{
              "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
              "font-weight": "700",
              "line-height": "1.2",
              margin: "0 0 4px",
              "letter-spacing": "-0.02em",
            }}
          >
            <span style={{ "font-size": "clamp(32px, 4vw, 52px)", color: "#94a3b8", display: "block" }}>
              {C.headline_1}
            </span>
            <span
              style={{ "font-size": "clamp(36px, 4.5vw, 58px)", color: "#f59e0b", display: "block", "margin-top": "8px" }}
            >
              {C.headline_2}
            </span>
          </h1>

          <p
            style={{
              "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
              "font-size": "16px",
              "line-height": "1.75",
              color: "#64748b",
              margin: "28px 0 40px",
              "max-width": "440px",
            }}
          >
            {C.description}
          </p>

          {/* CTAs */}
          <div style={{ display: "flex", gap: "12px", "flex-wrap": "wrap" }}>
            <button
              onClick={() => navigate("/chat")}
              style={{
                padding: "14px 32px",
                "border-radius": "8px",
                background: "#f59e0b",
                border: "none",
                cursor: "pointer",
                "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                "font-size": "15px",
                "font-weight": "700",
                color: "#fff",
                "letter-spacing": "0.02em",
                "box-shadow": "0 4px 20px rgba(245,158,11,0.35)",
                transition: "all 0.2s",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = "#d97706"
                e.currentTarget.style.transform = "translateY(-1px)"
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = "#f59e0b"
                e.currentTarget.style.transform = "translateY(0)"
              }}
            >
              {C.cta_primary} →
            </button>
            <button
              onClick={() => navigate("/roadmap")}
              style={{
                padding: "14px 28px",
                "border-radius": "8px",
                background: "transparent",
                border: "1px solid rgba(255,255,255,0.15)",
                cursor: "pointer",
                "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                "font-size": "15px",
                "font-weight": "600",
                color: "#94a3b8",
                "letter-spacing": "0.02em",
                transition: "all 0.2s",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.borderColor = "rgba(245,158,11,0.4)"
                e.currentTarget.style.color = "#f59e0b"
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.borderColor = "rgba(255,255,255,0.15)"
                e.currentTarget.style.color = "#94a3b8"
              }}
            >
              {C.cta_secondary}
            </button>
          </div>

          {/* Stats */}
          <div
            style={{
              display: "flex",
              gap: "40px",
              "margin-top": "56px",
              "padding-top": "32px",
              "border-top": "1px solid rgba(255,255,255,0.06)",
            }}
          >
            <For each={[C.stat_1, C.stat_2, C.stat_3]}>
              {(s) => (
                <div>
                  <div
                    style={{
                      "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                      "font-size": "22px",
                      "font-weight": "700",
                      color: "#f59e0b",
                      "letter-spacing": "-0.02em",
                    }}
                  >
                    {s.value}
                  </div>
                  <div style={{ "font-size": "12px", color: "#475569", "margin-top": "4px", "letter-spacing": "0.05em" }}>
                    {s.label}
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>

        {/* Right: app screenshot */}
        <div style={{ position: "relative" }}>
          <div
            style={{
              position: "absolute",
              inset: "-20px",
              background: "radial-gradient(ellipse, rgba(245,158,11,0.15) 0%, transparent 70%)",
              "border-radius": "50%",
              "pointer-events": "none",
            }}
          />
          <div
            style={{
              "border-radius": "12px",
              overflow: "hidden",
              border: "1px solid rgba(255,255,255,0.08)",
              "box-shadow": "0 32px 80px rgba(0,0,0,0.6), 0 0 0 1px rgba(245,158,11,0.1)",
              transform: "perspective(1000px) rotateY(-4deg) rotateX(2deg)",
              transition: "transform 0.5s ease",
              position: "relative",
              "z-index": "1",
            }}
          >
            <img src="/hone_page.jpg" alt="Hone Console" style={{ width: "100%", display: "block" }} />
          </div>
          {/* Floating badge */}
          <div
            style={{
              position: "absolute",
              bottom: "-16px",
              left: "-16px",
              background: "#0f172a",
              border: "1px solid rgba(245,158,11,0.3)",
              "border-radius": "10px",
              padding: "12px 18px",
              "z-index": "2",
              "box-shadow": "0 8px 24px rgba(0,0,0,0.4)",
            }}
          >
            <div
              style={{
                "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                "font-size": "10px",
                color: "#f59e0b",
                "letter-spacing": "0.15em",
                "text-transform": "uppercase",
                "margin-bottom": "4px",
              }}
            >
              OPEN SOURCE
            </div>
            <div style={{ "font-size": "12px", color: "#94a3b8" }}>Rust · SolidJS · MIT</div>
          </div>
        </div>
      </div>

      {/* Scroll indicator */}
      <button
        onClick={() => {
          const el = document.getElementById("trust-section")
          el?.scrollIntoView({ behavior: "smooth" })
        }}
        style={{
          position: "absolute",
          bottom: "32px",
          left: "50%",
          transform: "translateX(-50%)",
          display: "flex",
          "flex-direction": "column",
          "align-items": "center",
          gap: "10px",
          background: "none",
          border: "none",
          cursor: "pointer",
          padding: "8px 16px",
        }}
      >
        <span
          style={{
            "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
            "font-size": "10px",
            "font-weight": "600",
            color: "rgba(255,255,255,0.35)",
            "letter-spacing": "0.20em",
            "text-transform": "uppercase",
          }}
        >
          {CONTENT.hero.scroll_hint}
        </span>
        <div class="pub-scroll-chevron">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
            <path d="M5 7.5L10 12.5L15 7.5" stroke="rgba(245,158,11,0.6)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
      </button>
    </section>
  )
}

// ── Trust ─────────────────────────────────────────────────────────────────────
function TrustSection() {
  const C = CONTENT.trust

  return (
    <section id="trust-section" style={{ background: "#fff", padding: "100px 32px" }}>
      <FadeSection>
        <div style={{ "max-width": "1100px", margin: "0 auto" }}>
          <SectionLabel>{C.section_label}</SectionLabel>
          <div class="pub-trust-grid">
            <For each={C.items}>
              {(item, i) => (
                <div
                  style={{
                    padding: "40px 36px",
                    "border-top": "2px solid",
                    "border-top-color": i() === 0 ? "#f59e0b" : "rgba(0,0,0,0.06)",
                    background: i() === 0 ? "rgba(245,158,11,0.04)" : "#fff",
                  }}
                >
                  <div
                    style={{
                      "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                      "font-size": "20px",
                      color: i() === 0 ? "#f59e0b" : "#94a3b8",
                      "margin-bottom": "20px",
                    }}
                  >
                    {item.symbol}
                  </div>
                  <h3
                    style={{
                      "font-size": "18px",
                      "font-weight": "700",
                      color: "#0f172a",
                      margin: "0 0 14px",
                      "letter-spacing": "-0.01em",
                    }}
                  >
                    {item.title}
                  </h3>
                  <p style={{ "font-size": "14px", "line-height": "1.75", color: "#64748b", margin: "0" }}>
                    {item.body}
                  </p>
                </div>
              )}
            </For>
          </div>
        </div>
      </FadeSection>
    </section>
  )
}

// ── Cases ─────────────────────────────────────────────────────────────────────
function CasesSection() {
  const C = CONTENT.cases
  const [active, setActive] = createSignal(0)
  const item = () => C.items[active()]

  return (
    <section style={{ background: "#f8fafc", padding: "100px 32px" }}>
      <FadeSection>
        <div style={{ "max-width": "1100px", margin: "0 auto" }}>
          <SectionLabel>{C.section_label}</SectionLabel>
          <div
            style={{
              display: "flex",
              "align-items": "flex-end",
              "justify-content": "space-between",
              "margin-bottom": "48px",
            }}
          >
            <h2
              style={{
                "font-size": "clamp(24px,3vw,36px)",
                "font-weight": "700",
                color: "#0f172a",
                margin: "0",
                "letter-spacing": "-0.02em",
              }}
            >
              {C.section_sub}
            </h2>
          </div>

          <div class="pub-cases-grid">
            {/* Tab list */}
            <div style={{ display: "flex", "flex-direction": "column" }}>
              <For each={C.items}>
                {(it, i) => (
                  <button
                    onClick={() => setActive(i())}
                    style={{
                      padding: "18px 20px",
                      "text-align": "left",
                      background: active() === i() ? "#fff" : "transparent",
                      border: "none",
                      cursor: "pointer",
                      "border-left": `3px solid ${active() === i() ? "#f59e0b" : "transparent"}`,
                      transition: "all 0.2s",
                    }}
                  >
                    <div
                      style={{
                        "font-size": "11px",
                        "font-weight": "600",
                        "letter-spacing": "0.20em",
                        "text-transform": "uppercase",
                        color: active() === i() ? "#f59e0b" : "#94a3b8",
                        "margin-bottom": "6px",
                      }}
                    >
                      {it.tag}
                    </div>
                    <div
                      style={{
                        "font-size": "14px",
                        "font-weight": "600",
                        color: active() === i() ? "#0f172a" : "#64748b",
                        "line-height": "1.4",
                      }}
                    >
                      {it.title}
                    </div>
                  </button>
                )}
              </For>
            </div>

            {/* Content panel */}
            <div
              style={{
                background: "#fff",
                padding: "40px 48px",
                "border-left": "1px solid rgba(0,0,0,0.06)",
                display: "flex",
                "flex-direction": "column",
                "justify-content": "space-between",
              }}
            >
              <div>
                <div
                  style={{
                    display: "inline-block",
                    padding: "4px 12px",
                    "border-radius": "999px",
                    background: "rgba(245,158,11,0.10)",
                    border: "1px solid rgba(245,158,11,0.20)",
                    "font-size": "11px",
                    "font-weight": "600",
                    color: "#d97706",
                    "letter-spacing": "0.10em",
                    "text-transform": "uppercase",
                    "margin-bottom": "20px",
                  }}
                >
                  {item().tag}
                </div>

                <h3
                  style={{
                    "font-size": "24px",
                    "font-weight": "700",
                    color: "#0f172a",
                    margin: "0 0 16px",
                    "letter-spacing": "-0.01em",
                  }}
                >
                  {item().title}
                </h3>
                <p
                  style={{
                    "font-size": "15px",
                    "line-height": "1.8",
                    color: "#64748b",
                    margin: "0 0 32px",
                    "max-width": "480px",
                  }}
                >
                  {item().body}
                </p>
              </div>

              <Show
                when={item().image}
                fallback={
                  <div
                    style={{
                      "border-radius": "8px",
                      padding: "32px",
                      background: "rgba(245,158,11,0.04)",
                      border: "1px dashed rgba(245,158,11,0.20)",
                      display: "flex",
                      "align-items": "center",
                      gap: "16px",
                    }}
                  >
                    <div
                      style={{
                        "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                        "font-size": "28px",
                        color: "#f59e0b",
                        opacity: "0.5",
                      }}
                    >
                      ◈
                    </div>
                    <div
                      style={{
                        "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                        "font-size": "13px",
                        color: "#94a3b8",
                      }}
                    >
                      {`> hone: ${item().tag} ${CONTENT.cases.placeholder_suffix}`}
                    </div>
                  </div>
                }
              >
                <div
                  style={{
                    "border-radius": "8px",
                    overflow: "hidden",
                    border: "1px solid rgba(0,0,0,0.08)",
                    "box-shadow": "0 4px 16px rgba(0,0,0,0.06)",
                    "max-height": "200px",
                  }}
                >
                  <img
                    src={item().image!}
                    style={{ width: "100%", display: "block", "object-fit": "cover", "max-height": "200px" }}
                    alt={item().title}
                  />
                </div>
              </Show>
            </div>
          </div>
        </div>
      </FadeSection>
    </section>
  )
}

// ── Video ─────────────────────────────────────────────────────────────────────
function VideoSection() {
  const C = CONTENT.video
  const [playing, setPlaying] = createSignal(false)

  return (
    <section style={{ background: "#fff", padding: "100px 32px" }}>
      <FadeSection>
        <div style={{ "max-width": "1100px", margin: "0 auto" }}>
          <SectionLabel>{C.section_label}</SectionLabel>

          <div class="pub-video-grid">
            {/* Video player */}
            <div style={{ position: "relative" }}>
              <div
                onClick={() => setPlaying(true)}
                style={{
                  "border-radius": "12px",
                  overflow: "hidden",
                  border: "1px solid rgba(0,0,0,0.08)",
                  "box-shadow": "0 8px 32px rgba(0,0,0,0.10)",
                  cursor: "pointer",
                  position: "relative",
                  "aspect-ratio": "16/9",
                  background: "#0f172a",
                }}
              >
                <Show when={C.thumbnail && !playing()}>
                  <img
                    src={C.thumbnail}
                    alt="Video thumbnail"
                    style={{ width: "100%", height: "100%", "object-fit": "cover", display: "block", opacity: "0.85" }}
                  />
                </Show>
                <Show when={!playing()}>
                  <div
                    style={{
                      position: "absolute",
                      inset: "0",
                      display: "flex",
                      "align-items": "center",
                      "justify-content": "center",
                      background: "rgba(15,23,42,0.4)",
                    }}
                  >
                    <div
                      style={{
                        width: "72px",
                        height: "72px",
                        "border-radius": "50%",
                        background: "rgba(245,158,11,0.9)",
                        display: "flex",
                        "align-items": "center",
                        "justify-content": "center",
                        "box-shadow": "0 8px 32px rgba(245,158,11,0.4)",
                        transition: "transform 0.2s",
                      }}
                      onMouseEnter={(e) => { e.currentTarget.style.transform = "scale(1.08)" }}
                      onMouseLeave={(e) => { e.currentTarget.style.transform = "scale(1)" }}
                    >
                      <svg width="24" height="24" viewBox="0 0 24 24" fill="white">
                        <path d="M8 5v14l11-7z" />
                      </svg>
                    </div>
                  </div>
                </Show>
                <Show when={playing() && C.video_url}>
                  <iframe
                    src={`${C.video_url}?autoplay=1`}
                    style={{ position: "absolute", inset: "0", width: "100%", height: "100%", border: "none" }}
                    allow="autoplay; encrypted-media"
                    allowfullscreen
                  />
                </Show>
                <Show when={playing() && !C.video_url}>
                  <div
                    style={{
                      position: "absolute",
                      inset: "0",
                      display: "flex",
                      "flex-direction": "column",
                      "align-items": "center",
                      "justify-content": "center",
                      background: "#0f172a",
                      color: "#64748b",
                      "font-size": "14px",
                      gap: "8px",
                    }}
                  >
                    <div
                      style={{
                        "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                        "font-size": "20px",
                        color: "#334155",
                      }}
                    >
                      ▶
                    </div>
                    <span>{CONTENT.video.url_placeholder}</span>
                  </div>
                </Show>
              </div>
              {/* Duration badge */}
              <div
                style={{
                  position: "absolute",
                  bottom: "16px",
                  right: "16px",
                  background: "rgba(15,23,42,0.85)",
                  "border-radius": "6px",
                  padding: "4px 10px",
                  "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                  "font-size": "12px",
                  color: "#f59e0b",
                  "pointer-events": "none",
                }}
              >
                {C.duration}
              </div>
            </div>

            {/* Copy */}
            <div>
              <h2
                style={{
                  "font-size": "clamp(22px,2.5vw,30px)",
                  "font-weight": "700",
                  color: "#0f172a",
                  margin: "0 0 20px",
                  "letter-spacing": "-0.02em",
                  "line-height": "1.3",
                }}
              >
                {C.title}
              </h2>
              <p
                style={{
                  "font-size": "15px",
                  "line-height": "1.8",
                  color: "#64748b",
                  margin: "0 0 32px",
                }}
              >
                {C.description}
              </p>
              <div
                style={{
                  display: "flex",
                  "align-items": "center",
                  gap: "12px",
                  padding: "16px 20px",
                  "border-radius": "8px",
                  background: "rgba(245,158,11,0.06)",
                  border: "1px solid rgba(245,158,11,0.15)",
                }}
              >
                <span style={{ "font-family": "var(--font-mono, 'JetBrains Mono', monospace)", color: "#f59e0b", "font-size": "18px" }}>
                  ⚡
                </span>
                <span style={{ "font-size": "13px", color: "#64748b", "line-height": "1.5" }}>
                  {CONTENT.video.coverage}
                </span>
              </div>
            </div>
          </div>
        </div>
      </FadeSection>
    </section>
  )
}

// ── Capabilities ──────────────────────────────────────────────────────────────
function CapabilitiesSection() {
  const C = CONTENT.capabilities

  return (
    <section style={{ background: "#f8fafc", padding: "100px 32px" }}>
      <FadeSection>
        <div style={{ "max-width": "1100px", margin: "0 auto" }}>
          <SectionLabel>{C.section_label}</SectionLabel>
          <div class="pub-caps-grid">
            <For each={C.items}>
              {(item) => (
                <div
                  style={{ background: "#fff", padding: "36px 32px", transition: "background 0.2s" }}
                  onMouseEnter={(e) => { e.currentTarget.style.background = "rgba(245,158,11,0.04)" }}
                  onMouseLeave={(e) => { e.currentTarget.style.background = "#fff" }}
                >
                  <div
                    style={{
                      "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                      "font-size": "22px",
                      color: "#f59e0b",
                      "margin-bottom": "16px",
                    }}
                  >
                    {item.symbol}
                  </div>
                  <h3
                    style={{
                      "font-size": "16px",
                      "font-weight": "700",
                      color: "#0f172a",
                      margin: "0 0 10px",
                    }}
                  >
                    {item.title}
                  </h3>
                  <p style={{ "font-size": "13px", "line-height": "1.75", color: "#64748b", margin: "0" }}>
                    {item.body}
                  </p>
                </div>
              )}
            </For>
          </div>
        </div>
      </FadeSection>
    </section>
  )
}

// ── Community ─────────────────────────────────────────────────────────────────
function CommunitySection() {
  const C = CONTENT.community

  return (
    <section style={{ background: "#fff", padding: "100px 32px" }}>
      <FadeSection>
        <div style={{ "max-width": "1100px", margin: "0 auto" }}>
          <SectionLabel>{C.section_label}</SectionLabel>
          <h2
            style={{
              "font-size": "clamp(24px,3vw,36px)",
              "font-weight": "700",
              color: "#0f172a",
              margin: "0 0 56px",
              "letter-spacing": "-0.02em",
            }}
          >
            {C.section_sub}
          </h2>

          {/* Tier 1: QR cards */}
          <div class="pub-community-t1">
            <For each={C.tier1}>
              {(item) => (
                <div
                  style={{
                    padding: "36px",
                    "border-radius": "12px",
                    border: "1px solid rgba(245,158,11,0.20)",
                    background: "rgba(245,158,11,0.03)",
                    display: "flex",
                    gap: "28px",
                    "align-items": "flex-start",
                  }}
                >
                  {/* QR placeholder */}
                  <div
                    style={{
                      width: "100px",
                      height: "100px",
                      "flex-shrink": "0",
                      "border-radius": "8px",
                      background: "#f8fafc",
                      border: "2px dashed rgba(245,158,11,0.25)",
                      display: "flex",
                      "flex-direction": "column",
                      "align-items": "center",
                      "justify-content": "center",
                      gap: "6px",
                    }}
                  >
                    <Show
                      when={item.qr}
                      fallback={
                        <>
                          <div
                            style={{
                              width: "48px",
                              height: "48px",
                              background:
                                "repeating-linear-gradient(0deg, rgba(245,158,11,0.15) 0px, rgba(245,158,11,0.15) 4px, transparent 4px, transparent 8px), repeating-linear-gradient(90deg, rgba(245,158,11,0.15) 0px, rgba(245,158,11,0.15) 4px, transparent 4px, transparent 8px)",
                              "border-radius": "4px",
                            }}
                          />
                          <span style={{ "font-size": "10px", color: "#94a3b8", "letter-spacing": "0.05em" }}>
                            {CONTENT.community.qr_label}
                          </span>
                        </>
                      }
                    >
                      <img
                        src={item.qr!}
                        style={{ width: "100%", height: "100%", "object-fit": "cover", "border-radius": "6px" }}
                        alt="QR"
                      />
                    </Show>
                  </div>
                  <div>
                    <div
                      style={{
                        display: "inline-block",
                        padding: "2px 8px",
                        "border-radius": "999px",
                        background: "rgba(245,158,11,0.12)",
                        border: "1px solid rgba(245,158,11,0.2)",
                        "font-size": "10px",
                        "font-weight": "700",
                        color: "#d97706",
                        "letter-spacing": "0.15em",
                        "text-transform": "uppercase",
                        "margin-bottom": "10px",
                      }}
                    >
                      {item.tier_label}
                    </div>
                    <h3 style={{ "font-size": "18px", "font-weight": "700", color: "#0f172a", margin: "0 0 8px" }}>
                      {item.name}
                    </h3>
                    <p style={{ "font-size": "13px", "line-height": "1.6", color: "#64748b", margin: "0 0 16px" }}>
                      {item.desc}
                    </p>
                    <button
                      style={{
                        padding: "7px 18px",
                        "border-radius": "6px",
                        background: "#f59e0b",
                        border: "none",
                        cursor: "pointer",
                        "font-size": "13px",
                        "font-weight": "600",
                        color: "#fff",
                        "font-family": "inherit",
                      }}
                    >
                      {item.cta}
                    </button>
                  </div>
                </div>
              )}
            </For>
          </div>

          {/* Tier 2: channel list */}
          <div class="pub-community-t2">
            <For each={C.tier2}>
              {(item) => (
                <a
                  href={item.url}
                  style={{
                    padding: "20px",
                    "border-radius": "8px",
                    border: "1px solid rgba(0,0,0,0.08)",
                    background: "#f8fafc",
                    "text-decoration": "none",
                    display: "flex",
                    "flex-direction": "column",
                    gap: "8px",
                    transition: "border-color 0.2s, transform 0.2s",
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.borderColor = "rgba(245,158,11,0.30)"
                    e.currentTarget.style.transform = "translateY(-2px)"
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.borderColor = "rgba(0,0,0,0.08)"
                    e.currentTarget.style.transform = "translateY(0)"
                  }}
                >
                  <div style={{ display: "flex", "align-items": "center", "justify-content": "space-between" }}>
                    <span
                      style={{
                        "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                        "font-size": "18px",
                        color: "#94a3b8",
                      }}
                    >
                      {item.symbol}
                    </span>
                    <span
                      style={{
                        padding: "2px 8px",
                        "border-radius": "999px",
                        background: "rgba(0,0,0,0.05)",
                        "font-size": "10px",
                        "font-weight": "600",
                        color: "#94a3b8",
                        "letter-spacing": "0.10em",
                      }}
                    >
                      {item.label}
                    </span>
                  </div>
                  <div style={{ "font-size": "14px", "font-weight": "600", color: "#0f172a" }}>{item.name}</div>
                  <div style={{ "font-size": "12px", color: "#94a3b8" }}>{item.desc}</div>
                </a>
              )}
            </For>
          </div>
        </div>
      </FadeSection>
    </section>
  )
}

// ── Repo ──────────────────────────────────────────────────────────────────────
function RepoSection() {
  const C = CONTENT.repo

  return (
    <section style={{ background: "#0f172a", padding: "100px 32px" }}>
      <FadeSection>
        <div style={{ "max-width": "1100px", margin: "0 auto" }}>
          <SectionLabel light>{C.section_label}</SectionLabel>
          <div
            style={{
              display: "flex",
              "align-items": "flex-end",
              "justify-content": "space-between",
              "margin-bottom": "48px",
            }}
          >
            <h2
              style={{
                "font-size": "clamp(24px,3vw,36px)",
                "font-weight": "700",
                color: "#e2e8f0",
                margin: "0",
                "letter-spacing": "-0.02em",
              }}
            >
              {C.section_sub}
            </h2>
            <a
              href="https://github.com/B-M-Capital-Research/honeclaw"
              target="_blank"
              rel="noopener noreferrer"
              style={{
                "font-size": "13px",
                "font-weight": "600",
                color: "#f59e0b",
                "text-decoration": "none",
                padding: "8px 20px",
                "border-radius": "6px",
                border: "1px solid rgba(245,158,11,0.30)",
                "letter-spacing": "0.05em",
              }}
            >
              GitHub ↗
            </a>
          </div>

          <div class="pub-repo-grid">
            <For each={C.items}>
              {(item) => (
                <a
                  href={item.url}
                  target={item.url.startsWith("http") ? "_blank" : "_self"}
                  rel="noopener noreferrer"
                  style={{
                    padding: "28px",
                    background: "rgba(255,255,255,0.03)",
                    "text-decoration": "none",
                    transition: "background 0.2s",
                    display: "block",
                  }}
                  onMouseEnter={(e) => { e.currentTarget.style.background = "rgba(245,158,11,0.06)" }}
                  onMouseLeave={(e) => { e.currentTarget.style.background = "rgba(255,255,255,0.03)" }}
                >
                  <div
                    style={{
                      display: "flex",
                      "align-items": "center",
                      "justify-content": "space-between",
                      "margin-bottom": "14px",
                    }}
                  >
                    <span
                      style={{
                        "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                        "font-size": "16px",
                        color: "#f59e0b",
                        opacity: "0.6",
                      }}
                    >
                      {item.icon}
                    </span>
                    <span
                      style={{
                        padding: "2px 8px",
                        "border-radius": "999px",
                        background: "rgba(245,158,11,0.10)",
                        "font-size": "10px",
                        "font-weight": "600",
                        color: "#d97706",
                        "letter-spacing": "0.15em",
                        "text-transform": "uppercase",
                      }}
                    >
                      {item.tag}
                    </span>
                  </div>
                  <div
                    style={{
                      "font-size": "15px",
                      "font-weight": "700",
                      color: "#e2e8f0",
                      "margin-bottom": "8px",
                    }}
                  >
                    {item.title}
                  </div>
                  <div style={{ "font-size": "13px", color: "#475569", "line-height": "1.6" }}>{item.desc}</div>
                </a>
              )}
            </For>
          </div>
        </div>
      </FadeSection>
    </section>
  )
}

// ── PublicHomePage ────────────────────────────────────────────────────────────
export default function PublicHomePage() {
  return (
    <div
      class="pub-page"
      style={{
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
        "-webkit-font-smoothing": "antialiased",
        "text-rendering": "optimizeLegibility",
      }}
    >
      <PublicNav />
      <HeroSection />
      <TrustSection />
      <CasesSection />
      <VideoSection />
      <CapabilitiesSection />
      <CommunitySection />
      <RepoSection />
      <PublicFooter />
    </div>
  )
}
