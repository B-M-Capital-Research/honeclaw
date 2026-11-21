// public-home.tsx — HONE 官网首页（v7 · 机构级方角网格与 BamangTech 全新升级）
//
// 架构顺序：Hero & 交互式工作台终端 → 核心指标台账 → 01 产品矩阵 →
// 02 投资助手与交互式提问 → 03 研究台与洞察 → 04 hone-quant 量化引擎 →
// 05 巴芒科技（@BamangTech）视频与社区 → 06 投研与工程准则 → 底部行动区。
// 文案统一从 CONTENT.landing / CONTENT.quant 读取，保持中英双语同构。

import { Title } from "@solidjs/meta"
import { useNavigate } from "@solidjs/router"
import { For, Show, createMemo, createResource, createSignal, onCleanup, onMount, type JSX } from "solid-js"
import { CONTENT } from "@/lib/public-content"
import { latestPublicBlogPost } from "@/lib/public-blog"
import { displayGithubStars, fetchGithubStars } from "@/lib/github-stars"
import { PublicFooter, PublicNav } from "@/components/public-nav"
import "./public-site.css"
import "./public-home.css"

const Icon = {
  Arrow: () => (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="square" stroke-linejoin="miter" aria-hidden="true">
      <path d="M4 12h15M13 6l6 6-6 6" />
    </svg>
  ),
  UpRight: () => (
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="square" stroke-linejoin="miter" aria-hidden="true">
      <path d="M7 17L17 7M8 7h9v9" />
    </svg>
  ),
  Zoom: () => (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="square" aria-hidden="true">
      <path d="M15 4h5v5M9 20H4v-5M20 4l-6 6M4 20l6-6" />
    </svg>
  ),
  Github: () => (
    <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
    </svg>
  ),
  Check: () => (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" stroke-linecap="square" stroke-linejoin="miter" aria-hidden="true">
      <path d="M4.5 12.5l5 5L19.5 7.5" />
    </svg>
  ),
  Info: () => (
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="square" aria-hidden="true">
      <rect x="3" y="3" width="18" height="18" />
      <path d="M12 11v5M12 7.5v.5" />
    </svg>
  ),
}

/** One geometric glyph per product tile, keyed by `CONTENT.landing.products[].key`. */
const PRODUCT_ICONS: Record<string, () => JSX.Element> = {
  chat: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="square" stroke-linejoin="miter" aria-hidden="true">
      <path d="M21 15H8l-5 4V4h18v11z" />
      <path d="M8 8.5h8M8 12h5" />
    </svg>
  ),
  research: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="square" stroke-linejoin="miter" aria-hidden="true">
      <rect x="3" y="3" width="7.5" height="7.5" />
      <rect x="13.5" y="3" width="7.5" height="7.5" />
      <rect x="3" y="13.5" width="7.5" height="7.5" />
      <path d="M13.5 17.25h7.5M17.25 13.5V21" />
    </svg>
  ),
  insights: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="square" stroke-linejoin="miter" aria-hidden="true">
      <rect x="3" y="4" width="18" height="16" />
      <path d="M7 9h10M7 13h10M7 17h6" />
    </svg>
  ),
  tracking: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="square" stroke-linejoin="miter" aria-hidden="true">
      <path d="M4 19h16M6 15l4-4 3 3 5-6" />
      <path d="M15 8h3v3" />
    </svg>
  ),
}

