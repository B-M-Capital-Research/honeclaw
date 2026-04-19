import { describe, expect, it } from "bun:test";
import { normalizeInviteCode } from "./chat";

describe("normalizeInviteCode", () => {
  it("strips whitespace and normalizes to uppercase", () => {
    expect(normalizeInviteCode(" hone-a77b3f-\nc162dd \t")).toBe("HONE-A77B3F-C162DD");
  });
});
