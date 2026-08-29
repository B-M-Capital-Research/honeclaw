import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-market-data-parser-execution-attempt-claim-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 101 market-data parser execution-attempt claim panel", () => {
  it("consumes authorization before execution and keeps Stage 102 separate", () => {
    expect(source).toContain("第 101 阶段");
    expect(source).toContain("永久消费授权");
    expect(source).toContain("待 Stage 102");
    expect(source).toContain("no_entrypoint_runtime_mount_payload_read_parser_execution_or_parsed_rows_confirmed");
  });

  it("binds the exact artifact and fixed Stage 94 input manifest", () => {
    expect(source).toContain("expected_artifact_manifest_sha256");
    expect(source).toContain("expected_stage_94_validation_sha256");
    expect(source).toContain("expected_stage_93_receipt_sha256");
    expect(source).toContain("expected_fixed_input_manifest_sha256");
    expect(source).toContain("只显示元数据与摘要");
  });

  it("does not expose a parser execution action", () => {
    expect(source).toContain("本按钮不会运行 parser");
    expect(source).not.toContain("executeControlledShadowMarketDataParser");
  });
});
