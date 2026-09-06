import {
  normalizeMathToPlainText,
  parseMarkdown,
} from "@hone-financial/ui/markdown-utils";
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

  test("renders LaTeX formulas as readable plain-text arithmetic", async () => {
    // 生产终稿里的原文：`$` 既是货币符号又被当成数学分隔符，装数学引擎也解析不了。
    const html = await parseMarkdown(
      "$$\\text{目标价} = $38.00 \\times 32\\text{x} = \\mathbf{$1,216.00}$$",
    );
    const root = document.createElement("div");
    root.innerHTML = html;

    expect(root.textContent).toContain("目标价 = $38.00 × 32x = $1,216.00");
    expect(root.querySelector("strong")?.textContent).toBe("$1,216.00");
    expect(root.textContent).not.toContain("\\text");
    expect(root.textContent).not.toContain("$$");
  });

  test("turns fractions into a division a reader can follow", () => {
    expect(
      normalizeMathToPlainText("$$\\frac{$881.255}{$33.17} = \\mathbf{26.57\\text{x}}$$"),
    ).toBe("$881.255 / $33.17 = **26.57x**");
    expect(
      normalizeMathToPlainText(
        "$$\\frac{$780.16\\text{B}}{$93.71\\text{B}} = \\mathbf{8.32\\text{x}}$$",
      ),
    ).toBe("$780.16B / $93.71B = **8.32x**");
    // 分子含运算符时补括号，免得读成 a + b/c。
    expect(normalizeMathToPlainText("$$\\frac{a + b}{c}$$")).toBe("(a + b) / c");
  });

  test("leaves currency, prose backslashes and code untouched", () => {
    // 单个 `$` 永远是货币，不是数学分隔符。
    const currency = "现价 $881.26，市值 $790.49B，较 $38.00 上涨";
    expect(normalizeMathToPlainText(currency)).toBe(currency);
    // 只有「单个命令」这种不可能是金额的写法才翻译。
    expect(normalizeMathToPlainText("6.72 $\\rightarrow$ 6.30")).toBe("6.72 → 6.30");
    // 代码块里的反斜杠是内容。
    const code = "```bash\nprintf '$$\\times$$'\n```";
    expect(normalizeMathToPlainText(code)).toBe(code);
    expect(normalizeMathToPlainText("行内 `\\frac{a}{b}` 保持原样")).toBe(
      "行内 `\\frac{a}{b}` 保持原样",
    );
  });

  test("keeps paired tildes from striking an entire assistant paragraph", async () => {
    const html = await parseMarkdown(
      "Base Case：~~收入约 10 亿美元，估值区间保持不变~~",
    );
    const root = document.createElement("div");
    root.innerHTML = html;

    expect(root.querySelector("del")).toBeNull();
    expect(root.textContent).toContain(
      "Base Case：收入约 10 亿美元，估值区间保持不变",
    );
  });
});
