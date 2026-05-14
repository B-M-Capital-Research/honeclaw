export class ShareRenderError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ShareRenderError";
  }
}

export function isShareRenderError(error: unknown) {
  return error instanceof ShareRenderError;
}

export function isShareAbortError(error: unknown) {
  return (
    typeof error === "object" &&
    error !== null &&
    "name" in error &&
    (error as { name?: unknown }).name === "AbortError"
  );
}

export async function canvasToPngBlob(canvas: HTMLCanvasElement) {
  return new Promise<Blob>((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob) {
        resolve(blob);
      } else {
        reject(new ShareRenderError("Browser failed to encode share image"));
      }
    }, "image/png");
  });
}
