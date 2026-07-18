import { describe, expect, it } from "bun:test";
import {
  displayGithubStars,
  formatGithubStars,
  GITHUB_STARS_FALLBACK,
  parseGithubStarsCache,
} from "@/lib/github-stars";
import {
  applyPublicAssistantStreamEvent,
  canSendPublicChatMessage,
  findPendingPublicAssistantMessage,
  formatPublicAttachmentBytes,
  isPublicChatBusy,
  isPublicChatTerminalStreamEvent,
  isPublicChatQuotaExhausted,
  latestUnreadPushId,
  mergePublicPushItems,
  mergePublicHistoryWindow,
  normalizePhoneNumber,
  PUBLIC_RESTORE_MAX_ATTEMPTS,
  PUBLIC_CHAT_VIEWPORT_CONTENT,
  publicRestoreRetryDelay,
  publicAttachmentFileLabel,
  publicChatRunEventPatch,
  publicChatRunStartedAtLabel,
  publicChatTerminalEventPatch,
  publicChatToolStatusText,
  rekeyTrailingOptimisticIds,
  resolvePublicChatRecovery,
  resolvePublicChatView,
  shouldRetryPublicRestore,
  shouldPollPublicChatRecovery,
  shouldRecoverPublicChatAfterEof,
  shouldRecoverPinnedBottom,
  shouldPreventPublicChatPinch,
  shouldSubmitPublicChatEnter,
  shouldLoadOlderPublicMessages,
  splitPublicChatAttachments,
  stripAttachmentMarkers,
  toPublicChatMessages,
  unreadCountAfterScheduledPush,
} from "@/lib/public-chat";
import type { HistoryMsg } from "@/lib/types";

function messageIds(messages: Array<{ id: string }>): string[] {
  return messages.map((message) => message.id);
}

describe("normalizePhoneNumber", () => {
  it("keeps a leading plus and strips non-digits", () => {
    expect(normalizePhoneNumber(" +86 138-0013-8000 ")).toBe("+8613800138000");
  });
});

describe("public assistant stream events", () => {
  it("appends native deltas and clears a tool preamble on reset", () => {
    let content = applyPublicAssistantStreamEvent("", "assistant_delta", "先查");
    content = applyPublicAssistantStreamEvent(content, "assistant_delta", "一下");
    expect(content).toBe("先查一下");
    content = applyPublicAssistantStreamEvent(content, "assistant_reset");
    expect(content).toBe("");
    expect(
      applyPublicAssistantStreamEvent(content, "assistant_delta", "最终答案"),
    ).toBe("最终答案");
  });
});

describe("public chat active-run recovery", () => {
  const activeRun = {
    run_id: "run-42",
    started_at_ms: 1_753_000_000_000,
    phase: "running" as const,
    status_text: "正在核验实时行情",
    updated_at_ms: 1_753_000_012_000,
  };

  it("restores elapsed-time origin from the server instead of page load time", () => {
    const first = resolvePublicChatRecovery({
      activeRun,
      thinkingText: "HONE 思考中",
      interruptedText: "上次请求已中断，请重新发送",
    });
    const afterRefresh = resolvePublicChatRecovery({
      activeRun,
      thinkingText: "HONE 思考中",
      interruptedText: "上次请求已中断，请重新发送",
    });

    expect(first.activeRunId).toBe("run-42");
    expect(first.message?.startedAt).toBe(activeRun.started_at_ms);
    expect(afterRefresh.message?.startedAt).toBe(activeRun.started_at_ms);
    expect(afterRefresh.message?.phase).toBe("running");
    expect(afterRefresh.message?.statusText).toBe("正在核验实时行情");
  });

  it("formats the server-owned start in Beijing time and keeps it stable", () => {
    const startedAt = Date.parse("2026-07-17T00:30:00.000Z");
    const beforeRefresh = publicChatRunStartedAtLabel(startedAt);
    const afterRefresh = publicChatRunStartedAtLabel(startedAt);

    expect(beforeRefresh?.startsWith("查询开始：北京时间 ")).toBe(true);
    expect(beforeRefresh).toContain("2026");
    expect(beforeRefresh).toContain("08:30");
    expect(afterRefresh).toBe(beforeRefresh);
    expect(publicChatRunStartedAtLabel(undefined)).toBeUndefined();
  });

  it("does not infer a running turn from a trailing user message alone", () => {
    const recovery = resolvePublicChatRecovery({
      activeRun: null,
      interruptedRun: false,
      thinkingText: "HONE 思考中",
      interruptedText: "上次请求已中断，请重新发送",
    });

    expect(recovery).toEqual({});
  });

  it("trusts interrupted_run even when another visible card trails the user", () => {
    const recovery = resolvePublicChatRecovery({
      activeRun: null,
      interruptedRun: true,
      thinkingText: "HONE 思考中",
      interruptedText: "上次请求已中断，请重新发送",
    });

    expect(recovery.activeRunId).toBeUndefined();
    expect(recovery.message).toMatchObject({
      id: "_interrupted",
      role: "assistant",
      phase: "error",
      statusText: "上次请求已中断，请重新发送",
    });
    expect(recovery.message?.startedAt).toBeUndefined();
  });
});

