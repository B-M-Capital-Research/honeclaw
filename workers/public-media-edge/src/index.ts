// Edge data plane for user-uploaded chat images.
//
// Browser <-> nearest Cloudflare PoP <-> R2. The GCE origin in us-central1 mints
// capabilities but never carries the bytes, which is the whole point: the old
// path pushed every pasted screenshot across the Pacific twice on upload and
// twice again to render it.
//
// Authorization is split deliberately:
//   * PUT  — a single-use, exact-key capability token in the X-Hone-Media-Token
//            header. The client cannot choose where bytes land, what content
//            type is recorded, or how many bytes it may write. A custom header
//            is also the CSRF guard: cross-site form submissions cannot set one.
//   * GET  — an HttpOnly, SameSite=Strict cookie scoped to /_media/v1/. Read
//            capabilities never appear in a URL, so they cannot leak through
//            Referer, browser history, or an intermediary's access log.
//
// Every token is bound to a subject and to a key prefix owned by that subject,
// and the prefix shape is re-checked structurally here so that a signing bug on
// the origin cannot by itself turn into cross-tenant access.

const MEDIA_COOKIE_NAME = "hone_media_edge";
const MEDIA_TOKEN_HEADER = "X-Hone-Media-Token";
const MEDIA_TOKEN_AUDIENCE = "hone-media-edge-v1";
const MEDIA_TOKEN_VERSION = 1;
const TOKEN_CLOCK_SKEW_SECONDS = 60;
const MAX_READ_TOKEN_LIFETIME_SECONDS = 3600;
const MAX_WRITE_TOKEN_LIFETIME_SECONDS = 300;
const MAX_SUBJECT_LENGTH = 512;
const MIN_SECRET_BYTES = 32;
const MAX_SECRET_BYTES = 1024;

// Mirrors PUBLIC_UPLOAD_MAX_BYTES in crates/hone-web-api/src/routes/public.rs.
// The token carries its own lower cap; this is the ceiling the edge will honor
// no matter what a token claims.
const MAX_OBJECT_BYTES = 10 * 1024 * 1024;

const DEFAULT_UPLOAD_PREFIX = "public-uploads";
const ALLOWED_ORIGIN = "https://hone-claw.com";

// Uploaded objects are immutable: the key carries a UUID and the edge refuses to
// overwrite. Still `private`, because the objects are per-user.
const OBJECT_CACHE_CONTROL = "private, max-age=31536000, immutable";
const NO_STORE_CACHE_CONTROL = "private, no-store";

interface R2HttpMetadataLike {
  readonly contentType?: string;
}

interface R2ObjectMetadataLike {
  readonly size: number;
  readonly httpEtag?: string;
  readonly httpMetadata?: R2HttpMetadataLike;
}

interface R2ObjectBodyLike extends R2ObjectMetadataLike {
  readonly body: ReadableStream<Uint8Array> | null;
}

export interface MediaBucket {
  get(key: string): Promise<R2ObjectBodyLike | null>;
  head(key: string): Promise<R2ObjectMetadataLike | null>;
  put(
    key: string,
    value: ArrayBuffer,
    options?: { httpMetadata?: { contentType?: string } },
  ): Promise<unknown>;
}

export interface Env {
  MEDIA_BUCKET?: MediaBucket;
  MEDIA_EDGE_HMAC_SECRET?: string;
  MEDIA_EDGE_DISABLED?: string;
  MEDIA_UPLOAD_PREFIX?: string;
}

interface ReadTokenPayload {
  v: number;
  aud: string;
  op: "get";
  sub: string;
  pfx: string;
  iat: number;
  exp: number;
}

interface WriteTokenPayload {
  v: number;
  aud: string;
  op: "put";
  sub: string;
  pfx: string;
  key: string;
  ct: string;
  max: number;
  iat: number;
  exp: number;
}

type TokenPayload = ReadTokenPayload | WriteTokenPayload;

/**
 * Absence disables the Worker. A route that is live before its secret and
 * bucket bindings are in place must fail closed, not fall through to something
 * permissive, so activation is an explicit `MEDIA_EDGE_DISABLED=false`.
 */
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

