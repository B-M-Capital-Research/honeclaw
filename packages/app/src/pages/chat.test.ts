import { describe, expect, it } from "bun:test";
import {
  normalizeInviteCode,
  normalizePhoneNumber,
  resolvePublicChatView,
  toPublicChatMessages,
} from "@/lib/public-chat";
import type { HistoryMsg } from "@/lib/types";

describe("normalizeInviteCode", () => {
  it("strips whitespace and normalizes to uppercase", () => {
    expect(normalizeInviteCode(" hone-a77b3f-\nc162dd \t")).toBe("HONE-A77B3F-C162DD");
  });
});

describe("normalizePhoneNumber", () => {
  it("keeps a leading plus and strips non-digits", () => {
    expect(normalizePhoneNumber(" +86 138-0013-8000 ")).toBe("+8613800138000");
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
      { role: "user", content: "我是 zxr" },
      { role: "assistant", content: "已记住，你是 zxr。" },
    ];

    const first = toPublicChatMessages(history);
    const second = toPublicChatMessages(history);

    expect(first.map((message) => message.id)).toEqual(
      second.map((message) => message.id),
    );
  });

  it("preserves existing prefix ids when new history rows are appended", () => {
    const baseHistory: HistoryMsg[] = [
      { role: "user", content: "第一个问题" },
      { role: "assistant", content: "第一个回答" },
    ];

    const nextHistory: HistoryMsg[] = [
      ...baseHistory,
      { role: "user", content: "第二个问题" },
      { role: "assistant", content: "第二个回答" },
    ];

    const base = toPublicChatMessages(baseHistory);
    const next = toPublicChatMessages(nextHistory);

    expect(next.slice(0, base.length).map((message) => message.id)).toEqual(
      base.map((message) => message.id),
    );
  });
});
