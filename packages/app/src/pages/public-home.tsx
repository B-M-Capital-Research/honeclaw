// public-home.tsx — HONE 官网首页（v6 · AI 基础设施投研助手）
//
// 内容与 README 的产品定位保持一致：投资助手 → 研究台 / 洞察 → hone-quant →
// 社区 → 原则。截图只用线上实拍或当前界面生成的示例，并在图注里说明来源；
// 文案全部走 CONTENT.landing（中英双语同构）。

import { Title } from "@solidjs/meta"
import { useNavigate } from "@solidjs/router"
import { For, Show, createResource, createSignal, onCleanup, onMount, type JSX } from "solid-js"
import { CONTENT } from "@/lib/public-content"
import { latestPublicBlogPost } from "@/lib/public-blog"
import { displayGithubStars, fetchGithubStars } from "@/lib/github-stars"
import { PublicFooter, PublicNav } from "@/components/public-nav"
import "./public-site.css"
import "./public-home.css"

const Icon = {
  Arrow: () => (
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12h14M12 5l7 7-7 7" /></svg>
  ),
  Github: () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" /></svg>
  ),
  Check: () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m5 12.5 4.5 4.5L19 7.5" /></svg>
  ),
  Info: () => (
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="9" /><path d="M12 11v5M12 7.6v.1" /></svg>
  ),
}

/** One glyph per product tile, keyed by `CONTENT.landing.products[].key`. */
const PRODUCT_ICONS: Record<string, () => JSX.Element> = {
  chat: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M21 15a2 2 0 0 1-2 2H8l-5 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /><path d="M8 9h8M8 13h5" /></svg>
  ),
  research: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="3" width="7.5" height="7.5" rx="1.6" /><rect x="13.5" y="3" width="7.5" height="7.5" rx="1.6" /><rect x="3" y="13.5" width="7.5" height="7.5" rx="1.6" /><path d="M13.5 17.25h7.5M17.25 13.5V21" /></svg>
  ),
  insights: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M9 18h6M10 21h4" /><path d="M12 3a6 6 0 0 0-3.6 10.8c.7.6 1.1 1.3 1.1 2.2h5c0-.9.4-1.6 1.1-2.2A6 6 0 0 0 12 3z" /></svg>
  ),
  tracking: () => (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M6 8a6 6 0 0 1 12 0c0 7 3 8 3 8H3s3-1 3-8" /><path d="M10.3 20a2 2 0 0 0 3.4 0" /></svg>
  ),
}

/** Same-origin routes go through the router; everything else is a new tab. */
function isInternal(href: string) {
  return href.startsWith("/")
}