describe("public chat run progress", () => {
  const current = {
    id: "assistant-1",
    role: "assistant" as const,
    content: "",
    phase: "thinking" as const,
    statusText: "HONE 思考中",
    startedAt: 100,
  };

  it("applies server run identity, start time, phase, and status", () => {
    expect(
      publicChatRunEventPatch(
        current,
        {
          run_id: "run-1",
          started_at_ms: 1_000,
          phase: "running",
          status_text: "正在查询公司与行情",
          updated_at_ms: 1_200,
        },
        "HONE 执行中",
      ),
    ).toEqual({
      phase: "running",
      statusText: "正在查询公司与行情",
      startedAt: 1_000,
      runId: "run-1",
      statusUpdatedAt: 1_200,
    });
  });

  it("ignores stale progress and events for a different run", () => {
    const recovered = {
      ...current,
      runId: "run-1",
      statusUpdatedAt: 2_000,
    };
    expect(
      publicChatRunEventPatch(
        recovered,
        {
          run_id: "run-1",
          updated_at_ms: 1_999,
          status_text: "旧状态",
        },
        "HONE 执行中",
      ),
    ).toBeUndefined();
    expect(
      publicChatRunEventPatch(
        recovered,
        {
          run_id: "run-2",
          updated_at_ms: 2_001,
          status_text: "另一轮",
        },
        "HONE 执行中",
      ),
    ).toBeUndefined();
  });

  it("uses the running fallback when progress omits display text", () => {
    expect(
      publicChatRunEventPatch(
        current,
        { run_id: "run-1", phase: "running", updated_at_ms: 2_000 },
        "HONE 执行中",
      )?.statusText,
    ).toBe("HONE 执行中");
  });

  it("only displays public tool status text and ignores every raw field", () => {
    expect(
      publicChatToolStatusText(
        {
          public_status_text: "正在核验实时价格",
          tool: "raw-secret-tool",
          text: "raw-secret-text",
          reasoning: "raw-secret-reasoning",
        },
        "HONE 执行中",
      ),
    ).toBe("正在核验实时价格");
    expect(
      publicChatToolStatusText(
        {
          tool: "raw-secret-tool",
          text: "raw-secret-text",
          reasoning: "raw-secret-reasoning",
        },
        "HONE 执行中",
      ),
    ).toBe("HONE 执行中");
  });
});

describe("public chat terminal result", () => {
  it("keeps an explicitly partial committed answer stable without claiming success", () => {
    expect(
      publicChatTerminalEventPatch(
        { success: false, partial: true },
        "不应显示的运行错误",
        "请求出错，请重试",
      ),
    ).toEqual({ phase: "done", statusText: undefined });
  });

  it("still renders an ordinary failed run as an error", () => {
    expect(
      publicChatTerminalEventPatch(
        { success: false },
        "运行失败",
        "请求出错，请重试",
      ),
    ).toEqual({ phase: "error", statusText: "运行失败" });
  });
});

describe("formatGithubStars", () => {
  it("formats compact star counts for the public nav", () => {
    expect(formatGithubStars(42)).toBe("42");
    expect(formatGithubStars(1250)).toBe("1.3k");
    expect(formatGithubStars(12000)).toBe("12k");
  });
});

