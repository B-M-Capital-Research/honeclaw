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

    if (isStaticResourceRequest(url.pathname)) {
      if (response.status === 404 || responseLooksLikeHtml(response)) {
        return new Response("", {
          status: 404,
          headers: {
            "cache-control": "no-store",
            "x-content-type-options": "nosniff",
            "x-robots-tag": "noindex",
          },
        });
      }
      return response;
    }

    const blogMeta = blogMetaForPath(url.pathname);
    if (
      blogMeta &&
      request.method === "GET" &&
      requestWantsHtml(request) &&
      response.status !== 404 &&
      responseLooksLikeHtml(response)
    ) {
      return injectArticleMeta(response, blogMeta);
    }

    if (response.status !== 404) {
      return response;
    }

    if (
      (request.method === "GET" || request.method === "HEAD") &&
      requestWantsHtml(request)
    ) {
      const indexResponse = await env.ASSETS.fetch(cloneAsIndexRequest(request, url));
      if (blogMeta && request.method === "GET") {
        return injectArticleMeta(indexResponse, blogMeta);
      }
      return indexResponse;
    }

    return response;
  },
};
