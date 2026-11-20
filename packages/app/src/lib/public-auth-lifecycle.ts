/** No identity or credentials cross this channel; every consumer reauthenticates. */
export const PUBLIC_AUTH_INVALIDATED = "hone:auth-invalidated";
export const PUBLIC_AUTH_CHANNEL = "hone-auth-lifecycle";

type AuthChannel = Pick<BroadcastChannel, "postMessage" | "addEventListener" | "removeEventListener">;

/** Receiving an invalidation is deliberately local: rebroadcasting creates a loop. */
export function createPublicAuthLifecycle(target: EventTarget, channel?: AuthChannel) {
  let epoch = 0;
  const advanceEpoch = () => { epoch += 1; };
  target.addEventListener(PUBLIC_AUTH_INVALIDATED, advanceEpoch);
  const invalidateLocal = () => target.dispatchEvent(new Event(PUBLIC_AUTH_INVALIDATED));
  const receive = (event: MessageEvent) => {
    if (event.data?.type === "invalidate") invalidateLocal();
  };
  channel?.addEventListener("message", receive as EventListener);
  return {
    epoch: () => epoch,
    invalidate: () => {
      invalidateLocal();
      try { channel?.postMessage({ type: "invalidate" }); } catch { /* local clearing still completed */ }
    },
    dispose: () => {
      target.removeEventListener(PUBLIC_AUTH_INVALIDATED, advanceEpoch);
      channel?.removeEventListener("message", receive as EventListener);
    },
  };
}

let lifecycle: ReturnType<typeof createPublicAuthLifecycle> | undefined;
function currentLifecycle() {
  if (!lifecycle && typeof window !== "undefined") {
    let channel: BroadcastChannel | undefined;
    try {
      if (typeof window.BroadcastChannel === "function") channel = new window.BroadcastChannel(PUBLIC_AUTH_CHANNEL);
    } catch { /* Browsers may block cross-tab channels; local clearing still works. */ }
    lifecycle = createPublicAuthLifecycle(window, channel);
  }
  return lifecycle;
}

export function publicAuthEpoch() { return currentLifecycle()?.epoch() ?? 0; }
export function invalidatePublicAuth() { currentLifecycle()?.invalidate(); }
export function onPublicAuthInvalidated(listener: () => void) {
  currentLifecycle();
  if (typeof window === "undefined") return () => {};
  window.addEventListener(PUBLIC_AUTH_INVALIDATED, listener);
  return () => window.removeEventListener(PUBLIC_AUTH_INVALIDATED, listener);
}

export function assertPublicAuthCurrent(epoch: number) {
  if (epoch !== publicAuthEpoch()) throw new DOMException("Session invalidated", "AbortError");
}
