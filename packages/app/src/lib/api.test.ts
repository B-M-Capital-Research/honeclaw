import { setLocale } from "./i18n";
import { afterEach, describe, expect, test } from "bun:test";
import {
  ApiError,
  createPublicAdminInvite,
  disablePublicAdminInvite,
  getPublicAdminUsage,
  getPublicChatBootstrap,
  getPublicAuthMe,
  getPublicFinanceCalendar,
  getPublicGeneratedFileBlob,
  getPublicCommunity,
  getPublicCommunityResourceBlob,
  getPublicHistory,
  markPublicCommunitySeen,
  publicCommunityResourceDownloadName,
  publicCommunityResourceUrl,
  resetPublicCommunityEdgeDiscoveryForTests,
  resolvePublicCommunityResourceUrl,
  setPublicCommunityEdgeDiscoveryForTests,
  getPublicPushes,
  isUnauthorizedApiError,
  sendPublicChat,
  sendPublicFinanceCalendar,
  openPublicPush,
  publicLogout,
} from "./api";
import {
  FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE,
  resetApiFetchRetryDelayForTests,
  setApiFetchRetryDelayForTests,
} from "./backend";
import {
  cachedCommunityFeed,
  cachedPublicUser,
  setCachedCommunityFeed,
  setCachedPublicUser,
} from "./public-session-cache";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  resetApiFetchRetryDelayForTests();
  resetPublicCommunityEdgeDiscoveryForTests();
  setCachedPublicUser(null);
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
    let calls = 0;
    globalThis.fetch = ((() => {
      calls += 1;
      return Promise.resolve(
        new Response("nginx gateway failure\nwith stack details", {
          status: 503,
          statusText: "Service Unavailable",
        }),
      );
    }) as unknown) as typeof fetch;

    const error = await expectApiError(() => sendPublicChat("hello"));

    expect(calls).toBe(1);
    expect(error.status).toBe(503);
    expect(error.message).toBe(FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE);
  });

  test("streaming public chat never replays a POST after transport failure", async () => {
    let calls = 0;
    globalThis.fetch = ((() => {
      calls += 1;
      return Promise.reject(new TypeError("Failed to fetch"));
    }) as unknown) as typeof fetch;

    await expect(sendPublicChat("hello")).rejects.toThrow(
      FRIENDLY_BACKEND_UNAVAILABLE_MESSAGE,
    );
    expect(calls).toBe(1);
  });
});

describe("public earnings workflow API", () => {
  test("sends a structured workflow envelope instead of relying on prompt text", async () => {
    let requestedBody = "";
    globalThis.fetch = ((_: RequestInfo | URL, init?: RequestInit) => {
      requestedBody = String(init?.body ?? "");
      return Promise.resolve(new Response("event: done\ndata: {}\n\n"));
    }) as unknown as typeof fetch;

    // The reported language is per-device state, so pin it rather than
    // inheriting whatever an earlier test file left behind.
    setLocale("zh");
    await sendPublicChat(
      "请为 NVDA 生成财报前瞻，并完成证据核验和可分享 PDF。",
      [],
      undefined,
      { kind: "preview", company: "NVDA" },
    );

    expect(JSON.parse(requestedBody)).toEqual({
      message: "请为 NVDA 生成财报前瞻，并完成证据核验和可分享 PDF。",
      attachments: [],
      earnings_workflow: { kind: "preview", company: "NVDA" },
      // The server must be told which language the user is reading, rather
      // than inferring it from the conversation.
      language: "zh",
    });
  });
});

describe("public chat bootstrap API", () => {
  test("loads auth and history through one startup request", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      requestedInit = init;
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
    expect(requestedInit?.cache).toBe("no-store");
    expect(cachedPublicUser()?.user_id).toBe("web-user-1");
  });

  test("logout clears route caches before the request settles", async () => {
    setCachedPublicUser({ user_id: "old-user" } as never);
    setCachedCommunityFeed([{ id: "old-content" }]);
    let release!: () => void;
    globalThis.fetch = ((() =>
      new Promise<Response>((resolve) => {
        release = () =>
          resolve(
            new Response(JSON.stringify({ ok: true }), {
              headers: { "content-type": "application/json" },
            }),
          );
      })) as unknown) as typeof fetch;

    const pending = publicLogout();
    expect(cachedPublicUser()).toBeNull();
    expect(cachedCommunityFeed()).toBeNull();
    release();
    await pending;
  });

  test("requests the previous history page with a stable cursor", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      requestedInit = init;
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
    expect(requestedInit?.cache).toBe("no-store");
  });
});

