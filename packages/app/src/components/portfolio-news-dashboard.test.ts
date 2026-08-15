import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(new URL("./portfolio-news-dashboard.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("./portfolio-news-dashboard.css", import.meta.url), "utf8");

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

  test("keeps provenance, analysis status and fail-closed states visible", () => {
    expect(component).toContain("item.source_url");
    expect(component).toContain("published_at_beijing");
    expect(component).toContain("unassessed");
    expect(component).toContain("等待新闻数据源");
    expect(component).toContain("待模型分析");
  });

  test("sends only the saved actor report into chat and closes the panel", () => {
    expect(component).toContain("buildSavedReportPrompt");
    expect(component).toContain('marker: "HONE_SAVED_PORTFOLIO_NEWS_REPORT"');
    expect(component).toContain("待分析项目不得补造结论");
    expect(component).toContain("不要自动修改仓位");
    // Ask footer only exists when a chat sink is provided, and asking closes.
    expect(component).toContain("<Show when={props.onAsk}>");
    expect(component).toContain("props.onClose()");
  });

  test("uses hone design tokens with dark and mobile layouts", () => {
    expect(styles).toContain("var(--hone-ink-950)");
    expect(styles).toContain("var(--hone-ink-500)");
    expect(styles).toContain("var(--hone-coral-600)");
    expect(styles).toContain("var(--hone-signal-green)");
    expect(styles).toContain("var(--hone-signal-red)");
    expect(styles).toContain("var(--hone-signal-yellow");
    expect(styles).toContain("var(--hone-signal-red-soft)");
    expect(styles).not.toContain("#d3544a");
    expect(styles).not.toContain("#23845a");
    expect(styles).not.toContain("#d69c18");
    expect(styles).not.toContain("portfolio-news-launcher");
    expect(styles).toContain('[data-theme="dark"]');
    expect(styles).toContain("@media (max-width: 768px)");
  });
});
