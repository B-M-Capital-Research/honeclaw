const STATIC_RESOURCE_PATH =
  /(?:^\/assets\/|(?:\.(?:avif|css|gif|ico|jpe?g|js|json|map|mjs|otf|pdf|png|svg|ttf|txt|wasm|webmanifest|webp|woff2?|xml)$))/i;

const BLOG_META = {
  "/blog/why-hone-uses-rust": {
    title: "HONE 为什么采用 Rust，以及推荐大家都开始使用 Rust",
    description:
      "从 Python + Node.js 到 Rust 的完整重构复盘：为什么 Rust 更适合 AI Coding 时代的上下文治理、稳定性和多端工程。",
    image: "https://hone-claw.com/blog/why-hone-uses-rust-zh.png",
    url: "https://hone-claw.com/blog/why-hone-uses-rust",
  },
};

const STRICT_TRANSPORT_SECURITY = "max-age=31536000";

function isStaticResourceRequest(pathname) {
  return STATIC_RESOURCE_PATH.test(pathname);
}

function responseLooksLikeHtml(response) {
  return /\btext\/html\b/i.test(response.headers.get("content-type") || "");
}

function requestWantsHtml(request) {
  const fetchMode = request.headers.get("sec-fetch-mode");
  if (fetchMode === "navigate") return true;
  return (request.headers.get("accept") || "").includes("text/html");
}

const TRADE_MANAGER_ENTRIES = new Set(['/trade-manager/', '/trade-manager/index.html']);
const TRADE_MANAGER_CSP = "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'self'";
const HONE_ADVENTURE_ENTRIES = new Set(['/hone-adventure/', '/hone-adventure/index.html']);

export function withSecurityHeaders(response, pathname = '') {
  const headers = new Headers(response.headers);
  headers.set("strict-transport-security", STRICT_TRANSPORT_SECURITY);
  headers.set("x-content-type-options", "nosniff");
  headers.set("referrer-policy", "strict-origin-when-cross-origin");
  if (responseLooksLikeHtml(response)) {
    const tradeManager = TRADE_MANAGER_ENTRIES.has(pathname);
    headers.set("content-security-policy", tradeManager ? TRADE_MANAGER_CSP : "frame-ancestors 'none'");
    headers.set("x-frame-options", tradeManager ? "SAMEORIGIN" : "DENY");
    if (tradeManager) headers.set("cache-control", "no-store");
  }
  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers,
  });
}

function cloneAsIndexRequest(request, url) {
  const indexUrl = new URL("/index.html", url);
  return new Request(indexUrl, request);
}

function blogMetaForPath(pathname) {
  return BLOG_META[pathname.replace(/\/$/, "")];
}

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function articleMetaTags(meta) {
  const title = escapeHtml(`${meta.title} | HONE Blog`);
  const description = escapeHtml(meta.description);
  const url = escapeHtml(meta.url);
  const image = escapeHtml(meta.image);

  return [
    `<title>${title}</title>`,
    `<meta name="description" content="${description}" />`,
    `<meta property="og:type" content="article" />`,
    `<meta property="og:site_name" content="HONE" />`,
    `<meta property="og:title" content="${title}" />`,
    `<meta property="og:description" content="${description}" />`,
    `<meta property="og:url" content="${url}" />`,
    `<meta property="og:image" content="${image}" />`,
    `<meta property="og:image:width" content="1491" />`,
    `<meta property="og:image:height" content="1055" />`,
    `<meta property="og:image:type" content="image/png" />`,
    `<meta name="twitter:card" content="summary_large_image" />`,
    `<meta name="twitter:title" content="${title}" />`,
    `<meta name="twitter:description" content="${description}" />`,
    `<meta name="twitter:image" content="${image}" />`,
  ].join("\n    ");
}

