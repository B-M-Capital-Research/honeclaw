// public-roadmap.tsx — Hone Public Site Roadmap (content-driven)

import { createResource } from "solid-js"
import { useNavigate } from "@solidjs/router"
import { displayGithubStars, fetchGithubStars } from "@/lib/github-stars"
import { CONTENT } from "@/lib/public-content"
import { setLocale, useLocale } from "@/lib/i18n"
import { PublicContactMenu } from "@/components/public-contact-menu"
import "./public-site.css"

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
  const [stars] = createResource(fetchGithubStars)
  const C = CONTENT.nav

  return (
    <header class="page-header">
      <div onClick={() => navigate("/")} class="header-logo">
        <img src="/logo.svg" alt="Hone" />
        <span>Hone</span>
      </div>

      <div class="header-actions">
        <PublicContactMenu />

        <a href={C.github_url} target="_blank" rel="noopener noreferrer" class="star-badge header-github-stars">
          <span>GitHub</span>
          <span>{displayGithubStars(stars())}</span>
        </a>

        <div class="lang-switch">
          <button onClick={() => setLocale("zh")} class={useLocale() === "zh" ? "active" : ""}>中</button>
          <button onClick={() => setLocale("en")} class={useLocale() === "en" ? "active" : ""}>EN</button>
        </div>

        <div style={{ display: "flex", gap: "8px" }}>
          <button onClick={() => navigate("/")} class="btn-roadmap-nav mobile-hide">
            {C.back_home}
          </button>
          <button onClick={() => navigate("/chat")} class="btn-chat-nav">{C.chat}</button>
        </div>
      </div>
    </header>
  )
}

function SectionHeader(props: { eyebrow: string; title: string; intro?: string }) {
  return (
    <div class="section-head">
      <div class="section-eyebrow">{props.eyebrow}</div>
      <h2 class="card-title">{props.title}</h2>
      {props.intro ? <p class="card-intro">{props.intro}</p> : null}
    </div>
  )
}

