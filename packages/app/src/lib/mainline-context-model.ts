export type ProfileTickerSource = {
  profile_list: Array<{
    tickers: string[]
  }>
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
