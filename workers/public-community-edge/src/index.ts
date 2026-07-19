const EDGE_COOKIE_NAME = "hone_community_edge";
const WEB_SESSION_COOKIE_NAME = "hone_web_session";
const EDGE_TOKEN_AUDIENCE = "hone-community-edge-v1";
const EDGE_TOKEN_VERSION = 1;
const TOKEN_CLOCK_SKEW_SECONDS = 60;
const MAX_TOKEN_LIFETIME_SECONDS = 3600;
const MAX_SUBJECT_LENGTH = 512;
const MIN_SECRET_BYTES = 32;
const MAX_SECRET_BYTES = 1024;
const MAX_FEED_BYTES = 8 * 1024 * 1024;
const MAX_DESCRIPTOR_BYTES = 64 * 1024;
const MAX_ACTIVE_INDEX_BYTES = 1024 * 1024;
const MAX_RESOURCE_BYTES = 128 * 1024 * 1024;

const DEFAULT_FEED_PREFIX = "community/zsxq/51115212285814/delivery/v1";
const DEFAULT_RESOURCE_PREFIX = `${DEFAULT_FEED_PREFIX}/resources`;
const DEFAULT_ASSET_PREFIX = "community/zsxq/51115212285814/resources";
const DEFAULT_LEGACY_ORIGIN = "https://origin.hone-claw.com";

const FEED_CACHE_CONTROL = "private, max-age=30, stale-while-revalidate=30";
const RESOURCE_CACHE_CONTROL = "private, no-cache";
const NO_STORE_CACHE_CONTROL = "private, no-store";
const CACHE_MARKER_HEADER = "X-Hone-Community-Cache";
const CACHE_MARKER_VALUE = "r2-v2";

interface R2ObjectMetadataLike {
  readonly size: number;
  readonly httpEtag?: string;
}

interface R2ObjectBodyLike extends R2ObjectMetadataLike {
  readonly body: ReadableStream<Uint8Array> | null;
  text(): Promise<string>;
}

export interface CommunityBucket {
  get(key: string): Promise<R2ObjectBodyLike | null>;
  head(key: string): Promise<R2ObjectMetadataLike | null>;
}

export interface Env {
  COMMUNITY_BUCKET?: CommunityBucket;
  COMMUNITY_EDGE_HMAC_SECRET?: string;
  EDGE_DISABLED?: string;
  COMMUNITY_FEED_PREFIX?: string;
  COMMUNITY_RESOURCE_PREFIX?: string;
  COMMUNITY_ASSET_PREFIX?: string;
  LEGACY_ORIGIN_URL?: string;
}

export interface EdgeCache {
  match(request: Request): Promise<Response | undefined>;
  put(request: Request, response: Response): Promise<void>;
}

export interface EdgeExecutionContext {
  waitUntil(promise: Promise<unknown>): void;
}

interface EdgeTokenPayload {
  v: number;
  aud: string;
  sub: string;
  iat: number;
  exp: number;
}

interface ResourceDescriptor {
  resource_id: number;
  version: string;
  sha256: string;
  object_key: string;
  content_type: string;
  byte_size: number;
  display_name?: string | null;
}

interface ResourceActiveIndex {
  v: number;
  resources: Record<string, string>;
}

type EdgeRoute =
  | { kind: "feed-latest" }
  | { kind: "feed-page"; pageId: number }
  | { kind: "resource"; resourceId: number; version: string };

type DescriptorLoad =
  | { kind: "ok"; descriptor: ResourceDescriptor }
  | { kind: "unavailable" }
  | { kind: "invalid" };

type ActiveResourceState = "active" | "inactive" | "unavailable" | "invalid";

function edgeEnabled(raw: string | undefined): boolean {
  if (raw === undefined) return false;
  return ["false", "0", "no", "off"].includes(raw.trim().toLowerCase());
}

