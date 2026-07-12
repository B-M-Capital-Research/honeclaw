import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const css = readFileSync(new URL("./public-chat.css", import.meta.url), "utf8");
const nav = readFileSync(
  new URL("../components/public-nav.tsx", import.meta.url),
  "utf8",
);
const chat = readFileSync(new URL("./chat.tsx", import.meta.url), "utf8");

describe("public chat visual contract", () => {
  it("uses the flat mobile chat header instead of the floating site pill", () => {
    expect(nav).toContain('"is-chat-mode": props.chatMode');
    expect(nav).toContain("pub-nav-chat-copy");
    expect(css).toContain("border-radius: 0 !important");
    expect(css).toContain("padding-top: 58px !important");
    expect(css).toContain("align-items: center");
    expect(css).toContain("flex: 0 0 30px");
  });

  it("keeps quick actions on a horizontally scrollable mobile line", () => {
    expect(css).toContain("flex-wrap: nowrap !important");
    expect(css).toContain("overflow-x: auto !important");
    expect(css).toContain("scrollbar-width: none !important");
    expect(css).toContain("white-space: nowrap !important");
    expect(css).toContain("background: rgba(255, 255, 255, 0.99) !important");
    expect(css).toContain("backdrop-filter: none !important");
  });

  it("keeps assistant responses flat and user prompts softly filled", () => {
    const finalLayer = css.slice(css.indexOf("Final mobile chat overrides"));
    expect(finalLayer).toContain("pub-msg-bubble--assistant");
    expect(finalLayer).toContain("background: transparent !important");
    expect(finalLayer).toContain("pub-msg-bubble--user");
    expect(finalLayer).toContain("background: #f1f1ef !important");
    expect(finalLayer).toContain('[data-theme="dark"]');
  });

  it("removes the heavy composer shadow in the final mobile layer", () => {
    const finalLayer = css.slice(css.indexOf("Final mobile chat overrides"));
    expect(finalLayer).toContain("public-chat-composer-box");
    expect(finalLayer).toContain("box-shadow: none !important");
  });

  it("uses one HONE font contract across desktop chat controls", () => {
    expect(css).toContain("public-chat-sidebar-logout");
    expect(css).toContain("public-chat-proactive-tip");
    expect(css).toContain("font-family: var(--hone-font-body) !important");
    expect(css).toContain("font-weight: 700 !important");
  });

  it("keeps the desktop calendar dialog above the chat shell and inside the viewport", () => {
    const calendar = chat.slice(chat.indexOf("function FinanceCalendarQuickAction"));
    expect(calendar).toContain("<Portal>");
    expect(css).toContain("height: min(780px, calc(100dvh - 32px))");
    expect(css).toContain("grid-template-rows: auto minmax(0, 1fr)");
    expect(css).toContain(".public-chat-calendar-modal-body {\n  min-height: 0;");
  });
});
