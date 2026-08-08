import { beforeEach, describe, expect, it } from "bun:test";

import {
  cachedPublicUser,
  hasCachedPublicUser,
  setCachedPublicUser,
} from "@/lib/public-session-cache";
import type { PublicAuthUserInfo } from "@/lib/types";

const user = (userId: string) =>
  ({
    user_id: userId,
    created_at: "2026-08-07T00:00:00Z",
    daily_limit: 10,
    success_count: 0,
    in_flight: 0,
    remaining_today: 10,
    has_password: false,
    identity_kind: "domestic_invite",
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
    setCachedPublicUser(user("u-a"));

    expect(hasCachedPublicUser()).toBe(true);
    expect(cachedPublicUser()?.user_id).toBe("u-a");
  });

  it("forgets the user on sign-out", () => {
    // A stale cache after logout would paint the account page as signed in.
    setCachedPublicUser(user("u-a"));
    setCachedPublicUser(null);

    expect(hasCachedPublicUser()).toBe(false);
    expect(cachedPublicUser()).toBeNull();
  });

  it("stores a copy so a later mutation cannot rewrite history", () => {
    const original = user("u-a");
    setCachedPublicUser(original);
    original.user_id = "u-b";

    expect(cachedPublicUser()?.user_id).toBe("u-a");
  });
});
