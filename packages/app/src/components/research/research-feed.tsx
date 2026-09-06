import { For, Show, type JSX } from "solid-js";
import "./research-feed.css";

/**
 * A timeline of posts, read the way a social feed is read.
 *
 * The research panels had drifted into wrapping every post in product chrome:
 * a restated title, a model summary above the post, the post itself folded
 * away, then stance chips, index terms and a caveat box. Six blocks of
 * scaffolding around one piece of writing — so the writing stopped being what
 * you saw.
 *
 * Here the post is the content. Author, time and the text itself carry the
 * item; anything HONE derived from it collapses behind a single line that
 * opens only if the reader asks. Nothing is removed — it stops competing.
 *
 * The item can also carry the small furniture a post timeline is expected to
 * have: an avatar at the left edge, a kind label beside the name, reach
 * numbers under the text, and one or two actions. Every one of them is
 * optional, so a feed of filings or forum threads renders exactly as before.
 */
export function ResearchFeed(props: { children: JSX.Element }) {
  return <div class="research-feed">{props.children}</div>;
}

/** A dated divider between groups of posts: 今天 / 昨天 / MM-DD. */
export function ResearchFeedDay(props: { label: string; count?: number }) {
  return (
    <div class="research-feed-day" role="separator" aria-label={props.label}>
      <span>{props.label}</span>
      <Show when={props.count}>
        <small>{props.count} 条</small>
      </Show>
    </div>
  );
}

export function ResearchFeedItem(props: {
  author: string;
  /** `@handle`, source name — whatever identifies the writer. */
  handle?: string;
  /** Already-shortened local timestamp. */
  time?: string;
  /** Small facts about the post: kind, reach. Rendered as one quiet line. */
  meta?: string[];
  /**
   * One or two characters drawn in a circle at the left edge, the way a
   * timeline shows who is speaking without loading anyone's picture.
   */
  avatar?: string;
  /** A small label beside the name — 回复 / 引用 / 转推. */
  kind?: string;
  /** The post. This is the item's reason to exist, so it is never folded. */
  children: JSX.Element;
  /** The post being replied to or quoted, shown nested above the body. */
  quoted?: { label: string; text: string };
  media?: string[];
  mediaAlt?: string;
  /** Outbound links: original post, aggregation source. */
  links?: { href: string; label: string }[];
  /** Quiet numbers under the post: 阅读 11.1k · 赞 19. */
  stats?: string[];
  /** Buttons that act on the post, e.g. 问 HONE. */
  actions?: { label: string; onClick: () => void }[];
  /** Everything HONE inferred. Collapsed by default — it is commentary. */
  analysis?: JSX.Element;
  analysisLabel?: string;
  /** Left-edge accent, e.g. a verification or impact state. */
  accent?: string;
  /**
   * Controls that act on the post — like, comment, report, moderate.
   *
   * Interaction is not commentary, so it must not be folded into `analysis`;
   * it is also not part of the post, so it must not sit in `children` above the
   * pictures and the source links. It closes the item instead, the way a
   * social feed puts its action row under everything it is acting on.
   */
  footer?: JSX.Element;
}) {
  const hasBar = () =>
    !!(props.stats?.length || props.links?.length || props.actions?.length);
  return (
    <article
      class={`research-feed-item${props.accent ? ` is-${props.accent}` : ""}`}
      classList={{ "has-avatar": !!props.avatar }}
    >
      <Show when={props.avatar}>
        <span class="research-feed-item__avatar" aria-hidden="true">
          {props.avatar}
        </span>
      </Show>
      <div class="research-feed-item__main">
        <header class="research-feed-item__head">
          <b>{props.author}</b>
          <Show when={props.handle}>
            <span class="research-feed-item__handle">{props.handle}</span>
          </Show>
          <Show when={props.kind}>
            <span class="research-feed-item__kind">{props.kind}</span>
          </Show>
          <Show when={props.meta?.length}>
            <span class="research-feed-item__meta">{props.meta!.join(" · ")}</span>
          </Show>
          <Show when={props.time}>
            <time>{props.time}</time>
          </Show>
        </header>

        <Show when={props.quoted}>
          {(quoted) => (
            <blockquote class="research-feed-item__quoted">
              <cite>{quoted().label}</cite>
              <p>{quoted().text}</p>
            </blockquote>
          )}
        </Show>

        <div class="research-feed-item__body">{props.children}</div>

        <Show when={props.media?.length}>
          <div
            class="research-feed-item__media"
            classList={{ single: props.media!.length === 1 }}
          >
            <For each={props.media}>
              {(url) => (
                <a href={url} target="_blank" rel="noreferrer">
                  {/* Deliberately not `loading="lazy"`. The panel pins the page
                      with `body { position: fixed }` for its iOS-safe scroll
                      lock, and Chrome then stops resolving viewport intersection
                      for images inside this portal — they sit visible and never
                      request. */}
                  <img
                    src={url}
                    alt={props.mediaAlt ?? "原文配图"}
                    decoding="async"
                    referrerpolicy="no-referrer"
                  />
                </a>
              )}
            </For>
          </div>
        </Show>

        <Show when={props.analysis}>
          <details class="research-feed-item__analysis">
            <summary>{props.analysisLabel ?? "HONE 解读"}</summary>
            <div>{props.analysis}</div>
          </details>
        </Show>

        {/* Reach, links and actions share one quiet row under the post, the
            way a timeline keeps its numbers and buttons beneath what they
            belong to rather than scattering them through the text. */}
        <Show when={hasBar()}>
          <div class="research-feed-item__bar">
            <Show when={props.stats?.length}>
              <span class="research-feed-item__stats">{props.stats!.join(" · ")}</span>
            </Show>
            <Show when={props.links?.length}>
              <div class="research-feed-item__links">
                <For each={props.links}>
                  {(link) => (
                    <a href={link.href} target="_blank" rel="noreferrer">
                      {link.label}
                      <i aria-hidden="true">↗</i>
                    </a>
                  )}
                </For>
              </div>
            </Show>
            <Show when={props.actions?.length}>
              <div class="research-feed-item__actions">
                <For each={props.actions}>
                  {(action) => (
                    <button type="button" onClick={() => action.onClick()}>
                      {action.label}
                    </button>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </Show>

        {props.footer}
      </div>
    </article>
  );
}
