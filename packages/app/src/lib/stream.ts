import type { ChatStreamEvent } from "./types"

export function parseSseChunks(buffer: string) {
  const parts = buffer.split("\n\n")
  const pending = parts.pop() ?? ""
  const events = parts.flatMap((part) => {
    let event = ""
    let data = ""
    for (const line of part.split("\n")) {
      if (line.startsWith("event: ")) event = line.slice(7).trim()
      if (line.startsWith("data: ")) data += line.slice(6)
    }
    if (!event) return []

    try {
      return [{ event, data: JSON.parse(data || "{}") } as ChatStreamEvent]
    } catch {
      return []
    }
  })

  return {
    events,
    pending,
  }
}
