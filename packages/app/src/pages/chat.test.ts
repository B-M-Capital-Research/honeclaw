import { describe, expect, it } from "bun:test";
import { displayGithubStars, formatGithubStars, GITHUB_STARS_FALLBACK } from "@/lib/github-stars";
import {
  canSendPublicChatMessage,
  nextVisibleMessageCount,
  formatPublicAttachmentBytes,
  isPublicChatQuotaExhausted,
  normalizePhoneNumber,
  publicAttachmentFileLabel,
  rekeyTrailingOptimisticIds,
  resolvePublicChatView,
  selectVisibleRecentMessages,
  shouldRecoverPinnedBottom,
  shouldLoadOlderPublicMessages,
  splitPublicChatAttachments,
  stripAttachmentMarkers,
  toPublicChatMessages,
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

    const base = toPublicChatMessages(baseHistory);
    const next = toPublicChatMessages(nextHistory);

    expect(messageIds(next.slice(0, base.length))).toEqual(messageIds(base));
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
    expect(messages.every((message) => Array.isArray(message.attachments))).toBe(
      true,
    );
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
    const base = {
      draft: "",
      attachmentCount: 0,
      isSending: false,
      uploading: false,
      remaining: undefined,
      dailyLimit: undefined,
    };

    expect(canSendPublicChatMessage(base)).toBe(false);
    expect(canSendPublicChatMessage({ ...base, draft: "  hello " })).toBe(true);
    expect(canSendPublicChatMessage({ ...base, attachmentCount: 1 })).toBe(true);
    expect(
      canSendPublicChatMessage({ ...base, draft: "hello", isSending: true }),
    ).toBe(false);
    expect(
      canSendPublicChatMessage({ ...base, draft: "hello", uploading: true }),
    ).toBe(false);
    expect(
      canSendPublicChatMessage({
        ...base,
        draft: "hello",
        remaining: 0,
        dailyLimit: 10,
      }),
    ).toBe(false);
    expect(
      canSendPublicChatMessage({
        ...base,
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

describe("public chat history window", () => {
  it("starts from the most recent messages", () => {
    expect(selectVisibleRecentMessages([1, 2, 3, 4, 5], 3)).toEqual([
      3, 4, 5,
    ]);
    expect(selectVisibleRecentMessages([1, 2], 10)).toEqual([1, 2]);
  });

  it("expands the visible window without exceeding total history", () => {
    expect(nextVisibleMessageCount(100, 24, 24)).toBe(48);
    expect(nextVisibleMessageCount(40, 24, 24)).toBe(40);
    expect(nextVisibleMessageCount(10, -1, 24)).toBe(10);
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