function jsonResponse(status: number, body: unknown): Response {
  const headers = securityHeaders(NO_STORE_CACHE_CONTROL);
  headers.set("Content-Type", "application/json; charset=utf-8");
  return new Response(JSON.stringify(body), { status, headers });
}

function jsonError(status: number, code: string): Response {
  return jsonResponse(status, { error: code });
}

function serviceUnavailable(): Response {
  return jsonError(503, "media_edge_unavailable");
}

function decodeBase64Url(segment: string): Uint8Array | null {
  if (!/^[A-Za-z0-9_-]+$/.test(segment)) return null;
  const remainder = segment.length % 4;
  if (remainder === 1) return null;
  const base64 =
    segment.replace(/-/g, "+").replace(/_/g, "/") + "=".repeat((4 - remainder) % 4);
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

function isSafeKeySegment(segment: string): boolean {
  return (
    segment !== "" &&
    segment !== "." &&
    segment !== ".." &&
    /^[A-Za-z0-9._-]+$/.test(segment)
  );
}

function safePrefix(raw: string | undefined, fallback: string): string | null {
  const value = (raw ?? fallback).trim().replace(/\/+$/g, "");
  if (!value || value.startsWith("/")) return null;
  return value.split("/").every(isSafeKeySegment) ? value : null;
}

/**
 * A user's upload root: `<uploadPrefix>/<owner>/`. Two segments exactly, both
 * safe. Checking the shape here means a malformed or over-broad `pfx` claim is
 * rejected even though it carries a valid signature.
 */
function validOwnerPrefix(prefix: string, uploadPrefix: string): boolean {
  // The trailing slash is load-bearing, not cosmetic: without it a prefix for
  // `.../web-user-1` would also match `.../web-user-11/...` under startsWith.
  if (!prefix.endsWith("/")) return false;
  const segments = prefix.slice(0, -1).split("/");
  if (segments.length !== uploadPrefix.split("/").length + 1) return false;
  if (!segments.every(isSafeKeySegment)) return false;
  return segments.slice(0, -1).join("/") === uploadPrefix;
}

/**
 * An object key: `<uploadPrefix>/<owner>/<day>/<stored-name>`. The day segment
 * keeps listings shallow; the stored name is uuid-prefixed on the origin, so a
 * key can never be re-derived by a client from a filename it controls.
 */
function validObjectKey(key: string, uploadPrefix: string): boolean {
  if (key.includes("%") || key.startsWith("/") || key.endsWith("/")) return false;
  const segments = key.split("/");
  if (segments.length !== uploadPrefix.split("/").length + 3) return false;
  if (!segments.every(isSafeKeySegment)) return false;
  return segments.slice(0, uploadPrefix.split("/").length).join("/") === uploadPrefix;
}

function mediaCookie(cookieHeader: string | null): string | null {
  if (!cookieHeader) return null;
  const matches: string[] = [];
  for (const part of cookieHeader.split(";")) {
    const pair = part.trim();
    const separator = pair.indexOf("=");
    if (separator <= 0) continue;
    if (pair.slice(0, separator).trim() === MEDIA_COOKIE_NAME) {
      matches.push(pair.slice(separator + 1).trim());
    }
  }
  // Two cookies of the same name means someone is trying to confuse the parser;
  // there is no safe way to pick a winner, so reject.
  return matches.length === 1 && matches[0] !== "" ? matches[0] : null;
}

function validCommonClaims(payload: Partial<TokenPayload>, nowSeconds: number): boolean {
  if (payload.v !== MEDIA_TOKEN_VERSION || payload.aud !== MEDIA_TOKEN_AUDIENCE) return false;
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
  const maxLifetime =
    payload.op === "put" ? MAX_WRITE_TOKEN_LIFETIME_SECONDS : MAX_READ_TOKEN_LIFETIME_SECONDS;
  return lifetime > 0 && lifetime <= maxLifetime;
}

function validReadPayload(
  value: unknown,
  nowSeconds: number,
  uploadPrefix: string,
): value is ReadTokenPayload {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const payload = value as Partial<ReadTokenPayload>;
  if (payload.op !== "get") return false;
  if (!validCommonClaims(payload, nowSeconds)) return false;
  return typeof payload.pfx === "string" && validOwnerPrefix(payload.pfx, uploadPrefix);
}

function validWritePayload(
  value: unknown,
  nowSeconds: number,
  uploadPrefix: string,
): value is WriteTokenPayload {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const payload = value as Partial<WriteTokenPayload>;
  if (payload.op !== "put") return false;
  if (!validCommonClaims(payload, nowSeconds)) return false;
  if (typeof payload.pfx !== "string" || !validOwnerPrefix(payload.pfx, uploadPrefix)) return false;
  if (typeof payload.key !== "string" || !validObjectKey(payload.key, uploadPrefix)) return false;
  // The owner prefix is carried rather than re-derived, because deriving it here
  // would mean reimplementing the origin's key sanitizer in a second language.
  // Requiring the two signed claims to agree still catches a grant that mints a
  // key for one tenant while authenticating another.
  if (!payload.key.startsWith(payload.pfx)) return false;
  if (typeof payload.ct !== "string" || storedContentType(payload.ct) === null) return false;
  return (
    Number.isSafeInteger(payload.max) &&
    (payload.max as number) > 0 &&
    (payload.max as number) <= MAX_OBJECT_BYTES
  );
}

async function verifiedTokenPayload(
  token: string,
  secret: string,
  accept: (value: unknown) => boolean,
): Promise<unknown | null> {
  const segments = token.split(".");
  if (segments.length !== 2) return null;
  const [payloadSegment, signatureSegment] = segments;
  const payloadBytes = decodeBase64Url(payloadSegment);
  const signature = decodeBase64Url(signatureSegment);
  if (!payloadBytes || !signature || signature.byteLength !== 32) return null;

  let payload: unknown;
  try {
    payload = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(payloadBytes));
  } catch {
    return null;
  }
  if (!accept(payload)) return null;

  try {
    const key = await crypto.subtle.importKey(
      "raw",
      new TextEncoder().encode(secret),
      { name: "HMAC", hash: "SHA-256" },
      false,
      ["verify"],
    );
    // crypto.subtle.verify is constant-time; never compare signatures manually.
    const verified = await crypto.subtle.verify(
      "HMAC",
      key,
      copiedArrayBuffer(signature),
      new TextEncoder().encode(payloadSegment),
    );
    return verified ? payload : null;
  } catch {
    return null;
  }
}

