import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import {
  ShareRenderError,
  canSharePngFile,
  canvasToPngBlob,
  defaultShareMessageId,
  isLikelyIOSPlatform,
  isShareAbortError,
  isShareRenderError,
  recentShareMessages,
  shareDateLabel,
  sharePickerPreview,
  shareQuestionStyle,
  shareTextForClipboard,
} from "./chat-share-export";

async function expectCanvasEncodingError(
  canvas: HTMLCanvasElement,
): Promise<unknown> {
  try {
    await canvasToPngBlob(canvas);
  } catch (error) {
    return error;
  }
  throw new Error("expected canvasToPngBlob to fail");
}

describe("chat share export errors", () => {
  test("renders the shared question as a quiet left-aligned block", () => {
    const style = shareQuestionStyle(15);

    expect(style["text-align"]).toBe("left");
    expect(style["align-self"]).toBe("flex-end");
    expect(style["font-size"]).toBe("15px");
    // html2canvas cannot evaluate color-mix(); the fills stay literal.
    expect(style.background).toMatch(/^#[0-9a-f]{6}$/);
  });

  test("copies an exchange as labelled plain text with one attribution line", () => {
    const text = shareTextForClipboard(
      [
        { role: "user", content: "  戴尔超预期吗 " },
        { role: "assistant", content: "结论：营收与 EPS 双超预期。" },
      ],
      { user: "我", assistant: "HONE" },
      "—— 来自 HONE",
    );
    expect(text).toBe("我：戴尔超预期吗\n\nHONE：结论：营收与 EPS 双超预期。\n\n—— 来自 HONE");
  });

  test("previews a message as prose, without markdown syntax", () => {
    expect(
      sharePickerPreview("**结论：营收与 EPS 双超预期。** 盘后股价\n\n## 已核验事实\n\n- 营收 `297.8 亿`\n1. 若冲高"),
    ).toBe("结论：营收与 EPS 双超预期。 盘后股价 已核验事实 营收 297.8 亿 若冲高");
    expect(sharePickerPreview("   ")).toBe("—");
    expect(sharePickerPreview("x".repeat(100), 20)).toBe(`${"x".repeat(20)}…`);
  });

  test("stamps the card with a zero-padded local date", () => {
    expect(shareDateLabel(new Date(2026, 8, 6))).toBe("2026-09-06");
  });

  test("reports canvas encoding failures as render errors", async () => {
    const canvas = {
      toBlob(callback: BlobCallback) {
        callback(null);
      },
    } as HTMLCanvasElement;

    const error = await expectCanvasEncodingError(canvas);
    expect(error).toBeInstanceOf(ShareRenderError);
    expect(isShareRenderError(error)).toBe(true);
  });

  test("encodes share images as png blobs", async () => {
    const expected = new Blob(["png"], { type: "image/png" });
    let requestedType = "";
    const canvas = {
      toBlob(callback: BlobCallback, type?: string) {
        requestedType = type ?? "";
        callback(expected);
      },
    } as HTMLCanvasElement;

    await expect(canvasToPngBlob(canvas)).resolves.toBe(expected);
    expect(requestedType).toBe("image/png");
  });

  test("recognizes browser share cancellation errors", () => {
    const abortError = new Error("The user aborted a request.");
    abortError.name = "AbortError";

    expect(isShareAbortError(abortError)).toBe(true);
    expect(isShareAbortError({ name: "AbortError" })).toBe(true);
    expect(isShareAbortError(new Error("clipboard denied"))).toBe(false);
  });

  test("detects iOS and touch iPad platforms", () => {
    expect(isLikelyIOSPlatform("iPhone", 0)).toBe(true);
    expect(isLikelyIOSPlatform("iPad", 0)).toBe(true);
    expect(isLikelyIOSPlatform("MacIntel", 5)).toBe(true);
    expect(isLikelyIOSPlatform("MacIntel", 0)).toBe(false);
    expect(isLikelyIOSPlatform("Win32", 10)).toBe(false);
  });

  test("guards file sharing capability checks", () => {
    const file = new File(["png"], "share.png", { type: "image/png" });
    expect(canSharePngFile(undefined, file)).toBe(false);
    expect(
      canSharePngFile(
        {
          canShare(data?: ShareData) {
            return data?.files?.[0]?.type === "image/png";
          },
        } as Pick<Navigator, "canShare">,
        file,
      ),
    ).toBe(true);
    expect(
      canSharePngFile(
        {
          canShare() {
            throw new Error("unsupported");
          },
        } as Pick<Navigator, "canShare">,
        file,
      ),
    ).toBe(false);
  });

  test("limits the picker to the latest four messages by default", () => {
    const messages = [
      { id: "m1" },
      { id: "m2" },
      { id: "m3" },
      { id: "m4" },
      { id: "m5" },
    ];

    const recent = recentShareMessages(messages, 4);

    expect(recent.map((message) => message.id)).toEqual([
      "m2",
      "m3",
      "m4",
      "m5",
    ]);
    expect(defaultShareMessageId(recent)).toBe("m5");
    expect(defaultShareMessageId([])).toBeNull();
  });

  test("uses the clicked message as the final share picker item", () => {
    const messages = [
      { id: "m1" },
      { id: "m2" },
      { id: "m3" },
      { id: "m4" },
      { id: "m5" },
      { id: "m6" },
    ];

    const recent = recentShareMessages(messages, 4, 3);

    expect(recent.map((message) => message.id)).toEqual([
      "m1",
      "m2",
      "m3",
      "m4",
    ]);
    expect(defaultShareMessageId(recent)).toBe("m4");
  });
});

describe("share card raster safety", () => {
  const css = readFileSync(new URL("./chat-share.css", import.meta.url), "utf8");
  const rule = (selector: string) => {
    const start = css.indexOf(`${selector} {`);
    expect(start).toBeGreaterThan(-1);
    return css.slice(start, css.indexOf("}", start));
  };

  test("leaves inline code unpainted so a wrapped span cannot cover its neighbours", () => {
    // html2canvas paints an inline element from one bounding rectangle. A
    // padded background therefore sits above its own text, and a span that
    // wraps across two lines is drawn as a single rectangle over the words
    // beside it — a shared answer lost a line of text that way.
    const inlineCode = rule(".hf-share-card-md :not(pre) > code");
    expect(inlineCode).toContain("background: transparent");
    expect(inlineCode).toContain("padding: 0;");
    expect(inlineCode).not.toContain("border-radius");
    expect(inlineCode).toContain("font-family: var(--hone-font-label)");
  });

  test("keeps the fenced code block framed, since block elements rasterise correctly", () => {
    const fenced = rule(".hf-share-card-md .hf-markdown-code pre,\n.hf-share-card-md .hf-markdown-code pre.shiki");
    expect(fenced).toContain("background: #f1f0eb");
    expect(fenced).toContain("border: 1px solid");
  });
});