/** Brand marks for official media & community channels. */
const CHANNEL_ICONS: Record<string, () => JSX.Element> = {
  youtube: () => (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M23.5 6.9a3 3 0 0 0-2.1-2.1C19.5 4.3 12 4.3 12 4.3s-7.5 0-9.4.5A3 3 0 0 0 .5 6.9C0 8.8 0 12 0 12s0 3.2.5 5.1a3 3 0 0 0 2.1 2.1c1.9.5 9.4.5 9.4.5s7.5 0 9.4-.5a3 3 0 0 0 2.1-2.1c.5-1.9.5-5.1.5-5.1s0-3.2-.5-5.1ZM9.6 15.6V8.4l6.2 3.6-6.2 3.6Z" />
    </svg>
  ),
  bilibili: () => (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M18.2 3.4a1.4 1.4 0 0 1 0 2l-1.6 1.5h1.6A4.2 4.2 0 0 1 22.4 11v6.3a4.2 4.2 0 0 1-4.2 4.2H5.8a4.2 4.2 0 0 1-4.2-4.2V11a4.2 4.2 0 0 1 4.2-4.2h1.5L5.8 5.4a1.4 1.4 0 1 1 2-2l2.6 2.6q.2.2.3.4h2.6q.1-.2.3-.4l2.6-2.6a1.4 1.4 0 0 1 2 0ZM18.2 9.5H5.8a1.4 1.4 0 0 0-1.4 1.4v6.3a1.4 1.4 0 0 0 1.4 1.4h12.4a1.4 1.4 0 0 0 1.4-1.4V11a1.4 1.4 0 0 0-1.4-1.4Zm-9.8 2.6a1.3 1.3 0 0 1 1.3 1.3v1.3a1.3 1.3 0 0 1-2.6 0v-1.3a1.3 1.3 0 0 1 1.3-1.3Zm7.2 0a1.3 1.3 0 0 1 1.3 1.3v1.3a1.3 1.3 0 1 1-2.6 0v-1.3a1.3 1.3 0 0 1 1.3-1.3Z" />
    </svg>
  ),
  zsxq: () => (
    <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <circle cx="12" cy="12" r="8.5" stroke="currentColor" stroke-width="2" stroke-dasharray="40 12" />
      <rect x="9.5" y="9.5" width="5" height="5" fill="currentColor" />
    </svg>
  ),
  discord: () => (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M20.32 4.37a19.8 19.8 0 0 0-4.89-1.52.07.07 0 0 0-.07.04c-.21.38-.44.87-.61 1.25a18.27 18.27 0 0 0-5.49 0 12.64 12.64 0 0 0-.62-1.25.08.08 0 0 0-.07-.04c-1.7.29-3.34.8-4.89 1.52a.07.07 0 0 0-.03.03C.53 9.05-.32 13.58.1 18.06a.08.08 0 0 0 .03.05 19.9 19.9 0 0 0 5.99 3.03.08.08 0 0 0 .08-.03c.46-.63.87-1.3 1.22-2a.08.08 0 0 0-.04-.11 13.1 13.1 0 0 1-1.87-.9.08.08 0 0 1-.01-.12c.13-.09.25-.19.37-.29a.07.07 0 0 1 .08-.01c3.93 1.79 8.18 1.79 12.07 0a.07.07 0 0 1 .08.01c.12.1.25.2.37.29a.08.08 0 0 1-.01.12c-.6.35-1.22.65-1.87.9a.08.08 0 0 0-.04.11c.36.7.77 1.37 1.22 2a.08.08 0 0 0 .08.03 19.83 19.83 0 0 0 6-3.03.08.08 0 0 0 .03-.05c.5-5.18-.84-9.68-3.55-13.66a.06.06 0 0 0-.03-.03ZM8.02 15.33c-1.18 0-2.16-1.08-2.16-2.42 0-1.33.96-2.42 2.16-2.42 1.21 0 2.18 1.1 2.16 2.42 0 1.34-.96 2.42-2.16 2.42Zm7.97 0c-1.18 0-2.16-1.08-2.16-2.42 0-1.33.96-2.42 2.16-2.42 1.21 0 2.18 1.1 2.16 2.42 0 1.34-.95 2.42-2.16 2.42Z" />
    </svg>
  ),
}

function isInternal(href: string) {
  return href.startsWith("/")
}

