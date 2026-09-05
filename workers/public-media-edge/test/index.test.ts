import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { type Env, type MediaBucket, handleRequest } from "../src/index";

const SECRET = "test-only-media-edge-secret-32b!!";
const NOW = 1_800_000_000;
const UPLOAD_PREFIX = "public-uploads";
const OWNER = "web-user-1";
const OWNER_PREFIX = `${UPLOAD_PREFIX}/${OWNER}/`;
const KEY = `${OWNER_PREFIX}2026-08-30/0123456789abcdef-pasted.png`;

const PNG = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x01]);
const JPEG = new Uint8Array([0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10]);
const WEBP = new Uint8Array([
  0x52, 0x49, 0x46, 0x46, 0x10, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50, 0x00,
]);
const SVG = new TextEncoder().encode('<svg xmlns="http://www.w3.org/2000/svg"><script/></svg>');
const HTML = new TextEncoder().encode("<!doctype html><script>alert(1)</script>");

interface StoredObject {
  bytes: Uint8Array;
  contentType?: string;
  etag: string;
}

class MockBucket implements MediaBucket {
  readonly objects = new Map<string, StoredObject>();
  readonly putCalls: Array<{ key: string; contentType?: string; bytes: number }> = [];
  readonly failingKeys = new Set<string>();

  seed(key: string, bytes: Uint8Array, contentType?: string) {
    this.objects.set(key, { bytes, contentType, etag: '"seeded"' });
  }

  async get(key: string) {
    if (this.failingKeys.has(key)) throw new Error("simulated R2 failure");
    const object = this.objects.get(key);
    if (!object) return null;
    const bytes = Uint8Array.from(object.bytes);
    return {
      size: bytes.byteLength,
      httpEtag: object.etag,
      httpMetadata: { contentType: object.contentType },
      body: new Response(bytes.buffer).body,
    };
  }

  async head(key: string) {
    if (this.failingKeys.has(key)) throw new Error("simulated R2 failure");
    const object = this.objects.get(key);
    return object
      ? {
          size: object.bytes.byteLength,
          httpEtag: object.etag,
          httpMetadata: { contentType: object.contentType },
        }
      : null;
  }

  async put(key: string, value: ArrayBuffer, options?: { httpMetadata?: { contentType?: string } }) {
    if (this.failingKeys.has(key)) throw new Error("simulated R2 failure");
    const bytes = new Uint8Array(value);
    this.putCalls.push({
      key,
      contentType: options?.httpMetadata?.contentType,
      bytes: bytes.byteLength,
    });
    this.objects.set(key, {
      bytes,
      contentType: options?.httpMetadata?.contentType,
      etag: '"written"',
    });
    return undefined;
  }
}

function encodeBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

async function sign(payload: unknown, secret = SECRET): Promise<string> {
  const segment = encodeBase64Url(new TextEncoder().encode(JSON.stringify(payload)));
  const key = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  const signature = await crypto.subtle.sign("HMAC", key, new TextEncoder().encode(segment));
  return `${segment}.${encodeBase64Url(new Uint8Array(signature))}`;
}

function writeClaims(overrides: Record<string, unknown> = {}) {
  return {
    v: 1,
    aud: "hone-media-edge-v1",
    op: "put",
    sub: OWNER,
    pfx: OWNER_PREFIX,
    key: KEY,
    ct: "image/png",
    max: 1_000_000,
    iat: NOW,
    exp: NOW + 120,
    ...overrides,
  };
}

function readClaims(overrides: Record<string, unknown> = {}) {
  return {
    v: 1,
    aud: "hone-media-edge-v1",
    op: "get",
    sub: OWNER,
    pfx: OWNER_PREFIX,
    iat: NOW,
    exp: NOW + 600,
    ...overrides,
  };
}

let bucket: MockBucket;

function env(overrides: Partial<Env> = {}): Env {
  return {
    MEDIA_BUCKET: bucket,
    MEDIA_EDGE_HMAC_SECRET: SECRET,
    MEDIA_EDGE_DISABLED: "false",
    MEDIA_UPLOAD_PREFIX: UPLOAD_PREFIX,
    ...overrides,
  };
}

