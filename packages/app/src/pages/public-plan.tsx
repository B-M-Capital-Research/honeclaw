// public-plan.tsx — 定价页：开源免费版（GitHub 自托管，注明自备模型/自配渠道等）
// + 完整服务（中文 ¥1299/年，购买弹出知识星球二维码可长按/右键保存；
// 英文 $199.99/年，跳转 Whop 购买）。两种语言都提供企业微信客服指引。

import { createSignal, For, Show, onCleanup, createEffect } from "solid-js"
import { CONTENT } from "@/lib/public-content"
import { useLocale } from "@/lib/i18n"
import { PublicFooter, PublicNav } from "@/components/public-nav"
import "./public-site.css"

const WHOP_URL = "https://whop.com/edda1183-b297-4502-811f-339ae5e773be/vip-copy-18/"

/* 二维码放大层：点击图片放大展示，长按 / 右键即可保存原图。 */
function QrLightbox(props: {
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
      <div class="hone-plan-qr-backdrop" onClick={props.onClose}>
        <figure class="hone-plan-qr-pop" onClick={(event) => event.stopPropagation()}>
          <figcaption>
            <strong>{props.title}</strong>
            <button type="button" aria-label={CONTENT.plan.close_aria} onClick={props.onClose}>×</button>
          </figcaption>
          <img src={props.src} alt={props.title} />
          <small>{props.hint}</small>
        </figure>
      </div>
    </Show>
  )
}

