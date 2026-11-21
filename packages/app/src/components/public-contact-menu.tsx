import { CONTENT } from "@/lib/public-content";

export const PUBLIC_YOUTUBE_URL = "https://www.youtube.com/@BamangTech";
export const PUBLIC_BILIBILI_URL = "https://space.bilibili.com/224670487?spm_id_from=333.337.0.0";

function contactMenuTitle() {
  return CONTENT.nav.contact_title;
}

export function PublicContactCards() {
  const C = CONTENT.nav;

  return (
    <div class="pub-contact-card-grid">
      <a class="pub-contact-card pub-contact-card--bilibili" href={PUBLIC_BILIBILI_URL} target="_blank" rel="noopener noreferrer">
        <span class="pub-contact-card-icon">B</span>
        <span>
          <strong>{C.bilibili_label}</strong>
          <small>{C.bilibili_channel_name}</small>
        </span>
      </a>
      <a class="pub-contact-card pub-contact-card--youtube" href={PUBLIC_YOUTUBE_URL} target="_blank" rel="noopener noreferrer">
        <span class="pub-contact-card-icon">Y</span>
        <span>
          <strong>YouTube</strong>
          <small>{C.youtube_channel_name}</small>
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
        <span class="pub-contact-card-icon">{CONTENT.chat_page.misc.contact_wechat}</span>
        <span>
          <strong>{C.contact_wechat_group}</strong>
          <small>
            {C.contact_wechat_hint_prefix} {C.contact_wechat}
          </small>
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
