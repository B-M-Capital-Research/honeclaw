import type { LogEntry } from "./types"
import { actorFromSessionId, type ActorRef } from "./actors"

/**
 * 从一条日志里提取可"反向跳转"的实体引用。
 * 优先用 entry.extra 里的结构化字段(后端结构化日志),fallback 到 message 里的 Actor_ 模式串识别。
 *
 * 安全策略:不在自由文本里做模糊正则(避免把 ERROR/INFO/AAPL 等误识别为实体);
 * symbol/research/skill 等弱结构 ref 推迟到 Phase 3 引入 SymbolDrawer 时再加。
 */
export type LogRef =
  | { kind: "actor"; actor: ActorRef }
  | { kind: "session"; sessionId: string; actor?: ActorRef }
  | { kind: "task"; taskId: string }

const SESSION_ID_PATTERN = /Actor_[A-Za-z0-9_-]+__[A-Za-z0-9_-]+__[A-Za-z0-9_-]+/g

function readString(record: Record<string, unknown> | undefined, key: string): string | undefined {
  const v = record?.[key]
  return typeof v === "string" && v.trim() ? v.trim() : undefined
}

function readActorFromExtra(extra: Record<string, unknown> | undefined): ActorRef | undefined {
  if (!extra) return undefined
  // 优先 extra.actor: { channel, user_id, channel_scope }
  const nested = extra.actor
  if (nested && typeof nested === "object") {
    const obj = nested as Record<string, unknown>
    const channel = readString(obj, "channel")
    const user_id = readString(obj, "user_id")
    if (channel && user_id) {
      return {
        channel,
        user_id,
        channel_scope: readString(obj, "channel_scope"),
      }
    }
  }
  // fallback: 顶层 channel + user_id
  const channel = readString(extra, "channel") ?? readString(extra, "actor_channel")
  const user_id = readString(extra, "user_id") ?? readString(extra, "actor_user_id")
  if (channel && user_id) {
    return {
      channel,
      user_id,
      channel_scope: readString(extra, "channel_scope") ?? readString(extra, "actor_scope"),
    }
  }
  return undefined
}

function actorEqual(a: ActorRef, b: ActorRef): boolean {
  return (
    a.channel === b.channel &&
    a.user_id === b.user_id &&
    (a.channel_scope ?? "") === (b.channel_scope ?? "")
  )
}

export function extractLogRefs(entry: LogEntry): LogRef[] {
  const refs: LogRef[] = []
  const seenSessions = new Set<string>()
  const seenActors: ActorRef[] = []
  const seenTasks = new Set<string>()

  const pushActor = (actor: ActorRef) => {
    if (seenActors.some((a) => actorEqual(a, actor))) return
    seenActors.push(actor)
    refs.push({ kind: "actor", actor })
  }
  const pushSession = (sessionId: string) => {
    if (seenSessions.has(sessionId)) return
    seenSessions.add(sessionId)
    const actor = actorFromSessionId(sessionId)
    refs.push({ kind: "session", sessionId, actor })
    if (actor) pushActor(actor)
  }
  const pushTask = (taskId: string) => {
    if (seenTasks.has(taskId)) return
    seenTasks.add(taskId)
    refs.push({ kind: "task", taskId })
  }

  // 1) 结构化字段优先
  const actor = readActorFromExtra(entry.extra)
  if (actor) pushActor(actor)
  const sessionId = readString(entry.extra, "session_id")
  if (sessionId) pushSession(sessionId)
  const taskId =
    readString(entry.extra, "task_id") ?? readString(entry.extra, "cron_task_id")
  if (taskId) pushTask(taskId)

  // 2) message 文本里出现的 Actor_xxx__yy__zz 串
  const message = entry.message ?? ""
  const matches = message.match(SESSION_ID_PATTERN) ?? []
  for (const m of matches) pushSession(m)

  return refs
}

/** 检查日志是否匹配指定 user_id(用于工具栏按用户筛选) */
export function logMatchesUser(entry: LogEntry, userId: string): boolean {
  const target = userId.trim()
  if (!target) return true
  const refs = extractLogRefs(entry)
  for (const ref of refs) {
    if (ref.kind === "actor" && ref.actor.user_id === target) return true
    if (ref.kind === "session" && ref.actor?.user_id === target) return true
  }
  // 兜底:在 message 里 substring 匹配,捕获自定义格式
  if ((entry.message ?? "").includes(target)) return true
  return false
}
