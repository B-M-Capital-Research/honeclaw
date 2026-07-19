import { historyToTimeline } from "./messages";
import { formatShanghaiDateTime } from "./time";
import type {
  HistoryAttachment,
  HistoryFinanceCalendar,
  HistoryMsg,
  PublicChatActiveRun,
  PublicPushListItem,
} from "./types";

export type PublicChatAuthState =
  | "loading"
  | "logged_out"
  | "logging_in"
  | "ready";

type PublicChatView = "loading" | "login" | "chat";

export const PUBLIC_RESTORE_TIMEOUT_MS = 8000;
export const PUBLIC_RESTORE_MAX_ATTEMPTS = 4;
export const PUBLIC_RESTORE_RETRY_DELAYS_MS = [700, 1600, 3200] as const;
export const PUBLIC_CHAT_CONTROLLED_PINCH_SELECTOR =
  ".public-finance-calendar-lightbox-viewport";
export const PUBLIC_CHAT_VIEWPORT_CONTENT =
  "width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, viewport-fit=cover, interactive-widget=resizes-content";

export function isPublicChatRestoreCurrent(input: {
  requestedSyncGeneration: number;
  currentSyncGeneration: number;
  requestedSendGeneration: number;
  currentSendGeneration: number;
}): boolean {
  return (
    input.requestedSyncGeneration === input.currentSyncGeneration &&
    input.requestedSendGeneration === input.currentSendGeneration
  );
}

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
  runId?: string;
  statusUpdatedAt?: number;
  steps?: string[];
  attachments?: PublicChatAttachment[];
  financeCalendar?: HistoryFinanceCalendar;
  scheduledPush?: {
    pushId?: string;
    title: string;
    summary: string;
    fallbackContent?: string;
    createdAt?: string;
  };
};

export function applyPublicAssistantStreamEvent(
  current: string,
  event: "assistant_delta" | "assistant_reset",
  delta = "",
): string {
  return event === "assistant_reset" ? "" : current + delta;
}

type PublicChatRunEventData = Partial<PublicChatActiveRun> & {
  text?: string;
};

function validEpochMs(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) && value > 0;
}

/** Human-readable server-owned run start, stable across refresh recovery. */
export function publicChatRunStartedAtLabel(
  startedAt: number | undefined,
): string | undefined {
  if (!validEpochMs(startedAt)) return undefined;
  return `查询开始：北京时间 ${formatShanghaiDateTime(
    new Date(startedAt).toISOString(),
    { second: undefined },
  )}`;
}

function nonEmptyText(...values: Array<string | undefined>): string | undefined {
  for (const value of values) {
    const trimmed = value?.trim();
    if (trimmed) return trimmed;
  }
  return undefined;
}

/**
 * Converts a server run_started/run_progress payload into a safe local patch.
 * Run identity and updated_at ordering prevent a late event from rewinding a
 * newer recovered state.
 */
export function publicChatRunEventPatch(
  current: PublicChatMessage,
  event: PublicChatRunEventData,
  fallbackStatus: string,
): Partial<PublicChatMessage> | undefined {
  const incomingRunId = nonEmptyText(event.run_id);
  if (current.runId && incomingRunId && current.runId !== incomingRunId) {
    return undefined;
  }

  const incomingUpdatedAt = validEpochMs(event.updated_at_ms)
    ? event.updated_at_ms
    : undefined;
  if (
    incomingUpdatedAt !== undefined &&
    current.statusUpdatedAt !== undefined &&
    incomingUpdatedAt < current.statusUpdatedAt
  ) {
    return undefined;
  }

  const phase =
    event.phase === "running" || event.phase === "thinking"
      ? event.phase
      : current.phase === "running"
        ? "running"
        : "thinking";
  const statusText =
    nonEmptyText(
      event.status_text,
      event.text,
      fallbackStatus,
      current.statusText,
    ) ?? fallbackStatus;

  return {
    phase,
    statusText,
    startedAt: validEpochMs(event.started_at_ms)
      ? event.started_at_ms
      : current.startedAt,
    runId: incomingRunId ?? current.runId,
    statusUpdatedAt: incomingUpdatedAt ?? current.statusUpdatedAt,
  };
}

export function publicChatToolStatusText(
  event: {
    public_status_text?: string;
    tool?: unknown;
    text?: unknown;
    reasoning?: unknown;
  },
  fallbackStatus: string,
): string {
  return nonEmptyText(event.public_status_text, fallbackStatus) ?? fallbackStatus;
}

