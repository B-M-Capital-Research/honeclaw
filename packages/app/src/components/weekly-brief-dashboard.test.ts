import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(new URL("./weekly-brief-dashboard.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("./weekly-brief-dashboard.css", import.meta.url), "utf8");

/**
 * Source with comments stripped, so the "not contain" pins below can only be
 * satisfied by real code — a stale comment must not keep a contract green.
 */
const code = component.replace(/\/\*[\s\S]*?\*\//g, "").replace(/^\s*\/\/.*$/gm, "");
const headStart = code.indexOf("<ResearchPanelHead");
const panelHead = code.slice(headStart, code.indexOf("/>", headStart) + 2);
const propsStart = code.indexOf("type Props = {");
const propsType = code.slice(propsStart, code.indexOf("};", propsStart) + 2);

describe("weekly brief dashboard", () => {
  it("presents a standalone previous-week review and next-week agenda", () => {
    expect(component).toContain("周度简报");
    expect(component).toContain("上周重要事项");
    expect(component).toContain("下周重要事件点");
    expect(component).toContain("每周决策日程");
    expect(component).toContain("未来30天 AI");
    expect(component).toContain("重要 AI 公司财报与产业会议");
  });

  it("keeps schedules separate from confirmed outcomes", () => {
    expect(component).toContain("日程已发生 · 结果待核验");
    expect(component).toContain("未来日程 · 日期或调整");
  });

  it("uses structured readable agenda cards instead of an image", () => {
    expect(component).toContain("groupByDate");
    expect(component).toContain("weekly-brief-agenda");
    expect(component).not.toContain("<img");
    expect(component).toContain("weekly-brief-tabs");
    expect(component).toContain("activeView");
    // Sheet width belongs to `.research-panel` in research.css — this panel
    // used to declare its own on the same node, which is the collision the
    // shared shell exists to prevent.
    expect(styles).not.toContain("width: min(");
    expect(styles).toContain("@media (max-width: 800px)");
    expect(styles).not.toContain("@media(max-width:800px)");
  });

  it("stays a dated agenda rather than a social feed", () => {
    // These rows have no author and no authored text, and the reader's index
    // into them is the date — so the date column stays and ResearchFeedItem,
    // whose head is author + handle + time, is deliberately not used.
    expect(component).not.toContain("ResearchFeedItem");
    expect(component).toContain("weekly-brief-day");
    expect(component).toContain("shortDate(group.date)");
    expect(styles).toContain(".weekly-brief-day {");
  });

  it("prints each event's analysis in full instead of folding or clipping it", () => {
    // The reading of an event is two sentences; a disclosure around two
    // sentences is chrome, and its closed state clipped the first one.
    expect(component).toContain('<p class="weekly-brief-analysis">{item.analysis}</p>');
    expect(component).not.toContain("ResearchLongform");
    expect(styles).not.toContain("-webkit-line-clamp");
    // 提醒关注 is guidance, not a warning, so it lost its yellow box.
    expect(component).toContain("weekly-brief-attention");
    expect(component).not.toContain("<aside>");
    expect(styles).not.toContain(".weekly-brief-events aside");
  });

  it("collapses the three event badges and the tone chrome into one quiet line", () => {
    // One line: 重点 marker, evidence quality, category.
    expect(component).toContain("weekly-brief-event-meta");
    expect(component).toContain(
      "[evidenceLabel(item.evidence_status), categoryLabel(item.category)]",
    );
    // The category no longer paints a left bar, and the panel-level tone
    // kickers duplicated the tab labels above them.
    expect(component).not.toContain("data-category");
    expect(component).not.toContain("data-tone");
    expect(component).not.toContain("kicker=\"发生了什么变化\"");
    expect(styles).not.toContain("data-category");
    expect(styles).not.toContain("data-tone");
    expect(styles).not.toContain("article:before");
  });

  it("opens on the verdict through the shared head, not a header plus meta strip plus hero", () => {
    expect(component).toContain("ResearchPanelHead");
    // The head id has to stay the dialog's accessible name.
    expect(component).toContain('id="weekly-brief-title"');
    expect(component).toContain('labelledBy="weekly-brief-title"');
    expect(component).toContain('kicker="每周决策日程"');
    expect(component).toContain('title="周度简报"');
    // Conclusion first: next week's load, the coverage light, one sentence.
    expect(component).toContain("headline={report() ? `下周 ${report()!.next_week_items.length} 件`");
    expect(component).toContain("statusSignal");
    expect(component).toContain('"财报覆盖待补齐"');
    expect(component).toContain("summary={report()?.summary}");
    // Report day, generation clock and tracked scope are one secondary line;
    // the metadata strip and the hero card that repeated them are gone.
    expect(component).toContain("meta={metaLine()}");
    expect(component).toContain("跟踪 ${current.earnings_scope_count} 家公司");
    expect(component).not.toContain("weekly-brief-meta");
    expect(component).not.toContain("weekly-brief-hero");
    expect(component).not.toContain("weekly-brief-dialog-head");
    expect(styles).not.toContain("weekly-brief-meta");
    expect(styles).not.toContain("weekly-brief-hero");
    expect(styles).not.toContain("weekly-brief-dialog-head");
    // Coverage gaps keep their specific detail but not a second status label.
    expect(component).not.toContain("财报覆盖未完全就绪");
  });

  it("keeps methodology inside the scroll container instead of fixed chrome", () => {
    const method = component.indexOf('class="weekly-brief-method"');
    const contentOpen = component.indexOf('<main class="weekly-brief-content">');
    const contentClose = component.indexOf("</main>");
    expect(contentOpen).toBeGreaterThan(-1);
    expect(method).toBeGreaterThan(contentOpen);
    expect(method).toBeLessThan(contentClose);
    expect(styles).not.toContain(".weekly-brief-method {\n  flex: 0 0 auto;");
  });

  it("labels official AI dates without presenting unannounced dates as facts", () => {
    expect(component).toContain("官网已确认");
    expect(component).toContain("缺失日期不会被猜测补全");
    expect(component).toContain("ai_outlook_items");
  });

  it("is a controlled research panel that loads only when opened", () => {
    expect(component).toContain("export function WeeklyBriefPanel");
    expect(component).not.toContain("WeeklyBriefDashboard");
    expect(component).toContain("ResearchPanel");
    expect(component).toContain('backdropClass="weekly-brief-backdrop"');
    expect(component).toContain('dialogClass="weekly-brief-dialog"');
    expect(component).toContain("onClose={props.onClose}");
    // Mount == open, so mounting keeps the old "load on open" semantics.
    expect(component).toContain("onMount(() => void load())");
    // The shared shell owns Portal / backdrop / Escape / aria-modal.
    expect(component).not.toContain("Portal");
    expect(component).not.toContain("aria-modal");
    expect(component).not.toContain("weekly-brief-launcher");
    expect(component).not.toContain("setOpen");
    expect(styles).not.toContain("weekly-brief-launcher");
    // Loading / error / empty go through the shared state component.
    expect(component).toContain("ResearchState");
    expect(component).toContain("onRetry={() => void load()}");
  });

  it("drops the decorative amber theme for coral, keeping yellow for warnings only", () => {
    // Decorative amber is deprecated: no amber literals anywhere.
    expect(styles).not.toContain("#d7a82d");
    expect(styles).not.toContain("#9a720d");
    expect(styles).not.toContain("#b68108");
    expect(styles).not.toContain("#fff0c6");
    expect(styles).not.toContain("#fff5d8");
    expect(styles).toContain("var(--hone-coral-500)");
    expect(styles).toContain("var(--hone-coral-600)");
    // Yellow survives only as the warning semantic token.
    expect(styles).toContain("var(--hone-signal-yellow)");
    expect(styles).toContain("var(--hone-signal-yellow-soft)");
    expect(styles).toContain("var(--hone-signal-green)");
    // Dark mode rides the tokens end to end: the purple ticker chip and the
    // purple earnings bar were the last literals, and both are gone.
    expect(styles).not.toContain('[data-theme="dark"]');
    expect(styles).not.toContain("#111a28");
    expect(styles).not.toContain("#7a67d8");
    expect(styles).not.toContain("#765fc0");
    expect(styles).not.toContain("#e9e5ff");
    expect(styles).not.toContain("#5141a8");
    expect(styles).not.toContain("#28243d");
    expect(styles).not.toContain("#c1b5ff");
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