describe("displayGithubStars", () => {
  it("never exposes the loading placeholder in public nav", () => {
    expect(displayGithubStars(undefined)).toBe(GITHUB_STARS_FALLBACK);
    expect(displayGithubStars("...")).toBe(GITHUB_STARS_FALLBACK);
    expect(displayGithubStars("1.2k")).toBe("1.2k");
  });

  it("ignores legacy and expired star caches", () => {
    const now = 10 * 60 * 60 * 1000;
    expect(parseGithubStarsCache("742", now)).toBeUndefined();
    expect(
      parseGithubStarsCache(
        JSON.stringify({ value: "741", cachedAt: now - 7 * 60 * 60 * 1000 }),
        now,
      ),
    ).toBeUndefined();
    expect(
      parseGithubStarsCache(
        JSON.stringify({ value: "742", cachedAt: now - 60 * 1000 }),
        now,
      ),
    ).toBe("742");
  });
});

describe("public chat IME enter policy", () => {
  const base = {
    key: "Enter",
    shiftKey: false,
    eventIsComposing: false,
    compositionActive: false,
    keyCode: 13,
    now: 1_000,
    suppressEnterUntil: 0,
  };

  it("submits a normal Enter but not Chinese candidate confirmation", () => {
    expect(shouldSubmitPublicChatEnter(base)).toBe(true);
    expect(
      shouldSubmitPublicChatEnter({ ...base, eventIsComposing: true }),
    ).toBe(false);
    expect(shouldSubmitPublicChatEnter({ ...base, keyCode: 229 })).toBe(false);
    expect(
      shouldSubmitPublicChatEnter({ ...base, suppressEnterUntil: 1_120 }),
    ).toBe(false);
  });

  it("keeps Shift+Enter available for line breaks", () => {
    expect(shouldSubmitPublicChatEnter({ ...base, shiftKey: true })).toBe(false);
  });
});

describe("resolvePublicChatView", () => {
  it("keeps the loading shell visible while restoring an existing session", () => {
    expect(resolvePublicChatView("loading")).toBe("loading");
    expect(resolvePublicChatView("ready")).toBe("chat");
    expect(resolvePublicChatView("logged_out")).toBe("login");
    expect(resolvePublicChatView("logging_in")).toBe("login");
  });
});

describe("public chat restore retry policy", () => {
  it("retries slow restores with capped backoff before surfacing failure", () => {
    expect(shouldRetryPublicRestore(1)).toBe(true);
    expect(shouldRetryPublicRestore(PUBLIC_RESTORE_MAX_ATTEMPTS)).toBe(false);
    expect(publicRestoreRetryDelay(1)).toBeLessThan(
      publicRestoreRetryDelay(2),
    );
    expect(publicRestoreRetryDelay(99)).toBe(publicRestoreRetryDelay(3));
  });

  it("keeps background polling single-flight", () => {
    expect(
      shouldPollPublicChatRecovery({
        hasBackgroundPending: true,
        isSending: false,
        restoreInFlight: false,
      }),
    ).toBe(true);
    expect(
      shouldPollPublicChatRecovery({
        hasBackgroundPending: true,
        isSending: false,
        restoreInFlight: true,
      }),
    ).toBe(false);
  });

  it("recovers only a clean EOF that arrived without a terminal event", () => {
    expect(
      shouldRecoverPublicChatAfterEof({
        reachedEof: true,
        sawTerminalEvent: false,
      }),
    ).toBe(true);
    expect(
      shouldRecoverPublicChatAfterEof({
        reachedEof: true,
        sawTerminalEvent: true,
      }),
    ).toBe(false);
    expect(
      shouldRecoverPublicChatAfterEof({
        reachedEof: false,
        sawTerminalEvent: false,
      }),
    ).toBe(false);
  });

  it("treats only authoritative completion frames as terminal", () => {
    expect(isPublicChatTerminalStreamEvent("run_finished")).toBe(true);
    expect(isPublicChatTerminalStreamEvent("error")).toBe(true);
    expect(isPublicChatTerminalStreamEvent("done")).toBe(true);
    expect(isPublicChatTerminalStreamEvent("run_error")).toBe(false);
    expect(isPublicChatTerminalStreamEvent("assistant_delta")).toBe(false);
    expect(isPublicChatTerminalStreamEvent("assistant_reset")).toBe(false);
  });
});

