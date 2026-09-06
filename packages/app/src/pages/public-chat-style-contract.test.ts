import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const css = readFileSync(new URL("./public-chat.css", import.meta.url), "utf8");
const modalsCss = readFileSync(
  new URL("./public-chat-modals.css", import.meta.url),
  "utf8",
);
const accessibilityCss = readFileSync(
  new URL("./public-chat-accessibility.css", import.meta.url),
  "utf8",
);
const nav = readFileSync(
  new URL("../components/public-nav.tsx", import.meta.url),
  "utf8",
);
const chat = readFileSync(new URL("./chat.tsx", import.meta.url), "utf8");
const chatLib = readFileSync(
  new URL("../lib/public-chat.ts", import.meta.url),
  "utf8",
);
const workspace = readFileSync(
  new URL("../components/public-agent-workspace.tsx", import.meta.url),
  "utf8",
);
const workspaceCss = readFileSync(
  new URL("./public-agent-workspace.css", import.meta.url),
  "utf8",
);
const prefsCss = readFileSync(
  new URL("../components/public-prefs-button.css", import.meta.url),
  "utf8",
);
const shareCard = readFileSync(
  new URL("../components/chat-share-card.tsx", import.meta.url),
  "utf8",
);
const registry = readFileSync(
  new URL("../components/research/research-panels.tsx", import.meta.url),
  "utf8",
);
const researchPage = readFileSync(
  new URL("./public-research.tsx", import.meta.url),
  "utf8",
);
const ratingDashboard = readFileSync(
  new URL("../components/company-rating-dashboard.tsx", import.meta.url),
  "utf8",
);
const ratingCss = readFileSync(
  new URL("../components/company-rating-dashboard.css", import.meta.url),
  "utf8",
);
const portfolioNewsDashboard = readFileSync(
  new URL("../components/portfolio-news-dashboard.tsx", import.meta.url),
  "utf8",
);
const positionManagementDashboard = readFileSync(
  new URL("../components/position-management-dashboard.tsx", import.meta.url),
  "utf8",
);
const influencerDigestDashboard = readFileSync(
  new URL("../components/influencer-digest-dashboard.tsx", import.meta.url),
  "utf8",
);
const keyEventChainDashboard = readFileSync(
  new URL("../components/key-event-chain-dashboard.tsx", import.meta.url),
  "utf8",
);
const weeklyBriefDashboard = readFileSync(
  new URL("../components/weekly-brief-dashboard.tsx", import.meta.url),
  "utf8",
);

/** The CSS block that starts at `selector {` (first match). */
function block(source: string, selector: string) {
  const start = source.indexOf(`${selector} {`);
  expect(start).toBeGreaterThan(-1);
  return source.slice(start, source.indexOf("}", start));
}