export default function PublicPlanPage() {
  const C = () => CONTENT.plan
  const nav = () => CONTENT.nav
  const [qrOpen, setQrOpen] = createSignal<"join" | "support" | null>(null)
  const isZh = () => useLocale() === "zh"

  return (
    <div class="pub-page hone-plan-page">
      <PublicNav />
      <main class="hone-plan-main">
        <header class="hone-plan-hero">
          <div class="hone-plan-eyebrow">{C().eyebrow}</div>
          <h1>{C().title}</h1>
          <p>{C().sub}</p>
        </header>

        <section class="hone-plan-grid" aria-label={C().eyebrow}>
          {/* 开源免费版 */}
          <article class="hone-plan-card">
            <div class="hone-plan-card-head">
              <h2>{C().free.name}</h2>
            </div>
            <div class="hone-plan-price">
              <strong>{C().free.price}</strong>
              <small>{C().free.period}</small>
            </div>
            <p class="hone-plan-desc">{C().free.desc}</p>
            <div class="hone-plan-notes">
              <span>{C().free.notes_label}</span>
              <ul>
                <For each={C().free.notes}>
                  {(note) => (
                    <li>
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" aria-hidden="true"><path d="M12 8v5M12 16.6v.4" /><circle cx="12" cy="12" r="9" /></svg>
                      <span>{note}</span>
                    </li>
                  )}
                </For>
              </ul>
            </div>
            <a class="hone-plan-cta" href={nav().github_url} target="_blank" rel="noopener noreferrer">
              {C().free.cta}
            </a>
          </article>

          {/* 完整服务 */}
          <article class="hone-plan-card is-featured">
            <div class="hone-plan-card-head">
              <h2>{C().full.name}</h2>
              <span class="hone-plan-badge is-soon">{C().full.badge}</span>
            </div>
            <div class="hone-plan-price">
              <strong>{C().full.price}</strong>
              <small>{C().full.period}</small>
            </div>
            <Show when={C().full.promos.length > 0}>
              <div class="hone-plan-promos">
                <For each={C().full.promos}>{(promo) => <span>{promo}</span>}</For>
              </div>
            </Show>
            <p class="hone-plan-desc">{C().full.desc}</p>
            <ul class="hone-plan-features">
              <For each={C().full.features}>
                {(feature) => (
                  <li>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 6 9 17l-5-5" /></svg>
                    <span>{feature}</span>
                  </li>
                )}
              </For>
            </ul>
            <Show
              when={isZh()}
              fallback={
                <a class="hone-plan-cta is-primary is-buy" href={WHOP_URL} target="_blank" rel="noopener noreferrer">
                  {C().full.cta}
                </a>
              }
            >
              <button type="button" class="hone-plan-cta is-primary is-buy" onClick={() => setQrOpen("join")}>
                {C().full.cta}
              </button>
            </Show>
          </article>
        </section>

        {/* 客服指引：中英文都保留 */}
        <aside class="hone-plan-support">
          <button type="button" class="hone-plan-support-qr" onClick={() => setQrOpen("support")}>
            <img src="/membership_wechat.jpg" alt={C().support.title} loading="lazy" />
          </button>
          <div>
            <strong>{C().support.title}</strong>
            <p>{C().support.desc}</p>
          </div>
        </aside>

        <p class="hone-plan-foot">{C().foot}</p>
      </main>
      <PublicFooter />

      <QrLightbox
        open={qrOpen() === "join"}
        src="/membership_zsxq.jpg"
        title={C().full.qr_title}
        hint={C().full.qr_hint}
        onClose={() => setQrOpen(null)}
      />
      <QrLightbox
        open={qrOpen() === "support"}
        src="/membership_wechat.jpg"
        title={C().support.title}
        hint={C().support.desc}
        onClose={() => setQrOpen(null)}
      />

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
          width: min(920px, calc(100% - 40px));
          margin: 0 auto;
          padding: 132px 0 88px;
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
          grid-template-columns: minmax(0, 1fr) minmax(0, 1.1fr);
          gap: 14px;
          margin-top: 40px;
          align-items: stretch;
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
          font-size: 30px;
          font-weight: 800;
          letter-spacing: -0.03em;
          line-height: 1;
        }
        .hone-plan-price small {
          color: var(--hone-ink-400);
          font-size: 11px;
          font-weight: 650;
        }
        .hone-plan-promos {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
          margin-top: 12px;
        }
        .hone-plan-promos span {
          padding: 3px 10px;
          border-radius: 999px;
          background: var(--hone-coral-500);
          color: #fff;
          font-size: 11px;
          font-weight: 700;
          white-space: nowrap;
        }
        .hone-plan-desc {
          margin: 12px 0 0;
          color: var(--hone-ink-600);
          font-size: 12px;
          line-height: 1.6;
        }
        .hone-plan-features {
          display: grid;
          gap: 10px;
          margin: 18px 0 22px;
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
          line-height: 1.6;
        }
        .hone-plan-features svg {
          width: 13px;
          height: 13px;
          flex: 0 0 13px;
          margin-top: 2px;
          color: var(--hone-coral-500);
        }
        /* 免费版注意事项：与勾选权益区分的中性提示列表 */
        .hone-plan-notes {
          margin: 18px 0 22px;
          padding: 18px 0 0;
          border-top: 1px solid var(--hone-line);
          flex: 1;
        }
        .hone-plan-notes > span {
          color: var(--hone-ink-400);
          font-size: 10px;
          font-weight: 700;
          letter-spacing: 0.08em;
          text-transform: uppercase;
        }
        .hone-plan-notes ul {
          display: grid;
          gap: 9px;
          margin: 12px 0 0;
          padding: 0;
          list-style: none;
        }
        .hone-plan-notes li {
          display: flex;
          align-items: flex-start;
          gap: 9px;
          color: var(--hone-ink-600);
          font-size: 12px;
          line-height: 1.6;
        }
        .hone-plan-notes svg {
          width: 13px;
          height: 13px;
          flex: 0 0 13px;
          margin-top: 2px;
          color: var(--hone-ink-400);
        }
        .hone-plan-cta {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          min-height: 42px;
          padding: 0 16px;
          border: 1px solid var(--hone-line-strong);
          border-radius: var(--hone-radius-sm);
          background: #fff;
          color: var(--hone-ink-950);
          cursor: pointer;
          font-size: 13px;
          font-weight: 700;
          text-decoration: none;
          transition: background 0.16s ease, border-color 0.16s ease, color 0.16s ease, box-shadow 0.16s ease;
        }
        .hone-plan-cta:hover { border-color: var(--hone-ink-950); }
        .hone-plan-cta.is-primary {
          border-color: var(--hone-ink-950);
          background: var(--hone-ink-950);
          color: #fff;
        }
        .hone-plan-cta.is-primary:hover { background: var(--hone-ink-800); }
        .hone-plan-cta.is-buy {
          border-color: var(--hone-coral-500);
          background: var(--hone-coral-500);
          box-shadow: 0 8px 20px color-mix(in srgb, var(--hone-coral-500) 30%, transparent);
        }
        .hone-plan-cta.is-buy:hover {
          border-color: var(--hone-coral-600);
          background: var(--hone-coral-600);
        }

        /* 客服指引条 */
        .hone-plan-support {
          display: flex;
          align-items: center;
          gap: 18px;
          margin-top: 26px;
          padding: 18px 20px;
          border: 1px solid var(--hone-line);
          border-radius: 15px;
          background: #fff;
        }
        .hone-plan-support-qr {
          flex: 0 0 auto;
          padding: 0;
          border: 1px solid var(--hone-line);
          border-radius: 10px;
          background: #fff;
          overflow: hidden;
          cursor: zoom-in;
        }
        .hone-plan-support-qr img {
          width: 86px;
          height: 86px;
          display: block;
          object-fit: cover;
        }
        .hone-plan-support strong {
          color: var(--hone-ink-950);
          font-size: 14px;
          font-weight: 700;
        }
        .hone-plan-support p {
          margin: 6px 0 0;
          color: var(--hone-ink-600);
          font-size: 12px;
          line-height: 1.6;
        }
        .hone-plan-foot {
          margin: 16px 0 0;
          color: var(--hone-ink-400);
          font-size: 10px;
          line-height: 1.6;
        }

        /* 二维码放大层 */
        .hone-plan-qr-backdrop {
          position: fixed;
          inset: 0;
          z-index: 1200;
          display: grid;
          place-items: center;
          padding: 20px;
          background: rgba(23, 32, 31, 0.42);
          backdrop-filter: blur(6px);
          -webkit-backdrop-filter: blur(6px);
          animation: hone-plan-fade 160ms ease both;
        }
        .hone-plan-qr-pop {
          width: min(400px, 100%);
          margin: 0;
          padding: 16px 16px 14px;
          border: 1px solid var(--hone-line);
          border-radius: 18px;
          background: var(--hone-paper-50);
          box-shadow: 0 40px 110px rgba(23, 32, 31, 0.3);
          animation: hone-plan-pop 200ms var(--hone-ease) both;
        }
        .hone-plan-qr-pop figcaption {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 10px;
          margin-bottom: 12px;
        }
        .hone-plan-qr-pop figcaption strong {
          color: var(--hone-ink-950);
          font-size: 15px;
          font-weight: 700;
        }
        .hone-plan-qr-pop figcaption button {
          width: 30px;
          height: 30px;
          display: grid;
          place-items: center;
          border: 1px solid var(--hone-line);
          border-radius: 50%;
          background: #fff;
          color: var(--hone-ink-600);
          cursor: pointer;
          font-size: 17px;
          line-height: 1;
        }
        .hone-plan-qr-pop img {
          width: 100%;
          height: auto;
          border: 1px solid var(--hone-line);
          border-radius: 12px;
          background: #fff;
          -webkit-touch-callout: default;
          -webkit-user-select: auto;
          user-select: auto;
        }
        .hone-plan-qr-pop small {
          display: block;
          margin-top: 10px;
          color: var(--hone-ink-600);
          font-size: 11px;
          line-height: 1.55;
          text-align: center;
        }
        @keyframes hone-plan-fade { from { opacity: 0; } to { opacity: 1; } }
        @keyframes hone-plan-pop {
          from { opacity: 0; transform: translateY(12px) scale(0.985); }
          to { opacity: 1; transform: translateY(0) scale(1); }
        }

        @media (max-width: 900px) {
          .hone-plan-main { width: calc(100% - 36px); padding-top: 100px; }
          .hone-plan-hero p { font-size: 13px; }
          .hone-plan-grid { grid-template-columns: 1fr; gap: 12px; margin-top: 32px; }
          .hone-plan-card { padding: 22px 18px 20px; }
          .hone-plan-card.is-featured { order: -1; }
          .hone-plan-cta { min-height: 46px; }
          .hone-plan-support { gap: 14px; padding: 16px; }
          .hone-plan-support-qr img { width: 76px; height: 76px; }
          .hone-plan-qr-pop { width: min(360px, 100%); }
        }
      `}</style>
    </div>
  )
}
