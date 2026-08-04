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
    expect(shell).toContain("<PublicPushCenter");
    expect(shell).toContain("getPublicChatBootstrap");
    expect(shell).toContain("publicWorkspaceResearchFromHistory");
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
    expect(me).toContain("<Show when={props.user.is_admin}>");
    expect(me).toContain("<PublicAdminUsagePanel />");
    expect(me).toContain("<PublicAdminWhitelistPanel />");
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
    expect(startup).toContain("HONE AGENT");
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
