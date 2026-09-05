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