/**
 * The only content types this edge will ever store or serve inline.
 *
 * image/svg+xml is deliberately absent and must stay absent: an SVG served from
 * hone-claw.com is same-origin script, so accepting one would turn an upload
 * box into stored XSS against every logged-in user.
 */
function storedContentType(raw: string): string | null {
  const normalized = raw.split(";", 1)[0].trim().toLowerCase();
  switch (normalized) {
    case "image/jpg":
    case "image/jpeg":
      return "image/jpeg";
    case "image/png":
    case "image/webp":
    case "image/gif":
      return normalized;
    default:
      return null;
  }
}

/**
 * Content type implied by the leading bytes, or null when the bytes are not one
 * of the four formats we accept. A declared type is never trusted on its own:
 * without this, a token signed for image/png would happily store HTML.
 */
function sniffedContentType(bytes: Uint8Array): string | null {
  const startsWith = (...signature: number[]) =>
    signature.length <= bytes.length && signature.every((byte, index) => bytes[index] === byte);

  if (startsWith(0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a)) return "image/png";
  if (startsWith(0xff, 0xd8, 0xff)) return "image/jpeg";
  if (startsWith(0x47, 0x49, 0x46, 0x38, 0x37, 0x61)) return "image/gif";
  if (startsWith(0x47, 0x49, 0x46, 0x38, 0x39, 0x61)) return "image/gif";
  if (
    startsWith(0x52, 0x49, 0x46, 0x46) &&
    bytes.length >= 12 &&
    bytes[8] === 0x57 &&
    bytes[9] === 0x45 &&
    bytes[10] === 0x42 &&
    bytes[11] === 0x50
  ) {
    return "image/webp";
  }
  return null;
}

