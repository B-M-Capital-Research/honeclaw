import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(new URL("./position-management-dashboard.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("./position-management-dashboard.css", import.meta.url), "utf8");

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

  it("never claims execution when sending the saved report", () => {
    expect(component).toContain("buildSavedReportPrompt");
    expect(component).toContain('marker: "HONE_SAVED_POSITION_MANAGEMENT_REPORT"');
    expect(component).toContain("不得自动修改持仓或声称已经下单");
    expect(component).toContain("不得补造缺失行情、财务、估值或新闻");
    // Ask footer only exists when a chat sink is provided, and asking closes.
    expect(component).toContain("<Show when={props.onAsk}>");
    expect(component).toContain("props.onClose()");
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
    expect(styles).toContain("94dvh");
  });
});