async function injectArticleMeta(response, meta) {
  const html = await response.text();
  const withoutDefaultMeta = html
    .replace(/<title>[\s\S]*?<\/title>/i, "")
    .replace(/\s*<meta\s+name="description"[^>]*>\s*/i, "\n")
    .replace(/\s*<meta\s+property="og:[^"]+"[^>]*>\s*/gi, "\n")
    .replace(/\s*<meta\s+name="twitter:[^"]+"[^>]*>\s*/gi, "\n");
  const nextHtml = withoutDefaultMeta.replace(
    /<head>/i,
    `<head>\n    ${articleMetaTags(meta)}`,
  );
  const headers = new Headers(response.headers);
  headers.set("content-type", "text/html; charset=UTF-8");
  headers.set("cache-control", "public, max-age=300");
  return new Response(nextHtml, {
    status: response.status,
    statusText: response.statusText,
    headers,
  });
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const response = await env.ASSETS.fetch(request);

    // A missing tool bundle must never embed the /chat SPA recursively.
    if (url.pathname.startsWith('/trade-manager/')) {
      const entry = TRADE_MANAGER_ENTRIES.has(url.pathname);
      if (response.status === 404 || (!entry && responseLooksLikeHtml(response))) {
        return withSecurityHeaders(new Response('Trade manager asset unavailable', {
          status: 404, headers: { 'content-type': 'text/plain; charset=utf-8', 'cache-control': 'no-store' },
        }));
      }
      if (entry && request.method !== 'HEAD' && responseLooksLikeHtml(response)) {
        const html = await response.clone().text();
        if (!html.includes('<meta name="hone-tool" content="trade-manager">')) {
          return withSecurityHeaders(new Response('Trade manager bundle not installed', {
            status: 503, headers: { 'content-type': 'text/plain; charset=utf-8', 'cache-control': 'no-store' },
          }));
        }
      }
      return withSecurityHeaders(response, url.pathname);
    }

    if (url.pathname.startsWith('/hone-adventure/')) {
      const entry = HONE_ADVENTURE_ENTRIES.has(url.pathname);
      if (response.status === 404 || (!entry && responseLooksLikeHtml(response))) {
        return withSecurityHeaders(new Response('HONE Adventure asset unavailable', {
          status: 404, headers: { 'content-type': 'text/plain; charset=utf-8', 'cache-control': 'no-store' },
        }));
      }
      if (entry && request.method !== 'HEAD' && responseLooksLikeHtml(response)) {
        const html = await response.clone().text();
        if (!/<meta\s+name="hone-game"\s+content="hone-adventure"\s*\/?>/i.test(html)) {
          return withSecurityHeaders(new Response('HONE Adventure bundle not installed', {
            status: 503, headers: { 'content-type': 'text/plain; charset=utf-8', 'cache-control': 'no-store' },
          }));
        }
      }
      return withSecurityHeaders(response);
    }

    if (isStaticResourceRequest(url.pathname)) {
      if (response.status === 404 || responseLooksLikeHtml(response)) {
        return withSecurityHeaders(
          new Response("", {
            status: 404,
            headers: {
              "cache-control": "no-store",
              "x-content-type-options": "nosniff",
              "x-robots-tag": "noindex",
            },
          }),
        );
      }
      return withSecurityHeaders(response);
    }

    const blogMeta = blogMetaForPath(url.pathname);
    if (
      blogMeta &&
      request.method === "GET" &&
      requestWantsHtml(request) &&
      response.status !== 404 &&
      responseLooksLikeHtml(response)
    ) {
      return withSecurityHeaders(await injectArticleMeta(response, blogMeta));
    }

    if (response.status !== 404) {
      return withSecurityHeaders(response);
    }

    if (
      (request.method === "GET" || request.method === "HEAD") &&
      requestWantsHtml(request)
    ) {
      const indexResponse = await env.ASSETS.fetch(cloneAsIndexRequest(request, url));
      if (blogMeta && request.method === "GET") {
        return withSecurityHeaders(await injectArticleMeta(indexResponse, blogMeta));
      }
      return withSecurityHeaders(indexResponse);
    }

    return withSecurityHeaders(response);
  },
};
