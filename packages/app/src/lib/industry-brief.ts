import { latestAgeDays, isLatestStale } from "./industry-valuation";
import type {
  Industry,
  IndustryBrief,
  IndustrySource,
  IndustryUpstreamSignal,
} from "./types";

/**
 * 行业分析页首屏的两块——「当前重点」与「最近变化」——都从这里算。页面只渲染。
 *
 * 最近变化不是一张单独维护的表：它由底稿里本来就带日期的两类内容合并而成——上游信号的
 * 「最近动作」（latest + latest_as_of）和研报来源（takeaway + date）。多数行只有英伟达一条
 * 带日期的上游动作，来源才是「最近发生了什么」的主要供给，所以两者一起排、按日期倒序。
 */

export type ChangeItem = {
  /** 日期原样写法（"2026-08-26" 或 "2026-06"），排序用解析后的天数。 */
  asOf: string;
  ageDays: number;
  stale: boolean;
} & (
  | { kind: "signal"; signal: IndustryUpstreamSignal }
  | { kind: "source"; source: IndustrySource }
);

export type RecentChanges = {
  /** 按日期倒序的前若干条；同一天上游动作排在来源前面，其余保持底稿顺序。 */
  items: ChangeItem[];
  /** 没进前若干条的带日期条目数——页面用它写「其余 n 条在来源里」。 */
  rest: number;
  /** 还没有带日期动作的上游（latest 为空或 latest_as_of 解析不了）。 */
  undatedSignals: IndustryUpstreamSignal[];
};

export const RECENT_CHANGE_LIMIT = 5;

export function recentChanges(
  industry: Pick<Industry, "upstream_signals" | "sources">,
  now: Date = new Date(),
  limit: number = RECENT_CHANGE_LIMIT,
): RecentChanges {
  const dated: ChangeItem[] = [];
  const undatedSignals: IndustryUpstreamSignal[] = [];
  for (const signal of industry.upstream_signals ?? []) {
    const asOf = (signal.latest_as_of ?? "").trim();
    const ageDays = signal.latest?.trim() ? latestAgeDays(asOf, now) : undefined;
    if (ageDays === undefined) {
      undatedSignals.push(signal);
      continue;
    }
    dated.push({ kind: "signal", signal, asOf, ageDays, stale: isLatestStale(asOf, now) });
  }
  for (const source of industry.sources ?? []) {
    const asOf = (source.date ?? "").trim();
    const ageDays = latestAgeDays(asOf, now);
    if (ageDays === undefined || !source.takeaway?.trim()) continue;
    dated.push({ kind: "source", source, asOf, ageDays, stale: isLatestStale(asOf, now) });
  }
  // 稳定排序：Array.prototype.sort 在现代引擎里是稳定的，同一天的上游动作因为先 push 而在前。
  dated.sort((a, b) => a.ageDays - b.ageDays);
  return {
    items: dated.slice(0, Math.max(0, limit)),
    rest: Math.max(0, dated.length - limit),
    undatedSignals,
  };
}

export type IndustryBriefView = {
  /** brief = 管理员写的简报；derived = 底稿自动归纳，页面要标出来。 */
  source: "brief" | "derived";
  /** 现在值得研究的问题（简报）或这一行的一句话（派生，只用于提示词，页面不重复渲染）。 */
  lead: string;
  body: string[];
  /** 接下来先看什么，一条一件事。 */
  next: string[];
  /** 简报的截至日；派生时取最新一条变化的日期；都没有就是空串。 */
  asOf: string;
  /** 派生模式下带出最新的两条变化，让首屏不是一句空话；简报模式为空。 */
  changes: ChangeItem[];
};

function splitParagraphs(text: string): string[] {
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

/**
 * 首屏「当前重点」：有管理员写的简报就用它；没有就从底稿派生——最新两条变化与第一条关注点
 *（lead 带上一句话，供提示词用）。两样全空就返回 undefined，页面不渲染这一块。
 */
export function deriveIndustryBrief(
  industry: Pick<
    Industry,
    "one_liner" | "core_watch" | "upstream_signals" | "sources" | "valuation" | "brief"
  >,
  now: Date = new Date(),
): IndustryBriefView | undefined {
  const brief = industry.brief ?? undefined;
  if (brief && (brief.question.trim() || brief.body.trim())) {
    return {
      source: "brief",
      lead: brief.question.trim(),
      body: splitParagraphs(brief.body ?? ""),
      next: (brief.next ?? []).map((item) => item.trim()).filter(Boolean),
      asOf: (brief.as_of ?? "").trim(),
      changes: [],
    };
  }
  // 典型 State 与一句话页面正文已经单独展示，派生简报只补「最近发生了什么、下一次先看什么」。
  const lead = (industry.one_liner ?? "").trim();
  const changes = recentChanges(industry, now, 2).items;
  const first = (industry.core_watch ?? [])[0];
  const next = first?.what?.trim()
    ? [first.cadence?.trim() ? `${first.what.trim()}（${first.cadence.trim()}）` : first.what.trim()]
    : [];
  if (changes.length === 0 && next.length === 0) return undefined;
  return {
    source: "derived",
    lead,
    body: [],
    next,
    asOf: changes[0]?.asOf ?? "",
    changes,
  };
}

/** 给 brief 表单用：把简报还原成可编辑的四段文本。 */
export function briefToForm(brief: IndustryBrief | null | undefined) {
  return {
    question: brief?.question ?? "",
    body: brief?.body ?? "",
    next: (brief?.next ?? []).join("\n"),
    as_of: brief?.as_of ?? "",
  };
}

/**
 * 这一行内容里最新的事实截至日（与后端 `content_as_of` 同一规则，后端没带时前端自己算）：
 * 简报、上游动作、关注点、来源、传导链变量里能解析的最大日期，原样返回写法；同一天时日精度优先。
 */
export function contentAsOf(
  industry: Pick<
    Industry,
    "brief" | "upstream_signals" | "core_watch" | "sources" | "ai_valuation_logic"
  >,
  now: Date = new Date(),
): string | undefined {
  const candidates = [
    industry.brief?.as_of,
    ...(industry.upstream_signals ?? []).map((item) => item.latest_as_of),
    ...(industry.core_watch ?? []).map((item) => item.as_of),
    ...(industry.sources ?? []).map((item) => item.date),
    ...(industry.ai_valuation_logic?.key_variables ?? []).map((item) => item.as_of),
  ];
  let best: { text: string; age: number } | undefined;
  for (const raw of candidates) {
    const text = (raw ?? "").trim();
    const age = latestAgeDays(text, now);
    if (age === undefined) continue;
    if (!best || age < best.age || (age === best.age && text.length > best.text.length)) {
      best = { text, age };
    }
  }
  return best?.text;
}
