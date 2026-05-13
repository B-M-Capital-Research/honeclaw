import { historyToTimeline } from "./messages";
import type { HistoryAttachment, HistoryMsg } from "./types";

export type PublicChatAuthState =
  | "loading"
  | "logged_out"
  | "logging_in"
  | "ready";

type PublicChatView = "loading" | "login" | "chat";

export type PublicChatAttachment = {
  /** Absolute server-side path returned by `/api/public/upload` or carried in history. */
  path: string;
  /** Display name (sanitized filename). */
  name: string;
  /** `image` / `pdf` / `file`. */
  kind: string;
  /** Size in bytes (only known for freshly uploaded files). */
  size?: number;
  /** Blob URL used for local preview before upload completes. */
  previewUrl?: string;
};

export type PublicChatMessage = {
  id: string;
  role: "user" | "assistant";
  content: string;
  phase?: "thinking" | "running" | "streaming" | "done" | "error";
  statusText?: string;
  startedAt?: number;
  steps?: string[];
  attachments?: PublicChatAttachment[];
};

export function normalizePhoneNumber(value: string) {
  const trimmed = value.trim();
  const hasLeadingPlus = trimmed.startsWith("+");
  const digits = trimmed.replace(/\D+/g, "");
  return hasLeadingPlus ? `+${digits}` : digits;
}

export function resolvePublicChatView(
  authState: PublicChatAuthState,
): PublicChatView {
  if (authState === "ready") return "chat";
  if (authState === "loading") return "loading";
  return "login";
}

export function toPublicChatMessages(
  history: HistoryMsg[],
): PublicChatMessage[] {
  return historyToTimeline(history)
    .filter(
      (message) => message.kind === "user" || message.kind === "assistant",
    )
    .map((message) => ({
      id: message.id,
      role: message.kind,
      content: message.content,
      phase: "done" as const,
      steps: [],
      attachments: toPublicAttachments(message.attachments ?? []),
    }));
}

export function selectVisibleRecentMessages<T>(
  messages: readonly T[],
  visibleCount: number,
): T[] {
  if (visibleCount <= 0) return [];
  return messages.slice(Math.max(0, messages.length - visibleCount));
}

export function nextVisibleMessageCount(
  totalMessages: number,
  currentVisibleCount: number,
  pageSize: number,
) {
  return Math.min(totalMessages, Math.max(0, currentVisibleCount) + pageSize);
}

// Server-side stable IDs always look like `h{index}_{base36hash}`. Anything
// else in the local message list is an optimistic id minted at send time
// (crypto.randomUUID() or the Date.now() fallback in messageId()).
const STABLE_HISTORY_ID_PATTERN = /^h\d+_/;

/**
 * Rewrite trailing items in `next` so they keep the same `id` as the
 * matching trailing items in `existing` when those existing ids are still
 * optimistic. Without this, reconcile() would treat the optimistic UUIDs
 * and the server-side `stableHistoryId`s as different keys, drop the local
 * DOM nodes and insert fresh ones — which momentarily shrinks the messages
 * container and lets the browser clamp scrollTop down to "the very top of
 * the conversation" before the ResizeObserver pulls it back. Roles must
 * match for the swap to be safe; we stop at the first structural divergence.
 *
 * Mutates `next` in place and returns it for convenient chaining.
 */
export function rekeyTrailingOptimisticIds<
  M extends { id: string; role: string },
>(existing: readonly M[], next: M[]): M[] {
  for (let offset = 1; offset <= Math.min(existing.length, next.length); offset++) {
    const oldMsg = existing[existing.length - offset];
    const newMsg = next[next.length - offset];
    if (!oldMsg || !newMsg) break;
    if (oldMsg.role !== newMsg.role) break;
    if (oldMsg.id === newMsg.id) continue;
    if (STABLE_HISTORY_ID_PATTERN.test(oldMsg.id)) break;
    // Cast away readonly on the id field — `next` is owned by us here.
    (newMsg as { id: string }).id = oldMsg.id;
  }
  return next;
}

export function shouldLoadOlderPublicMessages(input: {
  scrollTop: number;
  previousScrollTop: number;
  distanceFromBottom: number;
  hasOlderMessages: boolean;
  loadingOlderMessages: boolean;
  sendingOrStreaming: boolean;
}) {
  return (
    input.hasOlderMessages &&
    !input.loadingOlderMessages &&
    !input.sendingOrStreaming &&
    input.scrollTop <= 24 &&
    input.scrollTop < input.previousScrollTop - 2 &&
    input.distanceFromBottom > 120
  );
}

function toPublicAttachments(
  items: HistoryAttachment[],
): PublicChatAttachment[] {
  return items
    .filter(
      (item) =>
        item &&
        typeof item.path === "string" &&
        typeof item.name === "string" &&
        typeof item.kind === "string",
    )
    .map((item) => ({
      path: item.path,
      name: item.name,
      kind: item.kind,
    }));
}

const ATTACHMENT_LINE = /^\[附件:\s*.+\]$/;

/**
 * Strips `[附件: <path>]` marker lines (inserted server-side when a user sends
 * attachments) so we can render the text content without the raw marker.
 * Attachments are surfaced separately via `PublicChatMessage.attachments`.
 */
export function stripAttachmentMarkers(content: string | null | undefined): string {
  return (content ?? "")
    .split("\n")
    .filter((line) => !ATTACHMENT_LINE.test(line.trim()))
    .join("\n")
    .trim();
}
