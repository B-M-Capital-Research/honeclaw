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
    expect(component).toContain("industry_analysis_health");
    expect(component).toContain("产业影响分析门禁关闭");
    expect(component).toContain("影响待分析");
  });

  it("uses structured readable agenda cards instead of an image", () => {
    expect(component).toContain("groupByDate");
    expect(component).toContain("weekly-brief-agenda");
    expect(component).not.toContain("<img");
    expect(component).toContain("weekly-brief-tabs");
    expect(component).toContain("activeView");
    expect(styles).toContain("width:min(1040px,100%)");
    expect(styles).toContain("@media(max-width:800px)");
  });

  it("passes a bounded saved report into follow-up chat", () => {
    expect(component).toContain("HONE_SAVED_WEEKLY_BRIEF");
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

  it("inherits key-event deduplication and keeps every supporting source visible", () => {
    expect(component).toContain("同一事件");
    expect(component).toContain("查看同一事件的全部来源");
    expect(component).toContain("supporting_sources");
    expect(component).toContain("不得按来源数重复加权");
    expect(styles).toContain("data-dedup=merged");
  });
});
