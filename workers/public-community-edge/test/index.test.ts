import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  type CommunityBucket,
  type EdgeCache,
  type EdgeExecutionContext,
  type Env,
  handleRequest,
} from "../src/index";

const SECRET = "test-only-community-edge-secret!";
const NOW = 1_800_000_000;
const FEED_PREFIX = "community/zsxq/51115212285814/delivery/v1";
const RESOURCE_PREFIX = `${FEED_PREFIX}/resources`;
const ASSET_PREFIX = "community/zsxq/51115212285814/resources";
const SHA256 = "a".repeat(64);
const VERSION = SHA256.slice(0, 12);

interface StoredObject {
  bytes: Uint8Array;
  etag: string;
}

class MockBucket implements CommunityBucket {
  readonly objects = new Map<string, StoredObject>();
  readonly getCalls: string[] = [];
  readonly headCalls: string[] = [];
  readonly failingKeys = new Set<string>();

  putText(key: string, value: string, etag = `"etag-${this.objects.size + 1}"`) {
    this.objects.set(key, { bytes: new TextEncoder().encode(value), etag });
  }

  putJson(key: string, value: unknown) {
    this.putText(key, JSON.stringify(value));
  }

  async get(key: string) {
    this.getCalls.push(key);
    if (this.failingKeys.has(key)) throw new Error("simulated R2 failure");
    const object = this.objects.get(key);
    if (!object) return null;
    const bytes = Uint8Array.from(object.bytes);
    return {
      size: bytes.byteLength,
      httpEtag: object.etag,
      body: new Response(bytes.buffer).body,
      text: async () => new TextDecoder().decode(bytes),
    };
  }

  async head(key: string) {
    this.headCalls.push(key);
    if (this.failingKeys.has(key)) throw new Error("simulated R2 failure");
    const object = this.objects.get(key);
    return object ? { size: object.bytes.byteLength, httpEtag: object.etag } : null;
  }
}

class MockCache implements EdgeCache {
  readonly entries = new Map<string, Response>();
  readonly matchCalls: string[] = [];
  readonly putCalls: string[] = [];
  readonly cookieHeaders: Array<string | null> = [];

  async match(request: Request) {
    this.matchCalls.push(request.url);
    this.cookieHeaders.push(request.headers.get("Cookie"));
    return this.entries.get(request.url)?.clone();
  }

  async put(request: Request, response: Response) {
    this.putCalls.push(request.url);
    this.cookieHeaders.push(request.headers.get("Cookie"));
    this.entries.set(request.url, response.clone());
  }
}

class MockContext implements EdgeExecutionContext {
  readonly pending: Promise<unknown>[] = [];

  waitUntil(promise: Promise<unknown>) {
    this.pending.push(promise);
  }

  async drain() {
    await Promise.all(this.pending);
  }
}

function encodeBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

async function signedToken(
  overrides: Partial<{ v: number; aud: string; sub: string; iat: number; exp: number }> = {},
) {
  const payload = {
    v: 1,
    aud: "hone-community-edge-v1",
    sub: "web:test-user",
    iat: NOW,
    exp: NOW + 900,
    ...overrides,
  };
  const payloadSegment = encodeBase64Url(new TextEncoder().encode(JSON.stringify(payload)));
  const key = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(SECRET),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  const signature = await crypto.subtle.sign(
    "HMAC",
    key,
    new TextEncoder().encode(payloadSegment),
  );
  return `${payloadSegment}.${encodeBase64Url(new Uint8Array(signature))}`;
}

function env(bucket: MockBucket, edgeDisabled = "false"): Env {
  return {
    COMMUNITY_BUCKET: bucket,
    COMMUNITY_EDGE_HMAC_SECRET: SECRET,
    EDGE_DISABLED: edgeDisabled,
    COMMUNITY_FEED_PREFIX: FEED_PREFIX,
    COMMUNITY_RESOURCE_PREFIX: RESOURCE_PREFIX,
    COMMUNITY_ASSET_PREFIX: ASSET_PREFIX,
    LEGACY_ORIGIN_URL: "https://origin.hone-claw.com",
  };
}

