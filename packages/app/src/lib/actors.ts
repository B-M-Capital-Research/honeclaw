import type { CronJobInfo, UserInfo } from "./types"

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
