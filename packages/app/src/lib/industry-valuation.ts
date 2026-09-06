import type {
  Industry,
  IndustryMapSnapshot,
  IndustryMember,
  IndustryMethodology,
  IndustrySubtype,
  IndustryValuation,
} from "./types";

/**
 * 行业本体 V3 的读端助手：找公司、定子类型、判最近动作是否过期、拼「问 HONE」的提示词。
 * 页面只做渲染，能离开 DOM 验证的规则都放这里。
 */

/** 后端与前端分开上线；旧快照还没带 V3 字段时按空结构渲染，页面各处不各自判空。 */
export function valuationOf(
  industry: Partial<Pick<Industry, "valuation">> | undefined,
): IndustryValuation {
  const value = industry?.valuation;
  return {
    logic: {
      summary: value?.logic?.summary ?? "",
      paragraphs: value?.logic?.paragraphs ?? [],
      formulas: value?.logic?.formulas ?? [],
      forward_focus: value?.logic?.forward_focus ?? [],
      state_note: value?.logic?.state_note ?? "",
    },
    anchor: {
      paragraphs: value?.anchor?.paragraphs ?? [],
      upper_range_drivers: value?.anchor?.upper_range_drivers ?? "",
      revision_optionality: value?.anchor?.revision_optionality ?? "",
      forbidden: value?.anchor?.forbidden ?? [],
    },
    subtypes: value?.subtypes ?? [],
  };
}

export function methodologyOf(
  snapshot: Partial<Pick<IndustryMapSnapshot, "methodology">> | undefined,
): IndustryMethodology {
  const value = snapshot?.methodology;
  return {
    version: value?.version ?? "",
    positioning: value?.positioning ?? "",
    principle: value?.principle ?? "",
    demand_chain: value?.demand_chain ?? [],
    core_principle: value?.core_principle ?? "",
    execution_rules: value?.execution_rules ?? [],
    hindsight_error: value?.hindsight_error ?? "",
    output_fields: value?.output_fields ?? [],
  };
}

/** 一家公司在这一行里属于哪个子类型：明确成员优先，其次是底稿推断的成员；都不在就 undefined。 */
export function subtypeOf(
  industry: Partial<Pick<Industry, "valuation">>,
  symbol: string,
): IndustrySubtype | undefined {
  const code = symbol.trim().toUpperCase();
  if (!code) return undefined;
  const subtypes = valuationOf(industry).subtypes;
  const has = (list: string[]) => list.some((member) => member.trim().toUpperCase() === code);
  return (
    subtypes.find((subtype) => has(subtype.members)) ??
    subtypes.find((subtype) => has(subtype.inferred_members))
  );
}

/** 这家公司在这个子类型里只是推断出来的（不在明确成员里）。 */
export function isInferredMember(subtype: IndustrySubtype, symbol: string): boolean {
  const code = symbol.trim().toUpperCase();
  const explicit = subtype.members.some((member) => member.trim().toUpperCase() === code);
  return !explicit && subtype.inferred_members.some((member) => member.trim().toUpperCase() === code);
}

export const INFERRED_MEMBER_TIP = "底稿按最近子类型推断，待人工确认";

export type MemberMatch = {
  industry: Industry;
  member: IndustryMember;
  subtype: IndustrySubtype | undefined;
};

/**
 * 找公司：代码精确 > 代码前缀 > 名称子串，全部不分大小写；同一档里按树的顺序取第一个。
 * 空查询不匹配任何东西。
 */
export function findMember(
  industries: readonly Industry[],
  query: string,
): MemberMatch | undefined {
  const needle = query.trim().toLowerCase();
  if (!needle) return undefined;
  const hit = (industry: Industry, member: IndustryMember): MemberMatch => ({
    industry,
    member,
    subtype: subtypeOf(industry, member.symbol),
  });
  let prefix: MemberMatch | undefined;
  let byName: MemberMatch | undefined;
  for (const industry of industries) {
    for (const member of industry.members) {
      const symbol = member.symbol.toLowerCase();
      if (symbol === needle) return hit(industry, member);
      if (!prefix && symbol.startsWith(needle)) prefix = hit(industry, member);
      if (!byName && member.name.toLowerCase().includes(needle)) byName = hit(industry, member);
    }
  }
  return prefix ?? byName;
}

/** 搜索框下面那一行：「SNDK · 存储 · 纯NAND/eSSD」；没有子类型时只给前两段。 */
export function matchLabel(match: MemberMatch): string {
  return [match.member.symbol, match.industry.name, match.subtype?.name]
    .filter((part): part is string => Boolean(part))
    .join(" · ");
}

/** 「最近动作」超过这个天数就标「可能已过期」：一个季度加上财报发布的滞后。 */
export const LATEST_STALE_AFTER_DAYS = 100;

/**
 * latest_as_of 支持 "2026-08-26"（日）与 "2026-06"（月，按月初算）两种写法，其它能被
 * Date 解析的字符串也认；解析不了就返回 undefined，页面不标，也不假装知道。
 */
export function latestAgeDays(asOf: string, now: Date = new Date()): number | undefined {
  const text = asOf.trim();
  if (!text) return undefined;
  const iso = /^\d{4}-\d{2}$/.test(text) ? `${text}-01` : text;
  const at = new Date(iso);
  if (Number.isNaN(at.getTime())) return undefined;
  return Math.floor((now.getTime() - at.getTime()) / 86_400_000);
}

export function isLatestStale(asOf: string, now: Date = new Date()): boolean {
  const age = latestAgeDays(asOf, now);
  return age !== undefined && age > LATEST_STALE_AFTER_DAYS;
}

/** 公司表每行的「问 HONE」：带着行业本体里的定位去开一轮前瞻估值。 */
export function askHonePrompt(symbol: string, name: string, subtypeName: string): string {
  return `按行业本体给 ${symbol}（${name}）做前瞻估值：先判 State 与 Capacity Unlock Gate，按子类型「${subtypeName}」的主锚次锚给 Valuation Horizon、Bear/Base/Bull 与 Market Implied，写明上游最近动作。`;
}

export function askHoneHref(prompt: string): string {
  return `/chat?q=${encodeURIComponent(prompt)}&send=1`;
}

/** 子类型 id：小写 kebab-case（与后端 UpsertSubtype 的校验同口径）。 */
export const SUBTYPE_ID_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

export function isSubtypeId(id: string): boolean {
  return SUBTYPE_ID_PATTERN.test(id);
}

/** 子类型表单里的成员：一行一个或逗号分隔，统一大写，去重。 */
export function splitSymbols(text: string): string[] {
  const seen = new Set<string>();
  for (const raw of text.split(/[\n,，、\s]+/)) {
    const code = raw.trim().toUpperCase();
    if (code) seen.add(code);
  }
  return [...seen];
}
