import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(new URL("./public-admin-controlled-shadow-observation-ledger-transition-specification-review-panel.tsx", import.meta.url), "utf8");
const governanceSource = readFileSync(new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url), "utf8");

describe("Stage 116 observation ledger transition specification review panel", () => {
  test("requires a chain-external second implementation rebuild from Stage 114", () => {
    expect(source).toContain("第二实现 · 责任链外");
    expect(source).toContain("不调用 Stage 115 builder，从当前 Stage 114 正式证据完整重建规格");
    expect(source).toContain("rebuilt_specification_exactly_matches_registered_specification_confirmed");
  });

  test("keeps opening state, accounting prices, gaps and company actions fail closed", () => {
    expect(source).toContain("Stage 88 绑定只是初始化来源，不是 opening positions");
    expect(source).toContain("opening portfolio snapshot 必须另行准入");
    expect(source).toContain("证券会计只用 raw close；adjusted prices 仅作非会计分析");
    expect(source).toContain("显式 gap 阻断 NAV");
    expect(source).toContain("分红和拆股在持仓与有效条款准入前只记 notice");
  });

  test("approval opens only the Stage 117 zero-capability registration gate", () => {
    expect(source).toContain("Stage 117 零能力实现登记");
    expect(source).toContain("尚未建账或计算绩效");
    expect(source).toContain("没有账本事件、持仓、现金、NAV/绩效、模型、训练/RL、reward、订单、券商或交易权限");
  });

  test("is mounted immediately after the Stage 115 panel", () => {
    expect(governanceSource.indexOf("<PublicAdminControlledShadowObservationLedgerTransitionSpecificationReviewPanel />"))
      .toBeGreaterThan(governanceSource.indexOf("<PublicAdminControlledShadowObservationLedgerTransitionSpecificationPanel />"));
  });
});
