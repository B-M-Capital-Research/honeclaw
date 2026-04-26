import type {
  ChannelStatusInfo,
  CompanyProfile,
  CompanyProfileImportApplyRequest,
  CompanyProfileImportApplyResult,
  CompanyProfileImportPreview,
  CompanyProfileSpaceSummary,
  CompanyProfileSummary,
  HistoryMsg,
  PublicAuthUserInfo,
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
  WebInviteActionResult,
  WebInviteInfo,
} from "./types";
import type { ActorRef } from "./actors";
import { apiFetch, createEventSource } from "./backend";

async function parseJson<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text();
    let message = "";
    try {
      const payload = JSON.parse(text) as { error?: string; message?: string };
      message = payload.error || payload.message || "";
    } catch {
      message = "";
    }
    throw new Error(message || text || response.statusText);
  }
  return response.json() as Promise<T>;
}

export async function getMeta() {
  const response = await apiFetch("/api/meta");
  return parseJson<MetaInfo>(response);
}

export async function getChannels() {
  const response = await apiFetch("/api/channels");
  return parseJson<ChannelStatusInfo[]>(response);
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
  action: "disable" | "enable" | "reset",
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
    const text = await response.text();
    throw new Error(text || response.statusText);
  }

  if (!response.body) {
    throw new Error("missing response body");
  }

  return response.body;
}

export async function connectEvents(actor: ActorRef) {
  return createEventSource(`/api/events?${actorQuery(actor)}`);
}

export async function publicInviteLogin(
  inviteCode: string,
  phoneNumber: string,
) {
  const response = await apiFetch("/api/public/auth/invite-login", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      invite_code: inviteCode,
      phone_number: phoneNumber,
    }),
  });
  const payload = await parseJson<{ user: PublicAuthUserInfo }>(response);
  return payload.user;
}

export async function publicPasswordLogin(input: {
  phone_number: string;
  password: string;
  remember: boolean;
}) {
  const response = await apiFetch("/api/public/auth/password-login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{ user: PublicAuthUserInfo }>(response);
  return payload.user;
}

export async function setPublicPassword(input: {
  new_password: string;
  tos_version: string;
}) {
  const response = await apiFetch("/api/public/auth/set-password", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{ user: PublicAuthUserInfo }>(response);
  return payload.user;
}

export async function changePublicPassword(input: {
  current_password: string;
  new_password: string;
}) {
  const response = await apiFetch("/api/public/auth/change-password", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  const payload = await parseJson<{ user: PublicAuthUserInfo }>(response);
  return payload.user;
}

export async function publicLogout() {
  const response = await apiFetch("/api/public/auth/logout", {
    method: "POST",
  });
  await parseJson<{ ok: boolean }>(response);
}

export async function getPublicAuthMe() {
  const response = await apiFetch("/api/public/auth/me");
  const payload = await parseJson<{ user: PublicAuthUserInfo }>(response);
  return payload.user;
}

export async function getPublicHistory() {
  const response = await apiFetch("/api/public/history");
  const payload = await parseJson<{ messages?: HistoryMsg[] }>(response);
  return payload.messages ?? [];
}

// ── Public digest context (read-only thesis + profiles surface) ─────────

export type ProfileSummary = {
  dir: string;
  tickers: string[];
  title: string;
  preview: string;
  bytes: number;
};

export type DigestContext = {
  actor: { channel: string; user_id: string };
  investment_global_style: string | null;
  investment_theses: Record<string, string>;
  global_digest_enabled: boolean;
  global_digest_floor_macro_picks: number;
  last_thesis_distilled_at: string | null;
  thesis_distill_skipped: string[];
  holdings: string[];
  profile_list: ProfileSummary[];
};

export async function getDigestContext(): Promise<DigestContext> {
  const response = await apiFetch("/api/public/digest-context");
  return parseJson<DigestContext>(response);
}

export async function refreshDigestContext(): Promise<{
  ok: boolean;
  theses_count: number;
  global_style_set: boolean;
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

export async function sendPublicChat(
  message: string,
  attachments: PublicChatAttachmentInput[] = [],
  signal?: AbortSignal,
) {
  const response = await apiFetch("/api/public/chat", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ message, attachments }),
    signal,
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || response.statusText);
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

export async function connectPublicEvents() {
  return createEventSource("/api/public/events");
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

/** 连接实时日志 SSE 流 */
export async function connectLogStream() {
  return createEventSource("/api/logs/stream");
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

export type NotificationPrefs = {
  enabled: boolean;
  portfolio_only: boolean;
  min_severity: "low" | "medium" | "high";
  allow_kinds: string[] | null;
  blocked_kinds: string[];
  /** IANA 时区名;null = 沿用全局 digest.timezone */
  timezone: string | null;
  /** 本地 HH:MM 列表;null = 沿用全局 [pre_market, post_market];[] = 关 digest */
  digest_windows: string[] | null;
  /** 价格异动即时推阈值(百分点);null = 沿用全局 thresholds.price_alert_high_pct */
  price_high_pct_override: number | null;
  /** 强制升 High 即时推的 kind tag 列表;null/[] = 不强升 */
  immediate_kinds: string[] | null;
};

export type NotificationPrefsBundle = {
  prefs: NotificationPrefs;
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
