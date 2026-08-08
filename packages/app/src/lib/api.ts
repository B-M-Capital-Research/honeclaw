import type {
  ChannelStatusInfo,
  CompanyProfile,
  CompanyProfileImportApplyRequest,
  CompanyProfileImportApplyResult,
  CompanyProfileImportPreview,
  CompanyProfileSpaceSummary,
  CompanyProfileSummary,
  HistoryMsg,
  PublicChatBootstrapResponse,
  PublicHistoryPageResponse,
  PublicPushListResponse,
  PublicPushOpenResponse,
  PublicAuthUserInfo,
  PublicBillingConfig,
  PublicBillingStatus,
  PublicAdminInviteList,
  PublicAdminInviteMutation,
  PublicAdminUsageReport,
  MetaInfo,
  SkillDetailInfo,
  SkillInfo,
  UserInfo,
  CronJobInfo,
  CronJobDetailInfo,
  CronJobUpsertInput,
  PortfolioInfo,
  PortfolioSummary,
  HoldingUpsertInput,
  LogEntry,
  DesktopChannelSettings,
  DesktopChannelSettingsInput,
  DesktopChannelSettingsUpdateResult,
  WebInviteActionResult,
  WebInviteInfo,
  FinanceCalendarPayload,
  PublicCommunityPage,
  PublicCommunityResource,
} from "./types";
import type { ActorRef } from "./actors";
import {
  apiFetch,
  buildApiUrl,
  createEventSource,
  friendlyBackendErrorMessage,
} from "./backend";
import { useLocale } from "./i18n";
import { setCachedPublicUser } from "./public-session-cache";

export class ApiError extends Error {
  status: number;
  statusText: string;

  constructor(message: string, response: Response) {
    super(message);
    this.name = "ApiError";
    this.status = response.status;
    this.statusText = response.statusText;
  }
}

export function isUnauthorizedApiError(error: unknown) {
  return (
    error instanceof ApiError && (error.status === 401 || error.status === 403)
  );
}

async function parseJson<T>(response: Response): Promise<T> {
  const contentType = response.headers.get("content-type") ?? "";
  if (!response.ok) {
    const friendlyMessage = friendlyBackendErrorMessage(response.status);
    if (friendlyMessage) {
      throw new ApiError(friendlyMessage, response);
    }
    const text = await response.text();
    let message = "";
    try {
      const payload = JSON.parse(text) as { error?: string; message?: string };
      message = payload.error || payload.message || "";
    } catch {
      message = "";
    }
    throw new ApiError(message || text || response.statusText, response);
  }
  if (!contentType.toLowerCase().includes("application/json")) {
    const text = await response.text();
    const snippet = text.trim().slice(0, 80);
    throw new ApiError(
      `Expected JSON response but received ${contentType || "unknown content type"}${
        snippet ? `: ${snippet}` : ""
      }`,
      response,
    );
  }
  return response.json() as Promise<T>;
}

async function apiErrorFromResponse(response: Response): Promise<ApiError> {
  const friendlyMessage = friendlyBackendErrorMessage(response.status);
  if (friendlyMessage) {
    return new ApiError(friendlyMessage, response);
  }
  const text = await response.text();
  return new ApiError(text || response.statusText, response);
}

export async function getMeta() {
  const response = await apiFetch("/api/meta");
  return parseJson<MetaInfo>(response);
}

export async function getChannels() {
  const response = await apiFetch("/api/channels");
  return parseJson<ChannelStatusInfo[]>(response);
}

export async function getChannelSettings() {
  const response = await apiFetch("/api/channel-settings");
  return parseJson<DesktopChannelSettings>(response);
}

export async function putChannelSettings(
  settings: DesktopChannelSettingsInput,
) {
  const response = await apiFetch("/api/channel-settings", {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(settings),
  });
  return parseJson<DesktopChannelSettingsUpdateResult>(response);
}

export async function getUsers() {
  const response = await apiFetch("/api/users");
  return parseJson<UserInfo[]>(response);
}

export async function getWebInvites() {
  const response = await apiFetch("/api/web-users/invites");
  const payload = await parseJson<{ invites?: WebInviteInfo[] }>(response);
  return payload.invites ?? [];
}

export async function createWebInvite(phoneNumber: string) {
  const response = await apiFetch("/api/web-users/invites", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ phone_number: phoneNumber }),
  });
  const payload = await parseJson<{ invite: WebInviteInfo }>(response);
  return payload.invite;
}

async function mutateWebInvite(
  userId: string,
  action: "disable" | "enable" | "reset" | "api-key" | "api-key/reset",
) {
  const response = await apiFetch(
    `/api/web-users/invites/${encodeURIComponent(userId)}/${action}`,
    {
      method: "POST",
    },
  );
  return parseJson<WebInviteActionResult>(response);
}

export async function disableWebInvite(userId: string) {
  return mutateWebInvite(userId, "disable");
}

export async function enableWebInvite(userId: string) {
  return mutateWebInvite(userId, "enable");
}

export async function resetWebInvite(userId: string) {
  return mutateWebInvite(userId, "reset");
}

export async function getWebInviteApiKey(userId: string) {
  return mutateWebInvite(userId, "api-key");
}

export async function resetWebInviteApiKey(userId: string) {
  return mutateWebInvite(userId, "api-key/reset");
}

function actorQuery(actor: ActorRef) {
  const params = new URLSearchParams({
    channel: actor.channel,
    user_id: actor.user_id,
  });
  if (actor.channel_scope) params.set("channel_scope", actor.channel_scope);
  return params.toString();
}

export async function getHistory(sessionId: string) {
  const response = await apiFetch(
    `/api/history?session_id=${encodeURIComponent(sessionId)}`,
  );
  const payload = await parseJson<{ messages?: HistoryMsg[] }>(response);
  return payload.messages ?? [];
}

export async function getSkills() {
  const response = await apiFetch("/api/skills");
  return parseJson<SkillInfo[]>(response);
}

export async function getSkill(skillId: string) {
  const response = await apiFetch(`/api/skills/${encodeURIComponent(skillId)}`);
  return parseJson<SkillDetailInfo>(response);
}

