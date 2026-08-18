import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(
  new URL("./influencer-digest-dashboard.tsx", import.meta.url),
  "utf8",
);
const styles = readFileSync(
  new URL("./influencer-digest-dashboard.css", import.meta.url),
  "utf8",
);
const shellStyles = readFileSync(new URL("./research/research.css", import.meta.url), "utf8");

const feed = readFileSync(
  new URL("./research/research-feed.tsx", import.meta.url),
  "utf8",
);
const feedStyles = readFileSync(
  new URL("./research/research-feed.css", import.meta.url),
  "utf8",
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

describe("influencer digest dashboard", () => {
  it("keeps source and opinion boundaries", () => {
    expect(component).toContain("大V速报");
    expect(component).toContain("查看作者原文");
    expect(component).toContain("翻译/聚合源");
    // Opinion never reads as fact: stance, fact/opinion split and the
    // counterpoint are printed on the card itself.
    expect(component).toContain("stanceLabel(item.stance)");
    expect(component).toContain("反方 / 未证实处：");
    expect(component).toContain("未配置来源不会被补造内容");
  });

  it("is cached and scheduled", () => {
    expect(component).toContain("每日 19:50 更新");
    expect(component).toContain("getPublicInfluencerDigest");
    expect(component).not.toContain("生成速报");
  });

  it("is a controlled research panel without its own launcher or modal chrome", () => {
    expect(component).toContain("export function InfluencerDigestPanel");
    expect(component).not.toContain("InfluencerDigestDashboard");
    expect(component).toContain("ResearchPanel");
    expect(component).toContain('backdropClass="influencer-digest-backdrop"');
    expect(component).toContain('dialogClass="influencer-digest-dialog"');
    expect(component).toContain("onClose={props.onClose}");
    // The shared shell owns Portal / backdrop / Escape / aria-modal.
    expect(component).not.toContain("Portal");
    expect(component).not.toContain("aria-modal");
    expect(component).not.toContain("influencer-digest-launcher");
    expect(component).not.toContain("setOpen");
    expect(styles).not.toContain("influencer-digest-launcher");
  });

  it("opens on the verdict through the shared head, not a bespoke header plus meta strip", () => {
    expect(component).toContain("ResearchPanelHead");
    // The head id has to stay the dialog's accessible name.
    expect(component).toContain('id="influencer-title"');
    expect(component).toContain('labelledBy="influencer-title"');
    expect(component).toContain('kicker="先看来源，再看观点"');
    expect(component).toContain('title="大V速报"');
    // Conclusion first: how many posts, and whether sources actually ran.
    expect(component).toContain("headline={snapshot() ? `${visible().length} 条原文`");
    expect(component).toContain("statusSignal");
    expect(component).toContain("summary={snapshot()?.summary}");
    // Provenance is one secondary line, no longer its own strip.
    expect(component).toContain("meta={metaLine()}");
    expect(component).not.toContain("influencer-digest-meta");
    expect(styles).not.toContain("influencer-digest-meta");
    expect(styles).not.toContain(".influencer-digest-dialog header");
    // The summary must not be printed a second time in the body.
    expect(component).not.toContain("detail={snapshot()?.summary}");
  });

  it("scrolls the author filter through the shared scroller", () => {
    // Overflow, the right-edge fade and the snap belong to one shared class,
    // so the row never cuts an author chip in half at the container edge.
    expect(component).toContain('class="influencer-authors research-scroller"');
    expect(styles).toContain(".influencer-authors {");
    expect(styles).not.toContain("flex-wrap: nowrap");
    expect(styles).not.toContain("overflow-x: auto");
    expect(shellStyles).toContain(".research-scroller {");
    expect(shellStyles).toContain("scrollbar-width: none");
    expect(shellStyles).toContain("scroll-snap-type: x proximity");
    // The shared shell owns the panel's only scroll container.
    expect(styles).toContain(".influencer-digest-body {");
  });

  it("keeps two judgement chips per post and drops the repeated timezone", () => {
    // Stance and fact-vs-opinion are the calls; topics and tickers are index
    // terms and read as one quiet line, not a dozen equal-weight chips.
    expect(component).toContain("stanceLabel(item.stance)");
    expect(component).toContain('item.content_type === "fact"');
    expect(component).not.toContain("<For each={item.topics}>{(topic) => <span>{topic}</span>}</For>");
    expect(component).not.toContain("<For each={item.tickers}>{(ticker) => <span>${ticker}</span>}</For>");
    // The head's meta line already prints the run timezone once.
    expect(component).toContain("shortLocalTimestamp(item.published_at_local)");
    expect(component).not.toContain("{item.published_at_local} {snapshot()?.timezone}");
  });

  it("routes loading, error and empty states through the shared research kit", () => {
    expect(component).toContain("ResearchState");
    expect(component).toContain('kind="error"');
    expect(component).toContain("onRetry={() => void load()}");
  });

  it("leads with the author's own words and folds our reading away", () => {
    // The feed reads like a feed: the post is the body, never a disclosure,
    // and it is not preceded by a restated title or by our digest. Everything
    // HONE inferred sits behind one line.
    expect(component).toContain("<ResearchFeed>");
    expect(component).toContain("<ResearchFeedItem");
    expect(component).toContain("{sourceText(item) || item.title}");
    expect(component).not.toContain("<summary>作者原文</summary>");
    expect(component).not.toContain("HONE 摘要");
    expect(component).toContain('analysisLabel="HONE 解读"');
    expect(component).toContain("English original");
    // Full text, not the 600-char excerpt: translation first, English when a
    // post was never translated, excerpt only for pre-full-text snapshots.
    expect(component).toContain("item.source_text_cn");
    expect(component).toContain("item.source_text_en");
    expect(component).toContain("item.source_excerpt");
    expect(component).toContain("englishOriginal");
    expect(feedStyles).toContain("white-space: pre-wrap");
  });

  it("renders post images and the replied-to context", () => {
    expect(component).toContain("media_urls");
    expect(component).toContain('item.post_kind === "quote" ? "引用" : "回复"');
    // Lazy loading is deliberately absent: the panel pins the page for its
    // iOS-safe scroll lock, and Chrome then never resolves the intersection
    // for images inside the portal, so they would sit visible and unloaded.
    // Asserted on the element, since the rationale comment names the attribute.
    const imgTag = feed.slice(feed.indexOf("<img"), feed.indexOf("/>", feed.indexOf("<img")));
    expect(imgTag).not.toContain("loading");
    expect(feed).toContain('referrerpolicy="no-referrer"');
    expect(feedStyles).toContain(".research-feed-item__media {");
    expect(feedStyles).toContain("aspect-ratio: 16 / 10");
    expect(feedStyles).toContain("border-radius: var(--hone-radius-sm)");
  });

  it("uses design tokens in readable multi-line CSS for mobile and dark", () => {
    expect(styles).not.toContain("!important");
    expect(styles).toContain("@media (max-width: 768px)");
    expect(styles).not.toContain("@media(max-width:768px)");
    expect(styles).toContain("var(--hone-paper-50)");
    expect(styles).toContain("var(--hone-coral-600)");
    expect(styles).toContain("var(--hone-signal-yellow-soft)");
    expect(styles).toContain("var(--hone-line)");
    // Dark mode rides the tokens; the old slate dark skin is gone.
    expect(styles).not.toContain("#111a28");
    expect(styles).not.toContain("[data-theme=dark]");
    // Mapped literals must not resurface.
    expect(styles).not.toContain("#c35b46");
    expect(styles).not.toContain("#b65340");
    expect(styles).not.toContain("#fff4d8");
    expect(styles).not.toContain("#fff6e3");
  });

  it("has no manual refresh button and no ask-the-chat footer", () => {
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
