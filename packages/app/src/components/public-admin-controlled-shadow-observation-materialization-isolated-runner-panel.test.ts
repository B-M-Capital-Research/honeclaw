import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(
  new URL("./public-admin-controlled-shadow-observation-materialization-isolated-runner-panel.tsx", import.meta.url),
  "utf8",
);
const governanceSource = readFileSync(
  new URL("./public-admin-historical-outcome-governance-panel.tsx", import.meta.url),
  "utf8",
);

describe("Stage 109 observation materialization isolated runner panel", () => {
  it("binds a proposed artifact while preserving zero capability", () => {
    expect(source).toContain("第 109 阶段 · 观察物化隔离 runner 规格登记");
    expect(source).toContain("proposed_runner_artifact_sha256");
    expect(source).toContain("ephemeral_deterministic_observation_materialization_specification");
    expect(source).toContain("Stage 104 已准入、只读、内容寻址的精确输出");
    expect(source).toContain("供应商发布时间仍未验证");
    expect(source).toContain("登记只开放 Stage 110 责任链外首次执行授权复核资格");
    expect(source).toContain("仍没有观察物化工件、入口、runtime、挂载或准入输入读取权限");
  });

  it("is mounted in the historical outcome governance workspace", () => {
    expect(governanceSource).toContain("PublicAdminControlledShadowObservationMaterializationIsolatedRunnerPanel");
  });
});
