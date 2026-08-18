import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const chat = readFileSync(new URL("./chat.tsx", import.meta.url), "utf8");
const sessionCache = readFileSync(
  new URL("../lib/public-session-cache.ts", import.meta.url),
  "utf8",
);

// The first paint used to cost two serial network round trips: every side
// load waited on `authState() === "ready"`, which only flips once /bootstrap
// answers. The guard — not a data dependency — was the whole delay, so a
// returning visitor now starts the independent reads immediately.
describe("chat first-paint contract", () => {
  it("remembers across reloads that this browser had a session", () => {
    // An in-memory signal is empty on a cold page load, which is exactly when
    // the warm start matters, so the hint has to outlive the tab.
    expect(sessionCache).toContain("localStorage.setItem(SESSION_HINT_KEY");
    expect(sessionCache).toContain("localStorage.removeItem(SESSION_HINT_KEY");
    expect(sessionCache).toContain("export function hadRecentSession");
  });

  it("starts the aside reads alongside bootstrap, not after it", () => {
    const mount = chat.slice(chat.indexOf("if (hadRecentSession())"));
    const warm = mount.slice(0, mount.indexOf("restoreSession"));
    expect(warm).toContain("loadWorkspaceAside()");
    expect(warm).toContain("refreshPushUnread()");
    // Chat writes the cache on every auth answer, or the hint never appears.
    expect(chat).toContain("setCachedPublicUser(user)");
    expect(chat).toContain("setCachedPublicUser(null)");
  });

  it("lets the post-bootstrap effects adopt the warm reads instead of repeating them", () => {
    expect(chat).toContain("const WARM_START_OWNER");
    // Both owners are claimed up front, and both effects bail when they see
    // the marker — otherwise the badge and the aside each fetch twice.
    expect(chat).toContain("workspaceLoadedFor = WARM_START_OWNER");
    expect(chat).toContain("pushUserId = WARM_START_OWNER");
    expect(chat.split("=== WARM_START_OWNER").length - 1).toBe(2);
  });

  it("reads the first-paint unread count from the list it already asks for", () => {
    // getPublicCommunity({limit: 3}) carries `unread`, so the aside load sets
    // the badge itself. The standalone limit=1 probe survives only as the 60s
    // visibility poll — one call site, and never on the first paint.
    const aside = chat.slice(chat.indexOf("const loadWorkspaceAside"));
    expect(aside.slice(0, aside.indexOf("};"))).toContain(
      "setCommunityUnread(community.unread)",
    );
    expect(chat.split("void refreshCommunityUnread();").length - 1).toBe(1);
    const poll = chat.slice(chat.indexOf("const refreshWhenVisible"));
    expect(poll.slice(0, poll.indexOf("};"))).toContain("refreshCommunityUnread()");
  });
});
