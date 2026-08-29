import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-ledger-transition-implementation-panel.tsx", import.meta.url),
  "utf8",
);
const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 117 observation-to-ledger zero-capability implementation panel", () => {
  it("keeps opening portfolio and every financial capability closed", () => {
    expect(source).toContain("第 117 阶段");
    expect(source).toContain("Stage 88 绑定只作为初始化来源");
    expect(source).toContain("不默认或推断本金、现金、持仓、股数或权重");
    expect(source).toContain("不创建 opening snapshot、账本/事件、持仓、现金、NAV/绩效");
  });

  it("freezes accounting, gap, action and event invariants before Stage 118", () => {
    expect(source).toContain("raw close 是唯一证券会计口径");
    expect(source).toContain("显式 gap 阻断 NAV");
    expect(source).toContain("superseding 或 reversal");
    expect(source).toContain("exact decimal、append-only、幂等事件、稳定顺序与双重记账");
    expect(source).toContain("Stage 118");
  });

  it("is mounted strictly after the Stage 116 specification review", () => {
    expect(governanceSource.indexOf("<PublicAdminControlledShadowObservationLedgerTransitionImplementationPanel />"))
      .toBeGreaterThan(governanceSource.indexOf("<PublicAdminControlledShadowObservationLedgerTransitionSpecificationReviewPanel />"));
  });
});
