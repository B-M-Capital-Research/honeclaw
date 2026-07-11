import { Markdown } from "@hone-financial/ui/markdown";
import { For, Show } from "solid-js";
import { useLocale } from "@/lib/i18n";
import type { PublicPushDetail, PublicPushListItem } from "@/lib/types";

export type ScheduledPushCardData = {
  pushId?: string;
  title: string;
  summary: string;
  fallbackContent?: string;
  createdAt?: string;
};

function PushGlyph() {
  return (
    <svg
      viewBox="0 0 24 24"
      width="20"
      height="20"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <path d="M18 8a6 6 0 0 0-12 0c0 7-3 7-3 9h18c0-2-3-2-3-9" />
      <path d="M10 21h4" />
    </svg>
  );
}

export function PushUnreadDot(props: { count: number }) {
  return (
    <Show when={props.count > 0}>
      <span
        class="public-push-unread-dot"
        aria-label={
          useLocale() === "zh"
            ? `${props.count} 条新推送`
            : `${props.count} new pushes`
        }
      />
    </Show>
  );
}

export function ScheduledPushCard(props: {
  push: ScheduledPushCardData;
  onOpen: (push: ScheduledPushCardData) => void;
}) {
  const copy = () => pushCopy(useLocale());
  return (
    <div class="pub-msg-in pub-msg-row public-scheduled-card-row">
      <button
        type="button"
        class="public-scheduled-card"
        onClick={() => props.onOpen(props.push)}
      >
        <span class="public-scheduled-card-accent" />
        <span class="public-scheduled-card-topline">
          <span class="public-scheduled-card-icon">
            <PushGlyph />
          </span>
          <span>{copy().brief}</span>
          <Show when={props.push.createdAt}>
            <time>{formatPushTime(props.push.createdAt!, useLocale())}</time>
          </Show>
        </span>
        <strong>{props.push.title}</strong>
        <span class="public-scheduled-card-summary">{props.push.summary}</span>
        <span class="public-scheduled-card-action">
          {copy().viewFull}
          <span aria-hidden="true">→</span>
        </span>
      </button>
    </div>
  );
}

export function PublicPushCenter(props: {
  open: boolean;
  items: PublicPushListItem[];
  loading: boolean;
  loadingMore: boolean;
  error?: string;
  nextBefore?: string;
  onClose: () => void;
  onOpenPush: (item: PublicPushListItem) => void;
  onLoadMore: () => void;
}) {
  const copy = () => pushCopy(useLocale());
  return (
    <Show when={props.open}>
      <div class="public-push-center-backdrop" onClick={props.onClose}>
        <section
          class="public-push-center"
          role="dialog"
          aria-modal="true"
          aria-label={copy().listAria}
          onClick={(event) => event.stopPropagation()}
        >
          <header>
            <div>
              <span class="public-push-center-kicker">HONE Dispatch</span>
              <h2>{copy().centerTitle}</h2>
              <p>{copy().centerIntro}</p>
            </div>
            <button type="button" aria-label={copy().closeCenter} onClick={props.onClose}>
              ×
            </button>
          </header>

          <div class="public-push-center-list">
            <Show when={props.loading && props.items.length === 0}>
              <div class="public-push-center-state">{copy().loadingList}</div>
            </Show>
            <Show when={props.error}>
              <div class="public-push-center-state is-error">{props.error}</div>
            </Show>
            <Show when={!props.loading && !props.error && props.items.length === 0}>
              <div class="public-push-center-empty">
                <span class="public-push-center-empty-icon"><PushGlyph /></span>
                <strong>{copy().emptyTitle}</strong>
                <p>{copy().emptyBody}</p>
              </div>
            </Show>
            <For each={props.items}>
              {(item, index) => (
                <button
                  type="button"
                  class="public-push-list-item"
                  onClick={() => props.onOpenPush(item)}
                >
                  <span class="public-push-list-index">
                    {String(index() + 1).padStart(2, "0")}
                  </span>
                  <span class="public-push-list-copy">
                    <span class="public-push-list-meta">
                      <span>{copy().brief}</span>
                      <time>{formatPushTime(item.created_at, useLocale())}</time>
                    </span>
                    <strong>{item.title}</strong>
                    <span>{item.summary}</span>
                  </span>
                  <span class="public-push-list-arrow" aria-hidden="true">↗</span>
                </button>
              )}
            </For>
            <Show when={props.nextBefore}>
              <button
                type="button"
                class="public-push-load-more"
                disabled={props.loadingMore}
                onClick={props.onLoadMore}
              >
                {props.loadingMore ? copy().loadingMore : copy().loadMore}
              </button>
            </Show>
          </div>
        </section>
      </div>
    </Show>
  );
}