export default function PublicHomePage() {
  const navigate = useNavigate()
  const [stars] = createResource(fetchGithubStars)
  const [enlarged, setEnlarged] = createSignal<{ src: string; alt: string } | null>(null)
  const L = CONTENT.landing
  const Q = CONTENT.quant
  const featuredPost = () => latestPublicBlogPost()

  const go = (href: string) => (event: MouseEvent) => {
    if (!isInternal(href)) return
    event.preventDefault()
    navigate(href)
  }

  const scrollToQuant = (behavior: ScrollBehavior = "smooth") =>
    document.getElementById("quant")?.scrollIntoView({ block: "start", behavior })

  onMount(() => {
    // 直接打开 /#quant 时，把 hone-quant 区块定位到视野内（instant：后台标签页
    // 里平滑滚动不会推进，且与浏览器原生锚点行为一致）。
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
            <img src={image().src} alt={image().alt} />
            <button type="button" aria-label="Close">×</button>
          </div>
        )}
      </Show>

      <main class="hone-home-main">
        {/* ── Hero ─────────────────────────────────────────────────────── */}
        <section class="hone-home-hero" aria-labelledby="hone-home-title">
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
          <div class="hone-home-eyebrow">{L.eyebrow}</div>
          <h1 id="hone-home-title">
            <span>{L.headline_1}</span>
            <em>{L.headline_2}</em>
          </h1>
          <p class="hone-home-lead">{L.description}</p>
          <div class="hone-home-actions">
            <a class="hone-home-cta" href="/chat" onClick={go("/chat")}>
              <span>{L.cta_primary}</span>
              <Icon.Arrow />
            </a>
            <a class="hone-home-cta is-secondary" href="/plan" onClick={go("/plan")}>
              {L.cta_member}
            </a>
            <a class="hone-home-cta is-ghost" href={CONTENT.nav.github_url} target="_blank" rel="noopener noreferrer">
              <Icon.Github />
              <span>GitHub</span>
              <Show when={displayGithubStars(stars())}>
                <small>★ {displayGithubStars(stars())}</small>
              </Show>
            </a>
          </div>
          <div class="hone-home-coverage" aria-label={L.coverage_label}>
            <span>{L.coverage_label}</span>
            <For each={L.coverage}>{(item) => <b>{item}</b>}</For>
          </div>
        </section>

        <figure class="hone-home-shot is-hero">
          <button type="button" onClick={() => setEnlarged({ src: "/home-assistant.jpg", alt: L.hero_image_alt })}>
            <img src="/home-assistant.jpg" alt={L.hero_image_alt} width="2000" height="1067" fetchpriority="high" />
          </button>
          <figcaption>{L.hero_caption}</figcaption>
        </figure>

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

        {/* ── Products ─────────────────────────────────────────────────── */}
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
                  <span class="hone-home-product__icon">{PRODUCT_ICONS[product.key]?.()}</span>
                  <h3>{product.title}</h3>
                  <p>{product.body}</p>
                  <span class="hone-home-product__cta">
                    {product.cta}
                    <Icon.Arrow />
                  </span>
                </a>
              )}
            </For>
          </div>
        </section>

        {/* ── Assistant ────────────────────────────────────────────────── */}
        <section class="hone-home-feature" aria-labelledby="hone-home-assistant">
          <div class="hone-home-feature__copy">
            <div class="hone-home-eyebrow">{L.assistant.eyebrow}</div>
            <h2 id="hone-home-assistant">{L.assistant.title}</h2>
            <p>{L.assistant.body}</p>
            <ul class="hone-home-points">
              <For each={L.assistant.points}>
                {(point) => (
                  <li>
                    <Icon.Check />
                    <span>{point}</span>
                  </li>
                )}
              </For>
            </ul>
            <a class="hone-home-link" href="/chat" onClick={go("/chat")}>
              {L.assistant.cta}
              <Icon.Arrow />
            </a>
          </div>
          <div class="hone-home-asks">
            <span>{L.assistant.examples_label}</span>
            <For each={L.assistant.examples}>
              {(example) => (
                <a
                  href={`/chat?q=${encodeURIComponent(example)}`}
                  onClick={go(`/chat?q=${encodeURIComponent(example)}`)}
                >
                  <b aria-hidden="true">“</b>
                  {example}
                </a>
              )}
            </For>
          </div>
        </section>

        {/* ── Research desk + Insights ─────────────────────────────────── */}
        <section class="hone-home-feature is-media" aria-labelledby="hone-home-research">
          <div class="hone-home-feature__copy">
            <div class="hone-home-eyebrow">{L.research.eyebrow}</div>
            <h2 id="hone-home-research">{L.research.title}</h2>
            <p>{L.research.body}</p>
            <dl class="hone-home-facts">
              <For each={L.research.points}>
                {(point) => (
                  <div>
                    <dt>{point.title}</dt>
                    <dd>{point.body}</dd>
                  </div>
                )}
              </For>
            </dl>
            <a class="hone-home-link" href="/research" onClick={go("/research")}>
              {L.research.cta}
              <Icon.Arrow />
            </a>
          </div>
          <figure class="hone-home-composite">
            <button
              type="button"
              class="hone-home-composite__desk"
              onClick={() => setEnlarged({ src: "/home-research.jpg", alt: L.research.image_alt })}
            >
              <img src="/home-research.jpg" alt={L.research.image_alt} width="1600" height="1000" loading="lazy" />
            </button>
            <button
              type="button"
              class="hone-home-composite__phone"
              onClick={() => setEnlarged({ src: "/home-influencer-mobile.jpg", alt: L.research.mobile_alt })}
            >
              <img src="/home-influencer-mobile.jpg" alt={L.research.mobile_alt} width="430" height="932" loading="lazy" />
            </button>
            <figcaption>{L.research.caption}</figcaption>
          </figure>
        </section>

        {/* ── hone-quant ───────────────────────────────────────────────── */}
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
                </a>
                <a class="hone-home-cta is-ghost" href={Q.contact_url}>
                  {Q.cta_contact}
                </a>
              </div>
            </div>
            <figure class="hone-home-shot">
              <button type="button" onClick={() => setEnlarged({ src: Q.image, alt: Q.image_alt })}>
                <img src={Q.image} alt={Q.image_alt} width="1600" height="949" loading="lazy" />
              </button>
            </figure>
          </div>
          <div class="hone-home-backtest">
            <header>
              <strong>{Q.backtest_label}</strong>
              <span>
                {Q.backtest_strategy} · {Q.backtest_range}
              </span>
            </header>
            <dl>
              <For each={Q.stats}>
                {(stat) => (
                  <div classList={{ "is-up": stat.tone === "up", "is-down": stat.tone === "down" }}>
                    <dt>{stat.label}</dt>
                    <dd>{stat.value}</dd>
                  </div>
                )}
              </For>
            </dl>
            <p class="hone-home-backtest__secondary">
              <strong>{Q.secondary_label}</strong>
              <span>{Q.secondary}</span>
            </p>
            <p class="hone-home-backtest__note">
              <Icon.Info />
              <span>{Q.disclaimer}</span>
            </p>
          </div>
        </section>

        {/* ── Community ────────────────────────────────────────────────── */}
        <section class="hone-home-community" aria-labelledby="hone-home-community">
          <div class="hone-home-feature__copy">
            <div class="hone-home-eyebrow">{L.community.eyebrow}</div>
            <h2 id="hone-home-community">{L.community.title}</h2>
            <p>{L.community.body}</p>
            <div class="hone-home-host">
              <img src="/hari-avatar.jpg" alt={L.community.host_name} width="56" height="56" loading="lazy" />
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
                {L.community.cta_trial}
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
              <span>{L.community.channels_label}</span>
              <For each={L.community.channels}>
                {(channel) => (
                  <a href={channel.href} target="_blank" rel="noopener noreferrer">
                    {channel.label}
                  </a>
                )}
              </For>
            </div>
            <p class="hone-home-community__note">{L.community.note}</p>
          </div>
        </section>

        {/* ── Principles ───────────────────────────────────────────────── */}
        <section class="hone-home-section" aria-labelledby="hone-home-principles">
          <header class="hone-home-head">
            <div class="hone-home-eyebrow">{L.principles.eyebrow}</div>
            <h2 id="hone-home-principles">{L.principles.title}</h2>
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
              <small>{CONTENT.home_page.blog_eyebrow}</small>
              <strong>{CONTENT.home_page.blog_title}</strong>
              <span>{CONTENT.home_page.blog_desc}</span>
            </a>
            <a href="/survey" onClick={go("/survey")}>
              <small>{CONTENT.survey.home_eyebrow}</small>
              <strong>{CONTENT.survey.home_title}</strong>
              <span>{CONTENT.survey.home_desc}</span>
            </a>
            <a href={CONTENT.nav.github_url} target="_blank" rel="noopener noreferrer">
              <small>GitHub</small>
              <strong>{L.principles.github_cta}</strong>
              <span>B-M-Capital-Research/honeclaw</span>
            </a>
          </div>
        </section>

        {/* ── Final CTA ────────────────────────────────────────────────── */}
        <section class="hone-home-final" aria-labelledby="hone-home-final">
          <h2 id="hone-home-final">{L.final.title}</h2>
          <p>{L.final.body}</p>
          <div class="hone-home-actions">
            <a class="hone-home-cta is-inverse" href="/chat" onClick={go("/chat")}>
              <span>{L.final.cta_primary}</span>
              <Icon.Arrow />
            </a>
            <a class="hone-home-cta is-inverse-ghost" href="/plan" onClick={go("/plan")}>
              {L.final.cta_secondary}
            </a>
          </div>
          <small>
            {L.final.contact} · <a href="mailto:contact@honeclaw.app">contact@honeclaw.app</a>
          </small>
        </section>
        <p class="hone-home-disclaimer">{L.disclaimer}</p>
      </main>

      <PublicFooter />
    </div>
  )
}
