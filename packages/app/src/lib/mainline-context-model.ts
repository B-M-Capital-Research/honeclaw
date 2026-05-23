type ProfileTickerSource = {
  profile_list: Array<{
    tickers: string[]
  }>
}

type ProfileTickerList = {
  tickers: string[]
}

type MainlineTickerSource = ProfileTickerSource & {
  mainline_by_ticker: Record<string, string | undefined>
  mainline_distill_skipped: string[]
}

type ProfileInventorySource = ProfileTickerList & {
  bytes: number
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

export function profileTickerSet(
  context: ProfileTickerSource | null | undefined,
): Set<string> {
  const tickers = new Set<string>()
  if (!context) return tickers
  for (const profile of context.profile_list) {
    for (const ticker of profile.tickers) tickers.add(ticker)
  }
  return tickers
}

export function firstProfileTicker(profile: ProfileTickerList): string | null {
  return profile.tickers[0] ?? null
}

export function mainlineHoldingCardState(
  context: MainlineTickerSource,
  ticker: string,
  availableProfiles = profileTickerSet(context),
): MainlineHoldingCardState {
  return {
    ticker,
    mainline: context.mainline_by_ticker[ticker],
    hasProfile: availableProfiles.has(ticker),
    isSkipped: context.mainline_distill_skipped.includes(ticker),
  }
}

export function profileInventoryRowState(
  profile: ProfileInventorySource,
): ProfileInventoryRowState {
  return {
    title: profile.title || profile.dir,
    tickerLabel: profile.tickers.join(" / "),
    sizeLabel: `${(profile.bytes / 1024).toFixed(1)} KB`,
    dir: profile.dir,
    viewTicker: firstProfileTicker(profile),
  }
}
