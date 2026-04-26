// public-roadmap.tsx — Hone Public Site Roadmap (Synced with Landing v4 Style)

import {
  createSignal,
  onCleanup,
  onMount,
} from "solid-js"
import { useNavigate } from "@solidjs/router"
import { CONTENT } from "@/lib/public-content"
import { setLocale, useLocale } from "@/lib/i18n"
import "./public-site.css"

// ── Components ───────────────────────────────────────────────────────────────

function AnimatedBackground() {
  return (
    <div class="animated-bg">
      <div class="circle circle-1"></div>
      <div class="circle circle-2"></div>
      <div class="circle circle-3"></div>
    </div>
  )
}

function Header() {
  const navigate = useNavigate()
  const C = CONTENT.nav

  return (
    <header class="page-header">
      <div onClick={() => navigate("/")} class="header-logo">
        <img src="/logo.svg" alt="Hone" />
        <span>Hone</span>
      </div>

      <div class="header-actions">
        <div class="lang-switch">
          <button onClick={() => setLocale("zh")} class={useLocale() === "zh" ? "active" : ""}>中</button>
          <button onClick={() => setLocale("en")} class={useLocale() === "en" ? "active" : ""}>EN</button>
        </div>

        <div style={{ display: "flex", gap: "8px" }}>
          <button onClick={() => navigate("/")} class="btn-roadmap-nav">
            {useLocale() === 'zh' ? '返回首页' : 'Home'}
          </button>
          <button onClick={() => navigate("/chat")} class="btn-chat-nav">{C.chat}</button>
        </div>
      </div>
    </header>
  )
}