describe("public chat mobile pinch policy", () => {
  it("locks native page zoom while preserving controlled calendar pinch", () => {
    expect(PUBLIC_CHAT_VIEWPORT_CONTENT).toContain("maximum-scale=1");
    expect(PUBLIC_CHAT_VIEWPORT_CONTENT).toContain("user-scalable=no");
    expect(
      shouldPreventPublicChatPinch({
        touchCount: 2,
        insideControlledSurface: false,
      }),
    ).toBe(true);
    expect(
      shouldPreventPublicChatPinch({
        touchCount: 2,
        insideControlledSurface: true,
      }),
    ).toBe(false);
    expect(
      shouldPreventPublicChatPinch({
        touchCount: 1,
        insideControlledSurface: false,
      }),
    ).toBe(false);
  });
});

describe("toPublicChatMessages", () => {
  it("keeps stable ids for unchanged history rows", () => {
    const history: HistoryMsg[] = [
      { role: "user", content: "我是 zxr", attachments: [] },
      { role: "assistant", content: "已记住，你是 zxr。", attachments: [] },
    ];

    const first = toPublicChatMessages(history);
    const second = toPublicChatMessages(history);

    expect(messageIds(first)).toEqual(messageIds(second));
  });

  it("preserves existing prefix ids when new history rows are appended", () => {
    const baseHistory: HistoryMsg[] = [
      { role: "user", content: "第一个问题", attachments: [] },
      { role: "assistant", content: "第一个回答", attachments: [] },
    ];

    const nextHistory: HistoryMsg[] = [
      ...baseHistory,
      { role: "user", content: "第二个问题", attachments: [] },
      { role: "assistant", content: "第二个回答", attachments: [] },
    ];

    const baselineMessages = toPublicChatMessages(baseHistory);
    const appendedMessages = toPublicChatMessages(nextHistory);

    expect(messageIds(appendedMessages.slice(0, baselineMessages.length))).toEqual(
      messageIds(baselineMessages),
    );
  });

  it("tolerates legacy history rows with missing content or attachments", () => {
    const messages = toPublicChatMessages([
      { role: "user", attachments: [] },
      { role: "assistant", content: { text: "ok" } },
      { role: "assistant", content: "done", attachments: undefined },
    ] as unknown as HistoryMsg[]);

    expect(messages.map((message) => message.content)).toEqual([
      "",
      '{"text":"ok"}',
      "done",
    ]);
    expect(messages.map((message) => message.attachments)).toEqual([
      [],
      [],
      [],
    ]);
  });

  it("keeps valid public attachment metadata and drops malformed rows", () => {
    const messages = toPublicChatMessages([
      {
        role: "user",
        content: "see attached",
        attachments: [
          { path: "/uploads/chart.png", name: "chart.png", kind: "image" },
          { path: "/uploads/report.pdf", name: "report.pdf", kind: "pdf" },
          { path: "/uploads/broken.bin", name: 42, kind: "file" },
        ],
      },
    ] as unknown as HistoryMsg[]);

    expect(messages[0]!.attachments).toEqual([
      { path: "/uploads/chart.png", name: "chart.png", kind: "image" },
      { path: "/uploads/report.pdf", name: "report.pdf", kind: "pdf" },
    ]);
  });

  it("maps scheduled history into a card model without restoring full text", () => {
    const messages = toPublicChatMessages([
      {
        role: "assistant",
        content: "",
        subtype: "scheduled_push",
        attachments: [],
        scheduled_push: {
          push_id: "job-1:2026-07-10:20:00",
          title: "收盘复盘",
          summary: "风险偏好回升，半导体领涨。",
        },
      },
    ]);

    expect(messages[0]!.content).toBe("");
    expect(messages[0]!.scheduledPush).toEqual({
      pushId: "job-1:2026-07-10:20:00",
      title: "收盘复盘",
      summary: "风险偏好回升，半导体领涨。",
      fallbackContent: undefined,
    });
  });

  it("keeps the backend-selected finance calendar image contract", () => {
    const messages = toPublicChatMessages([
      {
        role: "assistant",
        content:
          "这是你的 2026-07 财经日历：\n\nfile:///tmp/calendar.png\n\nfile:///tmp/calendar-mobile-v4.png",
        attachments: [],
        finance_calendar: {
          month: "2026-07",
          image_path: "/tmp/calendar-mobile-v4.png",
          variant: "mobile",
        },
      },
    ]);

    expect(messages[0]!.financeCalendar).toEqual({
      month: "2026-07",
      image_path: "/tmp/calendar-mobile-v4.png",
      variant: "mobile",
    });
  });
});

