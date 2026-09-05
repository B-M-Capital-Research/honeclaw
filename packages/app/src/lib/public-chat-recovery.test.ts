import { describe, expect, it } from "bun:test";

import { resolvePublicChatRecovery } from "@/lib/public-chat";
import type { PublicChatActiveRun } from "@/lib/types";

const run = (overrides: Partial<PublicChatActiveRun> = {}): PublicChatActiveRun => ({
  run_id: "run-1",
  started_at_ms: 1_786_000_000_000,
  updated_at_ms: 1_786_000_030_000,
  phase: "running",
  status_text: "正在输出最终回答",
  ...overrides,
});

const recover = (activeRun: PublicChatActiveRun | null, interruptedRun = false) =>
  resolvePublicChatRecovery({
    activeRun,
    interruptedRun,
    thinkingText: "正在思考",
    interruptedText: "上次请求已中断，请重新发送",
  });

describe("a refresh mid-run keeps the trail", () => {
  it("restores the stages the run already passed through", () => {
    // The pre-turn pass alone issues around twenty provider calls before the
    // first token. Recovering only the newest line made a refresh look like
    // the run had forgotten what it was doing.
    const recovered = recover(
      run({
        steps: [
          "正在准备并核验所需信息",
          "正在核验 NBIS 的证券身份",
          "正在读取 NBIS 的行情、季度财报与估值口径",
        ],
      }),
    );

    expect(recovered.activeRunId).toBe("run-1");
    expect(recovered.message?.steps).toEqual([
      "正在准备并核验所需信息",
      "正在核验 NBIS 的证券身份",
      "正在读取 NBIS 的行情、季度财报与估值口径",
    ]);
    expect(recovered.message?.statusText).toBe("正在输出最终回答");
    // The server's original start time is what keeps the elapsed counter from
    // restarting at zero on every refresh.
    expect(recovered.message?.startedAt).toBe(1_786_000_000_000);
  });

  it("survives a server that sends no trail at all", () => {
    // An older process, or a run that has not reported a stage yet.
    expect(recover(run()).message?.steps).toEqual([]);
    expect(recover(run({ steps: [] })).message?.steps).toEqual([]);
  });

  it("drops blank and repeated stages and keeps the trail bounded", () => {
    const recovered = recover(
      run({
        steps: ["  ", "读取行情", "读取行情", "", ...Array.from({ length: 10 }, (_, i) => `阶段 ${i}`)],
      }),
    );

    const steps = recovered.message?.steps ?? [];
    expect(steps.length).toBeLessThanOrEqual(8);
    expect(steps).not.toContain("");
    expect(steps.at(-1)).toBe("阶段 9");
    // A repeated stage is the same stage, not a new one.
    expect(steps.filter((step) => step === "读取行情").length).toBeLessThanOrEqual(1);
  });

  it("shows an interrupted run instead of a thinking card that never ends", () => {
    // The registry lives in the serving process, so a restart leaves an
    // unanswered turn with no active run. Failing closed here is what stops a
    // permanent "thinking" state.
    const recovered = recover(null, true);

    expect(recovered.activeRunId).toBeUndefined();
    expect(recovered.message?.phase).toBe("error");
    expect(recovered.message?.statusText).toBe("上次请求已中断，请重新发送");
  });

  it("shows nothing when there is neither an active nor an interrupted run", () => {
    expect(recover(null, false)).toEqual({});
  });
});