export async function updateSkillState(skillId: string, enabled: boolean) {
  const response = await apiFetch(
    `/api/skills/${encodeURIComponent(skillId)}/state`,
    {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ enabled }),
    },
  );
  return parseJson<SkillInfo>(response);
}

export async function resetSkillRegistry() {
  const response = await apiFetch("/api/skills/reset", {
    method: "POST",
  });
  return parseJson<SkillInfo[]>(response);
}

export async function sendChat(
  actor: ActorRef,
  message: string,
  signal?: AbortSignal,
) {
  const response = await apiFetch("/api/chat", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      channel: actor.channel,
      user_id: actor.user_id,
      channel_scope: actor.channel_scope,
      message,
    }),
    signal,
  });

  if (!response.ok) {
    throw await apiErrorFromResponse(response);
  }

  if (!response.body) {
    throw new Error("missing response body");
  }

  return response.body;
}

export async function connectEvents(actor: ActorRef) {
  return createEventSource(`/api/events?${actorQuery(actor)}`);
}

export async function getPublicCaptchaConfig() {
  const response = await apiFetch("/api/public/auth/captcha/config");
  return parseJson<{
    enabled: boolean;
    region: string;
    prefix: string;
    scene_id: string;
    script_url: string;
  }>(response);
}

export async function publicSendSmsCode(
  phoneNumber: string,
  captchaVerifyParam?: string,
) {
  const response = await apiFetch("/api/public/auth/sms/send", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      phone_number: phoneNumber,
      captcha_verify_param: captchaVerifyParam,
    }),
  });
  await parseJson<{ ok: boolean }>(response);
}

export async function publicSmsLogin(input: {
  phone_number: string;
  verify_code: string;
  remember: boolean;
  tos_version: string;
}) {
  const response = await apiFetch("/api/public/auth/sms/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{ user: PublicAuthUserInfo }>(response);
  return payload.user;
}

export async function publicSendEmailCode(
  emailAddress: string,
  intent?: "stripe_checkout",
) {
  const response = await apiFetch("/api/public/auth/email/send", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email_address: emailAddress, intent }),
  });
  return parseJson<{ ok: boolean; message: string }>(response);
}

export async function publicEmailLogin(input: {
  email_address: string;
  verify_code: string;
  remember: boolean;
  tos_version: string;
}) {
  const response = await apiFetch("/api/public/auth/email/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{ user: PublicAuthUserInfo }>(response);
  return payload.user;
}

export async function publicLogout() {
  try {
    const response = await apiFetch("/api/public/auth/logout", {
      method: "POST",
    });
    await parseJson<{ ok: boolean }>(response);
  } finally {
    // A later login must always obtain a grant for that session instead of
    // reusing the in-memory edge choice from the account that just logged out.
    resetPublicCommunityEdgeState();
  }
}

export async function getPublicAuthMe(signal?: AbortSignal) {
  const response = await apiFetch("/api/public/auth/me", { signal });
  const payload = await parseJson<{ user: PublicAuthUserInfo }>(response);
  // Remember it so the next route can paint before its own round-trip.
  setCachedPublicUser(payload.user);
  return payload.user;
}

export async function getPublicBillingConfig(signal?: AbortSignal) {
  const response = await apiFetch("/api/public/billing/config", {
    signal,
    cache: "no-store",
  });
  return parseJson<PublicBillingConfig>(response);
}

export async function getPublicBillingStatus(signal?: AbortSignal) {
  const response = await apiFetch("/api/public/billing/status", {
    signal,
    cache: "no-store",
  });
  return parseJson<PublicBillingStatus>(response);
}

export type StripeCheckoutOffer = "subscription" | "fixed_term";

export async function createStripeCheckout(offer: StripeCheckoutOffer) {
  const response = await apiFetch("/api/public/billing/checkout/stripe", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ offer }),
  });
  return parseJson<{ checkout_url: string }>(response);
}

export async function createStripePortal() {
  const response = await apiFetch("/api/public/billing/portal/stripe", {
    method: "POST",
  });
  return parseJson<{ portal_url: string }>(response);
}

const PUBLIC_ADMIN_ACTION_HEADERS = {
  "X-Hone-Admin-Action": "whitelist",
};

export async function getPublicAdminInvites(signal?: AbortSignal) {
  const response = await apiFetch("/api/public/admin/invites", {
    signal,
    cache: "no-store",
  });
  return parseJson<PublicAdminInviteList>(response);
}

export type PublicAdminUsageRangeDays = 14 | 30 | 90;

export async function getPublicAdminUsage(
  days: PublicAdminUsageRangeDays = 14,
  signal?: AbortSignal,
) {
  const response = await apiFetch(`/api/public/admin/usage?days=${days}`, {
    signal,
    cache: "no-store",
  });
  const report = await parseJson<PublicAdminUsageReport>(response);
  if (
    Number.isInteger(report.period_days) &&
    report.period_days > 0
  ) {
    return report;
  }
  const start = Date.parse(`${report.period_start}T00:00:00Z`);
  const end = Date.parse(`${report.period_end}T00:00:00Z`);
  const inferredDays =
    Number.isFinite(start) && Number.isFinite(end) && end >= start
      ? Math.floor((end - start) / 86_400_000) + 1
      : days;
  return { ...report, period_days: inferredDays };
}

export async function createPublicAdminInvite(phoneNumber: string) {
  const response = await apiFetch("/api/public/admin/invites", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...PUBLIC_ADMIN_ACTION_HEADERS,
    },
    body: JSON.stringify({ phone_number: phoneNumber }),
  });
  return parseJson<PublicAdminInviteMutation>(response);
}

export async function disablePublicAdminInvite(userId: string) {
  const response = await apiFetch(
    `/api/public/admin/invites/${encodeURIComponent(userId)}/disable`,
    {
      method: "POST",
      headers: PUBLIC_ADMIN_ACTION_HEADERS,
    },
  );
  return parseJson<PublicAdminInviteMutation>(response);
}

