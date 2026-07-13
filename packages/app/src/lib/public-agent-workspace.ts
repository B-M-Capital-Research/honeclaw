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
      eyebrow: index === 0 ? "新洞察" : "社区动态",
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
