// public-home.tsx — Hone Public Site Homepage (v4.2 - Refined Proportions & Balanced Layout)

import {
  createSignal,
  onCleanup,
  onMount,
  createResource,
  Show,
  For,
} from "solid-js"
import { useNavigate } from "@solidjs/router"
import { CONTENT } from "@/lib/public-content"
import { setLocale, useLocale } from "@/lib/i18n"
import "./public-site.css"

// ── GitHub Star Fetching ─────────────────────────────────────────────────────
async function fetchGithubStars() {
  try {
    const res = await fetch("https://api.github.com/repos/B-M-Capital-Research/honeclaw")
    const data = await res.json()
    return data.stargazers_count || "..."
  } catch (e) {
    return "..."
  }
}

// ── Icons ────────────────────────────────────────────────────────────────────
const ICONS = {
  Chat: () => (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
  ),
  Github: () => (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
  ),
  Youtube: () => (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.016 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.016 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/></svg>
  ),
  Bilibili: () => (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M17.813 4.653h.854c1.51.054 2.769.578 3.773 1.574 1.004.995 1.524 2.249 1.56 3.76v7.36c-.036 1.51-.556 2.769-1.56 3.773s-2.262 1.524-3.773 1.56H5.333c-1.51-.036-2.769-.556-3.773-1.56S.036 18.883 0 17.373v-7.36c.036-1.51.556-2.765 1.56-3.76 1.004-.996 2.262-1.52 3.773-1.574h.774l-1.174-1.12a1.277 1.277 0 0 1-.388-.933c0-.346.138-.64.414-.88a1.277 1.277 0 0 1 .906-.36c.345 0 .647.127.906.38l2.227 2.12h4.72l2.227-2.12c.27-.253.57-.38.906-.38.365 0 .65.12.853.36.277.24.414.534.414.88 0 .346-.13.653-.387.92zm-12.48 5.387c-.331.03-.593.15-.786.36-.193.21-.29.473-.29.787v3.507c0 .313.097.576.29.786.193.21.455.33.786.36.331-.03.593-.15.786-.36.193-.21.29-.473.29-.786v-3.507c0-.314-.097-.577-.29-.787-.193-.21-.455-.33-.786-.36zm10.707 0c-.331.03-.593.15-.786.36-.193.21-.29.473-.29.787v3.507c0 .313.097.576.29.786.193.21.455.33.786.36.345-.03.607-.15.786-.36.193-.21.29-.473.29-.786v-3.507c0-.314-.097-.577-.29-.787-.193-.21-.455-.33-.786-.36zM18 19.04H6.013c-.113 0-.17.053-.17.16 0 .12.057.18.17.18H18c.113 0 .17-.06.17-.18 0-.107-.057-.16-.17-.16z"/></svg>
  ),
  ArrowRight: () => (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
  )
}

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
  const [stars] = createResource(fetchGithubStars)
  const C = CONTENT.nav

  return (
    <header class="page-header">
      <div onClick={() => navigate("/")} class="header-logo">
        <img src="/logo.svg" alt="Hone" />
        <span>Hone</span>
      </div>

      <div class="header-actions">
        <div class="header-socials mobile-hide">
          <a href="https://www.youtube.com/@HoneFinancial" target="_blank" class="icon-btn-ghost" title="YouTube"><ICONS.Youtube /></a>
          <a href="https://www.bilibili.com/video/BV1ByXNBGET5/" target="_blank" class="icon-btn-ghost" title="Bilibili"><ICONS.Bilibili /></a>
          <a href="https://github.com/B-M-Capital-Research/honeclaw" target="_blank" class="star-badge">
            <ICONS.Github />
            <span>{stars() || "..."}</span>
          </a>
        </div>

        <div class="divider-v mobile-hide" />

        <div class="lang-switch">
          <button onClick={() => setLocale("zh")} class={useLocale() === "zh" ? "active" : ""}>中</button>
          <button onClick={() => setLocale("en")} class={useLocale() === "en" ? "active" : ""}>EN</button>
        </div>

        <div style={{ display: "flex", gap: "8px" }}>
          <button onClick={() => navigate("/roadmap")} class="btn-roadmap-nav mobile-hide">
            {useLocale() === 'zh' ? '产品路线图' : 'Roadmap'}
          </button>
          <button onClick={() => navigate("/chat")} class="btn-chat-nav">{C.chat}</button>
        </div>
      </div>
    </header>
  )
}