async function authenticatedRequest(
  path: string,
  init: RequestInit = {},
  token?: string,
): Promise<Request> {
  const edgeToken = token ?? (await signedToken());
  const headers = new Headers(init.headers);
  headers.set(
    "Cookie",
    `hone_web_session=public-session; ${"hone_community_edge"}=${edgeToken}; preference=compact`,
  );
  return new Request(`https://hone-claw.com${path}`, { ...init, headers });
}

function descriptor(
  resourceId: number,
  objectKey: string,
  overrides: Record<string, unknown> = {},
) {
  return {
    resource_id: resourceId,
    version: VERSION,
    sha256: SHA256,
    object_key: objectKey,
    content_type: "image/png",
    byte_size: 9,
    display_name: "chart.png",
    ...overrides,
  };
}

function putActiveIndex(bucket: MockBucket, resources: Record<string, string>) {
  bucket.putJson(`${RESOURCE_PREFIX}/active.json`, { v: 1, resources });
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date(NOW * 1000));
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("fail-closed activation", () => {
  it.each([undefined, "", "true", "TRUE", "unknown", "1"])(
    "returns 503 when EDGE_DISABLED is %s",
    async (edgeDisabled) => {
      const bucket = new MockBucket();
      const currentEnv = env(bucket);
      currentEnv.EDGE_DISABLED = edgeDisabled;
      const legacyFetch = vi.fn();
      vi.stubGlobal("fetch", legacyFetch);

      const response = await handleRequest(
        await authenticatedRequest("/_community/v1/feed/latest.json"),
        currentEnv,
      );

      expect(response.status).toBe(503);
      expect(response.headers.get("Cache-Control")).toBe("private, no-store");
      expect(legacyFetch).not.toHaveBeenCalled();
    },
  );

  it.each(["false", "FALSE", " 0 ", "no", "off"])(
    "only explicitly enables recognized false value %s",
    async (edgeDisabled) => {
      const response = await handleRequest(
        new Request("https://hone-claw.com/_community/v1/feed/latest.json"),
        env(new MockBucket(), edgeDisabled),
      );
      expect(response.status).toBe(401);
    },
  );

  it("fails closed when the secret or R2 binding is missing", async () => {
    const bucket = new MockBucket();
    const request = await authenticatedRequest("/_community/v1/feed/latest.json");
    expect(
      (await handleRequest(request, { ...env(bucket), COMMUNITY_EDGE_HMAC_SECRET: undefined })).status,
    ).toBe(503);
    expect(
      (
        await handleRequest(await authenticatedRequest("/_community/v1/feed/latest.json"), {
          ...env(bucket),
          COMMUNITY_BUCKET: undefined,
        })
      ).status,
    ).toBe(503);
  });

  it.each([
    "x".repeat(31),
    "x".repeat(1025),
    "密".repeat(342),
  ])("fails closed when the trimmed HMAC secret is outside 32..=1024 UTF-8 bytes", async (secret) => {
    const currentEnv = env(new MockBucket());
    currentEnv.COMMUNITY_EDGE_HMAC_SECRET = secret;
    const response = await handleRequest(
      await authenticatedRequest("/_community/v1/feed/latest.json"),
      currentEnv,
    );
    expect(response.status).toBe(503);
  });
});