function securityHeaders(cacheControl: string): Headers {
  const headers = new Headers();
  headers.set("Cache-Control", cacheControl);
  headers.set("Content-Security-Policy", "default-src 'none'; sandbox");
  headers.set("Cross-Origin-Resource-Policy", "same-origin");
  headers.set("Referrer-Policy", "no-referrer");
  headers.set("Vary", "Cookie");
  headers.set("X-Content-Type-Options", "nosniff");
  headers.set("X-Robots-Tag", "noindex, nofollow");
  return headers;
}

function jsonError(status: number, code: string): Response {
  const headers = securityHeaders(NO_STORE_CACHE_CONTROL);
  headers.set("Content-Type", "application/json; charset=utf-8");
  return new Response(JSON.stringify({ error: code }), { status, headers });
}

function serviceUnavailable(): Response {
  return jsonError(503, "community_edge_unavailable");
}

function parseRoute(pathname: string): EdgeRoute | null {
  if (pathname === "/_community/v1/feed/latest.json") {
    return { kind: "feed-latest" };
  }

  const page = /^\/_community\/v1\/feed\/pages\/([1-9][0-9]*)\.json$/.exec(pathname);
  if (page) {
    const pageId = Number(page[1]);
    if (Number.isSafeInteger(pageId)) return { kind: "feed-page", pageId };
  }

  const resource = /^\/_community\/v1\/resources\/([1-9][0-9]*)\/([0-9a-f]{12})$/.exec(
    pathname,
  );
  if (resource) {
    const resourceId = Number(resource[1]);
    if (Number.isSafeInteger(resourceId)) {
      return { kind: "resource", resourceId, version: resource[2] };
    }
  }

  return null;
}

function edgeCookie(cookieHeader: string | null): string | null {
  if (!cookieHeader) return null;
  const matches: string[] = [];
  for (const part of cookieHeader.split(";")) {
    const pair = part.trim();
    const separator = pair.indexOf("=");
    if (separator <= 0) continue;
    if (pair.slice(0, separator).trim() === EDGE_COOKIE_NAME) {
      matches.push(pair.slice(separator + 1).trim());
    }
  }
  return matches.length === 1 && matches[0] !== "" ? matches[0] : null;
}

function legacySessionCookie(cookieHeader: string | null): string | null {
  if (!cookieHeader) return null;
  const matches: string[] = [];
  for (const part of cookieHeader.split(";")) {
    const pair = part.trim();
    const separator = pair.indexOf("=");
    if (separator <= 0 || pair.slice(0, separator).trim() !== WEB_SESSION_COOKIE_NAME) continue;
    const value = pair.slice(separator + 1).trim();
    if (value.length > 0 && value.length <= 1024 && /^[\x21-\x7e]+$/.test(value)) {
      matches.push(value);
    }
  }
  return matches.length === 1 ? `${WEB_SESSION_COOKIE_NAME}=${matches[0]}` : null;
}

function decodeBase64Url(segment: string): Uint8Array | null {
  if (!/^[A-Za-z0-9_-]+$/.test(segment)) return null;
  const remainder = segment.length % 4;
  if (remainder === 1) return null;
  const base64 = segment.replace(/-/g, "+").replace(/_/g, "/") + "=".repeat((4 - remainder) % 4);
  try {
    const binary = atob(base64);
    const decoded = Uint8Array.from(binary, (character) => character.charCodeAt(0));
    return encodeBase64Url(decoded) === segment ? decoded : null;
  } catch {
    return null;
  }
}

function encodeBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function copiedArrayBuffer(bytes: Uint8Array): ArrayBuffer {
  const copy = new Uint8Array(bytes.byteLength);
  copy.set(bytes);
  return copy.buffer;
}

function validSecret(secret: string): boolean {
  const byteLength = new TextEncoder().encode(secret).byteLength;
  return byteLength >= MIN_SECRET_BYTES && byteLength <= MAX_SECRET_BYTES;
}

