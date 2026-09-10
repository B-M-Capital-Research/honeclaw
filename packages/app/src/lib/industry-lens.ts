import type { Industry, IndustryMember } from "./types";

/**
 * 公司视角：选中一家公司后，整页围绕它重排——哪些上游动作、关注点、传导链变量、来源提到了它。
 * 判断「提到了」靠文本匹配，规则都在这里，页面只拿结果。
 */

type MemberName = Pick<IndustryMember, "symbol" | "name">;

/** 名称里这些片段不能当匹配词：它们描述的是上市地或公司形态，不是这家公司。 */
const GENERIC_NAME_PARTS = /ADR|ADS|美股|台股|港股|公司$/;

/** 中文全名常带的后缀；去掉后剩下的简称（美光科技 → 美光、迈威尔科技 → 迈威尔）才是正文里常见的写法。 */
const CHINESE_NAME_SUFFIX = /(科技|半导体|集团|控股|投控|股份|有限公司)$/;

/**
 * 一家公司的匹配词：代码 + 名称括号外的部分 + 括号里第一段，中文全名再加一个去掉「科技」这类后缀的简称。
 * 「西部数据（Western Digital）」→ WDC / 西部数据 / Western Digital；
 * 「美光科技（Micron）」→ MU / 美光科技 / 美光 / Micron；
 * 「台积电（TSMC，美股 ADR；台股 2330.TW）」→ TSM / 台积电 / TSMC（后面那些是上市地，不算）。
 * 两个字以下的片段丢掉，否则「光」「电」这种会命中整页。
 */
export function memberMentionTokens(member: MemberName): string[] {
  const tokens: string[] = [];
  const push = (raw: string) => {
    const text = raw.trim();
    if (text.length < 2 || GENERIC_NAME_PARTS.test(text)) return;
    if (!tokens.some((item) => item.toLowerCase() === text.toLowerCase())) tokens.push(text);
  };
  const pushName = (raw: string) => {
    push(raw);
    const short = raw.trim().replace(CHINESE_NAME_SUFFIX, "");
    if (short !== raw.trim()) push(short);
  };
  push(member.symbol.toUpperCase());
  const name = member.name ?? "";
  const bracket = name.match(/^(.*?)[（(](.*?)[）)]/);
  if (bracket) {
    pushName(bracket[1]);
    const inner = bracket[2].split(/[，,；;]/)[0] ?? "";
    pushName(inner);
  } else {
    pushName(name);
  }
  return tokens;
}

function escapeRegExp(text: string) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/**
 * 一段文本是否提到这家公司。代码要求两侧不是字母数字（MU 不能命中 MUX、ARM 不能命中 warm）；
 * 名称片段是不分大小写的子串。
 */
export function mentionsMember(text: string, member: MemberName): boolean {
  if (!text) return false;
  const symbol = member.symbol.trim();
  if (symbol.length >= 2) {
    const pattern = new RegExp(`(^|[^A-Za-z0-9])${escapeRegExp(symbol)}(?![A-Za-z0-9])`, "i");
    if (pattern.test(text)) return true;
  }
  const lower = text.toLowerCase();
  return memberMentionTokens(member)
    .filter((token) => token.toUpperCase() !== symbol.toUpperCase())
    .some((token) => lower.includes(token.toLowerCase()));
}

export type Ranked<T> = { item: T; related: boolean };

/**
 * 相关的稳定前置：没有选中公司时全部 related=false 且原序不动，页面在两种状态下用同一段渲染。
 */
export function rankByMention<T>(
  items: readonly T[],
  textOf: (item: T) => string,
  member: MemberName | undefined,
): Ranked<T>[] {
  if (!member) return items.map((item) => ({ item, related: false }));
  const ranked = items.map((item) => ({ item, related: mentionsMember(textOf(item), member) }));
  return [...ranked.filter((row) => row.related), ...ranked.filter((row) => !row.related)];
}

export type LensSummary = {
  signals: number;
  watch: number;
  variables: number;
  sources: number;
};

export const signalText = (signal: Industry["upstream_signals"][number]) =>
  [signal.symbol, signal.name, signal.why, signal.latest, ...(signal.pull ?? [])].join(" ");
export const watchText = (watch: Industry["core_watch"][number]) => `${watch.what} ${watch.why}`;
export const variableText = (variable: Industry["ai_valuation_logic"]["key_variables"][number]) =>
  `${variable.name} ${variable.why} ${variable.where}`;
export const sourceText = (source: Industry["sources"][number]) =>
  `${source.house} ${source.title} ${source.takeaway}`;

/** 公司卡上那一行计数：「上游动作 n · 关注点 n · 传导链变量 n · 来源 n」。 */
export function lensSummary(
  industry: Pick<Industry, "upstream_signals" | "core_watch" | "ai_valuation_logic" | "sources">,
  member: MemberName,
): LensSummary {
  const count = <T>(items: readonly T[] | undefined, textOf: (item: T) => string) =>
    (items ?? []).filter((item) => mentionsMember(textOf(item), member)).length;
  return {
    signals: count(industry.upstream_signals, signalText),
    watch: count(industry.core_watch, watchText),
    variables: count(industry.ai_valuation_logic?.key_variables, variableText),
    sources: count(industry.sources, sourceText),
  };
}