export function resolvePublicChatRecovery(input: {
  activeRun?: PublicChatActiveRun | null;
  interruptedRun?: boolean;
  thinkingText: string;
  interruptedText: string;
}): { message?: PublicChatMessage; activeRunId?: string } {
  const activeRun = input.activeRun;
  const runId = nonEmptyText(activeRun?.run_id);
  if (activeRun && runId && validEpochMs(activeRun.started_at_ms)) {
    const updatedAt = validEpochMs(activeRun.updated_at_ms)
      ? activeRun.updated_at_ms
      : activeRun.started_at_ms;
    return {
      activeRunId: runId,
      message: {
        id: "_background",
        role: "assistant",
        content: "",
        phase: activeRun.phase === "running" ? "running" : "thinking",
        statusText:
          nonEmptyText(activeRun.status_text, input.thinkingText) ??
          input.thinkingText,
        startedAt: activeRun.started_at_ms,
        runId,
        statusUpdatedAt: updatedAt,
        steps: [],
      },
    };
  }

  if (input.interruptedRun) {
    return {
      message: {
        id: "_interrupted",
        role: "assistant",
        content: "",
        phase: "error",
        statusText: input.interruptedText,
        steps: [],
      },
    };
  }

  return {};
}

type PublicChatComposerState = {
  draft: string;
  attachmentCount: number;
  isSending: boolean;
  uploading: boolean;
  remaining: number | undefined;
  dailyLimit: number | undefined;
};

export function shouldSubmitPublicChatEnter(input: {
  key: string;
  shiftKey: boolean;
  eventIsComposing: boolean;
  compositionActive: boolean;
  keyCode: number;
  now: number;
  suppressEnterUntil: number;
}) {
  return (
    input.key === "Enter" &&
    !input.shiftKey &&
    !input.eventIsComposing &&
    !input.compositionActive &&
    input.keyCode !== 229 &&
    input.now >= input.suppressEnterUntil
  );
}

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

export function publicRestoreRetryDelay(attempt: number) {
  const index = Math.max(
    0,
    Math.min(attempt - 1, PUBLIC_RESTORE_RETRY_DELAYS_MS.length - 1),
  );
  return PUBLIC_RESTORE_RETRY_DELAYS_MS[index]!;
}

export function shouldRetryPublicRestore(
  attempt: number,
  maxAttempts = PUBLIC_RESTORE_MAX_ATTEMPTS,
) {
  return attempt < maxAttempts;
}

export function isPublicChatBusy(input: {
  isSending: boolean;
  hasPendingAssistant: boolean;
  hasBackgroundPending: boolean;
}) {
  return (
    input.isSending ||
    input.hasPendingAssistant ||
    input.hasBackgroundPending
  );
}

export function shouldPollPublicChatRecovery(input: {
  hasBackgroundPending: boolean;
  isSending: boolean;
  restoreInFlight: boolean;
}) {
  return (
    input.hasBackgroundPending && !input.isSending && !input.restoreInFlight
  );
}

export function shouldRecoverPublicChatAfterEof(input: {
  reachedEof: boolean;
  sawTerminalEvent: boolean;
}) {
  return input.reachedEof && !input.sawTerminalEvent;
}

export function resolvePublicChatStreamInterruption(input: {
  aborted: boolean;
  recoveringText: string;
  stoppedText: string;
}): {
  shouldRecover: boolean;
  patch: Pick<PublicChatMessage, "phase" | "statusText">;
} {
  if (input.aborted) {
    return {
      shouldRecover: false,
      patch: { phase: "error", statusText: input.stoppedText },
    };
  }
  return {
    shouldRecover: true,
    patch: { phase: "thinking", statusText: input.recoveringText },
  };
}

/**
 * Only these frames end the HTTP chat stream. `run_error` is deliberately not
 * terminal: a runner attempt can fail and still be recovered by the server
 * before the final `run_finished` frame arrives.
 */
export function isPublicChatTerminalStreamEvent(event: string) {
  return event === "run_finished" || event === "error" || event === "done";
}

/**
 * Maps the authoritative run_finished payload without turning an explicitly
 * partial committed stream into an error card. `partial` never means success;
 * it only says that already-rendered bytes must remain stable.
 */
export function publicChatTerminalEventPatch(
  event: { success?: boolean; partial?: boolean },
  lastRunError: string | undefined,
  fallbackError: string,
): Pick<PublicChatMessage, "phase" | "statusText"> {
  if (event.partial === true) {
    return { phase: "done", statusText: undefined };
  }
  if (event.success === false) {
    return {
      phase: "error",
      statusText: lastRunError ?? fallbackError,
    };
  }
  return { phase: "done", statusText: undefined };
}

