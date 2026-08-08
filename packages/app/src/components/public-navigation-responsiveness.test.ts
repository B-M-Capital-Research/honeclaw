import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";

const workspace = readFileSync(
  new URL("./public-agent-workspace.tsx", import.meta.url),
  "utf8",
);
const mePage = readFileSync(
  new URL("../pages/public-me.tsx", import.meta.url),
  "utf8",
);
const api = readFileSync(new URL("../lib/api.ts", import.meta.url), "utf8");
const prefetch = readFileSync(
  new URL("../lib/route-prefetch.ts", import.meta.url),
  "utf8",
);
const communityPage = readFileSync(
  new URL("../pages/public-community.tsx", import.meta.url),
  "utf8",
);

describe("switching sections feels immediate", () => {
  it("warms a lazy route's chunk before the click that needs it", () => {
    // Every route is lazy(), so the first navigation to one paid for a network
    // round-trip before anything could render. Pointer entry and focus both
    // precede activation.
    expect(prefetch).toContain('import("@/pages/public-me")');
    expect(prefetch).toContain('import("@/pages/public-community")');
    expect(prefetch).toContain("onPointerEnter");
    expect(prefetch).toContain("onFocus");
  });

  it("attaches the warm-up to every control that leaves the workspace", () => {
    expect(workspace).toContain('routePrefetchHandlers("me")');
    expect(workspace).toContain('routePrefetchHandlers("community")');
    // The rail, the drawer and the mobile header all navigate away, and any
    // one of them left unwired is a slow path a user will find.
    const wired = workspace.split("routePrefetchHandlers(").length - 1;
    expect(wired).toBeGreaterThanOrEqual(6);
  });

  it("paints the account page from what is already known", () => {
    // Arriving from a chat that already resolved the user used to blank the
    // whole page for one round-trip; the navigation was never the slow part.
    expect(api).toContain("setCachedPublicUser(payload.user)");
    expect(mePage).toContain("createSignal<PublicAuthUserInfo | null>(cachedPublicUser())");
    expect(mePage).toContain("createSignal(!hasCachedPublicUser())");
  });

  it("repaints the community feed before revalidating it", () => {
    // Reopening the section showed a loading line over a page that was still
    // perfectly readable.
    expect(communityPage).toContain("cachedCommunityFeed()");
    expect(communityPage).toContain("setCachedCommunityFeed(page.items)");
    // A refresh that fails, or that is slower than the eye, must not replace
    // content already on screen.
    expect(communityPage).toContain("if (items().length === 0) setState(\"loading\")");
    expect(communityPage).toContain("if (items().length === 0) setState(\"error\")");
    // ...and a signed-out visitor must stop seeing it.
    expect(communityPage).toContain("setCachedCommunityFeed(null)");
  });

  it("drops the cached user whenever the session ends", () => {
    // A cache that outlives the session would paint a signed-out visitor as
    // signed in — worse than the loading state it replaced.
    const clears = mePage.split("setCachedPublicUser(null)").length - 1;
    expect(clears).toBeGreaterThanOrEqual(2);
  });
});
