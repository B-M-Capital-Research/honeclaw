import { createSignal } from "solid-js";

import type { PublicAuthUserInfo } from "@/lib/types";

/**
 * The signed-in user, remembered across route changes.
 *
 * Every page used to call `/api/public/auth/me` from zero and blank itself
 * until the round-trip returned, so opening the account page from a chat that
 * already knew who the user was still showed a full-screen loading state. The
 * navigation itself is instant; that blank screen is the entire perceived
 * delay. Pages now render from this cache immediately and revalidate behind
 * the visible content.
 */
const [cachedPublicUser, setCachedPublicUserSignal] =
  createSignal<PublicAuthUserInfo | null>(null);

const [cachedCommunityFeed, setCachedCommunityFeedSignal] = createSignal<
  readonly unknown[] | null
>(null);

export { cachedPublicUser };

export function setCachedPublicUser(user: PublicAuthUserInfo | null) {
  const previousUserId = cachedPublicUser()?.user_id;
  if (user === null || (previousUserId && previousUserId !== user.user_id)) {
    // Community data belongs to the authenticated session. Never let a later
    // account paint the previous account's cached page while it revalidates.
    setCachedCommunityFeedSignal(null);
  }
  setCachedPublicUserSignal(user === null ? null : { ...user });
}

/** A route may render optimistically only when something is actually known. */
export function hasCachedPublicUser(): boolean {
  return cachedPublicUser() !== null;
}

/**
 * The last community feed page, remembered the same way and for the same
 * reason: reopening the section used to show a loading line even when the
 * previous page was still perfectly good to look at while it revalidated.
 */
export { cachedCommunityFeed };

export function setCachedCommunityFeed(items: readonly unknown[] | null) {
  setCachedCommunityFeedSignal(items === null ? null : [...items]);
}
