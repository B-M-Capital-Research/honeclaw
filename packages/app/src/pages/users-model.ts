import { USERS } from "@/lib/admin-content/users"
import type { ActorListItem, ActorRef } from "@/lib/actors"
import { tpl } from "@/lib/i18n"

export type UsersTab = "portfolio" | "profiles" | "mainline" | "sessions" | "research"

type UsersTabConfig = {
  id: UsersTab
  labelKey:
    | "tab_portfolio"
    | "tab_profiles"
    | "tab_mainline"
    | "tab_sessions"
    | "tab_research"
  capability?: string
}

export const USER_TAB_CONFIG: UsersTabConfig[] = [
  { id: "portfolio", labelKey: "tab_portfolio" },
  { id: "profiles", labelKey: "tab_profiles", capability: "company_profiles" },
  { id: "mainline", labelKey: "tab_mainline" },
  { id: "sessions", labelKey: "tab_sessions" },
  { id: "research", labelKey: "tab_research", capability: "research" },
]

export const DEFAULT_MANUAL_ACTOR_DRAFT: ActorRef = {
  channel: "imessage",
  user_id: "",
  channel_scope: "",
}

export function resolveUsersTab(raw?: string): UsersTab {
  return USER_TAB_CONFIG.some((tab) => tab.id === raw)
    ? (raw as UsersTab)
    : "portfolio"
}

export function availableUsersTabs(
  hasCapability: (capability: string) => boolean,
): UsersTabConfig[] {
  return USER_TAB_CONFIG.filter(
    (tab) => !tab.capability || hasCapability(tab.capability),
  )
}

export function uniqueSortedSymbols(
  ...lists: Array<Array<{ symbol: string }>>
): string[] {
  const set = new Set<string>()
  for (const list of lists) {
    for (const item of list) {
      const symbol = item.symbol.trim()
      if (symbol) set.add(symbol.toUpperCase())
    }
  }
  return Array.from(set).sort()
}

export function patchActorDraft(
  draft: ActorRef,
  patch: Partial<ActorRef>,
): ActorRef {
  return { ...draft, ...patch }
}

export function actorFromManualDraft(draft: ActorRef): ActorRef | null {
  const channel = draft.channel.trim()
  const user_id = draft.user_id.trim()
  const channel_scope = draft.channel_scope?.trim()
  if (!channel || !user_id) return null
  return {
    channel,
    user_id,
    channel_scope: channel_scope || undefined,
  }
}

export function actorListSearchText(item: ActorListItem): string {
  return [
    item.actor.user_id,
    item.actor.channel,
    item.actor.channel_scope ?? "",
    item.sessionLabel ?? "",
  ]
    .join(" ")
    .toLowerCase()
}

export function filterActorList(
  items: ActorListItem[],
  rawQuery: string,
): ActorListItem[] {
  const normalizedQuery = rawQuery.trim().toLowerCase()
  if (!normalizedQuery) return items
  return items.filter((item) =>
    actorListSearchText(item).includes(normalizedQuery),
  )
}

export function actorListStatsText(item: ActorListItem): string {
  const parts: string[] = []
  if (item.holdingsCount != null && item.holdingsCount > 0) {
    parts.push(tpl(USERS.list.stat_holdings, { count: item.holdingsCount }))
  }
  if (item.watchlistCount != null && item.watchlistCount > 0) {
    parts.push(tpl(USERS.list.stat_watchlist, { count: item.watchlistCount }))
  }
  if (item.profileCount != null && item.profileCount > 0) {
    parts.push(tpl(USERS.list.stat_profiles, { count: item.profileCount }))
  }
  if (item.lastSessionTime) parts.push(USERS.list.stat_sessions)
  return parts.length > 0 ? parts.join(" · ") : USERS.list.stat_empty
}
