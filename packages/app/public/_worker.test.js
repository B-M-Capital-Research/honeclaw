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
