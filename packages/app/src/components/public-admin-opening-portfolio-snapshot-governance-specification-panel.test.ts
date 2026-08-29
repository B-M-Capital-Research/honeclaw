import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-opening-portfolio-snapshot-governance-specification-panel.tsx", import.meta.url),
  "utf8",
);
const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 125 opening portfolio snapshot governance specification panel", () => {
  it("defines a complete external source and financial-state contract", () => {
    expect(source).toContain("第 125 阶段");
    expect(source).toContain("券商/托管机构机器导出");
    expect(source).toContain("现金、持仓、负债和未结算活动必须完整覆盖");
    expect(source).toContain("证券身份与公司行动必须完成对账");
    expect(source).toContain("statement_market_values_are_informational_not_accounting_marks_confirmed");
  });

  it("keeps registration at zero capability and requires Stage 126 review", () => {
    expect(source).toContain("specification_only_no_artifact_upload_read_parse_or_snapshot_materialization_confirmed");
    expect(source).toContain("no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed");
    expect(source).toContain("future_stage_126_independent_specification_review_required_confirmed");
    expect(source).toContain("尚无来源文件、期初组合、持仓、现金、净值或交易权限");
  });

  it("is mounted immediately after Stage 124", () => {
    const stage124 = governanceSource.indexOf("<PublicAdminControlledShadowObservationLedgerTransitionCandidateAdmissionPanel />");
    const stage125 = governanceSource.indexOf("<PublicAdminOpeningPortfolioSnapshotGovernanceSpecificationPanel />");
    expect(stage124).toBeGreaterThan(-1);
    expect(stage125).toBeGreaterThan(stage124);
  });
});