export async function getPublicChatBootstrap(signal?: AbortSignal) {
  const response = await apiFetch("/api/public/bootstrap", {
    signal,
    cache: "no-store",
  });
  return parseJson<PublicChatBootstrapResponse>(response);
}

export async function getPublicHistory(before?: number, signal?: AbortSignal) {
  const query = new URLSearchParams({ limit: "20" });
  if (before !== undefined) query.set("before", String(before));
  const response = await apiFetch(`/api/public/history?${query.toString()}`, {
    signal,
    cache: "no-store",
  });
  return parseJson<PublicHistoryPageResponse>(response);
}

export async function getPublicPushes(
  before?: string,
  limit = 30,
): Promise<PublicPushListResponse> {
  const query = new URLSearchParams({ limit: String(limit) });
  if (before) query.set("before", before);
  const response = await apiFetch(`/api/public/pushes?${query.toString()}`);
  return parseJson<PublicPushListResponse>(response);
}

export async function openPublicPush(
  pushId: string,
): Promise<PublicPushOpenResponse> {
  const response = await apiFetch(
    `/api/public/pushes/${encodeURIComponent(pushId)}/open`,
    { method: "POST" },
  );
  return parseJson<PublicPushOpenResponse>(response);
}

// ── Public investment context (mainline/profile reads + refresh) ──────────

export type ProfileSummary = {
  dir: string;
  ticker?: string;
  tickers?: string[];
  title?: string;
  preview?: string;
  bytes?: number;
};

export type DigestContext = {
  actor: { channel: string; user_id: string };
  mainline_style: string | null;
  mainline_by_ticker: Record<string, string>;
  global_digest_enabled?: boolean;
  global_digest_floor_macro_picks?: number;
  last_mainline_distilled_at: string | null;
  mainline_distill_skipped: string[];
  holdings: string[];
  profile_list: ProfileSummary[];
};

export async function getDigestContext(): Promise<DigestContext> {
  const response = await apiFetch("/api/public/digest-context");
  return parseJson<DigestContext>(response);
}

export type PublicQuote = {
  symbol: string;
  name?: string;
  price: number;
  change?: number;
  change_percent?: number;
};

export type PublicQuotesResponse = {
  available: boolean;
  quotes: PublicQuote[];
};

export async function getPublicQuotes(): Promise<PublicQuotesResponse> {
  const response = await apiFetch("/api/public/quotes");
  return parseJson<PublicQuotesResponse>(response);
}

// ── 我的：自选与持仓 ───────────────────────────────────────────────────────

export type PublicHolding = {
  symbol: string;
  name?: string | null;
  /** 仓位占比(%)，自选条目为 null。 */
  weight?: number | null;
  avg_cost?: number | null;
  notes?: string | null;
  tracking_only: boolean;
};

export type PublicPortfolioResponse = {
  holdings: PublicHolding[];
  limit: number;
};

export type PublicHoldingInput = {
  symbol: string;
  name?: string;
  weight?: number;
  avg_cost?: number;
};

export async function getPublicPortfolio(): Promise<PublicPortfolioResponse> {
  const response = await apiFetch("/api/public/portfolio");
  return parseJson<PublicPortfolioResponse>(response);
}

export async function createPublicHolding(
  input: PublicHoldingInput,
): Promise<PublicPortfolioResponse> {
  const response = await apiFetch("/api/public/portfolio/holdings", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  return parseJson<PublicPortfolioResponse>(response);
}

export async function updatePublicHolding(
  symbol: string,
  input: PublicHoldingInput,
): Promise<PublicPortfolioResponse> {
  const response = await apiFetch(
    `/api/public/portfolio/holdings/${encodeURIComponent(symbol)}`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(input),
    },
  );
  return parseJson<PublicPortfolioResponse>(response);
}

export async function deletePublicHolding(
  symbol: string,
): Promise<PublicPortfolioResponse> {
  const response = await apiFetch(
    `/api/public/portfolio/holdings/${encodeURIComponent(symbol)}`,
    { method: "DELETE" },
  );
  return parseJson<PublicPortfolioResponse>(response);
}

// ── 我的：设置 ────────────────────────────────────────────────────────────

export type PublicSettings = {
  style: string | null;
  distilled_style: string | null;
  user_edited: boolean;
  last_distilled_at?: string | null;
};

export async function getPublicSettings(): Promise<PublicSettings> {
  const response = await apiFetch("/api/public/settings");
  return parseJson<PublicSettings>(response);
}

export async function putPublicInvestorStyle(
  style: string,
): Promise<PublicSettings> {
  const response = await apiFetch("/api/public/settings/investor-style", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ style }),
  });
  return parseJson<PublicSettings>(response);
}

// ── Admin: mainline context for any actor ─────────────────────────────────

export type AdminMainlineContext = DigestContext & {
  actor: { channel: string; user_id: string; channel_scope?: string | null };
};

export async function getAdminMainlineContext(
  actor: ActorRef,
): Promise<AdminMainlineContext> {
  const q = actorQuery(actor);
  const response = await apiFetch(`/api/event-engine/mainline-context?${q}`);
  return parseJson<AdminMainlineContext>(response);
}

export async function getAdminCompanyProfile(
  actor: ActorRef,
  ticker: string,
): Promise<{ ticker: string; dir: string; markdown: string }> {
  const queryParams = new URLSearchParams({
    channel: actor.channel,
    user_id: actor.user_id,
    ticker,
  });
  if (actor.channel_scope) queryParams.set("channel_scope", actor.channel_scope);
  const response = await apiFetch(
    `/api/event-engine/company-profile?${queryParams.toString()}`,
  );
  return parseJson(response);
}

export async function adminTriggerMainlineDistill(actor: ActorRef): Promise<{
  ok: boolean;
  mainline_count: number;
  mainline_style_set: boolean;
  skipped_tickers: string[];
  last_distilled_at: string | null;
}> {
  const q = actorQuery(actor);
  const response = await apiFetch(`/api/event-engine/mainline-distill?${q}`, {
    method: "POST",
  });
  return parseJson(response);
}

