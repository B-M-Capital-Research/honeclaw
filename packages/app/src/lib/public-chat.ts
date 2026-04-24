import { historyToTimeline } from "./messages";
import type { HistoryAttachment, HistoryMsg } from "./types";

export type PublicChatAuthState =
  | "loading"
  | "logged_out"
  | "logging_in"
  | "ready";

export type PublicChatView = "loading" | "login" | "chat";

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

export function normalizeInviteCode(value: string) {
  return value.replace(/\s+/g, "").trim().toUpperCase();
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

function toPublicAttachments(
  items: HistoryAttachment[],
): PublicChatAttachment[] {
  return items.map((item) => ({
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
export function stripAttachmentMarkers(content: string): string {
  return content
    .split("\n")
    .filter((line) => !ATTACHMENT_LINE.test(line.trim()))
    .join("\n")
    .trim();
}