function putRequest(
  token: string,
  body: Uint8Array,
  options: { key?: string; contentLength?: number; headers?: Record<string, string> } = {},
) {
  const headers = new Headers({
    "X-Hone-Media-Token": token,
    "Content-Length": String(options.contentLength ?? body.byteLength),
    Origin: "https://hone-claw.com",
    ...options.headers,
  });
  return new Request(`https://hone-claw.com/_media/v1/o/${options.key ?? KEY}`, {
    method: "PUT",
    headers,
    body: Uint8Array.from(body),
  });
}

function getRequest(token: string | null, key = KEY, method: "GET" | "HEAD" = "GET") {
  return getRequestWithCookieHeader(
    token === null ? null : `hone_media_edge=${token}`,
    key,
    method,
  );
}

function getRequestWithCookieHeader(
  cookie: string | null,
  key = KEY,
  method: "GET" | "HEAD" = "GET",
) {
  const headers = new Headers();
  if (cookie !== null) headers.set("Cookie", cookie);
  return new Request(`https://hone-claw.com/_media/v1/o/${key}`, { method, headers });
}

beforeEach(() => {
  bucket = new MockBucket();
  vi.useFakeTimers();
  vi.setSystemTime(NOW * 1000);
});

afterEach(() => {
  vi.useRealTimers();
});

// Byte-for-byte tokens produced by the Rust origin
// (crates/hone-web-api/src/routes/public_media.rs, same test secret). These pin
// the wire format across the two languages: if either side changes its claim
// order, JSON encoding, or signing input, one of these stops verifying and the
// break shows up here instead of as blank images in production.
describe("cross-language token vectors", () => {
  const ORIGIN_READ_TOKEN =
    "eyJ2IjoxLCJhdWQiOiJob25lLW1lZGlhLWVkZ2UtdjEiLCJvcCI6ImdldCIsInN1YiI6IndlYi11c2VyLTEiLCJwZngiOiJwdWJsaWMtdXBsb2Fkcy93ZWItdXNlci0xLyIsImlhdCI6MTcwMDAwMDAwMCwiZXhwIjoxNzAwMDAwOTAwfQ.cVELwXFuzF90eMPYKsVnPYAIXvx2Gj53NnuMyhS6VLg";
  const ORIGIN_WRITE_TOKEN =
    "eyJ2IjoxLCJhdWQiOiJob25lLW1lZGlhLWVkZ2UtdjEiLCJvcCI6InB1dCIsInN1YiI6IndlYi11c2VyLTEiLCJwZngiOiJwdWJsaWMtdXBsb2Fkcy93ZWItdXNlci0xLyIsImtleSI6InB1YmxpYy11cGxvYWRzL3dlYi11c2VyLTEvMjAyNi0wOC0zMC9hYmMtaW1hZ2UucG5nIiwiY3QiOiJpbWFnZS9wbmciLCJtYXgiOjQwOTYsImlhdCI6MTcwMDAwMDAwMCwiZXhwIjoxNzAwMDAwMTIwfQ.7vEJxs8VnJ4ENcOzGkKpcfScPUDGD4NJJI8NCgxNfjk";
  const ORIGIN_KEY = `${UPLOAD_PREFIX}/${OWNER}/2026-08-30/abc-image.png`;

  beforeEach(() => {
    vi.setSystemTime(1_700_000_060 * 1000);
  });

  it("accepts the origin's read cookie", async () => {
    bucket.seed(ORIGIN_KEY, PNG, "image/png");
    const response = await handleRequest(getRequest(ORIGIN_READ_TOKEN, ORIGIN_KEY), env());
    expect(response.status).toBe(200);
    expect(new Uint8Array(await response.arrayBuffer())).toEqual(PNG);
  });

  it("accepts the origin's upload grant", async () => {
    const request = putRequest(ORIGIN_WRITE_TOKEN, PNG, { key: ORIGIN_KEY });
    const response = await handleRequest(request, env());
    expect(response.status).toBe(201);
    expect(bucket.putCalls).toEqual([
      { key: ORIGIN_KEY, contentType: "image/png", bytes: PNG.byteLength },
    ]);
  });
});