export async function refreshDigestContext(): Promise<{
  ok: boolean;
  mainline_count: number;
  mainline_style_set: boolean;
  skipped_tickers: string[];
  last_distilled_at: string | null;
}> {
  const response = await apiFetch("/api/public/digest-context/refresh", {
    method: "POST",
  });
  return parseJson(response);
}

export async function getCompanyProfileMarkdown(ticker: string): Promise<{
  ticker: string;
  dir: string;
  markdown: string;
}> {
  const response = await apiFetch(
    `/api/public/company-profile?ticker=${encodeURIComponent(ticker)}`,
  );
  return parseJson(response);
}

export async function getPublicFinanceCalendar(
  month?: string,
): Promise<FinanceCalendarPayload> {
  const query = month ? `?month=${encodeURIComponent(month)}` : "";
  const response = await apiFetch(`/api/public/finance-calendar${query}`);
  return parseJson<FinanceCalendarPayload>(response);
}

export async function sendPublicFinanceCalendar(input: {
  path: string;
  mobile_path: string;
  month: string;
}): Promise<{ ok: boolean; message: string }> {
  const response = await apiFetch("/api/public/finance-calendar/send", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(input),
  });
  return parseJson<{ ok: boolean; message: string }>(response);
}

export type PublicUploadedAttachment = {
  path: string;
  name: string;
  kind: string;
  size: number;
};

export type PublicChatAttachmentInput = {
  path: string;
  name?: string;
};

export type PublicEarningsWorkflowInput = {
  kind: "preview" | "analysis";
  company: string;
};

export async function sendPublicChat(
  message: string,
  attachments: PublicChatAttachmentInput[] = [],
  signal?: AbortSignal,
  earningsWorkflow?: PublicEarningsWorkflowInput,
) {
  const response = await apiFetch("/api/public/chat", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      message,
      attachments,
      earnings_workflow: earningsWorkflow,
      // Tell the server what the user is actually reading, so the answer is
      // written in that language instead of guessed from the conversation.
      language: useLocale(),
    }),
    signal,
  });

  if (!response.ok) {
    throw await apiErrorFromResponse(response);
  }

  if (!response.body) {
    throw new Error("missing response body");
  }

  return response.body;
}

export async function uploadPublicAttachments(files: File[]) {
  if (!files.length) return [] as PublicUploadedAttachment[];
  const form = new FormData();
  for (const file of files) {
    form.append("files", file, file.name);
  }
  const response = await apiFetch("/api/public/upload", {
    method: "POST",
    body: form,
  });
  const payload = await parseJson<{ attachments: PublicUploadedAttachment[] }>(
    response,
  );
  return payload.attachments ?? [];
}

export async function getPublicGeneratedFileBlob(path: string) {
  const response = await apiFetch(
    `/api/public/file?path=${encodeURIComponent(path)}`,
  );
  if (!response.ok) throw await apiErrorFromResponse(response);
  return response.blob();
}

export async function connectPublicEvents() {
  return createEventSource("/api/public/events");
}

type PublicCommunityEdgeSession = {
  enabled: boolean;
  mode: "off" | "shadow" | "prefer" | string;
  base_path?: string | null;
  expires_at?: number | null;
};

type PublicCommunityState = {
  unread: boolean;
  latest_content_id?: number | null;
};

type ActiveCommunityEdge = {
  basePath: "/_community/v1";
  expiresAt: number;
};

const PUBLIC_COMMUNITY_EDGE_BASE_PATH = "/_community/v1" as const;
const PUBLIC_COMMUNITY_EDGE_RETRY_DELAY_MS = 30_000;
let publicCommunityEdgeDiscoveryEnabled =
  import.meta.env.VITE_HONE_APP_COMMUNITY_EDGE_DISCOVERY === "1";
let activePublicCommunityEdge: ActiveCommunityEdge | null = null;
let publicCommunityEdgeRetryAt = 0;

function resetPublicCommunityEdgeState() {
  activePublicCommunityEdge = null;
  publicCommunityEdgeRetryAt = 0;
}

/** Test-only override; production behavior remains a compile-time flag. */
export function setPublicCommunityEdgeDiscoveryForTests(enabled: boolean) {
  publicCommunityEdgeDiscoveryEnabled = enabled;
  resetPublicCommunityEdgeState();
}

export function resetPublicCommunityEdgeDiscoveryForTests() {
  publicCommunityEdgeDiscoveryEnabled =
    import.meta.env.VITE_HONE_APP_COMMUNITY_EDGE_DISCOVERY === "1";
  resetPublicCommunityEdgeState();
}

function normalizedPublicCommunityEdgeSession(
  payload: PublicCommunityEdgeSession,
): ActiveCommunityEdge | null {
  const now = Math.floor(Date.now() / 1_000);
  const expiresAt = Number(payload.expires_at);
  if (
    !payload.enabled ||
    payload.mode !== "prefer" ||
    payload.base_path !== PUBLIC_COMMUNITY_EDGE_BASE_PATH ||
    !Number.isSafeInteger(expiresAt) ||
    expiresAt <= now + 5
  ) {
    return null;
  }
  return { basePath: PUBLIC_COMMUNITY_EDGE_BASE_PATH, expiresAt };
}

async function discoverPublicCommunityEdge(signal?: AbortSignal) {
  if (!publicCommunityEdgeDiscoveryEnabled) return null;
  const now = Date.now();
  if (
    activePublicCommunityEdge &&
    activePublicCommunityEdge.expiresAt * 1_000 > now + 5_000
  ) {
    return activePublicCommunityEdge;
  }
  if (now < publicCommunityEdgeRetryAt) return null;

  try {
    const response = await apiFetch("/api/public/community/edge-session", {
      method: "POST",
      signal,
    });
    const payload = await parseJson<PublicCommunityEdgeSession>(response);
    activePublicCommunityEdge = normalizedPublicCommunityEdgeSession(payload);
    if (!activePublicCommunityEdge) {
      publicCommunityEdgeRetryAt = now + PUBLIC_COMMUNITY_EDGE_RETRY_DELAY_MS;
    }
    return activePublicCommunityEdge;
  } catch (error) {
    if (signal?.aborted) throw error;
    activePublicCommunityEdge = null;
    publicCommunityEdgeRetryAt = now + PUBLIC_COMMUNITY_EDGE_RETRY_DELAY_MS;
    return null;
  }
}

