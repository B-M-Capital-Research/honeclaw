import { describe, expect, test } from "bun:test";

import {
  ShareRenderError,
  canSharePngFile,
  canvasToPngBlob,
  isLikelyIOSPlatform,
  isShareAbortError,
  isShareRenderError,
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
});