function validPayload(value: unknown, nowSeconds: number): value is EdgeTokenPayload {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const payload = value as Partial<EdgeTokenPayload>;
  if (payload.v !== EDGE_TOKEN_VERSION || payload.aud !== EDGE_TOKEN_AUDIENCE) return false;
  if (
    typeof payload.sub !== "string" ||
    payload.sub.length === 0 ||
    payload.sub.length > MAX_SUBJECT_LENGTH
  ) {
    return false;
  }
  if (!Number.isSafeInteger(payload.iat) || !Number.isSafeInteger(payload.exp)) return false;
  const issuedAt = payload.iat as number;
  const expiresAt = payload.exp as number;
  if (issuedAt > nowSeconds + TOKEN_CLOCK_SKEW_SECONDS) return false;
  if (expiresAt <= nowSeconds) return false;
  const lifetime = expiresAt - issuedAt;
  return lifetime > 0 && lifetime <= MAX_TOKEN_LIFETIME_SECONDS;
}

async function validEdgeToken(token: string, secret: string): Promise<boolean> {
  const segments = token.split(".");
  if (segments.length !== 2) return false;
  const [payloadSegment, signatureSegment] = segments;
  const payloadBytes = decodeBase64Url(payloadSegment);
  const signature = decodeBase64Url(signatureSegment);
  if (!payloadBytes || !signature || signature.byteLength !== 32) return false;

  let payload: unknown;
  try {
    payload = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(payloadBytes));
  } catch {
    return false;
  }
  if (!validPayload(payload, Math.floor(Date.now() / 1000))) return false;

  try {
    const key = await crypto.subtle.importKey(
      "raw",
      new TextEncoder().encode(secret),
      { name: "HMAC", hash: "SHA-256" },
      false,
      ["verify"],
    );
    return await crypto.subtle.verify(
      "HMAC",
      key,
      copiedArrayBuffer(signature),
      new TextEncoder().encode(payloadSegment),
    );
  } catch {
    return false;
  }
}

function safePrefix(raw: string | undefined, fallback: string): string | null {
  const value = (raw ?? fallback).trim().replace(/\/+$/g, "");
  if (!value || value.startsWith("/")) return null;
  const segments = value.split("/");
  return segments.every(isSafeKeySegment) ? value : null;
}

function isSafeKeySegment(segment: string): boolean {
  return (
    segment !== "" &&
    segment !== "." &&
    segment !== ".." &&
    /^[A-Za-z0-9._-]+$/.test(segment)
  );
}

function safeAssetKey(objectKey: string, assetPrefix: string): boolean {
  if (!objectKey.startsWith(`${assetPrefix}/`)) return false;
  return objectKey.split("/").every(isSafeKeySegment);
}

function parseDescriptor(
  value: unknown,
  resourceId: number,
  version: string,
  assetPrefix: string,
): ResourceDescriptor | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const descriptor = value as Partial<ResourceDescriptor>;
  if (descriptor.resource_id !== resourceId || descriptor.version !== version) return null;
  if (typeof descriptor.sha256 !== "string" || !/^[0-9a-f]{64}$/.test(descriptor.sha256)) {
    return null;
  }
  if (descriptor.sha256.slice(0, 12) !== version) return null;
  if (typeof descriptor.object_key !== "string" || !safeAssetKey(descriptor.object_key, assetPrefix)) {
    return null;
  }
  if (
    typeof descriptor.content_type !== "string" ||
    descriptor.content_type.length === 0 ||
    descriptor.content_type.length > 256 ||
    /[\u0000-\u001f\u007f]/.test(descriptor.content_type)
  ) {
    return null;
  }
  if (
    !Number.isSafeInteger(descriptor.byte_size) ||
    (descriptor.byte_size as number) <= 0 ||
    (descriptor.byte_size as number) > MAX_RESOURCE_BYTES
  ) {
    return null;
  }
  if (
    descriptor.display_name !== undefined &&
    descriptor.display_name !== null &&
    (typeof descriptor.display_name !== "string" ||
      new TextEncoder().encode(descriptor.display_name).byteLength > 1024 ||
      /[\u0000\r\n]/.test(descriptor.display_name))
  ) {
    return null;
  }
  return descriptor as ResourceDescriptor;
}

