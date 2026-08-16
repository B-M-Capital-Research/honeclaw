import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(new URL("./key-event-chain-dashboard.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("./key-event-chain-dashboard.css", import.meta.url), "utf8");

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
    expect(styles).toContain('data-verification="confirmed"');
    expect(styles).toContain('data-verification="clue"');
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

  it("keeps the topic map a single scrollable row on phones", () => {
    expect(styles).toContain(".key-chain-topics {");
    expect(styles).toContain("flex-wrap: nowrap");
    expect(styles).toContain("overflow-x: auto");
    expect(styles).toContain("scrollbar-width: none");
    // The shared shell owns the panel's only scroll container.
    expect(styles).toContain(".key-chain-body {");
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
    expect(styles).toContain("var(--hone-signal-green)");
    expect(styles).toContain("var(--hone-signal-green-soft)");
    expect(styles).toContain("var(--hone-signal-yellow)");
    expect(styles).toContain("var(--hone-signal-yellow-soft)");
    expect(styles).toContain("var(--hone-signal-red)");
    expect(styles).toContain("var(--hone-paper-50)");
    // Dark mode rides the tokens; the old slate dark skin is gone.
    expect(styles).not.toContain("[data-theme=dark]");
    expect(styles).not.toContain("#111a28");
    // Mapped literals must not resurface.
    expect(styles).not.toContain("#28745b");
    expect(styles).not.toContain("#966d19");
    expect(styles).not.toContain("#f8f0dd");
  });
});
