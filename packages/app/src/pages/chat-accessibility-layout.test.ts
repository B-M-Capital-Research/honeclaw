import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const chat = readFileSync(new URL("./chat.tsx", import.meta.url), "utf8");
const css = readFileSync(
  new URL("./public-chat-accessibility.css", import.meta.url),
  "utf8",
);

const translatedSurfaces = [
  "../components/daily-signal-dashboard.tsx",
  "../components/weekly-brief-dashboard.tsx",
  "../components/influencer-digest-dashboard.tsx",
  "../components/key-event-chain-dashboard.tsx",
  "../components/portfolio-news-dashboard.tsx",
  "../components/position-management-dashboard.tsx",
  "../components/company-rating-dashboard.tsx",
  "../components/community-forum.tsx",
  "../components/public-chat-startup.tsx",
  "../components/public-agent-workspace.tsx",
  "../components/public-push-center.tsx",
  "../components/public-push-inbox.tsx",
  "../components/research-preview.tsx",
  "../components/finance-calendar-card.tsx",
  "../components/finance-calendar-mobile-card.tsx",
  "../lib/finance-calendar-mobile-renderer.ts",
].map((path) => readFileSync(new URL(path, import.meta.url), "utf8"));

describe("chat accessibility layout", () => {
  it("keeps the conversation page a conversation: one tool row, no data-fetching dashboards", () => {
    // The daily research products live on /research. The chat page keeps one
    // tool row and must never re-grow eager-fetching dashboard mounts or the
    // old launcher rail.
    expect(chat).toContain("<ChatToolsMenu isAdmin=");
    expect(chat).toContain('navigate("/research")');
    expect(chat).not.toContain("chat-feature-rail");
    // The eager dashboards must not come back; the on-demand panel mount is
    // a different thing and is asserted in the chat visual contract.
    expect(chat).not.toContain("Dashboard onAsk=");
    expect(chat).not.toContain("<DailySignalDashboard");
    // Destinations belong in the menu. A second chip row that navigates away
    // at the same visual weight as the in-chat actions is the thing this
    // replaced, so it must not come back.
    expect(chat).not.toContain('class="chat-research-entry"');
    expect(css).toContain(".chat-tools__menu");
    expect(css).not.toContain("chat-feature-rail");
    // The row is chrome, not a component zoo: tokens, no layout !important.
    expect(css).toContain("var(--hone-line)");
    expect(css).not.toContain("width: 144px !important");
  });

  it("no longer carries the retired ask-the-agent hand-off", () => {
    // The panels lost their ask footer, so the chat has nothing to collect.
    expect(chat).not.toContain("takeResearchAsk");
    expect(chat).not.toContain('searchParams.ask');
  });

  it("shows five personalized research hooks in a blank conversation", () => {
    expect(chat).toContain("buildChatStarterPrompts");
    expect(chat).toContain('class="chat-empty-prompts"');
    expect(chat).toContain("CONTENT.chat_page.workspace.starter_kicker");
    expect(chat).toContain("visibleMessages().length === 0");
    expect(chat).toContain("setPendingAutoSend(prompt.question)");
    expect(chat).toContain("setConversationStartIndex(messages.length)");
    expect(css).toContain("grid-template-columns: repeat(2, minmax(0, 1fr))");
  });

  it("uses larger readable defaults for messages, controls and tool cards", () => {
    expect(css).toContain('data-chat-fs="m"');
    expect(css).toContain("font-size: 18px !important");
    expect(css).toContain("font-size: 17px !important");
    expect(css).toContain("min-height: 44px");
    expect(css).toContain("font-size: 16px !important");
  });

  it("keeps visible navigation copy Chinese while retaining brand names", () => {
    const source = translatedSurfaces.join("\n");
    for (const obsoleteLabel of [
      "WEEKLY DECISION AGENDA",
      "HONE AGENT",
      "PORTFOLIO INTELLIGENCE",
      "HARI PORTFOLIO DISCIPLINE",
      "SOURCE BEFORE OPINION",
      "FIRST-PRINCIPLES INDUSTRY MAP",
      "HONE RESEARCH SIGNALS",
      "MEMBER DISCUSSION",
      "HONE Dispatch",
      "HONE RESEARCH",
      "SIGNAL CALENDAR",
      "FINANCE CALENDAR",
      "Ahead of Curve",
      "AI Sustainability",
    ]) {
      expect(source).not.toContain(obsoleteLabel);
    }
    expect(source).toContain("每周决策日程");
    expect(source).toContain("HONE 投资助手");
    expect(source).toContain("<span>投资助手</span>");
    expect(source).not.toContain("<span>Agent</span>");
    expect(source).toContain("运行时时区");
  });
});
