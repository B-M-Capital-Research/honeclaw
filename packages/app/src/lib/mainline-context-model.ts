type ProfileTickerSource = {
  profile_list?: Array<{
    tickers?: string[]
    ticker?: string
  }>
}

type ProfileTickerList = {
  tickers?: string[]
  ticker?: string
}

type MainlineTickerSource = ProfileTickerSource & {
  mainline_by_ticker?: Record<string, string | undefined>
  mainline_distill_skipped?: string[]
}

type ProfileInventorySource = ProfileTickerList & {
  bytes?: number
  dir: string
  title?: string | null
}

export type MainlineHoldingCardState = {
  ticker: string
  mainline: string | undefined
  hasProfile: boolean
  isSkipped: boolean
}

export type ProfileInventoryRowState = {
  title: string
  tickerLabel: string
  sizeLabel: string
  dir: string
  viewTicker: string | null
}

/* 后端历史版本的 profile_list 只有 { ticker, dir, preview }——这里对缺失字段
 * 一律兜底，避免旧服务端 + 新前端组合让投资页整页抛错。 */
function profileTickers(profile: {
  tickers?: string[]
  ticker?: string
}): string[] {
  if (Array.isArray(profile.tickers)) return profile.tickers
  return profile.ticker ? [profile.ticker] : []
}

export function profileTickerSet(
  context: ProfileTickerSource | null | undefined,
): Set<string> {
  const tickers = new Set<string>()
  if (!context) return tickers
  for (const profile of context.profile_list ?? []) {
    for (const ticker of profileTickers(profile)) tickers.add(ticker)
  }
  return tickers
}

export function firstProfileTicker(profile: ProfileTickerList): string | null {
  return profileTickers(profile)[0] ?? null
}

export function mainlineHoldingCardState(
  context: MainlineTickerSource,
  ticker: string,
  availableProfiles = profileTickerSet(context),
): MainlineHoldingCardState {
  return {
    ticker,
    mainline: (context.mainline_by_ticker ?? {})[ticker],
    hasProfile: availableProfiles.has(ticker),
    isSkipped: (context.mainline_distill_skipped ?? []).includes(ticker),
  }
}

export function profileInventoryRowState(
  profile: ProfileInventorySource,
): ProfileInventoryRowState {
  const tickers = profileTickers(profile)
  return {
    title: profile.title || profile.dir,
    tickerLabel: tickers.join(" / "),
    sizeLabel:
      typeof profile.bytes === "number" && Number.isFinite(profile.bytes)
        ? `${(profile.bytes / 1024).toFixed(1)} KB`
        : "—",
    dir: profile.dir,
    viewTicker: firstProfileTicker(profile),
  }
}
