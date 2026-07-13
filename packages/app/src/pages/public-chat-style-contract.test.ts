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

describe("public chat visual contract", () => {
  it("uses one responsive Agent workspace with real product destinations", () => {
    expect(chat).toContain("<AgentWorkspaceSidebar");
    expect(chat).toContain("<AgentWorkspaceRightRail");
    expect(chat).toContain("<AgentWorkspaceMobileNav");
    expect(chat).toContain('onInvest={() => navigate("/portfolio")}');
    expect(chat).toContain('onInsights={() => navigate("/community")}');
    expect(workspace).toContain("今日研究线索");
    expect(workspace).toContain("重要事件");
    expect(workspaceCss).toContain(
      "grid-template-columns: minmax(520px, 1fr) 270px",
    );
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
    expect(workspaceCss).toContain("--agent-mobile-nav-height: 42px");
    expect(workspaceCss).toContain("--agent-mobile-composer-gap: 4px");
    expect(workspaceCss).toContain("width: 20px; height: 20px");
    expect(workspaceCss).toContain("margin-top: 0; border: 0");
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
    expect(chat).toContain('>("conversation")');
    expect(chat).toContain("merged.messages.length > 0 ? \"conversation\" : \"overview\"");
    expect(chat).toContain("<AgentWorkspaceHistoryDrawer");
    expect(workspace).toContain('aria-label="会话历史"');
    expect(workspaceCss).toContain("agent-workspace-history-drawer");
    expect(workspaceCss).toContain("agent-workspace-restore-notice");
  });

  it("keeps quick actions on a horizontally scrollable mobile line", () => {
    expect(css).toContain("flex-wrap: nowrap !important");
    expect(css).toContain("overflow-x: auto !important");
    expect(css).toContain("scrollbar-width: none !important");
    expect(css).toContain("white-space: nowrap !important");
    expect(css).toContain("background: rgba(255, 255, 255, 0.99) !important");
    expect(css).toContain("backdrop-filter: none !important");
  });

  it("keeps assistant responses flat and user prompts softly filled", () => {
    const finalLayer = css.slice(css.indexOf("Final mobile chat overrides"));
    expect(finalLayer).toContain("pub-msg-bubble--assistant");
    expect(finalLayer).toContain("background: transparent !important");
    expect(finalLayer).toContain("pub-msg-bubble--user");
    expect(finalLayer).toContain("background: #f1f1ef !important");
    expect(finalLayer).toContain('[data-theme="dark"]');
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
      ".agent-workspace-body { min-height: 0; flex: 1; display: grid; grid-template-columns: minmax(520px, 1fr) 270px; overflow: hidden; }",
    );
  });
});
