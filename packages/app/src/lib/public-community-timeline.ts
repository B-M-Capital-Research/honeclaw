import type { PublicCommunityContent, PublicCommunityPage } from "./types";

/** Keep already-read older pages only when the refreshed page joins them. */
export function mergeCommunityTimeline(
  current: readonly PublicCommunityContent[],
  currentNextBefore: number | null,
  page: PublicCommunityPage,
  append: boolean,
) {
  if (append) {
    const existing = new Set(current.map((item) => item.content_id));
    return {
      items: [...current, ...page.items.filter((item) => !existing.has(item.content_id))],
      nextBefore: page.next_before ?? null,
    };
  }

  const lastIncoming = page.items.at(-1)?.content_id;
  const overlap = current.findIndex((item) => item.content_id === lastIncoming);
  if (page.next_before == null || overlap < 0 || overlap === current.length - 1) {
    return { items: page.items, nextBefore: page.next_before ?? null };
  }

  const incoming = new Set(page.items.map((item) => item.content_id));
  return {
    items: [
      ...page.items,
      ...current.slice(overlap + 1).filter((item) => !incoming.has(item.content_id)),
    ],
    nextBefore: currentNextBefore,
  };
}

/** The three Insights streams. Only "official" is backed by the PG timeline. */
export type CommunityView = "official" | "influencer" | "forum";

/** `?view=` deep link; anything unknown falls back to the official feed. */
export function communityViewFromParam(value: unknown): CommunityView {
  return value === "influencer" || value === "forum" ? value : "official";
}
