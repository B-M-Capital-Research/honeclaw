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
const dataCenter = readFileSync(
  new URL("./public-data-center.tsx", import.meta.url),
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

  it("releases the commentator digest to every reader under its own group", () => {
    // Sliced to the next section rather than to a named neighbour: sections
    // get reordered, and a slice anchored on one must not silently invert.
    const start = page.indexOf('key: "influencer-digest"');
    const block = page.slice(start, page.indexOf('key: "', start + 20));
    expect(start).toBeGreaterThan(-1);
    expect(block).toContain('group: "voices"');
    expect(block).not.toContain("adminOnly");
    expect(page).toContain('{ key: "voices", label: "大V观点" }');
    // A snapshot whose model did not run still carries its sources, so it is
    // a finding, not an empty day.
    expect(page).not.toContain('"source_only"');
  });

  it("keeps the industry ontology admin-only, on the desk and on the way in", () => {
    // The ontology is research draft — transmission chains, multiple anchors,
    // upstream signals. The server refuses it to non-administrators, so the
    // desk entry must not advertise a page a reader cannot open.
    const start = page.indexOf('key: "industry-map"');
    expect(start).toBeGreaterThan(-1);
    expect(page.slice(start, page.indexOf('key: "', start + 20))).toContain("adminOnly: true");
    // The 3D scene stays public, so it must not hand a reader a door that
    // answers 403: every way it offers into the ontology sits behind the
    // administrator gate, checked by looking at what precedes each link.
    expect(dataCenter).toContain("user()?.is_admin === true");
    const ways = [...dataCenter.matchAll(/\/industry-map|industryHref\(/g)].map((m) => m.index ?? 0);
    expect(ways.length).toBeGreaterThanOrEqual(2);
    for (const at of ways) {
      const lead = dataCenter.slice(Math.max(0, at - 400), at);
      expect(lead.includes("<Show when={isAdmin()}>") || lead.includes("import")).toBe(true);
    }
  });

  it("explains itself and gives administrators the reader's view", () => {
    // Every section states what it is and how to read it; the desk used to
    // assume the reader already knew what a 关键事件链 was for.
    for (const key of ["what:", "howTo:", "question:"]) {
      expect(page).toContain(key);
    }
    expect(page).toContain("public-research-what");
    expect(page).toContain("研究台是什么、怎么用");
    expect(css).toContain(".public-research-guide {");
    // The 问 HONE hand-off is a plain navigation with the question in the
    // URL, sent on arrival — no stash, no second click.
    expect(page).toContain("/chat?q=${encodeURIComponent(section.question)}&send=1");
    expect(page).toContain("public-research-tochat");
    // Administrators can switch to exactly what a reader sees; the choice
    // lives on the device, never in the URL a reader could be handed.
    expect(page).toContain("viewAsUser");
    expect(page).toContain("管理员视角");
    expect(page).toContain("用户视角");
    expect(page).toContain("const adminView = createMemo(() => isAdmin() && !viewAsUser())");
    expect(page).toContain("!item.adminOnly || adminView()");
    expect(page).toContain("!section.adminOnly || adminView()");
    expect(page).toContain('activeGroup() === "admin" && adminView()');
    expect(css).toContain(".public-research-view {");
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
