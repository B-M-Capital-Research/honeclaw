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
const portfolio = readFileSync(new URL("./public-portfolio.tsx", import.meta.url), "utf8");
const me = readFileSync(new URL("./public-me.tsx", import.meta.url), "utf8");

describe("public workspace page contract", () => {
  it("shares one desktop and mobile chrome across insights, tracking, and account", () => {
    expect(shell).toContain("<AgentWorkspaceSidebar");
    expect(shell).toContain("<AgentWorkspaceMobileHeader");
    expect(shell).toContain("<AgentWorkspaceMobileNav");
    expect(community).toContain('<PublicWorkspaceShell\n          active="insights"');
    expect(portfolio).toContain('<PublicWorkspaceShell active="tracking"');
    expect(me).toContain('<PublicWorkspaceShell active="me"');
  });

  it("uses a continuous insight stream and separate desktop/mobile tracking views", () => {
    expect(css).toContain("one continuous editorial stream");
    expect(css).toContain(".public-community-card:last-of-type");
    expect(portfolio).toContain("<TrackingCalendar view={trackingView()} />");
    expect(portfolio).toContain('setTrackingView("today")');
    expect(portfolio).toContain('setTrackingView("tasks")');
    expect(portfolio).toContain('setTrackingView("history")');
    expect(css).toContain(".public-tracking-weekdays,.public-tracking-grid { display: none; }");
    expect(css).toContain(".public-tracking-agenda { display: grid; }");
  });

  it("keeps restoration inside the Agent visual language", () => {
    expect(startup).toContain("HONE AGENT");
    expect(startup).not.toContain("HONE CONVERSATION");
    expect(startup).toContain('class="public-chat-startup-tabs"');
    expect(me).toContain("正在加载个人空间");
    expect(portfolio).toContain("正在加载跟踪");
  });
});
