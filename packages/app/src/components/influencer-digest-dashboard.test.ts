import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const component = readFileSync(
  new URL("./influencer-digest-dashboard.tsx", import.meta.url),
  "utf8",
);
const styles = readFileSync(
  new URL("./influencer-digest-dashboard.css", import.meta.url),
  "utf8",
);

describe("influencer digest dashboard", () => {
  it("keeps source and opinion boundaries", () => {
    expect(component).toContain("大V速报");
    expect(component).toContain("作者观点不等于事实或 HONE 结论");
    expect(component).toContain("查看作者原文");
    expect(component).toContain("翻译/聚合源");
    expect(component).toContain("聚合翻译不是独立事实来源");
    expect(component).toContain("不得补造未配置作者内容");
  });

  it("is cached and scheduled", () => {
    expect(component).toContain("每日 19:50 更新");
    expect(component).toContain("getPublicInfluencerDigest");
    expect(component).not.toContain("生成速报");
  });

  it("is a controlled research panel without its own launcher or modal chrome", () => {
    expect(component).toContain("export function InfluencerDigestPanel");
    expect(component).not.toContain("InfluencerDigestDashboard");
    expect(component).toContain("ResearchPanel");
    expect(component).toContain('backdropClass="influencer-digest-backdrop"');
    expect(component).toContain('dialogClass="influencer-digest-dialog"');
    expect(component).toContain("props.onClose()");
    // The shared shell owns Portal / backdrop / Escape / aria-modal.
    expect(component).not.toContain("Portal");
    expect(component).not.toContain("aria-modal");
    expect(component).not.toContain("influencer-digest-launcher");
    expect(component).not.toContain("setOpen");
    expect(styles).not.toContain("influencer-digest-launcher");
  });

  it("routes states and the ask prompt through the shared research kit", () => {
    expect(component).toContain("ResearchState");
    expect(component).toContain('kind="error"');
    expect(component).toContain("onRetry={() => void load()}");
    expect(component).toContain("buildSavedReportPrompt");
    expect(component).toContain('marker: "HONE_SAVED_INFLUENCER_DIGEST"');
    // No onAsk → no ask footer.
    expect(component).toContain("onAsk?: (message: string) => void");
    expect(component).toContain("<Show when={props.onAsk}>");
  });

  it("uses design tokens in readable multi-line CSS for mobile and dark", () => {
    expect(styles).toContain("@media (max-width: 768px)");
    expect(styles).not.toContain("@media(max-width:768px)");
    expect(styles).toContain("var(--hone-paper-50)");
    expect(styles).toContain("var(--hone-coral-600)");
    expect(styles).toContain("var(--hone-signal-yellow-soft)");
    expect(styles).toContain("var(--hone-line)");
    // Dark mode rides the tokens; the old slate dark skin is gone.
    expect(styles).not.toContain("#111a28");
    expect(styles).not.toContain("[data-theme=dark]");
    // Mapped literals must not resurface.
    expect(styles).not.toContain("#c35b46");
    expect(styles).not.toContain("#b65340");
    expect(styles).not.toContain("#fff4d8");
    expect(styles).not.toContain("#fff6e3");
  });
});