export function PublicPushDetailDialog(props: {
  open: boolean;
  detail?: PublicPushDetail;
  loading: boolean;
  error?: string;
  onClose: () => void;
}) {
  const copy = () => pushCopy(useLocale());
  return (
    <Show when={props.open}>
      <div class="public-push-detail-backdrop" onClick={props.onClose}>
        <article
          class="public-push-detail"
          role="dialog"
          aria-modal="true"
          aria-label={copy().detailAria}
          onClick={(event) => event.stopPropagation()}
        >
          <header>
            <div class="public-push-detail-mark"><PushGlyph /></div>
            <div>
              <span>{copy().brief}</span>
              <h2>{props.detail?.title ?? copy().opening}</h2>
              <Show when={props.detail?.created_at}>
                <time>{formatPushTime(props.detail!.created_at, useLocale())}</time>
              </Show>
            </div>
            <button type="button" aria-label={copy().closeDetail} onClick={props.onClose}>×</button>
          </header>
          <div class="public-push-detail-body">
            <Show when={props.loading}>
              <div class="public-push-center-state">{copy().loadingDetail}</div>
            </Show>
            <Show when={props.error}>
              <div class="public-push-center-state is-error">{props.error}</div>
            </Show>
            <Show when={props.detail && !props.loading && !props.error}>
              <Markdown
                text={props.detail!.content}
                class="public-chat-markdown public-push-detail-markdown"
              />
            </Show>
          </div>
        </article>
      </div>
    </Show>
  );
}

export function PushNavIcon() {
  return <PushGlyph />;
}

function formatPushTime(value: string, locale: "zh" | "en"): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(locale === "zh" ? "zh-CN" : "en-US", {
    month: "numeric",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
}

function pushCopy(locale: "zh" | "en") {
  if (locale === "en") {
    return {
      brief: "Scheduled brief",
      viewFull: "View full update",
      listAria: "Push list",
      centerTitle: "Push center",
      centerIntro: "Every scheduled brief, quietly archived in one place.",
      closeCenter: "Close push center",
      loadingList: "Organizing pushes…",
      emptyTitle: "No scheduled pushes yet",
      emptyBody: "New briefs will appear here automatically.",
      loadingMore: "Loading…",
      loadMore: "View more pushes",
      detailAria: "Full push content",
      opening: "Opening push",
      closeDetail: "Close full content",
      loadingDetail: "Loading full content…",
    };
  }
  return {
    brief: "定时简报",
    viewFull: "查看完整内容",
    listAria: "推送列表",
    centerTitle: "推送中心",
    centerIntro: "所有定时简报，在一个地方安静归档。",
    closeCenter: "关闭推送中心",
    loadingList: "正在整理推送…",
    emptyTitle: "还没有定时推送",
    emptyBody: "任务产生新简报后，会自动收进这里。",
    loadingMore: "加载中…",
    loadMore: "查看更多推送",
    detailAria: "推送完整内容",
    opening: "正在打开推送",
    closeDetail: "关闭完整内容",
    loadingDetail: "正在加载完整内容…",
  };
}