/**
 * Read at most `limit` bytes, then give up. Content-Length is a client claim, so
 * it gates the request but cannot be the thing that bounds memory.
 */
async function readCappedBody(
  body: ReadableStream<Uint8Array> | null,
  limit: number,
): Promise<Uint8Array | null> {
  if (!body) return null;
  const reader = body.getReader();
  const chunks: Uint8Array[] = [];
  let total = 0;
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      if (!value) continue;
      total += value.byteLength;
      if (total > limit) {
        await reader.cancel().catch(() => undefined);
        return null;
      }
      chunks.push(value);
    }
  } catch {
    return null;
  } finally {
    reader.releaseLock();
  }
  const merged = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    merged.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return merged;
}

/**
 * Same-origin only. There is no CORS handling anywhere in this Worker and there
 * must not be: a leaked write token still cannot be spent from another site,
 * because the browser will not attach the custom header cross-origin without a
 * preflight this Worker refuses to answer.
 */
function sameOriginRequest(request: Request): boolean {
  const origin = request.headers.get("Origin");
  if (origin !== null && origin !== ALLOWED_ORIGIN) return false;
  const site = request.headers.get("Sec-Fetch-Site");
  if (site !== null && site !== "same-origin" && site !== "none") return false;
  return true;
}

function objectKeyFromPath(pathname: string): string | null {
  const prefix = "/_media/v1/o/";
  if (!pathname.startsWith(prefix)) return null;
  const key = pathname.slice(prefix.length);
  return key.length > 0 ? key : null;
}

async function handleUpload(
  request: Request,
  env: Env,
  key: string,
  secret: string,
  uploadPrefix: string,
): Promise<Response> {
  const bucket = env.MEDIA_BUCKET;
  if (!bucket) return serviceUnavailable();

  const rawToken = request.headers.get(MEDIA_TOKEN_HEADER);
  if (!rawToken) return jsonError(401, "missing_upload_token");
  const payload = (await verifiedTokenPayload(rawToken.trim(), secret, (value) =>
    validWritePayload(value, Math.floor(Date.now() / 1000), uploadPrefix),
  )) as WriteTokenPayload | null;
  if (!payload) return jsonError(401, "invalid_upload_token");

  // The URL is not an independent input: it must name exactly the object the
  // token was minted for.
  if (payload.key !== key) return jsonError(403, "upload_token_key_mismatch");

  const declared = storedContentType(payload.ct);
  if (!declared) return jsonError(403, "upload_token_content_type_rejected");

  const limit = Math.min(payload.max, MAX_OBJECT_BYTES);
  const rawLength = request.headers.get("Content-Length");
  if (!rawLength || !/^[0-9]+$/.test(rawLength)) return jsonError(411, "length_required");
  const declaredLength = Number(rawLength);
  if (!Number.isSafeInteger(declaredLength) || declaredLength <= 0 || declaredLength > limit) {
    return jsonError(413, "upload_too_large");
  }

  // Immutable keys: refuse to replace an object that already exists. Together
  // with the uuid in the key this makes a captured token single-use — replaying
  // it can only target the object it already wrote.
  let existing: R2ObjectMetadataLike | null;
  try {
    existing = await bucket.head(key);
  } catch {
    return jsonError(503, "media_store_unavailable");
  }
  if (existing) return jsonError(409, "object_already_exists");

  const bytes = await readCappedBody(request.body, limit);
  if (!bytes) return jsonError(413, "upload_too_large");
  if (bytes.byteLength === 0) return jsonError(400, "empty_upload");
  if (bytes.byteLength !== declaredLength) return jsonError(400, "content_length_mismatch");

  const sniffed = sniffedContentType(bytes);
  if (!sniffed) return jsonError(415, "unsupported_image_format");
  if (sniffed !== declared) return jsonError(415, "content_type_mismatch");

  try {
    // `bytes` owns a buffer sized exactly to its contents, so hand it over
    // rather than copying another 10MB on every upload.
    await bucket.put(key, bytes.buffer as ArrayBuffer, {
      httpMetadata: { contentType: declared },
    });
  } catch {
    return jsonError(503, "media_store_unavailable");
  }

  return jsonResponse(201, { ok: true, bytes: bytes.byteLength });
}

