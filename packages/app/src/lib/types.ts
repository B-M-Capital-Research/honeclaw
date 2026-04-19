export type UserInfo = {
  channel: string;
  user_id: string;
  channel_scope?: string;
  session_id: string;
  session_kind: "direct" | "group" | string;
  session_label: string;
  actor_user_id?: string;
  last_message: string;
  last_role: string;
  last_time: string;
  message_count: number;
};

export type SkillInfo = {
  id: string;
  display_name: string;
  description: string;
  when_to_use?: string;
  aliases: string[];
  allowed_tools: string[];
  user_invocable: boolean;
  context: string;
  loaded_from: string;
  enabled: boolean;
  disabled_reason?: string;
  has_script: boolean;
  has_path_gate: boolean;
  paths: string[];
};

export type SkillDetailInfo = {
  summary: SkillInfo;
  markdown: string;
  detail_path: string;
};

export type HistoryMsg = {
  role: "user" | "assistant" | "system" | string;
  content: string;
  subtype?:
    | "compact_boundary"
    | "compact_summary"
    | "compact_skill_snapshot"
    | string;
  synthetic?: boolean;
  transcript_only?: boolean;
};

export type WebInviteInfo = {
  user_id: string;
  invite_code: string;
  phone_number: string;
  created_at: string;
  last_login_at?: string;
  revoked_at?: string;
  enabled: boolean;
  active_session_count: number;
  daily_limit: number;
  success_count: number;
  in_flight: number;
  remaining_today: number;
};

export type WebInviteActionResult = {
  invite: WebInviteInfo;
  cleared_session_count: number;
  message: string;
};

export type PublicAuthUserInfo = {
  user_id: string;
  created_at: string;
  last_login_at?: string;
  daily_limit: number;
  success_count: number;
  in_flight: number;
  remaining_today: number;
};

export type MetaInfo = {
  name: string;
  version: string;
  channel: string;
  supportsImessage: boolean;
  apiVersion: string;
  capabilities: string[];
  deploymentMode: "local" | "remote";
};

export type BackendConfig = {
  mode: "bundled" | "remote";
  baseUrl: string;
  bearerToken: string;
};

export type BackendStatusInfo = {
  config: BackendConfig;
  resolvedBaseUrl?: string;
  connected: boolean;
  lastError?: string;
  meta?: MetaInfo;
  diagnostics?: {
    configDir: string;
    dataDir: string;
    logsDir: string;
    desktopLog: string;
    sidecarLog: string;
  };
};

export type DesktopChannelSettings = {
  configPath: string;
  imessageEnabled: boolean;
  feishuEnabled: boolean;
  feishuAppId?: string;
  feishuAppSecret?: string;
  telegramEnabled: boolean;
  telegramBotToken?: string;
  discordEnabled: boolean;
  discordBotToken?: string;
};

/** agent.runner 可选执行器 */
export type AgentProvider =
  | "function_calling"
  | "gemini_cli"
  | "gemini_acp"
  | "codex_cli"
  | "codex_acp"
  | "opencode_acp"
  | "multi-agent";

export type MultiAgentSearchSettings = {
  baseUrl: string;
  apiKey: string;
  model: string;
  maxIterations: number;
};

export type MultiAgentAnswerSettings = {
  baseUrl: string;
  apiKey: string;
  model: string;
  variant: string;
  maxToolCalls: number;
};

export type MultiAgentSettings = {
  search: MultiAgentSearchSettings;
  answer: MultiAgentAnswerSettings;
};

export type AuxiliarySettings = {
  baseUrl: string;
  apiKey: string;
  model: string;
};

export type AgentSettings = {
  /** function_calling | gemini_cli | gemini_acp | codex_cli | codex_acp | opencode_acp */
  runner: AgentProvider;
  /** codex_cli 专用；其他 provider 留空 */
  codexModel: string;
  /** OpenAI 协议渠道 Base URL（agent.opencode.api_base_url） */
  openaiUrl: string;
  /** OpenAI 协议渠道模型名（agent.opencode.model） */
  openaiModel: string;
  /** OpenAI 协议渠道 API Key（agent.opencode.api_key） */
  openaiApiKey: string;
  /** OpenAI-compatible auxiliary 配置，用于心跳/压缩等后台任务 */
  auxiliary?: AuxiliarySettings;
  /** multi-agent 双阶段设置 */
  multiAgent?: MultiAgentSettings;
};