describe("activation and transport", () => {
  it("fails closed when the disable switch is absent", async () => {
    const response = await handleRequest(
      getRequest(await sign(readClaims())),
      env({ MEDIA_EDGE_DISABLED: undefined }),
    );
    expect(response.status).toBe(503);
  });

  it("fails closed when the secret is missing, short, or the bucket is unbound", async () => {
    const cookie = await sign(readClaims());
    for (const overrides of [
      { MEDIA_EDGE_HMAC_SECRET: undefined },
      { MEDIA_EDGE_HMAC_SECRET: "too-short" },
      { MEDIA_BUCKET: undefined },
    ] satisfies Partial<Env>[]) {
      const response = await handleRequest(getRequest(cookie), env(overrides));
      expect(response.status).toBe(503);
    }
  });

  it("rejects a cross-origin request outright", async () => {
    const token = await sign(writeClaims());
    const response = await handleRequest(
      putRequest(token, PNG, { headers: { Origin: "https://evil.example" } }),
      env(),
    );
    expect(response.status).toBe(403);
    expect(await response.json()).toEqual({ error: "cross_origin_rejected" });
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("rejects a cross-site fetch that omits Origin but declares Sec-Fetch-Site", async () => {
    const token = await sign(writeClaims());
    const request = new Request(`https://hone-claw.com/_media/v1/o/${KEY}`, {
      method: "PUT",
      headers: {
        "X-Hone-Media-Token": token,
        "Content-Length": String(PNG.byteLength),
        "Sec-Fetch-Site": "cross-site",
      },
      body: Uint8Array.from(PNG),
    });
    const response = await handleRequest(request, env());
    expect(response.status).toBe(403);
  });

  it("refuses methods outside GET, HEAD and PUT, including CORS preflight", async () => {
    for (const method of ["OPTIONS", "POST", "DELETE"]) {
      const response = await handleRequest(
        new Request(`https://hone-claw.com/_media/v1/o/${KEY}`, { method }),
        env(),
      );
      expect(response.status).toBe(405);
      expect(response.headers.get("Allow")).toBe("GET, HEAD, PUT");
    }
  });

  it("locks down headers on every response", async () => {
    bucket.seed(KEY, PNG, "image/png");
    const response = await handleRequest(getRequest(await sign(readClaims())), env());
    expect(response.status).toBe(200);
    expect(response.headers.get("Content-Security-Policy")).toBe("default-src 'none'; sandbox");
    expect(response.headers.get("X-Content-Type-Options")).toBe("nosniff");
    expect(response.headers.get("Referrer-Policy")).toBe("no-referrer");
    expect(response.headers.get("Cross-Origin-Resource-Policy")).toBe("same-origin");
    expect(response.headers.get("Content-Disposition")).toBe("inline");
    expect(response.headers.get("Cache-Control")).toContain("private");
    expect(response.headers.has("Access-Control-Allow-Origin")).toBe(false);
  });
});

describe("object key shape", () => {
  it("rejects traversal, encoded separators, and wrong depth", async () => {
    const cookie = await sign(readClaims());
    for (const key of [
      `${UPLOAD_PREFIX}/${OWNER}/../other/x.png`,
      `${UPLOAD_PREFIX}/${OWNER}/./x.png`,
      `${UPLOAD_PREFIX}/${OWNER}/2026-08-30/%2e%2e/x.png`,
      `${UPLOAD_PREFIX}/${OWNER}/x.png`,
      `${UPLOAD_PREFIX}/${OWNER}/2026-08-30/nested/x.png`,
      `other-prefix/${OWNER}/2026-08-30/x.png`,
      `${UPLOAD_PREFIX}/${OWNER}/2026-08-30/`,
    ]) {
      const response = await handleRequest(getRequest(cookie, key), env());
      expect([403, 404]).toContain(response.status);
    }
  });
});

describe("read authorization", () => {
  it("requires a session cookie", async () => {
    bucket.seed(KEY, PNG, "image/png");
    const response = await handleRequest(getRequest(null), env());
    expect(response.status).toBe(401);
    expect(await response.json()).toEqual({ error: "missing_media_session" });
  });

  it("rejects a forged, expired, or wrong-audience cookie", async () => {
    bucket.seed(KEY, PNG, "image/png");
    const forged = await sign(readClaims(), "a-different-secret-of-32-bytes!!!");
    for (const cookie of [
      forged,
      await sign(readClaims({ exp: NOW - 1 })),
      await sign(readClaims({ aud: "hone-community-edge-v1" })),
      await sign(readClaims({ v: 2 })),
      await sign(readClaims({ iat: NOW - 7200, exp: NOW + 600 })),
      "not-a-token",
      `${(await sign(readClaims())).split(".")[0]}.`,
    ]) {
      const response = await handleRequest(getRequest(cookie), env());
      expect(response.status).toBe(401);
    }
  });

  it("refuses a duplicated cookie rather than picking a winner", async () => {
    bucket.seed(KEY, PNG, "image/png");
    const cookie = await sign(readClaims());
    const response = await handleRequest(
      getRequestWithCookieHeader(`hone_media_edge=${cookie}; hone_media_edge=${cookie}`),
      env(),
    );
    expect(response.status).toBe(401);
  });

  it("refuses a write token presented as a read session", async () => {
    bucket.seed(KEY, PNG, "image/png");
    const response = await handleRequest(getRequest(await sign(writeClaims())), env());
    expect(response.status).toBe(401);
  });

  it("refuses to serve another tenant's object", async () => {
    const otherKey = `${UPLOAD_PREFIX}/web-user-2/2026-08-30/0123456789abcdef-secret.png`;
    bucket.seed(otherKey, PNG, "image/png");
    const response = await handleRequest(getRequest(await sign(readClaims()), otherKey), env());
    expect(response.status).toBe(403);
    expect(await response.json()).toEqual({ error: "object_outside_session_scope" });
  });

  it("does not treat a sibling account as a prefix match", async () => {
    // web-user-11 starts with web-user-1. The trailing slash on the signed
    // prefix is what keeps that from reading as ownership.
    const siblingKey = `${UPLOAD_PREFIX}/web-user-11/2026-08-30/0123456789abcdef-x.png`;
    bucket.seed(siblingKey, PNG, "image/png");
    const response = await handleRequest(
      getRequest(await sign(readClaims()), siblingKey),
      env(),
    );
    expect(response.status).toBe(403);
  });

  it("answers 403 for another tenant's key whether or not it exists", async () => {
    // The ownership check runs before R2 is touched, so response codes cannot
    // be used to probe which keys exist outside the caller's own prefix.
    const cookie = await sign(readClaims());
    const present = `${UPLOAD_PREFIX}/web-user-2/2026-08-30/0123456789abcdef-a.png`;
    const absent = `${UPLOAD_PREFIX}/web-user-2/2026-08-30/0123456789abcdef-b.png`;
    bucket.seed(present, PNG, "image/png");
    expect((await handleRequest(getRequest(cookie, present), env())).status).toBe(403);
    expect((await handleRequest(getRequest(cookie, absent), env())).status).toBe(403);
  });

  it("refuses a session whose signed prefix is over-broad", async () => {
    bucket.seed(KEY, PNG, "image/png");
    for (const pfx of [UPLOAD_PREFIX, `${UPLOAD_PREFIX}/`, "", `${UPLOAD_PREFIX}/../`, "/"]) {
      const response = await handleRequest(getRequest(await sign(readClaims({ pfx }))), env());
      expect(response.status).toBe(401);
    }
  });

  it("refuses to serve a stored object whose type is outside the allowlist", async () => {
    bucket.seed(KEY, SVG, "image/svg+xml");
    const response = await handleRequest(getRequest(await sign(readClaims())), env());
    expect(response.status).toBe(502);
    expect(await response.json()).toEqual({ error: "object_content_type_rejected" });
  });

  it("serves an owned object inline and answers HEAD without a body", async () => {
    bucket.seed(KEY, PNG, "image/png");
    const cookie = await sign(readClaims());

    const response = await handleRequest(getRequest(cookie), env());
    expect(response.status).toBe(200);
    expect(response.headers.get("Content-Type")).toBe("image/png");
    expect(response.headers.get("Content-Length")).toBe(String(PNG.byteLength));
    expect(new Uint8Array(await response.arrayBuffer())).toEqual(PNG);

    const head = await handleRequest(getRequest(cookie, KEY, "HEAD"), env());
    expect(head.status).toBe(200);
    expect(head.headers.get("Content-Length")).toBe(String(PNG.byteLength));
    expect(await head.text()).toBe("");
  });

  it("reports a missing object as 404 and an unreachable store as 503", async () => {
    const cookie = await sign(readClaims());
    expect((await handleRequest(getRequest(cookie), env())).status).toBe(404);

    bucket.seed(KEY, PNG, "image/png");
    bucket.failingKeys.add(KEY);
    expect((await handleRequest(getRequest(cookie), env())).status).toBe(503);
  });
});

describe("upload authorization", () => {
  it("requires a token in the custom header", async () => {
    const request = new Request(`https://hone-claw.com/_media/v1/o/${KEY}`, {
      method: "PUT",
      headers: { "Content-Length": String(PNG.byteLength), Origin: "https://hone-claw.com" },
      body: Uint8Array.from(PNG),
    });
    const response = await handleRequest(request, env());
    expect(response.status).toBe(401);
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("rejects forged, expired, over-long, and read-scoped tokens", async () => {
    for (const token of [
      await sign(writeClaims(), "a-different-secret-of-32-bytes!!!"),
      await sign(writeClaims({ exp: NOW - 1 })),
      await sign(writeClaims({ iat: NOW, exp: NOW + 3600 })),
      await sign(readClaims()),
      await sign(writeClaims({ op: "PUT" })),
    ]) {
      const response = await handleRequest(putRequest(token, PNG), env());
      expect(response.status).toBe(401);
    }
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("refuses a token whose key does not match the request path", async () => {
    const token = await sign(writeClaims());
    const response = await handleRequest(
      putRequest(token, PNG, {
        key: `${UPLOAD_PREFIX}/${OWNER}/2026-08-30/0123456789abcdef-other.png`,
      }),
      env(),
    );
    expect(response.status).toBe(403);
    expect(await response.json()).toEqual({ error: "upload_token_key_mismatch" });
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("refuses a signed token whose key escapes its own owner prefix", async () => {
    // Models a grant bug on the origin: authenticated as web-user-1, key minted
    // under web-user-2. The signature is valid, so only the edge's independent
    // prefix check stands between that bug and cross-tenant writes.
    const key = `${UPLOAD_PREFIX}/web-user-2/2026-08-30/0123456789abcdef-x.png`;
    const token = await sign(writeClaims({ key, pfx: OWNER_PREFIX }));
    const response = await handleRequest(putRequest(token, PNG, { key }), env());
    expect(response.status).toBe(401);
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("refuses a write token whose prefix claim is over-broad", async () => {
    for (const pfx of [UPLOAD_PREFIX, `${UPLOAD_PREFIX}/`, "", "/"]) {
      const token = await sign(writeClaims({ pfx }));
      const response = await handleRequest(putRequest(token, PNG), env());
      expect(response.status).toBe(401);
    }
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("refuses a content type outside the allowlist, SVG included", async () => {
    for (const ct of ["image/svg+xml", "text/html", "application/pdf", "image/bmp"]) {
      const response = await handleRequest(await mintAndPut(ct, PNG), env());
      expect(response.status).toBe(401);
    }
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("refuses bytes that do not match the signed content type", async () => {
    const token = await sign(writeClaims({ ct: "image/png" }));
    const response = await handleRequest(putRequest(token, JPEG), env());
    expect(response.status).toBe(415);
    expect(await response.json()).toEqual({ error: "content_type_mismatch" });
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("refuses markup smuggled under an image token", async () => {
    for (const bytes of [SVG, HTML]) {
      const token = await sign(writeClaims());
      const response = await handleRequest(putRequest(token, bytes), env());
      expect(response.status).toBe(415);
      expect(await response.json()).toEqual({ error: "unsupported_image_format" });
    }
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("requires a declared length within the token's cap", async () => {
    const token = await sign(writeClaims({ max: 8 }));
    const response = await handleRequest(putRequest(token, PNG), env());
    expect(response.status).toBe(413);

    const missing = new Request(`https://hone-claw.com/_media/v1/o/${KEY}`, {
      method: "PUT",
      headers: { "X-Hone-Media-Token": await sign(writeClaims()) },
      body: Uint8Array.from(PNG),
    });
    // Undici sets Content-Length itself, so force the header away to model a
    // chunked upload.
    missing.headers.delete("Content-Length");
    expect((await handleRequest(missing, env())).status).toBe(411);
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("refuses a body that does not match its declared length", async () => {
    const token = await sign(writeClaims());
    const response = await handleRequest(
      putRequest(token, PNG, { contentLength: PNG.byteLength + 1 }),
      env(),
    );
    expect(response.status).toBe(400);
    expect(await response.json()).toEqual({ error: "content_length_mismatch" });
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("stops reading a body that overruns the cap it declared", async () => {
    const token = await sign(writeClaims({ max: 16 }));
    const oversized = new Uint8Array(64);
    oversized.set(PNG);
    const request = new Request(`https://hone-claw.com/_media/v1/o/${KEY}`, {
      method: "PUT",
      headers: {
        "X-Hone-Media-Token": token,
        "Content-Length": "16",
        Origin: "https://hone-claw.com",
      },
      body: oversized,
    });
    request.headers.set("Content-Length", "16");
    const response = await handleRequest(request, env());
    expect(response.status).toBe(413);
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("refuses to overwrite an object that already exists", async () => {
    bucket.seed(KEY, PNG, "image/png");
    const token = await sign(writeClaims());
    const response = await handleRequest(putRequest(token, PNG), env());
    expect(response.status).toBe(409);
    expect(await response.json()).toEqual({ error: "object_already_exists" });
    expect(bucket.putCalls).toHaveLength(0);
  });

  it("makes a replayed token useless once its object exists", async () => {
    const token = await sign(writeClaims());
    expect((await handleRequest(putRequest(token, PNG), env())).status).toBe(201);
    expect((await handleRequest(putRequest(token, PNG), env())).status).toBe(409);
    expect(bucket.putCalls).toHaveLength(1);
  });

  it("stores the signed content type, not anything the client asserts", async () => {
    const token = await sign(writeClaims({ ct: "image/webp" }));
    const response = await handleRequest(
      putRequest(token, WEBP, { headers: { "Content-Type": "image/svg+xml" } }),
      env(),
    );
    expect(response.status).toBe(201);
    expect(bucket.putCalls).toEqual([
      { key: KEY, contentType: "image/webp", bytes: WEBP.byteLength },
    ]);
  });

  it("round-trips an upload back through the owner's read session", async () => {
    expect((await handleRequest(putRequest(await sign(writeClaims()), PNG), env())).status).toBe(
      201,
    );
    const response = await handleRequest(getRequest(await sign(readClaims())), env());
    expect(response.status).toBe(200);
    expect(response.headers.get("Content-Type")).toBe("image/png");
    expect(new Uint8Array(await response.arrayBuffer())).toEqual(PNG);
  });

  it("reports an unreachable store without claiming success", async () => {
    bucket.failingKeys.add(KEY);
    const response = await handleRequest(putRequest(await sign(writeClaims()), PNG), env());
    expect(response.status).toBe(503);
  });
});

async function mintAndPut(contentType: string, bytes: Uint8Array) {
  return putRequest(await sign(writeClaims({ ct: contentType })), bytes);
}
