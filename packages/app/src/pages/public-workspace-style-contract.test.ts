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
    expect(me).toContain("<PublicAdminWhitelistPanel />");
    expect(css).toContain(".public-admin-panel");
    expect(css).toContain(".public-admin-table td::before");
  });

  it("keeps restoration inside the Agent visual language", () => {
    expect(startup).toContain("HONE AGENT");
    expect(startup).not.toContain("HONE CONVERSATION");
    expect(startup).toContain('class="public-chat-startup-tabs"');
    expect(me).toContain("正在加载个人空间");
  });

  it("supports complete dark workspace surfaces and compact empty states", () => {
    expect(css).toContain('[data-theme="dark"] .public-workspace-page');
    expect(css).toContain("--workspace-muted: #b8beb8");
    expect(css).toContain("min-height: 124px");
  });
});
