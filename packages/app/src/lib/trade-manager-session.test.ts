import { describe, expect, it } from "bun:test";
import { createTradeManagerDownload, TRADE_EXPORT_MAX_BYTES, TRADE_EXPORT_LIFETIME_MS, type TradeManagerDownload, readTradeManagerMessage, tradeManagerCloseAction, createTradeManagerSession, type TradeManagerSessionState } from "./trade-manager-session";

function deferred() {
  let resolve!: (value: { user_id: string }) => void;
  const promise = new Promise<{ user_id: string }>((done) => { resolve = done; });
  return { promise, resolve };
}

describe("trade manager iframe session lease", () => {
  it("authenticates ordinary accounts on every open and never mounts on failure", async () => {
    const states: TradeManagerSessionState[] = [];
    let calls = 0;
    const session = createTradeManagerSession(async () => {
      calls += 1;
      if (calls === 2) throw new Error("401");
      return { user_id: "ordinary-user" };
    }, (state) => states.push(state));
    await session.open("ordinary-user");
    expect(states.at(-1)).toEqual({ status: "ready" });
    session.close();
    await session.open("ordinary-user");
    expect(calls).toBe(2);
    expect(states.at(-1)).toEqual({ status: "error", reason: "unavailable" });
  });

  it("rejects another account even if the server request succeeds", async () => {
    let state: TradeManagerSessionState = { status: "closed" };
    const session = createTradeManagerSession(async () => ({ user_id: "account-b" }), (next) => { state = next; });
    await session.open("account-a");
    expect(state as TradeManagerSessionState).toEqual({ status: "error", reason: "identity" });
  });

  it("aborts on logout and ignores the late response even when transport ignores abort", async () => {
    const result = deferred();
    let signal: AbortSignal | undefined;
    const states: TradeManagerSessionState[] = [];
    const session = createTradeManagerSession(async (next) => { signal = next; return result.promise; }, (state) => states.push(state));
    const pending = session.open("account-a");
    session.close();
    expect(signal?.aborted).toBe(true);
    result.resolve({ user_id: "account-a" });
    await pending;
    expect(states.at(-1)).toEqual({ status: "closed" });
    expect(states.some((state) => state.status === "ready")).toBe(false);
  });

  it("does not let the previous opening replace a newer account", async () => {
    const old = deferred();
    let calls = 0;
    const states: TradeManagerSessionState[] = [];
    const session = createTradeManagerSession(async () => ++calls === 1 ? old.promise : { user_id: "b" }, (state) => states.push(state));
    const oldOpen = session.open("a");
    await session.open("b");
    const settledCount = states.length;
    old.resolve({ user_id: "a" });
    await oldOpen;
    expect(states).toHaveLength(settledCount);
    expect(states.at(-1)).toEqual({ status: "ready" });
  });
});

describe("trade manager frame boundary", () => {
  it("prioritizes an in-flight save over discard and confirms dirty forms", () => {
    expect(tradeManagerCloseAction({ busy: true, dirty: true })).toBe("wait");
    expect(tradeManagerCloseAction({ busy: false, dirty: true })).toBe("confirm");
    expect(tradeManagerCloseAction({ busy: false, dirty: false })).toBe("close");
  });
  it("only accepts status and close requests from the exact same-origin child", () => {
    const child = window;
    const event = (origin: string, source: Window | null, data: unknown) => ({ origin, source, data }) as MessageEvent;
    const status = { type: "hone:trade-status", dirty: true, busy: false };
    expect(readTradeManagerMessage(event("https://hone.test", child, status), child, "https://hone.test")).toEqual({ type: "status", dirty: true, busy: false });
    expect(readTradeManagerMessage(event("https://evil.test", child, status), child, "https://hone.test")).toBeNull();
    expect(readTradeManagerMessage(event("https://hone.test", null, status), child, "https://hone.test")).toBeNull();
    expect(readTradeManagerMessage(event("https://hone.test", child, { ...status, busy: "false" }), child, "https://hone.test")).toBeNull();
    expect(readTradeManagerMessage(event("https://hone.test", child, { type: "hone:trade-close-request" }), child, "https://hone.test")).toEqual({ type: "close" });
  });
});


