import { beforeEach, describe, expect, it } from "bun:test";

import {
  cachedPublicUser,
  hasCachedPublicUser,
  setCachedPublicUser,
} from "@/lib/public-session-cache";
import type { PublicAuthUserInfo } from "@/lib/types";

const user = (email: string) =>
  ({
    id: 1,
    email,
    is_admin: false,
    billing: { access_granted: true },
  }) as unknown as PublicAuthUserInfo;

describe("signed-in user survives a route change", () => {
  beforeEach(() => setCachedPublicUser(null));

  it("has nothing to paint before the first fetch", () => {
    // Rendering optimistically from an empty cache would show a signed-out
    // shell to a signed-in user, which is worse than a brief loading state.
    expect(hasCachedPublicUser()).toBe(false);
    expect(cachedPublicUser()).toBeNull();
  });

  it("keeps the user so the next route paints without a round-trip", () => {
    setCachedPublicUser(user("a@example.com"));

    expect(hasCachedPublicUser()).toBe(true);
    expect(cachedPublicUser()?.email).toBe("a@example.com");
  });

  it("forgets the user on sign-out", () => {
    // A stale cache after logout would paint the account page as signed in.
    setCachedPublicUser(user("a@example.com"));
    setCachedPublicUser(null);

    expect(hasCachedPublicUser()).toBe(false);
    expect(cachedPublicUser()).toBeNull();
  });

  it("stores a copy so a later mutation cannot rewrite history", () => {
    const original = user("a@example.com");
    setCachedPublicUser(original);
    original.email = "b@example.com";

    expect(cachedPublicUser()?.email).toBe("a@example.com");
  });
});
