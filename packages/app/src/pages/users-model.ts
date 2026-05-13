export type UsersTab = "portfolio" | "profiles" | "mainline" | "sessions" | "research"

export type UsersTabConfig = {
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
