import type {
  ChannelStatusInfo,
  HistoryMsg,
  KbEntry,
  MetaInfo,
  SkillDetailInfo,
  SkillInfo,
  StockRow,
  UserInfo,
  CronJobInfo,
  CronJobDetailInfo,
  CronJobUpsertInput,
  PortfolioInfo,
  PortfolioSummary,
  HoldingUpsertInput,
  LogEntry,
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

export async function sendChat(actor: ActorRef, message: string, signal?: AbortSignal) {
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

// ── Knowledge Base ────────────────────────────────────────────────────────────

export async function getKbEntries() {
  const response = await apiFetch("/api/kb");
  const payload = await parseJson<{ entries: KbEntry[] }>(response);
  return payload.entries;
}

export async function getKbEntry(id: string) {
  const response = await apiFetch(`/api/kb/${encodeURIComponent(id)}`);
  return parseJson<{ entry: KbEntry; parsed_text?: string }>(response);
}

export async function getKbStockTable() {
  const response = await apiFetch("/api/kb-stock-table");
  const payload = await parseJson<{ rows: StockRow[] }>(response);
  return payload.rows;
}

export async function updateStockKnowledge(params: {
  company_name: string;
  stock_code: string;
  key_knowledge: string[];
}) {
  const response = await apiFetch("/api/kb-stock-table/knowledge", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(params),
  });
  return parseJson<{ ok: boolean }>(response);
}

export async function analyzeKbEntry(id: string) {
  const response = await apiFetch(`/api/kb/${encodeURIComponent(id)}/analyze`, {
    method: "POST",
  });
  return parseJson<{ ok: boolean }>(response);
}

export async function deleteKbEntry(id: string) {
  const response = await apiFetch(`/api/kb/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
  return parseJson<{ ok: boolean }>(response);
}

export async function uploadKbFile(file: File) {
  const form = new FormData();
  form.append("file", file);
  // 注意：不设 Content-Type，让浏览器自动带上 multipart boundary
  const response = await apiFetch("/api/kb/upload", {
    method: "POST",
    body: form,
  });
  return parseJson<{ ok: boolean; entry: KbEntry }>(response);
}
