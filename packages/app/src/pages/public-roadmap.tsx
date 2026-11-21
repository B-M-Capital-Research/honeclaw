// public-roadmap.tsx — HONE Public Site Roadmap (content-driven)

import { Title } from "@solidjs/meta"
import { useNavigate } from "@solidjs/router"
import { CONTENT } from "@/lib/public-content"
import { PublicFooter, PublicNav } from "@/components/public-nav"
import "./public-site.css"

function AnimatedBackground() {
  return <div class="animated-bg" aria-hidden="true" />
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
    <div class="pub-page hone-roadmap-v4">
      <Title>{R().hero_title} | HONE</Title>
      <AnimatedBackground />
      <PublicNav />

      <main class="main-content">
        <section class="roadmap-hero">
          <div class="roadmap-hero-top">
            <div class="meta">{R().hero_meta}</div>
            <div class="version-pill">{R().version}</div>
          </div>
          <h1 class="title">{R().hero_title}</h1>
          <p class="subtitle">{R().hero_sub}</p>
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
                <span class="channel-icon" aria-hidden="true">{channel.name.slice(0, 1)}</span>
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
                <strong class="skill-row-name">{skill.desc}</strong>
                <code class="skill-row-id">{skill.name}</code>
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
          <div class="bottom-cta-copy">
            <h2>{R().bottom_cta.title}</h2>
            <p>{R().bottom_cta.desc}</p>
          </div>
          <button onClick={() => navigate("/chat")} class="btn-primary large">
            {R().bottom_cta.primary} →
          </button>
        </section>
      </main>

      <PublicFooter />

      <style>{`
        .hone-roadmap-v4 {
          background: var(--hone-paper-50);
          color: var(--hone-ink-800);
          font-family: var(--hone-font-body);
          font-size: 14px;
          line-height: 1.7;
          min-height: 100vh;
          position: relative;
          display: flex;
          flex-direction: column;
        }

        .animated-bg {
          position: absolute;
          inset: 0 0 auto;
          height: 680px;
          z-index: 0;
          overflow: hidden;
          pointer-events: none;
          background-image:
            radial-gradient(880px 380px at 78% 0%, color-mix(in srgb, var(--hone-coral-500) 8%, transparent), transparent 72%),
            linear-gradient(to right, color-mix(in srgb, var(--hone-line) 50%, transparent) 1px, transparent 1px),
            linear-gradient(to bottom, color-mix(in srgb, var(--hone-line) 50%, transparent) 1px, transparent 1px);
          background-size: 100% 100%, 64px 64px, 64px 64px;
          mask-image: linear-gradient(to bottom, rgba(0, 0, 0, 0.9) 0%, rgba(0, 0, 0, 0.3) 65%, transparent 100%);
          -webkit-mask-image: linear-gradient(to bottom, rgba(0, 0, 0, 0.9) 0%, rgba(0, 0, 0, 0.3) 65%, transparent 100%);
        }

        .main-content {
          position: relative;
          z-index: 1;
          padding: 108px 0 80px;
          width: min(1160px, calc(100% - 40px));
          margin: 0 auto;
          flex: 1;
          display: grid;
          gap: 24px;
        }

        .roadmap-hero {
          padding: 20px 0 28px;
          border-bottom: 1px solid var(--hone-line);
          margin-bottom: 8px;
        }
        .roadmap-hero-top {
          display: flex;
          align-items: center;
          justify-content: space-between;
          flex-wrap: wrap;
          gap: 12px;
          margin-bottom: 12px;
        }
        .roadmap-hero .meta {
          font-family: var(--hone-font-label);
          font-size: 12px;
          font-weight: 600;
          color: var(--hone-coral-600);
          letter-spacing: 0.1em;
          text-transform: uppercase;
        }
        .roadmap-hero .title {
          font-family: var(--hone-font-display);
          font-size: clamp(30px, 3.8vw, 42px);
          font-weight: 700;
          line-height: 1.18;
          letter-spacing: -0.03em;
          color: var(--hone-ink-950);
          margin: 0 0 14px;
        }
        .roadmap-hero .subtitle {
          font-size: 16px;
          color: var(--hone-ink-600);
          max-width: 740px;
          margin: 0;
          line-height: 1.75;
        }
        .version-pill {
          display: inline-flex;
          align-items: center;
          padding: 4px 12px;
          border: 1px solid var(--hone-line-strong);
          border-radius: 2px;
          background: var(--hone-surface-raised);
          font-family: var(--hone-font-label);
          font-size: 12px;
          font-weight: 600;
          color: var(--hone-ink-950);
        }

        .roadmap-card {
          background: var(--hone-surface-raised);
          border: 1px solid var(--hone-line);
          border-radius: 2px;
          padding: 32px;
        }
        .roadmap-card.dark {
          background: var(--hone-inverse-bg);
          color: var(--hone-inverse-fg);
          border: 1px solid var(--hone-line-strong);
        }
        .section-head { margin-bottom: 24px; }
        .section-eyebrow {
          font-family: var(--hone-font-label);
          font-size: 12px;
          font-weight: 600;
          color: var(--hone-coral-600);
          letter-spacing: 0.1em;
          margin-bottom: 8px;
          text-transform: uppercase;
        }
        .roadmap-card.dark .section-eyebrow { color: #60a5fa; }
        .card-title {
          font-family: var(--hone-font-display);
          font-size: clamp(22px, 2.4vw, 28px);
          font-weight: 700;
          line-height: 1.25;
          letter-spacing: -0.02em;
          margin: 0;
          color: var(--hone-ink-950);
        }
        .roadmap-card.dark .card-title { color: var(--hone-inverse-fg); }
        .card-intro { margin: 10px 0 0; color: var(--hone-ink-600); font-size: 14px; line-height: 1.72; max-width: 880px; }
        .roadmap-card.dark .card-intro { color: var(--hone-inverse-muted); }

        .install-grid, .phases-container, .architecture-grid, .boundary-grid, .contrib-grid {
          display: grid;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          gap: 16px;
        }
        .install-label {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 8px;
          font-family: var(--hone-font-label);
          font-size: 12px;
          font-weight: 600;
          color: var(--hone-inverse-fg);
        }
        .install-label em {
          font-style: normal;
          padding: 1px 6px;
          border-radius: 2px;
          background: rgba(59, 130, 246, 0.22);
          font-size: 11px;
          color: #93c5fd;
        }
        .code-block {
          background: rgba(5, 8, 15, 0.55);
          border: 1px solid rgba(226, 232, 240, 0.14);
          padding: 16px;
          border-radius: 2px;
          font-family: var(--hone-font-label);
          font-size: 12px;
          line-height: 1.65;
          color: var(--hone-inverse-fg);
          overflow-x: auto;
        }
        .code-block pre { margin: 0; }
        .code-block code { white-space: pre; }

        .phase {
          padding: 20px;
          border: 1px solid var(--hone-line);
          border-radius: 2px;
          background: var(--hone-control-surface);
        }
        .phase-tag {
          display: inline-block;
          padding: 3px 10px;
          border: 1px solid var(--hone-line);
          border-radius: 2px;
          font-family: var(--hone-font-label);
          font-size: 12px;
          font-weight: 600;
          margin-bottom: 14px;
        }
        .phase-tag.now { background: var(--hone-signal-green-soft); color: var(--hone-signal-green-ink); border-color: color-mix(in srgb, var(--hone-signal-green) 35%, var(--hone-line)); }
        .phase-tag.next { background: color-mix(in srgb, var(--hone-coral-500) 10%, var(--hone-surface-raised)); color: var(--hone-coral-600); }
        .phase-tag.later { background: var(--hone-paper-100); color: var(--hone-ink-600); }
        .phase-list { list-style: none; padding: 0; margin: 0; display: grid; gap: 10px; }
        .phase-list li {
          font-size: 14px;
          color: var(--hone-ink-800);
          line-height: 1.65;
          display: flex;
          gap: 10px;
          align-items: baseline;
        }
        .phase-list li::before {
          content: "";
          flex: 0 0 6px;
          width: 6px;
          height: 6px;
          margin-top: 7px;
          background-color: var(--hone-coral-500);
        }

        .matrix-container { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 24px; }
        .matrix-group { display: flex; flex-direction: column; gap: 0; min-width: 0; }
        .group-name {
          font-family: var(--hone-font-label);
          font-size: 12px;
          font-weight: 600;
          color: var(--hone-ink-400);
          letter-spacing: 0.1em;
          text-transform: uppercase;
          border-bottom: 1px solid var(--hone-line-strong);
          padding-bottom: 10px;
        }
        .matrix-row {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          gap: 12px;
          padding: 12px 0;
          border-bottom: 1px solid var(--hone-line);
        }
        .row-name { display: block; font-size: 14px; font-weight: 650; color: var(--hone-ink-950); line-height: 1.4; }
        .row-note { display: block; margin-top: 4px; font-size: 12px; color: var(--hone-ink-500); line-height: 1.5; }
        .status-badge {
          flex: 0 0 auto;
          font-family: var(--hone-font-label);
          font-size: 11px;
          font-weight: 600;
          padding: 2px 8px;
          border-radius: 2px;
        }
        .status-badge.stable { background: #ecfdf5; color: #047857; }
        .status-badge.beta { background: color-mix(in srgb, var(--hone-coral-500) 10%, transparent); color: var(--hone-coral-600); }
        .status-badge.planned { background: var(--hone-paper-200); color: var(--hone-ink-600); }

        .channel-grid, .skills-grid, .docs-grid, .faq-list { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
        .channel-row, .skill-row, .architecture-point, .doc-card, .contrib-card, details {
          border: 1px solid var(--hone-line);
          border-radius: 2px;
          padding: 18px;
          background: var(--hone-control-surface);
          min-width: 0;
        }
        .channel-row { display: grid; grid-template-columns: auto minmax(0, 1fr); gap: 14px; }
        .channel-icon {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          width: 36px;
          height: 36px;
          border: 1px solid var(--hone-line);
          border-radius: 2px;
          background: var(--hone-paper-100);
          color: var(--hone-coral-600);
          font-family: var(--hone-font-label);
          font-size: 14px;
          font-weight: 700;
        }
        .channel-title { display: flex; justify-content: space-between; gap: 10px; align-items: center; font-size: 15px; font-weight: 650; color: var(--hone-ink-950); }
        .channel-row p, .skill-row p, .architecture-point p, .contrib-card p, details p { margin: 8px 0 0; color: var(--hone-ink-600); font-size: 14px; line-height: 1.68; }
        .architecture-grid, .boundary-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
        .architecture-point h3, .boundary-grid h3 { margin: 0 0 10px; font-size: 16px; font-weight: 650; color: var(--hone-ink-950); }
        .text-link { display: inline-block; margin-top: 20px; color: var(--hone-link); font-size: 14px; font-weight: 600; text-decoration: none; border-bottom: 1px solid var(--hone-coral-600); padding-bottom: 2px; }
        .skill-row { display: flex; flex-direction: column; gap: 8px; }
        .skill-row-name { font-size: 14px; font-weight: 600; color: var(--hone-ink-950); line-height: 1.5; }
        .skill-row-id {
          align-self: flex-start;
          padding: 2px 8px;
          border: 1px solid var(--hone-line);
          border-radius: 2px;
          background: var(--hone-paper-100);
          font-family: var(--hone-font-label);
          font-size: 12px;
          font-weight: 600;
          color: var(--hone-coral-600);
          overflow-wrap: anywhere;
        }
        .docs-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
        .doc-card, .contrib-card { color: inherit; text-decoration: none; transition: border-color 0.16s ease, background 0.16s ease; }
        .doc-card:hover, .contrib-card:hover { border-color: var(--hone-coral-500); background: var(--hone-paper-100); }
        .doc-card strong, .contrib-card strong { display: block; font-size: 15px; font-weight: 650; color: var(--hone-ink-950); }
        .doc-card span { display: block; margin-top: 6px; font-size: 13px; color: var(--hone-ink-600); line-height: 1.55; }
        .contrib-card span { display: inline-block; margin-bottom: 6px; color: var(--hone-coral-600); font-size: 18px; }
        details { cursor: pointer; }
        summary { font-size: 14px; font-weight: 650; color: var(--hone-ink-950); }

        .bottom-cta {
          display: flex;
          align-items: center;
          justify-content: space-between;
          flex-wrap: wrap;
          gap: 24px;
          margin-top: 16px;
          padding: 40px;
          border: 1px solid var(--hone-line-strong);
          border-radius: 2px;
          background: var(--hone-inverse-bg);
          color: var(--hone-inverse-fg);
        }
        .bottom-cta h2 { font-family: var(--hone-font-display); font-size: clamp(22px, 2.6vw, 28px); font-weight: 700; margin: 0 0 8px; color: var(--hone-inverse-fg); }
        .bottom-cta p { margin: 0; color: var(--hone-inverse-muted); font-size: 14px; }
        .btn-primary.large {
          background: var(--hone-action-bg);
          color: var(--hone-action-fg);
          border: 1px solid var(--hone-action-bg);
          padding: 0 24px;
          min-height: 44px;
          border-radius: 2px;
          font-size: 14px;
          font-weight: 600;
          cursor: pointer;
          white-space: nowrap;
          transition: background 0.16s ease, border-color 0.16s ease;
        }
        .btn-primary.large:hover {
          background: var(--hone-inverse-fg);
          border-color: var(--hone-inverse-fg);
          color: #090d14;
        }

        @media (max-width: 960px) {
          .install-grid, .phases-container, .matrix-container, .channel-grid, .skills-grid, .architecture-grid, .boundary-grid, .docs-grid, .contrib-grid, .faq-list { grid-template-columns: 1fr; }
        }

        @media (max-width: 768px) {
          .roadmap-card { padding: 20px; border-radius: 2px; }
          .main-content { width: calc(100% - 28px); padding: 88px 0 56px; }
          .channel-title { align-items: flex-start; flex-direction: column; }
          .bottom-cta { padding: 28px 20px; }
        }
      `}</style>
    </div>
  )
}
