import { stripAttachmentMarkers, toPublicChatMessages } from "@/lib/public-chat";
import type { HistoryMsg } from "@/lib/types";

export type PublicWorkspaceResearchItem = {
  id: string;
  title: string;
};

export function publicWorkspaceResearchFromHistory(
  history: HistoryMsg[],
  historyStart = 0,
  limit = 6,
  attachmentFallback = "带附件的问题",
): PublicWorkspaceResearchItem[] {
  return toPublicChatMessages(history, historyStart)
    .filter((message) => message.role === "user")
    .slice(-limit)
    .reverse()
    .map((message) => ({
      id: message.id,
      title:
        stripAttachmentMarkers(message.content).replace(/\s+/g, " ").trim() ||
        attachmentFallback,
    }));
}
