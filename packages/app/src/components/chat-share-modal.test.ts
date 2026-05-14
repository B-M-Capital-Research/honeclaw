import { describe, expect, test } from "bun:test";

import {
  ShareRenderError,
  canvasToPngBlob,
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
});