function InstallCommands(props: { tab: { key: "curl" | "brew" | "source"; label: string; badge: string | null } }) {
  const R = () => CONTENT.roadmap
  const lines = () => {
    if (props.tab.key === "brew") return R().install.brew
    if (props.tab.key === "source") return R().install.source
    return R().install.curl
  }

  return (
    <div class="install-panel">
      <div class="install-label">
        <span>{props.tab.label}</span>
        {props.tab.badge ? <em>{props.tab.badge}</em> : null}
      </div>
      <div class="code-block">
        <pre><code>{lines().join("\n")}</code></pre>
      </div>
    </div>
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
          <div class="version-pill">{R().version}</div>
        </section>

        <section id="quick-start" class="roadmap-card dark">
          <SectionHeader
            eyebrow={R().sections.quick_start.eyebrow}
            title={R().sections.quick_start.title}
            intro={`${R().sections.quick_start.intro} ${R().install.requirements_prefix} ${R().requirements}`}
          />
          <div class="install-grid">
            {R().install.tabs.map(tab => <InstallCommands tab={tab} />)}
          </div>
        </section>

        <section id="roadmap" class="roadmap-card">
          <SectionHeader
            eyebrow={R().sections.roadmap.eyebrow}
            title={R().sections.roadmap.title}
            intro={`${R().sections.roadmap.intro_lead} ${R().sections.roadmap.intro_highlight} ${R().sections.roadmap.intro_trail}`}
          />
          <div class="phases-container">
            <div class="phase">
              <div class="phase-tag now">{R().now.label}</div>
              <ul class="phase-list">{R().now.items.map(item => <li>{item}</li>)}</ul>
            </div>
            <div class="phase">
              <div class="phase-tag next">{R().next.label}</div>
              <ul class="phase-list">{R().next.items.map(item => <li>{item}</li>)}</ul>
            </div>
            <div class="phase">
              <div class="phase-tag later">{R().later.label}</div>
              <ul class="phase-list">{R().later.items.map(item => <li>{item}</li>)}</ul>
            </div>
          </div>
        </section>

        <section id="capabilities" class="roadmap-card">
          <SectionHeader
            eyebrow={R().sections.capabilities.eyebrow}
            title={R().sections.capabilities.title}
          />
          <div class="matrix-container">
            {R().capability_matrix.map(group => (
              <div class="matrix-group">
                <div class="group-name">{group.group}</div>
                {group.rows.map(row => (
                  <div class="matrix-row">
                    <div>
                      <span class="row-name">{row.name}</span>
                      <span class="row-note">{row.note}</span>
                    </div>
                    <span class={`status-badge ${row.status}`}>
                      {R().sections.capabilities.legend[row.status as "stable" | "beta" | "planned"]}
                    </span>
                  </div>
                ))}
              </div>
            ))}
          </div>
        </section>

        <section id="channels" class="roadmap-card">
          <SectionHeader
            eyebrow={R().sections.channels.eyebrow}
            title={R().sections.channels.title}
            intro={R().sections.channels.intro}
          />
          <div class="channel-grid">
            {R().channels.map(channel => (
              <div class="channel-row">
                <span class="channel-icon">{channel.icon}</span>
                <div>
                  <div class="channel-title">
                    <span>{channel.name}</span>
                    <span class={`status-badge ${channel.status}`}>{R().sections.capabilities.legend[channel.status as "stable" | "beta" | "planned"]}</span>
                  </div>
                  <p>{channel.desc}</p>
                </div>
              </div>
            ))}
          </div>
        </section>

        <section id="architecture" class="roadmap-card">
          <SectionHeader
            eyebrow={R().sections.architecture.eyebrow}
            title={R().sections.architecture.title}
            intro={R().sections.architecture.intro}
          />
          <div class="architecture-grid">
            {R().architecture_points.map(point => (
              <div class="architecture-point">
                <h3>{point.title}</h3>
                <p>{point.desc}</p>
              </div>
            ))}
          </div>
          <a class="text-link" href="https://github.com/B-M-Capital-Research/honeclaw/blob/main/docs/repo-map.md">
            {R().sections.architecture.footnote_prefix} {R().sections.architecture.footnote_link}
          </a>
        </section>

        <section id="skills" class="roadmap-card">
          <SectionHeader
            eyebrow={R().sections.skills.eyebrow}
            title={R().sections.skills.title}
            intro={`${R().sections.skills.intro_prefix} skills/ ${R().sections.skills.intro_suffix}`}
          />
          <div class="skills-grid">
            {R().skills.map(skill => (
              <div class="skill-row">
                <code>{skill.name}</code>
                <p>{skill.desc}</p>
              </div>
            ))}
          </div>
        </section>

        <section id="boundary" class="roadmap-card">
          <SectionHeader
            eyebrow={R().sections.boundary.eyebrow}
            title={R().sections.boundary.title}
            intro={R().sections.boundary.intro}
          />
          <div class="boundary-grid">
            <div>
              <h3>{R().sections.boundary.open_label}</h3>
              <ul class="phase-list">{R().boundary.open.map(item => <li>{item}</li>)}</ul>
            </div>
            <div>
              <h3>{R().sections.boundary.closed_label}</h3>
              <ul class="phase-list">{R().boundary.closed.map(item => <li>{item}</li>)}</ul>
            </div>
          </div>
        </section>

        <section id="docs" class="roadmap-card">
          <SectionHeader eyebrow={R().sections.docs.eyebrow} title={R().sections.docs.title} />
          <div class="docs-grid">
            {R().docs.map(doc => (
              <a class="doc-card" href={doc.url} target="_blank" rel="noreferrer">
                <strong>{doc.title}</strong>
                <span>{doc.desc}</span>
              </a>
            ))}
          </div>
        </section>

        <section id="contributing" class="roadmap-card">
          <SectionHeader
            eyebrow={R().sections.contributing.eyebrow}
            title={R().sections.contributing.title}
            intro={R().sections.contributing.intro}
          />
          <div class="contrib-grid">
            {R().contributing.map(item => (
              <a class="contrib-card" href={item.href} target="_blank" rel="noreferrer">
                <span>{item.icon}</span>
                <strong>{item.title}</strong>
                <p>{item.desc}</p>
              </a>
            ))}
          </div>
        </section>

        <section id="faq" class="roadmap-card">
          <SectionHeader eyebrow={R().sections.faq.eyebrow} title={R().sections.faq.title} />
          <div class="faq-list">
            {R().faqs.map(item => (
              <details>
                <summary>{item.q}</summary>
                <p>{item.a}</p>
              </details>
            ))}
          </div>
        </section>

        <section class="bottom-cta">
          <h2>{R().bottom_cta.title}</h2>
          <p>{R().bottom_cta.desc}</p>
          <button onClick={() => navigate("/chat")} class="btn-primary large">
            {R().bottom_cta.primary}
          </button>
        </section>
      </main>

      <style>{`
        .hone-roadmap-v4 {
          background: #fff;
          color: #1e293b;
          font-family: var(--font-sans, 'Plus Jakarta Sans', sans-serif);
          min-height: 100vh;
          position: relative;
          display: flex;
          flex-direction: column;
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
        .header-logo span { font-weight: 800; font-size: 22px; color: #000; }
        .header-actions { display: flex; align-items: center; gap: 24px; }
        .lang-switch { display: flex; background: #f1f5f9; padding: 3px; border-radius: 10px; }
        .lang-switch button { padding: 4px 14px; border: none; background: transparent; cursor: pointer; font-size: 13px; font-weight: 700; color: #64748b; }
        .lang-switch button.active { background: #fff; color: #000; border-radius: 7px; box-shadow: 0 2px 4px rgba(0,0,0,0.06); }
        .btn-chat-nav { background: #000; color: #fff; border: none; padding: 10px 24px; border-radius: 100px; font-size: 14px; font-weight: 700; cursor: pointer; }
        .btn-roadmap-nav { background: transparent; color: #64748b; border: 1.5px solid #e2e8f0; padding: 8px 20px; border-radius: 100px; font-size: 14px; font-weight: 700; cursor: pointer; transition: all 0.2s; }

        .main-content { position: relative; z-index: 1; padding: 120px 40px 80px; max-width: 1200px; margin: 0 auto; width: 100%; display: grid; gap: 40px; }
        .roadmap-hero { text-align: center; margin-bottom: 32px; }
        .roadmap-hero .meta { font-size: 14px; font-weight: 800; color: #f59e0b; letter-spacing: 0.2em; margin-bottom: 16px; }
        .roadmap-hero .title { font-size: 56px; font-weight: 800; color: #0f172a; margin-bottom: 24px; }
        .roadmap-hero .subtitle { font-size: 20px; color: #475569; max-width: 820px; margin: 0 auto; line-height: 1.6; }
        .version-pill { display: inline-flex; margin-top: 24px; padding: 7px 13px; border: 1px solid #e2e8f0; border-radius: 999px; background: #fff; font-size: 13px; font-weight: 800; color: #0f172a; }

        .roadmap-card { background: #fff; border: 1.5px solid #f1f5f9; border-radius: 24px; padding: 40px; box-shadow: 0 20px 50px rgba(0,0,0,0.02); }
        .roadmap-card.dark { background: #0f172a; color: #fff; border: none; }
        .section-head { margin-bottom: 28px; }
        .section-eyebrow { font-size: 12px; font-weight: 800; color: #f59e0b; letter-spacing: 0.16em; margin-bottom: 10px; }
        .card-title { font-size: 30px; font-weight: 800; margin: 0; color: #0f172a; }
        .roadmap-card.dark .card-title { color: #fff; }
        .card-intro { margin: 12px 0 0; color: #64748b; line-height: 1.7; max-width: 920px; }
        .roadmap-card.dark .card-intro { color: #cbd5e1; }

        .install-grid, .phases-container, .architecture-grid, .boundary-grid, .contrib-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 20px; }
        .install-label { display: flex; align-items: center; gap: 8px; margin-bottom: 10px; font-weight: 800; color: #e2e8f0; }
        .install-label em { font-style: normal; font-size: 11px; color: #fbbf24; }
        .code-block { background: rgba(0,0,0,0.3); padding: 18px; border-radius: 14px; font-family: var(--font-mono, monospace); font-size: 13px; color: #cbd5e1; overflow-x: auto; }
        .code-block pre { margin: 0; }
        .code-block code { white-space: pre; }

        .phase-tag { display: inline-block; padding: 4px 12px; border-radius: 8px; font-size: 12px; font-weight: 800; margin-bottom: 16px; }
        .phase-tag.now { background: #fff7ed; color: #d97706; }
        .phase-tag.next { background: #eff6ff; color: #1d4ed8; }
        .phase-tag.later { background: #fdf2f8; color: #be185d; }
        .phase-list { list-style: none; padding: 0; margin: 0; display: grid; gap: 11px; }
        .phase-list li { font-size: 15px; color: #475569; line-height: 1.5; display: flex; gap: 10px; }
        .phase-list li::before { content: '->'; color: #cbd5e1; flex: 0 0 auto; }

        .matrix-container { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 28px; }
        .matrix-group { display: flex; flex-direction: column; gap: 12px; min-width: 0; }
        .group-name { font-size: 13px; font-weight: 800; color: #94a3b8; letter-spacing: 0.1em; text-transform: uppercase; border-bottom: 1px solid #f1f5f9; padding-bottom: 8px; }
        .matrix-row { display: flex; justify-content: space-between; align-items: flex-start; gap: 14px; padding: 10px 0; border-bottom: 1px solid #f8fafc; }
        .row-name { display: block; font-size: 15px; font-weight: 700; color: #1e293b; line-height: 1.35; }
        .row-note { display: block; margin-top: 4px; font-size: 12px; color: #64748b; line-height: 1.4; }
        .status-badge { flex: 0 0 auto; font-size: 11px; font-weight: 800; padding: 3px 8px; border-radius: 6px; }
        .status-badge.stable { background: #ecfdf5; color: #059669; }
        .status-badge.beta { background: #fffbeb; color: #d97706; }
        .status-badge.planned { background: #f1f5f9; color: #64748b; }

        .channel-grid, .skills-grid, .docs-grid, .faq-list { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
        .channel-row, .skill-row, .architecture-point, .doc-card, .contrib-card, details { border: 1px solid #e2e8f0; border-radius: 14px; padding: 18px; background: #fff; min-width: 0; }
        .channel-row { display: grid; grid-template-columns: auto minmax(0, 1fr); gap: 14px; }
        .channel-icon { font-size: 20px; color: #f59e0b; }
        .channel-title { display: flex; justify-content: space-between; gap: 10px; align-items: center; font-weight: 800; color: #0f172a; }
        .channel-row p, .skill-row p, .architecture-point p, .contrib-card p, details p { margin: 8px 0 0; color: #64748b; line-height: 1.55; }
        .architecture-grid, .boundary-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
        .architecture-point h3, .boundary-grid h3 { margin: 0 0 10px; font-size: 16px; color: #0f172a; }
        .text-link { display: inline-block; margin-top: 22px; color: #d97706; font-weight: 800; text-decoration: none; }
        .skill-row code { display: block; font-size: 13px; font-weight: 800; color: #b45309; overflow-wrap: anywhere; }
        .docs-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
        .doc-card, .contrib-card { color: inherit; text-decoration: none; transition: border-color 0.2s, transform 0.2s; }
        .doc-card:hover, .contrib-card:hover { border-color: #f59e0b; transform: translateY(-1px); }
        .doc-card strong, .contrib-card strong { display: block; color: #0f172a; }
        .doc-card span { display: block; margin-top: 8px; color: #64748b; line-height: 1.45; }
        .contrib-card span { color: #f59e0b; font-size: 20px; }
        details { cursor: pointer; }
        summary { font-weight: 800; color: #0f172a; }

        .bottom-cta { text-align: center; margin-top: 32px; padding: 64px 32px; background: #f8fafc; border-radius: 30px; }
        .bottom-cta h2 { font-size: 32px; font-weight: 800; margin: 0 0 12px; color: #0f172a; }
        .bottom-cta p { margin: 0 0 28px; color: #64748b; }
        .btn-primary.large { background: #000; color: #fff; border: none; padding: 16px 40px; border-radius: 100px; font-size: 17px; font-weight: 700; cursor: pointer; transition: transform 0.2s; }
        .btn-primary.large:hover { transform: scale(1.03); }

        @media (max-width: 960px) {
          .install-grid, .phases-container, .matrix-container, .channel-grid, .skills-grid, .architecture-grid, .boundary-grid, .docs-grid, .contrib-grid, .faq-list { grid-template-columns: 1fr; }
        }

        @media (max-width: 768px) {
          .roadmap-hero .title { font-size: 36px; }
          .roadmap-card { padding: 24px; border-radius: 22px; }
          .main-content { padding: 100px 20px 40px; }
          .channel-title { align-items: flex-start; flex-direction: column; }
        }
      `}</style>
    </div>
  )
}
