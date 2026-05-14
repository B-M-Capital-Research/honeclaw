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

export function isLikelyIOSPlatform(platform: string, maxTouchPoints: number) {
  return (
    /iPad|iPhone|iPod/.test(platform) ||
    (platform === "MacIntel" && maxTouchPoints > 1)
  );
}

export function canSharePngFile(
  nav: Pick<Navigator, "canShare"> | undefined,
  file: File,
) {
  if (!nav || typeof nav.canShare !== "function") return false;
  try {
    return nav.canShare({ files: [file] });
  } catch {
    return false;
  }
}

export function recentShareMessages<T>(
  messages: readonly T[],
  limit = 4,
  seedIndex = messages.length - 1,
): T[] {
  if (messages.length === 0) return [];
  const safeLimit = Math.max(1, limit);
  const endIndex = Math.min(
    messages.length - 1,
    Math.max(0, Math.trunc(seedIndex)),
  );
  return messages.slice(Math.max(0, endIndex - safeLimit + 1), endIndex + 1);
}

export function defaultShareMessageId<T extends { id: string }>(
  messages: readonly T[],
) {
  return messages[messages.length - 1]?.id ?? null;
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
