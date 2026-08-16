import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const shell = readFileSync(
  new URL("../components/public-workspace-shell.tsx", import.meta.url),
  "utf8",
);
const startup = readFileSync(
  new URL("../components/public-chat-startup.tsx", import.meta.url),
  "utf8",
);
const css = readFileSync(new URL("./public-workspace.css", import.meta.url), "utf8");
const community = readFileSync(new URL("./public-community.tsx", import.meta.url), "utf8");
const me = readFileSync(new URL("./public-me.tsx", import.meta.url), "utf8");
const research = readFileSync(new URL("./public-research.tsx", import.meta.url), "utf8");
const pushes = readFileSync(new URL("./public-pushes.tsx", import.meta.url), "utf8");
const pushInbox = readFileSync(
  new URL("../components/public-push-inbox.tsx", import.meta.url),
  "utf8",
);
const adminUsage = readFileSync(
  new URL("../components/public-admin-usage-panel.tsx", import.meta.url),
  "utf8",
);
const adminWhitelist = readFileSync(
  new URL("../components/public-admin-whitelist-panel.tsx", import.meta.url),
  "utf8",
);

describe("public workspace page contract", () => {
  it("shares one desktop and mobile chrome across 洞察 and 我的", () => {
    expect(shell).toContain("<AgentWorkspaceSidebar");
    expect(shell).toContain("<AgentWorkspaceMobileHeader");
    expect(shell).toContain("<AgentWorkspaceMobileNav");
    expect(community).toContain('<PublicWorkspaceShell\n          active="insights"');
    expect(me).toContain('<PublicWorkspaceShell active="me"');
    expect(shell).toContain("<PublicPrefsButton");
    expect(shell).toContain('const openPushCenter = () => navigate("/pushes")');
    expect(shell).toContain("unreadPushCount={pushUnreadCount()}");
    expect(shell).toContain("getPublicChatBootstrap");
    expect(shell).toContain("publicWorkspaceResearchFromHistory");
  });

  it("keeps push content and subscription management in one state-closed destination", () => {
    expect(pushes).toContain("<PublicPushInbox");
    expect(pushes).toContain("<PublicSubscriptionManager");
    expect(pushes).toContain('searchParams.view === "manage"');
    expect(pushes).toContain('view() === "messages" ? publicPushUnreadCount() : undefined');
    expect(pushInbox).toContain("publicPushCategories(items())");
    expect(pushInbox).toContain("latestUnreadPushId(loadedItems, unreadCount)");
    expect(pushInbox).toContain("props.onUnreadCountChange(payload.unread_count)");
    expect(pushInbox).not.toContain("onUnreadCountChange(0)");
  });

  it("uses a continuous insight stream and separate desktop/mobile tracking views", () => {
    expect(css).toContain("one continuous editorial stream");
    expect(css).toContain(".public-community-card:last-of-type");
  });

  it("folds holdings and settings into the 我的 page", () => {
    expect(me).toContain("<PublicHoldingsPanel />");
    expect(me).toContain("<PublicSettingsPanel />");
    expect(css).toContain(".public-holdings-list");
    expect(css).toContain(".public-holding-bubble");
    expect(css).toContain(".public-settings-input");
  });

  it("shows whitelist management only for server-authoritative administrators", () => {
    // 用户端管理能力集中在研究台「管理」分类，仍由服务端 is_admin 把关。
    expect(me).toContain("<Show when={props.user.is_admin}>");
    expect(research).toContain("isAdmin()");
    expect(research).toContain("<PublicAdminUsagePanel />");
    expect(research).toContain("<PublicAdminWhitelistPanel />");
    expect(research).toContain('activeGroup() === "admin" && isAdmin()');
    expect(css).toContain(".public-admin-panel");
    expect(css).toContain(".public-admin-live-summary");
    expect(css).toContain(".public-admin-table td::before");
    expect(adminUsage).toContain("<details");
    expect(adminUsage).toContain("selectedChannel()");
    expect(adminUsage).toContain("PUBLIC_ADMIN_USAGE_RANGES");
    expect(adminUsage).toContain("PUBLIC_ADMIN_USAGE_CHANNELS");
    expect(adminUsage).toContain("CONTENT.chat_page.admin.u_daily_users");
    expect(adminUsage).toContain("CONTENT.chat_page.admin.u_daily_questions");
    expect(adminUsage).toContain("public-admin-trend-detail");
    expect(adminUsage).toContain("formatUsageChannel(row.channel)");
    expect(adminWhitelist).toContain("<details");
    expect(css).toContain(".public-admin-section-summary");
    expect(css).toContain(".public-admin-trend-grid");
    expect(css).toContain("max-height: min(58vh, 540px)");
  });

  it("keeps restoration inside the Agent visual language", () => {
    expect(startup).toContain("HONE 投资助手");
    expect(startup).not.toContain("HONE CONVERSATION");
    expect(startup).toContain('class="public-chat-startup-tabs"');
    expect(me).toContain("CONTENT.chat_page.me_page.loading_title");
  });

  it("supports complete dark workspace surfaces and compact empty states", () => {
    expect(css).toContain('[data-theme="dark"] .public-workspace-page');
    expect(css).toContain("--workspace-muted: #b8beb8");
    expect(css).toContain("min-height: 124px");
  });
});
