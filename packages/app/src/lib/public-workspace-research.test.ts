import { describe, expect, it } from "bun:test";
import { publicWorkspaceResearchFromHistory } from "./public-workspace-research";

describe("public workspace research history", () => {
  it("uses stable timeline ids and keeps the newest user prompts first", () => {
    const research = publicWorkspaceResearchFromHistory(
      [
        { role: "user", content: "第一项研究", attachments: [] },
        { role: "assistant", content: "回答", attachments: [] },
        { role: "user", content: "第二项研究", attachments: [] },
      ],
      40,
    );

    expect(research.map((item) => item.title)).toEqual([
      "第二项研究",
      "第一项研究",
    ]);
    expect(research.every((item) => item.id.length > 0)).toBe(true);
    expect(research[0]?.id).not.toBe(research[1]?.id);
  });

  it("uses the attachment fallback for attachment-only prompts", () => {
    expect(
      publicWorkspaceResearchFromHistory(
        [{ role: "user", content: "[附件: report.pdf]", attachments: [] }],
        0,
        6,
        "附件研究",
      )[0]?.title,
    ).toBe("附件研究");
  });
});
