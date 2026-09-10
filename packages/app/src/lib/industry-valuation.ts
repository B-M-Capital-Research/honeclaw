import type {
  Industry,
  IndustryCoreWatch,
  IndustryMapSnapshot,
  IndustryMember,
  IndustryMethodology,
  IndustrySource,
  IndustrySubtype,
  IndustryUpstreamRelation,
  IndustryUpstreamSignal,
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
    // V5.3 行级散文与可观测变量表：归一化时必须带上，否则页面永远看不到它们。
    upstream_summary: value?.upstream_summary ?? "",
    transmission: value?.transmission ?? "",
    observables: value?.observables ?? [],
    subtype_intro: value?.subtype_intro ?? "",
    sources_note: value?.sources_note ?? "",
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
    technical_conventions: value?.technical_conventions ?? [],
    acceptance_cases: value?.acceptance_cases ?? [],
    references: value?.references ?? [],
  };
}

export const RELATION_LABELS: Record<IndustryUpstreamRelation, string> = {
  demand_source: "需求来源",
  capex_source: "资本开支来源",
  supply_gate: "供给卡口",
  peer_signal: "同业信号",
};

/** 上游关系的中文标签；底稿里出现没见过的值就原样回显，不吞掉。 */
export function relationLabel(value: string): string {
  return (RELATION_LABELS as Record<string, string>)[value] ?? value;
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

/** 提示词里引用的长段落只带开头：URL 要短，模型也不需要整段 why。 */
function clip(text: string, limit: number): string {
  const trimmed = text.trim();
  if (trimmed.length <= limit) return trimmed;
  return `${trimmed.slice(0, limit).trimEnd()}…`;
}

type Focus = Pick<IndustryMember, "symbol" | "name">;

function focusLabel(focus: Focus | undefined, fallback: string): string {
  if (!focus) return fallback;
  return focus.name?.trim() ? `${focus.symbol}（${focus.name.trim()}）` : focus.symbol;
}

/**
 * 从「最近变化」里的一条上游动作发问：这个动作影响谁、怎么传导、该去核什么。
 * 缺字段就省掉对应片段，不能出现「截至 」「undefined」这种半句。
 */
export function askSignalPrompt(
  industryName: string,
  signal: Pick<
    IndustryUpstreamSignal,
    "symbol" | "name" | "relation" | "latest" | "latest_as_of" | "pull"
  >,
  focus?: Focus,
): string {
  const who = focusLabel({ symbol: signal.symbol, name: signal.name }, signal.symbol);
  const asOf = signal.latest_as_of?.trim() ? `截至 ${signal.latest_as_of.trim()} 的` : "";
  const latest = signal.latest?.trim()
    ? `${who}${asOf}最近动作：「${clip(signal.latest, 220)}」。`
    : `${who}最近一季实际做了什么？`;
  const relation = signal.relation ? `它是${industryName}的${relationLabel(signal.relation)}。` : "";
  const label = focusLabel(focus, "");
  const target = focus
    ? `这会影响 ${label}${label.endsWith("）") ? "" : " "}哪部分业务、传导要几个季度？`
    : `这先影响${industryName}里哪几家公司、哪部分业务？`;
  const pull = (signal.pull ?? []).map((item) => item.trim()).filter(Boolean);
  const tail = pull.length > 0 ? `按行业本体给结论，再列要核的读数：${pull.join("；")}` : "按行业本体给结论。";
  return `${latest}${relation}${target}${tail}`;
}

/** 从「最近变化」里的一条来源发问：这份材料改变了哪条假设、先影响谁。 */
export function askSourcePrompt(
  industryName: string,
  source: Pick<IndustrySource, "house" | "title" | "date" | "takeaway">,
  focus?: Focus,
): string {
  const when = source.date?.trim() ? `（${source.date.trim()}）` : "";
  const takeaway = source.takeaway?.trim() ? `要点：${clip(source.takeaway, 220)}。` : "";
  const target = focus
    ? `这对 ${focusLabel(focus, "")} 意味着什么、改变了哪条假设？`
    : `这改变了${industryName}的哪条假设、先影响哪几家公司？`;
  return `${source.house.trim()}｜${source.title.trim()}${when}。${takeaway}${target}按行业本体给结论，并说明还要核什么。`;
}

/** 从一条关注点发问：最近一期读数、相对上一期的变化、要不要改估值结论。 */
export function askWatchPrompt(
  industryName: string,
  watch: Pick<IndustryCoreWatch, "what" | "why" | "cadence">,
  focus?: Focus,
): string {
  const cadence = watch.cadence?.trim() ? `（${watch.cadence.trim()}）` : "";
  const target = focusLabel(focus, "这一行");
  const why = watch.why?.trim() ? `看它的原因：${clip(watch.why, 160)}` : "";
  return `${industryName}的关注点「${watch.what.trim()}」${cadence}：最近一期读数是多少、相对上一期变了什么、对 ${target} 的估值结论要不要改？${why}`;
}

/** 从简报的「下一次先看」发问：把那一步做完，给数据、来源与对结论的影响。 */
export function askNextStepPrompt(industryName: string, lead: string, step: string): string {
  const context = lead.trim() ? `${industryName}当前重点：${clip(lead, 120)}。` : `${industryName}：`;
  return `${context}请完成下一步「${step.trim()}」，给出数据、来源与对估值结论的影响。`;
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