async function handleRead(
  request: Request,
  env: Env,
  key: string,
  secret: string,
  uploadPrefix: string,
): Promise<Response> {
  const bucket = env.MEDIA_BUCKET;
  if (!bucket) return serviceUnavailable();

  const cookie = mediaCookie(request.headers.get("Cookie"));
  if (!cookie) return jsonError(401, "missing_media_session");
  const payload = (await verifiedTokenPayload(cookie, secret, (value) =>
    validReadPayload(value, Math.floor(Date.now() / 1000), uploadPrefix),
  )) as ReadTokenPayload | null;
  if (!payload) return jsonError(401, "invalid_media_session");

  // The signed prefix is what separates tenants. `validObjectKey` has already
  // rejected traversal and odd segments; this is the ownership check.
  if (!key.startsWith(payload.pfx)) return jsonError(403, "object_outside_session_scope");

  let object: R2ObjectMetadataLike | R2ObjectBodyLike | null;
  try {
    object = request.method === "HEAD" ? await bucket.head(key) : await bucket.get(key);
  } catch {
    return jsonError(503, "media_store_unavailable");
  }
  if (!object) return jsonError(404, "not_found");
  if (!Number.isSafeInteger(object.size) || object.size <= 0 || object.size > MAX_OBJECT_BYTES) {
    return jsonError(502, "object_size_invalid");
  }

  // Only ever serve back a type from the allowlist. The stored metadata was
  // written by this Worker, but re-deriving it here means a tampered or legacy
  // object cannot dictate a content type the browser will execute.
  const contentType = storedContentType(object.httpMetadata?.contentType ?? "");
  if (!contentType) return jsonError(502, "object_content_type_rejected");

  const headers = securityHeaders(OBJECT_CACHE_CONTROL);
  headers.set("Content-Type", contentType);
  headers.set("Content-Disposition", "inline");
  headers.set("Content-Length", String(object.size));
  const etag = object.httpEtag;
  if (etag && /^(W\/)?"[^"\r\n]+"$/.test(etag)) headers.set("ETag", etag);

  if (request.method === "HEAD") {
    return new Response(null, { status: 200, headers });
  }
  const body = "body" in object ? (object as R2ObjectBodyLike).body : null;
  if (!body) return jsonError(503, "media_store_unavailable");
  return new Response(body, { status: 200, headers });
}

export async function handleRequest(request: Request, env: Env): Promise<Response> {
  if (!edgeEnabled(env.MEDIA_EDGE_DISABLED)) return serviceUnavailable();

  const secret = env.MEDIA_EDGE_HMAC_SECRET?.trim();
  if (!secret || !validSecret(secret) || !env.MEDIA_BUCKET) return serviceUnavailable();

  const uploadPrefix = safePrefix(env.MEDIA_UPLOAD_PREFIX, DEFAULT_UPLOAD_PREFIX);
  if (!uploadPrefix) return serviceUnavailable();

  if (!sameOriginRequest(request)) return jsonError(403, "cross_origin_rejected");

  const method = request.method;
  if (method !== "GET" && method !== "HEAD" && method !== "PUT") {
    const response = jsonError(405, "method_not_allowed");
    response.headers.set("Allow", "GET, HEAD, PUT");
    return response;
  }

  const key = objectKeyFromPath(new URL(request.url).pathname);
  if (!key || !validObjectKey(key, uploadPrefix)) return jsonError(404, "not_found");

  return method === "PUT"
    ? handleUpload(request, env, key, secret, uploadPrefix)
    : handleRead(request, env, key, secret, uploadPrefix);
}

export default {
  fetch(request: Request, env: Env): Promise<Response> {
    return handleRequest(request, env);
  },
};