export default function PublicHomePage() {
  const [index, setIndex] = createSignal(0)
  const [enlargeImg, setEnlargeImg] = createSignal<string | null>(null)
  const navigate = useNavigate()

  const slides = () => [
    ...CONTENT.cases.items.filter(i => i.image).map(i => ({
      tag: i.tag,
      title: i.title,
      body: i.body,
      image: i.image,
      link: null,
    })),
    {
      tag: useLocale() === "zh" ? "路线图" : "ROADMAP",
      title: CONTENT.roadmap.hero_title,
      body: CONTENT.roadmap.hero_sub,
      image: useLocale() === "zh" ? "/hone_solution_zh.jpg" : "/hone_solution.jpg",
      link: "/roadmap",
    }
  ]

  const videoUrl = () => useLocale() === "zh"
    ? "https://player.bilibili.com/player.html?bvid=BV1ByXNBGET5&page=1&high_quality=1&danmaku=0&autoplay=0"
    : "https://www.youtube.com/embed/hJr-81OdYcQ?autoplay=0"

  let timer: any
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

  return (
    <div class="hone-landing-v4">
      <AnimatedBackground />
      <Header />
      
      {/* Lightbox */}
      <Show when={enlargeImg()}>
        <div class="lightbox-overlay" onClick={() => setEnlargeImg(null)}>
          <img src={enlargeImg()!} class="lightbox-img" />
          <button class="lightbox-close">×</button>
        </div>
      </Show>

      <main class="main-content">
        {/* HERO SECTION */}
        <section class="hero-section">
          <div class="hero-logo-tag">
            <img src="/logo.svg" class="hero-logo" />
            <h1 class="hero-tagline">
              {useLocale() === "zh" 
                ? "并非迎合你的聊天玩具，而是你投资纪律的无情捍卫者。" 
                : "Not a chatbot that flatters you, but a ruthless defender of your investment discipline."}
            </h1>
          </div>

          <div class="hero-btns">
            <button onClick={() => navigate("/chat")} class="btn-primary refined">
              <ICONS.Chat />
              <span>{useLocale() === "zh" ? "开始试用" : "Start Now"}</span>
            </button>
            <a href="https://github.com/B-M-Capital-Research/honeclaw" target="_blank" class="btn-secondary refined">
              <ICONS.Github />
              <span>GitHub</span>
            </a>
            <a href="https://www.bilibili.com/video/BV1ByXNBGET5/" target="_blank" class="btn-secondary refined bilibili-btn mobile-hide">
              <ICONS.Bilibili />
              <span>Bilibili</span>
            </a>
            <a href="https://www.youtube.com/@HoneFinancial" target="_blank" class="btn-secondary refined youtube-btn mobile-hide">
              <ICONS.Youtube />
              <span>YouTube</span>
            </a>
          </div>

          <div class="video-container">
            <div class="video-label">{useLocale() === "zh" ? "视频演示" : "VIDEO DEMO"}</div>
            <div class="video-wrapper">
              <iframe src={videoUrl()} allowfullscreen />
            </div>
          </div>
        </section>

        <div class="section-separator">
          <div class="line"></div>
          <div class="carousel-nav">
            <For each={slides()}>
              {(s, i) => (
                <button 
                  onClick={() => { setIndex(i()); startTimer(); }}
                  class={`nav-item ${i() === index() ? 'active' : ''}`}
                >
                  {s.tag}
                </button>
              )}
            </For>
          </div>
          <div class="line"></div>
        </div>

        {/* FEATURES CAROUSEL */}
        <section class="carousel-section">
          <div class="carousel-container">
            <div class="carousel-text">
              <h2 class="feature-title">{current().title}</h2>
              <p class="feature-body">{current().body}</p>
              
              <Show when={current().link}>
                <button 
                  onClick={() => navigate(current().link!)}
                  class="btn-feature-link"
                >
                  <span>{useLocale() === 'zh' ? '完整路线图' : 'View Full Roadmap'}</span>
                  <ICONS.ArrowRight />
                </button>
              </Show>

              <div class="carousel-progress">
                <div class="progress-bar" style={{ width: `${((index() + 1) / slides().length) * 100}%` }}></div>
              </div>
            </div>

            <div class="carousel-image" onClick={() => setEnlargeImg(current().image)}>
              <img src={current().image} class="feature-img" />
              <div class="zoom-hint">{useLocale() === 'zh' ? '查看详情' : 'Zoom In'}</div>
            </div>
          </div>
        </section>
      </main>

      <style>{`
        .hone-landing-v4 {
          background: #fff; color: #1e293b;
          font-family: var(--font-sans, 'Plus Jakarta Sans', sans-serif);
          min-height: 100vh; overflow-x: hidden; position: relative;
          display: flex; flex-direction: column;
        }

        /* Animated Background */
        .animated-bg { position: absolute; inset: 0; z-index: 0; overflow: hidden; pointer-events: none; }
        .circle { position: absolute; border-radius: 50%; filter: blur(100px); opacity: 0.12; animation: float 30s infinite alternate ease-in-out; }
        .circle-1 { width: 800px; height: 800px; background: #f59e0b; top: -200px; left: -100px; }
        .circle-2 { width: 900px; height: 900px; background: #3b82f6; bottom: -200px; right: -100px; animation-delay: -5s; }
        .circle-3 { width: 500px; height: 500px; background: #ec4899; top: 40%; left: 50%; animation-delay: -10s; }
        @keyframes float { 0% { transform: translate(0, 0) scale(1); } 100% { transform: translate(50px, 100px) scale(1.1); } }

        /* Header */
        .page-header {
          position: fixed; top: 0; left: 0; right: 0; height: 72px;
          display: flex; align-items: center; justify-content: space-between;
          padding: 0 40px; background: rgba(255, 255, 255, 0.85);
          backdrop-filter: blur(16px); border-bottom: 1px solid rgba(0,0,0,0.04); z-index: 100;
        }
        .header-logo { display: flex; align-items: center; gap: 12px; cursor: pointer; }
        .header-logo img { height: 32px; }
        .header-logo span { font-weight: 800; font-size: 20px; color: #000; letter-spacing: -0.01em; }
        .header-actions { display: flex; align-items: center; gap: 24px; }
        .header-socials { display: flex; align-items: center; gap: 12px; }
        .icon-btn-ghost { color: #64748b; padding: 8px; border-radius: 8px; display: flex; transition: all 0.2s; }
        .icon-btn-ghost:hover { background: #f1f5f9; color: #000; }
        .star-badge { display: flex; align-items: center; gap: 6px; color: #1e293b; text-decoration: none; font-size: 14px; font-weight: 700; background: #f1f5f9; padding: 6px 12px; border-radius: 8px; transition: all 0.2s; }
        .star-badge:hover { background: #e2e8f0; }
        .divider-v { width: 1px; height: 20px; background: #e2e8f0; }
        .lang-switch { display: flex; background: #f1f5f9; padding: 3px; border-radius: 10px; }
        .lang-switch button { padding: 4px 14px; border: none; background: transparent; cursor: pointer; font-size: 13px; font-weight: 700; color: #64748b; }
        .lang-switch button.active { background: #fff; color: #000; border-radius: 7px; box-shadow: 0 2px 4px rgba(0,0,0,0.06); }
        .btn-chat-nav { background: #000; color: #fff; border: none; padding: 8px 20px; border-radius: 100px; font-size: 14px; font-weight: 700; cursor: pointer; }
        .btn-roadmap-nav { background: transparent; color: #64748b; border: 1.5px solid #e2e8f0; padding: 8px 20px; border-radius: 100px; font-size: 14px; font-weight: 700; cursor: pointer; transition: all 0.2s; }
        .btn-roadmap-nav:hover { background: #f8fafc; border-color: #cbd5e1; color: #1e293b; }

        /* Main */
        .main-content { position: relative; z-index: 1; padding: 72px 20px 80px; display: flex; flex-direction: column; align-items: center; max-width: 1400px; margin: 0 auto; width: 100%; }

        /* Hero */
        .hero-section { padding: 80px 0 40px; display: flex; flex-direction: column; align-items: center; gap: 32px; width: 100%; }
        .hero-logo-tag { display: flex; flex-direction: column; align-items: center; gap: 16px; }
        .hero-logo { height: 130px; filter: drop-shadow(0 10px 30px rgba(0,0,0,0.06)); } /* REFINED LOGO SIZE */
        .hero-tagline { font-size: 38px; font-weight: 800; color: #1e293b; text-align: center; max-width: 800px; line-height: 1.2; letter-spacing: -0.02em; }
        .hero-btns { display: flex; gap: 12px; flex-wrap: wrap; justify-content: center; }
        
        .btn-primary.refined { background: #000; color: #fff; border: none; padding: 14px 32px; border-radius: 100px; font-size: 16px; font-weight: 700; cursor: pointer; display: flex; align-items: center; gap: 10px; transition: transform 0.2s, box-shadow 0.2s; box-shadow: 0 8px 20px rgba(0,0,0,0.12); }
        .btn-secondary.refined { background: #fff; color: #000; border: 1.5px solid #e2e8f0; padding: 14px 28px; border-radius: 100px; font-size: 16px; font-weight: 700; cursor: pointer; text-decoration: none; display: flex; align-items: center; gap: 10px; transition: all 0.2s; }
        .btn-primary.refined:hover { transform: translateY(-2px); box-shadow: 0 12px 25px rgba(0,0,0,0.18); }
        .btn-secondary.refined:hover { border-color: #cbd5e1; background: #f8fafc; transform: translateY(-1px); }
        .bilibili-btn:hover { color: #fb7299; border-color: #fb7299; }
        .youtube-btn:hover { color: #ff0000; border-color: #ff0000; }

        .video-container { width: 100%; max-width: 960px; display: flex; flex-direction: column; align-items: center; gap: 12px; margin-top: 20px; }
        .video-label { font-size: 12px; font-weight: 800; color: #94a3b8; letter-spacing: 0.2em; text-transform: uppercase; }
        .video-wrapper { width: 100%; aspect-ratio: 16/9; background: #000; border-radius: 28px; overflow: hidden; box-shadow: 0 30px 60px rgba(0,0,0,0.1); border: 1px solid #f1f5f9; }
        .video-wrapper iframe { width: 100%; height: 100%; border: none; }

        /* Carousel Nav */
        .section-separator { width: 100%; margin: 80px 0 48px; display: flex; align-items: center; gap: 32px; }
        .section-separator .line { flex: 1; height: 1px; background: #f1f5f9; }
        .carousel-nav { display: flex; gap: 24px; overflow-x: auto; padding: 4px; }
        .nav-item { border: none; background: transparent; color: #94a3b8; font-size: 14px; font-weight: 800; cursor: pointer; white-space: nowrap; transition: all 0.2s; position: relative; padding: 4px 0; }
        .nav-item:hover { color: #64748b; }
        .nav-item.active { color: #000; }
        .nav-item.active::after { content: ''; position: absolute; bottom: -4px; left: 0; right: 0; height: 2px; background: #f59e0b; border-radius: 2px; }

        /* Carousel Content */
        .carousel-section { width: 100%; max-width: 1100px; margin-bottom: 60px; }
        .carousel-container { display: flex; align-items: center; gap: 64px; }
        .carousel-text { flex: 1; display: flex; flex-direction: column; align-items: flex-start; }
        .feature-title { font-size: 44px; font-weight: 800; color: #0f172a; margin-bottom: 20px; line-height: 1.2; letter-spacing: -0.01em; }
        .feature-body { font-size: 18px; color: #475569; line-height: 1.6; margin-bottom: 32px; }
        .btn-feature-link { display: flex; align-items: center; gap: 8px; background: #000; color: #fff; border: none; padding: 12px 24px; border-radius: 12px; font-size: 14px; font-weight: 700; cursor: pointer; transition: all 0.2s; margin-bottom: 32px; }
        .btn-feature-link:hover { transform: translateX(4px); }

        .carousel-progress { height: 4px; width: 100%; background: #f1f5f9; border-radius: 2px; overflow: hidden; }
        .progress-bar { height: 100%; background: #000; transition: width 0.3s; }

        .carousel-image { flex: 1.4; aspect-ratio: 16/10; border-radius: 24px; overflow: hidden; border: 1.5px solid #f1f5f9; box-shadow: 0 20px 50px rgba(0,0,0,0.04); cursor: zoom-in; position: relative; }
        .carousel-image:hover .zoom-hint { opacity: 1; }
        .zoom-hint { position: absolute; inset: 0; background: rgba(0,0,0,0.1); display: flex; align-items: center; justify-content: center; color: #fff; font-weight: 700; font-size: 14px; opacity: 0; transition: opacity 0.3s; backdrop-filter: blur(4px); }
        .feature-img { width: 100%; height: 100%; object-fit: cover; animation: fade-up 0.6s cubic-bezier(0.16, 1, 0.3, 1); }

        /* Lightbox */
        .lightbox-overlay { position: fixed; inset: 0; background: rgba(255,255,255,0.96); backdrop-filter: blur(20px); z-index: 1000; display: flex; align-items: center; justify-content: center; cursor: zoom-out; animation: fade-in 0.3s; }
        .lightbox-img { max-width: 92%; max-height: 88vh; border-radius: 16px; box-shadow: 0 40px 100px rgba(0,0,0,0.2); border: 1px solid #e2e8f0; }
        .lightbox-close { position: absolute; top: 32px; right: 48px; font-size: 64px; border: none; background: transparent; cursor: pointer; color: #94a3b8; line-height: 1; }

        @keyframes fade-in { from { opacity: 0; } to { opacity: 1; } }
        @keyframes fade-up { from { opacity: 0; transform: translateY(20px); } to { opacity: 1; transform: translateY(0); } }

        /* Responsive */
        @media (max-width: 1024px) {
          .carousel-container { flex-direction: column; gap: 40px; }
          .carousel-text { text-align: center; width: 100%; align-items: center; }
          .carousel-image { width: 100%; }
          .feature-title { font-size: 32px; }
        }
        @media (max-width: 640px) {
          .hero-logo { height: 90px; }
          .hero-tagline { font-size: 26px; }
          .btn-primary.refined, .btn-secondary.refined { width: 100%; padding: 14px 24px; font-size: 16px; }
          .page-header { padding: 0 20px; }
          .carousel-nav { gap: 16px; }
          .nav-item { font-size: 13px; }
          .feature-title { font-size: 28px; }
          .feature-body { font-size: 16px; }
        }
      `}</style>
    </div>
  )
}