export default function PublicRoadmapPage() {
  const R = () => CONTENT.roadmap
  const navigate = useNavigate()

  return (
    <div class="hone-roadmap-v4">
      <AnimatedBackground />
      <Header />
      
      <main class="main-content">
        <section class="roadmap-hero">
          <div class="meta">{R().hero_meta}</div>
          <h1 class="title">{R().hero_title}</h1>
          <p class="subtitle">{R().hero_sub}</p>
        </section>

        <div class="roadmap-grid">
          {/* Timeline / Roadmap Phases */}
          <section class="roadmap-card">
            <h2 class="card-title">{R().sections.roadmap.title}</h2>
            <div class="phases-container">
              <div class="phase">
                <div class="phase-tag now">{R().now.label}</div>
                <ul class="phase-list">
                  {R().now.items.map(item => <li>{item}</li>)}
                </ul>
              </div>
              <div class="phase">
                <div class="phase-tag next">{R().next.label}</div>
                <ul class="phase-list">
                  {R().next.items.map(item => <li>{item}</li>)}
                </ul>
              </div>
              <div class="phase">
                <div class="phase-tag later">{R().later.label}</div>
                <ul class="phase-list">
                  {R().later.items.map(item => <li>{item}</li>)}
                </ul>
              </div>
            </div>
          </section>

          {/* Capability Matrix */}
          <section class="roadmap-card">
            <h2 class="card-title">{R().sections.capabilities.title}</h2>
            <div class="matrix-container">
              {R().capability_matrix.map(group => (
                <div class="matrix-group">
                  <div class="group-name">{group.group}</div>
                  {group.rows.map(row => (
                    <div class="matrix-row">
                      <span class="row-name">{row.name}</span>
                      <span class={`status-badge ${row.status}`}>{R().sections.capabilities.legend[row.status as 'stable' | 'beta' | 'planned']}</span>
                    </div>
                  ))}
                </div>
              ))}
            </div>
          </section>

          {/* Quick Start */}
          <section id="quick-start" class="roadmap-card dark">
            <h2 class="card-title">{R().sections.quick_start.title}</h2>
            <p class="card-intro">{R().sections.quick_start.intro}</p>
            <div class="code-block">
              <pre><code>{R().install.curl.join('\n')}</code></pre>
            </div>
          </section>
        </div>

        <section class="bottom-cta">
          <h2>{R().bottom_cta.title}</h2>
          <button onClick={() => navigate("/chat")} class="btn-primary large">
            {R().bottom_cta.primary}
          </button>
        </section>
      </main>

      <style>{`
        .hone-roadmap-v4 {
          background: #fff; color: #1e293b;
          font-family: var(--font-sans, 'Plus Jakarta Sans', sans-serif);
          min-height: 100vh; position: relative;
          display: flex; flex-direction: column;
        }

        .animated-bg { position: absolute; inset: 0; z-index: 0; overflow: hidden; pointer-events: none; }
        .circle { position: absolute; border-radius: 50%; filter: blur(100px); opacity: 0.08; animation: float 25s infinite alternate ease-in-out; }
        .circle-1 { width: 800px; height: 800px; background: #f59e0b; top: -200px; left: -150px; }
        .circle-2 { width: 900px; height: 900px; background: #3b82f6; bottom: -250px; right: -150px; animation-delay: -5s; }
        .circle-3 { width: 400px; height: 400px; background: #ec4899; top: 40%; left: 50%; animation-delay: -10s; }
        @keyframes float { 0% { transform: translate(0, 0) scale(1); } 100% { transform: translate(60px, 120px) scale(1.15); } }

        .page-header {
          position: fixed; top: 0; left: 0; right: 0; height: 72px;
          display: flex; align-items: center; justify-content: space-between;
          padding: 0 40px; background: rgba(255, 255, 255, 0.85);
          backdrop-filter: blur(16px); border-bottom: 1px solid rgba(0,0,0,0.04); z-index: 100;
        }
        .header-logo { display: flex; align-items: center; gap: 12px; cursor: pointer; }
        .header-logo img { height: 32px; }
        .header-logo span { font-weight: 800; font-size: 22px; color: #000; letter-spacing: -0.01em; }
        .header-actions { display: flex; align-items: center; gap: 24px; }
        .lang-switch { display: flex; background: #f1f5f9; padding: 3px; border-radius: 10px; }
        .lang-switch button { padding: 4px 14px; border: none; background: transparent; cursor: pointer; font-size: 13px; font-weight: 700; color: #64748b; }
        .lang-switch button.active { background: #fff; color: #000; border-radius: 7px; box-shadow: 0 2px 4px rgba(0,0,0,0.06); }
        .btn-chat-nav { background: #000; color: #fff; border: none; padding: 10px 24px; border-radius: 100px; font-size: 14px; font-weight: 700; cursor: pointer; }
        .btn-roadmap-nav { background: transparent; color: #64748b; border: 1.5px solid #e2e8f0; padding: 8px 20px; border-radius: 100px; font-size: 14px; font-weight: 700; cursor: pointer; transition: all 0.2s; }

        .main-content { position: relative; z-index: 1; padding: 120px 40px 80px; max-width: 1200px; margin: 0 auto; width: 100%; }

        .roadmap-hero { text-align: center; margin-bottom: 80px; }
        .roadmap-hero .meta { font-size: 14px; font-weight: 800; color: #f59e0b; letter-spacing: 0.2em; margin-bottom: 16px; }
        .roadmap-hero .title { font-size: 56px; font-weight: 800; color: #0f172a; margin-bottom: 24px; letter-spacing: -0.02em; }
        .roadmap-hero .subtitle { font-size: 20px; color: #475569; max-width: 800px; margin: 0 auto; line-height: 1.6; }

        .roadmap-grid { display: grid; gap: 40px; }
        .roadmap-card { background: #fff; border: 1.5px solid #f1f5f9; border-radius: 32px; padding: 48px; box-shadow: 0 20px 50px rgba(0,0,0,0.02); }
        .roadmap-card.dark { background: #0f172a; color: #fff; border: none; }
        .card-title { font-size: 32px; font-weight: 800; margin-bottom: 32px; }

        .phases-container { display: grid; grid-template-columns: repeat(3, 1fr); gap: 32px; }
        .phase-tag { display: inline-block; padding: 4px 12px; border-radius: 8px; font-size: 12px; font-weight: 800; margin-bottom: 20px; }
        .phase-tag.now { background: #fff7ed; color: #d97706; }
        .phase-tag.next { background: #eff6ff; color: #1d4ed8; }
        .phase-tag.later { background: #fdf2f8; color: #be185d; }
        .phase-list { list-style: none; padding: 0; margin: 0; display: grid; gap: 12px; }
        .phase-list li { font-size: 15px; color: #475569; line-height: 1.4; display: flex; gap: 10px; }
        .phase-list li::before { content: '→'; color: #cbd5e1; }
        .roadmap-card.dark .phase-list li { color: #94a3b8; }

        .matrix-container { display: grid; grid-template-columns: repeat(2, 1fr); gap: 40px; }
        .matrix-group { display: flex; flex-direction: column; gap: 16px; }
        .group-name { font-size: 14px; font-weight: 800; color: #94a3b8; letter-spacing: 0.1em; text-transform: uppercase; border-bottom: 1px solid #f1f5f9; padding-bottom: 8px; }
        .matrix-row { display: flex; justify-content: space-between; align-items: center; padding: 8px 0; border-bottom: 1px solid #f8fafc; }
        .row-name { font-size: 16px; font-weight: 600; color: #1e293b; }
        .status-badge { font-size: 11px; font-weight: 800; padding: 2px 8px; border-radius: 6px; }
        .status-badge.stable { background: #ecfdf5; color: #059669; }
        .status-badge.beta { background: #fffbeb; color: #d97706; }
        .status-badge.planned { background: #f1f5f9; color: #64748b; }

        .code-block { background: rgba(0,0,0,0.3); padding: 24px; border-radius: 16px; font-family: monospace; font-size: 14px; color: #94a3b8; overflow-x: auto; }
        .code-block code { white-space: pre; }

        .bottom-cta { text-align: center; margin-top: 100px; padding: 80px; background: #f8fafc; border-radius: 40px; }
        .bottom-cta h2 { font-size: 32px; font-weight: 800; margin-bottom: 32px; }
        .btn-primary.large { background: #000; color: #fff; border: none; padding: 18px 48px; border-radius: 100px; font-size: 18px; font-weight: 700; cursor: pointer; transition: transform 0.2s; }
        .btn-primary.large:hover { transform: scale(1.05); }

        @media (max-width: 768px) {
          .phases-container, .matrix-container { grid-template-columns: 1fr; }
          .roadmap-hero .title { font-size: 36px; }
          .roadmap-card { padding: 24px; border-radius: 24px; }
          .main-content { padding: 100px 20px 40px; }
        }
      `}</style>
    </div>
  )
}
