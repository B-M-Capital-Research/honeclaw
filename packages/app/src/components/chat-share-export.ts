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

/**
 * The shared question is a quiet, left-aligned block — the same shape it has
 * in the conversation — rather than a dark centred bubble that reads as a
 * different product from the answer beneath it. Colours are literals: the
 * card is rasterised by html2canvas, which cannot evaluate color-mix().
 */
export function shareQuestionStyle(fontSize: number) {
  return {
    "max-width": "88%",
    "align-self": "flex-end",
    background: "#f1f0eb",
    color: "#17201f",
    padding: "9px 13px",
    "border-radius": "12px 12px 4px 12px",
    "font-size": `${fontSize}px`,
    "line-height": "1.55",
    "white-space": "pre-wrap",
    "text-align": "left",
    "word-break": "break-word",
    "box-sizing": "border-box",
  } as const;
}

/**
 * Plain-text form of a shared exchange for the clipboard: speaker labels,
 * blank lines between turns, and one attribution line at the end so a pasted
 * answer still says where it came from.
 */
export function shareTextForClipboard(
  messages: readonly { role: "user" | "assistant"; content: string }[],
  labels: { user: string; assistant: string },
  footer: string,
): string {
  const turns = messages
    .map((message) => {
      const label = message.role === "user" ? labels.user : labels.assistant;
      return `${label}：${message.content.trim()}`;
    })
    .filter((line) => line.length > 0);
  return [...turns, footer].join("\n\n");
}

/**
 * One line of a message for the picker: markdown syntax removed so a row
 * reads as prose ("结论：营收与 EPS 双超预期…"), not as source ("**结论…**").
 */
export function sharePickerPreview(content: string, limit = 90): string {
  const text = content
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/`([^`]*)`/g, "$1")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, " ")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/^\s{0,3}(#{1,6}\s+|>\s?|[-*+]\s+|\d+\.\s+)/gm, "")
    .replace(/(\*\*|__)(.*?)\1/g, "$2")
    .replace(/(\*|_)(.*?)\1/g, "$2")
    .replace(/\|/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  if (!text) return "—";
  return text.length > limit ? `${text.slice(0, limit)}…` : text;
}

/** Date stamp on the card, in the reader's local time. */
export function shareDateLabel(now = new Date()) {
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${now.getFullYear()}-${month}-${day}`;
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
