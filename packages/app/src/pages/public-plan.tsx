// public-plan.tsx — Plan 与定价（预留页）：开源自托管 + 云端 Pro/Team 档位预告。
// 视觉遵循 HONE foundation 令牌与工作台面板语言，价格权益以正式上线为准。

import { For, Show } from "solid-js"
import { CONTENT } from "@/lib/public-content"
import { PublicFooter, PublicNav } from "@/components/public-nav"
import "./public-site.css"

export default function PublicPlanPage() {
  const C = CONTENT.plan
  const nav = CONTENT.nav

  const tierHref = (id: string) =>
    id === "free" ? nav.github_url : id === "team" ? `mailto:${nav.contact_email}` : null

  return (
    <div class="pub-page hone-plan-page">
      <PublicNav />
      <main class="hone-plan-main">
        <header class="hone-plan-hero">
          <div class="hone-plan-eyebrow">{C.eyebrow}</div>
          <h1>{C.title}</h1>
          <p>{C.sub}</p>
        </header>

        <section class="hone-plan-grid" aria-label={C.eyebrow}>
          <For each={C.tiers}>
            {(tier) => {
              const href = tierHref(tier.id)
              const soon = tier.id === "pro"
              return (
                <article class={`hone-plan-card ${tier.id === "pro" ? "is-featured" : ""}`}>
                  <div class="hone-plan-card-head">
                    <h2>{tier.name}</h2>
                    <span class={`hone-plan-badge ${soon ? "is-soon" : ""}`}>
                      {soon ? C.badge_soon : tier.id === "free" ? C.badge_current : C.badge_soon}
                    </span>
                  </div>
                  <div class="hone-plan-price">
                    <strong>{tier.price}</strong>
                    <small>{tier.period}</small>
                  </div>
                  <p class="hone-plan-desc">{tier.desc}</p>
                  <ul class="hone-plan-features">
                    <For each={tier.features}>
                      {(feature) => (
                        <li>
                          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 6 9 17l-5-5" /></svg>
                          <span>{feature}</span>
                        </li>
                      )}
                    </For>
                  </ul>
                  <Show
                    when={href}
                    fallback={
                      <button type="button" class="hone-plan-cta" disabled>
                        {tier.cta}
                      </button>
                    }
                  >
                    <a
                      class={`hone-plan-cta ${tier.id === "free" ? "is-primary" : ""}`}
                      href={href!}
                      target={tier.id === "free" ? "_blank" : undefined}
                      rel={tier.id === "free" ? "noopener noreferrer" : undefined}
                    >
                      {tier.cta}
                    </a>
                  </Show>
                </article>
              )
            }}
          </For>
        </section>

        <aside class="hone-plan-note">{C.note}</aside>
      </main>
      <PublicFooter />

      <style>{`
        .hone-plan-page {
          min-height: 100vh;
          display: flex;
          flex-direction: column;
          background:
            radial-gradient(820px 400px at 82% -60px, color-mix(in srgb, var(--hone-coral-500) 9%, transparent), transparent 70%),
            var(--hone-paper-50);
          color: var(--hone-ink-800);
          font-family: var(--hone-font-body);
        }
        .hone-plan-main {
          width: min(1040px, calc(100% - 40px));
          margin: 0 auto;
          padding: 148px 0 96px;
          flex: 1;
        }
        .hone-plan-hero { max-width: 620px; }
        .hone-plan-eyebrow {
          color: var(--hone-coral-600);
          font-family: var(--hone-font-label);
          font-size: 11px;
          font-weight: 700;
          letter-spacing: 0.16em;
        }
        .hone-plan-hero h1 {
          margin: 14px 0 0;
          color: var(--hone-ink-950);
          font-size: clamp(30px, 4vw, 42px);
          line-height: 1.08;
          letter-spacing: -0.04em;
        }
        .hone-plan-hero p {
          margin: 14px 0 0;
          color: var(--hone-ink-600);
          font-size: 14px;
          line-height: 1.75;
        }
        .hone-plan-grid {
          display: grid;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          gap: 14px;
          margin-top: 44px;
        }
        .hone-plan-card {
          display: flex;
          flex-direction: column;
          padding: 26px 24px 24px;
          border: 1px solid var(--hone-line);
          border-radius: 17px;
          background: #fff;
          transition: border-color 0.18s var(--hone-ease), transform 0.18s var(--hone-ease), box-shadow 0.18s var(--hone-ease);
        }
        .hone-plan-card:hover {
          transform: translateY(-3px);
          border-color: var(--hone-line-strong);
          box-shadow: var(--hone-shadow-md);
        }
        .hone-plan-card.is-featured {
          border-color: color-mix(in srgb, var(--hone-coral-500) 42%, var(--hone-line));
          background: linear-gradient(180deg, color-mix(in srgb, var(--hone-coral-500) 4%, #fff), #fff 52%);
          box-shadow: var(--hone-shadow-sm);
        }
        .hone-plan-card-head {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 10px;
        }
        .hone-plan-card-head h2 {
          margin: 0;
          color: var(--hone-ink-950);
          font-size: 15px;
          font-weight: 700;
          letter-spacing: -0.02em;
        }
        .hone-plan-badge {
          padding: 3px 9px;
          border: 1px solid var(--hone-line);
          border-radius: 999px;
          background: var(--hone-paper-100);
          color: var(--hone-ink-600);
          font-size: 10px;
          font-weight: 650;
          white-space: nowrap;
        }
        .hone-plan-badge.is-soon {
          border-color: color-mix(in srgb, var(--hone-coral-500) 36%, transparent);
          background: color-mix(in srgb, var(--hone-coral-500) 9%, #fff);
          color: var(--hone-coral-600);
        }
        .hone-plan-price {
          display: flex;
          align-items: baseline;
          gap: 8px;
          margin-top: 22px;
        }
        .hone-plan-price strong {
          color: var(--hone-ink-950);
          font-size: 27px;
          font-weight: 800;
          letter-spacing: -0.03em;
          line-height: 1;
        }
        .hone-plan-price small {
          color: var(--hone-ink-400);
          font-size: 11px;
          font-weight: 650;
        }
        .hone-plan-desc {
          margin: 10px 0 0;
          color: var(--hone-ink-600);
          font-size: 12px;
          line-height: 1.6;
        }
        .hone-plan-features {
          display: grid;
          gap: 9px;
          margin: 20px 0 24px;
          padding: 20px 0 0;
          border-top: 1px solid var(--hone-line);
          list-style: none;
          flex: 1;
        }
        .hone-plan-features li {
          display: flex;
          align-items: flex-start;
          gap: 9px;
          color: var(--hone-ink-800);
          font-size: 12px;
          line-height: 1.5;
        }
        .hone-plan-features svg {
          width: 13px;
          height: 13px;
          flex: 0 0 13px;
          margin-top: 2px;
          color: var(--hone-coral-500);
        }
        .hone-plan-cta {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          min-height: 40px;
          padding: 0 16px;
          border: 1px solid var(--hone-line-strong);
          border-radius: var(--hone-radius-sm);
          background: #fff;
          color: var(--hone-ink-950);
          cursor: pointer;
          font-size: 12px;
          font-weight: 700;
          text-decoration: none;
          transition: background 0.16s ease, border-color 0.16s ease, color 0.16s ease;
        }
        .hone-plan-cta:hover { border-color: var(--hone-ink-950); }
        .hone-plan-cta.is-primary {
          border-color: var(--hone-ink-950);
          background: var(--hone-ink-950);
          color: #fff;
        }
        .hone-plan-cta.is-primary:hover { background: var(--hone-ink-800); }
        .hone-plan-cta:disabled {
          border-color: var(--hone-line);
          background: var(--hone-paper-100);
          color: var(--hone-ink-400);
          cursor: default;
        }
        .hone-plan-note {
          margin-top: 26px;
          padding: 16px 18px;
          border: 1px dashed var(--hone-line-strong);
          border-radius: var(--hone-radius-md);
          background: color-mix(in srgb, var(--hone-paper-100) 72%, transparent);
          color: var(--hone-ink-600);
          font-size: 11px;
          line-height: 1.7;
        }
        @media (max-width: 900px) {
          .hone-plan-main { width: calc(100% - 32px); padding-top: 118px; }
          .hone-plan-grid { grid-template-columns: 1fr; gap: 12px; }
          .hone-plan-card.is-featured { order: -1; }
        }
      `}</style>
    </div>
  )
}
