import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const css = readFileSync(new URL("./public-chat.css", import.meta.url), "utf8");
const nav = readFileSync(
  new URL("../components/public-nav.tsx", import.meta.url),
  "utf8",
);
const chat = readFileSync(new URL("./chat.tsx", import.meta.url), "utf8");
const workspace = readFileSync(
  new URL("../components/public-agent-workspace.tsx", import.meta.url),
  "utf8",
);
const workspaceCss = readFileSync(
  new URL("./public-agent-workspace.css", import.meta.url),
  "utf8",
);
const shareCard = readFileSync(
  new URL("../components/chat-share-card.tsx", import.meta.url),
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

describe("public chat visual contract", () => {
  it("uses one responsive Agent workspace with real product destinations", () => {
    expect(chat).toContain("<AgentWorkspaceSidebar");
    expect(chat).toContain("<AgentWorkspaceMobileNav");
    expect(chat).toContain('onInsights={() => navigate("/community")}');
    // 五项导航：研究 / 洞察 / 投资助手 / 推送 / 我的。核心动作居中，是拇指
    // 最好够到的位置，两侧分别是内容与个人。
    expect(workspace).toContain("<span>投资助手</span>");
    const nav = workspace.slice(workspace.indexOf("agent-workspace-mobile-nav"));
    const navEnd = nav.indexOf("</nav>");
    const order = ["workspace.research", "workspace.insights", "<span>投资助手</span>", "workspace.pushes_tab", "workspace.me"]
      .map((token) => nav.slice(0, navEnd).indexOf(token));
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

  it("uses the flat mobile chat header instead of the floating site pill", () => {
    expect(nav).toContain('"is-chat-mode": props.chatMode');
    expect(nav).toContain("pub-nav-chat-copy");
    expect(css).toContain("border-radius: 0 !important");
    expect(css).toContain("padding-top: 58px !important");
    expect(css).toContain("align-items: center");
    expect(css).toContain("flex: 0 0 30px");
  });

  it("keeps the marketing navigation out of the authenticated workspace", () => {
    expect(chat).toContain('<Show when={authState() === "logged_out"}>');
    expect(workspace).toContain('class="agent-workspace-nav-with-dot"');
    expect(chat).toContain("communityUnread={communityUnread()}");
    expect(chat).toContain('onInsights={() => navigate("/community")}');
  });

  it("keeps the mobile composer clear of the fixed primary tabs", () => {
    expect(css).toContain(
      "padding-bottom: calc(76px + env(safe-area-inset-bottom)) !important",
    );
    expect(css).toContain('[data-theme="dark"] .public-chat-page .pub-mobile-tabs');
  });

  it("docks the mobile composer directly above the compact workspace tabs", () => {
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
    // 四项导航不再有中间凸起的 Agent 悬浮按钮
    expect(workspaceCss).not.toContain("button.is-agent");
    expect(workspaceCss).toContain(
      ".public-chat-page--ready { --agent-mobile-safe-bottom: 0px; }",
    );
    expect(workspaceCss).not.toContain("margin-top: -18px");
    expect(workspaceCss).toContain(
      ".public-chat-page.public-chat-page--ready .public-chat-composer",
    );
    expect(css).not.toContain(
      ".public-chat-page .public-chat-composer {\n    padding-bottom: calc(76px",
    );
  });

  it("restores inside the chat shell and exposes mobile conversation history", () => {
    expect(chat).not.toContain('<Match when={authState() === "loading"}>');
    // Agent 页只有对话视图，不再有 overview 分支
    expect(chat).not.toContain("AgentWorkspaceOverview");
    expect(chat).toContain('class="public-chat-shell is-conversation"');
    expect(chat).toContain("<AgentWorkspaceLoadingState");
    expect(chat).toContain("<AgentWorkspaceHistoryDrawer");
    expect(workspace).toContain("CONTENT.chat_page.workspace.drawer_aria");
    expect(workspaceCss).toContain("agent-workspace-history-drawer");
    expect(workspaceCss).toContain("agent-workspace-restore-notice");
    expect(workspaceCss).toContain("agent-workspace-loading");
  });

  it("anchors attachments to the centered composer and names the trigger", () => {
    expect(chat).toContain('class="public-chat-composer-frame"');
    // The label is localized now; the contract is that the trigger is named.
    expect(chat).toContain("CONTENT.chat_page.recovery.attach_aria");
    expect(chat).toContain('aria-haspopup="menu"');
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

  it("keeps quick actions on a horizontally scrollable mobile line", () => {
    expect(css).toContain("flex-wrap: nowrap !important");
    expect(css).toContain("overflow-x: auto !important");
    expect(css).toContain("scrollbar-width: none !important");
    expect(css).toContain("white-space: nowrap !important");
    expect(css).toContain("background: rgba(255, 255, 255, 0.99) !important");
    expect(css).toContain("backdrop-filter: none !important");
  });

  it("gates both earnings actions on the authenticated admin flag", () => {
    expect(chat).toContain("<Show when={props.isAdmin}>");
    expect(chat).toContain('kind="preview"');
    expect(chat).toContain('kind="analysis"');
    expect(chat).toContain("isAdmin={currentUser()?.is_admin === true}");
    expect(chat).toContain("onStartEarnings={startEarningsWorkflow}");
    expect(css).toContain(".public-chat-earnings-modal");
  });

  it("keeps assistant responses flat and user prompts softly filled", () => {
    const finalLayer = css.slice(css.indexOf("Final mobile chat overrides"));
    const darkMobileLayer = finalLayer.slice(
      finalLayer.indexOf('[data-theme="dark"] .public-chat-page--ready'),
      finalLayer.indexOf(".public-chat-community-action"),
    );
    expect(finalLayer).toContain("pub-msg-bubble--assistant");
    expect(finalLayer).toContain("background: transparent !important");
    expect(finalLayer).toContain("pub-msg-bubble--user");
    expect(finalLayer).toContain("background: #f1f1ef !important");
    expect(finalLayer).toContain('[data-theme="dark"]');
    expect(darkMobileLayer).toContain(
      "background: var(--agent-paper, var(--hone-paper-50)) !important",
    );
    expect(darkMobileLayer).toContain(
      "background: var(--hone-paper-200) !important",
    );
    expect(darkMobileLayer).toContain(
      "color: var(--agent-ink, var(--hone-ink-950)) !important",
    );
    expect(darkMobileLayer).not.toContain("#212121");
    expect(darkMobileLayer).not.toContain("#303030");
    expect(darkMobileLayer).not.toContain("#ececec");
  });

  it("removes the heavy composer shadow in the final mobile layer", () => {
    const finalLayer = css.slice(css.indexOf("Final mobile chat overrides"));
    expect(finalLayer).toContain("public-chat-composer-box");
    expect(finalLayer).toContain("box-shadow: none !important");
  });

  it("uses one HONE font contract across desktop chat controls", () => {
    expect(css).toContain("public-chat-sidebar-logout");
    expect(css).toContain("public-chat-proactive-tip");
    expect(css).toContain("font-family: var(--hone-font-body) !important");
    expect(css).toContain("font-weight: 700 !important");
  });

  it("keeps the desktop calendar dialog above the chat shell and inside the viewport", () => {
    const calendar = chat.slice(chat.indexOf("function FinanceCalendarQuickAction"));
    expect(calendar).toContain("<Portal>");
    expect(css).toContain("height: min(780px, calc(100dvh - 32px))");
    expect(css).toContain("grid-template-rows: auto minmax(0, 1fr)");
    expect(css).toContain(".public-chat-calendar-modal-body {\n  min-height: 0;");
  });

  it("keeps the desktop composer inside the workspace viewport", () => {
    expect(css).not.toContain(
      ".public-chat-page--ready .public-chat-shell {\n    height: 100dvh !important",
    );
    expect(workspaceCss).toContain(
      ".public-chat-page.public-chat-page--ready .agent-workspace-body > .public-chat-shell",
    );
    expect(workspaceCss).toContain("height: 100% !important; max-height: 100%");
    expect(workspaceCss).toContain(
      ".agent-workspace-body { min-height: 0; flex: 1; display: grid; grid-template-columns: minmax(0, 1fr); overflow: hidden; }",
    );
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

  it("keeps grounded ask-markers and discipline rules in the research panels", () => {
    expect(portfolioNewsDashboard).toContain("HONE_SAVED_PORTFOLIO_NEWS_REPORT");
    expect(portfolioNewsDashboard).toContain("不要自动修改仓位");
    expect(positionManagementDashboard).toContain("HONE_SAVED_POSITION_MANAGEMENT_REPORT");
    expect(positionManagementDashboard).toContain("不得自动修改持仓或声称已经下单");
    expect(influencerDigestDashboard).toContain("HONE_SAVED_INFLUENCER_DIGEST");
    expect(weeklyBriefDashboard).toContain("HONE_SAVED_WEEKLY_BRIEF");
    expect(keyEventChainDashboard).toContain("HONE_SAVED_KEY_EVENT_CHAIN");
  });
});
