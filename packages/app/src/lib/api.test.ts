import { afterEach, describe, expect, test } from "bun:test";
import { ApiError, getPublicAuthMe, isUnauthorizedApiError } from "./api";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
});

describe("public API errors", () => {
  test("preserves status for auth restore decisions", async () => {
    globalThis.fetch = ((() =>
      Promise.resolve(
        new Response(JSON.stringify({ error: "未登录" }), {
          status: 401,
          statusText: "Unauthorized",
        }),
      )) as unknown) as typeof fetch;

    try {
      await getPublicAuthMe();
      throw new Error("expected getPublicAuthMe to fail");
    } catch (error) {
      expect(error).toBeInstanceOf(ApiError);
      expect(isUnauthorizedApiError(error)).toBe(true);
      expect((error as ApiError).status).toBe(401);
      expect((error as Error).message).toBe("未登录");
    }
  });

  test("does not classify server errors as logged-out sessions", async () => {
    const error = new ApiError(
      "temporary outage",
      new Response("", { status: 502, statusText: "Bad Gateway" }),
    );

    expect(isUnauthorizedApiError(error)).toBe(false);
  });
});
