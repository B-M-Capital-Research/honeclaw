import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(new URL("./position-management-dashboard.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("./position-management-dashboard.css", import.meta.url), "utf8");

/**
 * Source with comments stripped, so the "not contain" pins below can only be
 * satisfied by real code — a stale comment must not keep a contract green.
 */
const code = component.replace(/\/\*[\s\S]*?\*\//g, "").replace(/^\s*\/\/.*$/gm, "");
const headStart = code.indexOf("<ResearchPanelHead");
const panelHead = code.slice(headStart, code.indexOf("/>", headStart) + 2);
const propsStart = code.indexOf("type Props = {");
const propsType = code.slice(propsStart, code.indexOf("};", propsStart) + 2);

describe("position management dashboard contract", () => {
  it("exposes the cached report as a controlled research panel", () => {
    expect(component).toContain("export function PositionManagementPanel");
    expect(component).toContain("<ResearchPanel");
    expect(component).toContain('dialogClass="position-management-dialog"');
    expect(component).toContain('backdropClass="position-management-backdrop"');
    expect(component).toContain("仓位管理建议");
    expect(component).toContain("每日 20:00 更新");
    expect(component).toContain("getPublicPositionManagement");
    expect(component).not.toContain("生成建议");
    // The launcher chip moved to the research desk; the panel owns no trigger.
    expect(component).not.toContain("position-management-launcher");
    expect(component).not.toContain('from "solid-js/web"');
  });

  it("leads with the verdict instead of a title-then-metadata preamble", () => {
    // How many positions need handling, the evidence-completeness light and
    // one sentence sit in the shared head; the bespoke header, the metadata
    // strip and the duplicated summary paragraph are gone.
    expect(component).toContain("ResearchPanelHead");
    expect(component).toContain('id="position-management-title"');
    expect(component).toContain("provenanceLine");
    expect(component).toContain("项待处理");
    expect(component).not.toContain("position-management-meta");
    expect(styles).not.toContain(".position-management-meta");
    expect(styles).not.toContain("> header");
  });

  it("delegates modal behaviour and states to the shared research shell", () => {
    // ESC / aria-modal / scroll lock live in ResearchPanel, not here.
    expect(component).not.toContain('aria-modal="true"');
    expect(component).not.toContain('"Escape"');
    expect(component).toContain("<ResearchState");
    expect(component).toContain('kind="loading"');
    expect(component).toContain('kind="error"');
    expect(component).toContain("onRetry={() => void load()}");
    expect(component).toContain('kind="empty"');
  });

  it("keeps Hari logic separate from HONE concentration controls", () => {
    expect(component).toContain("framework_version");
    expect(component).toContain("methodology_note");
    expect(component).toContain("证伪条件");
    expect(component).toContain("数据日期");
  });

  it("uses hone design tokens with mobile and dark surfaces", () => {
    expect(styles).toContain("var(--hone-ink-950)");
    expect(styles).toContain("var(--hone-ink-500)");
    expect(styles).toContain("var(--hone-signal-green)");
    expect(styles).toContain("var(--hone-signal-red)");
    expect(styles).toContain("var(--hone-signal-yellow");
    expect(styles).toContain("var(--hone-signal-green-soft)");
    expect(styles).toContain("var(--hone-signal-neutral-soft)");
    expect(styles).not.toContain("#c84f43");
    expect(styles).not.toContain("#23845a");
    expect(styles).not.toContain("#28785b");
    expect(styles).not.toContain("position-management-launcher");
    expect(styles).toContain("@media (max-width: 768px)");
    // Sheet mode: both count strips collapse so advice rows clear the fold.
    expect(styles).toContain("@media (max-width: 760px)");
    expect(styles).toContain('[data-theme="dark"]');
    // Sheet height is the shared shell's (`max-height: min(86vh, 920px)`,
    // `100dvh` on phones); a second dvh cap here would race it.
    expect(styles).not.toContain("dvh");
  });

  it("has no manual refresh button and no ask-the-chat footer", () => {
    // The head is kicker + close. Panels read one saved snapshot when they
    // open, so there is no manual refresh control to press and no stale
    // "read again" affordance implying the numbers can be recomputed here.
    expect(panelHead).not.toContain("action=");
    expect(code).not.toContain("重新读取");
    expect(code).not.toContain("读取中…");
    // 「发送到对话」is gone end to end: no composer in the footer, no prompt
    // envelope, and no chat sink on the props.
    expect(propsType).toBe("type Props = {\n  onClose: () => void;\n};");
    expect(code).not.toContain("onAsk");
    expect(code).not.toContain("发送到对话");
    expect(code).not.toContain("buildSavedReportPrompt");
    expect(code).not.toContain("HONE_SAVED");
    // load() survives for exactly two callers: first paint and error retry.
    expect(code).toContain("onMount(() => void load())");
  });
});
