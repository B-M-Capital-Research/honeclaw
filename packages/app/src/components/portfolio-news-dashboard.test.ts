import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(new URL("./portfolio-news-dashboard.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("./portfolio-news-dashboard.css", import.meta.url), "utf8");

describe("portfolio news dashboard contract", () => {
  test("exposes the fourth cached dashboard and daily boundary", () => {
    expect(component).toContain("持仓重点新闻分析");
    expect(component).toContain("近 48 小时可信来源");
    expect(component).toContain("每日 20:00 更新");
    expect(component).toContain("getPublicPortfolioNews");
    expect(component).not.toContain("重新生成");
  });

  test("keeps provenance, analysis status and fail-closed states visible", () => {
    expect(component).toContain("item.source_url");
    expect(component).toContain("published_at_beijing");
    expect(component).toContain("unassessed");
    expect(component).toContain("等待新闻数据源");
    expect(component).toContain("待模型分析");
  });

  test("sends only the saved actor report into chat", () => {
    expect(component).toContain("HONE_SAVED_PORTFOLIO_NEWS_REPORT");
    expect(component).toContain("待分析项目不得补造结论");
    expect(component).toContain("不要自动修改仓位");
  });

  test("supports dark and mobile layouts", () => {
    expect(styles).toContain('[data-theme="dark"]');
    expect(styles).toContain("@media (max-width: 768px)");
  });
});