describe("public push inbox model", () => {
  it("merges paginated and live pushes without duplicates", () => {
    const p1 = {
      push_id: "p1",
      job_id: "j1",
      title: "One",
      summary: "one",
      created_at: "2026-07-10T10:00:00+08:00",
    };
    const p2 = { ...p1, push_id: "p2", title: "Two" };

    expect(mergePublicPushItems([p2], [p2, p1])).toEqual([p2, p1]);
  });

  it("prefers authoritative unread counts and falls back to incrementing", () => {
    expect(unreadCountAfterScheduledPush(2, 7)).toBe(7);
    expect(unreadCountAfterScheduledPush(2)).toBe(3);
    expect(unreadCountAfterScheduledPush(2, -4)).toBe(0);
  });

  it("acknowledges only the latest item visible when the inbox opens", () => {
    const items = [
      {
        push_id: "latest",
        job_id: "j1",
        title: "Latest",
        summary: "latest",
        created_at: "2026-07-10T20:00:00+08:00",
      },
      {
        push_id: "older",
        job_id: "j1",
        title: "Older",
        summary: "older",
        created_at: "2026-07-09T20:00:00+08:00",
      },
    ];

    expect(latestUnreadPushId(items, 2)).toBe("latest");
    expect(latestUnreadPushId(items, 0)).toBeUndefined();
    expect(latestUnreadPushId([], 3)).toBeUndefined();
  });
});

describe("stripAttachmentMarkers", () => {
  it("treats absent content as empty text", () => {
    expect(stripAttachmentMarkers(undefined)).toBe("");
  });

  it("removes server attachment marker lines without stripping inline text", () => {
    expect(
      stripAttachmentMarkers(
        "问题正文\n[附件: /uploads/chart.png]\n正文里提到 [附件: label] 不应被删",
      ),
    ).toBe("问题正文\n正文里提到 [附件: label] 不应被删");
  });
});

describe("public chat composer state", () => {
  it("treats a recovered or locally pending run as the same busy state", () => {
    expect(
      isPublicChatBusy({
        isSending: false,
        hasPendingAssistant: false,
        hasBackgroundPending: true,
      }),
    ).toBe(true);
    expect(
      isPublicChatBusy({
        isSending: false,
        hasPendingAssistant: true,
        hasBackgroundPending: false,
      }),
    ).toBe(true);
    expect(
      isPublicChatBusy({
        isSending: false,
        hasPendingAssistant: false,
        hasBackgroundPending: false,
      }),
    ).toBe(false);
  });

  it("derives send eligibility from draft, attachments, busy state, and quota", () => {
    const emptyComposerState = {
      draft: "",
      attachmentCount: 0,
      isSending: false,
      uploading: false,
      remaining: undefined,
      dailyLimit: undefined,
    };

    expect(canSendPublicChatMessage(emptyComposerState)).toBe(false);
    expect(
      canSendPublicChatMessage({ ...emptyComposerState, draft: "  hello " }),
    ).toBe(true);
    expect(
      canSendPublicChatMessage({ ...emptyComposerState, attachmentCount: 1 }),
    ).toBe(true);
    expect(
      canSendPublicChatMessage({
        ...emptyComposerState,
        draft: "hello",
        isSending: true,
      }),
    ).toBe(false);
    expect(
      canSendPublicChatMessage({
        ...emptyComposerState,
        draft: "hello",
        uploading: true,
      }),
    ).toBe(false);
    expect(
      canSendPublicChatMessage({
        ...emptyComposerState,
        draft: "hello",
        remaining: 0,
        dailyLimit: 10,
      }),
    ).toBe(false);
    expect(
      canSendPublicChatMessage({
        ...emptyComposerState,
        draft: "hello",
        remaining: 0,
        dailyLimit: 0,
      }),
    ).toBe(true);
  });

  it("only treats positive daily limits as capped quota", () => {
    expect(
      isPublicChatQuotaExhausted({ remaining: 0, dailyLimit: undefined }),
    ).toBe(false);
    expect(isPublicChatQuotaExhausted({ remaining: 0, dailyLimit: 0 })).toBe(
      false,
    );
    expect(isPublicChatQuotaExhausted({ remaining: 0, dailyLimit: 3 })).toBe(
      true,
    );
  });
});

