// public-plan.tsx — 会员分享海报页：六张 9:16 竖版海报（bm1 邀请函 → bm6 社群
// 好评）+ 购买/客服动作。别人问“哪里买会员”时直接丢这个链接，手机 PC 通用。
// 中文「立即购买」弹知识星球二维码（可长按/右键保存），英文跳 Whop。

import { createSignal, For, Show, onCleanup, createEffect } from "solid-js"
import { CONTENT } from "@/lib/public-content"
import { useLocale } from "@/lib/i18n"
import { PublicFooter, PublicNav } from "@/components/public-nav"
import "./public-site.css"

const WHOP_URL = "https://whop.com/edda1183-b297-4502-811f-339ae5e773be/vip-copy-18/"

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
        <header class="hone-share-head">
          <div class="hone-share-eyebrow">{C().eyebrow}</div>
          <h1>{C().share_title}</h1>
          <p>{C().share_sub}</p>
          <div class="hone-share-actions">
            <button type="button" class="hone-share-buy" onClick={buy}>
              <span>{C().full.cta}</span>
              <b>{C().full.price}{C().full.period}</b>
            </button>
            <button type="button" class="hone-share-service" onClick={() => setLightbox({ kind: "support" })}>
              {C().support.title}
            </button>
          </div>
          <Show when={C().full.promos.length > 0}>
            <div class="hone-share-promos">
              <For each={C().full.promos}>{(promo) => <span>{promo}</span>}</For>
            </div>
          </Show>
        </header>

        <section class="hone-share-wall" aria-label={C().share_title}>
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
        </section>

        <aside class="hone-share-bottom">
          <div>
            <strong>{C().share_title}</strong>
            <p>{C().foot}</p>
          </div>
          <button type="button" class="hone-share-buy" onClick={buy}>
            <span>{C().full.cta}</span>
            <b>{C().full.price}{C().full.period}</b>
          </button>
        </aside>
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
          padding: 128px 0 88px;
          flex: 1;
        }
        .hone-share-head { max-width: 720px; }
        .hone-share-eyebrow {
          color: var(--hone-coral-600);
          font-family: var(--hone-font-label);
          font-size: 11px;
          font-weight: 700;
          letter-spacing: 0.16em;
        }
        .hone-share-head h1 {
          margin: 14px 0 0;
          color: var(--hone-ink-950);
          font-size: clamp(30px, 4vw, 42px);
          line-height: 1.08;
          letter-spacing: -0.04em;
        }
        .hone-share-head > p {
          margin: 14px 0 0;
          color: var(--hone-ink-600);
          font-size: 14px;
          line-height: 1.75;
        }
        .hone-share-actions {
          display: flex;
          align-items: center;
          gap: 10px;
          flex-wrap: wrap;
          margin-top: 22px;
        }
        .hone-share-buy {
          display: inline-flex;
          align-items: center;
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
        .hone-share-promos {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
          margin-top: 12px;
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

        /* 海报墙：桌面 3 列（9:16 原比例），点击放大 */
        .hone-share-wall {
          display: grid;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          gap: 14px;
          margin-top: 34px;
        }
        .hone-share-poster {
          padding: 0;
          border: 1px solid var(--hone-line);
          border-radius: 15px;
          background: var(--hone-paper-100);
          overflow: hidden;
          cursor: zoom-in;
          transition: transform 0.18s var(--hone-ease), box-shadow 0.18s var(--hone-ease), border-color 0.18s ease;
        }
        .hone-share-poster:hover {
          transform: translateY(-4px);
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

        /* 底部再购卡 */
        .hone-share-bottom {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 22px;
          margin-top: 26px;
          padding: 22px 24px;
          border: 1px solid color-mix(in srgb, var(--hone-coral-500) 32%, var(--hone-line));
          border-radius: 17px;
          background:
            radial-gradient(420px 200px at 92% 0, color-mix(in srgb, var(--hone-coral-500) 9%, transparent), transparent 70%),
            #fff;
        }
        .hone-share-bottom strong {
          color: var(--hone-ink-950);
          font-size: 16px;
          font-weight: 700;
          letter-spacing: -0.02em;
        }
        .hone-share-bottom p {
          margin: 7px 0 0;
          color: var(--hone-ink-600);
          font-size: 11px;
          line-height: 1.6;
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

        /* ── 移动端：海报全宽串读 + 吸附购买栏 ── */
        @media (max-width: 900px) {
          .hone-share-main { width: calc(100% - 32px); padding: 96px 0 24px; }
          .hone-share-head > p { font-size: 13px; }
          .hone-share-actions { display: none; }
          .hone-share-promos { margin-top: 16px; }
          .hone-share-wall { grid-template-columns: 1fr; gap: 14px; margin-top: 20px; }
          .hone-share-poster { border-radius: 16px; }
          .hone-share-bottom { flex-direction: column; align-items: flex-start; gap: 14px; margin-top: 20px; padding: 18px 16px; }
          .hone-share-bottom .hone-share-buy { width: 100%; justify-content: center; }

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