function publicCommunityEdgeFeedPath(
  edge: ActiveCommunityEdge,
  before?: number,
) {
  if (before && Number.isSafeInteger(before) && before > 0) {
    return `${edge.basePath}/feed/pages/${before}.json`;
  }
  return `${edge.basePath}/feed/latest.json`;
}

async function fetchPublicCommunityEdge(path: string, init: RequestInit = {}) {
  return fetch(buildApiUrl(path), {
    credentials: "include",
    ...init,
  });
}

function verifiedPublicCommunityDeliveryPath(
  resourceId: number,
  version?: string | null,
  deliveryPath?: string | null,
) {
  const normalizedVersion = version?.trim().toLowerCase();
  if (
    !activePublicCommunityEdge ||
    !Number.isSafeInteger(resourceId) ||
    resourceId <= 0 ||
    !normalizedVersion ||
    !/^[a-f0-9]{12}$/.test(normalizedVersion)
  ) {
    return null;
  }
  const expected = `${activePublicCommunityEdge.basePath}/resources/${resourceId}/${normalizedVersion}`;
  return deliveryPath === expected ? expected : null;
}

export async function getPublicCommunity(input: {
  before?: number;
  limit?: number;
  signal?: AbortSignal;
} = {}) {
  const edge =
    input.limit == null || input.limit === 20
      ? await discoverPublicCommunityEdge(input.signal)
      : null;
  if (edge) {
    try {
      const [feedResponse, stateResponse] = await Promise.all([
        fetchPublicCommunityEdge(publicCommunityEdgeFeedPath(edge, input.before), {
          signal: input.signal,
        }),
        fetch(buildApiUrl("/api/public/community/state"), {
          credentials: "include",
          signal: input.signal,
        }),
      ]);
      const [page, state] = await Promise.all([
        parseJson<PublicCommunityPage>(feedResponse),
        parseJson<PublicCommunityState>(stateResponse),
      ]);
      return {
        ...page,
        unread: state.unread,
        latest_content_id: state.latest_content_id,
      };
    } catch (error) {
      if (input.signal?.aborted) throw error;
      activePublicCommunityEdge = null;
      publicCommunityEdgeRetryAt =
        Date.now() + PUBLIC_COMMUNITY_EDGE_RETRY_DELAY_MS;
    }
  }

  const query = new URLSearchParams();
  if (input.before) query.set("before", String(input.before));
  if (input.limit) query.set("limit", String(input.limit));
  const suffix = query.size ? `?${query}` : "";
  const response = await apiFetch(`/api/public/community${suffix}`, {
    signal: input.signal,
  });
  return parseJson<PublicCommunityPage>(response);
}

export async function markPublicCommunitySeen(contentId: number) {
  const response = await apiFetch("/api/public/community/seen", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ content_id: contentId }),
  });
  return parseJson<{ ok: boolean }>(response);
}

function publicCommunityResourcePath(resourceId: number, version?: string | null) {
  const normalizedVersion = version?.trim();
  const suffix = normalizedVersion
    ? `?${new URLSearchParams({ v: normalizedVersion }).toString()}`
    : "";
  return `/api/public/community/resources/${resourceId}${suffix}`;
}

export function publicCommunityResourceUrl(
  resourceId: number,
  version?: string | null,
  deliveryPath?: string | null,
) {
  return buildApiUrl(
    verifiedPublicCommunityDeliveryPath(resourceId, version, deliveryPath) ??
      publicCommunityResourcePath(resourceId, version),
  );
}

export async function resolvePublicCommunityResourceUrl(
  resourceId: number,
  version?: string | null,
  deliveryPath?: string | null,
) {
  const legacyUrl = buildApiUrl(publicCommunityResourcePath(resourceId, version));
  const edgePath = verifiedPublicCommunityDeliveryPath(
    resourceId,
    version,
    deliveryPath,
  );
  if (!edgePath) return legacyUrl;
  try {
    const response = await fetchPublicCommunityEdge(edgePath, { method: "HEAD" });
    return response.ok ? buildApiUrl(edgePath) : legacyUrl;
  } catch {
    return legacyUrl;
  }
}

export function publicCommunityResourceDownloadName(
  resource: Pick<PublicCommunityResource, "resource_id" | "display_name" | "content_type">,
) {
  const fallback = `community-resource-${resource.resource_id}`;
  const displayName = resource.display_name?.trim() || fallback;
  const contentType = (resource.content_type ?? "")
    .split(";", 1)[0]!
    .trim()
    .toLowerCase();
  if (
    contentType ===
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" &&
    /\.xls$/i.test(displayName)
  ) {
    return displayName.replace(/\.xls$/i, ".xlsx");
  }
  return displayName;
}

export async function getPublicCommunityResourceBlob(
  resourceId: number,
  version?: string | null,
  deliveryPath?: string | null,
) {
  const edgePath = verifiedPublicCommunityDeliveryPath(
    resourceId,
    version,
    deliveryPath,
  );
  if (edgePath) {
    try {
      const edgeResponse = await fetchPublicCommunityEdge(edgePath);
      if (edgeResponse.ok) return edgeResponse.blob();
    } catch {
      // The legacy authenticated API remains the per-resource safety net.
    }
  }
  const response = await apiFetch(publicCommunityResourcePath(resourceId, version));
  if (!response.ok) throw await apiErrorFromResponse(response);
  return response.blob();
}

export async function getCronJobs(actor?: ActorRef) {
  const url = actor ? `/api/cron-jobs?${actorQuery(actor)}` : "/api/cron-jobs";
  const response = await apiFetch(url);
  const payload = await parseJson<{ jobs: CronJobInfo[] }>(response);
  return payload.jobs;
}

