import { afterEach, describe, expect, test } from "bun:test";
import {
  ApiError,
  getPublicChatBootstrap,
  getPublicAuthMe,
  getPublicFinanceCalendar,
  getPublicCommunity,
  getPublicCommunityResourceBlob,
  getPublicHistory,
  markPublicCommunitySeen,
  publicCommunityResourceDownloadName,
  publicCommunityResourceUrl,
  getPublicPushes,
  isUnauthorizedApiError,
  sendPublicChat,
  sendPublicFinanceCalendar,
  openPublicPush,
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

describe("public chat bootstrap API", () => {
  test("loads auth and history through one startup request", async () => {
    let requestedUrl = "";
    globalThis.fetch = ((url: RequestInfo | URL) => {
      requestedUrl = String(url);
      return Promise.resolve(
        new Response(
          JSON.stringify({
            user: { user_id: "web-user-1", remaining_today: 9, daily_limit: 10 },
            messages: [{ role: "user", content: "hello" }],
            history_start: 42,
            next_before: 42,
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const payload = await getPublicChatBootstrap();

    expect(requestedUrl).toContain("/api/public/bootstrap");
    expect(payload.user.user_id).toBe("web-user-1");
    expect(payload.messages?.[0]?.content).toBe("hello");
    expect(payload.history_start).toBe(42);
    expect(payload.next_before).toBe(42);
  });

  test("requests the previous history page with a stable cursor", async () => {
    let requestedUrl = "";
    globalThis.fetch = ((url: RequestInfo | URL) => {
      requestedUrl = String(url);
      return Promise.resolve(
        new Response(
          JSON.stringify({ messages: [], history_start: 20, next_before: 20 }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const payload = await getPublicHistory(40);

    expect(requestedUrl).toContain("/api/public/history?limit=20&before=40");
    expect(payload.history_start).toBe(20);
  });
});

describe("public community API", () => {
  test("uses an opaque content cursor and returns read-only timeline data", async () => {
    let requestedUrl = "";
    globalThis.fetch = ((url: RequestInfo | URL) => {
      requestedUrl = String(url);
      return Promise.resolve(
        new Response(
          JSON.stringify({
            community: { id: "51115212285814", name: "HONE 官方社区" },
            items: [{ content_id: 42, body_text: "hello", resources: [] }],
            next_before: 42,
            unread: true,
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const payload = await getPublicCommunity({ before: 88, limit: 20 });

    expect(requestedUrl).toContain("/api/public/community?before=88&limit=20");
    expect(payload.items[0]?.content_id).toBe(42);
    expect(payload.unread).toBe(true);
  });

  test("marks the latest community content as seen without sending a social action", async () => {
    let body = "";
    globalThis.fetch = ((_: RequestInfo | URL, init?: RequestInit) => {
      body = String(init?.body);
      return Promise.resolve(
        new Response(JSON.stringify({ ok: true }), {
          headers: { "content-type": "application/json" },
        }),
      );
    }) as typeof fetch;

    await markPublicCommunitySeen(42);

    expect(body).toBe('{"content_id":42}');
  });

  test("downloads a protected community resource through the authenticated API", async () => {
    let requestedUrl = "";
    let credentials: RequestCredentials | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      credentials = init?.credentials;
      return Promise.resolve(
        new Response(new Uint8Array([1, 2, 3]), {
          headers: { "content-type": "image/jpeg" },
        }),
      );
    }) as typeof fetch;

    const blob = await getPublicCommunityResourceBlob(99, "0123456789ab");

    expect(requestedUrl).toContain(
      "/api/public/community/resources/99?v=0123456789ab",
    );
    expect(credentials).toBe("include");
    expect(blob.size).toBe(3);
  });

  test("keeps legacy resources revalidating while versioning hashed resources", () => {
    expect(publicCommunityResourceUrl(99)).toContain(
      "/api/public/community/resources/99",
    );
    expect(publicCommunityResourceUrl(99)).not.toContain("?v=");
    expect(publicCommunityResourceUrl(99, "0123456789ab")).toContain(
      "/api/public/community/resources/99?v=0123456789ab",
    );
  });

  test("corrects a source-mislabeled OOXML workbook download extension", () => {
    expect(
      publicCommunityResourceDownloadName({
        resource_id: 295,
        display_name: "投资组合.xls",
        content_type:
          "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      }),
    ).toBe("投资组合.xlsx");
    expect(
      publicCommunityResourceDownloadName({
        resource_id: 1,
        display_name: "报告.pdf",
        content_type: "application/pdf",
      }),
    ).toBe("报告.pdf");
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
      mobile_path: "/tmp/public/web-user/calendar-mobile.png",
      month: "2026-07",
    });

    expect(requestBody).toEqual({
      path: "/tmp/public/web-user/calendar.png",
      mobile_path: "/tmp/public/web-user/calendar-mobile.png",
      month: "2026-07",
    });
    expect(result.ok).toBe(true);
  });
});

describe("public push API", () => {
  test("loads a cursor page of scheduled pushes", async () => {
    let requestedUrl = "";
    globalThis.fetch = ((url: RequestInfo | URL) => {
      requestedUrl = String(url);
      return Promise.resolve(
        new Response(
          JSON.stringify({ items: [], unread_count: 2, next_before: null }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      );
    }) as unknown as typeof fetch;

    const payload = await getPublicPushes("job-1:2026-07-10:20:00", 20);

    expect(requestedUrl).toContain("/api/public/pushes?");
    expect(requestedUrl).toContain("limit=20");
    expect(requestedUrl).toContain(
      "before=job-1%3A2026-07-10%3A20%3A00",
    );
    expect(payload.unread_count).toBe(2);
  });

  test("opens a push through a POST action", async () => {
    let requestedUrl = "";
    let requestedMethod = "";
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      requestedMethod = init?.method ?? "GET";
      return Promise.resolve(
        new Response(
          JSON.stringify({
            push: {
              push_id: "job-1:2026-07-10:20:00",
              job_id: "job-1",
              title: "收盘复盘",
              summary: "摘要",
              content: "完整内容",
              created_at: "2026-07-10T20:00:00+08:00",
            },
            unread_count: 0,
          }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      );
    }) as unknown as typeof fetch;

    const payload = await openPublicPush("job-1:2026-07-10:20:00");

    expect(requestedMethod).toBe("POST");
    expect(requestedUrl).toContain(
      "/api/public/pushes/job-1%3A2026-07-10%3A20%3A00/open",
    );
    expect(payload.unread_count).toBe(0);
    expect(payload.push.content).toBe("完整内容");
  });
});
