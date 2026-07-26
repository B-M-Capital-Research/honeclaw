// public-plan.tsx — 品牌聚合落地页（link-in-bio 式）：
// 头像身份区（Hari老王 × HONE）→ 左栏免费内容（YouTube 频道卡 + 近期视频
// 缩略图 + Bilibili + HONE 工具）｜右栏会员转化（价格 + 权益 + 免费体验群）
// → 六张海报横滑条。中文「立即购买」弹知识星球二维码，英文跳 Whop。

import { createSignal, For, Show, onCleanup, createEffect } from "solid-js"
import { CONTENT } from "@/lib/public-content"
import { useLocale } from "@/lib/i18n"
import { PublicFooter, PublicNav } from "@/components/public-nav"
import "./public-site.css"

const WHOP_URL = "https://whop.com/edda1183-b297-4502-811f-339ae5e773be/bm-research-membership/"
const YOUTUBE_URL = "https://www.youtube.com/@Hari%E8%80%81%E7%8E%8B/videos"
const BILIBILI_URL = "https://www.bilibili.com/video/BV1ByXNBGET5/"
const GITHUB_URL = "https://github.com/B-M-Capital-Research/honeclaw"

/* 频道近期视频（手工挑选置顶三条；缩略图走 YouTube 官方 CDN）。 */
const CHANNEL_VIDEOS = [
  { id: "VkPJOPwrDdI", title: "AI 时代真正卡脖子的，可能不是 GPU：美光 MU 的重估逻辑" },
  { id: "m2VLkhoPeVw", title: "AI 芯片良率背后的赢家：KLAC 凭什么拿下 60% 检测市场？" },
  { id: "ii5M8eyta2g", title: "AMAT 凭什么成为半导体设备第一龙头？" },
]

/* 六张海报：1052×1870（9:16）。bm1 是邀请函总览，放在首位。 */
const POSTERS = [
  { src: "/bm1.webp", alt: "巴芒投研会员邀请：研报、财报前瞻、直播与社群总览" },
  { src: "/bm2.webp", alt: "星球内容：每年 300+ 份美股公司原创万字研报" },
  { src: "/bm3.webp", alt: "星球内容：KANO / SWOT / DCF 等方式深度估值" },
  { src: "/bm4.webp", alt: "星球内容：每季 100+ 份财报前瞻精准预测" },
  { src: "/bm5.webp", alt: "星球内容：每周主理人亲自直播讲解精选公司" },
  { src: "/bm6.webp", alt: "星球内容：社群内和数百优质会员实时探讨分享" },
]

function Lightbox(props: {
  open: boolean
  src: string
  title: string
  hint: string
  onClose: () => void
}) {
  createEffect(() => {
    if (!props.open) return
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") props.onClose()
    }
    document.addEventListener("keydown", onKeyDown)
    onCleanup(() => document.removeEventListener("keydown", onKeyDown))
  })
  return (
    <Show when={props.open}>
      <div class="hone-share-pop" onClick={props.onClose}>
        <figure onClick={(event) => event.stopPropagation()}>
          <figcaption>
            <strong>{props.title}</strong>
            <button type="button" aria-label={CONTENT.plan.close_aria} onClick={props.onClose}>×</button>
          </figcaption>
          <img src={props.src} alt={props.title} />
          <Show when={props.hint}>
            <small>{props.hint}</small>
          </Show>
        </figure>
      </div>
    </Show>
  )
}

