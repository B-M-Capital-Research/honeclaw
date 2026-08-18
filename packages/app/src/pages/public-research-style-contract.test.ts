import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const page = readFileSync(new URL("./public-research.tsx", import.meta.url), "utf8");
const css = readFileSync(new URL("./public-research.css", import.meta.url), "utf8");
const panelShell = readFileSync(
  new URL("../components/research/research-panel.tsx", import.meta.url),
  "utf8",
);
const shellCss = readFileSync(
  new URL("../components/research/research.css", import.meta.url),
  "utf8",
);
const foundation = readFileSync(
  new URL("./public-foundation.css", import.meta.url),
  "utf8",
);

describe("research desk contract", () => {
  it("is the URL-addressable home of every daily research product", () => {
    // One card per section; a section is either an in-place panel or a page.
    for (const key of [
      "daily-signal-macro",
      "daily-signal-ai",
      "company-ratings",
      "valuation-lab",
      "portfolio-news",
      "position-management",
      "influencer-digest",
      "weekly-brief",
      "key-event-chain",
      "research-library",
    ]) {
      expect(page).toContain(`"${key}"`);
    }
    // The open panel lives in the URL so it is shareable and the browser
    // back button closes it — never a plain boolean signal.
    expect(page).toContain("searchParams.panel");
    expect(page).toContain("setSearchParams({ panel: section.key })");
    expect(page).toContain("setSearchParams({ panel: undefined })");
  });

  it("parks valuation lab under the admin group until public rollout", () => {
    const block = page.slice(
      page.indexOf('key: "valuation-lab"'),
      page.indexOf('key: "portfolio-news"'),
    );
    expect(block).toContain('group: "admin"');
    expect(block).toContain("adminOnly: true");
    expect(page).toContain('{ key: "admin", label: "管理", adminOnly: true }');
  });

  it("paints the grid from one aggregate call and degrades to static cards", () => {
    expect(page).toContain("getPublicResearchOverview");
    // Two API touchpoints on the page itself (auth + overview), each named
    // once in the import and once at the call site; panels fetch on open.
    expect(page.split("getPublic").length - 1).toBeLessThanOrEqual(4);
    // Overview failure must not break navigation: cards keep static blurbs.
    expect(page).toContain("card()?.summary || section.blurb");
  });

  it("opens a panel and closes it — it never hands a question to chat", () => {
    // The panels are read-only views of a saved snapshot; the "发送到对话"
    // hand-off (a sessionStorage stash plus /chat?ask=research) is gone, and
    // the only prop the desk passes down is the close callback.
    expect(page).toContain("<Dynamic component={section().panel!} onClose={closePanel} />");
    expect(page).not.toContain("onAsk");
    expect(page).not.toContain("stashResearchAsk");
    expect(page).not.toContain("research-ask");
    expect(page).not.toContain("ask=research");
  });

  it("styles with foundation tokens only, including traffic-light semantics", () => {
    expect(foundation).toContain("--hone-signal-green:");
    expect(foundation).toContain("--hone-signal-yellow:");
    expect(foundation).toContain("--hone-signal-red:");
    for (const sheet of [css, shellCss]) {
      expect(sheet).toContain("var(--hone-");
      // No hex literals: the token layer owns every color on this surface.
      expect(sheet).not.toMatch(/#[0-9a-fA-F]{3,8}\b/);
      expect(sheet).not.toContain("!important");
    }
  });

  it("gives every panel a modal shell with escape, aria and scroll lock", () => {
    expect(panelShell).toContain('aria-modal="true"');
    expect(panelShell).toContain('role="dialog"');
    expect(panelShell).toContain('event.key === "Escape"');
    // The lock has to survive iOS Safari, which ignores overflow:hidden for
    // touch scrolling — the page is pinned and its offset restored on close.
    expect(panelShell).toContain('body.style.overflow = "hidden"');
    expect(panelShell).toContain('body.style.position = "fixed"');
    expect(panelShell).toContain("window.scrollTo(0, scrollY)");
    // Exactly one scroll container per panel, and it must not chain its
    // overscroll into the page behind the sheet.
    expect(shellCss).toContain("overscroll-behavior: contain");
    expect(shellCss).toContain("min-height: 0");
    expect(shellCss).toContain("max-height: 100dvh");
  });

  it("keeps the head to one band: kicker and close, with no refresh control", () => {
    const headTop = panelShell.indexOf('class="research-panel__head-top"');
    const close = panelShell.indexOf('class="research-panel__close"');
    const meta = panelShell.indexOf('class="research-panel__meta"');
    expect(headTop).toBeGreaterThan(-1);
    expect(close).toBeGreaterThan(headTop);
    expect(close).toBeLessThan(meta);
    // A panel reads one saved snapshot when it opens, so the head carries no
    // manual refresh: the `action` slot and its shared skin are both gone.
    expect(panelShell).not.toContain("props.action");
    expect(panelShell).not.toContain("action?:");
    expect(panelShell).not.toContain("research-panel__head-action");
    expect(shellCss).not.toContain("research-panel__head-action");
  });

  it("folds long source text into a disclosure rather than a dead ellipsis", () => {
    expect(panelShell).toContain("export function ResearchLongform");
    expect(panelShell).toContain("research-longform__preview");
    expect(panelShell).toContain("research-longform__full");
    // Two lines and an explicit way to read the rest — the clamp only ever
    // appears inside a <details> that can open.
    expect(shellCss).toContain(".research-longform__preview {");
    expect(shellCss).toContain("-webkit-line-clamp: 2");
    expect(shellCss).toContain('content: "展开原文"');
    expect(shellCss).toContain('content: "收起原文"');
  });

  it("gives every sideways row one scroller with a faded edge", () => {
    expect(shellCss).toContain(".research-scroller {");
    expect(shellCss).toContain("scrollbar-width: none");
    expect(shellCss).toContain("scroll-snap-type: x proximity");
    expect(shellCss).toContain("mask-image: linear-gradient(");
    expect(shellCss).toContain(".research-scroller > * {");
    expect(shellCss).toContain("flex: 0 0 auto");
    // Keyboard focus must never land inside the faded edge.
    expect(shellCss).toContain(".research-scroller:focus-within {");
  });

  it("prints MM-DD HH:mm and leaves the timezone to the head", () => {
    expect(panelShell).toContain("export function shortLocalTimestamp");
    expect(panelShell).toContain('.replace(/^\\d{4}[-/]/, "")');
  });
});
