import type {
  CompanyProfileSpaceSummary,
  CronJobInfo,
  PortfolioSummary,
  UserInfo,
} from "./types"

export type ActorRef = {
  channel: string
  user_id: string
  channel_scope?: string
}

export function actorKey(actor: ActorRef) {
  return [actor.channel, actor.channel_scope ?? "", actor.user_id].join("|")
}

export function parseActorKey(key?: string): ActorRef | undefined {
  if (!key) return undefined
  const [channel, channel_scope, user_id, ...rest] = key.split("|")
  if (!channel || !user_id || rest.length > 0) return undefined
  return {
    channel,
    user_id,
    channel_scope: channel_scope || undefined,
  }
}

export function actorLabel(actor: ActorRef) {
  return actor.channel_scope ? `${actor.user_id} · ${actor.channel_scope}` : actor.user_id
}

export function actorFromUser(user: UserInfo): ActorRef {
  return {
    channel: user.channel,
    user_id: user.user_id,
    channel_scope: user.channel_scope,
  }
}

export function actorFromJob(job: CronJobInfo): ActorRef {
  return {
    channel: job.channel,
    user_id: job.user_id,
    channel_scope: job.channel_scope,
  }
}

function decodeActorComponent(raw: string): string {
  const bytes: number[] = []
  let i = 0
  while (i < raw.length) {
    const ch = raw[i]
    if (ch === "_" && i + 2 < raw.length) {
      const hex = raw.slice(i + 1, i + 3)
      if (/^[0-9a-fA-F]{2}$/.test(hex)) {
        bytes.push(parseInt(hex, 16))
        i += 3
        continue
      }
    }
    for (const b of new TextEncoder().encode(ch)) bytes.push(b)
    i += 1
  }
  return new TextDecoder().decode(new Uint8Array(bytes))
}

/** 用户中心(/users)左栏列表的合并 actor 摘要 */
export type ActorListItem = {
  actor: ActorRef
  key: string
  /** 来自持仓 */
  holdingsCount?: number
  watchlistCount?: number
  /** 来自公司画像 */
  profileCount?: number
  /** 来自会话 */
  sessionLabel?: string
  /** 最近会话时间(ISO),用于排序 */
  lastSessionTime?: string
  /** 任意一处的 updated_at,用于排序兜底 */
  updatedAt?: string
}

/**
 * 合并 portfolio / company-profiles / sessions 三处 actor 集合并去重。
 * 用于 /users 左栏 ActorList 一次性展示所有可看的人。
 */
export function mergeActorSummaries(input: {
  portfolios?: PortfolioSummary[]
  profiles?: CompanyProfileSpaceSummary[]
  sessions?: UserInfo[]
}): ActorListItem[] {
  const map = new Map<string, ActorListItem>()
  const ensure = (actor: ActorRef): ActorListItem => {
    const key = actorKey(actor)
    let item = map.get(key)
    if (!item) {
      item = { actor, key }
      map.set(key, item)
    }
    return item
  }

  for (const p of input.portfolios ?? []) {
    const item = ensure({
      channel: p.channel,
      user_id: p.user_id,
      channel_scope: p.channel_scope,
    })
    item.holdingsCount = p.holdings_count
    item.watchlistCount = p.watchlist_count
    if (p.updated_at && (!item.updatedAt || p.updated_at > item.updatedAt)) {
      item.updatedAt = p.updated_at
    }
  }

  for (const sp of input.profiles ?? []) {
    const item = ensure({
      channel: sp.channel,
      user_id: sp.user_id,
      channel_scope: sp.channel_scope,
    })
    item.profileCount = sp.profile_count
    if (sp.updated_at && (!item.updatedAt || sp.updated_at > item.updatedAt)) {
      item.updatedAt = sp.updated_at
    }
  }

  for (const u of input.sessions ?? []) {
    const item = ensure(actorFromUser(u))
    item.sessionLabel = u.session_label
    if (u.last_time) {
      if (!item.lastSessionTime || u.last_time > item.lastSessionTime) {
        item.lastSessionTime = u.last_time
      }
    }
  }

  return Array.from(map.values()).sort((a, b) => {
    const at = a.lastSessionTime ?? a.updatedAt ?? ""
    const bt = b.lastSessionTime ?? b.updatedAt ?? ""
    if (at !== bt) return bt.localeCompare(at)
    return a.actor.user_id.localeCompare(b.actor.user_id)
  })
}

/**
 * 从后端 session_id 反解 ActorRef。
 * 后端规则(crates/hone-core ActorIdentity::session_id):
 *   "Actor_" + encode(channel) + "__" + encode(scope|"direct") + "__" + encode(user_id)
 * encode: 字母数字和 "-" 保留,其它字节编码为 "_xx"(小写两位十六进制)
 */
export function actorFromSessionId(sessionId?: string): ActorRef | undefined {
  if (!sessionId || !sessionId.startsWith("Actor_")) return undefined
  const encoded = sessionId.slice("Actor_".length)
  const parts = encoded.split("__")
  if (parts.length !== 3) return undefined
  const channel = decodeActorComponent(parts[0]).trim()
  const scope = decodeActorComponent(parts[1])
  const user_id = decodeActorComponent(parts[2])
  if (!channel || !user_id) return undefined
  return {
    channel,
    user_id,
    channel_scope: scope && scope !== "direct" ? scope : undefined,
  }
}