describe("edge session authentication", () => {
  it("accepts the Rust golden vector and trims configured secret whitespace", async () => {
    const rustToken =
      "eyJ2IjoxLCJhdWQiOiJob25lLWNvbW11bml0eS1lZGdlLXYxIiwic3ViIjoid2ViOnVzZXItMTIzIiwiaWF0IjoxNzAwMDAwMDAwLCJleHAiOjE3MDAwMDA5MDB9.2TlI3FNyPYD4yUZeH31lyy2p3obnWcICpICJOzTz7V4";
    vi.setSystemTime(new Date(1_700_000_100 * 1000));
    const bucket = new MockBucket();
    bucket.putJson(`${FEED_PREFIX}/feed/latest.json`, { items: [] });
    const currentEnv = env(bucket);
    currentEnv.COMMUNITY_EDGE_HMAC_SECRET = "  edge-secret-test-vector-32-bytes\n";

    const response = await handleRequest(
      await authenticatedRequest("/_community/v1/feed/latest.json", {}, rustToken),
      currentEnv,
    );
    expect(response.status).toBe(200);
  });

  it("rejects an expired token", async () => {
    const token = await signedToken({ iat: NOW - 900, exp: NOW });
    const response = await handleRequest(
      await authenticatedRequest("/_community/v1/feed/latest.json", {}, token),
      env(new MockBucket()),
    );
    expect(response.status).toBe(401);
  });

  it("rejects a tampered signature and wrong audience", async () => {
    const valid = await signedToken();
    const tampered = `${valid.slice(0, -1)}${valid.endsWith("A") ? "B" : "A"}`;
    expect(
      (
        await handleRequest(
          await authenticatedRequest("/_community/v1/feed/latest.json", {}, tampered),
          env(new MockBucket()),
        )
      ).status,
    ).toBe(401);

    expect(
      (
        await handleRequest(
          await authenticatedRequest(
            "/_community/v1/feed/latest.json",
            {},
            await signedToken({ aud: "another-audience" }),
          ),
          env(new MockBucket()),
        )
      ).status,
    ).toBe(401);
  });

  it("rejects duplicate edge cookies", async () => {
    const token = await signedToken();
    const request = new Request("https://hone-claw.com/_community/v1/feed/latest.json", {
      headers: { Cookie: `hone_community_edge=${token}; hone_community_edge=${token}` },
    });
    expect((await handleRequest(request, env(new MockBucket()))).status).toBe(401);
  });
});

describe("R2 feed delivery", () => {
  it("serves latest and cursor pages from their exact keys", async () => {
    const bucket = new MockBucket();
    bucket.putJson(`${FEED_PREFIX}/feed/latest.json`, { items: ["latest"], next_before: 42 });
    bucket.putJson(`${FEED_PREFIX}/feed/pages/42.json`, { items: ["older"] });

    const latest = await handleRequest(
      await authenticatedRequest("/_community/v1/feed/latest.json"),
      env(bucket),
    );
    expect(latest.status).toBe(200);
    expect(await latest.json()).toEqual({ items: ["latest"], next_before: 42 });
    expect(latest.headers.get("Content-Type")).toBe("application/json; charset=utf-8");
    expect(latest.headers.get("Cache-Control")).toContain("max-age=30");

    const page = await handleRequest(
      await authenticatedRequest("/_community/v1/feed/pages/42.json"),
      env(bucket),
    );
    expect(page.status).toBe(200);
    expect(await page.json()).toEqual({ items: ["older"] });
    expect(bucket.getCalls).toEqual([
      `${FEED_PREFIX}/feed/latest.json`,
      `${FEED_PREFIX}/feed/pages/42.json`,
    ]);
  });

  it("uses R2 HEAD without reading the body", async () => {
    const bucket = new MockBucket();
    const key = `${FEED_PREFIX}/feed/latest.json`;
    bucket.putText(key, "{\"items\":[]}");

    const response = await handleRequest(
      await authenticatedRequest("/_community/v1/feed/latest.json", { method: "HEAD" }),
      env(bucket),
    );
    expect(response.status).toBe(200);
    expect(await response.text()).toBe("");
    expect(bucket.headCalls).toEqual([key]);
    expect(bucket.getCalls).toEqual([]);
  });

  it("honors a safe R2 ETag", async () => {
    const bucket = new MockBucket();
    const key = `${FEED_PREFIX}/feed/latest.json`;
    bucket.putText(key, "{\"items\":[]}", '"feed-etag"');
    const request = await authenticatedRequest("/_community/v1/feed/latest.json", {
      headers: { "If-None-Match": 'W/"feed-etag"' },
    });
    const response = await handleRequest(request, env(bucket));
    expect(response.status).toBe(304);
    expect(await response.text()).toBe("");
  });
});

