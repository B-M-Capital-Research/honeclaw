import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(new URL("./portfolio-news-dashboard.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("./portfolio-news-dashboard.css", import.meta.url), "utf8");
const feed = readFileSync(new URL("./research/research-feed.tsx", import.meta.url), "utf8");
const feedStyles = readFileSync(new URL("./research/research-feed.css", import.meta.url), "utf8");

/** The one feed item, from its opening tag to its close. */
const feedItem = () =>
  component.slice(
    component.indexOf("<ResearchFeedItem"),
    component.indexOf("</ResearchFeedItem>"),
  );

/**
 * Source with comments stripped, so the "not contain" pins below can only be
 * satisfied by real code — a stale comment must not keep a contract green.
 */
const code = component.replace(/\/\*[\s\S]*?\*\//g, "").replace(/^\s*\/\/.*$/gm, "");
const headStart = code.indexOf("<ResearchPanelHead");
const panelHead = code.slice(headStart, code.indexOf("/>", headStart) + 2);
const propsStart = code.indexOf("type Props = {");
const propsType = code.slice(propsStart, code.indexOf("};", propsStart) + 2);

describe("portfolio news dashboard contract", () => {
  test("exposes the cached report as a controlled research panel", () => {
    expect(component).toContain("export function PortfolioNewsPanel");
    expect(component).toContain("<ResearchPanel");
    expect(component).toContain('dialogClass="portfolio-news-dialog"');
    expect(component).toContain('backdropClass="portfolio-news-backdrop"');
    expect(component).toContain("持仓重点新闻分析");
    expect(component).toContain("近 48 小时可信来源");
    expect(component).toContain("每日 20:00 更新");
    expect(component).toContain("getPublicPortfolioNews");
    expect(component).not.toContain("重新生成");
    // The launcher chip moved to the research desk; the panel owns no trigger.
    expect(component).not.toContain("portfolio-news-launcher");
    expect(component).not.toContain('from "solid-js/web"');
  });

  test("leads with the verdict instead of a title-then-metadata preamble", () => {
    // Count, status light and one sentence sit in the shared head; the old
    // bespoke header and the metadata strip under it are gone, and the count
    // tiles no longer repeat the total or the summary the head carries.
    expect(component).toContain("ResearchPanelHead");
    expect(component).toContain('id="portfolio-news-title"');
    expect(component).toContain("provenanceLine");
    expect(component).not.toContain("portfolio-news-meta");
    expect(component).not.toContain("<span>重点新闻</span>");
    expect(styles).not.toContain(".portfolio-news-meta");
    expect(styles).not.toContain("> header");
  });

  test("delegates modal behaviour and states to the shared research shell", () => {
    // ESC / aria-modal / scroll lock live in ResearchPanel, not here.
    expect(component).not.toContain('aria-modal="true"');
    expect(component).not.toContain('"Escape"');
    expect(component).toContain("<ResearchState");
    expect(component).toContain('kind="loading"');
    expect(component).toContain('kind="error"');
    expect(component).toContain("onRetry={() => void load()}");
    expect(component).toContain('kind="empty"');
  });

  test("reads each item as a feed post, the news itself never folded", () => {
    expect(component).toContain("<ResearchFeed>");
    expect(component).toContain("<ResearchFeedItem");
    // Header carries source facts only: which holding, who published it, when.
    expect(component).toContain("author={item.symbol}");
    expect(component).toContain("handle={item.source}");
    expect(component).toContain("time={shortLocalTimestamp(item.published_at_local)}");
    // The news in the source's own words is the body — open, whole, original
    // line breaks — not our 60-character digest inside a disclosure.
    expect(component).toContain("<p>{newsText(item)}</p>");
    expect(component).toContain("item.source_summary");
    expect(component).not.toContain("ResearchLongform");
    expect(component).not.toContain("<h3>{item.title}</h3>");
    expect(feedStyles).toContain("white-space: pre-wrap");
    expect(feedItem().slice(feedItem().indexOf("<p>{newsText(item)}</p>"))).not.toContain(
      "<details",
    );
    // One left edge for the impact call; the chip row and card skin are gone.
    expect(component).toContain("accent={impactAccent(item.impact)}");
    expect(feedStyles).toContain(".research-feed-item.is-green");
    expect(feedStyles).toContain(".research-feed-item.is-red");
    expect(styles).not.toContain("portfolio-news-item__");
    expect(styles).not.toContain(".portfolio-news-list");
  });

  test("folds every HONE judgement behind one line", () => {
    const item = feedItem();
    const analysis = item.slice(item.indexOf("analysis={"), item.indexOf("<p>{newsText"));
    for (const judgement of [
      "IMPACT_LABEL[item.impact]",
      "HORIZON_LABEL[item.horizon]",
      "THESIS_LABEL[item.thesis_effect]",
      "item.summary",
      "item.why_it_matters",
      "item.attention",
      "CONFIDENCE_LABEL[item.confidence]",
    ]) {
      expect(analysis).toContain(judgement);
    }
    expect(component).toContain('analysisLabel="HONE 解读"');
    expect(feed).toContain('<details class="research-feed-item__analysis">');
    // Nothing to fold when the model has not run on an item.
    expect(component).toContain('item.analysis_status === "model_analyzed" ?');
    // The source link stays outside the fold: it is a source fact.
    expect(component).toContain("href: item.source_url");
  });

  test("keeps provenance, analysis status and fail-closed states visible", () => {
    expect(component).toContain("item.source_url");
    expect(component).toContain("published_at_local");
    expect(component).toContain("unassessed");
    expect(component).toContain("等待新闻数据源");
    expect(component).toContain("待模型分析");
  });

  test("uses hone design tokens with dark and mobile layouts", () => {
    expect(styles).toContain("var(--hone-ink-950)");
    expect(styles).toContain("var(--hone-ink-500)");
    expect(styles).toContain("var(--hone-line)");
    expect(styles).toContain("var(--hone-signal-green)");
    expect(styles).toContain("var(--hone-signal-red)");
    expect(styles).toContain("var(--hone-signal-orange)");
    // Action tokens: dark-mode coral is a light peach, so white type on it
    // would be unreadable.
    expect(styles).toContain("var(--hone-action-bg)");
    expect(styles).toContain("var(--hone-action-fg)");
    expect(styles).not.toContain("!important");
    expect(styles).not.toContain("#d3544a");
    expect(styles).not.toContain("#23845a");
    expect(styles).not.toContain("#d69c18");
    expect(styles).not.toContain("portfolio-news-launcher");
    // Dark mode rides the tokens; the bespoke slate override block is gone.
    expect(styles).not.toContain('[data-theme="dark"]');
    expect(styles).not.toContain("#111a28");
    expect(styles).toContain("@media (max-width: 768px)");
    // Sheet mode: chrome between the head and the first article is squeezed.
    expect(styles).toContain("@media (max-width: 760px)");
    // Per-item impact colors are the shared feed's accent, token-backed there.
    expect(feedStyles).toContain("var(--hone-signal-red)");
    expect(feedStyles).toContain("var(--hone-signal-neutral)");
  });

  test("has no manual refresh button and no ask-the-chat footer", () => {
    // The head is kicker + close. Panels read one saved snapshot when they
    // open, so there is no manual refresh control to press and no stale
    // "read again" affordance implying the numbers can be recomputed here.
    expect(panelHead).not.toContain("action=");
    expect(code).not.toContain("重新读取");
    expect(code).not.toContain("读取中…");
    // 「发送到对话」is gone end to end: no composer in the footer, no prompt
    // envelope, and no chat sink on the props.
    expect(propsType).toBe("type Props = {\n  onClose: () => void;\n};");
    expect(code).not.toContain("onAsk");
    expect(code).not.toContain("发送到对话");
    expect(code).not.toContain("buildSavedReportPrompt");
    expect(code).not.toContain("HONE_SAVED");
    // load() survives for exactly two callers: first paint and error retry.
    expect(code).toContain("onMount(() => void load())");
  });
});
