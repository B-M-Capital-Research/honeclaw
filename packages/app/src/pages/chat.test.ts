import { describe, expect, it } from "bun:test";
import {
  normalizeInviteCode,
  normalizePhoneNumber,
  resolvePublicChatView,
} from "./chat";

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
