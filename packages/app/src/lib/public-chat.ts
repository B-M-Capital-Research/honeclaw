import { historyToTimeline } from "./messages";
import type { HistoryMsg } from "./types";

export type PublicChatAuthState =
  | "loading"
  | "logged_out"
  | "logging_in"
  | "ready";

export type PublicChatView = "loading" | "login" | "chat";

export type PublicChatMessage = {
  id: string;
  role: "user" | "assistant";
  content: string;
  phase?: "thinking" | "running" | "streaming" | "done" | "error";
  statusText?: string;
  startedAt?: number;
  steps?: string[];
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
    }));
}
