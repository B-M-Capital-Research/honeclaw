import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(new URL("./public-admin-opening-portfolio-snapshot-materialization-implementation-panel.tsx", import.meta.url), "utf8");
const governanceSource = readFileSync(new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url), "utf8");

describe("Stage 134 opening portfolio snapshot materialization implementation panel", () => {
  test("explains completeness, exact decimals, provenance and zero capability", () => {
    expect(source).toContain("第 134 阶段 · 期初快照物化零能力实现登记");
    expect(source).toContain("现金、持仓、上市期权、负债与未结算活动");
    expect(source).toContain("禁止二进制浮点");
    expect(source).toContain("每行绑定工件摘要与来源位置");
    expect(source).toContain("没有解密、金融行解析、快照候选、真实持仓或交易权限");
    expect(source).toContain("Stage 135 独立实现复核");
  });

  test("is mounted immediately after the independently validated receipt gate", () => {
    const validationPanel = governanceSource.indexOf("<PublicAdminOpeningPortfolioSourceArtifactReceiptValidationPanel />");
    const implementationPanel = governanceSource.indexOf("<PublicAdminOpeningPortfolioSnapshotMaterializationImplementationPanel />");

    expect(validationPanel).toBeGreaterThan(-1);
    expect(implementationPanel).toBeGreaterThan(validationPanel);
    const validationPanelEnd = validationPanel + "<PublicAdminOpeningPortfolioSourceArtifactReceiptValidationPanel />".length;
    expect(governanceSource.slice(validationPanelEnd, implementationPanel)).not.toContain("<PublicAdmin");
  });
});