function parseActiveIndex(value: unknown): ResourceActiveIndex | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const index = value as Partial<ResourceActiveIndex>;
  if (index.v !== 1 || !index.resources || typeof index.resources !== "object") return null;
  if (Array.isArray(index.resources)) return null;
  for (const [resourceId, version] of Object.entries(index.resources)) {
    if (!/^[1-9][0-9]*$/.test(resourceId) || !Number.isSafeInteger(Number(resourceId))) return null;
    if (typeof version !== "string" || !/^[0-9a-f]{12}$/.test(version)) return null;
  }
  return index as ResourceActiveIndex;
}

async function activeResourceState(
  bucket: CommunityBucket,
  resourcePrefix: string,
  resourceId: number,
  version: string,
): Promise<ActiveResourceState> {
  let object: R2ObjectBodyLike | null;
  try {
    object = await bucket.get(`${resourcePrefix}/active.json`);
  } catch {
    return "unavailable";
  }
  if (!object) return "unavailable";
  if (!Number.isSafeInteger(object.size) || object.size <= 0 || object.size > MAX_ACTIVE_INDEX_BYTES) {
    return "invalid";
  }
  let value: unknown;
  try {
    value = JSON.parse(await object.text());
  } catch {
    return "invalid";
  }
  const index = parseActiveIndex(value);
  if (!index) return "invalid";
  return index.resources[String(resourceId)] === version ? "active" : "inactive";
}

async function loadDescriptor(
  bucket: CommunityBucket,
  descriptorKey: string,
  resourceId: number,
  version: string,
  assetPrefix: string,
): Promise<DescriptorLoad> {
  let object: R2ObjectBodyLike | null;
  try {
    object = await bucket.get(descriptorKey);
  } catch {
    return { kind: "unavailable" };
  }
  if (!object) return { kind: "unavailable" };
  if (!Number.isSafeInteger(object.size) || object.size < 0 || object.size > MAX_DESCRIPTOR_BYTES) {
    return { kind: "invalid" };
  }

  let value: unknown;
  try {
    value = JSON.parse(await object.text());
  } catch {
    return { kind: "invalid" };
  }
  const descriptor = parseDescriptor(value, resourceId, version, assetPrefix);
  return descriptor ? { kind: "ok", descriptor } : { kind: "invalid" };
}

function feedKey(route: Extract<EdgeRoute, { kind: "feed-latest" | "feed-page" }>, prefix: string) {
  return route.kind === "feed-latest"
    ? `${prefix}/feed/latest.json`
    : `${prefix}/feed/pages/${route.pageId}.json`;
}

function responseEtag(raw: string | undefined): string | null {
  return raw && /^"[^"\r\n]+"$/.test(raw) ? raw : null;
}

