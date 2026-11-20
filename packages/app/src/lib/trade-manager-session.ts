export type TradeManagerSessionState =
  | { status: "closed" | "checking" | "ready" }
  | { status: "error"; reason: "identity" | "unavailable" };

/** An iframe lease is valid for one opening and one authenticated owner only. */
export function createTradeManagerSession(
  verify: (signal: AbortSignal) => Promise<{ user_id: string }>,
  update: (state: TradeManagerSessionState) => void,
) {
  let generation = 0;
  let controller: AbortController | undefined;
  const close = () => {
    generation += 1;
    controller?.abort();
    controller = undefined;
    update({ status: "closed" });
  };
  return {
    close,
    async open(expectedUserId: string) {
      close();
      if (!expectedUserId) return;
      const requested = generation;
      const request = new AbortController();
      controller = request;
      update({ status: "checking" });
      try {
        const user = await verify(request.signal);
        if (request.signal.aborted || generation !== requested) return;
        update(user.user_id === expectedUserId
          ? { status: "ready" }
          : { status: "error", reason: "identity" });
      } catch {
        if (!request.signal.aborted && generation === requested) update({ status: "error", reason: "unavailable" });
      }
    },
  };
}

export type TradeManagerChildStatus = { dirty: boolean; busy: boolean };
export function tradeManagerCloseAction(status: TradeManagerChildStatus) {
  return status.busy ? "wait" : status.dirty ? "confirm" : "close";
}

export function readTradeManagerMessage(event: MessageEvent, child: Window | null | undefined, origin: string) {
  if (!child || event.source !== child || event.origin !== origin || !event.data || typeof event.data !== "object") return null;
  if (event.data.type === "hone:trade-auth-invalidated") return { type: "auth-invalidated" as const };
  if (event.data.type === "hone:trade-export") return readTradeManagerExport(event.data);
  if (event.data.type === "hone:trade-close-request") return { type: "close" as const };
  if (event.data.type === "hone:trade-status" && typeof event.data.dirty === "boolean" && typeof event.data.busy === "boolean") {
    return { type: "status" as const, dirty: event.data.dirty, busy: event.data.busy };
  }
  return null;
}


export const TRADE_EXPORT_MAX_BYTES = 8 * 1024 * 1024;
export const TRADE_EXPORT_LIFETIME_MS = 15 * 60 * 1000;
export type TradeManagerExport = { type: "export"; name: string; content: string; mime: string };
export type TradeManagerDownload = { name: string; url: string };

function readTradeManagerExport(value: Record<string, unknown>): TradeManagerExport | null {
  const { name, content, mime } = value;
  if (typeof name !== "string" || typeof content !== "string" || typeof mime !== "string") return null;
  // Cross-platform filenames only; reject path syntax, control/bidi characters,
  // and names that could disguise an executable as an allowed text export.
  if (!name || name.length > 180 || name.trim() !== name || /[\/\\:*?"<>|\x00-\x1f\x7f-\x9f\u202a-\u202e\u2066-\u2069]/.test(name)) return null;
  const normalized = /^(application\/json|text\/csv|text\/markdown)(?:\s*;\s*charset\s*=\s*utf-8)?$/i.exec(mime.trim())?.[1]?.toLowerCase();
  if (!normalized) return null;
  const extensions: Record<string, string> = { "application/json": "json", "text/csv": "csv", "text/markdown": "md" };
  const extension = extensions[normalized];
  if (!extension || !new RegExp(`^.+\\.${extension}$`, "i").test(name)) return null;
  if (content.length > TRADE_EXPORT_MAX_BYTES || new TextEncoder().encode(content).byteLength > TRADE_EXPORT_MAX_BYTES) return null;
  return { type: "export", name, content, mime: `${normalized};charset=utf-8` };
}

type DownloadRuntime = {
  createUrl: (blob: Blob) => string;
  revokeUrl: (url: string) => void;
  schedule: (callback: () => void, delay: number) => number;
  cancel: (timer: number) => void;
};

/** Keeps only one ephemeral object URL; never stores or broadcasts file content. */
export function createTradeManagerDownload(
  update: (download: TradeManagerDownload | undefined) => void,
  runtime: DownloadRuntime = {
    createUrl: (blob) => URL.createObjectURL(blob),
    revokeUrl: (url) => URL.revokeObjectURL(url),
    schedule: (callback, delay) => window.setTimeout(callback, delay),
    cancel: (timer) => window.clearTimeout(timer),
  },
) {
  let current: TradeManagerDownload | undefined;
  let timer: number | undefined;
  let generation = 0;
  const clear = () => {
    generation += 1;
    if (timer !== undefined) runtime.cancel(timer);
    timer = undefined;
    if (current) runtime.revokeUrl(current.url);
    current = undefined;
    update(undefined);
  };
  return {
    clear,
    prepare(file: TradeManagerExport) {
      clear();
      current = { name: file.name, url: runtime.createUrl(new Blob([file.content], { type: file.mime })) };
      const preparedGeneration = generation;
      timer = runtime.schedule(() => { if (generation === preparedGeneration) clear(); }, TRADE_EXPORT_LIFETIME_MS);
      update(current);
    },
  };
}
