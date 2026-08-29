import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";
const component = readFileSync(new URL("./key-event-chain-dashboard.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("./key-event-chain-dashboard.css", import.meta.url), "utf8");
describe("key event chain dashboard", () => {
  it("renders the first-principles industry chain", () => { expect(component).toContain("关键事件链"); expect(component).toContain("模型 → 应用 → 数据中心 → 算力 → 光互连 → 存储 → 电力"); expect(component).toContain("第一性原理"); expect(component).toContain("下一验证点") });
  it("separates confirmed milestones from clues", () => { expect(component).toContain("只看一手确认"); expect(component).toContain("verification_status === \"confirmed\""); expect(component).toContain("查看一手原文"); expect(component).toContain("查看线索原文"); expect(styles).toContain("data-verification=confirmed") });
  it("preserves evidence and action boundaries", () => { expect(component).toContain("聚合翻译和管理员研究资料不是一手事实"); expect(component).toContain("影响待验证时不得补造结论"); expect(component).toContain("HONE_SAVED_KEY_EVENT_CHAIN"); expect(component).toContain("analysisHealth"); expect(component).toContain("模型影响门禁关闭"); expect(component).toContain("影响待分析") });
  it("shows deterministic event deduplication without hiding provenance", () => {
    expect(component).toContain("独立事件");
    expect(component).toContain("同一事件");
    expect(component).toContain("查看合并依据与全部来源");
    expect(component).toContain("supporting_sources");
    expect(component).toContain("event_fingerprint_sha256");
    expect(component).toContain("不得按来源数重复加权");
    expect(styles).toContain("data-dedup=merged");
  });
  it("keeps the timeline focused after the weekly brief is extracted", () => {
    expect(component).not.toContain("过去10日复盘");
    expect(component).not.toContain("未来10日验证问题");
    expect(component).not.toContain("tenDayBrief");
    expect(component.match(/key-chain-launcher/g)?.length).toBeGreaterThan(0);
  });
  it("supports mobile and dark", () => { expect(styles).toContain("@media(max-width:768px)"); expect(styles).toContain("[data-theme=dark]") });
});