export type AgentSettingsUpdateResult = {
  settings: AgentSettings;
  restartedBundledBackend: boolean;
  message: string;
  backendStatus?: BackendStatusInfo;
};

export type CliCheckResult = {
  ok: boolean;
  message: string;
};

/** OpenRouter API Key 设置（保存在运行时覆盖层的 llm.openrouter.api_keys，支持多 Key fallback） */
export type OpenRouterSettings = {
  /** 多 Key 列表，按顺序 fallback */
  apiKeys: string[];
};

/** FMP API Key 设置（保存在运行时覆盖层的 fmp.api_keys，支持多 Key fallback） */
export type FmpSettings = {
  /** 多 Key 列表，按顺序 fallback */
  apiKeys: string[];
};

/** Tavily API Key 设置（保存在运行时覆盖层的 search.api_keys，支持多 Key fallback） */
export type TavilySettings = {
  /** 多 Key 列表，按顺序 fallback */
  apiKeys: string[];
};

export type DesktopChannelSettingsInput = Omit<
  DesktopChannelSettings,
  "configPath"
>;

export type DesktopChannelSettingsUpdateResult = {
  settings: DesktopChannelSettings;
  restartedBundledBackend: boolean;
  message: string;
  backendStatus?: BackendStatusInfo;
};

export type ChannelProcessCleanupEntry = {
  channel: string;
  keptPid?: number;
  removedPids: number[];
};

export type ChannelProcessCleanupResult = {
  entries: ChannelProcessCleanupEntry[];
  message: string;
};

export type ChannelStatusInfo = {
  id: string;
  label: string;
  enabled: boolean;
  running: boolean;
  status: "running" | "disabled" | "stopped" | "unsupported" | string;
  pid?: number;
  last_heartbeat_at?: string;
  detail: string;
  processes: ChannelProcessInfo[];
};

export type ChannelProcessInfo = {
  pid: number;
  running: boolean;
  started_at?: string;
  last_heartbeat_at?: string;
  managed_by_desktop?: boolean;
  source?: string;
};

export type ChatStreamEvent =
  | { event: "run_started"; data: { runner?: string; text?: string } }
  | { event: "assistant_delta"; data: { content?: string } }
  | {
      event: "tool_call";
      data: {
        tool?: string;
        status?: string;
        text?: string;
        reasoning?: string;
      };
    }
  | { event: "run_error"; data: { message?: string } }
  | { event: "run_finished"; data: { success?: boolean } }
  /** actor 创建失败等路径（chat.rs 早期返回） */
  | { event: "error"; data: { text?: string } }
  /** 流结束标记（与 run_finished 二选一出现） */
  | { event: "done"; data: Record<string, unknown> };

/** 消息处理阶段 */
export type PendingPhase =
  | "queued" // 已发出请求，等待后端确认
  | "thinking" // run_started 到达，AI 正在思考
  | "running" // tool_call 到达，正在调用工具
  | "streaming" // assistant_delta 到达，流式输出中
  | "error" // 发生错误
  | "timeout"; // 请求超时

/** 每个会话独立的消息处理状态（替代全局 thinking/sending/thinkingText） */
export type PendingState = {
  id: string;
  startedAt: number; // Date.now()，用于计算已运行时长
  phase: PendingPhase;
  statusText: string; // "正在思考…" / "调用工具: web_search" / 错误原因
  partialContent: string; // 流式累积的 assistant 文本
};

export type PushScheduledMessageEvent = {
  text?: string;
  job_name?: string;
  job_id?: string;
};

export type TimelineMessage =
  | {
      id: string;
      kind: "user";
      content: string;
      subtype?: string;
      synthetic?: boolean;
      transcriptOnly?: boolean;
    }
  | {
      id: string;
      kind: "assistant";
      content: string;
      subtype?: string;
      synthetic?: boolean;
      transcriptOnly?: boolean;
    }
  | {
      id: string;
      kind: "system";
      content: string;
      subtype?: string;
      synthetic?: boolean;
      transcriptOnly?: boolean;
    }
  | {
      id: string;
      kind: "scheduled";
      content: string;
      jobName?: string;
      synthetic?: boolean;
      transcriptOnly?: boolean;
    };

