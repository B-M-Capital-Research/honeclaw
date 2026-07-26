import type {
  FinanceCalendarEvent,
  PublicCommunityContent,
} from "./types";

export type AgentWorkspaceInsight = {
  id: string;
  eyebrow: string;
  title: string;
  summary: string;
};

export type AgentWorkspaceEvent = {
  id: string;
  date: string;
  time: string;
  title: string;
  summary: string;
};

const compactText = (value: string, limit: number) => {
  const normalized = value.replace(/\s+/g, " ").trim();
  if (normalized.length <= limit) return normalized;
  return `${normalized.slice(0, Math.max(0, limit - 1)).trimEnd()}…`;
};

export function communityToWorkspaceInsights(
  items: PublicCommunityContent[],
  limit = 3,
): AgentWorkspaceInsight[] {
  return items.slice(0, limit).map((item, index) => {
    const body = item.body_text.replace(/\s+/g, " ").trim();
    const splitAt = body.search(/[。！？!?\n]/);
    const titleSource = splitAt > 8 ? body.slice(0, splitAt) : body;
    const summarySource = splitAt > 8 ? body.slice(splitAt + 1) : body;
    return {
      id: String(item.content_id),
      eyebrow: index === 0 ? "社区新帖" : "社区动态",
      title: compactText(titleSource || item.author_name, 34),
      summary: compactText(
        summarySource || `${item.author_name} 发布了新的研究内容`,
        58,
      ),
    };
  });
}

export function calendarToWorkspaceEvents(
  events: FinanceCalendarEvent[],
  today: string,
  limit = 3,
): AgentWorkspaceEvent[] {
  return [...events]
    .filter((event) => event.date >= today)
    .sort((left, right) => left.date.localeCompare(right.date))
    .slice(0, limit)
    .map((event, index) => {
      const [date = event.date, time = ""] = event.date.split("T");
      return {
        id: `${event.date}-${event.title}-${index}`,
        date: date.slice(5).replace("-", "/"),
        time: time.slice(0, 5),
        title: event.title,
        summary:
          event.subtitle ||
          (event.kind === "earnings" ? "持仓相关财报事件" : event.source),
      };
    });
}

export type ResearchGroupItem = { id: string; title: string; at?: string };

export type ResearchGroup = { label: string; items: ResearchGroupItem[] };

/**
 * 把聊天记录按发生时间分组（今天 / 昨天 / 近 7 天 / 更早），
 * 输入按新→旧排列；缺时间戳的记录归入最后一组，空组不输出。
 */
export function groupResearchByDate(
  items: ResearchGroupItem[],
  nowMs = Date.now(),
): ResearchGroup[] {
  const startOfDay = (ms: number) => {
    const date = new Date(ms);
    date.setHours(0, 0, 0, 0);
    return date.getTime();
  };
  const today = startOfDay(nowMs);
  const yesterday = today - 24 * 3600 * 1000;
  const weekAgo = today - 6 * 24 * 3600 * 1000;
  const groups: ResearchGroup[] = [
    { label: "今天", items: [] },
    { label: "昨天", items: [] },
    { label: "近 7 天", items: [] },
    { label: "更早", items: [] },
  ];
  for (const item of items) {
    const parsed = item.at ? new Date(item.at).getTime() : Number.NaN;
    if (Number.isNaN(parsed)) {
      groups[3]!.items.push(item);
      continue;
    }
    const day = startOfDay(parsed);
    if (day >= today) groups[0]!.items.push(item);
    else if (day >= yesterday) groups[1]!.items.push(item);
    else if (day >= weekAgo) groups[2]!.items.push(item);
    else groups[3]!.items.push(item);
  }
  return groups.filter((group) => group.items.length > 0);
}

/** 对话时间线的日期分隔标签：今天 / 昨天 / M月D日（跨年补年份）。 */
export function messageDayLabel(at: string, nowMs = Date.now()): string | null {
  const parsed = new Date(at);
  if (Number.isNaN(parsed.getTime())) return null;
  const startOfDay = (date: Date) => {
    const clone = new Date(date);
    clone.setHours(0, 0, 0, 0);
    return clone.getTime();
  };
  const now = new Date(nowMs);
  const day = startOfDay(parsed);
  const today = startOfDay(now);
  if (day === today) return "今天";
  if (day === today - 24 * 3600 * 1000) return "昨天";
  const sameYear = parsed.getFullYear() === now.getFullYear();
  return sameYear
    ? `${parsed.getMonth() + 1}月${parsed.getDate()}日`
    : `${parsed.getFullYear()}年${parsed.getMonth() + 1}月${parsed.getDate()}日`;
}

/** 相邻消息跨天时返回新一天的标签，同一天返回 null。 */
export function daySeparatorLabel(
  previousAt: string | undefined,
  currentAt: string | undefined,
  nowMs = Date.now(),
): string | null {
  if (!currentAt) return null;
  const current = messageDayLabel(currentAt, nowMs);
  if (!current) return null;
  if (!previousAt) return current;
  const previous = messageDayLabel(previousAt, nowMs);
  return previous === current ? null : current;
}

export function workspaceGreeting(hour: number, name: string) {
  const normalizedName = name.trim() || "HONE 用户";
  if (hour < 6) return `夜深了，${normalizedName}`;
  if (hour < 12) return `早上好，${normalizedName}`;
  if (hour < 18) return `下午好，${normalizedName}`;
  return `晚上好，${normalizedName}`;
}

export function workspaceUserName(userId: string) {
  const normalized = userId.trim();
  if (!normalized || normalized.startsWith("web-user-")) return "HONE 用户";
  if (/^1\d{10}$/.test(normalized)) return `用户 ${normalized.slice(-4)}`;
  return compactText(normalized, 12);
}
