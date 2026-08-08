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

export { cachedPublicUser };

export function setCachedPublicUser(user: PublicAuthUserInfo | null) {
  setCachedPublicUserSignal(user === null ? null : { ...user });
}

/** A route may render optimistically only when something is actually known. */
export function hasCachedPublicUser(): boolean {
  return cachedPublicUser() !== null;
}
