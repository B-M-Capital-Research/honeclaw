import { afterEach, describe, expect, test } from "bun:test";

import {
  ensurePublicMediaEdgeSession,
  publicMediaEdgeObjectPath,
  publicMediaObjectKey,
  resetPublicMediaEdgeStateForTests,
  uploadPublicAttachments,
} from "./api";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  resetPublicMediaEdgeStateForTests();
});

const KEY = "public-uploads/web-user-1/2026-08-30/0123456789abcdef-image.png";
const OSS_PATH = `oss://honeclaw/${KEY}`;

function pngFile(name = "pasted.png", bytes = 64) {
  return new File([new Uint8Array(bytes)], name, { type: "image/png" });
}

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

function activeSessionPayload(overrides: Record<string, unknown> = {}) {
  return {
    enabled: true,
    mode: "prefer",
    base_path: "/_media/v1",
    expires_at: Math.floor(Date.now() / 1000) + 900,
    ...overrides,
  };
}

function grantPayload(files: File[], overrides: Record<string, unknown> = {}) {
  return {
    ...activeSessionPayload(),
    uploads: files.map((file, index) => ({
      upload_path: `/_media/v1/o/public-uploads/web-user-1/2026-08-30/000000000000000${index}-image.png`,
      token: `token-${index}`,
      path: `oss://honeclaw/public-uploads/web-user-1/2026-08-30/000000000000000${index}-image.png`,
      name: "image.png",
      kind: "image",
      content_type: "image/png",
      size: file.size,
    })),
    ...overrides,
  };
}

describe("object key parsing", () => {
  test("accepts a well-formed managed upload URI", () => {
    expect(publicMediaObjectKey(OSS_PATH)).toBe(KEY);
  });

  test("rejects anything the edge Worker would also reject", () => {
    for (const path of [
      "/srv/honeclaw/sessions/public-uploads/web-user-1/x.png",
      "file:///etc/passwd",
      "oss://honeclaw/public-uploads/web-user-1/../../etc/passwd",
      "oss://honeclaw/public-uploads/web-user-1/2026-08-30/%2e%2e%2fsecret.png",
      "oss://honeclaw/public-uploads/web-user-1/2026-08-30/",
      "oss://honeclaw/public-uploads/web-user-1/x.png",
      "oss://honeclaw/public-uploads/web-user-1/2026-08-30/nested/x.png",
      "oss://honeclaw/",
      "oss://",
      "",
    ]) {
      expect(publicMediaObjectKey(path)).toBeNull();
    }
  });
});

describe("read path selection", () => {
  test("stays on the origin proxy until a session is established", () => {
    expect(publicMediaEdgeObjectPath(OSS_PATH)).toBeNull();
  });

  test("uses the edge once the session is active", async () => {
    globalThis.fetch = (async () => jsonResponse(activeSessionPayload())) as unknown as typeof fetch;
    await ensurePublicMediaEdgeSession();
    expect(publicMediaEdgeObjectPath(OSS_PATH)).toBe(`/_media/v1/o/${KEY}`);
  });

  test("ignores a session the server declined or left in shadow mode", async () => {
    for (const payload of [
      activeSessionPayload({ enabled: false }),
      activeSessionPayload({ mode: "shadow" }),
      activeSessionPayload({ mode: "off" }),
      activeSessionPayload({ base_path: "/_community/v1" }),
      activeSessionPayload({ expires_at: Math.floor(Date.now() / 1000) - 1 }),
      activeSessionPayload({ expires_at: null }),
    ]) {
      resetPublicMediaEdgeStateForTests();
      globalThis.fetch = (async () => jsonResponse(payload)) as unknown as typeof fetch;
      await ensurePublicMediaEdgeSession();
      expect(publicMediaEdgeObjectPath(OSS_PATH)).toBeNull();
    }
  });

  test("never points at an object outside the managed key shape", async () => {
    globalThis.fetch = (async () => jsonResponse(activeSessionPayload())) as unknown as typeof fetch;
    await ensurePublicMediaEdgeSession();
    expect(publicMediaEdgeObjectPath("oss://honeclaw/other/web-user-2/../x.png")).toBeNull();
    expect(publicMediaEdgeObjectPath("/srv/honeclaw/sessions/x.png")).toBeNull();
  });
});