export async function getCronJob(id: string, actor?: ActorRef) {
  const url = actor
    ? `/api/cron-jobs/${encodeURIComponent(id)}?${actorQuery(actor)}`
    : `/api/cron-jobs/${encodeURIComponent(id)}`;
  const response = await apiFetch(url);
  const payload = await parseJson<{ job: CronJobDetailInfo }>(response);
  return payload.job;
}

export async function createCronJob(input: CronJobUpsertInput) {
  const response = await apiFetch("/api/cron-jobs", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{ job: CronJobInfo }>(response);
  return payload.job;
}

export async function updateCronJob(
  id: string,
  input: CronJobUpsertInput,
  actor?: ActorRef,
) {
  const url = actor
    ? `/api/cron-jobs/${encodeURIComponent(id)}?${actorQuery(actor)}`
    : `/api/cron-jobs/${encodeURIComponent(id)}`;
  const response = await apiFetch(url, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{ job: CronJobInfo }>(response);
  return payload.job;
}

export async function toggleCronJob(id: string, actor?: ActorRef) {
  const url = actor
    ? `/api/cron-jobs/${encodeURIComponent(id)}/toggle?${actorQuery(actor)}`
    : `/api/cron-jobs/${encodeURIComponent(id)}/toggle`;
  const response = await apiFetch(url, { method: "POST" });
  const payload = await parseJson<{ job: CronJobInfo }>(response);
  return payload.job;
}

export async function deleteCronJob(id: string, actor?: ActorRef) {
  const url = actor
    ? `/api/cron-jobs/${encodeURIComponent(id)}?${actorQuery(actor)}`
    : `/api/cron-jobs/${encodeURIComponent(id)}`;
  const response = await apiFetch(url, { method: "DELETE" });
  await parseJson(response);
  return true;
}

export async function listPortfolioActors() {
  const response = await apiFetch("/api/portfolio/actors");
  const payload = await parseJson<{ actors: PortfolioSummary[] }>(response);
  return payload.actors ?? [];
}

export async function getPortfolio(actor: ActorRef) {
  const response = await apiFetch(`/api/portfolio?${actorQuery(actor)}`);
  const payload = await parseJson<{
    portfolio: PortfolioInfo;
    summary: PortfolioSummary;
  }>(response);
  return payload;
}

export async function createHolding(input: HoldingUpsertInput) {
  const response = await apiFetch(`/api/portfolio/holdings`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{
    portfolio: PortfolioInfo;
    summary: PortfolioSummary;
  }>(response);
  return payload;
}

export async function updateHolding(symbol: string, input: HoldingUpsertInput) {
  const response = await apiFetch(
    `/api/portfolio/holdings/${encodeURIComponent(symbol)}`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(input),
    },
  );
  const payload = await parseJson<{
    portfolio: PortfolioInfo;
    summary: PortfolioSummary;
  }>(response);
  return payload;
}

export async function deleteHolding(symbol: string, actor: ActorRef) {
  const response = await apiFetch(
    `/api/portfolio/holdings/${encodeURIComponent(symbol)}?${actorQuery(actor)}`,
    {
      method: "DELETE",
    },
  );
  const payload = await parseJson<{
    portfolio: PortfolioInfo;
    summary: PortfolioSummary;
  }>(response);
  return payload;
}

// ── 个股深度研究 API ──────────────────────────────────────────────────────────

export type ResearchStartResponse = {
  message: string;
  task_id: string;
  task_name: string;
};

export type ResearchStatusResponse = {
  task_id: string;
  task_name: string;
  status: string;
  progress: string;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
  info: string | null;
  answer_file_path?: string;
  answer_exists?: boolean;
  /** 任务完成且文件存在时，直接返回 Markdown 原文 */
  answer_markdown?: string;
};

/** 接口一：发起深度研究，返回 task_id */
export async function startResearch(
  companyName: string,
): Promise<ResearchStartResponse> {
  const response = await apiFetch("/api/research/start", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ companyName }),
  });
  return parseJson<ResearchStartResponse>(response);
}

/** 接口二：轮询研究进度（完成时含 answer_markdown 原文） */
export async function getResearchStatus(
  taskId: string,
): Promise<ResearchStatusResponse> {
  const response = await apiFetch(
    `/api/research/status/${encodeURIComponent(taskId)}`,
  );
  return parseJson<ResearchStatusResponse>(response);
}

// ── 日志 API ─────────────────────────────────────────────────────────────────

/** 获取历史日志（最多 500 条） */
export async function getLogs(): Promise<LogEntry[]> {
  const response = await apiFetch("/api/logs");
  const payload = await parseJson<{ logs: LogEntry[] }>(response);
  return payload.logs ?? [];
}

// ── Task runs (周期任务观测) ────────────────────────────────────────────────

export type TaskOutcome = "ok" | "skipped" | "failed";

export interface TaskRunRecord {
  task: string;
  started_at: string;
  ended_at: string;
  outcome: TaskOutcome;
  items: number;
  error?: string | null;
}

export interface TaskSummary {
  last_seen_at: string | null;
  runs_24h: number;
  ok_24h: number;
  skipped_24h: number;
  failed_24h: number;
  last_error: string | null;
  last_failure_at: string | null;
  /// 最近一次失败之后又跑了多少次(ok/skipped 都算)。
  /// null = 24h 内没失败过;0 = 最新这次就是失败;>0 = 已恢复 N 次。
  runs_since_last_failure: number | null;
}

export interface TaskRunsResponse {
  runs: TaskRunRecord[];
  summary_by_task: Record<string, TaskSummary>;
  runtime_dir: string;
}

export async function getTaskRuns(opts?: {
  days?: number;
  limit?: number;
  task?: string;
}): Promise<TaskRunsResponse> {
  const params = new URLSearchParams();
  if (opts?.days != null) params.set("days", String(opts.days));
  if (opts?.limit != null) params.set("limit", String(opts.limit));
  if (opts?.task) params.set("task", opts.task);
  const qs = params.toString();
  const path = qs ? `/api/admin/task-runs?${qs}` : "/api/admin/task-runs";
  const response = await apiFetch(path);
  return parseJson<TaskRunsResponse>(response);
}