describe("R2 resource delivery", () => {
  it("validates the descriptor and streams an immutable passive object", async () => {
    const bucket = new MockBucket();
    const assetKey = `${ASSET_PREFIX}/7-${SHA256}.png`;
    putActiveIndex(bucket, { "7": VERSION });
    bucket.putText(assetKey, "png-bytes");
    bucket.putJson(`${RESOURCE_PREFIX}/7/${VERSION}.json`, descriptor(7, assetKey));

    const response = await handleRequest(
      await authenticatedRequest(`/_community/v1/resources/7/${VERSION}`),
      env(bucket),
    );
    expect(response.status).toBe(200);
    expect(await response.text()).toBe("png-bytes");
    expect(response.headers.get("Content-Type")).toBe("image/png");
    expect(response.headers.get("Content-Disposition")).toBe("inline");
    expect(response.headers.get("Cache-Control")).toBe("private, no-cache");
    expect(response.headers.get("ETag")).toBe(`"${SHA256}"`);
    expect(bucket.getCalls).toEqual([
      `${RESOURCE_PREFIX}/active.json`,
      `${RESOURCE_PREFIX}/7/${VERSION}.json`,
      assetKey,
    ]);
  });

  it("serves resource HEAD from metadata and downloads non-passive types", async () => {
    const bucket = new MockBucket();
    const assetKey = `${ASSET_PREFIX}/8-${SHA256}.docx`;
    putActiveIndex(bucket, { "8": VERSION });
    bucket.putText(assetKey, "doc-bytes");
    bucket.putJson(
      `${RESOURCE_PREFIX}/8/${VERSION}.json`,
      descriptor(8, assetKey, {
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      }),
    );

    const response = await handleRequest(
      await authenticatedRequest(`/_community/v1/resources/8/${VERSION}`, { method: "HEAD" }),
      env(bucket),
    );
    expect(response.status).toBe(200);
    expect(await response.text()).toBe("");
    expect(response.headers.get("Content-Type")).toBe("application/octet-stream");
    expect(response.headers.get("Content-Disposition")).toContain("attachment");
    expect(bucket.headCalls).toEqual([assetKey]);
  });

  it("rejects descriptor traversal without reading the escaped key or falling back", async () => {
    const bucket = new MockBucket();
    const descriptorKey = `${RESOURCE_PREFIX}/9/${VERSION}.json`;
    putActiveIndex(bucket, { "9": VERSION });
    bucket.putJson(descriptorKey, descriptor(9, `${ASSET_PREFIX}/../private-secret`));
    const legacyFetch = vi.fn();
    vi.stubGlobal("fetch", legacyFetch);

    const response = await handleRequest(
      await authenticatedRequest(`/_community/v1/resources/9/${VERSION}`),
      env(bucket),
    );
    expect(response.status).toBe(502);
    expect(await response.json()).toEqual({ error: "invalid_resource_descriptor" });
    expect(bucket.getCalls).toEqual([`${RESOURCE_PREFIX}/active.json`, descriptorKey]);
    expect(legacyFetch).not.toHaveBeenCalled();
  });

  it.each([
    { resource_id: 11 },
    { version: "b".repeat(12) },
    { sha256: "b".repeat(64) },
    { byte_size: -1 },
  ])("rejects inconsistent descriptor fields: %j", async (override) => {
    const bucket = new MockBucket();
    const assetKey = `${ASSET_PREFIX}/10-${SHA256}.png`;
    putActiveIndex(bucket, { "10": VERSION });
    bucket.putJson(`${RESOURCE_PREFIX}/10/${VERSION}.json`, descriptor(10, assetKey, override));
    const response = await handleRequest(
      await authenticatedRequest(`/_community/v1/resources/10/${VERSION}`),
      env(bucket),
    );
    expect(response.status).toBe(502);
  });

  it("rejects an object whose size differs from the signed descriptor metadata", async () => {
    const bucket = new MockBucket();
    const assetKey = `${ASSET_PREFIX}/12-${SHA256}.png`;
    putActiveIndex(bucket, { "12": VERSION });
    bucket.putText(assetKey, "different-size");
    bucket.putJson(`${RESOURCE_PREFIX}/12/${VERSION}.json`, descriptor(12, assetKey));
    const response = await handleRequest(
      await authenticatedRequest(`/_community/v1/resources/12/${VERSION}`),
      env(bucket),
    );
    expect(response.status).toBe(502);
    expect(response.headers.get("Cache-Control")).toBe("private, no-store");
  });

  it("rejects a descriptor above the shared 128 MiB resource limit", async () => {
    const bucket = new MockBucket();
    const assetKey = `${ASSET_PREFIX}/13-${SHA256}.png`;
    putActiveIndex(bucket, { "13": VERSION });
    bucket.putJson(
      `${RESOURCE_PREFIX}/13/${VERSION}.json`,
      descriptor(13, assetKey, { byte_size: 128 * 1024 * 1024 + 1 }),
    );

    const response = await handleRequest(
      await authenticatedRequest(`/_community/v1/resources/13/${VERSION}`),
      env(bucket),
    );
    expect(response.status).toBe(502);
    expect(await response.json()).toEqual({ error: "invalid_resource_descriptor" });
    expect(bucket.getCalls).toEqual([
      `${RESOURCE_PREFIX}/active.json`,
      `${RESOURCE_PREFIX}/13/${VERSION}.json`,
    ]);
  });

  it.each(["missing", "error"])(
    "never internally falls back for resource HEAD when the descriptor is %s",
    async (failure) => {
      const bucket = new MockBucket();
      const descriptorKey = `${RESOURCE_PREFIX}/15/${VERSION}.json`;
      putActiveIndex(bucket, { "15": VERSION });
      if (failure === "error") bucket.failingKeys.add(descriptorKey);
      const legacyFetch = vi.fn();
      vi.stubGlobal("fetch", legacyFetch);

      const response = await handleRequest(
        await authenticatedRequest(`/_community/v1/resources/15/${VERSION}`, { method: "HEAD" }),
        env(bucket),
      );
      expect(response.status).toBe(502);
      expect(legacyFetch).not.toHaveBeenCalled();
    },
  );

  it("checks the active index before shared cache so revocation cannot serve cached bytes", async () => {
    const bucket = new MockBucket();
    const assetKey = `${ASSET_PREFIX}/16-${SHA256}.png`;
    putActiveIndex(bucket, { "16": VERSION });
    bucket.putText(assetKey, "png-bytes");
    bucket.putJson(`${RESOURCE_PREFIX}/16/${VERSION}.json`, descriptor(16, assetKey));
    const cache = new MockCache();
    const context = new MockContext();
    const path = `/_community/v1/resources/16/${VERSION}`;

    const first = await handleRequest(await authenticatedRequest(path), env(bucket), context, cache);
    expect(first.status).toBe(200);
    expect(await first.text()).toBe("png-bytes");
    await context.drain();
    expect(cache.putCalls).toHaveLength(1);

    putActiveIndex(bucket, {});
    const legacyFetch = vi.fn();
    vi.stubGlobal("fetch", legacyFetch);
    const second = await handleRequest(
      await authenticatedRequest(path),
      env(bucket),
      new MockContext(),
      cache,
    );
    expect(second.status).toBe(404);
    expect(await second.json()).toEqual({ error: "resource_not_active" });
    expect(cache.matchCalls).toHaveLength(1);
    expect(bucket.getCalls.filter((key) => key === assetKey)).toHaveLength(1);
    expect(legacyFetch).not.toHaveBeenCalled();
  });

  it("fails closed before cache or origin when the active index is missing", async () => {
    const bucket = new MockBucket();
    const cache = new MockCache();
    const legacyFetch = vi.fn();
    vi.stubGlobal("fetch", legacyFetch);

    const response = await handleRequest(
      await authenticatedRequest(`/_community/v1/resources/17/${VERSION}`),
      env(bucket),
      new MockContext(),
      cache,
    );
    expect(response.status).toBe(503);
    expect(cache.matchCalls).toEqual([]);
    expect(legacyFetch).not.toHaveBeenCalled();
  });
});