describe("public chat attachment model", () => {
  it("splits preview attachments into image and file groups", () => {
    const groups = splitPublicChatAttachments([
      { path: "/a.png", name: "a.png", kind: "image" },
      { path: "/b.pdf", name: "b.pdf", kind: "pdf" },
      { path: "/c.bin", name: "c.bin", kind: "file" },
    ]);

    expect(groups.images.map((attachment) => attachment.path)).toEqual([
      "/a.png",
    ]);
    expect(groups.files.map((attachment) => attachment.path)).toEqual([
      "/b.pdf",
      "/c.bin",
    ]);
  });

  it("derives compact file labels and byte labels for attachment chips", () => {
    expect(publicAttachmentFileLabel("report.pdf")).toBe("PDF");
    expect(publicAttachmentFileLabel("archive.longext")).toBe("LONG");
    expect(publicAttachmentFileLabel("README")).toBe("FILE");
    expect(formatPublicAttachmentBytes(undefined)).toBe("");
    expect(formatPublicAttachmentBytes(512)).toBe("512 B");
    expect(formatPublicAttachmentBytes(1536)).toBe("1.5 KB");
    expect(formatPublicAttachmentBytes(2 * 1024 * 1024)).toBe("2.0 MB");
  });
});

describe("public chat pending assistant state", () => {
  it("selects the latest non-terminal assistant message", () => {
    const running = {
      id: "a2",
      role: "assistant" as const,
      content: "",
      phase: "running" as const,
    };
    const messages = [
      { id: "u1", role: "user" as const, content: "hi" },
      {
        id: "a1",
        role: "assistant" as const,
        content: "done",
        phase: "done" as const,
      },
      running,
      {
        id: "a3",
        role: "assistant" as const,
        content: "failed",
        phase: "error" as const,
      },
    ];

    expect(findPendingPublicAssistantMessage(messages)).toBe(running);
  });

});