/** 连接实时日志 SSE 流 */
export async function connectLogStream() {
  return createEventSource("/api/logs/stream");
}

// ── 推送日志 API (cron 执行记录跨任务聚合) ────────────────────────────────

export interface NotificationRecord {
  run_id: number;
  record_source: "cron_job" | "event_engine" | string;
  job_id: string;
  job_name: string;
  event_kind?: string | null;
  channel: string;
  user_id: string;
  channel_scope?: string | null;
  channel_target: string;
  heartbeat: boolean;
  executed_at: string;
  execution_status: string;
  message_send_status: string;
  should_deliver: boolean;
  delivered: boolean;
  response_preview?: string | null;
  error_message?: string | null;
  detail?: unknown;
}

export interface NotificationHistogramBucket {
  bucket_start: string;
  total: number;
  sent: number;
  failed: number;
  skipped: number;
}

export interface NotificationsSummary {
  total: number;
  sent: number;
  failed: number;
  skipped: number;
  duplicate_suppressed: number;
  distinct_users: number;
}

export interface NotificationsResponse {
  records: NotificationRecord[];
  histogram_24h: NotificationHistogramBucket[];
  summary_24h: NotificationsSummary;
}

export interface NotificationsQuery {
  since?: string;
  until?: string;
  channel?: string;
  user_id?: string;
  channel_scope?: string;
  job_id?: string;
  execution_status?: string;
  message_send_status?: string;
  heartbeat_only?: boolean;
  limit?: number;
}

export async function getNotifications(
  q: NotificationsQuery = {},
): Promise<NotificationsResponse> {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(q)) {
    if (value === undefined || value === null || value === "") continue;
    params.set(key, String(value));
  }
  const qs = params.toString();
  const path = qs
    ? `/api/admin/notifications?${qs}`
    : "/api/admin/notifications";
  const response = await apiFetch(path);
  return parseJson<NotificationsResponse>(response);
}

// ── 推送日程 API (per-actor 拍平视图) ────────────────────────────────────────

export type ScheduleSource = "digest" | "cron_job";

export interface ScheduleEntry {
  time_local: string;
  source: ScheduleSource;
  content_hint: string;
  frequency: string;
  job_id?: string | null;
  will_be_held_by_quiet: boolean;
  bypass_quiet_hours: boolean;
  edit_hint: string;
}

export interface QuietHoursView {
  from: string;
  to: string;
  exempt_kinds: string[];
}

export type PricePolicySource = "system" | "actor_common" | "actor_direction";

export interface EffectivePriceDirectionPolicy {
  configured_first_pct: number;
  configured_first_source: PricePolicySource;
  first_direct_pct: number;
  system_floor_applied: boolean;
  large_position_first_direct_pct: number;
  first_candidate_band_pct: number;
  large_position_first_candidate_band_pct: number;
}

export interface EffectivePriceAlertPolicy {
  up: EffectivePriceDirectionPolicy;
  down: EffectivePriceDirectionPolicy;
  repeat_step_pct: number;
  repeat_step_source: PricePolicySource;
  candidate_first_pct: number;
  candidate_step_pct: number;
  min_direct_pct: number;
  large_position_weight_pct: number;
  close_direct_enabled: boolean;
}

export interface ImmediateConfig {
  event_engine_enabled: boolean;
  globally_disabled_kinds: string[];
  enabled: boolean;
  min_severity: string;
  portfolio_only: boolean;
  high_severity_daily_cap: number;
  same_symbol_cooldown_minutes: number;
  price_high_pct?: number | null;
  price_high_pct_up?: number | null;
  price_high_pct_down?: number | null;
  price_realert_step_pct?: number | null;
  large_position_weight_pct?: number | null;
  effective_price_alert_policy: EffectivePriceAlertPolicy;
  price_ladder_examples: { up: number[]; down: number[] };
  allow_kinds?: string[] | null;
  blocked_kinds: string[];
  immediate_kinds?: string[] | null;
  exempt_in_quiet: string[];
}

export interface ScheduleOverview {
  actor: string;
  timezone: string;
  quiet_hours?: QuietHoursView | null;
  schedule: ScheduleEntry[];
  immediate: ImmediateConfig;
}

export async function getSchedule(actor: string): Promise<ScheduleOverview> {
  const params = new URLSearchParams();
  params.set("actor", actor);
  const path = `/api/admin/schedule?${params.toString()}`;
  const response = await apiFetch(path);
  return parseJson<ScheduleOverview>(response);
}

// ── LLM Audit API ─────────────────────────────────────────────────────────────

import type {
  AuditQueryFilter,
  AuditRecordSummary,
  LlmAuditRecord,
} from "./types";

export async function getAuditRecords(filter: AuditQueryFilter) {
  const params = new URLSearchParams();
  for (const [k, v] of Object.entries(filter)) {
    if (v !== undefined && v !== "") {
      params.set(k, String(v));
    }
  }
  const response = await apiFetch(`/api/llm-audit?${params.toString()}`);
  return parseJson<{ records: AuditRecordSummary[]; total: number }>(response);
}

export async function getAuditRecordDetail(id: string) {
  const response = await apiFetch(`/api/llm-audit/${encodeURIComponent(id)}`);
  return parseJson<LlmAuditRecord>(response);
}

export async function listCompanyProfileActors() {
  const response = await apiFetch("/api/company-profiles/actors");
  const payload = await parseJson<{ actors: CompanyProfileSpaceSummary[] }>(
    response,
  );
  return payload.actors ?? [];
}

export async function listCompanyProfiles(actor: ActorRef) {
  const response = await apiFetch(`/api/company-profiles?${actorQuery(actor)}`);
  const payload = await parseJson<{ profiles: CompanyProfileSummary[] }>(
    response,
  );
  return payload.profiles;
}

export async function getCompanyProfile(profileId: string, actor: ActorRef) {
  const response = await apiFetch(
    `/api/company-profiles/${encodeURIComponent(profileId)}?${actorQuery(actor)}`,
  );
  const payload = await parseJson<{ profile: CompanyProfile }>(response);
  return payload.profile;
}

