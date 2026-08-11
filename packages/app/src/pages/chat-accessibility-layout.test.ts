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
  it("keeps every feature in an always-visible horizontal shelf above the composer", () => {
    expect(chat).toContain('class="chat-feature-rail"');
    expect(chat).toContain('class="chat-feature-rail__track"');
    expect(chat).not.toContain("featurePanelOpen");
    expect(chat).not.toContain("展开 9 项工具");
    expect(css).toContain(".chat-feature-rail__track");
    expect(css).toContain("overflow-x: auto");
    expect(css).toContain("scroll-snap-type: x proximity");
    expect(css).toContain("width: 144px !important");
    expect(css).toContain("height: 52px !important");
    expect(css).toContain("min-height: 180px !important");
    expect(css).toContain("padding-bottom: 218px !important");
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
    expect(source).toContain("北京时间");
  });
});
