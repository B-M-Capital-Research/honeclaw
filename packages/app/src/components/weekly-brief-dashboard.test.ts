import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(new URL("./weekly-brief-dashboard.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("./weekly-brief-dashboard.css", import.meta.url), "utf8");

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
    expect(component).toContain("未来日程不是预测");
    expect(component).toContain("过去日程也不能据此补造公布值");
  });

  it("uses structured readable agenda cards instead of an image", () => {
    expect(component).toContain("groupByDate");
    expect(component).toContain("weekly-brief-agenda");
    expect(component).not.toContain("<img");
    expect(component).toContain("weekly-brief-tabs");
    expect(component).toContain("activeView");
    expect(styles).toContain("width: min(1040px, 100%)");
    expect(styles).toContain("@media (max-width: 800px)");
    expect(styles).not.toContain("@media(max-width:800px)");
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

  it("passes a bounded saved report into follow-up chat", () => {
    expect(component).toContain("buildSavedReportPrompt");
    expect(component).toContain('marker: "HONE_SAVED_WEEKLY_BRIEF"');
    expect(component).toContain("lastWeekItems");
    expect(component).toContain("nextWeekItems");
    expect(component).toContain("aiOutlookItems");
    expect(component).toContain("优先核对公司 IR、监管文件和官方数据");
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
    expect(component).toContain("props.onClose()");
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
    expect(component).toContain("<Show when={props.onAsk}>");
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
    // Dark mode rides the tokens; only the purple ticker chip keeps a skin.
    expect(styles.match(/\[data-theme="dark"\]/g)?.length).toBe(1);
    expect(styles).not.toContain("#111a28");
  });
});
