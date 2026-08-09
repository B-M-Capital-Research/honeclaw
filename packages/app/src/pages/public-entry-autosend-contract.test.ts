import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const chat = readFileSync(new URL("./chat.tsx", import.meta.url), "utf8");
const holdings = readFileSync(
  new URL("../components/public-holdings-panel.tsx", import.meta.url),
  "utf8",
);

describe("an entry point that already knows the question sends it", () => {
  it("asks the chat to send rather than to prefill", () => {
    // Tapping "问问财报" on a holding used to land on the composer with the
    // text typed in, so the user pressed send a second time for something
    // they had just chosen.
    expect(holdings).toContain("&send=1");
    expect(holdings).toContain("holdingAskPrompt(row, kind)");
  });

  it("queues the question instead of dropping it while the session settles", () => {
    // sendChatTurn returns silently when auth is not ready or another turn is
    // in flight. Calling it straight from the URL effect would lose the
    // question on a cold load, which is exactly when someone arrives by link.
    expect(chat).toContain("const [pendingAutoSend, setPendingAutoSend]");
    expect(chat).toContain('searchParams.send === "1"');
    expect(chat).toContain("setPendingAutoSend(prefill)");
  });

  it("dispatches once the turn can actually start, and only once", () => {
    const dispatch = chat.slice(chat.indexOf("const text = pendingAutoSend();"));
    expect(dispatch).toContain('authState() !== "ready"');
    expect(dispatch).toContain("isSendingOrStreaming()");
    expect(dispatch).toContain("uploading()");
    // Clearing before dispatch is what stops the effect from re-firing.
    expect(dispatch.indexOf("setPendingAutoSend(undefined)")).toBeLessThan(
      dispatch.indexOf("void sendChatTurn("),
    );
  });

  it("clears both parameters so a refresh does not send again", () => {
    // Leaving `send=1` in the address bar would resend on every reload.
    expect(chat).toContain("setSearchParams({ q: undefined, send: undefined }");
  });

  it("still only prefills when no send was requested", () => {
    // Other callers pass `q` alone and expect the composer to stay editable.
    const effect = chat.slice(chat.indexOf("const prefill = searchParams.q;"));
    expect(effect).toContain("setDraft(prefill)");
    expect(effect).toContain("focusWorkspaceComposer()");
  });
});
