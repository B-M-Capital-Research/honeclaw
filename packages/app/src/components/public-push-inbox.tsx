import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { PublicPushDetailDialog } from "@/components/public-push-center";
import { getPublicPushes, openPublicPush } from "@/lib/api";
import { mergePublicPushItems, latestUnreadPushId } from "@/lib/public-chat";
import { CONTENT } from "@/lib/public-content";
import {
  ALL_PUBLIC_PUSHES,
  filterPublicPushes,
  publicPushCategories,
} from "@/lib/public-push-inbox";
import { useLocale } from "@/lib/i18n";
import type { PublicPushDetail, PublicPushListItem } from "@/lib/types";

export function PublicPushInbox(props: {
  onUnreadCountChange: (count: number) => void;
}) {
  const [items, setItems] = createSignal<PublicPushListItem[]>([]);
  const [selectedCategory, setSelectedCategory] = createSignal<string>(ALL_PUBLIC_PUSHES);
  const [nextBefore, setNextBefore] = createSignal<string>();
  const [loading, setLoading] = createSignal(true);
  const [loadingMore, setLoadingMore] = createSignal(false);
  const [error, setError] = createSignal<string>();
  const [readError, setReadError] = createSignal(false);
  const [detailOpen, setDetailOpen] = createSignal(false);
  const [detailLoading, setDetailLoading] = createSignal(false);
  const [detailError, setDetailError] = createSignal<string>();
  const [detail, setDetail] = createSignal<PublicPushDetail>();
  const categories = createMemo(() => publicPushCategories(items()));
  const visibleItems = createMemo(() =>
    filterPublicPushes(items(), selectedCategory()),
  );
  let disposed = false;
  let unreadRequest = 0;
  let detailRequest = 0;

  const copy = () => CONTENT.chat_page.push_center;

  const acknowledgeVisiblePushes = async (
    request: number,
    loadedItems: PublicPushListItem[],
    unreadCount: number,
  ) => {
    const latestPushId = latestUnreadPushId(loadedItems, unreadCount);
    if (!latestPushId) return;
    try {
      const payload = await openPublicPush(latestPushId);
      if (disposed || request !== unreadRequest) return;
      props.onUnreadCountChange(payload.unread_count);
      setReadError(false);
    } catch {
      if (disposed || request !== unreadRequest) return;
      // Keep the server-reported unread count visible. A focus refresh retries.
      setReadError(true);
    }
  };

  const loadPushes = async (mode: "reset" | "more" = "reset") => {
    if (mode === "more") {
      if (!nextBefore() || loadingMore()) return;
      setLoadingMore(true);
    } else {
      setLoading(true);
      setError(undefined);
    }
    const request = ++unreadRequest;
    try {
      const payload = await getPublicPushes(
        mode === "more" ? nextBefore() : undefined,
      );
      if (disposed || request !== unreadRequest) return;
      const loadedItems =
        mode === "more"
          ? mergePublicPushItems(items(), payload.items)
          : payload.items;
      setItems(loadedItems);
      setNextBefore(payload.next_before ?? undefined);
      props.onUnreadCountChange(payload.unread_count);
      setReadError(false);
      await acknowledgeVisiblePushes(request, loadedItems, payload.unread_count);
    } catch (caught) {
      if (!disposed && request === unreadRequest) {
        setError(caught instanceof Error ? caught.message : String(caught));
      }
    } finally {
      if (!disposed && request === unreadRequest) {
        setLoading(false);
        setLoadingMore(false);
      }
    }
  };

  const openItem = async (item: PublicPushListItem) => {
    setDetailOpen(true);
    setDetail(undefined);
    setDetailError(undefined);
    setDetailLoading(true);
    const request = ++unreadRequest;
    const detailToken = ++detailRequest;
    try {
      const payload = await openPublicPush(item.push_id);
      if (disposed || detailToken !== detailRequest) return;
      setDetail(payload.push);
      if (request === unreadRequest) {
        props.onUnreadCountChange(payload.unread_count);
        setReadError(false);
      }
    } catch (caught) {
      if (!disposed && detailToken === detailRequest) {
        setDetailError(caught instanceof Error ? caught.message : String(caught));
      }
    } finally {
      if (!disposed && detailToken === detailRequest) setDetailLoading(false);
    }
  };

  onMount(() => {
    void loadPushes();
    const refresh = () => {
      if (document.visibilityState === "visible") void loadPushes();
    };
    window.addEventListener("focus", refresh);
    document.addEventListener("visibilitychange", refresh);
    onCleanup(() => {
      disposed = true;
      window.removeEventListener("focus", refresh);
      document.removeEventListener("visibilitychange", refresh);
    });
  });

  return (
    <section class="public-push-inbox" aria-labelledby="public-push-inbox-title">
      <header class="public-push-inbox-heading">
        <span>HONE 快报</span>
        <h2 id="public-push-inbox-title">{copy().messages_title}</h2>
        <p>{copy().messages_hint}</p>
      </header>

      <Show when={categories().length > 0}>
        <div class="public-push-categories" aria-label={copy().categories_aria}>
          <button
            type="button"
            classList={{ "is-active": selectedCategory() === ALL_PUBLIC_PUSHES }}
            onClick={() => setSelectedCategory(ALL_PUBLIC_PUSHES)}
          >
            {copy().all_categories}<i>{items().length}</i>
          </button>
          <For each={categories()}>
            {(category) => (
              <button
                type="button"
                classList={{ "is-active": selectedCategory() === category.jobId }}
                onClick={() => setSelectedCategory(category.jobId)}
              >
                {category.title}<i>{category.count}</i>
              </button>
            )}
          </For>
        </div>
      </Show>

      <Show when={readError()}>
        <div class="public-push-read-warning" role="status">
          {copy().read_sync_failed}
        </div>
      </Show>
      <Show when={loading() && items().length === 0}>
        <div class="public-push-inbox-state" role="status">{copy().sorting}</div>
      </Show>
      <Show when={error()}>
        <div class="public-push-inbox-state is-error">
          <span>{error()}</span>
          <button type="button" onClick={() => void loadPushes()}>{copy().retry}</button>
        </div>
      </Show>
      <Show when={!loading() && !error() && visibleItems().length === 0}>
        <div class="public-push-inbox-empty">
          <strong>{copy().empty_title}</strong>
          <p>{copy().empty_hint}</p>
        </div>
      </Show>

      <div class="public-push-inbox-list">
        <For each={visibleItems()}>
          {(item) => (
            <button type="button" class="public-push-inbox-item" onClick={() => void openItem(item)}>
              <span class="public-push-inbox-item-meta">
                <span>{item.title}</span>
                <time>{formatPushTime(item.created_at, useLocale())}</time>
              </span>
              <strong>{item.summary}</strong>
              <span class="public-push-inbox-item-action">
                {copy().view_full}<span aria-hidden="true">→</span>
              </span>
            </button>
          )}
        </For>
      </div>

      <Show when={nextBefore()}>
        <button
          type="button"
          class="public-push-inbox-more"
          disabled={loadingMore()}
          onClick={() => void loadPushes("more")}
        >
          {loadingMore() ? copy().loading : copy().load_more}
        </button>
      </Show>

      <PublicPushDetailDialog
        open={detailOpen()}
        detail={detail()}
        loading={detailLoading()}
        error={detailError()}
        onClose={() => setDetailOpen(false)}
      />
    </section>
  );
}

function formatPushTime(value: string, locale: "zh" | "en"): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(locale === "zh" ? "zh-CN" : "en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
}