describe("public chat visual contract", () => {
  it("uses one responsive Agent workspace with real product destinations", () => {
    expect(chat).toContain("<AgentWorkspaceSidebar");
    expect(chat).toContain("<AgentWorkspaceMobileNav");
    expect(chat).toContain('onInsights={() => navigate("/community")}');
    // 五项导航：研究 / 洞察 / 投资助手 / 推送 / 我的。核心动作居中，是拇指
    // 最好够到的位置，两侧分别是内容与个人。
    expect(workspace).toContain("<span>投资助手</span>");
    const mobileNav = workspace.slice(workspace.indexOf("agent-workspace-mobile-nav"));
    const navEnd = mobileNav.indexOf("</nav>");
    const order = ["workspace.research", "workspace.insights", "<span>投资助手</span>", "workspace.pushes_tab", "workspace.me"]
      .map((token) => mobileNav.slice(0, navEnd).indexOf(token));
    expect(order.every((at) => at >= 0)).toBe(true);
    expect(order).toEqual([...order].sort((a, b) => a - b));
    expect(workspace).toContain("agent-workspace-mobile-primary");
    expect(workspaceCss).toContain(".agent-workspace-mobile-primary > svg");
    expect(workspace).toContain("CONTENT.chat_page.workspace.research");
    expect(workspace).toContain('routePrefetchHandlers("research")');
    expect(workspace).toContain("CONTENT.chat_page.workspace.pushes_tab");
    expect(workspace).toContain("CONTENT.chat_page.workspace.insights");
    expect(workspace).toContain("CONTENT.chat_page.workspace.me");
    expect(workspace).toContain("props.unreadPushCount > 0");
    expect(chat).toContain("unreadPushCount={pushUnreadCount()}");
    expect(workspaceCss).toContain("grid-template-columns: minmax(0, 1fr)");
    expect(workspaceCss).toContain("grid-template-columns: repeat(5,1fr)");
    expect(workspaceCss).toContain("env(safe-area-inset-bottom, 0px)");
  });

  it("keeps one token layer, one breakpoint and no !important across the conversation layers", () => {
    // Four files used to override each other with five mobile media blocks
    // and two breakpoints (768 / 820); every tweak meant guessing which layer
    // won. The redesign is one component layer per file, written to lose
    // gracefully to nothing.
    for (const layer of [css, modalsCss, accessibilityCss, workspaceCss, prefsCss]) {
      expect(layer).not.toContain("!important");
      expect(layer).not.toContain("max-width: 768px");
      expect(layer).not.toContain("min-width: 769px");
    }
    expect(css).toContain("--hc-canvas: var(--hone-paper-50)");
    expect(css).toContain("--hc-accent: var(--hone-coral-500)");
    expect(workspaceCss).toContain("--agent-ink: var(--hone-ink-950)");
    expect(workspaceCss).toContain("--agent-rail: var(--hone-paper-100)");
    // The theme is the token flip: no hardcoded dark palette in the shell.
    expect(workspaceCss).not.toContain('[data-theme="dark"]');
    expect(workspaceCss).not.toContain("#fff;");
    expect(workspaceCss).not.toContain('"Avenir Next"');
  });

  it("renders every answer as a document with a byline, and the work as a trail", () => {
    // Assistant turns run the full column with no bubble; the only filled
    // block is the user's question.
    expect(chat).toContain('class="hc-turn hc-turn--assistant"');
    expect(chat).toContain('class="hc-turn__who">HONE');
    expect(chat).toContain('class="hc-turn hc-turn--user"');
    expect(block(css, ".hc-turn--assistant")).not.toContain("background");
    expect(block(css, ".hc-user")).toContain("background: var(--hc-fill)");
    expect(css).not.toContain("pub-msg-bubble");
    // The run's steps are a visible trail while pending and fold into one
    // line once the answer streams; the fold survives the post-answer restore.
    expect(chat).toContain("<TrailList steps={steps()} pending={pending()} />");
    expect(chat).toContain('class="hc-trail hc-trail--summary"');
    expect(chat).toContain("CONTENT.chat_page.trail.summary");
    expect(chat).toContain("merged.messages = carryOverLocalRunTrail(messages, merged.messages)");
    expect(chatLib).toContain("export function carryOverLocalRunTrail");
    expect(chat).toContain("finishedAt: Date.now()");
    // Research typography: report weights, ledger tables, tabular digits.
    expect(block(css, ".hc-turn__body .hf-markdown strong")).toContain("font-weight: 600");
    expect(block(css, ".hc-turn__body .hf-markdown table")).toContain("font-variant-numeric: tabular-nums");
    expect(css).toContain("--hc-answer-size: 15px");
    expect(css).toContain("--hc-answer-size: 16px");
  });

  it("puts every tool in the composer toolbar and lets the composer stop a stream", () => {
    expect(chat).toContain('class="hc-toolbar"');
    const toolbar = chat.slice(chat.indexOf('class="hc-toolbar__tools"'), chat.indexOf('class="hc-toolbar__end"'));
    for (const tool of [
      "<ChatToolsMenu isAdmin=",
      "<DataCenterQuickAction",
      "<FinanceCalendarQuickAction",
      "<CommunityQuickAction",
    ]) {
      expect(toolbar).toContain(tool);
    }
    // Every entry is one chip class, so the row reads as one control strip
    // rather than the old mix of pills at different weights.
    expect(chat).not.toContain("public-chat-proactive-tip");
    expect(chat).toContain('class="public-chat-send-button is-stop"');
    expect(chat).toContain("onStop={canStop() ? () => activeController?.abort() : undefined}");
    expect(chat).toContain("CONTENT.chat_page.composer.quota_remaining");
    // On phones the chips scroll and drop their labels; the send control
    // stays put outside the scroller.
    expect(block(css, ".hc-toolbar__tools")).toContain("overflow-x: auto");
    const phone = css.slice(css.indexOf("@media (max-width: 820px)"));
    expect(phone).toContain(".hc-tool .hc-tool__label {\n    display: none;");
    expect(phone).toContain("--hc-input-size: 16px");
    expect(chat).toContain('data-testid="composer-send-button"');
    expect(chat).toContain('data-testid="composer-attach-button"');
  });

  it("uses the workspace header on phones and the flat marketing header when signed out", () => {
    expect(nav).toContain('"is-chat-mode": props.chatMode');
    expect(nav).toContain("pub-nav-chat-copy");
    expect(workspaceCss).toContain("--agent-mobile-header-height: 52px");
    expect(block(workspaceCss, "  .agent-workspace-mobile-header")).toContain("position: fixed");
    expect(chat).toContain('<Show when={authState() === "logged_out"}>');
    expect(workspace).toContain('class="agent-workspace-nav-with-dot"');
    expect(chat).toContain("communityUnread={communityUnread()}");
  });

  it("docks the phone composer above the tabs and pads the list for its measured height", () => {
    expect(chat).toContain('class="public-chat-composer-dock"');
    expect(workspaceCss).toContain(
      "bottom: calc(var(--agent-mobile-nav-height) + var(--agent-mobile-composer-gap) + var(--agent-mobile-safe-bottom))",
    );
    expect(workspaceCss).toContain(
      "height: calc(var(--agent-mobile-nav-height) + var(--agent-mobile-safe-bottom))",
    );
    expect(workspaceCss).toContain("--agent-mobile-nav-height: 56px");
    expect(workspaceCss).toContain("--agent-mobile-composer-gap: 4px");
    expect(workspaceCss).toContain("grid-template-rows: 30px 14px");
    expect(workspaceCss).toContain("grid-row: 2");
    expect(workspaceCss).not.toContain("button.is-agent");
    expect(workspaceCss).toContain(
      ".public-chat-page--ready { --agent-mobile-safe-bottom: 0px; }",
    );
    // A fixed padding used to hide the newest turn behind a taller dock
    // (attachments, a multi-line draft); the page now measures the dock.
    expect(workspaceCss).toContain("padding-bottom: calc(var(--hc-dock-height, 120px) + 12px)");
    expect(chat).toContain('shell?.style.setProperty("--hc-dock-height"');
    expect(css).toContain("padding-bottom: calc(76px + env(safe-area-inset-bottom))");
  });

  it("restores inside the chat shell and exposes mobile conversation history", () => {
    expect(chat).not.toContain('<Match when={authState() === "loading"}>');
    // Agent 页只有对话视图，不再有 overview 分支
    expect(chat).not.toContain("AgentWorkspaceOverview");
    expect(workspace).not.toContain("AgentWorkspaceOverview");
    expect(chat).toContain('class="public-chat-shell is-conversation"');
    expect(chat).toContain("<AgentWorkspaceLoadingState");
    expect(chat).toContain("<AgentWorkspaceHistoryDrawer");
    expect(workspace).toContain("CONTENT.chat_page.workspace.drawer_aria");
    expect(workspaceCss).toContain("agent-workspace-history-drawer");
    expect(workspaceCss).toContain("agent-workspace-restore-notice");
    expect(workspaceCss).toContain("agent-workspace-loading");
    expect(chat).toContain("CONTENT.chat_page.workspace.restore_notice");
  });

  it("anchors attachments to the centered composer and names the trigger", () => {
    expect(chat).toContain('class="public-chat-composer-frame"');
    // The label is localized now; the contract is that the trigger is named.
    expect(chat).toContain("CONTENT.chat_page.recovery.attach_aria");
    expect(chat).toContain('aria-haspopup="menu"');
    expect(chat).toContain('data-testid="composer-attach-preview"');
  });

  it("downloads generated files through an authenticated blob with visible status", () => {
    expect(chat).toContain("getPublicGeneratedFileBlob");
    expect(chat).toContain('aria-busy={downloadState() === "working"}');
    expect(chat).toContain("CONTENT.chat_page.attachments.download_failed");
    expect(chat).not.toContain("href={publicAttachmentDownloadUrl(props.file)}");
  });

  it("keeps the exported share card readable under the dark application theme", () => {
    expect(shareCard).toContain('"--hone-ink-950": "#17201f"');
    expect(shareCard).toContain('"--hone-paper-100": "#f8f4ec"');
    expect(shareCard).toContain('"--hone-line": "rgba(23, 32, 31, 0.11)"');
  });

  it("gates both earnings actions on the authenticated admin flag", () => {
    expect(chat).toContain("<Show when={props.isAdmin}>");
    expect(chat).toContain('kind="preview"');
    expect(chat).toContain('kind="analysis"');
    expect(chat).toContain("isAdmin={currentUser()?.is_admin === true}");
    expect(chat).toContain("onStartEarnings={startEarningsWorkflow}");
    expect(modalsCss).toContain(".public-chat-earnings-modal");
  });

  it("keeps the desktop calendar dialog above the chat shell and inside the viewport", () => {
    const calendar = chat.slice(chat.indexOf("function FinanceCalendarQuickAction"));
    expect(calendar).toContain("<Portal>");
    expect(modalsCss).toContain("height: min(780px, calc(100dvh - 32px))");
    expect(modalsCss).toContain("grid-template-rows: auto minmax(0, 1fr)");
    expect(modalsCss).toContain(".public-chat-calendar-modal-body {\n  min-height: 0;");
    // The exported image depends on the artboard geometry.
    expect(modalsCss).toContain("width: 1080px;\n  height: 1350px;");
    expect(modalsCss).toContain("transform: scale(0.2778)");
  });

  it("keeps the desktop composer inside the workspace viewport", () => {
    expect(workspaceCss).toContain(
      ".public-chat-page.public-chat-page--ready .agent-workspace-body > .public-chat-shell",
    );
    expect(workspaceCss).toContain("height: 100%;\n  max-height: 100%;");
    expect(block(workspaceCss, ".agent-workspace-body")).toContain("overflow: hidden");
    expect(block(css, ".public-chat-messages")).toContain("overflow-y: auto");
  });

  it("keeps the review fixture out of production builds", () => {
    // `/chat?demo=1` seeds every turn state for design review; only a dev
    // build can reach it.
    expect(chat).toContain("import.meta.env.DEV &&");
    expect(chat).toContain('get("demo") === "1"');
    expect(chat).toContain("seedDemoConversation()");
  });

  it("hosts every daily research product on the research desk, not above chat", () => {
    // The chat page carries a slim navigation entry only; the dashboards
    // mount as URL-addressable panels on /research.
    for (const legacyMount of [
      "<DailySignalDashboard",
      "<CompanyRatingDashboard",
      "<PortfolioNewsDashboard",
      "<PositionManagementDashboard",
      "<InfluencerDigestDashboard",
      "<WeeklyBriefDashboard",
      "<KeyEventChainDashboard",
    ]) {
      expect(chat).not.toContain(legacyMount);
    }
    expect(chat).toContain("<ChatToolsMenu isAdmin=");
    // A tool picked from the composer opens where the reader already is —
    // navigating to the desk would drop the conversation they were in. The
    // panels stay lazy so the chat bundle does not carry seven dashboards.
    expect(chat).toContain("<ResearchPanelFor");
    expect(chat).toContain("onOpenPanel={openChatPanel}");
    expect(registry).toContain("lazy(() =>");
    expect(researchPage).toContain('<DailySignalPanel kind="macro"');
    expect(researchPage).toContain('<DailySignalPanel kind="ai"');
    expect(researchPage).toContain("<CompanyRatingPanel");
    expect(researchPage).toContain("<PortfolioNewsPanel");
    expect(researchPage).toContain("<PositionManagementPanel");
    expect(researchPage).toContain("<InfluencerDigestPanel");
    expect(researchPage).toContain("<WeeklyBriefPanel");
    expect(researchPage).toContain("<KeyEventChainPanel");
    // The open panel is shareable state, and back closes it.
    expect(researchPage).toContain("searchParams.panel");
    expect(researchPage).toContain("setSearchParams({ panel: section.key })");
    expect(researchPage).toContain("getPublicResearchOverview");
  });

  it("keeps the explainable rating methodology inside the rating panel", () => {
    expect(ratingDashboard).toContain("每日公司评级");
    expect(ratingDashboard).toContain("filterRatings");
    expect(ratingDashboard).toContain("item.valuation_method");
    expect(ratingDashboard).toContain("今日不计估值分");
    expect(ratingDashboard).toContain("valuation().probability_weighted_value");
    expect(ratingDashboard).toContain("评分标准与估值同步");
    expect(ratingDashboard).toContain("25 / 45 / 60 / 75 / 90");
    expect(ratingDashboard).toContain("没有生成或沿用旧目标价");
    expect(ratingDashboard).toContain("item.falsifiers");
    // Panel styling flows from the shared token system now.
    expect(ratingCss).toContain("var(--hone-");
  });

  it("leaves the panels read-only: no manual refresh, no ask-the-agent footer", () => {
    // Both controls were removed on request. The refresh button re-read the
    // same stored snapshot, so it looked broken; the ask footer is gone with
    // its whole prompt-envelope pipeline.
    for (const panel of [
      portfolioNewsDashboard,
      positionManagementDashboard,
      influencerDigestDashboard,
      keyEventChainDashboard,
      weeklyBriefDashboard,
      ratingDashboard,
    ]) {
      expect(panel).not.toContain("HONE_SAVED_");
      expect(panel).not.toContain("props.onAsk");
      expect(panel).not.toContain("重新读取");
      expect(panel).not.toContain("发送到对话");
    }
  });
});
