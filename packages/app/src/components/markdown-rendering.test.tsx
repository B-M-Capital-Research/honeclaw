import { parseMarkdown } from "@hone-financial/ui/markdown-utils";
import { describe, expect, test } from "bun:test";

describe("Markdown rendering", () => {
  test("renders common markdown syntax as semantic HTML", async () => {
    const html = await parseMarkdown("**Bold**\n\n1. First\n2. Second\n\n- Bullet");
    const root = document.createElement("div");
    root.innerHTML = html;

    expect(root.querySelector("strong")?.textContent).toBe("Bold");
    expect(
      [...root.querySelectorAll("ol > li")].map((item) => item.textContent),
    ).toEqual(["First", "Second"]);
    expect(root.querySelector("ul > li")?.textContent).toBe("Bullet");
  });
});