export type CronJobInfo = {
  id: string;
  channel: string;
  user_id: string;
  channel_scope?: string;
  name: string;
  task_prompt: string;
  schedule: {
    hour: number;
    minute: number;
    repeat: string;
    weekday?: number;
  };
  tags?: string[];
  push?: Record<string, unknown>;
  enabled: boolean;
  channel_target: string;
  created_at: string;
  updated_at: string;
  last_run_at?: string;
  next_run_at?: string;
};

export type CronJobExecutionInfo = {
  run_id: number;
  job_id: string;
  job_name: string;
  channel: string;
  user_id: string;
  channel_scope?: string;
  channel_target: string;
  heartbeat: boolean;
  executed_at: string;
  execution_status: string;
  message_send_status: string;
  should_deliver: boolean;
  delivered: boolean;
  response_preview?: string;
  error_message?: string;
  detail?: Record<string, unknown> | null;
};

export type CronJobDetailInfo = {
  job: CronJobInfo;
  executions: CronJobExecutionInfo[];
};

export type CronJobUpsertInput = {
  channel?: string;
  user_id?: string;
  channel_scope?: string;
  name?: string;
  hour?: number;
  minute?: number;
  repeat?: string;
  weekday?: number;
  task_prompt?: string;
  push?: Record<string, unknown>;
  enabled?: boolean;
  channel_target?: string;
  tags?: string[];
};

export type HoldingInfo = {
  symbol: string;
  asset_type?: string;
  shares: number;
  avg_cost: number;
  underlying?: string;
  option_type?: string;
  strike_price?: number;
  expiration_date?: string;
  contract_multiplier?: number;
  holding_horizon?: "long_term" | "short_term";
  strategy_notes?: string;
  notes?: string;
};

export type PortfolioInfo = {
  actor?: {
    channel: string;
    user_id: string;
    channel_scope?: string;
  };
  user_id: string;
  holdings: HoldingInfo[];
  updated_at: string;
} | null;

export type PortfolioSummary = {
  channel: string;
  user_id: string;
  channel_scope?: string;
  holdings_count: number;
  total_shares: number;
  updated_at?: string;
};

export type HoldingUpsertInput = {
  channel?: string;
  user_id?: string;
  channel_scope?: string;
  symbol?: string;
  asset_type?: string;
  shares?: number;
  quantity?: number;
  avg_cost?: number;
  cost_basis?: number;
  underlying?: string;
  option_type?: string;
  strike_price?: number;
  expiration_date?: string;
  contract_multiplier?: number;
  holding_horizon?: "long_term" | "short_term" | "";
  strategy_notes?: string;
  notes?: string;
};

// ── 日志 ─────────────────────────────────────────────────────────────────────

export type LogEntry = {
  timestamp: string;
  level: string;
  target: string;
  message: string;
  file?: string;
  line?: number;
  message_id?: string;
  state?: string;
  extra?: Record<string, unknown>;
};

// ── 个股深度研究 ─────────────────────────────────────────────────────────────

export type ResearchTaskStatus = "pending" | "running" | "completed" | "error";

export type ResearchTask = {
  /** 本地生成的唯一 ID（用于前端列表 key） */
  id: string;
  /** 外部 API 返回的 task_id */
  task_id: string;
  /** 外部 API 返回的 task_name */
  task_name: string;
  /** 用户输入的公司名称 */
  company_name: string;
  /** 当前任务状态 */
  status: ResearchTaskStatus;
  /** 进度字符串，例如 "60%"、"100%" */
  progress: string;
  /** 任务创建时间（ISO 字符串） */
  created_at: string;
  /** 最近更新时间 */
  updated_at?: string;
  /** 完成时间 */
  completed_at?: string;
  /** 研究结果 Markdown 文件的绝对路径（仅供参考，不再用于读取内容） */
  answer_file_path?: string;
  /** 研究报告 Markdown 原文（轮询完成时从 API 直接获取，本地持久化） */
  answer_markdown?: string;
  /** 错误信息 */
  error_message?: string;
};

