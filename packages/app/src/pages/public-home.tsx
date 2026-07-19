// public-home.tsx — HONE 首页（v5 · Agent 工作台同源视觉）
// 编辑式排版 + hone 令牌：mono eyebrow、双行主标题、工作台窗口式演示框、
// 卖点/案例面板、Plan 预告与统一页脚。文案全部走 CONTENT（中英双语）。

import {
  createResource,
  createSignal,
  onCleanup,
  onMount,
  Show,
  For,
} from "solid-js"
import { useNavigate } from "@solidjs/router"
import { CONTENT } from "@/lib/public-content"
import { latestPublicBlogPost } from "@/lib/public-blog"
import { useLocale } from "@/lib/i18n"
import { displayGithubStars, fetchGithubStars } from "@/lib/github-stars"
import { PublicFooter, PublicNav } from "@/components/public-nav"
import { PublicMembershipModal } from "@/components/public-membership-modal"
import { HoneBrand } from "@/components/hone-brand"
import "./public-site.css"

const ICONS = {
  Chat: () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
  ),
  Github: () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
  ),
  ArrowRight: () => (
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
  ),
}

export default function PublicHomePage() {
  const [index, setIndex] = createSignal(0)
  const [enlargeImg, setEnlargeImg] = createSignal<string | null>(null)
  const [stars] = createResource(fetchGithubStars)
  const [buyOpen, setBuyOpen] = createSignal(false)
  const navigate = useNavigate()
  const C = CONTENT

  const slides = () => [
    ...C.cases.items.filter((i) => i.image).map((i) => ({
      tag: i.tag,
      title: i.title,
      body: i.body,
      image: i.image,
      link: null as string | null,
    })),
    {
      tag: C.home_page.roadmap_slide_tag,
      title: C.roadmap.hero_title,
      body: C.roadmap.hero_sub,
      image: useLocale() === "zh" ? "/hone_solution_zh.jpg" : "/hone_solution.jpg",
      link: "/roadmap",
    },
  ]

  const videoUrl = () =>
    useLocale() === "zh"
      ? "https://player.bilibili.com/player.html?bvid=BV1ByXNBGET5&page=1&high_quality=1&danmaku=0&autoplay=0"
      : "https://www.youtube.com/embed/hJr-81OdYcQ?autoplay=0"

  let timer: ReturnType<typeof setInterval> | undefined
  const startTimer = () => {
    clearInterval(timer)
    timer = setInterval(() => {
      setIndex((prev) => (prev + 1) % slides().length)
    }, 10000)
  }

  onMount(() => {
    startTimer()
    onCleanup(() => clearInterval(timer))
  })

  const current = () => slides()[index()]
  const featuredPost = () => latestPublicBlogPost()
  const stats = () => [C.hero.stat_1, C.hero.stat_2, C.hero.stat_3]

  return (
    <div class="pub-page hone-home">
      <div class="hone-home-bg" aria-hidden="true" />
      <PublicNav />

      <Show when={enlargeImg()}>
        <div class="hone-home-lightbox" onClick={() => setEnlargeImg(null)}>
          <img src={enlargeImg()!} alt="" />
          <button type="button" aria-label="Close">×</button>
        </div>
      </Show>

      <main class="hone-home-main">
        {/* ── Hero ── */}
        <section class="hone-home-hero">
          <div class="hone-home-eyebrow">{C.hero.eyebrow}</div>
          <h1>
            <span>{C.hero.headline_1}</span>
            <em>{C.hero.headline_2}</em>
          </h1>
          <p class="hone-home-hero-desc">{C.hero.description}</p>
          <div class="hone-home-hero-actions">
            <button type="button" class="hone-home-cta" onClick={() => navigate("/chat")}>
              <ICONS.Chat />
              <span>{C.hero.cta_primary}</span>
            </button>
            <button type="button" class="hone-home-cta is-buy" onClick={() => setBuyOpen(true)}>
              <span>{C.nav.buy}</span>
              <small>¥100↓</small>
            </button>
            <a
              class="hone-home-cta is-ghost"
              href={C.nav.github_url}
              target="_blank"
              rel="noopener noreferrer"
            >
              <ICONS.Github />
              <span>GitHub</span>
              <Show when={displayGithubStars(stars())}>
                <small>★ {displayGithubStars(stars())}</small>
              </Show>
            </a>
          </div>
          <div class="hone-home-stats" role="list">
            <For each={stats()}>
              {(stat) => (
                <div role="listitem">
                  <strong>{stat.value}</strong>
                  <small>{stat.label}</small>
                </div>
              )}
            </For>
          </div>
        </section>

        {/* ── 视频演示：工作台窗口式外框 ── */}
        <section class="hone-home-demo">
          <div class="hone-home-window">
            <header>
              <i /><i /><i />
              <span>{C.home_page.video_demo}</span>
            </header>
            <div class="hone-home-window-body">
              <iframe src={videoUrl()} allowfullscreen title={C.home_page.video_demo} />
            </div>
          </div>
        </section>

        {/* ── 为什么是 HONE ── */}
        <section class="hone-home-section">
          <header class="hone-home-section-head">
            <div class="hone-home-eyebrow">{C.trust.section_label}</div>
          </header>
          <div class="hone-home-trust">
            <For each={C.trust.items}>
              {(item) => (
                <article>
                  <span aria-hidden="true">{item.symbol}</span>
                  <h3>{item.title}</h3>
                  <p>{item.body}</p>
                </article>
              )}
            </For>
          </div>
        </section>

        {/* ── 真实工作流轮播 ── */}
        <section class="hone-home-section">
          <header class="hone-home-section-head">
            <div class="hone-home-eyebrow">{C.cases.section_label}</div>
            <p>{C.cases.section_sub}</p>
          </header>
          <div class="hone-home-cases">
            <nav class="hone-home-case-tabs" aria-label={C.cases.section_label}>
              <For each={slides()}>
                {(slide, i) => (
                  <button
                    type="button"
                    classList={{ "is-active": i() === index() }}
                    onClick={() => {
                      setIndex(i())
                      startTimer()
                    }}
                  >
                    {slide.tag}
                  </button>
                )}
              </For>
            </nav>
            <div class="hone-home-case-body">
              <div class="hone-home-case-copy">
                <h2>{current().title}</h2>
                <p>{current().body}</p>
                <Show when={current().link}>
                  <button
                    type="button"
                    class="hone-home-case-link"
                    onClick={() => navigate(current().link!)}
                  >
                    <span>{C.home_page.view_full_roadmap}</span>
                    <ICONS.ArrowRight />
                  </button>
                </Show>
                <div class="hone-home-case-progress" aria-hidden="true">
                  <div style={{ width: `${((index() + 1) / slides().length) * 100}%` }} />
                </div>
              </div>
              <button
                type="button"
                class="hone-home-case-shot"
                onClick={() => current().image && setEnlargeImg(current().image)}
              >
                <img src={current().image || undefined} alt={current().title} />
                <span>{C.home_page.zoom_hint}</span>
              </button>
            </div>
          </div>
        </section>

        {/* ── 博客精选 ── */}
        <section
          class="hone-home-blog"
          onClick={() => navigate(`/blog/${featuredPost().slug}`)}
        >
          <div class="hone-home-blog-copy">
            <div class="hone-home-eyebrow">{C.home_page.blog_eyebrow}</div>
            <h2>{C.home_page.blog_title}</h2>
            <p>{C.home_page.blog_desc}</p>
            <button type="button">
              <span>{C.home_page.blog_cta}</span>
              <ICONS.ArrowRight />
            </button>
          </div>
          <div class="hone-home-blog-shot">
            <img src={featuredPost().heroImage} alt={featuredPost().title} loading="lazy" />
          </div>
        </section>

        {/* ── Plan 预告 ── */}
        <section class="hone-home-plan">
          <div>
            <div class="hone-home-eyebrow">{C.home_page.plan_eyebrow}</div>
            <h2>{C.home_page.plan_title}</h2>
            <p>{C.home_page.plan_desc}</p>
          </div>
          <button type="button" onClick={() => navigate("/plan")}>
            <span>{C.home_page.plan_cta}</span>
            <ICONS.ArrowRight />
          </button>
        </section>
      </main>

      <PublicMembershipModal open={buyOpen()} onClose={() => setBuyOpen(false)} />

      <PublicFooter />

      <style>{`
        .hone-home {
          position: relative;
          min-height: 100vh;
          display: flex;
          flex-direction: column;
          overflow-x: clip;
          background: var(--hone-paper-50);
          color: var(--hone-ink-800);
          font-family: var(--hone-font-body);
        }
        .hone-home-bg {
          position: absolute;
          inset: 0 0 auto;
          height: 860px;
          background:
            radial-gradient(760px 420px at 78% -80px, color-mix(in srgb, var(--hone-coral-500) 10%, transparent), transparent 68%),
            radial-gradient(640px 380px at 12% 120px, color-mix(in srgb, var(--hone-sage-100) 70%, transparent), transparent 70%);
          pointer-events: none;
        }
        .hone-home-main {
          position: relative;
          z-index: 1;
          width: min(1060px, calc(100% - 40px));
          margin: 0 auto;
          flex: 1;
        }

        /* Eyebrow：全站统一的 mono 小标 */
        .hone-home-eyebrow {
          color: var(--hone-coral-600);
          font-family: var(--hone-font-label);
          font-size: 11px;
          font-weight: 700;
          letter-spacing: 0.16em;
          text-transform: uppercase;
        }

        /* ── Hero ── */
        .hone-home-hero {
          display: flex;
          flex-direction: column;
          align-items: center;
          padding: 158px 0 0;
          text-align: center;
        }
        .hone-home-hero h1 {
          margin: 16px 0 0;
          color: var(--hone-ink-950);
          font-size: clamp(34px, 5.4vw, 58px);
          font-weight: 800;
          line-height: 1.07;
          letter-spacing: -0.045em;
        }
        .hone-home-hero h1 span,
        .hone-home-hero h1 em {
          display: block;
          font-style: normal;
        }
        .hone-home-hero h1 em {
          color: var(--hone-coral-600);
        }
        .hone-home-hero-desc {
          max-width: 560px;
          margin: 18px 0 0;
          color: var(--hone-ink-600);
          font-size: 14px;
          line-height: 1.8;
        }
        .hone-home-hero-actions {
          display: flex;
          align-items: center;
          gap: 10px;
          flex-wrap: wrap;
          justify-content: center;
          margin-top: 28px;
        }
        .hone-home-cta {
          display: inline-flex;
          align-items: center;
          gap: 8px;
          min-height: 46px;
          padding: 0 22px;
          border: 1px solid var(--hone-ink-950);
          border-radius: 999px;
          background: var(--hone-ink-950);
          color: #fff;
          cursor: pointer;
          font-size: 13px;
          font-weight: 700;
          text-decoration: none;
          transition: transform 0.18s var(--hone-ease), box-shadow 0.18s var(--hone-ease), background 0.18s ease, border-color 0.18s ease;
          box-shadow: 0 10px 26px rgba(23, 32, 31, 0.14);
        }
        .hone-home-cta:hover {
          transform: translateY(-2px);
          box-shadow: 0 14px 32px rgba(23, 32, 31, 0.18);
        }
        .hone-home-cta.is-buy {
          border-color: var(--hone-coral-500);
          background: var(--hone-coral-500);
          box-shadow: 0 10px 26px color-mix(in srgb, var(--hone-coral-500) 30%, transparent);
        }
        .hone-home-cta.is-buy:hover {
          border-color: var(--hone-coral-600);
          background: var(--hone-coral-600);
          box-shadow: 0 14px 32px color-mix(in srgb, var(--hone-coral-500) 38%, transparent);
        }
        .hone-home-cta.is-buy small {
          padding: 2px 8px;
          border-radius: 999px;
          background: rgba(255, 255, 255, 0.22);
          color: #fff;
          font-size: 11px;
          font-weight: 700;
        }
        .hone-home-cta.is-ghost {
          border-color: var(--hone-line-strong);
          background: rgba(255, 255, 255, 0.88);
          color: var(--hone-ink-950);
          box-shadow: none;
        }
        .hone-home-cta.is-ghost:hover {
          border-color: var(--hone-ink-950);
          background: #fff;
          box-shadow: var(--hone-shadow-sm);
        }
        .hone-home-cta small {
          padding: 2px 8px;
          border-radius: 999px;
          background: color-mix(in srgb, var(--hone-coral-500) 10%, transparent);
          color: var(--hone-coral-600);
          font-size: 11px;
          font-weight: 700;
          font-variant-numeric: tabular-nums;
        }
        .hone-home-stats {
          display: flex;
          align-items: stretch;
          gap: 0;
          margin-top: 34px;
        }
        .hone-home-stats > div {
          display: flex;
          flex-direction: column;
          gap: 4px;
          padding: 0 26px;
          border-left: 1px solid var(--hone-line);
        }
        .hone-home-stats > div:first-child { border-left: 0; }
        .hone-home-stats strong {
          color: var(--hone-ink-950);
          font-family: var(--hone-font-label);
          font-size: 17px;
          font-weight: 700;
          letter-spacing: -0.01em;
        }
        .hone-home-stats small {
          color: var(--hone-ink-400);
          font-size: 10px;
          font-weight: 650;
          letter-spacing: 0.08em;
          text-transform: uppercase;
        }

        /* ── 工作台窗口式演示框 ── */
        .hone-home-demo { margin-top: 54px; }
        .hone-home-window {
          overflow: hidden;
          border: 1px solid var(--hone-line);
          border-radius: 17px;
          background: #fff;
          box-shadow: 0 30px 80px rgba(23, 32, 31, 0.1);
        }
        .hone-home-window > header {
          display: flex;
          align-items: center;
          gap: 6px;
          padding: 11px 14px;
          border-bottom: 1px solid var(--hone-line);
          background: var(--hone-paper-100);
        }
        .hone-home-window > header i {
          width: 9px;
          height: 9px;
          border-radius: 50%;
          background: var(--hone-paper-200);
          border: 1px solid var(--hone-line);
        }
        .hone-home-window > header i:first-child {
          background: color-mix(in srgb, var(--hone-coral-500) 55%, #fff);
          border-color: color-mix(in srgb, var(--hone-coral-500) 40%, var(--hone-line));
        }
        .hone-home-window > header span {
          margin-left: 8px;
          color: var(--hone-ink-400);
          font-family: var(--hone-font-label);
          font-size: 10px;
          font-weight: 700;
          letter-spacing: 0.14em;
        }
        .hone-home-window-body { aspect-ratio: 16 / 9; background: var(--hone-ink-950); }
        .hone-home-window-body iframe { width: 100%; height: 100%; border: 0; display: block; }

        /* ── 区块通用 ── */
        .hone-home-section { margin-top: 84px; }
        .hone-home-section-head p {
          margin: 8px 0 0;
          color: var(--hone-ink-600);
          font-size: 13px;
        }

        /* 卖点三联 */
        .hone-home-trust {
          display: grid;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          gap: 14px;
          margin-top: 18px;
        }
        .hone-home-trust article {
          padding: 24px 22px 26px;
          border: 1px solid var(--hone-line);
          border-radius: 17px;
          background: #fff;
          transition: transform 0.18s var(--hone-ease), border-color 0.18s ease, box-shadow 0.18s var(--hone-ease);
        }
        .hone-home-trust article:hover {
          transform: translateY(-3px);
          border-color: var(--hone-line-strong);
          box-shadow: var(--hone-shadow-md);
        }
        .hone-home-trust span {
          display: grid;
          place-items: center;
          width: 34px;
          height: 34px;
          border-radius: 10px;
          background: color-mix(in srgb, var(--hone-coral-500) 9%, transparent);
          color: var(--hone-coral-600);
          font-size: 15px;
        }
        .hone-home-trust h3 {
          margin: 14px 0 0;
          color: var(--hone-ink-950);
          font-size: 15px;
          font-weight: 700;
          letter-spacing: -0.02em;
        }
        .hone-home-trust p {
          margin: 8px 0 0;
          color: var(--hone-ink-600);
          font-size: 12px;
          line-height: 1.7;
        }

        /* 案例面板 */
        .hone-home-cases {
          overflow: hidden;
          margin-top: 18px;
          border: 1px solid var(--hone-line);
          border-radius: 17px;
          background: #fff;
        }
        .hone-home-case-tabs {
          display: flex;
          gap: 2px;
          padding: 8px 10px;
          border-bottom: 1px solid var(--hone-line);
          background: var(--hone-paper-100);
          overflow-x: auto;
          scrollbar-width: none;
        }
        .hone-home-case-tabs::-webkit-scrollbar { display: none; }
        .hone-home-case-tabs button {
          flex: 0 0 auto;
          min-height: 32px;
          padding: 0 13px;
          border: 0;
          border-radius: 9px;
          background: transparent;
          color: var(--hone-ink-600);
          cursor: pointer;
          font-size: 12px;
          font-weight: 650;
          white-space: nowrap;
          transition: background 0.15s ease, color 0.15s ease;
        }
        .hone-home-case-tabs button:hover { color: var(--hone-ink-950); }
        .hone-home-case-tabs button.is-active {
          background: #fff;
          color: var(--hone-ink-950);
          box-shadow: var(--hone-shadow-sm);
        }
        .hone-home-case-body {
          display: grid;
          grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.1fr);
          gap: 30px;
          align-items: center;
          padding: 30px;
        }
        .hone-home-case-copy h2 {
          margin: 0;
          color: var(--hone-ink-950);
          font-size: clamp(20px, 2.6vw, 27px);
          line-height: 1.22;
          letter-spacing: -0.03em;
        }
        .hone-home-case-copy > p {
          margin: 12px 0 0;
          color: var(--hone-ink-600);
          font-size: 13px;
          line-height: 1.75;
        }
        .hone-home-case-link {
          display: inline-flex;
          align-items: center;
          gap: 7px;
          min-height: 38px;
          margin-top: 18px;
          padding: 0 16px;
          border: 1px solid var(--hone-ink-950);
          border-radius: var(--hone-radius-sm);
          background: var(--hone-ink-950);
          color: #fff;
          cursor: pointer;
          font-size: 12px;
          font-weight: 700;
          transition: transform 0.16s var(--hone-ease);
        }
        .hone-home-case-link:hover { transform: translateX(3px); }
        .hone-home-case-progress {
          height: 3px;
          margin-top: 24px;
          border-radius: 2px;
          background: var(--hone-paper-200);
          overflow: hidden;
        }
        .hone-home-case-progress > div {
          height: 100%;
          background: var(--hone-coral-500);
          transition: width 0.3s var(--hone-ease);
        }
        .hone-home-case-shot {
          position: relative;
          padding: 0;
          border: 1px solid var(--hone-line);
          border-radius: 13px;
          background: var(--hone-paper-100);
          overflow: hidden;
          cursor: zoom-in;
          aspect-ratio: 16 / 10;
        }
        .hone-home-case-shot img {
          width: 100%;
          height: 100%;
          object-fit: cover;
          display: block;
          animation: hone-home-fade 0.5s var(--hone-ease);
        }
        .hone-home-case-shot span {
          position: absolute;
          inset: 0;
          display: grid;
          place-items: center;
          background: rgba(23, 32, 31, 0.24);
          color: #fff;
          font-size: 12px;
          font-weight: 700;
          opacity: 0;
          transition: opacity 0.2s ease;
          backdrop-filter: blur(2px);
        }
        .hone-home-case-shot:hover span { opacity: 1; }

        /* 博客精选 */
        .hone-home-blog {
          display: grid;
          grid-template-columns: minmax(0, 0.92fr) minmax(0, 1fr);
          gap: 26px;
          margin-top: 84px;
          padding: 26px;
          border: 1px solid var(--hone-line);
          border-radius: 17px;
          background: #fff;
          cursor: pointer;
          transition: transform 0.18s var(--hone-ease), border-color 0.18s ease, box-shadow 0.18s var(--hone-ease);
        }
        .hone-home-blog:hover {
          transform: translateY(-3px);
          border-color: color-mix(in srgb, var(--hone-coral-500) 38%, var(--hone-line));
          box-shadow: var(--hone-shadow-md);
        }
        .hone-home-blog-copy {
          display: flex;
          flex-direction: column;
          align-items: flex-start;
          justify-content: center;
        }
        .hone-home-blog-copy h2 {
          margin: 12px 0 0;
          color: var(--hone-ink-950);
          font-size: clamp(21px, 2.8vw, 28px);
          line-height: 1.18;
          letter-spacing: -0.03em;
        }
        .hone-home-blog-copy p {
          margin: 12px 0 20px;
          color: var(--hone-ink-600);
          font-size: 13px;
          line-height: 1.7;
        }
        .hone-home-blog-copy button {
          display: inline-flex;
          align-items: center;
          gap: 7px;
          min-height: 38px;
          padding: 0 16px;
          border: 1px solid var(--hone-ink-950);
          border-radius: var(--hone-radius-sm);
          background: var(--hone-ink-950);
          color: #fff;
          cursor: pointer;
          font-size: 12px;
          font-weight: 700;
        }
        .hone-home-blog-shot {
          overflow: hidden;
          border: 1px solid var(--hone-line);
          border-radius: 13px;
          background: var(--hone-paper-100);
        }
        .hone-home-blog-shot img {
          width: 100%;
          height: 100%;
          min-height: 220px;
          object-fit: cover;
          display: block;
        }

        /* Plan 预告 */
        .hone-home-plan {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 26px;
          margin: 84px 0 96px;
          padding: 30px;
          border: 1px solid color-mix(in srgb, var(--hone-coral-500) 30%, var(--hone-line));
          border-radius: 17px;
          background:
            radial-gradient(420px 200px at 90% 0, color-mix(in srgb, var(--hone-coral-500) 9%, transparent), transparent 70%),
            #fff;
        }
        .hone-home-plan h2 {
          margin: 12px 0 0;
          color: var(--hone-ink-950);
          font-size: clamp(19px, 2.4vw, 24px);
          letter-spacing: -0.03em;
        }
        .hone-home-plan p {
          max-width: 560px;
          margin: 10px 0 0;
          color: var(--hone-ink-600);
          font-size: 13px;
          line-height: 1.7;
        }
        .hone-home-plan > button {
          display: inline-flex;
          align-items: center;
          gap: 8px;
          flex: 0 0 auto;
          min-height: 44px;
          padding: 0 20px;
          border: 1px solid var(--hone-ink-950);
          border-radius: 999px;
          background: var(--hone-ink-950);
          color: #fff;
          cursor: pointer;
          font-size: 13px;
          font-weight: 700;
          transition: transform 0.16s var(--hone-ease), box-shadow 0.16s var(--hone-ease);
        }
        .hone-home-plan > button:hover {
          transform: translateY(-2px);
          box-shadow: 0 12px 26px rgba(23, 32, 31, 0.16);
        }

        /* Lightbox */
        .hone-home-lightbox {
          position: fixed;
          inset: 0;
          z-index: 1000;
          display: grid;
          place-items: center;
          background: rgba(255, 253, 248, 0.96);
          backdrop-filter: blur(18px);
          cursor: zoom-out;
          animation: hone-home-fade 0.25s ease;
        }
        .hone-home-lightbox img {
          max-width: 92%;
          max-height: 88vh;
          border: 1px solid var(--hone-line);
          border-radius: var(--hone-radius-md);
          box-shadow: 0 40px 100px rgba(23, 32, 31, 0.2);
        }
        .hone-home-lightbox button {
          position: absolute;
          top: 26px;
          right: 34px;
          border: 0;
          background: transparent;
          color: var(--hone-ink-400);
          cursor: pointer;
          font-size: 44px;
          line-height: 1;
        }

        @keyframes hone-home-fade {
          from { opacity: 0; transform: translateY(10px); }
          to { opacity: 1; transform: translateY(0); }
        }

        /* ── 响应式：移动端做整体减负——少边框、少盒子、松弛的纵向节奏 ── */
        @media (max-width: 900px) {
          .hone-home-main { width: calc(100% - 36px); }
          .hone-home-hero { padding-top: 104px; text-align: left; align-items: flex-start; }
          .hone-home-hero h1 { margin-top: 14px; }
          .hone-home-hero-desc { margin-top: 14px; font-size: 13px; line-height: 1.75; }
          .hone-home-hero-actions { justify-content: flex-start; margin-top: 24px; }

          /* 统计行：去竖分割线，改为轻量行内标注，弱化盒子感 */
          .hone-home-stats { flex-wrap: wrap; gap: 8px 26px; margin-top: 28px; }
          .hone-home-stats > div {
            flex-direction: row;
            align-items: baseline;
            gap: 7px;
            padding: 0;
            border-left: 0;
          }
          .hone-home-stats strong { font-size: 14px; }

          /* 卖点三卡合并为一块单面板，卡片间用细分隔线，消除三层描边堆叠 */
          .hone-home-trust {
            grid-template-columns: 1fr;
            gap: 0;
            overflow: hidden;
            border: 1px solid var(--hone-line);
            border-radius: 16px;
            background: #fff;
          }
          .hone-home-trust article {
            display: grid;
            grid-template-columns: 34px minmax(0, 1fr);
            gap: 4px 14px;
            padding: 18px 16px;
            border: 0;
            border-bottom: 1px solid var(--hone-line);
            border-radius: 0;
          }
          .hone-home-trust article:last-child { border-bottom: 0; }
          .hone-home-trust article:hover { transform: none; box-shadow: none; }
          .hone-home-trust span { width: 34px; height: 34px; grid-row: 1 / 3; }
          .hone-home-trust h3 { margin: 0; align-self: center; font-size: 14px; }
          .hone-home-trust p { grid-column: 2; margin: 2px 0 0; font-size: 12px; line-height: 1.65; }

          .hone-home-section { margin-top: 56px; }
          .hone-home-section-head p { font-size: 12px; }
          .hone-home-demo { margin-top: 36px; }
          .hone-home-window { border-radius: 14px; }
          .hone-home-window > header { padding: 10px 12px; }

          .hone-home-cases { border-radius: 15px; }
          .hone-home-case-tabs { padding: 7px 8px; }
          .hone-home-case-body { grid-template-columns: 1fr; gap: 18px; padding: 18px 16px 20px; }
          .hone-home-case-copy > p { font-size: 12px; }
          .hone-home-case-progress { margin-top: 18px; }

          .hone-home-blog { grid-template-columns: 1fr; gap: 14px; margin-top: 56px; padding: 16px 16px 20px; border-radius: 15px; }
          .hone-home-blog-shot { order: -1; }
          .hone-home-blog-shot img { min-height: 0; aspect-ratio: 16 / 9; }
          .hone-home-blog-copy p { margin-bottom: 16px; font-size: 12px; }

          .hone-home-plan { flex-direction: column; align-items: flex-start; gap: 18px; margin: 56px 0 64px; padding: 20px 16px; border-radius: 15px; }
          .hone-home-plan p { font-size: 12px; }
          .hone-home-plan > button { width: 100%; justify-content: center; }
        }
        @media (max-width: 480px) {
          .hone-home-hero { padding-top: 96px; }
          .hone-home-hero h1 { font-size: 29px; letter-spacing: -0.035em; }
          .hone-home-hero-actions { width: 100%; flex-direction: column; align-items: stretch; gap: 9px; }
          .hone-home-cta { width: 100%; justify-content: center; min-height: 48px; }
          .hone-home-stats { gap: 6px 22px; }
        }
      `}</style>
    </div>
  )
}