export async function deleteCompanyProfile(profileId: string, actor: ActorRef) {
  const response = await apiFetch(
    `/api/company-profiles/${encodeURIComponent(profileId)}?${actorQuery(actor)}`,
    {
      method: "DELETE",
    },
  );
  return parseJson<{ ok: boolean }>(response);
}

function parseDownloadFilename(response: Response, fallback: string) {
  const disposition = response.headers.get("content-disposition") ?? "";
  const match = disposition.match(/filename="([^"]+)"/i);
  return match?.[1]?.trim() || fallback;
}

export async function exportCompanyProfiles(actor: ActorRef) {
  const response = await apiFetch(
    `/api/company-profiles/export?${actorQuery(actor)}`,
  );
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || response.statusText);
  }
  const blob = await response.blob();
  const fallback = `company-profiles-${actor.channel}-${actor.user_id}.zip`;
  return {
    blob,
    fileName: parseDownloadFilename(response, fallback),
  };
}

export async function previewImportCompanyProfiles(
  actor: ActorRef,
  bundle: File,
) {
  const form = new FormData();
  form.append("bundle", bundle);
  const response = await apiFetch(
    `/api/company-profiles/import/preview?${actorQuery(actor)}`,
    {
      method: "POST",
      body: form,
    },
  );
  const payload = await parseJson<{ preview: CompanyProfileImportPreview }>(
    response,
  );
  return payload.preview;
}

export async function applyImportCompanyProfiles(
  actor: ActorRef,
  bundle: File,
  request: CompanyProfileImportApplyRequest,
) {
  const form = new FormData();
  form.append("bundle", bundle);
  form.append("mode", request.mode);
  form.append("decisions", JSON.stringify(request.decisions));
  const response = await apiFetch(
    `/api/company-profiles/import/apply?${actorQuery(actor)}`,
    {
      method: "POST",
      body: form,
    },
  );
  const payload = await parseJson<{ result: CompanyProfileImportApplyResult }>(
    response,
  );
  return payload.result;
}

// ── 通知偏好 API ──────────────────────────────────────────────────────────

/** 单个 digest 槽位 —— 后端 v0.4.x 起的新 schema(替代旧 digest_windows: string[])。
 *  时刻按 prefs.timezone 解释为本地 HH:MM;label 用于渲染 header,floor_macro 控制
 *  Pass 2 personalize 至少保留几条 macro_floor。前端编辑面板只渲染/写 id+time,
 *  label/floor_macro 透传不破坏。 */
export type DigestSlot = {
  id: string;
  time: string;
  label?: string | null;
  floor_macro?: number | null;
};

/** 勿扰时段:from/to 都是 prefs.timezone 解释的本地 HH:MM。在区间内 hold immediate
 *  推送 + 跳过 digest 触发,到 to 时刻一次性 quiet_flush;exempt_kinds 命中的 kind
 *  即使在 quiet 内也立即推。 */
export type QuietHoursPrefs = {
  from: string;
  to: string;
  exempt_kinds: string[];
};

export type NotificationPrefs = {
  enabled: boolean;
  portfolio_only: boolean;
  min_severity: "low" | "medium" | "high";
  allow_kinds: string[] | null;
  blocked_kinds: string[];
  /** IANA 时区名;null = 沿用全局 digest.timezone */
  timezone: string | null;
  /** digest 触发槽位列表;null = 沿用全局 default_slots;[] = 关 digest */
  digest_slots: DigestSlot[] | null;
  /** 价格异动即时推阈值(百分点);null = 沿用全局 thresholds.price_alert_high_pct */
  price_high_pct_override: number | null;
  /** 上涨方向价格异动阈值;null = 回落到通用 actor override / 全局阈值 */
  price_high_pct_up_override: number | null;
  /** 下跌方向价格异动阈值;null = 回落到通用 actor override / 全局阈值 */
  price_high_pct_down_override: number | null;
  /** 首次命中后的重复提醒最小前进步长;null = 沿用全局价格 band 推送步长 */
  price_realert_step_pct_override: number | null;
  /** 被视为大仓位的持仓权重百分比;null = 沿用全局 router 配置 */
  large_position_weight_pct: number | null;
  /** 强制升 High 即时推的 kind tag 列表;null/[] = 不强升 */
  immediate_kinds: string[] | null;
  /** 勿扰时段,null = 不启用 */
  quiet_hours: QuietHoursPrefs | null;
};

export type NotificationPrefsBundle = {
  prefs: NotificationPrefs;
  kind_tags: string[];
};

export type NotificationPrefsBatchEntry = {
  actor: ActorRef;
  prefs: NotificationPrefs;
};

export type NotificationPrefsBatchBundle = {
  entries: NotificationPrefsBatchEntry[];
  kind_tags: string[];
};

export async function getNotificationPrefs(
  actor: ActorRef,
): Promise<NotificationPrefsBundle> {
  const response = await apiFetch(
    `/api/notification-prefs?${actorQuery(actor)}`,
  );
  return parseJson<NotificationPrefsBundle>(response);
}

export async function getNotificationPrefsBatch(
  actors: ActorRef[],
): Promise<NotificationPrefsBatchBundle> {
  const response = await apiFetch("/api/notification-prefs/batch", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ actors }),
  });
  return parseJson<NotificationPrefsBatchBundle>(response);
}

export async function putNotificationPrefs(
  actor: ActorRef,
  prefs: NotificationPrefs,
): Promise<NotificationPrefs> {
  const body = {
    channel: actor.channel,
    user_id: actor.user_id,
    channel_scope: actor.channel_scope,
    prefs,
  };
  const response = await apiFetch("/api/notification-prefs", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const payload = await parseJson<{ prefs: NotificationPrefs }>(response);
  return payload.prefs;
}

export async function putLanguage(language: "zh" | "en"): Promise<"zh" | "en"> {
  const response = await apiFetch("/api/language", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ language }),
  });
  const payload = await parseJson<{ language: "zh" | "en" }>(response);
  return payload.language;
}
