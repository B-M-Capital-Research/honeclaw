import { createSignal } from "solid-js";

/**
 * Route-stable unread state. API responses remain authoritative, while the
 * shared signal prevents navigation to /pushes from visually clearing a badge
 * before the read-through POST has actually succeeded.
 */
export const [publicPushUnreadCount, setPublicPushUnreadCount] = createSignal(0);