describe("authenticated Cloudflare cache", () => {
  it("never reads shared cache before edge authentication", async () => {
    const cache = new MockCache();
    const response = await handleRequest(
      new Request("https://hone-claw.com/_community/v1/feed/latest.json"),
      env(new MockBucket()),
      undefined,
      cache,
    );

    expect(response.status).toBe(401);
    expect(cache.matchCalls).toEqual([]);
    expect(cache.putCalls).toEqual([]);
  });

  it("shares only authenticated static R2 bytes across users with a cookie-free key", async () => {
    const bucket = new MockBucket();
    bucket.putJson(`${FEED_PREFIX}/feed/latest.json`, { items: ["shared-static-feed"] });
    const cache = new MockCache();
    const firstContext = new MockContext();

    const first = await handleRequest(
      await authenticatedRequest(
        "/_community/v1/feed/latest.json",
        {},
        await signedToken({ sub: "web:user-a" }),
      ),
      env(bucket),
      firstContext,
      cache,
    );
    expect(first.status).toBe(200);
    expect(await first.json()).toEqual({ items: ["shared-static-feed"] });
    await firstContext.drain();

    const cacheKey = "https://hone-claw.com/_community/v1/feed/latest.json";
    expect(cache.putCalls).toEqual([cacheKey]);
    const stored = cache.entries.get(cacheKey);
    expect(stored?.headers.get("Cache-Control")).toBe("public, max-age=30, s-maxage=30");
    expect(stored?.headers.get("Vary")).toBeNull();

    const second = await handleRequest(
      await authenticatedRequest(
        "/_community/v1/feed/latest.json",
        {},
        await signedToken({ sub: "web:user-b" }),
      ),
      env(bucket),
      new MockContext(),
      cache,
    );
    expect(second.status).toBe(200);
    expect(await second.json()).toEqual({ items: ["shared-static-feed"] });
    expect(second.headers.get("Cache-Control")).toContain("private");
    expect(second.headers.get("Vary")).toBe("Cookie");
    expect(second.headers.get("X-Hone-Community-Cache")).toBeNull();
    expect(bucket.getCalls).toEqual([`${FEED_PREFIX}/feed/latest.json`]);
    expect(cache.matchCalls).toEqual([cacheKey, cacheKey]);
    expect(cache.cookieHeaders.every((value) => value === null)).toBe(true);
  });

  it("does not cache HEAD or a legacy fallback", async () => {
    const bucket = new MockBucket();
    const cache = new MockCache();
    const legacyFetch = vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) =>
      new Response(JSON.stringify({ items: ["legacy"] }), {
        status: 200,
        headers: { "Content-Length": "20" },
      }),
    );
    vi.stubGlobal("fetch", legacyFetch);

    const head = await handleRequest(
      await authenticatedRequest("/_community/v1/feed/latest.json", { method: "HEAD" }),
      env(bucket),
      new MockContext(),
      cache,
    );
    expect(head.status).toBe(200);

    const get = await handleRequest(
      await authenticatedRequest("/_community/v1/feed/latest.json"),
      env(bucket),
      new MockContext(),
      cache,
    );
    expect(get.status).toBe(200);
    expect(cache.matchCalls).toEqual(["https://hone-claw.com/_community/v1/feed/latest.json"]);
    expect(cache.putCalls).toEqual([]);
  });
});

