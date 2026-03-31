import type { HistoryMsg, TimelineMessage } from "./types"
import { buildApiUrl, hasRuntimeCapability } from "./backend"

const imagePattern = /(file:\/\/[^\s]+\.(?:jpg|jpeg|png|webp|gif|bmp))/gi

export type MessagePart =
  | { type: "text"; value: string }
  | { type: "image"; value: string }

export function messageId() {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID()
  }
  return `${Date.now()}-${Math.random().toString(16).slice(2)}`
}

/**
 * 为历史消息生成稳定的确定性 ID。
 *
 * 使用 index + role + 内容 djb2 hash 组合，确保同一条消息在多次轮询中
 * 始终拿到相同的 ID，让 VList / reconcile 可以跳过未变化的消息节点，
 * 彻底消除每次轮询带来的全量重渲染。
 *
 * 注意：后端返回的历史消息是"已完成"状态，内容稳定不会再改变，
 * 因此基于内容的 hash 完全可以作为稳定键。
 */
function stableHistoryId(index: number, role: string, content: string): string {
  // djb2 hash（快速、无依赖、碰撞极低）
  let h = 5381
  const str = role + content
  for (let i = 0; i < str.length; i++) {
    h = ((h << 5) + h) ^ str.charCodeAt(i)
  }
  return `h${index}_${(h >>> 0).toString(36)}`
}

export function historyToTimeline(messages: HistoryMsg[]): TimelineMessage[] {
  return messages
    .filter(
      (message) =>
        message.subtype === "compact_boundary" || !message.transcript_only,
    )
    .map((message, index) => ({
      id: stableHistoryId(index, message.role, message.content),
      kind: message.role === "user" || message.role === "assistant" ? message.role : "system",
      content: message.content,
      subtype: message.subtype,
      synthetic: message.synthetic,
      transcriptOnly: message.transcript_only,
    }))
}

export function parseMessageContent(text: string) {
  const matches = text.match(imagePattern)
  if (!matches) {
    return [{ type: "text", value: text }] as MessagePart[]
  }

  return text.split(imagePattern).flatMap<MessagePart>((part) => {
    if (!part) return []
    if (part.startsWith("file://")) {
      if (!hasRuntimeCapability("local_file_proxy")) {
        return [{ type: "text", value: part }]
      }
      return [{ type: "image", value: buildApiUrl(`/api/image?path=${encodeURIComponent(part.replace("file://", ""))}`) }]
    }
    return [{ type: "text", value: part }]
  })
}
