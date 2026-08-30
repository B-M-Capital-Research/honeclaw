import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(
  new URL("./daily-signal-dashboard.tsx", import.meta.url),
  "utf8",
);
const styles = readFileSync(
  new URL("./daily-signal-dashboard.css", import.meta.url),
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

describe("daily signal dashboard contract", () => {
  test("is a controlled panel that reads cached reports and never regenerates", () => {
    expect(component).toContain("export function DailySignalPanel");
    expect(component).toContain("kind: DailySignalKind");
    expect(component).toContain("宏观红绿灯");
    expect(component).toContain("AI 红绿灯");
    expect(component).not.toContain("重新生成");
    // Shell behaviour (ESC, aria-modal, scroll lock) comes from the shared panel.
    expect(component).toContain("ResearchPanel");
    expect(component).not.toContain("<Portal>");
    // History is its own tab and loads lazily on first switch.
    expect(component).toContain("loadHistory");
  });

  test("leads with the verdict instead of a title-then-metadata preamble", () => {
    // Score, light and one sentence sit in the shared head, above the fold on
    // a phone. The bespoke header and the five-date strip that used to sit
    // between the hero and the dimensions are gone, not merely restyled.
    expect(component).toContain("ResearchPanelHead");
    expect(component).toContain('id="daily-signal-title"');
    expect(component).toContain("provenanceLine");
    expect(component).not.toContain("daily-signal-header");
    expect(component).not.toContain("daily-signal-meta");
    expect(styles).not.toContain(".daily-signal-header");
    expect(styles).not.toContain(".daily-signal-meta");
    // The hero keeps only what the head cannot show: the scale and the deltas.
    expect(component).toContain("较昨日");
    expect(component).toContain("原始风险");
  });

  test("removes unsupported AI and hardware placeholder factors", () => {
    expect(component).toContain("需求旁证 · 商业化 · 融资 · 资本开支");
    expect(component).toContain("company.metric_total");
    expect(component).not.toContain("硬件与电力链市场确认");
    expect(component).not.toContain("hardwareSignals:");
    expect(component).toContain("云厂商可核验财务框架");
    expect(component).not.toContain("云厂商十项框架");
    expect(component).not.toContain("company.coverage}/10");
  });

  test("has responsive layouts styled from the shared token system", () => {
    expect(styles).toContain("@media (max-width: 700px)");
    expect(styles).toContain("var(--hone-");
    // The half-circle dial reprinted the score the head already leads with,
    // and charged a fixed 240px column for it.
    expect(styles).toContain("daily-signal-scale");
    expect(styles).not.toContain("daily-signal-gauge");
    // Below the shared sheet breakpoint the hero shrinks so the scale and the
    // deltas both clear the fold without a scroll.
    expect(styles).toContain("@media (max-width: 760px)");
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
    expect(propsType).toBe("type Props = {\n  kind: DailySignalKind;\n  onClose: () => void;\n};");
    expect(code).not.toContain("onAsk");
    expect(code).not.toContain("发送到对话");
    expect(code).not.toContain("buildSavedReportPrompt");
    expect(code).not.toContain("HONE_SAVED");
    // load() survives for exactly two callers: first paint and error retry.
    expect(code).toContain("onMount(() => void load())");
  });

  test("labels the market chart as an index, never as the ETF", () => {
    expect(code).toContain("仅展示，不参与健康分");
    expect(code).toContain("跟踪该指数");
    expect(code).toContain("不是 QQQ / SPY 基金的价格或净值");
    expect(code).not.toContain("基金价格");
  });

  test("degrades the market chart to one line instead of hiding the section", () => {
    // A row the server could not fetch arrives with no points. It must be
    // named rather than blanking the chart, the relative reading has to be
    // withdrawn while it is missing, and the warning needs a style that
    // separates it from the boilerplate footnote below the chart.
    expect(code).toContain("const drawable = createMemo");
    expect(code).toContain("const missing = createMemo");
    expect(code).toContain("不能当相对强弱看");
    expect(code).not.toContain("props.series[0]?.points");
    expect(styles).toContain(".daily-signal-market__note.is-partial");
    // Colour follows the row's position in the server list, not its position
    // after filtering, so a line does not change colour on a degraded day.
    expect(code).toContain("is-line-${item.line}");
    expect(code).not.toContain("is-line-${index()}");
    // Nothing fetched at all still returns an empty list and the section is
    // gated on that upstream, so this fallback covers only the case where the
    // selected window leaves a drawn row with fewer than two points.
    expect(code).toContain(
      'fallback={<p class="daily-signal-market__empty">本次快照没有可对照的指数序列。</p>}',
    );
  });
});