export default function PublicHomePage() {
  const navigate = useNavigate()
  const [stars] = createResource(fetchGithubStars)
  const [showcaseIndex, setShowcaseIndex] = createSignal(0)
  const [quantPeriodKey, setQuantPeriodKey] = createSignal<"3y" | "1y">("3y")
  const [enlarged, setEnlarged] = createSignal<{ src: string; alt: string } | null>(null)
  const L = CONTENT.landing
  const Q = CONTENT.quant
  const featuredPost = () => latestPublicBlogPost()

  const activeShowcase = createMemo(() => {
    const tabs = L.showcase_tabs
    return tabs[showcaseIndex()] ?? tabs[0]
  })

  const activeQuantPeriod = createMemo(() => {
    return Q.periods.find((item) => item.key === quantPeriodKey()) ?? Q.periods[0]
  })

  const go = (href: string) => (event: MouseEvent) => {
    if (href === "#quant") {
      event.preventDefault()
      scrollToQuant()
      return
    }
    if (!isInternal(href)) return
    event.preventDefault()
    navigate(href)
  }

  const scrollToQuant = (behavior: ScrollBehavior = "smooth") =>
    document.getElementById("quant")?.scrollIntoView({ block: "start", behavior })

  onMount(() => {
    let hashTimer: ReturnType<typeof setTimeout> | undefined
    if (window.location.hash === "#quant") hashTimer = setTimeout(() => scrollToQuant("instant"), 60)
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setEnlarged(null)
    }
    window.addEventListener("keydown", closeOnEscape)
    onCleanup(() => {
      clearTimeout(hashTimer)
      window.removeEventListener("keydown", closeOnEscape)
    })
  })

  return (
    <div class="pub-page hone-home">
      <Title>{L.page_title}</Title>
      <div class="hone-home-bg" aria-hidden="true" />
      <PublicNav />

      <Show when={enlarged()}>
        {(image) => (
          <div class="hone-home-lightbox" role="dialog" aria-modal="true" aria-label={image().alt} onClick={() => setEnlarged(null)}>
            <div class="hone-home-lightbox__frame" onClick={(event) => event.stopPropagation()}>
              <div class="hone-home-lightbox__bar">
                <span>{image().alt}</span>
                <button type="button" aria-label="Close" onClick={() => setEnlarged(null)}>
                  ESC ×
                </button>
              </div>
              <img src={image().src} alt={image().alt} />
            </div>
          </div>
        )}
      </Show>

      <main class="hone-home-main">
        {/* ── Hero ─────────────────────────────────────────────────────── */}
        <section class="hone-home-hero" aria-labelledby="hone-home-title">
          <div class="hone-home-topbar">
            <a
              class="hone-home-chip"
              href="#quant"
              onClick={(event) => {
                event.preventDefault()
                scrollToQuant()
              }}
            >
              <b>{Q.hero_chip_tag}</b>
              <span>{Q.hero_chip}</span>
              <Icon.Arrow />
            </a>
            <span class="hone-home-topbar__meta">{L.license_badge}</span>
          </div>

          <div class="hone-home-hero__grid">
            <div class="hone-home-hero__primary">
              <div class="hone-home-eyebrow">{L.eyebrow}</div>
              <h1 id="hone-home-title">
                <span>{L.headline_1}</span>
                <em>{L.headline_2}</em>
              </h1>
            </div>
            <div class="hone-home-hero__aside">
              <p class="hone-home-lead">{L.description}</p>
              <div class="hone-home-actions is-left">
                <a class="hone-home-cta" href="/chat" onClick={go("/chat")}>
                  <span>{L.cta_primary}</span>
                  <Icon.Arrow />
                </a>
                <a class="hone-home-cta is-secondary" href="/plan" onClick={go("/plan")}>
                  <span>{L.cta_member}</span>
                </a>
                <a class="hone-home-cta is-ghost" href={CONTENT.nav.github_url} target="_blank" rel="noopener noreferrer">
                  <Icon.Github />
                  <span>GitHub</span>
                  <Show when={displayGithubStars(stars())}>
                    <small>★ {displayGithubStars(stars())}</small>
                  </Show>
                </a>
              </div>
            </div>
          </div>

          <div class="hone-home-coverage" aria-label={L.coverage_label}>
            <span>{L.coverage_label}</span>
            <div class="hone-home-coverage__list">
              <For each={L.coverage}>{(item) => <b>{item}</b>}</For>
            </div>
          </div>
        </section>

        {/* ── Interactive Workspace Terminal ────────────────────────────── */}
        <figure class="hone-home-shot is-hero hone-home-terminal">
          <div class="hone-home-terminal__chrome">
            <div class="hone-home-terminal__tabs" role="tablist" aria-label={L.eyebrow}>
              <For each={L.showcase_tabs}>
                {(tab, index) => (
                  <button
                    type="button"
                    role="tab"
                    aria-selected={showcaseIndex() === index()}
                    classList={{ "is-active": showcaseIndex() === index() }}
                    onClick={() => setShowcaseIndex(index())}
                  >
                    <small>{tab.index}</small>
                    <span>{tab.label}</span>
                  </button>
                )}
              </For>
            </div>
            <div class="hone-home-terminal__tools">
              <code>{activeShowcase().route}</code>
              <button
                type="button"
                class="hone-home-terminal__tool"
                onClick={() =>
                  setEnlarged({
                    src: activeShowcase()?.image ?? "/home-assistant.jpg",
                    alt: activeShowcase()?.alt ?? L.hero_image_alt,
                  })
                }
              >
                <Icon.Zoom />
                <span>{L.zoom_cta}</span>
              </button>
              <a
                class="hone-home-terminal__tool is-accent"
                href={activeShowcase().href}
                onClick={go(activeShowcase().href)}
              >
                <span>{L.open_surface_cta}</span>
                <Icon.Arrow />
              </a>
            </div>
          </div>
          <button
            type="button"
            class="hone-home-shot__trigger"
            onClick={() =>
              setEnlarged({
                src: activeShowcase()?.image ?? "/home-assistant.jpg",
                alt: activeShowcase()?.alt ?? L.hero_image_alt,
              })
            }
          >
            <img
              src={activeShowcase()?.image ?? "/home-assistant.jpg"}
              alt={activeShowcase()?.alt ?? L.hero_image_alt}
              width="2000"
              height="1067"
              fetchpriority="high"
            />
          </button>
          <figcaption>{activeShowcase()?.caption ?? L.hero_caption}</figcaption>
        </figure>

        {/* ── Proof Ledger ─────────────────────────────────────────────── */}
        <dl class="hone-home-proof">
          <For each={L.proof}>
            {(item) => (
              <div>
                <dt>{item.value}</dt>
                <dd>{item.label}</dd>
              </div>
            )}
          </For>
        </dl>

        {/* ── 01 · Products ────────────────────────────────────────────── */}
        <section class="hone-home-section" aria-labelledby="hone-home-products">
          <header class="hone-home-head">
            <div class="hone-home-eyebrow">{L.products_eyebrow}</div>
            <h2 id="hone-home-products">{L.products_title}</h2>
            <p>{L.products_sub}</p>
          </header>
          <div class="hone-home-products">
            <For each={L.products}>
              {(product) => (
                <a class="hone-home-product" href={product.href} onClick={go(product.href)}>
                  <div class="hone-home-product__top">
                    <span class="hone-home-product__icon">{PRODUCT_ICONS[product.key]?.()}</span>
                    <span class="hone-home-product__index">{product.index}</span>
                  </div>
                  <h3>{product.title}</h3>
                  <p>{product.body}</p>
                  <div class="hone-home-product__foot">
                    <code>{product.route_label}</code>
                    <span class="hone-home-product__cta">
                      <span>{product.cta}</span>
                      <Icon.Arrow />
                    </span>
                  </div>
                </a>
              )}
            </For>
          </div>
        </section>

        {/* ── 02 · Assistant & Interactive Queries ─────────────────────── */}
        <section class="hone-home-feature" aria-labelledby="hone-home-assistant">
          <div class="hone-home-feature__copy">
            <div class="hone-home-eyebrow">{L.assistant.eyebrow}</div>
            <h2 id="hone-home-assistant">{L.assistant.title}</h2>
            <p>{L.assistant.body}</p>
            <ul class="hone-home-points">
              <For each={L.assistant.points}>
                {(point) => (
                  <li>
                    <span class="hone-home-points__mark">
                      <Icon.Check />
                    </span>
                    <span>{point}</span>
                  </li>
                )}
              </For>
            </ul>
            <a class="hone-home-link" href="/chat" onClick={go("/chat")}>
              <span>{L.assistant.cta}</span>
              <Icon.Arrow />
            </a>
          </div>
          <div class="hone-home-asks">
            <header class="hone-home-asks__head">
              <span>{L.assistant.examples_label}</span>
              <small>{L.assistant.examples_hint}</small>
            </header>
            <div class="hone-home-asks__list">
              <For each={L.assistant.examples}>
                {(example, idx) => (
                  <a
                    href={`/chat?q=${encodeURIComponent(example)}`}
                    onClick={go(`/chat?q=${encodeURIComponent(example)}`)}
                  >
                    <div class="hone-home-asks__meta">
                      <code>0{idx() + 1}</code>
                      <em>{L.assistant.example_tags[idx()] ?? "QUERY"}</em>
                    </div>
                    <p>{example}</p>
                    <span class="hone-home-asks__go">
                      <span>{L.assistant.ask_action}</span>
                      <Icon.Arrow />
                    </span>
                  </a>
                )}
              </For>
            </div>
          </div>
        </section>

        {/* ── 03 · Research Desk + Insights ────────────────────────────── */}
        <section class="hone-home-feature is-media" aria-labelledby="hone-home-research">
          <div class="hone-home-feature__copy">
            <div class="hone-home-eyebrow">{L.research.eyebrow}</div>
            <h2 id="hone-home-research">{L.research.title}</h2>
            <p>{L.research.body}</p>
            <dl class="hone-home-facts">
              <For each={L.research.points}>
                {(point) => (
                  <div>
                    <dt>
                      <code>{point.index}</code>
                      <span>{point.title}</span>
                      <small>{point.cadence}</small>
                    </dt>
                    <dd>{point.body}</dd>
                  </div>
                )}
              </For>
            </dl>
            <a class="hone-home-link" href="/research" onClick={go("/research")}>
              <span>{L.research.cta}</span>
              <Icon.Arrow />
            </a>
          </div>
          <figure class="hone-home-composite">
            <div class="hone-home-composite__grid">
              <button
                type="button"
                class="hone-home-composite__desk"
                onClick={() => setEnlarged({ src: "/home-research.jpg", alt: L.research.image_alt })}
              >
                <span class="hone-home-composite__bar">
                  <span>{L.research.desk_badge}</span>
                  <Icon.Zoom />
                </span>
                <img src="/home-research.jpg" alt={L.research.image_alt} width="1600" height="1000" loading="lazy" />
              </button>
              <button
                type="button"
                class="hone-home-composite__phone"
                onClick={() => setEnlarged({ src: "/home-influencer-mobile.jpg", alt: L.research.mobile_alt })}
              >
                <span class="hone-home-composite__bar">
                  <span>{L.research.mobile_badge}</span>
                  <Icon.Zoom />
                </span>
                <img src="/home-influencer-mobile.jpg" alt={L.research.mobile_alt} width="430" height="932" loading="lazy" />
              </button>
            </div>
            <figcaption>{L.research.caption}</figcaption>
          </figure>
        </section>

        {/* ── 04 · hone-quant ──────────────────────────────────────────── */}
        <section id="quant" class="hone-home-quant" aria-labelledby="hone-home-quant-title">
          <div class="hone-home-quant__intro">
            <div class="hone-home-feature__copy">
              <div class="hone-home-kicker">
                <span class="hone-home-eyebrow">{Q.eyebrow}</span>
                <span class="hone-home-badge">{Q.badge}</span>
              </div>
              <h2 id="hone-home-quant-title">{Q.title}</h2>
              <p>{Q.lead}</p>
              <ol class="hone-home-steps">
                <For each={Q.pillars}>
                  {(pillar) => (
                    <li>
                      <small>
                        {pillar.step} · {pillar.tag}
                      </small>
                      <strong>{pillar.title}</strong>
                      <span>{pillar.body}</span>
                    </li>
                  )}
                </For>
              </ol>
              <div class="hone-home-actions is-left">
                <a class="hone-home-cta" href={Q.github_url} target="_blank" rel="noopener noreferrer">
                  <Icon.Github />
                  <span>{Q.cta_github}</span>
                  <Icon.UpRight />
                </a>
                <a class="hone-home-cta is-ghost" href={Q.contact_url}>
                  <span>{Q.cta_contact}</span>
                </a>
              </div>
            </div>
            <figure class="hone-home-shot hone-home-quant__shot">
              <button
                type="button"
                class="hone-home-shot__trigger"
                onClick={() => setEnlarged({ src: Q.image, alt: Q.image_alt })}
              >
                <span class="hone-home-composite__bar">
                  <span>HONE-QUANT · BACKTEST LEDGER</span>
                  <Icon.Zoom />
                </span>
                <img src={Q.image} alt={Q.image_alt} width="1600" height="949" loading="lazy" />
              </button>
            </figure>
          </div>
          <div class="hone-home-backtest">
            <header>
              <div class="hone-home-backtest__title">
                <strong>{Q.backtest_label}</strong>
                <span>
                  {Q.backtest_strategy} · {activeQuantPeriod().range}
                </span>
              </div>
              <div class="hone-home-backtest__tabs" role="tablist" aria-label={Q.backtest_label}>
                <For each={Q.periods}>
                  {(period) => (
                    <button
                      type="button"
                      role="tab"
                      aria-selected={quantPeriodKey() === period.key}
                      classList={{ "is-active": quantPeriodKey() === period.key }}
                      onClick={() => setQuantPeriodKey(period.key as "3y" | "1y")}
                    >
                      {period.tab}
                    </button>
                  )}
                </For>
              </div>
            </header>
            <dl>
              <For each={activeQuantPeriod().stats}>
                {(stat) => (
                  <div classList={{ "is-up": stat.tone === "up", "is-down": stat.tone === "down" }}>
                    <dt>{stat.label}</dt>
                    <dd>{stat.value}</dd>
                  </div>
                )}
              </For>
            </dl>
            <p class="hone-home-backtest__secondary">
              <strong>{activeQuantPeriod().compare_label}</strong>
              <span>{activeQuantPeriod().compare_summary}</span>
            </p>
            <p class="hone-home-backtest__note">
              <Icon.Info />
              <span>{Q.disclaimer}</span>
            </p>
          </div>
        </section>

        {/* ── 05 · BamangTech Community & Video Channels ───────────────── */}
        <section class="hone-home-community" aria-labelledby="hone-home-community">
          <div class="hone-home-feature__copy">
            <div class="hone-home-eyebrow">{L.community.eyebrow}</div>
            <h2 id="hone-home-community">{L.community.title}</h2>
            <p>{L.community.body}</p>
            <div class="hone-home-host">
              <img src="/hari-avatar.jpg" alt={L.community.host_name} width="64" height="64" decoding="async" />
              <div>
                <strong>{L.community.host_name}</strong>
                <span>{L.community.host_role}</span>
                <small>{L.community.host_bio}</small>
              </div>
            </div>
            <div class="hone-home-actions is-left">
              <a class="hone-home-cta" href="/plan" onClick={go("/plan")}>
                <span>{L.community.cta}</span>
                <Icon.Arrow />
              </a>
              <a class="hone-home-cta is-ghost" href="/plan" onClick={go("/plan")}>
                <span>{L.community.cta_trial}</span>
              </a>
            </div>
          </div>
          <div class="hone-home-community__panel">
            <dl class="hone-home-stats">
              <For each={L.community.stats}>
                {(stat) => (
                  <div>
                    <dt>{stat.value}</dt>
                    <dd>{stat.label}</dd>
                  </div>
                )}
              </For>
            </dl>
            <div class="hone-home-channels">
              <div class="hone-home-channels__head">
                <strong>{L.community.channels_label}</strong>
                <small>{L.community.channels_sub}</small>
              </div>
              <div class="hone-home-channels__grid">
                <For each={L.community.channels}>
                  {(channel) => (
                    <a
                      class={`hone-home-channel is-${channel.key}`}
                      href={channel.href}
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      <span class="hone-home-channel__icon">{CHANNEL_ICONS[channel.key]?.()}</span>
                      <span class="hone-home-channel__body">
                        <strong>
                          <span>{channel.label}</span>
                          <code>{channel.handle}</code>
                        </strong>
                        <small>{channel.desc}</small>
                      </span>
                      <span class="hone-home-channel__arrow">
                        <Icon.UpRight />
                      </span>
                    </a>
                  )}
                </For>
              </div>
            </div>
            <p class="hone-home-community__note">{L.community.note}</p>
          </div>
        </section>

        {/* ── 06 · Principles & Engineering Index ──────────────────────── */}
        <section class="hone-home-section" aria-labelledby="hone-home-principles">
          <header class="hone-home-head">
            <div class="hone-home-eyebrow">{L.principles.eyebrow}</div>
            <h2 id="hone-home-principles">{L.principles.title}</h2>
            <p>{L.principles.sub}</p>
          </header>
          <div class="hone-home-principles">
            <For each={L.principles.items}>
              {(item, index) => (
                <article>
                  <span aria-hidden="true">0{index() + 1}</span>
                  <h3>{item.title}</h3>
                  <p>{item.body}</p>
                </article>
              )}
            </For>
          </div>
          <div class="hone-home-more">
            <a href={`/blog/${featuredPost().slug}`} onClick={go(`/blog/${featuredPost().slug}`)}>
              <div class="hone-home-more__top">
                <small>{CONTENT.home_page.blog_eyebrow}</small>
                <Icon.Arrow />
              </div>
              <strong>{CONTENT.home_page.blog_title}</strong>
              <span>{CONTENT.home_page.blog_desc}</span>
            </a>
            <a href="/survey" onClick={go("/survey")}>
              <div class="hone-home-more__top">
                <small>{CONTENT.survey.home_eyebrow}</small>
                <Icon.Arrow />
              </div>
              <strong>{CONTENT.survey.home_title}</strong>
              <span>{CONTENT.survey.home_desc}</span>
            </a>
            <a href={CONTENT.nav.github_url} target="_blank" rel="noopener noreferrer">
              <div class="hone-home-more__top">
                <small>GITHUB · MIT</small>
                <Icon.UpRight />
              </div>
              <strong>{L.principles.github_cta}</strong>
              <span>{L.principles.github_desc}</span>
            </a>
          </div>
        </section>

        {/* ── Final CTA ────────────────────────────────────────────────── */}
        <section class="hone-home-final" aria-labelledby="hone-home-final">
          <div class="hone-home-final__copy">
            <div class="hone-home-final__eyebrow">{L.final.eyebrow}</div>
            <h2 id="hone-home-final">{L.final.title}</h2>
            <p>{L.final.body}</p>
          </div>
          <div class="hone-home-final__side">
            <div class="hone-home-actions is-left">
              <a class="hone-home-cta is-inverse" href="/chat" onClick={go("/chat")}>
                <span>{L.final.cta_primary}</span>
                <Icon.Arrow />
              </a>
              <a class="hone-home-cta is-inverse-ghost" href="/plan" onClick={go("/plan")}>
                <span>{L.final.cta_secondary}</span>
              </a>
            </div>
            <small>
              {L.final.contact} · <a href="mailto:contact@honeclaw.app">contact@honeclaw.app</a>
            </small>
          </div>
        </section>
        <p class="hone-home-disclaimer">{L.disclaimer}</p>
      </main>

      <PublicFooter />
    </div>
  )
}
