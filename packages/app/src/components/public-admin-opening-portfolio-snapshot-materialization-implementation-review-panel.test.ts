import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-opening-portfolio-snapshot-materialization-implementation-review-panel.tsx", import.meta.url),
  "utf8",
);
const backend = readFileSync(
  new URL("../../../../crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_snapshot_materialization_implementation_reviews.rs", import.meta.url),
  "utf8",
);
const host = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 135 opening portfolio snapshot materialization independent review panel", () => {
  test("requires a second complete contract rebuild without the Stage 134 builder", () => {
    expect(source).toContain("未调用 Stage 134 builder 自证");
    expect(backend).toContain("rebuild_contract_without_stage_134_builder");
    expect(backend).not.toContain("implementation_contract(source");
    expect(source).toContain("Stage 136 隔离物化器规格登记");
  });

  test("keeps exact decimals, full-account failure and provenance explicit", () => {
    expect(source).toContain("精确十进制字符串和有符号数量");
    expect(source).toContain("缺失、歧义和不支持资产使整份快照失败");
    expect(source).toContain("每行绑定工件 SHA-256 与来源位置");
    expect(source).toContain("当前明确为空：key/input read、receipt 解密");
    expect(source).not.toContain("source_artifact_bytes");
  });

  test("is mounted immediately after the Stage 134 implementation panel", () => {
    const stage134 = host.indexOf("PublicAdminOpeningPortfolioSnapshotMaterializationImplementationPanel />");
    const stage135 = host.indexOf("PublicAdminOpeningPortfolioSnapshotMaterializationImplementationReviewPanel />");
    expect(stage134).toBeGreaterThan(0);
    expect(stage135).toBeGreaterThan(stage134);
    const stage134End = stage134 + "PublicAdminOpeningPortfolioSnapshotMaterializationImplementationPanel />".length;
    expect(host.slice(stage134End, stage135)).not.toContain("<PublicAdmin");
  });
});