function requestEtagMatches(request: Request, etag: string): boolean {
  const raw = request.headers.get("If-None-Match");
  if (!raw) return false;
  return raw.split(",").some((candidate) => {
    const normalized = candidate.trim();
    return normalized === "*" || normalized === etag || normalized.replace(/^W\//, "") === etag;
  });
}

function successResponse(
  request: Request,
  body: BodyInit | null,
  headers: Headers,
  etag: string | null,
): Response {
  if (etag) {
    headers.set("ETag", etag);
    if (requestEtagMatches(request, etag)) {
      headers.delete("Content-Length");
      return new Response(null, { status: 304, headers });
    }
  }
  return new Response(request.method === "HEAD" ? null : body, { status: 200, headers });
}

async function serveFeedFromR2(
  request: Request,
  env: Env,
  route: Extract<EdgeRoute, { kind: "feed-latest" | "feed-page" }>,
  feedPrefix: string,
): Promise<Response | null> {
  const bucket = env.COMMUNITY_BUCKET;
  if (!bucket) return null;
  const key = feedKey(route, feedPrefix);

  let object: R2ObjectMetadataLike | R2ObjectBodyLike | null;
  try {
    object = request.method === "HEAD" ? await bucket.head(key) : await bucket.get(key);
  } catch {
    return null;
  }
  if (!object || !Number.isSafeInteger(object.size) || object.size < 0 || object.size > MAX_FEED_BYTES) {
    return null;
  }
  if (request.method !== "HEAD" && (!("body" in object) || object.body === null)) return null;

  const headers = securityHeaders(FEED_CACHE_CONTROL);
  headers.set("Content-Type", "application/json; charset=utf-8");
  headers.set("Content-Length", String(object.size));
  return successResponse(
    request,
    request.method === "HEAD" ? null : (object as R2ObjectBodyLike).body,
    headers,
    responseEtag(object.httpEtag),
  );
}

function inlineContentType(raw: string): string | null {
  const normalized = raw.split(";", 1)[0].trim().toLowerCase();
  switch (normalized) {
    case "image/jpeg":
    case "image/jpg":
      return "image/jpeg";
    case "image/png":
    case "image/webp":
    case "image/gif":
    case "image/avif":
    case "application/pdf":
      return normalized;
    default:
      return null;
  }
}

async function serveResourceFromR2(
  request: Request,
  env: Env,
  route: Extract<EdgeRoute, { kind: "resource" }>,
  resourcePrefix: string,
  assetPrefix: string,
): Promise<Response | null> {
  const bucket = env.COMMUNITY_BUCKET;
  if (!bucket) return null;
  const descriptorKey = `${resourcePrefix}/${route.resourceId}/${route.version}.json`;
  const loaded = await loadDescriptor(
    bucket,
    descriptorKey,
    route.resourceId,
    route.version,
    assetPrefix,
  );
  if (loaded.kind === "unavailable") return null;
  if (loaded.kind === "invalid") return jsonError(502, "invalid_resource_descriptor");

  const { descriptor } = loaded;
  let object: R2ObjectMetadataLike | R2ObjectBodyLike | null;
  try {
    object =
      request.method === "HEAD"
        ? await bucket.head(descriptor.object_key)
        : await bucket.get(descriptor.object_key);
  } catch {
    return null;
  }
  if (!object) return null;
  if (object.size !== descriptor.byte_size) return jsonError(502, "resource_integrity_mismatch");
  if (request.method !== "HEAD" && (!("body" in object) || object.body === null)) return null;

  const contentType = inlineContentType(descriptor.content_type);
  const headers = securityHeaders(RESOURCE_CACHE_CONTROL);
  headers.set("Content-Type", contentType ?? "application/octet-stream");
  headers.set(
    "Content-Disposition",
    contentType ? "inline" : `attachment; filename="community-resource-${route.resourceId}"`,
  );
  headers.set("Content-Length", String(object.size));
  return successResponse(
    request,
    request.method === "HEAD" ? null : (object as R2ObjectBodyLike).body,
    headers,
    `"${descriptor.sha256}"`,
  );
}

function legacyUrl(env: Env, route: EdgeRoute): URL | null {
  let origin: URL;
  try {
    origin = new URL(env.LEGACY_ORIGIN_URL?.trim() || DEFAULT_LEGACY_ORIGIN);
  } catch {
    return null;
  }
  if (
    origin.origin !== DEFAULT_LEGACY_ORIGIN ||
    origin.username !== "" ||
    origin.password !== "" ||
    origin.pathname !== "/" ||
    origin.search !== "" ||
    origin.hash !== ""
  ) {
    return null;
  }

  switch (route.kind) {
    case "feed-latest":
      return new URL("/api/public/community?limit=20", origin);
    case "feed-page":
      return new URL(`/api/public/community?before=${route.pageId}&limit=20`, origin);
    case "resource":
      return new URL(
        `/api/public/community/resources/${route.resourceId}?v=${route.version}`,
        origin,
      );
  }
}

function legacyResponse(request: Request, route: EdgeRoute, upstream: Response): Response {
  const successful = upstream.status >= 200 && upstream.status < 300;
  const rawLength = upstream.headers.get("Content-Length");
  const maxLength = route.kind === "resource" ? MAX_RESOURCE_BYTES : MAX_FEED_BYTES;
  const validatedLength =
    rawLength && /^[0-9]+$/.test(rawLength) ? Number(rawLength) : Number.NaN;
  if (
    successful &&
    (!Number.isSafeInteger(validatedLength) || validatedLength <= 0 || validatedLength > maxLength)
  ) {
    return jsonError(502, "legacy_response_size_invalid");
  }
  const cacheControl = successful
    ? route.kind === "resource"
      ? RESOURCE_CACHE_CONTROL
      : NO_STORE_CACHE_CONTROL
    : NO_STORE_CACHE_CONTROL;
  const headers = securityHeaders(cacheControl);

  if (route.kind === "resource" && successful) {
    const contentType = inlineContentType(upstream.headers.get("Content-Type") ?? "");
    headers.set("Content-Type", contentType ?? "application/octet-stream");
    headers.set(
      "Content-Disposition",
      contentType ? "inline" : `attachment; filename="community-resource-${route.resourceId}"`,
    );
  } else {
    headers.set("Content-Type", "application/json; charset=utf-8");
  }

  for (const name of ["ETag", "Last-Modified"] as const) {
    const value = upstream.headers.get(name);
    if (value && !/[\r\n]/.test(value)) headers.set(name, value);
  }
  if (successful) {
    headers.set("Content-Length", String(validatedLength));
  } else if (rawLength && /^[0-9]+$/.test(rawLength)) {
    headers.set("Content-Length", rawLength);
  }

  const bodyForbidden = request.method === "HEAD" || [204, 205, 304].includes(upstream.status);
  return new Response(bodyForbidden ? null : upstream.body, {
    status: upstream.status,
    statusText: upstream.statusText,
    headers,
  });
}

async function fallbackToLegacy(request: Request, env: Env, route: EdgeRoute): Promise<Response> {
  const url = legacyUrl(env, route);
  if (!url) return jsonError(502, "legacy_origin_unavailable");

  const headers = new Headers();
  // Keep the streamed body and response metadata in the same representation. Workers may
  // otherwise negotiate gzip/br for the subrequest while this compatibility proxy deliberately
  // strips all untrusted origin headers, including Content-Encoding.
  headers.set("Accept-Encoding", "identity");
  const cookie = legacySessionCookie(request.headers.get("Cookie"));
  if (cookie) headers.set("Cookie", cookie);
  const etag = request.headers.get("If-None-Match");
  if (etag) headers.set("If-None-Match", etag);

  let upstream: Response;
  try {
    upstream = await fetch(url, {
      method: request.method,
      headers,
      redirect: "manual",
    });
  } catch {
    return jsonError(502, "legacy_origin_unavailable");
  }
  return legacyResponse(request, route, upstream);
}

function defaultEdgeCache(): EdgeCache | null {
  const cacheStorage = (globalThis as unknown as { caches?: { default?: EdgeCache } }).caches;
  return cacheStorage?.default ?? null;
}

function canonicalCacheRequest(request: Request): Request {
  const pathname = new URL(request.url).pathname;
  return new Request(`https://hone-claw.com${pathname}`, { method: "GET" });
}

function cacheControlForRoute(route: EdgeRoute): string {
  return route.kind === "resource" ? RESOURCE_CACHE_CONTROL : FEED_CACHE_CONTROL;
}

function sharedCacheControlForRoute(route: EdgeRoute): string {
  return route.kind === "resource"
    ? "public, max-age=3600, s-maxage=3600"
    : "public, max-age=30, s-maxage=30";
}

function cachedResponseForBrowser(
  request: Request,
  route: EdgeRoute,
  cached: Response,
): Response | null {
  if (cached.status !== 200 || cached.headers.get(CACHE_MARKER_HEADER) !== CACHE_MARKER_VALUE) {
    return null;
  }

  if (route.kind === "resource") {
    const contentLength = cached.headers.get("Content-Length");
    if (
      !contentLength ||
      !/^[0-9]+$/.test(contentLength) ||
      Number(contentLength) <= 0 ||
      Number(contentLength) > MAX_RESOURCE_BYTES
    ) {
      return null;
    }
  }

  const headers = securityHeaders(cacheControlForRoute(route));
  for (const name of ["Content-Type", "Content-Disposition", "Content-Length", "ETag"] as const) {
    const value = cached.headers.get(name);
    if (value && !/[\r\n]/.test(value)) headers.set(name, value);
  }
  const etag = responseEtag(headers.get("ETag") ?? undefined);
  return successResponse(request, cached.body, headers, etag);
}

function responseForSharedCache(route: EdgeRoute, response: Response): Response {
  const clone = response.clone();
  const headers = new Headers(clone.headers);
  headers.set("Cache-Control", sharedCacheControlForRoute(route));
  headers.set(CACHE_MARKER_HEADER, CACHE_MARKER_VALUE);
  headers.delete("Vary");
  return new Response(clone.body, { status: 200, headers });
}

async function readEdgeCache(
  cache: EdgeCache,
  request: Request,
  route: EdgeRoute,
): Promise<Response | null> {
  try {
    const cached = await cache.match(canonicalCacheRequest(request));
    return cached ? cachedResponseForBrowser(request, route, cached) : null;
  } catch {
    return null;
  }
}

function writeEdgeCache(
  cache: EdgeCache,
  context: EdgeExecutionContext | undefined,
  request: Request,
  route: EdgeRoute,
  response: Response,
) {
  const write = cache
    .put(canonicalCacheRequest(request), responseForSharedCache(route, response))
    .catch(() => undefined);
  if (context) {
    context.waitUntil(write);
  } else {
    void write;
  }
}

export async function handleRequest(
  request: Request,
  env: Env,
  context?: EdgeExecutionContext,
  cacheOverride?: EdgeCache | null,
): Promise<Response> {
  if (!edgeEnabled(env.EDGE_DISABLED)) return serviceUnavailable();
  if (request.method !== "GET" && request.method !== "HEAD") {
    const response = jsonError(405, "method_not_allowed");
    response.headers.set("Allow", "GET, HEAD");
    return response;
  }

  const secret = env.COMMUNITY_EDGE_HMAC_SECRET?.trim();
  if (!secret || !validSecret(secret) || !env.COMMUNITY_BUCKET) return serviceUnavailable();
  const token = edgeCookie(request.headers.get("Cookie"));
  if (!token || !(await validEdgeToken(token, secret))) {
    return jsonError(401, "invalid_edge_session");
  }

  const route = parseRoute(new URL(request.url).pathname);
  if (!route) return jsonError(404, "not_found");

  const feedPrefix = safePrefix(env.COMMUNITY_FEED_PREFIX, DEFAULT_FEED_PREFIX);
  const resourcePrefix = safePrefix(env.COMMUNITY_RESOURCE_PREFIX, DEFAULT_RESOURCE_PREFIX);
  const assetPrefix = safePrefix(env.COMMUNITY_ASSET_PREFIX, DEFAULT_ASSET_PREFIX);
  if (!feedPrefix || !resourcePrefix || !assetPrefix) return serviceUnavailable();

  if (route.kind === "resource") {
    const active = await activeResourceState(
      env.COMMUNITY_BUCKET,
      resourcePrefix,
      route.resourceId,
      route.version,
    );
    if (active === "inactive") return jsonError(404, "resource_not_active");
    if (active === "invalid") return jsonError(502, "invalid_resource_active_index");
    if (active === "unavailable") return jsonError(503, "resource_active_index_unavailable");
  }

  const cache = cacheOverride === undefined ? defaultEdgeCache() : cacheOverride;
  if (request.method === "GET" && cache) {
    const cached = await readEdgeCache(cache, request, route);
    if (cached) return cached;
  }

  const edgeResponse =
    route.kind === "resource"
      ? await serveResourceFromR2(request, env, route, resourcePrefix, assetPrefix)
      : await serveFeedFromR2(request, env, route, feedPrefix);
  if (edgeResponse) {
    if (request.method === "GET" && edgeResponse.status === 200 && cache) {
      writeEdgeCache(cache, context, request, route, edgeResponse);
    }
    return edgeResponse;
  }
  if (route.kind === "resource" && request.method === "HEAD") {
    return jsonError(502, "resource_edge_unavailable");
  }
  return fallbackToLegacy(request, env, route);
}

export default {
  fetch(request: Request, env: Env, context: EdgeExecutionContext): Promise<Response> {
    return handleRequest(request, env, context);
  },
};
