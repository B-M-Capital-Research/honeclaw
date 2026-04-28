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

// ── Admin: thesis context for any actor ─────────────────────────────────

export type AdminThesisContext = DigestContext & {
  actor: { channel: string; user_id: string; channel_scope?: string | null };
};

export async function getAdminThesisContext(
  actor: ActorRef,
): Promise<AdminThesisContext> {
  const q = actorQuery(actor);
  const response = await apiFetch(`/api/event-engine/thesis-context?${q}`);
  return parseJson<AdminThesisContext>(response);
}

export async function getAdminCompanyProfile(
  actor: ActorRef,
  ticker: string,
): Promise<{ ticker: string; dir: string; markdown: string }> {
  const p = new URLSearchParams({
    channel: actor.channel,
    user_id: actor.user_id,
    ticker,
  });
  if (actor.channel_scope) p.set("channel_scope", actor.channel_scope);
  const response = await apiFetch(
    `/api/event-engine/company-profile?${p.toString()}`,
  );
  return parseJson(response);
}

export async function adminTriggerThesisDistill(actor: ActorRef): Promise<{
  ok: boolean;
  theses_count: number;
  global_style_set: boolean;
  skipped_tickers: string[];
  last_distilled_at: string | null;
}> {
  const q = actorQuery(actor);
  const response = await apiFetch(`/api/event-engine/thesis-distill?${q}`, {
    method: "POST",
  });
  return parseJson(response);
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
  job_id: string;
  job_name: string;
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

export type ScheduleSource = "portfolio_digest" | "global_digest" | "cron_job";

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

export interface ImmediateConfig {
  enabled: boolean;
  min_severity: string;
  portfolio_only: boolean;
  price_high_pct?: number | null;
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
