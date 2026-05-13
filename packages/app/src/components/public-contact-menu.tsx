import { CONTENT } from "@/lib/public-content";
import { useLocale } from "@/lib/i18n";

export const PUBLIC_YOUTUBE_URL = "https://www.youtube.com/@巴芒投研美股频道";
export const PUBLIC_BILIBILI_URL = "https://www.bilibili.com/video/BV1ByXNBGET5/";

export function contactMenuTitle() {
  return useLocale() === "zh" ? "联系我们" : "Contact us";
}

export function PublicContactCards() {
  const C = CONTENT.nav;

  return (
    <div class="pub-contact-card-grid">
      <a class="pub-contact-card pub-contact-card--bilibili" href={PUBLIC_BILIBILI_URL} target="_blank" rel="noopener noreferrer">
        <span class="pub-contact-card-icon">B</span>
        <span>
          <strong>B站</strong>
          <small>Bilibili</small>
        </span>
      </a>
      <a class="pub-contact-card pub-contact-card--youtube" href={PUBLIC_YOUTUBE_URL} target="_blank" rel="noopener noreferrer">
        <span class="pub-contact-card-icon">Y</span>
        <span>
          <strong>YouTube</strong>
          <small>巴芒投研美股频道</small>
        </span>
      </a>
      <a class="pub-contact-card" href={`mailto:${C.contact_email}`}>
        <span class="pub-contact-card-icon">@</span>
        <span>
          <strong>{C.contact_email_label}</strong>
          <small>{C.contact_email}</small>
        </span>
      </a>
      <div class="pub-contact-card">
        <span class="pub-contact-card-icon">微</span>
        <span>
          <strong>{useLocale() === "zh" ? "微信社群" : "WeChat community"}</strong>
          <small>联系 {C.contact_wechat}</small>
        </span>
      </div>
    </div>
  );
}

export function PublicContactMenu() {
  const C = CONTENT.nav;

  return (
    <div class="pub-contact-menu">
      <button
        type="button"
        class="header-contact-link pub-contact-trigger"
        title={`${C.contact_email_label}: ${C.contact_email} / ${C.contact_wechat_label}: ${C.contact_wechat}`}
        aria-label={contactMenuTitle()}
      >
        <span class="header-contact-text">{contactMenuTitle()}</span>
      </button>
      <div class="pub-contact-popover">
        <div class="pub-contact-popover-title">{contactMenuTitle()}</div>
        <PublicContactCards />
      </div>
    </div>
  );
}
