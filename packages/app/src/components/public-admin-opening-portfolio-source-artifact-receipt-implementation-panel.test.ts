import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-opening-portfolio-source-artifact-receipt-implementation-panel.tsx", import.meta.url),
  "utf8",
);
const host = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 127 source artifact receipt implementation panel", () => {
  test("freezes private streaming, content addressing and privacy without uploading bytes", () => {
    expect(source).toContain("流式哈希与原子提交");
    expect(source).toContain("格式与主动内容拒绝");
    expect(source).toContain("匿名化与日志脱敏");
    expect(source).toContain("私有隔离区静态加密");
    expect(source).not.toContain("source_artifact_bytes");
  });

  test("keeps receipt, materialization, validation and admission separate", () => {
    expect(source).toContain("接收、快照物化、输出校验和快照准入继续保持分离");
    expect(source).toContain("Stage 128");
    expect(source).toContain("没有上传入口、来源字节、parser");
  });

  test("is mounted immediately after the Stage 126 review panel", () => {
    const stage126 = host.indexOf("PublicAdminOpeningPortfolioSnapshotGovernanceSpecificationReviewPanel />");
    const stage127 = host.indexOf("PublicAdminOpeningPortfolioSourceArtifactReceiptImplementationPanel />");
    expect(stage126).toBeGreaterThan(0);
    expect(stage127).toBeGreaterThan(stage126);
  });
});
