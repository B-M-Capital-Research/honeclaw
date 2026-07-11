import { describe, expect, it } from "bun:test";
import { displayGithubStars, formatGithubStars, GITHUB_STARS_FALLBACK } from "@/lib/github-stars";
import {
  canSendPublicChatMessage,
  findPendingPublicAssistantMessage,
  formatPublicAttachmentBytes,
  isPublicChatQuotaExhausted,
  latestUnreadPushId,
  mergePublicPushItems,
  mergePublicHistoryWindow,
  normalizePhoneNumber,
  PUBLIC_RESTORE_MAX_ATTEMPTS,
  publicRestoreRetryDelay,
  publicComposerPendingMessage,
  publicAttachmentFileLabel,
  rekeyTrailingOptimisticIds,
  resolvePublicChatView,
  shouldRetryPublicRestore,
  shouldRecoverPinnedBottom,
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

  it("falls back to background pending state for the composer strip", () => {
    expect(
      publicComposerPendingMessage({
        local: undefined,
        background: { since: 1778749318381 },
      }),
    ).toMatchObject({
      id: "_background",
      role: "assistant",
      phase: "thinking",
      startedAt: 1778749318381,
    });

    const local = {
      id: "local",
      role: "assistant" as const,
      content: "streaming",
      phase: "streaming" as const,
    };
    expect(
      publicComposerPendingMessage({
        local,
        background: { since: 1 },
      }),
    ).toBe(local);
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
