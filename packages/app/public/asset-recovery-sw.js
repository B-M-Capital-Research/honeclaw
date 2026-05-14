const ASSET_PATH = /^\/assets\//;
const RECOVERY_MESSAGE = { type: "hone:asset-recovery:reload" };

self.addEventListener("install", (event) => {
  event.waitUntil(self.skipWaiting());
});

self.addEventListener("activate", (event) => {
  event.waitUntil(self.clients.claim());
});

self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);
  if (url.origin !== self.location.origin || !ASSET_PATH.test(url.pathname)) {
    return;
  }

  event.respondWith(fetchAssetOrRecover(event.request));
});

async function fetchAssetOrRecover(request) {
  const response = await fetch(request);
  if (response.status !== 404 && !responseLooksLikeHtml(response)) {
    return response;
  }

  await notifyWindows();
  return new Response("", {
    status: 404,
    headers: {
      "cache-control": "no-store",
      "x-content-type-options": "nosniff",
    },
  });
}

function responseLooksLikeHtml(response) {
  return /\btext\/html\b/i.test(response.headers.get("content-type") || "");
}

async function notifyWindows() {
  const windows = await self.clients.matchAll({
    type: "window",
    includeUncontrolled: true,
  });
  for (const client of windows) {
    client.postMessage(RECOVERY_MESSAGE);
  }
}
