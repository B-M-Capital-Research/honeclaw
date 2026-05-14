const STATIC_RESOURCE_PATH =
  /(?:^\/assets\/|(?:\.(?:avif|css|gif|ico|jpe?g|js|json|map|mjs|otf|pdf|png|svg|ttf|txt|wasm|webmanifest|webp|woff2?|xml)$))/i;

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

    if (response.status !== 404) {
      return response;
    }

    if (
      (request.method === "GET" || request.method === "HEAD") &&
      requestWantsHtml(request)
    ) {
      return env.ASSETS.fetch(cloneAsIndexRequest(request, url));
    }

    return response;
  },
};
