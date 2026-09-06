/** Resolve against the loaded tree so links also support administrator-added industries. */
export function resolveIndustryMapSelection(
  industries: readonly { id: string }[],
  requested: string | string[] | undefined,
): string | undefined {
  if (typeof requested === "string" && industries.some((industry) => industry.id === requested)) {
    return requested;
  }
  return industries[0]?.id;
}

/**
 * URL 里的 `?symbol=` 只在它真是当前行业的成员时才算选中（不分大小写，返回底稿里的写法）；
 * 不是就当没传——页面会把它从 URL 里规范化掉。
 */
export function resolveIndustryMapLens(
  industry: { members: readonly { symbol: string }[] } | undefined,
  requested: string | string[] | undefined,
): string | undefined {
  if (!industry || typeof requested !== "string") return undefined;
  const code = requested.trim().toUpperCase();
  if (!code) return undefined;
  return industry.members.find((member) => member.symbol.toUpperCase() === code)?.symbol;
}