export function shouldPreventPublicChatPinch(input: {
  touchCount: number;
  insideControlledSurface: boolean;
}) {
  return input.touchCount > 1 && !input.insideControlledSurface;
}

export function toPublicChatMessages(
  history: HistoryMsg[],
  historyStart = 0,
): PublicChatMessage[] {
  return historyToTimeline(history, historyStart)
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
      financeCalendar: message.financeCalendar,
      scheduledPush: message.scheduledPush
        ? {
            pushId: message.scheduledPush.push_id,
            title: message.scheduledPush.title,
            summary: message.scheduledPush.summary,
            fallbackContent: message.scheduledPush.fallback_content,
          }
        : undefined,
    }));
}

export function mergePublicHistoryWindow(
  current: readonly PublicChatMessage[],
  currentStart: number,
  latest: PublicChatMessage[],
  latestStart: number,
): { messages: PublicChatMessage[]; start: number } {
  if (current.length === 0 || latestStart <= currentStart) {
    return { messages: latest, start: latestStart };
  }
  const prefixLength = Math.min(current.length, latestStart - currentStart);
  return {
    messages: [...current.slice(0, prefixLength), ...latest],
    start: currentStart,
  };
}

export function mergePublicPushItems(
  existing: readonly PublicPushListItem[],
  incoming: readonly PublicPushListItem[],
): PublicPushListItem[] {
  const seen = new Set<string>();
  return [...existing, ...incoming].filter((item) => {
    if (seen.has(item.push_id)) return false;
    seen.add(item.push_id);
    return true;
  });
}

export function unreadCountAfterScheduledPush(
  current: number,
  serverCount?: number,
): number {
  return typeof serverCount === "number" && Number.isFinite(serverCount)
    ? Math.max(0, Math.floor(serverCount))
    : current + 1;
}

export function latestUnreadPushId(
  items: readonly PublicPushListItem[],
  unreadCount: number,
): string | undefined {
  if (!Number.isFinite(unreadCount) || unreadCount <= 0) return undefined;
  return items[0]?.push_id;
}

// History IDs generated by historyToTimeline() look like `h{index}_{base36hash}`.
// Anything else in the local message list is an optimistic id minted at send
// time (crypto.randomUUID() or the Date.now() fallback in messageId()).
const STABLE_HISTORY_ID_PATTERN = /^h\d+_/;

/**
 * Rewrite trailing items in `next` so they keep the same `id` as the
 * matching trailing items in `existing` when those existing ids are still
 * optimistic. Without this, reconcile() would treat the optimistic UUIDs
 * and the history-derived stable IDs as different keys, drop the local DOM
 * nodes and insert fresh ones — which momentarily shrinks the messages
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

export function shouldRecoverPinnedBottom(input: {
  scrollTop: number;
  distanceFromBottom: number;
  pinnedToBottom: boolean;
}) {
  return (
    input.pinnedToBottom &&
    input.scrollTop <= 24 &&
    input.distanceFromBottom > 120
  );
}

function isPublicChatQuotaCapped(dailyLimit: number | undefined) {
  return !!dailyLimit && dailyLimit > 0;
}

export function isPublicChatQuotaExhausted(input: {
  remaining: number | undefined;
  dailyLimit: number | undefined;
}) {
  return isPublicChatQuotaCapped(input.dailyLimit) && input.remaining === 0;
}

export function canSendPublicChatMessage(input: PublicChatComposerState) {
  return (
    !input.isSending &&
    !input.uploading &&
    (!!input.draft.trim() || input.attachmentCount > 0) &&
    !isPublicChatQuotaExhausted(input)
  );
}

export function splitPublicChatAttachments(
  attachments: readonly PublicChatAttachment[] | undefined,
) {
  const items = attachments ?? [];
  return {
    images: items.filter((attachment) => attachment.kind === "image"),
    files: items.filter((attachment) => attachment.kind !== "image"),
  };
}

export function formatPublicAttachmentBytes(bytes?: number) {
  if (!bytes) return "";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function publicAttachmentFileLabel(name: string) {
  const parts = name.split(".");
  return parts.length > 1
    ? parts[parts.length - 1]!.toUpperCase().slice(0, 4)
    : "FILE";
}

export function findPendingPublicAssistantMessage(
  messages: readonly PublicChatMessage[],
): PublicChatMessage | undefined {
  for (let i = messages.length - 1; i >= 0; i--) {
    const message = messages[i];
    if (
      message?.role === "assistant" &&
      message.phase &&
      message.phase !== "done" &&
      message.phase !== "error"
    ) {
      return message;
    }
  }
  return undefined;
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
