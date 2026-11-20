import { describe, expect, it } from "bun:test";
import { createPublicAuthLifecycle, PUBLIC_AUTH_INVALIDATED } from "./public-auth-lifecycle";

class TestChannel extends EventTarget {
  sent: unknown[] = [];
  postMessage(value: unknown) { this.sent.push(value); }
}

describe("public auth lifecycle", () => {
  it("clears local consumers synchronously and broadcasts no account identity", () => {
    const target = new EventTarget();
    const channel = new TestChannel();
    const lifecycle = createPublicAuthLifecycle(target, channel);
    let cleared = false;
    target.addEventListener(PUBLIC_AUTH_INVALIDATED, () => { cleared = true; });
    lifecycle.invalidate();
    expect(cleared).toBe(true);
    expect(lifecycle.epoch()).toBe(1);
    expect(channel.sent).toEqual([{ type: "invalidate" }]);
    lifecycle.dispose();
  });

  it("receives cross-tab invalidation without broadcasting it back", () => {
    const target = new EventTarget();
    const channel = new TestChannel();
    const lifecycle = createPublicAuthLifecycle(target, channel);
    let count = 0;
    target.addEventListener(PUBLIC_AUTH_INVALIDATED, () => { count += 1; });
    channel.dispatchEvent(new MessageEvent("message", { data: { type: "unrelated" } }));
    channel.dispatchEvent(new MessageEvent("message", { data: { type: "invalidate" } }));
    expect(count).toBe(1);
    expect(lifecycle.epoch()).toBe(1);
    expect(channel.sent).toEqual([]);
    lifecycle.dispose();
    channel.dispatchEvent(new MessageEvent("message", { data: { type: "invalidate" } }));
    expect(count).toBe(1);
  });
});