describe("upload routing", () => {
  test("PUTs each file to the edge and reports the signed oss paths", async () => {
    const files = [pngFile("a.png"), pngFile("b.png", 128)];
    const calls: Array<{ url: string; method?: string; token?: string; credentials?: string }> = [];
    globalThis.fetch = (async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = typeof input === "string" ? input : input.toString();
      const headers = new Headers(init?.headers);
      calls.push({
        url,
        method: init?.method,
        token: headers.get("X-Hone-Media-Token") ?? undefined,
        credentials: init?.credentials,
      });
      if (url.includes("/api/public/media/upload-grant")) {
        return jsonResponse(grantPayload(files));
      }
      return new Response(JSON.stringify({ ok: true }), { status: 201 });
    }) as typeof fetch;

    const uploaded = await uploadPublicAttachments(files);
    expect(uploaded.map((item) => item.path)).toEqual([
      "oss://honeclaw/public-uploads/web-user-1/2026-08-30/0000000000000000-image.png",
      "oss://honeclaw/public-uploads/web-user-1/2026-08-30/0000000000000001-image.png",
    ]);
    expect(calls.filter((call) => call.method === "PUT")).toHaveLength(2);
    expect(calls.filter((call) => call.method === "PUT").map((call) => call.token)).toEqual([
      "token-0",
      "token-1",
    ]);
    // The capability is the only thing that authorizes the write; sending the
    // session cookie alongside it would widen what a PUT can reach.
    for (const call of calls.filter((entry) => entry.method === "PUT")) {
      expect(call.credentials).toBe("omit");
    }
    expect(calls.some((call) => call.url.includes("/api/public/upload"))).toBe(false);
  });

  test("treats the Pages SPA fallback as a failed upload", async () => {
    // With the Worker route missing, Cloudflare Pages answers /_media/v1/* with
    // index.html and a 200. Accepting that would record an oss:// path for bytes
    // that were never written.
    const files = [pngFile()];
    let originUploads = 0;
    globalThis.fetch = (async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = typeof input === "string" ? input : input.toString();
      if (url.includes("/api/public/media/upload-grant")) return jsonResponse(grantPayload(files));
      if (init?.method === "PUT") {
        return new Response("<!doctype html><div id=root></div>", {
          status: 200,
          headers: { "Content-Type": "text/html" },
        });
      }
      originUploads += 1;
      return jsonResponse({ attachments: [{ path: "/local/x.png", name: "x.png", kind: "image", size: 1 }] });
    }) as typeof fetch;

    const uploaded = await uploadPublicAttachments(files);
    expect(originUploads).toBe(1);
    expect(uploaded[0]?.path).toBe("/local/x.png");
  });

  test("falls back to the origin upload when the edge is off", async () => {
    const files = [pngFile()];
    let originUploads = 0;
    globalThis.fetch = (async (input: RequestInfo | URL) => {
      const url = typeof input === "string" ? input : input.toString();
      if (url.includes("/api/public/media/upload-grant")) {
        return jsonResponse({ enabled: false, mode: "off", base_path: "/_media/v1", uploads: [] });
      }
      originUploads += 1;
      return jsonResponse({ attachments: [{ path: "/local/x.png", name: "x.png", kind: "image", size: 1 }] });
    }) as typeof fetch;

    const uploaded = await uploadPublicAttachments(files);
    expect(originUploads).toBe(1);
    expect(uploaded[0]?.path).toBe("/local/x.png");
  });

  test("falls back when a single edge PUT fails, without half-committing", async () => {
    const files = [pngFile("a.png"), pngFile("b.png", 128)];
    let originUploads = 0;
    let put = 0;
    globalThis.fetch = (async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = typeof input === "string" ? input : input.toString();
      if (url.includes("/api/public/media/upload-grant")) return jsonResponse(grantPayload(files));
      if (init?.method === "PUT") {
        put += 1;
        return put === 1
          ? new Response(JSON.stringify({ ok: true, bytes: 64 }), { status: 201 })
          : new Response(JSON.stringify({ error: "object_already_exists" }), { status: 409 });
      }
      originUploads += 1;
      return jsonResponse({ attachments: [{ path: "/local/x.png", name: "x.png", kind: "image", size: 1 }] });
    }) as typeof fetch;

    const uploaded = await uploadPublicAttachments(files);
    expect(originUploads).toBe(1);
    expect(uploaded.every((item) => item.path === "/local/x.png")).toBe(true);
  });

  test("sends non-image and mixed batches straight to the origin", async () => {
    const calls: string[] = [];
    globalThis.fetch = (async (input: RequestInfo | URL) => {
      const url = typeof input === "string" ? input : input.toString();
      calls.push(url);
      return jsonResponse({ attachments: [] });
    }) as typeof fetch;

    await uploadPublicAttachments([
      new File([new Uint8Array(4)], "notes.pdf", { type: "application/pdf" }),
    ]);
    await uploadPublicAttachments([
      pngFile(),
      new File([new Uint8Array(4)], "notes.pdf", { type: "application/pdf" }),
    ]);
    await uploadPublicAttachments([
      new File([new Uint8Array(4)], "logo.svg", { type: "image/svg+xml" }),
    ]);

    expect(calls.every((url) => url.includes("/api/public/upload"))).toBe(true);
    expect(calls.some((url) => url.includes("upload-grant"))).toBe(false);
  });

  test("refuses a grant that does not line up with what was requested", async () => {
    const files = [pngFile("a.png"), pngFile("b.png", 128)];
    for (const payload of [
      grantPayload(files.slice(0, 1)),
      grantPayload(files, { base_path: "/_community/v1" }),
      {
        ...grantPayload(files),
        uploads: grantPayload(files).uploads.map((upload) => ({
          ...upload,
          upload_path: "/api/public/upload",
        })),
      },
      {
        ...grantPayload(files),
        uploads: grantPayload(files).uploads.map((upload) => ({
          ...upload,
          path: "oss://honeclaw/public-uploads/web-user-1/../../evil.png",
        })),
      },
      {
        ...grantPayload(files),
        uploads: grantPayload(files).uploads.map((upload) => ({ ...upload, size: 1 })),
      },
    ]) {
      let originUploads = 0;
      globalThis.fetch = (async (input: RequestInfo | URL) => {
        const url = typeof input === "string" ? input : input.toString();
        if (url.includes("/api/public/media/upload-grant")) return jsonResponse(payload);
        originUploads += 1;
        return jsonResponse({ attachments: [] });
      }) as typeof fetch;

      await uploadPublicAttachments(files);
      expect(originUploads).toBe(1);
      resetPublicMediaEdgeStateForTests();
    }
  });
});
