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

describe("daily signal dashboard contract", () => {
  test("is a controlled panel that reads cached reports and never regenerates", () => {
    expect(component).toContain("export function DailySignalPanel");
    expect(component).toContain("kind: DailySignalKind");
    expect(component).toContain("宏观红绿灯");
    expect(component).toContain("AI 红绿灯");
    expect(component).toContain("重新读取快照");
    expect(component).not.toContain("重新生成");
    // Shell behaviour (ESC, aria-modal, scroll lock) comes from the shared panel.
    expect(component).toContain("ResearchPanel");
    expect(component).not.toContain("<Portal>");
    // History is its own tab and loads lazily on first switch.
    expect(component).toContain("loadHistory");
  });

  test("preserves report context and cutoff rules when asking chat", () => {
    expect(component).toContain("HONE_SAVED_DAILY_SIGNAL_REPORT");
    expect(component).toContain("只使用上述已保存报告和当前对话");
    expect(component).toContain("不要在回答中改写正式评分");
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
    expect(styles).toContain("daily-signal-gauge");
  });
});