describe("legacy fallback and route boundary", () => {
  it("falls back on an R2 miss and strips only the edge cookie", async () => {
    const legacyFetch = vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) =>
      new Response(JSON.stringify({ items: ["legacy"] }), {
        status: 200,
        headers: { "Content-Type": "application/json", "Content-Length": "20" },
      }),
    );
    vi.stubGlobal("fetch", legacyFetch);

    const response = await handleRequest(
      await authenticatedRequest("/_community/v1/feed/pages/77.json"),
      env(new MockBucket()),
    );
    expect(response.status).toBe(200);
    expect(await response.json()).toEqual({ items: ["legacy"] });
    expect(legacyFetch).toHaveBeenCalledOnce();
    const [input, init] = legacyFetch.mock.calls[0];
    expect(String(input)).toBe(
      "https://origin.hone-claw.com/api/public/community?before=77&limit=20",
    );
    const forwarded = new Headers(init?.headers);
    expect(forwarded.get("Accept-Encoding")).toBe("identity");
    expect(forwarded.get("Cookie")).toBe(
      "hone_web_session=public-session",
    );
    expect(forwarded.get("Cookie")).not.toContain("hone_community_edge");
    expect(response.headers.get("Cache-Control")).toBe("private, no-store");
  });

  it("falls back on an R2 error for a versioned resource", async () => {
    const bucket = new MockBucket();
    putActiveIndex(bucket, { "14": VERSION });
    bucket.failingKeys.add(`${RESOURCE_PREFIX}/14/${VERSION}.json`);
    const legacyFetch = vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) =>
      new Response("legacy-png", {
        status: 200,
        headers: { "Content-Type": "image/png", "Content-Length": "10" },
      }),
    );
    vi.stubGlobal("fetch", legacyFetch);

    const response = await handleRequest(
      await authenticatedRequest(`/_community/v1/resources/14/${VERSION}`),
      env(bucket),
    );
    expect(response.status).toBe(200);
    expect(await response.text()).toBe("legacy-png");
    expect(String(legacyFetch.mock.calls[0][0])).toBe(
      `https://origin.hone-claw.com/api/public/community/resources/14?v=${VERSION}`,
    );
  });

  it.each([null, "0", String(128 * 1024 * 1024 + 1)])(
    "fails closed for a successful legacy resource with invalid Content-Length %s",
    async (contentLength) => {
      const bucket = new MockBucket();
      putActiveIndex(bucket, { "18": VERSION });
      bucket.failingKeys.add(`${RESOURCE_PREFIX}/18/${VERSION}.json`);
      const headers = new Headers({ "Content-Type": "image/png" });
      if (contentLength !== null) headers.set("Content-Length", contentLength);
      const legacyFetch = vi.fn(async () => new Response("legacy-png", { status: 200, headers }));
      vi.stubGlobal("fetch", legacyFetch);

      const response = await handleRequest(
        await authenticatedRequest(`/_community/v1/resources/18/${VERSION}`),
        env(bucket),
      );
      expect(response.status).toBe(502);
      expect(await response.json()).toEqual({ error: "legacy_response_size_invalid" });
    },
  );

  it("refuses a configured legacy origin outside the fixed allowlist", async () => {
    const legacyFetch = vi.fn();
    vi.stubGlobal("fetch", legacyFetch);
    const currentEnv = env(new MockBucket());
    currentEnv.LEGACY_ORIGIN_URL = "https://attacker.example";

    const response = await handleRequest(
      await authenticatedRequest("/_community/v1/feed/latest.json"),
      currentEnv,
    );
    expect(response.status).toBe(502);
    expect(legacyFetch).not.toHaveBeenCalled();
  });

  it("rejects unsupported methods and paths without origin access", async () => {
    const legacyFetch = vi.fn();
    vi.stubGlobal("fetch", legacyFetch);
    const currentEnv = env(new MockBucket());

    const method = await handleRequest(
      await authenticatedRequest("/_community/v1/feed/latest.json", { method: "POST" }),
      currentEnv,
    );
    expect(method.status).toBe(405);
    expect(method.headers.get("Allow")).toBe("GET, HEAD");

    const path = await handleRequest(
      await authenticatedRequest("/_community/v1/resources/1/%2e%2e%2fsecret"),
      currentEnv,
    );
    expect(path.status).toBe(404);
    expect(legacyFetch).not.toHaveBeenCalled();
  });
});
