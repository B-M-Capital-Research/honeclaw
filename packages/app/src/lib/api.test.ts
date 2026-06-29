import { afterEach, describe, expect, test } from "bun:test";
import {
  ApiError,
  getPublicAuthMe,
  getPublicFinanceCalendar,
  isUnauthorizedApiError,
  sendPublicChat,
  sendPublicFinanceCalendar,
} from "./api";
import {
  FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE,
  resetApiFetchRetryDelayForTests,
  setApiFetchRetryDelayForTests,
} from "./backend";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  resetApiFetchRetryDelayForTests();
});

function mockFetch(response: Response) {
  globalThis.fetch = ((() => Promise.resolve(response)) as unknown) as typeof fetch;
}

async function expectApiError(
  action: () => Promise<unknown>,
): Promise<ApiError> {
  try {
    await action();
  } catch (error) {
    expect(error).toBeInstanceOf(ApiError);
    return error as ApiError;
  }
  throw new Error("expected API call to fail");
}

describe("public API errors", () => {
  test("preserves status for auth restore decisions", async () => {
    mockFetch(
      new Response(JSON.stringify({ error: "未登录" }), {
        status: 401,
        statusText: "Unauthorized",
      }),
    );

    const error = await expectApiError(getPublicAuthMe);
    expect(isUnauthorizedApiError(error)).toBe(true);
    expect(error.status).toBe(401);
    expect(error.message).toBe("未登录");
  });

  test("does not classify server errors as logged-out sessions", async () => {
    const error = new ApiError(
      "temporary outage",
      new Response("", { status: 502, statusText: "Bad Gateway" }),
    );

    expect(isUnauthorizedApiError(error)).toBe(false);
  });

  test("rewrites repeated 502 responses to a friendly message", async () => {
    setApiFetchRetryDelayForTests(0);
    let calls = 0;
    globalThis.fetch = ((() => {
      calls += 1;
      return Promise.resolve(
        new Response("<html>Bad Gateway</html>", {
          status: 502,
          statusText: "Bad Gateway",
        }),
      );
    }) as unknown) as typeof fetch;

    const error = await expectApiError(getPublicAuthMe);

    expect(calls).toBe(2);
    expect(error.status).toBe(502);
    expect(error.message).toBe(FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE);
  });

  test("streaming public chat uses the same friendly backend failure", async () => {
    setApiFetchRetryDelayForTests(0);
    globalThis.fetch = ((() =>
      Promise.resolve(
        new Response("nginx gateway failure\nwith stack details", {
          status: 503,
          statusText: "Service Unavailable",
        }),
      )) as unknown) as typeof fetch;

    const error = await expectApiError(() => sendPublicChat("hello"));

    expect(error.status).toBe(503);
    expect(error.message).toBe(FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE);
  });
});

describe("public finance calendar API", () => {
  test("loads a selected calendar month", async () => {
    let requestedUrl = "";
    globalThis.fetch = ((url: RequestInfo | URL) => {
      requestedUrl = String(url);
      return Promise.resolve(
        new Response(
          JSON.stringify({
            today: "2026-06-29",
            month: "2026-07",
            months: [],
            holdings: [],
            events: [],
            earnings_status: "empty_portfolio",
            errors: [],
          }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      );
    }) as unknown as typeof fetch;

    const payload = await getPublicFinanceCalendar("2026-07");

    expect(requestedUrl).toContain("/api/public/finance-calendar?month=2026-07");
    expect(payload.month).toBe("2026-07");
  });

  test("sends a rendered calendar image", async () => {
    let requestBody: unknown;
    globalThis.fetch = ((_: RequestInfo | URL, init?: RequestInit) => {
      requestBody = JSON.parse(String(init?.body ?? "{}"));
      return Promise.resolve(
        new Response(JSON.stringify({ ok: true, message: "done" }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      );
    }) as unknown as typeof fetch;

    const result = await sendPublicFinanceCalendar({
      path: "/tmp/public/web-user/calendar.png",
      month: "2026-07",
    });

    expect(requestBody).toEqual({
      path: "/tmp/public/web-user/calendar.png",
      month: "2026-07",
    });
    expect(result.ok).toBe(true);
  });
});