describe("public administrator whitelist API", () => {
  test("loads the administrator usage report without caching", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      requestedInit = init;
      return Promise.resolve(
        new Response(
          JSON.stringify({
            generated_at: "2026-08-02T12:00:00+08:00",
            period_start: "2026-07-20",
            period_end: "2026-08-02",
            summary: {
              today: "2026-08-02",
              today_active_users: 2,
              today_question_count: 5,
              today_delivered_push_count: 3,
              last_week_same_day_active_users: 3,
              active_user_change: -1,
              leading_decline_question_delta: 2,
              text: "今日 HONE 总使用人数 2 人",
            },
            rows: [],
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const result = await getPublicAdminUsage(30);

    expect(requestedUrl).toContain("/api/public/admin/usage?days=30");
    expect(requestedInit?.cache).toBe("no-store");
    expect(result.period_days).toBe(14);
    expect(result.summary.today_question_count).toBe(5);
  });

  test("creates through a one-shot POST with the administrator action marker", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    let calls = 0;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      calls += 1;
      requestedUrl = String(url);
      requestedInit = init;
      return Promise.resolve(
        new Response(
          JSON.stringify({
            invite: {
              user_id: "web-user-2",
              phone_number: "13900000000",
              created_at: "2026-07-31T10:00:00+08:00",
              enabled: true,
              can_disable: true,
            },
            daily_create_limit: 5,
            created_today: 1,
            remaining_today: 4,
            cleared_session_count: 0,
            message: "已加入会员白名单",
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const result = await createPublicAdminInvite("13900000000");
    const headers = new Headers(requestedInit?.headers);

    expect(calls).toBe(1);
    expect(requestedUrl).toContain("/api/public/admin/invites");
    expect(requestedInit?.method).toBe("POST");
    expect(headers.get("x-hone-admin-action")).toBe("whitelist");
    expect(result.remaining_today).toBe(4);
  });

  test("disables through a one-shot marked POST to the exact user", async () => {
    let requestedUrl = "";
    let requestedInit: RequestInit | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      requestedInit = init;
      return Promise.resolve(
        new Response(
          JSON.stringify({
            invite: {
              user_id: "web-user/member",
              phone_number: "13900000000",
              created_at: "2026-07-31T10:00:00+08:00",
              enabled: false,
              can_disable: false,
            },
            daily_create_limit: 5,
            created_today: 1,
            remaining_today: 4,
            cleared_session_count: 1,
            message: "已禁用会员白名单",
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    await disablePublicAdminInvite("web-user/member");
    const headers = new Headers(requestedInit?.headers);

    expect(requestedUrl).toContain(
      "/api/public/admin/invites/web-user%2Fmember/disable",
    );
    expect(requestedInit?.method).toBe("POST");
    expect(headers.get("x-hone-admin-action")).toBe("whitelist");
  });

  test("keeps the server daily-limit explanation", async () => {
    mockFetch(
      new Response(
        JSON.stringify({ error: "今日新增白名单已达到 5 人上限" }),
        {
          status: 429,
          statusText: "Too Many Requests",
          headers: { "content-type": "application/json" },
        },
      ),
    );

    const error = await expectApiError(() =>
      createPublicAdminInvite("13900000000"),
    );
    expect(error.status).toBe(429);
    expect(error.message).toBe("今日新增白名单已达到 5 人上限");
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

  test("uses the private edge feed only after a prefer grant", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    const requested: string[] = [];
    globalThis.fetch = ((url: RequestInfo | URL) => {
      requested.push(String(url));
      if (String(url).includes("/edge-session")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              enabled: true,
              mode: "prefer",
              base_path: "/_community/v1",
              expires_at: Math.floor(Date.now() / 1_000) + 900,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (String(url).includes("/api/public/community/state")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({ unread: true, latest_content_id: 42 }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      return Promise.resolve(
        new Response(
          JSON.stringify({
            community: { id: "51115212285814", name: "HONE 官方社区" },
            items: [
              {
                content_id: 42,
                body_text: "edge",
                resources: [
                  {
                    resource_id: 99,
                    version: "0123456789ab",
                    delivery_path:
                      "/_community/v1/resources/99/0123456789ab",
                  },
                ],
              },
            ],
            next_before: null,
            unread: false,
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const payload = await getPublicCommunity();

    expect(requested).toHaveLength(3);
    expect(requested).toContainEqual(
      expect.stringContaining("/api/public/community/edge-session"),
    );
    expect(requested).toContainEqual(
      expect.stringContaining("/_community/v1/feed/latest.json"),
    );
    expect(requested).toContainEqual(
      expect.stringContaining("/api/public/community/state"),
    );
    expect(payload.items[0]?.body_text).toBe("edge");
    expect(payload.unread).toBe(true);
    expect(payload.latest_content_id).toBe(42);
    expect(
      publicCommunityResourceUrl(
        99,
        "0123456789ab",
        payload.items[0]?.resources[0]?.delivery_path,
      ),
    ).toContain("/_community/v1/resources/99/0123456789ab");
  });

  test("falls back immediately to the legacy feed when the edge is unavailable", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    const requested: string[] = [];
    globalThis.fetch = ((url: RequestInfo | URL) => {
      const raw = String(url);
      requested.push(raw);
      if (raw.includes("/edge-session")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              enabled: true,
              mode: "prefer",
              base_path: "/_community/v1",
              expires_at: Math.floor(Date.now() / 1_000) + 900,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (raw.includes("/_community/v1/")) {
        return Promise.resolve(
          new Response("edge unavailable", {
            status: 503,
            statusText: "Service Unavailable",
          }),
        );
      }
      return Promise.resolve(
        new Response(
          JSON.stringify({
            community: { id: "51115212285814", name: "HONE 官方社区" },
            items: [{ content_id: 41, body_text: "legacy", resources: [] }],
            next_before: null,
            unread: true,
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const payload = await getPublicCommunity();

    expect(
      requested.filter((url) => url.includes("/_community/v1/feed/latest.json")),
    ).toHaveLength(1);
    expect(requested.at(-1)).toContain("/api/public/community");
    expect(payload.items[0]?.body_text).toBe("legacy");
  });

  test("falls back to the legacy feed when personal state is unavailable", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    const requested: string[] = [];
    globalThis.fetch = ((url: RequestInfo | URL) => {
      const raw = String(url);
      requested.push(raw);
      if (raw.includes("/edge-session")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              enabled: true,
              mode: "prefer",
              base_path: "/_community/v1",
              expires_at: Math.floor(Date.now() / 1_000) + 900,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (raw.includes("/_community/v1/")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              community: { id: "51115212285814", name: "HONE 官方社区" },
              items: [{ content_id: 42, body_text: "edge", resources: [] }],
              next_before: null,
              unread: false,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (raw.includes("/api/public/community/state")) {
        return Promise.resolve(new Response("state unavailable", { status: 503 }));
      }
      return Promise.resolve(
        new Response(
          JSON.stringify({
            community: { id: "51115212285814", name: "HONE 官方社区" },
            items: [{ content_id: 41, body_text: "legacy", resources: [] }],
            next_before: null,
            unread: true,
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    const payload = await getPublicCommunity();

    expect(
      requested.filter((url) => url.endsWith("/api/public/community")),
    ).toHaveLength(1);
    expect(payload.items[0]?.body_text).toBe("legacy");
    expect(payload.unread).toBe(true);
  });

  test("starts personal state before the edge grant finishes to avoid a serial origin round trip", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    let grant!: (response: Response) => void;
    const requested: string[] = [];
    globalThis.fetch = ((url: RequestInfo | URL) => {
      const raw = String(url);
      requested.push(raw);
      if (raw.includes("/edge-session")) {
        return new Promise<Response>((resolve) => { grant = resolve; });
      }
      if (raw.includes("/community/state")) {
        return Promise.resolve(Response.json({ unread: true, latest_content_id: 42 }));
      }
      return Promise.resolve(Response.json({
        items: [{ content_id: 42, resources: [] }], next_before: null,
      }));
    }) as typeof fetch;

    const pending = getPublicCommunity();
    expect(requested.some((url) => url.endsWith("/community/state"))).toBe(true);
    expect(requested.some((url) => url.includes("/feed/"))).toBe(false);
    grant(Response.json({
      enabled: true, mode: "prefer", base_path: "/_community/v1",
      expires_at: Math.floor(Date.now() / 1_000) + 900,
    }));
    expect((await pending).items[0]?.content_id).toBe(42);
    expect(requested).toHaveLength(3);
  });

  test("uses canonical latest data when the edge snapshot is stale, without comparing IDs numerically", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    const requested: string[] = [];
    globalThis.fetch = ((url: RequestInfo | URL) => {
      const raw = String(url);
      requested.push(raw);
      if (raw.includes("/edge-session")) {
        return Promise.resolve(Response.json({
          enabled: true, mode: "prefer", base_path: "/_community/v1",
          expires_at: Math.floor(Date.now() / 1_000) + 900,
        }));
      }
      if (raw.includes("/community/state")) {
        return Promise.resolve(Response.json({ unread: true, latest_content_id: 42 }));
      }
      return Promise.resolve(Response.json({
        items: [{ content_id: raw.includes("/_community/") ? 99 : 42, resources: [] }],
        next_before: null, unread: true,
      }));
    }) as typeof fetch;

    expect((await getPublicCommunity()).items[0]?.content_id).toBe(42);
    expect(requested.at(-1)).toMatch(/\/api\/public\/community$/);
    expect(publicCommunityResourceUrl(1, "0123456789ab", "/_community/v1/resources/1/0123456789ab"))
      .toContain("/api/public/community/resources/1");
    const requestsAfterFallback = requested.length;
    await getPublicCommunity();
    expect(requested).toHaveLength(requestsAfterFallback + 1);
  });

  test("accepts an older cursor page even though it does not contain the latest content", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    const requested: string[] = [];
    globalThis.fetch = ((url: RequestInfo | URL) => {
      const raw = String(url);
      requested.push(raw);
      if (raw.includes("/edge-session")) {
        return Promise.resolve(Response.json({
          enabled: true, mode: "prefer", base_path: "/_community/v1",
          expires_at: Math.floor(Date.now() / 1_000) + 900,
        }));
      }
      if (raw.includes("/community/state")) {
        return Promise.resolve(Response.json({ unread: false, latest_content_id: 42 }));
      }
      return Promise.resolve(Response.json({ items: [{ content_id: 7 }], next_before: 7 }));
    }) as typeof fetch;

    expect((await getPublicCommunity({ before: 20 })).items[0]?.content_id).toBe(7);
    expect(requested).toHaveLength(3);
    expect(requested.at(-1)).toContain("/_community/v1/feed/pages/20.json");
  });

  test("falls back as soon as edge parsing fails while the state request is still pending", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    globalThis.fetch = ((url: RequestInfo | URL) => {
      const raw = String(url);
      if (raw.includes("/edge-session")) {
        return Promise.resolve(Response.json({
          enabled: true, mode: "prefer", base_path: "/_community/v1",
          expires_at: Math.floor(Date.now() / 1_000) + 900,
        }));
      }
      if (raw.includes("/community/state")) return new Promise<Response>(() => {});
      if (raw.includes("/_community/")) return Promise.resolve(new Response("down", { status: 503 }));
      return Promise.resolve(Response.json({ items: [{ content_id: 42 }], unread: true }));
    }) as typeof fetch;

    expect((await getPublicCommunity()).items[0]?.content_id).toBe(42);
  });

  test("discovers a fresh edge grant after logout", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    let grants = 0;
    globalThis.fetch = ((url: RequestInfo | URL) => {
      const raw = String(url);
      if (raw.includes("/edge-session")) {
        grants += 1;
        return Promise.resolve(
          new Response(
            JSON.stringify({
              enabled: true,
              mode: "prefer",
              base_path: "/_community/v1",
              expires_at: Math.floor(Date.now() / 1_000) + 900,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (raw.includes("/auth/logout")) {
        return Promise.resolve(
          new Response(JSON.stringify({ ok: true }), {
            headers: { "content-type": "application/json" },
          }),
        );
      }
      if (raw.includes("/community/state")) {
        return Promise.resolve(
          new Response(JSON.stringify({ unread: false, latest_content_id: 42 }), {
            headers: { "content-type": "application/json" },
          }),
        );
      }
      return Promise.resolve(
        new Response(
          JSON.stringify({
            community: { id: "51115212285814", name: "HONE 官方社区" },
            items: [],
            next_before: null,
            unread: false,
          }),
          { headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;

    await getPublicCommunity();
    await publicLogout();
    await getPublicCommunity();

    expect(grants).toBe(2);
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

  test("falls back after successful edge headers when the file body is interrupted", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    const requested: string[] = [];
    globalThis.fetch = ((url: RequestInfo | URL) => {
      const raw = String(url);
      requested.push(raw);
      if (raw.includes("/edge-session")) {
        return Promise.resolve(Response.json({
          enabled: true, mode: "prefer", base_path: "/_community/v1",
          expires_at: Math.floor(Date.now() / 1_000) + 900,
        }));
      }
      if (raw.includes("/community/state")) {
        return Promise.resolve(Response.json({ unread: false, latest_content_id: null }));
      }
      if (raw.includes("/feed/")) return Promise.resolve(Response.json({ items: [] }));
      if (raw.includes("/_community/v1/resources/")) {
        return Promise.resolve(new Response(new ReadableStream({
          start(controller) { controller.error(new Error("interrupted edge stream")); },
        }), { headers: { "content-type": "application/pdf" } }));
      }
      return Promise.resolve(new Response("%PDF-complete", {
        headers: { "content-type": "application/pdf" },
      }));
    }) as typeof fetch;

    await getPublicCommunity();
    const blob = await getPublicCommunityResourceBlob(99, "0123456789ab", "/_community/v1/resources/99/0123456789ab");
    expect(await blob.text()).toBe("%PDF-complete");
    expect(requested.at(-1)).toContain("/api/public/community/resources/99?v=0123456789ab");
  });

  test("cancels a resource request without starting a fallback transfer", async () => {
    const controller = new AbortController();
    let requests = 0;
    globalThis.fetch = ((_url: RequestInfo | URL, init?: RequestInit) => {
      requests += 1;
      return new Promise<Response>((_resolve, reject) => {
        init?.signal?.addEventListener("abort", () => reject(new DOMException("Aborted", "AbortError")), { once: true });
      });
    }) as typeof fetch;

    const pending = getPublicCommunityResourceBlob(99, undefined, undefined, controller.signal);
    controller.abort();
    await expect(pending).rejects.toMatchObject({ name: "AbortError" });
    expect(requests).toBe(1);
  });

  test("downloads a generated file through the authenticated public route", async () => {
    let requestedUrl = "";
    let credentials: RequestCredentials | undefined;
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      requestedUrl = String(url);
      credentials = init?.credentials;
      return Promise.resolve(
        new Response(new Uint8Array([37, 80, 68, 70, 45]), {
          headers: { "content-type": "application/pdf" },
        }),
      );
    }) as typeof fetch;

    const blob = await getPublicGeneratedFileBlob(
      "<absolute-path>/ANET-财报前瞻.pdf",
    );

    expect(requestedUrl).toContain(
      "/api/public/file?path=%3Cabsolute-path%3E%2FANET-%E8%B4%A2%E6%8A%A5%E5%89%8D%E7%9E%BB.pdf",
    );
    expect(credentials).toBe("include");
    expect(blob.type).toBe("application/pdf");
    expect(blob.size).toBe(5);
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

  test("rejects mismatched edge paths and preflights document fallback", async () => {
    setPublicCommunityEdgeDiscoveryForTests(true);
    const requested: Array<{ url: string; method: string }> = [];
    globalThis.fetch = ((url: RequestInfo | URL, init?: RequestInit) => {
      const raw = String(url);
      requested.push({ url: raw, method: init?.method ?? "GET" });
      if (raw.includes("/edge-session")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              enabled: true,
              mode: "prefer",
              base_path: "/_community/v1",
              expires_at: Math.floor(Date.now() / 1_000) + 900,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (raw.includes("/feed/latest.json")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              community: { id: "51115212285814", name: "HONE 官方社区" },
              items: [],
              next_before: null,
              unread: false,
            }),
            { headers: { "content-type": "application/json" } },
          ),
        );
      }
      if (raw.includes("/api/public/community/state")) {
        return Promise.resolve(
          new Response(JSON.stringify({ unread: false, latest_content_id: null }), {
            headers: { "content-type": "application/json" },
          }),
        );
      }
      return Promise.resolve(new Response("edge unavailable", { status: 503 }));
    }) as typeof fetch;
    await getPublicCommunity();

    expect(
      publicCommunityResourceUrl(
        99,
        "0123456789ab",
        "/_community/v1/resources/100/0123456789ab",
      ),
    ).toContain("/api/public/community/resources/99?v=0123456789ab");
    expect(
      publicCommunityResourceUrl(99, "0123456789ab", "https://evil.example/x"),
    ).toContain("/api/public/community/resources/99?v=0123456789ab");

    const resolved = await resolvePublicCommunityResourceUrl(
      99,
      "0123456789ab",
      "/_community/v1/resources/99/0123456789ab",
    );
    expect(requested.at(-1)?.method).toBe("HEAD");
    expect(resolved).toContain("/api/public/community/resources/99?v=0123456789ab");
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
