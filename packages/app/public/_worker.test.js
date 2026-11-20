import { describe, expect, test } from "bun:test";
import worker, { withSecurityHeaders } from "./_worker.js";

describe("Pages worker security headers", () => {
  test("prevents framing and enables HSTS on HTML responses", async () => {
    const response = withSecurityHeaders(
      new Response("<!doctype html>", {
        headers: { "content-type": "text/html; charset=utf-8" },
      }),
    );

    expect(response.headers.get("content-security-policy")).toBe(
      "frame-ancestors 'none'",
    );
    expect(response.headers.get("x-frame-options")).toBe("DENY");
    expect(response.headers.get("strict-transport-security")).toBe(
      "max-age=31536000",
    );
  });

  test("applies transport headers to static responses without an HTML CSP", () => {
    const response = withSecurityHeaders(
      new Response("body {}", {
        headers: { "content-type": "text/css" },
      }),
    );

    expect(response.headers.get("strict-transport-security")).toBe(
      "max-age=31536000",
    );
    expect(response.headers.get("content-security-policy")).toBeNull();
    expect(response.headers.get("x-frame-options")).toBeNull();
  });

  test("covers SPA fallback responses", async () => {
    const env = {
      ASSETS: {
        async fetch(request) {
          const path = new URL(request.url).pathname;
          if (path === "/index.html") {
            return new Response("<!doctype html>", {
              headers: { "content-type": "text/html; charset=utf-8" },
            });
          }
          return new Response("missing", { status: 404 });
        },
      },
    };

    const response = await worker.fetch(
      new Request("https://hone-claw.com/chat", {
        headers: { accept: "text/html" },
      }),
      env,
    );

    expect(response.status).toBe(200);
    expect(response.headers.get("content-security-policy")).toBe(
      "frame-ancestors 'none'",
    );
    expect(response.headers.get("x-frame-options")).toBe("DENY");
  });
});


describe("trade manager scoped embedding", () => {
  test("only the fixed tool entry allows same-origin framing", () => {
    const html = () => new Response("<!doctype html>", {headers: {"content-type": "text/html"}});
    const result = withSecurityHeaders(html(), "/trade-manager/index.html");
    expect(result.headers.get("x-frame-options")).toBe("SAMEORIGIN");
    expect(result.headers.get("content-security-policy")).toContain("frame-ancestors 'self'");
    expect(result.headers.get("content-security-policy")).toContain("connect-src 'self'");
    expect(result.headers.get("content-security-policy")).toContain("script-src 'self'");
    expect(result.headers.get("cache-control")).toBe("no-store");
    for (const path of ["/chat", "/trade-manager/not-the-tool.html", "/trade-manager-evil/index.html"]) {
      expect(withSecurityHeaders(html(), path).headers.get("x-frame-options")).toBe("DENY");
    }
  });
  test("serves the tool without relaxing other documents and refuses SPA asset fallback", async () => {
    let calls = 0;
    const env = {ASSETS: {fetch: async () => {calls++; return new Response("missing", {status:404});}}};
    const missing = await worker.fetch(new Request("https://hone-claw.com/trade-manager/index.html", {headers:{accept:"text/html"}}), env);
    expect(missing.status).toBe(404);
    expect(calls).toBe(1);
    const spa = {ASSETS:{fetch:async()=>new Response("<!doctype html>", {headers:{"content-type":"text/html"}})}};
    const missingJs = await worker.fetch(new Request("https://hone-claw.com/trade-manager/missing.js"), spa);
    expect(missingJs.status).toBe(404);
    const rootFallback = await worker.fetch(new Request("https://hone-claw.com/trade-manager/index.html"), spa);
    expect(rootFallback.status).toBe(503);
    const tool = {ASSETS:{fetch:async()=>new Response('<!doctype html><meta name="hone-tool" content="trade-manager">', {headers:{"content-type":"text/html"}})}};
    const found = await worker.fetch(new Request("https://hone-claw.com/trade-manager/index.html"), tool);
    expect(found.headers.get("x-frame-options")).toBe("SAMEORIGIN");
  });
});
