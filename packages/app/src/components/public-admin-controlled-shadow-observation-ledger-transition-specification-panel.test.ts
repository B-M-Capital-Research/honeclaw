import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-ledger-transition-specification-panel.tsx", import.meta.url),
  "utf8",
);
const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 115 observation-to-ledger transition specification panel", () => {
  it("does not mistake Stage 88 initialization provenance for opening positions", () => {
    expect(source).toContain("Stage 88 绑定只是初始化来源，不能当作开仓持仓");
    expect(source).toContain("separately_admitted_opening_portfolio_snapshot_required_confirmed");
    expect(source).toContain("no_default_notional_cash_positions_or_share_quantities_confirmed");
  });

  it("freezes accounting price, gap and corporate-action semantics", () => {
    expect(source).toContain("未来持仓估值只用 raw close");
    expect(source).toContain("explicit_gap_blocks_nav_no_fill_interpolation_or_substitution_confirmed");
    expect(source).toContain("dividend_and_split_notices_require_position_and_effective_term_validation_before_posting_confirmed");
    expect(source).toContain("修正必须来自新准入证据");
  });

  it("opens only Stage 116 review and has no ledger authority", () => {
    expect(source).toContain("只开放 Stage 116 独立复核");
    expect(source).toContain("零会计写入");
    expect(source).toContain("no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed");
  });

  it("is mounted immediately after Stage 114 admission", () => {
    expect(governanceSource.indexOf("<PublicAdminControlledShadowObservationLedgerTransitionSpecificationPanel />"))
      .toBeGreaterThan(governanceSource.indexOf("<PublicAdminControlledShadowObservationEvidenceAdmissionPanel />"));
  });
});