describe("same-origin export handoff", () => {
  const data = { type: "hone:trade-export", name: "交易备份.json", content: '{"note":"个人记录"}', mime: "application/json" };
  const message = (payload: unknown, origin = "https://hone.test", source: Window | null = window) =>
    readTradeManagerMessage({ data: payload, origin, source } as MessageEvent, window, "https://hone.test");

  it("accepts only supported text exports from the exact same-origin iframe", () => {
    expect(message(data)).toEqual({ ...data, type: "export", mime: "application/json;charset=utf-8" });
    expect(message({ ...data, name: "交易.csv", mime: "text/csv;charset=utf-8" })?.type).toBe("export");
    expect(message({ ...data, name: "复盘.md", mime: "TEXT/MARKDOWN; charset=UTF-8" })?.type).toBe("export");
    expect(message(data, "https://outside.test")).toBeNull();
    expect(message(data, "https://hone.test", null)).toBeNull();
    expect(readTradeManagerMessage({ data, source: window, origin: "https://hone.test" } as unknown as MessageEvent, undefined, "https://hone.test")).toBeNull();
  });

  it("rejects path/control filenames, executable types and MIME/extension mismatches", () => {
    for (const name of ["", ".json", "../a.json", "/a.json", "a\\b.json", "C:a.json", "a\n.json", "a\u0000.json", "a\u007f.json", "a\u202e.json", "a.json ", "a.html", "a.json.exe", "a".repeat(181)+".json"]) {
      expect(message({ ...data, name })).toBeNull();
    }
    for (const mime of ["text/html", "application/javascript", "image/svg+xml", "text/plain", "text/csv", "application/json;charset=latin1", "application/json;foo=bar", "application/json\ntext/html"]) {
      expect(message({ ...data, mime })).toBeNull();
    }
    expect(message({ ...data, content: {} })).toBeNull();
    expect(message({ ...data, name: 42 })).toBeNull();
  });

  it("limits UTF-8 bytes, allowing exactly 8 MiB and rejecting one byte more", () => {
    const boundary = "中".repeat(Math.floor(TRADE_EXPORT_MAX_BYTES / 3)) + "ab";
    expect(new TextEncoder().encode(boundary).byteLength).toBe(TRADE_EXPORT_MAX_BYTES);
    expect(message({ ...data, content: boundary })?.type).toBe("export");
    expect(message({ ...data, content: boundary + "a" })).toBeNull();
    expect(message({ ...data, content: "x".repeat(TRADE_EXPORT_MAX_BYTES + 1) })).toBeNull();
  });

  it("revokes and clears on replacement, expiration and explicit close/invalidation", () => {
    const revoked: string[] = [];
    const timers = new Map<number, { callback: () => void; delay: number }>();
    const blobs: Blob[] = [];
    let timerId = 0;
    let shown: TradeManagerDownload | undefined;
    const lease = createTradeManagerDownload((next) => { shown = next; }, {
      createUrl: (blob) => { blobs.push(blob); return `blob:download-${blobs.length}`; },
      revokeUrl: (url) => { revoked.push(url); },
      schedule: (callback, delay) => { timers.set(++timerId, { callback, delay }); return timerId; },
      cancel: (id) => { timers.delete(id); },
    });
    const file = message(data);
    if (file?.type !== "export") throw new Error("invalid fixture");
    lease.prepare(file);
    expect(shown).toEqual({ name: data.name, url: "blob:download-1" });
    expect(Object.keys(shown!)).toEqual(["name", "url"]);
    expect(blobs[0].type).toBe("application/json;charset=utf-8");
    const oldTimer = timers.get(1)!;
    expect(oldTimer.delay).toBe(TRADE_EXPORT_LIFETIME_MS);
    lease.prepare(file);
    expect(revoked).toEqual(["blob:download-1"]);
    expect(timers.has(1)).toBe(false);
    oldTimer.callback();
    expect(shown?.url).toBe("blob:download-2");
    timers.get(2)!.callback();
    expect(shown).toBeUndefined();
    expect(revoked).toEqual(["blob:download-1", "blob:download-2"]);
    lease.prepare(file);
    lease.clear();
    lease.clear();
    expect(shown).toBeUndefined();
    expect(timers.size).toBe(0);
    expect(revoked).toEqual(["blob:download-1", "blob:download-2", "blob:download-3"]);
  });
});


it("only a validated child auth failure can invalidate the parent tool and its exports", () => {
  const data = { type: "hone:trade-auth-invalidated" };
  const event = (origin: string, source: Window | null) => ({ data, origin, source }) as MessageEvent;
  expect(readTradeManagerMessage(event("https://hone.test", window), window, "https://hone.test")).toEqual({ type: "auth-invalidated" });
  expect(readTradeManagerMessage(event("https://evil.test", window), window, "https://hone.test")).toBeNull();
  expect(readTradeManagerMessage(event("https://hone.test", null), window, "https://hone.test")).toBeNull();
  expect(readTradeManagerMessage(event("https://hone.test", window), undefined, "https://hone.test")).toBeNull();
});