export default function PublicPlanPage() {
  const C = () => CONTENT.plan
  const [lightbox, setLightbox] = createSignal<
    | { kind: "poster"; index: number }
    | { kind: "join" }
    | { kind: "support" }
    | null
  >(null)
  const isZh = () => useLocale() === "zh"

  const buy = () => {
    if (isZh()) setLightbox({ kind: "join" })
    else window.open(WHOP_URL, "_blank", "noopener,noreferrer")
  }

  const activePoster = () => {
    const state = lightbox()
    return state?.kind === "poster" ? POSTERS[state.index] : null
  }

  return (
    <div class="pub-page hone-share-page">
      <PublicNav />
      <main class="hone-share-main">
        {/* ── 身份区：头像 + 名字 + 数据 ── */}
        <header class="hone-hub-hero">
          <img class="hone-hub-avatar" src="/hari-avatar.jpg" alt="Hari老王头像" width="96" height="96" />
          <div class="hone-hub-id">
            <div class="hone-share-eyebrow">{C().eyebrow}</div>
            <h1>{C().host.name}<span>{C().host.role}</span></h1>
            <p>{C().host.bio}</p>
            <div class="hone-hub-stats">
              <For each={C().host.stats}>
                {(stat) => (
                  <div>
                    <strong>{stat.value}</strong>
                    <small>{stat.label}</small>
                  </div>
                )}
              </For>
            </div>
            <p class="hone-hub-stats-note">{C().host.stats_note}</p>
          </div>
        </header>

        {/* ── 双栏：左内容 / 右转化 ── */}
        <div class="hone-hub-grid">
          <div class="hone-hub-content">
            {/* 免费内容：YouTube 频道 + 近期视频 + Bilibili */}
            <section class="hone-hub-card" aria-label={C().channel.label}>
              <div class="hone-hub-label">{C().channel.label}</div>
              <div class="hone-hub-channel">
                <img src="/hari-avatar.jpg" alt="" width="46" height="46" />
                <div>
                  <strong>{C().channel.title}</strong>
                  <small>{C().channel.handle}</small>
                </div>
                <a class="hone-hub-ghost-btn" href={YOUTUBE_URL} target="_blank" rel="noopener noreferrer">
                  {C().channel.cta} →
                </a>
              </div>
              <div class="hone-hub-videos-label">{C().channel.videos_label}</div>
              <div class="hone-hub-videos">
                <For each={CHANNEL_VIDEOS}>
                  {(video) => (
                    <a
                      class="hone-hub-video"
                      href={`https://www.youtube.com/watch?v=${video.id}`}
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      <span class="hone-hub-video-thumb">
                        <img src={`https://i.ytimg.com/vi/${video.id}/mqdefault.jpg`} alt="" loading="lazy" />
                        <i aria-hidden="true">▶</i>
                      </span>
                      <strong>{video.title}</strong>
                    </a>
                  )}
                </For>
              </div>
              <div class="hone-hub-channel is-bilibili">
                <span class="hone-hub-bili-mark" aria-hidden="true">b</span>
                <div>
                  <strong>{C().channel.bilibili_title}</strong>
                  <small>{C().channel.bilibili_desc}</small>
                </div>
                <a class="hone-hub-ghost-btn" href={BILIBILI_URL} target="_blank" rel="noopener noreferrer">
                  {C().channel.bilibili_cta} →
                </a>
              </div>
            </section>

            {/* HONE 工具 */}
            <section class="hone-hub-card" aria-label={C().product_card.label}>
              <div class="hone-hub-label">{C().product_card.label}</div>
              <div class="hone-hub-product">
                <span class="hone-hub-product-mark" aria-hidden="true">H</span>
                <div>
                  <strong>{C().product_card.title}</strong>
                  <p>{C().product_card.desc}</p>
                  <div class="hone-hub-product-actions">
                    <a class="hone-hub-solid-btn" href="/chat">{C().product_card.cta_chat} →</a>
                    <a class="hone-hub-ghost-btn" href={GITHUB_URL} target="_blank" rel="noopener noreferrer">
                      {C().product_card.cta_github}
                    </a>
                  </div>
                </div>
              </div>
            </section>
          </div>

          {/* 右栏：会员卡（桌面吸顶） */}
          <aside class="hone-hub-rail">
            <section class="hone-hub-card hone-hub-member" aria-label={C().member.label}>
              <div class="hone-hub-label">{C().member.label}</div>
              <div class="hone-hub-member-head">
                <strong>{C().member.title}</strong>
                <div class="hone-hub-price">
                  <b>{C().full.price}</b>
                  <span>{C().full.period}</span>
                </div>
              </div>
              <Show when={C().full.promos.length > 0}>
                <div class="hone-share-promos">
                  <For each={C().full.promos}>{(promo) => <span>{promo}</span>}</For>
                </div>
              </Show>
              <ul class="hone-hub-benefits">
                <For each={C().member.benefits}>
                  {(benefit) => (
                    <li>
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 6 9 17l-5-5" /></svg>
                      <span><strong>{benefit.title}</strong><small>{benefit.desc}</small></span>
                    </li>
                  )}
                </For>
              </ul>
              <button type="button" class="hone-share-buy hone-hub-buy" onClick={buy}>
                {C().full.cta}
              </button>
              <button type="button" class="hone-hub-trial-btn" onClick={() => setLightbox({ kind: "support" })}>
                {C().member.trial_cta}
              </button>
              <p class="hone-hub-trial-hint">{C().member.trial_hint}</p>
              <p class="hone-hub-foot">{C().foot}</p>
            </section>
          </aside>
        </div>

        {/* ── 海报横滑条 ── */}
        <section class="hone-hub-posters" aria-label={C().posters_label}>
          <div class="hone-hub-label">{C().posters_label}</div>
          <div class="hone-share-wall">
            <For each={POSTERS}>
              {(poster, i) => (
                <button
                  type="button"
                  class="hone-share-poster"
                  onClick={() => setLightbox({ kind: "poster", index: i() })}
                >
                  <img
                    src={poster.src}
                    alt={poster.alt}
                    loading={i() === 0 ? "eager" : "lazy"}
                    decoding="async"
                  />
                </button>
              )}
            </For>
          </div>
        </section>

        <p class="hone-share-disclaimer">
          海报为会员服务介绍。过往表现不代表未来收益，所有内容仅供研究参考，不构成投资建议；市场有风险，投资决策需独立判断。
        </p>
      </main>

      {/* 移动端吸附购买栏 */}
      <div class="hone-share-dock">
        <button type="button" class="hone-share-buy" onClick={buy}>
          <span>{C().full.cta}</span>
          <b>{C().full.price}{C().full.period}</b>
        </button>
        <button type="button" class="hone-share-service" onClick={() => setLightbox({ kind: "support" })}>
          {C().support.title}
        </button>
      </div>

      <PublicFooter />

      <Show when={activePoster()}>
        {(poster) => (
          <Lightbox
            open
            src={poster().src}
            title={C().share_title}
            hint=""
            onClose={() => setLightbox(null)}
          />
        )}
      </Show>
      <Lightbox
        open={lightbox()?.kind === "join"}
        src="/membership_zsxq.jpg"
        title={C().full.qr_title}
        hint={C().full.qr_hint}
        onClose={() => setLightbox(null)}
      />
      <Lightbox
        open={lightbox()?.kind === "support"}
        src="/membership_wechat.jpg"
        title={C().support.title}
        hint={C().support.desc}
        onClose={() => setLightbox(null)}
      />

      <style>{`
        .hone-share-page {
          min-height: 100vh;
          display: flex;
          flex-direction: column;
          background:
            radial-gradient(820px 400px at 82% -60px, color-mix(in srgb, var(--hone-coral-500) 10%, transparent), transparent 70%),
            var(--hone-paper-50);
          color: var(--hone-ink-800);
          font-family: var(--hone-font-body);
        }
        .hone-share-main {
          width: min(1080px, calc(100% - 40px));
          margin: 0 auto;
          padding: 122px 0 88px;
          flex: 1;
        }

        /* ── 身份区 ── */
        .hone-hub-hero {
          display: flex;
          align-items: flex-start;
          gap: 24px;
        }
        .hone-hub-avatar {
          width: 96px;
          height: 96px;
          flex: 0 0 96px;
          border: 1px solid var(--hone-line);
          border-radius: 26px;
          background: #fff;
          box-shadow: var(--hone-shadow-md);
          object-fit: cover;
        }
        .hone-hub-id { min-width: 0; flex: 1; }
        .hone-share-eyebrow {
          color: var(--hone-coral-600);
          font-family: var(--hone-font-label);
          font-size: 11px;
          font-weight: 700;
          letter-spacing: 0.16em;
        }
        .hone-hub-id h1 {
          display: flex;
          align-items: baseline;
          flex-wrap: wrap;
          gap: 10px 12px;
          margin: 8px 0 0;
          color: var(--hone-ink-950);
          font-size: clamp(26px, 3.4vw, 34px);
          line-height: 1.1;
          letter-spacing: -0.04em;
        }
        .hone-hub-id h1 span {
          padding: 4px 10px;
          border: 1px solid var(--hone-line);
          border-radius: 999px;
          background: #fff;
          color: var(--hone-ink-600);
          font-size: 11px;
          font-weight: 650;
          letter-spacing: 0;
        }
        .hone-hub-id > p {
          max-width: 640px;
          margin: 10px 0 0;
          color: var(--hone-ink-600);
          font-size: 13px;
          line-height: 1.75;
        }
        .hone-hub-stats {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
          margin-top: 14px;
        }
        .hone-hub-stats > div {
          display: flex;
          align-items: baseline;
          gap: 7px;
          padding: 8px 13px;
          border: 1px solid var(--hone-line);
          border-radius: 11px;
          background: #fff;
        }
        .hone-hub-stats strong {
          color: var(--hone-ink-950);
          font-size: 16px;
          font-weight: 750;
          letter-spacing: -0.02em;
          font-variant-numeric: tabular-nums;
        }
        .hone-hub-stats small { color: var(--hone-ink-600); font-size: 11px; }
        .hone-hub-stats-note {
          margin: 7px 2px 0;
          color: var(--hone-ink-400, #a3a6a1);
          font-size: 10px;
        }

        /* ── 双栏 ── */
        .hone-hub-grid {
          display: grid;
          grid-template-columns: minmax(0, 1fr) 356px;
          gap: 16px;
          align-items: start;
          margin-top: 28px;
        }
        .hone-hub-content { display: grid; gap: 16px; min-width: 0; }
        .hone-hub-grid > * { min-width: 0; }
        .hone-hub-card {
          min-width: 0;
          padding: 20px 22px;
          border: 1px solid var(--hone-line);
          border-radius: 17px;
          background: #fff;
        }
        .hone-hub-label {
          margin-bottom: 14px;
          color: var(--hone-ink-400, #9aa09b);
          font-family: var(--hone-font-label);
          font-size: 10px;
          font-weight: 700;
          letter-spacing: 0.15em;
          text-transform: uppercase;
        }

        /* 频道行 */
        .hone-hub-channel {
          display: flex;
          align-items: center;
          gap: 13px;
        }
        .hone-hub-channel img {
          width: 46px;
          height: 46px;
          border: 1px solid var(--hone-line);
          border-radius: 50%;
          object-fit: cover;
        }
        .hone-hub-channel > div { min-width: 0; flex: 1; }
        .hone-hub-channel strong {
          display: block;
          color: var(--hone-ink-950);
          font-size: 15px;
          font-weight: 750;
          letter-spacing: -0.01em;
        }
        .hone-hub-channel small {
          display: block;
          margin-top: 2px;
          color: var(--hone-ink-600);
          font-size: 12px;
        }
        .hone-hub-channel.is-bilibili {
          margin-top: 16px;
          padding-top: 16px;
          border-top: 1px solid var(--hone-line);
        }
        .hone-hub-bili-mark {
          width: 46px;
          height: 46px;
          flex: 0 0 46px;
          display: grid;
          place-items: center;
          border-radius: 50%;
          background: #e9f4fd;
          color: #1a9ad6;
          font-family: var(--hone-font-label);
          font-size: 21px;
          font-weight: 800;
        }
        .hone-hub-ghost-btn {
          flex: 0 0 auto;
          display: inline-flex;
          align-items: center;
          min-height: 36px;
          padding: 0 14px;
          border: 1px solid var(--hone-line-strong);
          border-radius: 10px;
          background: #fff;
          color: var(--hone-ink-950);
          cursor: pointer;
          font-size: 12px;
          font-weight: 700;
          white-space: nowrap;
          text-decoration: none;
          transition: border-color 0.16s ease;
        }
        .hone-hub-ghost-btn:hover { border-color: var(--hone-ink-950); }
        .hone-hub-solid-btn {
          display: inline-flex;
          align-items: center;
          min-height: 36px;
          padding: 0 15px;
          border: 1px solid var(--hone-ink-950);
          border-radius: 10px;
          background: var(--hone-ink-950);
          color: #fff;
          cursor: pointer;
          font-size: 12px;
          font-weight: 700;
          white-space: nowrap;
          text-decoration: none;
        }
        .hone-hub-solid-btn:hover { opacity: 0.9; }

        /* 近期视频 */
        .hone-hub-videos-label {
          margin: 16px 0 10px;
          color: var(--hone-ink-400, #9aa09b);
          font-size: 11px;
          font-weight: 700;
        }
        .hone-hub-videos {
          display: grid;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          gap: 12px;
        }
        .hone-hub-video {
          display: block;
          color: var(--hone-ink-950);
          text-decoration: none;
        }
        .hone-hub-video-thumb {
          display: block;
          position: relative;
          overflow: hidden;
          border: 1px solid var(--hone-line);
          border-radius: 12px;
          background: var(--hone-paper-100);
        }
        .hone-hub-video-thumb img {
          width: 100%;
          aspect-ratio: 16 / 9;
          display: block;
          object-fit: cover;
          transition: transform 0.2s var(--hone-ease);
        }
        .hone-hub-video:hover .hone-hub-video-thumb img { transform: scale(1.04); }
        .hone-hub-video-thumb i {
          position: absolute;
          right: 8px;
          bottom: 8px;
          display: grid;
          place-items: center;
          width: 26px;
          height: 26px;
          border-radius: 50%;
          background: rgba(16, 18, 17, 0.78);
          color: #fff;
          font-size: 10px;
          font-style: normal;
        }
        .hone-hub-video strong {
          display: -webkit-box;
          -webkit-box-orient: vertical;
          -webkit-line-clamp: 2;
          overflow: hidden;
          margin-top: 8px;
          font-size: 12px;
          font-weight: 650;
          line-height: 1.5;
          letter-spacing: -0.01em;
        }

        /* HONE 工具卡 */
        .hone-hub-product { display: flex; gap: 15px; }
        .hone-hub-product-mark {
          width: 46px;
          height: 46px;
          flex: 0 0 46px;
          display: grid;
          place-items: center;
          border-radius: 13px;
          background: #171917;
          color: #fff;
          font-size: 19px;
          font-weight: 800;
        }
        .hone-hub-product strong {
          color: var(--hone-ink-950);
          font-size: 15px;
          font-weight: 750;
          letter-spacing: -0.01em;
        }
        .hone-hub-product p {
          margin: 6px 0 0;
          color: var(--hone-ink-600);
          font-size: 12px;
          line-height: 1.7;
        }
        .hone-hub-product-actions {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
          margin-top: 12px;
        }

        /* ── 右栏会员卡 ── */
        .hone-hub-rail { position: sticky; top: 96px; }
        .hone-hub-member {
          border-color: color-mix(in srgb, var(--hone-coral-500) 32%, var(--hone-line));
          background:
            radial-gradient(420px 200px at 92% 0, color-mix(in srgb, var(--hone-coral-500) 9%, transparent), transparent 70%),
            #fff;
        }
        .hone-hub-member-head {
          display: flex;
          align-items: baseline;
          justify-content: space-between;
          gap: 12px;
        }
        .hone-hub-member-head strong {
          color: var(--hone-ink-950);
          font-size: 17px;
          font-weight: 750;
          letter-spacing: -0.02em;
        }
        .hone-hub-price { display: flex; align-items: baseline; gap: 3px; white-space: nowrap; }
        .hone-hub-price b {
          color: var(--hone-ink-950);
          font-size: 24px;
          font-weight: 800;
          letter-spacing: -0.03em;
          font-variant-numeric: tabular-nums;
        }
        .hone-hub-price span { color: var(--hone-ink-600); font-size: 12px; }
        .hone-share-promos {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
          margin-top: 10px;
        }
        .hone-share-promos span {
          padding: 3px 10px;
          border-radius: 999px;
          border: 1px solid color-mix(in srgb, var(--hone-coral-500) 36%, transparent);
          background: color-mix(in srgb, var(--hone-coral-500) 8%, #fff);
          color: var(--hone-coral-600);
          font-size: 11px;
          font-weight: 700;
          white-space: nowrap;
        }
        .hone-hub-benefits {
          display: grid;
          gap: 11px;
          margin: 16px 0 0;
          padding: 16px 0 0;
          border-top: 1px solid var(--hone-line);
          list-style: none;
        }
        .hone-hub-benefits li { display: flex; gap: 10px; }
        .hone-hub-benefits svg {
          width: 15px;
          height: 15px;
          flex: 0 0 15px;
          margin-top: 2px;
          color: var(--hone-coral-600);
        }
        .hone-hub-benefits strong {
          display: block;
          color: var(--hone-ink-950);
          font-size: 13px;
          font-weight: 700;
        }
        .hone-hub-benefits small {
          display: block;
          margin-top: 2px;
          color: var(--hone-ink-600);
          font-size: 11px;
          line-height: 1.55;
        }
        .hone-share-buy {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          gap: 10px;
          min-height: 46px;
          padding: 0 20px;
          border: 1px solid var(--hone-coral-500);
          border-radius: var(--hone-radius-sm);
          background: var(--hone-coral-500);
          color: #fff;
          cursor: pointer;
          font-size: 14px;
          font-weight: 700;
          white-space: nowrap;
          transition: background 0.16s ease, border-color 0.16s ease, transform 0.14s var(--hone-ease), box-shadow 0.16s ease;
          box-shadow: 0 8px 22px color-mix(in srgb, var(--hone-coral-500) 32%, transparent);
        }
        .hone-share-buy:hover {
          border-color: var(--hone-coral-600);
          background: var(--hone-coral-600);
          transform: translateY(-1px);
        }
        .hone-share-buy b {
          padding: 3px 10px;
          border-radius: 999px;
          background: rgba(255, 255, 255, 0.22);
          font-size: 12px;
          font-weight: 700;
          font-variant-numeric: tabular-nums;
        }
        .hone-hub-buy { width: 100%; margin-top: 16px; }
        .hone-hub-trial-btn {
          width: 100%;
          min-height: 40px;
          margin-top: 9px;
          border: 1px solid var(--hone-line-strong);
          border-radius: var(--hone-radius-sm);
          background: #fff;
          color: var(--hone-ink-950);
          cursor: pointer;
          font-size: 13px;
          font-weight: 700;
          transition: border-color 0.16s ease;
        }
        .hone-hub-trial-btn:hover { border-color: var(--hone-ink-950); }
        .hone-hub-trial-hint {
          margin: 9px 0 0;
          color: var(--hone-ink-600);
          font-size: 11px;
          line-height: 1.6;
        }
        .hone-hub-foot {
          margin: 12px 0 0;
          padding-top: 12px;
          border-top: 1px solid var(--hone-line);
          color: var(--hone-ink-400, #a3a6a1);
          font-size: 10px;
          line-height: 1.6;
        }
        .hone-share-service {
          display: inline-flex;
          align-items: center;
          min-height: 46px;
          padding: 0 16px;
          border: 1px solid var(--hone-line-strong);
          border-radius: var(--hone-radius-sm);
          background: #fff;
          color: var(--hone-ink-950);
          cursor: pointer;
          font-size: 13px;
          font-weight: 700;
          white-space: nowrap;
          transition: border-color 0.16s ease;
        }
        .hone-share-service:hover { border-color: var(--hone-ink-950); }

        /* ── 海报横滑条 ── */
        .hone-hub-posters { margin-top: 34px; }
        .hone-share-wall {
          display: grid;
          grid-auto-flow: column;
          grid-auto-columns: 196px;
          gap: 12px;
          overflow-x: auto;
          padding-bottom: 8px;
          scrollbar-width: thin;
        }
        .hone-share-poster {
          padding: 0;
          border: 1px solid var(--hone-line);
          border-radius: 14px;
          background: var(--hone-paper-100);
          overflow: hidden;
          cursor: zoom-in;
          transition: transform 0.18s var(--hone-ease), box-shadow 0.18s var(--hone-ease), border-color 0.18s ease;
        }
        .hone-share-poster:hover {
          transform: translateY(-3px);
          border-color: color-mix(in srgb, var(--hone-coral-500) 36%, var(--hone-line));
          box-shadow: var(--hone-shadow-md);
        }
        .hone-share-poster img {
          width: 100%;
          aspect-ratio: 1052 / 1870;
          height: auto;
          display: block;
          object-fit: cover;
        }

        .hone-share-disclaimer {
          margin: 22px 4px 0;
          color: var(--hone-ink-400, #a3a6a1);
          font-size: 11px;
          line-height: 1.7;
          text-align: center;
        }

        /* 移动端吸附购买栏（桌面隐藏） */
        .hone-share-dock { display: none; }

        /* 放大层：海报按屏高展示，二维码可长按/右键保存 */
        .hone-share-pop {
          position: fixed;
          inset: 0;
          z-index: 1200;
          display: grid;
          place-items: center;
          padding: 16px;
          background: rgba(23, 32, 31, 0.5);
          backdrop-filter: blur(8px);
          -webkit-backdrop-filter: blur(8px);
          animation: hone-share-fade 160ms ease both;
          cursor: zoom-out;
        }
        .hone-share-pop figure {
          display: flex;
          flex-direction: column;
          max-width: min(480px, 100%);
          max-height: calc(100dvh - 32px);
          margin: 0;
          padding: 14px 14px 12px;
          border: 1px solid var(--hone-line);
          border-radius: 18px;
          background: var(--hone-paper-50);
          box-shadow: 0 40px 110px rgba(23, 32, 31, 0.32);
          animation: hone-share-rise 200ms var(--hone-ease) both;
          cursor: default;
        }
        .hone-share-pop figcaption {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 10px;
          margin-bottom: 11px;
        }
        .hone-share-pop figcaption strong {
          color: var(--hone-ink-950);
          font-size: 14px;
          font-weight: 700;
        }
        .hone-share-pop figcaption button {
          width: 30px;
          height: 30px;
          display: grid;
          place-items: center;
          border: 1px solid var(--hone-line);
          border-radius: 50%;
          background: #fff;
          color: var(--hone-ink-600);
          cursor: pointer;
          font-size: 16px;
          line-height: 1;
        }
        .hone-share-pop img {
          min-height: 0;
          width: 100%;
          height: auto;
          max-height: calc(100dvh - 140px);
          border: 1px solid var(--hone-line);
          border-radius: 12px;
          background: #fff;
          object-fit: contain;
          -webkit-touch-callout: default;
          -webkit-user-select: auto;
          user-select: auto;
        }
        .hone-share-pop small {
          display: block;
          margin-top: 9px;
          color: var(--hone-ink-600);
          font-size: 11px;
          line-height: 1.55;
          text-align: center;
        }
        @keyframes hone-share-fade { from { opacity: 0; } to { opacity: 1; } }
        @keyframes hone-share-rise {
          from { opacity: 0; transform: translateY(12px) scale(0.985); }
          to { opacity: 1; transform: translateY(0) scale(1); }
        }

        /* ── 移动端：单列 + 吸附购买栏 ── */
        @media (max-width: 900px) {
          .hone-share-main { width: calc(100% - 32px); padding: 96px 0 24px; }
          .hone-hub-hero { gap: 14px; }
          .hone-hub-avatar { width: 68px; height: 68px; flex-basis: 68px; border-radius: 19px; }
          .hone-hub-id h1 { font-size: 24px; }
          .hone-hub-id > p { font-size: 12px; }
          .hone-hub-stats { gap: 6px; }
          .hone-hub-stats > div { padding: 7px 10px; }
          .hone-hub-stats strong { font-size: 14px; }
          .hone-hub-grid { grid-template-columns: 1fr; gap: 12px; margin-top: 20px; }
          .hone-hub-rail { position: static; }
          .hone-hub-card { padding: 17px 16px; border-radius: 15px; }
          .hone-hub-videos { grid-template-columns: repeat(3, 200px); max-width: 100%; overflow-x: auto; padding-bottom: 4px; scrollbar-width: none; }
          .hone-hub-videos::-webkit-scrollbar { display: none; }
          .hone-hub-channel .hone-hub-ghost-btn { padding: 0 11px; font-size: 11px; }
          .hone-share-wall { grid-auto-columns: 160px; }
          .hone-hub-posters { margin-top: 24px; }

          .hone-share-dock {
            position: fixed;
            right: 0;
            bottom: 0;
            left: 0;
            z-index: 190;
            display: flex;
            gap: 9px;
            padding: 10px max(14px, env(safe-area-inset-right)) calc(10px + env(safe-area-inset-bottom)) max(14px, env(safe-area-inset-left));
            border-top: 1px solid var(--hone-line);
            background: color-mix(in srgb, var(--hone-paper-50) 96%, transparent);
            backdrop-filter: blur(16px);
            -webkit-backdrop-filter: blur(16px);
          }
          .hone-share-dock .hone-share-buy { flex: 1; justify-content: center; min-height: 48px; }
          .hone-share-dock .hone-share-service { flex: 0 0 auto; min-height: 48px; }

          /* 吸附栏替代了底部 tabs 的空间占位 */
          .hone-share-page { padding-bottom: 0 !important; }
          .hone-share-page .pub-footer { padding-bottom: 92px; }
        }
      `}</style>
    </div>
  )
}
