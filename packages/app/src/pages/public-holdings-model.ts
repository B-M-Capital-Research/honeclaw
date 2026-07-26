// 「我的 · 自选与持仓」的纯逻辑：展示格式化、表单校验，以及点击持仓后
// 四个气泡菜单要跳去 Agent 问的问题。抽成纯函数便于单测。

export type HoldingRow = {
  symbol: string;
  name?: string | null;
  weight?: number | null;
  avg_cost?: number | null;
  tracking_only: boolean;
};

export type HoldingAskKind = "news" | "valuation" | "earnings";

/** 仓位展示：自选无占比，持仓保留一位小数。 */
export function formatHoldingWeight(row: HoldingRow): string {
  if (row.tracking_only || row.weight == null || !Number.isFinite(row.weight)) {
    return "自选";
  }
  return `${row.weight.toFixed(1)}%`;
}

export function formatHoldingCost(row: HoldingRow): string | null {
  if (row.avg_cost == null || !Number.isFinite(row.avg_cost) || row.avg_cost <= 0) {
    return null;
  }
  return `成本 ${row.avg_cost.toFixed(2)}`;
}

/** 持仓合计占比，用于顶部概览。自选不计入。 */
export function totalHoldingWeight(rows: readonly HoldingRow[]): number {
  return rows.reduce((sum, row) => {
    if (row.tracking_only || row.weight == null || !Number.isFinite(row.weight)) {
      return sum;
    }
    return sum + row.weight;
  }, 0);
}

function holdingLabel(row: HoldingRow): string {
  const name = row.name?.trim();
  return name ? `${name}（${row.symbol}）` : row.symbol;
}

/** 气泡菜单跳转 Agent 时带上的提问，始终包含公司名与代码。 */
export function holdingAskPrompt(row: HoldingRow, kind: HoldingAskKind): string {
  const label = holdingLabel(row);
  switch (kind) {
    case "news":
      return `${label} 近期最重要的新闻有哪些？请按重要性排序，说明每条对投资逻辑的影响，并注明来源和时间。`;
    case "valuation":
      return `${label} 目前的估值处于什么水平？请给出行业内普遍使用的估值方法与可比公司区间，并说明当前估值偏贵还是偏便宜。`;
    case "earnings":
      return `${label} 下一次财报是什么时间？请给出上一季财报的关键数据概要，以及这次财报需要重点关注的指标。`;
  }
}

export type HoldingFormInput = {
  symbol: string;
  name: string;
  weight: string;
  avgCost: string;
};

export type HoldingFormResult =
  | { ok: true; value: { symbol: string; name?: string; weight?: number; avg_cost?: number } }
  | { ok: false; error: string };

/**
 * 校验添加 / 调整表单。占比与成本都可以留空 —— 留空即「只加自选」。
 */
export function validateHoldingForm(input: HoldingFormInput): HoldingFormResult {
  const symbol = input.symbol
    .trim()
    .replace(/[^A-Za-z0-9.\-_]/g, "")
    .toUpperCase();
  if (!symbol) return { ok: false, error: "请填写股票代码，例如 AAPL" };

  const parseNumber = (raw: string, label: string) => {
    const trimmed = raw.trim();
    if (!trimmed) return { ok: true as const, value: undefined };
    const value = Number(trimmed);
    if (!Number.isFinite(value) || value <= 0) {
      return { ok: false as const, error: `${label}请填写大于 0 的数字` };
    }
    return { ok: true as const, value };
  };

  const weight = parseNumber(input.weight, "仓位占比");
  if (!weight.ok) return { ok: false, error: weight.error };
  if (weight.value !== undefined && weight.value > 100) {
    return { ok: false, error: "仓位占比不能超过 100%" };
  }
  const avgCost = parseNumber(input.avgCost, "成本价");
  if (!avgCost.ok) return { ok: false, error: avgCost.error };

  const name = input.name.trim();
  return {
    ok: true,
    value: {
      symbol,
      ...(name ? { name } : {}),
      ...(weight.value !== undefined ? { weight: weight.value } : {}),
      ...(avgCost.value !== undefined ? { avg_cost: avgCost.value } : {}),
    },
  };
}

/** 达到上限时禁用添加入口。 */
export function canAddHolding(count: number, limit: number): boolean {
  return count < limit;
}
