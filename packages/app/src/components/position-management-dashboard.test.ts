import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(new URL("./position-management-dashboard.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("./position-management-dashboard.css", import.meta.url), "utf8");

describe("position management dashboard contract", () => {
  it("exposes the fifth cached dashboard and daily boundary", () => {
    expect(component).toContain("仓位管理建议");
    expect(component).toContain("每日 20:00 更新");
    expect(component).toContain("getPublicPositionManagement");
    expect(component).not.toContain("生成建议");
  });

  it("keeps Hari logic separate from HONE concentration controls", () => {
    expect(component).toContain("framework_version");
    expect(component).toContain("methodology_note");
    expect(component).toContain("证伪条件");
    expect(component).toContain("数据日期");
  });

  it("never claims execution when sending the saved report", () => {
    expect(component).toContain("HONE_SAVED_POSITION_MANAGEMENT_REPORT");
    expect(component).toContain("不得自动修改持仓或声称已经下单");
    expect(component).toContain("不得补造缺失行情、财务、估值或新闻");
  });

  it("supports mobile and dark surfaces", () => {
    expect(styles).toContain("@media(max-width:768px)");
    expect(styles).toContain('[data-theme="dark"]');
    expect(styles).toContain("94dvh");
  });
});
