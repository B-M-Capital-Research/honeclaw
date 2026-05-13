import { describe, expect, it } from "bun:test";
import { displayGithubStars, formatGithubStars, GITHUB_STARS_FALLBACK } from "@/lib/github-stars";
import {
  nextVisibleMessageCount,
  normalizePhoneNumber,
  resolvePublicChatView,
  selectVisibleRecentMessages,
  shouldLoadOlderPublicMessages,
  stripAttachmentMarkers,
  toPublicChatMessages,
} from "@/lib/public-chat";
import type { HistoryMsg } from "@/lib/types";

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

    expect(first.map((message) => message.id)).toEqual(
      second.map((message) => message.id),
    );
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

    expect(next.slice(0, base.length).map((message) => message.id)).toEqual(
      base.map((message) => message.id),
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
    expect(messages.every((message) => Array.isArray(message.attachments))).toBe(
      true,
    );
  });
});

describe("stripAttachmentMarkers", () => {
  it("treats absent content as empty text", () => {
    expect(stripAttachmentMarkers(undefined)).toBe("");
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
});