describe("public chat history window", () => {
  it("uses absolute history offsets for stable IDs across pages", () => {
    const history = [
      { role: "user", content: "older", attachments: [] },
      { role: "assistant", content: "newer", attachments: [] },
    ] as HistoryMsg[];
    const page = toPublicChatMessages(history, 40);
    expect(page[0]!.id.startsWith("h40_")).toBe(true);
    expect(page[1]!.id.startsWith("h41_")).toBe(true);
  });

  it("keeps already loaded older rows when refreshing the latest page", () => {
    const current = Array.from({ length: 20 }, (_, index) => ({
      id: `h${20 + index}_old`,
      role: (index % 2 === 0 ? "user" : "assistant") as "user" | "assistant",
      content: `old-${index}`,
    }));
    const latest = Array.from({ length: 20 }, (_, index) => ({
      id: `h${22 + index}_new`,
      role: (index % 2 === 0 ? "user" : "assistant") as "user" | "assistant",
      content: `new-${index}`,
    }));

    const merged = mergePublicHistoryWindow(current, 20, latest, 22);

    expect(merged.start).toBe(20);
    expect(merged.messages).toHaveLength(22);
    expect(merged.messages.slice(0, 2)).toEqual(current.slice(0, 2));
    expect(merged.messages.slice(2)).toEqual(latest);
  });

  it("rewrites trailing history ids to preserve optimistic UUIDs", () => {
    const existing = [
      { id: "h0_abc", role: "user" as const },
      { id: "h1_def", role: "assistant" as const },
      { id: "5f3a8e1c-1234-5678-9abc-def012345678", role: "user" as const },
      { id: "7c9d0e2f-1234-5678-9abc-def012345678", role: "assistant" as const },
    ];
    const next = [
      { id: "h0_abc", role: "user" as const },
      { id: "h1_def", role: "assistant" as const },
      { id: "h2_xyz", role: "user" as const },
      { id: "h3_uvw", role: "assistant" as const },
    ];
    rekeyTrailingOptimisticIds(existing, next);
    // Trailing pair takes the optimistic UUIDs so reconcile patches in place.
    expect(next[2]!.id).toBe("5f3a8e1c-1234-5678-9abc-def012345678");
    expect(next[3]!.id).toBe("7c9d0e2f-1234-5678-9abc-def012345678");
    // Stable-id history rows above the trailing pair are left untouched.
    expect(next[0]!.id).toBe("h0_abc");
    expect(next[1]!.id).toBe("h1_def");
  });

  it("stops rewriting once a stable id is encountered", () => {
    // Mid-array stable id means no more optimistic IDs above it; walk halts.
    const existing = [
      { id: "h0_abc", role: "user" as const },
      { id: "abc-uuid-1", role: "assistant" as const }, // optimistic
    ];
    const next = [
      { id: "h0_abc", role: "user" as const },
      { id: "h1_zzz", role: "assistant" as const },
    ];
    rekeyTrailingOptimisticIds(existing, next);
    expect(next[1]!.id).toBe("abc-uuid-1");
    expect(next[0]!.id).toBe("h0_abc");
  });

  it("reuses a recovered thinking card id when the reply arrives", () => {
    const existing = [
      { id: "h0_abc", role: "user" as const },
      { id: "_background", role: "assistant" as const },
    ];
    const next = [
      { id: "h0_abc", role: "user" as const },
      { id: "h1_reply", role: "assistant" as const },
    ];

    rekeyTrailingOptimisticIds(existing, next);

    expect(next[1]!.id).toBe("_background");
  });

  it("stops walking when roles diverge", () => {
    // Role mismatch indicates structural disagreement; do not rewrite further.
    const existing = [
      { id: "uuid-1", role: "user" as const },
      { id: "uuid-2", role: "user" as const },
    ];
    const next = [
      { id: "h0_a", role: "user" as const },
      { id: "h1_b", role: "assistant" as const }, // role differs from tail of existing
    ];
    rekeyTrailingOptimisticIds(existing, next);
    expect(next[0]!.id).toBe("h0_a");
    expect(next[1]!.id).toBe("h1_b");
  });

  it("loads older messages only for explicit upward scroll near top", () => {
    expect(
      shouldLoadOlderPublicMessages({
        scrollTop: 12,
        previousScrollTop: 80,
        distanceFromBottom: 600,
        hasOlderMessages: true,
        loadingOlderMessages: false,
        sendingOrStreaming: false,
      }),
    ).toBe(true);
    expect(
      shouldLoadOlderPublicMessages({
        scrollTop: 0,
        previousScrollTop: 0,
        distanceFromBottom: 900,
        hasOlderMessages: true,
        loadingOlderMessages: false,
        sendingOrStreaming: false,
      }),
    ).toBe(false);
    expect(
      shouldLoadOlderPublicMessages({
        scrollTop: 10,
        previousScrollTop: 80,
        distanceFromBottom: 600,
        hasOlderMessages: true,
        loadingOlderMessages: false,
        sendingOrStreaming: true,
      }),
    ).toBe(false);
  });

  it("recovers accidental top jumps while pinned to the newest message", () => {
    expect(
      shouldRecoverPinnedBottom({
        scrollTop: 0,
        distanceFromBottom: 1800,
        pinnedToBottom: true,
      }),
    ).toBe(true);
    expect(
      shouldRecoverPinnedBottom({
        scrollTop: 0,
        distanceFromBottom: 1800,
        pinnedToBottom: false,
      }),
    ).toBe(false);
    expect(
      shouldRecoverPinnedBottom({
        scrollTop: 48,
        distanceFromBottom: 1800,
        pinnedToBottom: true,
      }),
    ).toBe(false);
  });
});
