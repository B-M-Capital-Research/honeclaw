import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-opening-portfolio-source-artifact-receipt-implementation-review-panel.tsx", import.meta.url),
  "utf8",
);
const backend = readFileSync(
  new URL("../../../../crates/hone-web-api/src/routes/controlled_shadow_opening_portfolio_source_artifact_receipt_implementation_reviews.rs", import.meta.url),
  "utf8",
);
const host = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 128 source artifact receipt implementation independent review panel", () => {
  test("requires a second contract rebuild without calling the Stage 127 builder", () => {
    expect(source).toContain("未调用 Stage 127 builder 自证");
    expect(backend).toContain("independently_rebuild_contract");
    expect(backend).not.toContain("build_implementation_contract(");
    expect(source).toContain("Stage 129 隔离接收器规格登记");
  });

  test("revalidates seventeen confirmations while keeping every data authority closed", () => {
    expect(source).toContain("已重新校验 Stage 127 全部 17 项登记确认");
    expect(source).toContain("无上传、来源字节、存储写入、parser/runtime、网络、secret、工具或子进程");
    expect(source).not.toContain("source_artifact_bytes");
    expect(source).toContain("当前明确为空：上传入口、来源字节");
  });

  test("is mounted immediately after the Stage 127 implementation panel", () => {
    const stage127 = host.indexOf("PublicAdminOpeningPortfolioSourceArtifactReceiptImplementationPanel />");
    const stage128 = host.indexOf("PublicAdminOpeningPortfolioSourceArtifactReceiptImplementationReviewPanel />");
    expect(stage127).toBeGreaterThan(0);
    expect(stage128).toBeGreaterThan(stage127);
  });
});