// ── LLM Audit ────────────────────────────────────────────────────────────────

export type AuditRecordSummary = {
  id: string;
  created_at: string;
  session_id: string;
  actor_channel?: string;
  actor_user_id?: string;
  actor_scope?: string;
  source: string;
  operation: string;
  provider: string;
  model?: string;
  success: boolean;
  latency_ms?: number;
  prompt_tokens?: number;
  completion_tokens?: number;
  total_tokens?: number;
};

export type LlmAuditRecord = AuditRecordSummary & {
  request: unknown;
  response?: unknown;
  error?: string;
  metadata: unknown;
};

export type AuditQueryFilter = {
  actor_channel?: string;
  actor_user_id?: string;
  actor_scope?: string;
  session_id?: string;
  success?: boolean;
  source?: string;
  provider?: string;
  date_from?: string;
  date_to?: string;
  page?: number;
  page_size?: number;
};

// ── Company Profiles ────────────────────────────────────────────────────────

export type IndustryTemplate =
  | "general"
  | "saas"
  | "semiconductor_hardware"
  | "consumer"
  | "industrial_defense"
  | "financials";

export type ProfileTrackingConfig = {
  enabled: boolean;
  cadence: string;
  focus_metrics: string[];
};

export type ProfileMetadata = {
  company_name: string;
  stock_code: string;
  aliases: string[];
  sector: string;
  industry_template: IndustryTemplate;
  status: string;
  tracking: ProfileTrackingConfig;
  created_at: string;
  updated_at: string;
  last_reviewed_at?: string;
};

export type CompanyProfileEvent = {
  id: string;
  filename: string;
  title: string;
  updated_at?: string;
  markdown: string;
};

export type CompanyProfile = {
  profile_id: string;
  title: string;
  updated_at?: string;
  markdown: string;
  events: CompanyProfileEvent[];
};

export type CompanyProfileSummary = {
  profile_id: string;
  title: string;
  updated_at?: string;
  event_count: number;
};

export type CompanyProfileSpaceSummary = {
  channel: string;
  user_id: string;
  channel_scope?: string;
  profile_count: number;
  updated_at?: string;
};

export type CompanyProfileCreateInput = {
  channel?: string;
  user_id?: string;
  channel_scope?: string;
  company_name: string;
  stock_code?: string;
  sector?: string;
  aliases?: string[];
  industry_template?: IndustryTemplate;
  sections?: Record<string, string>;
};

export type CompanyProfileConflictDecision = "skip" | "replace";

export type CompanyProfileImportMode =
  | "keep_existing"
  | "replace_all"
  | "interactive";

export type CompanyProfileImportProfileSummary = {
  profile_id: string;
  company_name: string;
  stock_code: string;
  updated_at: string;
  event_count: number;
  thesis_excerpt: string;
};

export type CompanyProfileImportConflict = {
  imported: CompanyProfileImportProfileSummary;
  existing: CompanyProfileImportProfileSummary;
  reasons: string[];
};

export type CompanyProfileTransferManifestProfile = {
  profile_id: string;
  company_name: string;
  stock_code: string;
  event_count: number;
  updated_at: string;
};

export type CompanyProfileTransferManifest = {
  version: string;
  exported_at: string;
  profile_count: number;
  event_count: number;
  profiles: CompanyProfileTransferManifestProfile[];
};

export type CompanyProfileImportPreview = {
  manifest: CompanyProfileTransferManifest;
  profiles: CompanyProfileImportProfileSummary[];
  conflicts: CompanyProfileImportConflict[];
  importable_count: number;
  conflict_count: number;
  suggested_mode: CompanyProfileImportMode;
};

export type CompanyProfileImportApplyRequest = {
  mode: CompanyProfileImportMode;
  decisions: Record<string, CompanyProfileConflictDecision>;
};

export type CompanyProfileImportApplyResult = {
  imported_profile_ids: string[];
  replaced_profile_ids: string[];
  skipped_profile_ids: string[];
  changed_profile_ids: string[];
  imported_count: number;
  replaced_count: number;
  skipped_count: number;
};
