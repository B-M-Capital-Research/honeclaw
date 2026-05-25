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
    let actorSummary = map.get(key)
    if (!actorSummary) {
      actorSummary = { actor, key }
      map.set(key, actorSummary)
    }
    return actorSummary
  }

  for (const portfolio of input.portfolios ?? []) {
    const actorSummary = ensure({
      channel: portfolio.channel,
      user_id: portfolio.user_id,
      channel_scope: portfolio.channel_scope,
    })
    actorSummary.holdingsCount = portfolio.holdings_count
    actorSummary.watchlistCount = portfolio.watchlist_count
    if (
      portfolio.updated_at &&
      (!actorSummary.updatedAt || portfolio.updated_at > actorSummary.updatedAt)
    ) {
      actorSummary.updatedAt = portfolio.updated_at
    }
  }

  for (const profileSpace of input.profiles ?? []) {
    const actorSummary = ensure({
      channel: profileSpace.channel,
      user_id: profileSpace.user_id,
      channel_scope: profileSpace.channel_scope,
    })
    actorSummary.profileCount = profileSpace.profile_count
    if (
      profileSpace.updated_at &&
      (!actorSummary.updatedAt || profileSpace.updated_at > actorSummary.updatedAt)
    ) {
      actorSummary.updatedAt = profileSpace.updated_at
    }
  }

  for (const sessionUser of input.sessions ?? []) {
    const actorSummary = ensure(actorFromUser(sessionUser))
    actorSummary.sessionLabel = sessionUser.session_label
    if (sessionUser.last_time) {
      if (
        !actorSummary.lastSessionTime ||
        sessionUser.last_time > actorSummary.lastSessionTime
      ) {
        actorSummary.lastSessionTime = sessionUser.last_time
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
