import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(new URL("./key-event-chain-dashboard.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("./key-event-chain-dashboard.css", import.meta.url), "utf8");
const shellStyles = readFileSync(
  new URL("./research/research.css", import.meta.url),
  "utf8",
);
const feed = readFileSync(new URL("./research/research-feed.tsx", import.meta.url), "utf8");
const feedStyles = readFileSync(new URL("./research/research-feed.css", import.meta.url), "utf8");

/** The one feed item, from its opening tag to its close. */
const feedItem = () =>
  component.slice(
    component.indexOf("<ResearchFeedItem"),
    component.indexOf("</ResearchFeedItem>"),
  );

describe("key event chain dashboard", () => {
  it("renders the first-principles industry chain", () => {
    expect(component).toContain("关键事件链");
    // The layer chain is the topic map's accessible name now that the head
    // spends its space on the verdict instead of a decorative subtitle.
    expect(component).toContain(
      'aria-label="产业主线：模型 → 应用 → 数据中心 → 算力 → 光互连 → 存储 → 电力"',
    );
    expect(component).toContain("第一性原理");
    expect(component).toContain("下一验证点");
  });

  it("separates confirmed milestones from clues", () => {
    expect(component).toContain("只看一手确认");
    expect(component).toContain('verification_status === "confirmed"');
    expect(component).toContain("查看一手原文");
    expect(component).toContain("查看线索原文");
    // One left edge carries the state — green for a first-hand confirmation,
    // yellow for a clue — instead of a chip row per event.
    expect(component).toContain(
      'accent={event.verification_status === "confirmed" ? "green" : "yellow"}',
    );
    expect(feedStyles).toContain(".research-feed-item.is-green");
    expect(feedStyles).toContain(".research-feed-item.is-yellow");
    // The label itself stays readable as text inside the fold, so the state is
    // never carried by color alone.
    expect(feedItem()).toContain("verificationLabel(event.verification_status)");
    // The bespoke verification / direction chips are gone with the timeline.
    expect(styles).not.toContain("data-verification");
    expect(styles).not.toContain("data-direction");
  });

  it("opens on the verdict through the shared head, not a bespoke header plus meta strip", () => {
    expect(component).toContain("ResearchPanelHead");
    // The head id has to stay the dialog's accessible name.
    expect(component).toContain('id="key-chain-title"');
    expect(component).toContain('labelledBy="key-chain-title"');
    expect(component).toContain('kicker="第一性原理产业图谱"');
    expect(component).toContain('title="关键事件链"');
    // Conclusion first: how much moved, how much of it is first-hand.
    expect(component).toContain("headline={snapshot() ? `${totals().events} 条变化`");
    expect(component).toContain("一手确认 ${totals().confirmed}");
    expect(component).toContain("statusSignal");
    expect(component).toContain("summary={snapshot()?.summary}");
    // Report day, lookback, refresh clock and timezone are one secondary line.
    expect(component).toContain("meta={metaLine()}");
    expect(component).toContain("每日 19:55 更新");
    expect(component).not.toContain("key-chain-meta");
    expect(styles).not.toContain("key-chain-meta");
    expect(styles).not.toContain(".key-chain-dialog header");
  });

  it("scrolls the topic map and the evidence filter through the shared scroller", () => {
    // Overflow, the right-edge fade and the snap belong to one shared class,
    // so a sideways row never cuts a card in half at the container edge.
    expect(component).toContain('class="key-chain-topics research-scroller"');
    expect(component).toContain('class="key-chain-evidence-filter research-scroller"');
    expect(styles).toContain(".key-chain-topics {");
    // The panel must not re-declare what the shared class already owns.
    expect(styles).not.toContain("flex-wrap: nowrap");
    expect(styles).not.toContain("overflow-x: auto");
    expect(shellStyles).toContain(".research-scroller {");
    expect(shellStyles).toContain("scrollbar-width: none");
    expect(shellStyles).toContain("scroll-snap-type: x proximity");
    expect(shellStyles).toContain("mask-image: linear-gradient(");
    expect(shellStyles).toContain("flex: 0 0 auto");
    // The shared shell owns the panel's only scroll container.
    expect(styles).toContain(".key-chain-body {");
  });

  it("reads each change as a feed post, the event itself never folded", () => {
    expect(component).toContain("<ResearchFeed>");
    expect(component).toContain("<ResearchFeedItem");
    // Byline: who published it, what kind of change, which way it cuts. The
    // chip row and the timeline rail that carried these are gone.
    expect(component).toContain("author={event.source_name}");
    expect(component).toContain(
      "meta={[changeLabel(event.change_type), directionLabel(event.direction)]}",
    );
    expect(component).toContain("time={shortLocalTimestamp(event.published_at_local)}");
    expect(component).not.toContain("{event.published_at_local} {snapshot()?.timezone}");
    // The source's own text is the body: open, whole, original line breaks.
    expect(component).toContain("<p>{eventText(event)}</p>");
    expect(component).toContain("const eventText");
    expect(feedStyles).toContain("white-space: pre-wrap");
    // Not folded, not clamped, and not preceded by a restated headline.
    expect(component).not.toContain("ResearchLongform");
    expect(component).not.toContain("<h4>{event.title}</h4>");
    expect(feedItem().slice(feedItem().indexOf("<p>{eventText(event)}</p>"))).not.toContain(
      "<details",
    );
    expect(styles).not.toContain("key-chain-timeline");
    expect(styles).not.toContain("key-chain-event-meta");
  });

  it("folds every HONE inference behind one line", () => {
    const item = feedItem();
    // Everything the model derived sits inside `analysis`, which the shared
    // feed renders as a closed <details>.
    const analysis = item.slice(item.indexOf("analysis={"), item.indexOf("<p>{eventText"));
    for (const inference of [
      "证据口径",
      "影响：",
      "下一验证点：",
      "sourceTierLabel(event.source_tier)",
      "event.tickers.map((ticker) => `$${ticker}`)",
    ]) {
      expect(analysis).toContain(inference);
    }
    expect(component).toContain('analysisLabel="HONE 解读"');
    expect(feed).toContain('<details class="research-feed-item__analysis">');
    // Ticker chips and the standalone impact / next-watch blocks are gone.
    expect(component).not.toContain("key-chain-tickers");
    expect(styles).not.toContain("key-chain-tickers");
    expect(styles).not.toContain("key-chain-verification");
    // The source link stays outside the fold: it is a source fact.
    expect(component).toContain("href: event.source_url");
  });

  it("preserves evidence and action boundaries", () => {
    expect(component).toContain("聚合翻译和管理员研究资料不是一手事实");
    expect(component).toContain("影响待验证时不得补造结论");
    expect(component).toContain("buildSavedReportPrompt");
    expect(component).toContain('marker: "HONE_SAVED_KEY_EVENT_CHAIN"');
  });

  it("keeps the timeline focused after the weekly brief is extracted", () => {
    expect(component).not.toContain("过去10日复盘");
    expect(component).not.toContain("未来10日验证问题");
    expect(component).not.toContain("tenDayBrief");
  });

  it("is a controlled research panel without its own launcher or modal chrome", () => {
    expect(component).toContain("export function KeyEventChainPanel");
    expect(component).not.toContain("KeyEventChainDashboard");
    expect(component).toContain("ResearchPanel");
    expect(component).toContain('backdropClass="key-chain-backdrop"');
    expect(component).toContain('dialogClass="key-chain-dialog"');
    expect(component).toContain("props.onClose()");
    // The shared shell owns Portal / backdrop / Escape / aria-modal.
    expect(component).not.toContain("Portal");
    expect(component).not.toContain("aria-modal");
    expect(component).not.toContain("key-chain-launcher");
    expect(component).not.toContain("setOpen");
    expect(styles).not.toContain("key-chain-launcher");
    // Loading / error / empty go through the shared state component.
    expect(component).toContain("ResearchState");
    expect(component).toContain("onRetry={() => void load()}");
    expect(component).toContain("<Show when={props.onAsk}>");
  });

  it("uses traffic-light tokens in readable multi-line CSS for mobile and dark", () => {
    expect(styles).toContain("@media (max-width: 768px)");
    expect(styles).not.toContain("@media(max-width:768px)");
    expect(styles).not.toContain("!important");
    expect(styles).toContain("var(--hone-signal-green)");
    expect(styles).toContain("var(--hone-signal-green-soft)");
    expect(styles).toContain("var(--hone-paper-50)");
    expect(styles).toContain("var(--hone-line)");
    // The per-event traffic light moved to the shared feed accent, which is
    // token-backed there rather than re-declared here.
    expect(feedStyles).toContain("var(--hone-signal-green)");
    expect(feedStyles).toContain("var(--hone-signal-yellow)");
    expect(feedStyles).toContain("var(--hone-signal-red)");
    // Dark mode rides the tokens; the old slate dark skin is gone.
    expect(styles).not.toContain("[data-theme=dark]");
    expect(styles).not.toContain("#111a28");
    // Mapped literals must not resurface.
    expect(styles).not.toContain("#28745b");
    expect(styles).not.toContain("#966d19");
    expect(styles).not.toContain("#f8f0dd");
  });
});
