import type {
  CompanyRating,
  CompanyRatingLight,
  CompanyRatingSnapshot,
} from "./types";

export type CompanyRatingFilter = "all" | CompanyRatingLight;

export function lightForScore(score: number): CompanyRatingLight {
  if (score >= 75) return "green";
  if (score >= 55) return "yellow";
  return "red";
}

export function ratingCounts(items: CompanyRating[]) {
  return items.reduce(
    (counts, item) => {
      counts[item.light] += 1;
      return counts;
    },
    { green: 0, yellow: 0, red: 0, unknown: 0 },
  );
}

export function filterRatings(
  items: CompanyRating[],
  filter: CompanyRatingFilter,
  query: string,
) {
  const needle = query.trim().toLocaleLowerCase();
  return items
    .filter((item) => filter === "all" || item.light === filter)
    .filter(
      (item) =>
        !needle ||
        item.symbol.toLocaleLowerCase().includes(needle) ||
        item.name.toLocaleLowerCase().includes(needle) ||
        item.theme.toLocaleLowerCase().includes(needle),
    )
    .slice()
    .sort((a, b) => b.score - a.score || a.symbol.localeCompare(b.symbol));
}

export function coverageLabel(snapshot: CompanyRatingSnapshot) {
  const { coverage, data_status: status } = snapshot;
  if (status === "simulation") return `Codex 模拟预览 · 8/8 因子 · 非真实数据`;
  if (status === "live") return `行情、财报与当日估值 ${coverage.companies}/${coverage.companies}`;
  if (status === "stale") return "数据源异常 · 展示上次成功结果";
  if (status === "transcript_only") {
    return `仅演讲研究基线 · 行情 ${coverage.quotes}/${coverage.companies} · 财报 ${coverage.financials}/${coverage.companies} · 当日估值 ${coverage.valuations ?? 0}/${coverage.companies}`;
  }
  return `部分更新 · 行情 ${coverage.quotes}/${coverage.companies} · 财报 ${coverage.financials}/${coverage.companies} · 当日估值 ${coverage.valuations ?? 0}/${coverage.companies}`;
}
