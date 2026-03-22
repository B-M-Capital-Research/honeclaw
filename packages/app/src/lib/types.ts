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
  aliases: string[];
  tools: string[];
  guide: string;
};

export type HistoryMsg = {
  role: "user" | "assistant";
  content: string;
};

export type MetaInfo = {
  name: string;
  version: string;
  channel: string;
  supports_imessage: boolean;
  api_version: string;
  capabilities: string[];
  deployment_mode: "local" | "remote";
};

export type BackendConfig = {
  mode: "bundled" | "remote";
  baseUrl: string;
  bearerToken: string;
};

export type BackendStatusInfo = {
  config: BackendConfig;
  resolved_base_url?: string;
  connected: boolean;
  last_error?: string;
  meta?: MetaInfo;
  diagnostics?: {
    config_dir: string;
    data_dir: string;
    logs_dir: string;
    desktop_log: string;
    sidecar_log: string;
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
  | "opencode_acp";

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

export type ChannelStatusInfo = {
  id: string;
  label: string;
  enabled: boolean;
  running: boolean;
  status: "running" | "disabled" | "stopped" | "unsupported" | string;
  pid?: number;
  last_heartbeat_at?: string;
  detail: string;
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
  | "queued"     // 已发出请求，等待后端确认
  | "thinking"   // run_started 到达，AI 正在思考
  | "running"    // tool_call 到达，正在调用工具
  | "streaming"  // assistant_delta 到达，流式输出中
  | "error"      // 发生错误
  | "timeout";   // 请求超时

/** 每个会话独立的消息处理状态（替代全局 thinking/sending/thinkingText） */
export type PendingState = {
  id: string;
  startedAt: number;      // Date.now()，用于计算已运行时长
  phase: PendingPhase;
  statusText: string;     // "正在思考…" / "调用工具: web_search" / 错误原因
  partialContent: string; // 流式累积的 assistant 文本
};

export type PushScheduledMessageEvent = {
  text?: string;
  job_name?: string;
  job_id?: string;
};

export type TimelineMessage =
  | { id: string; kind: "user"; content: string }
  | { id: string; kind: "assistant"; content: string }
  | { id: string; kind: "system"; content: string }
  | { id: string; kind: "scheduled"; content: string; jobName?: string };

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
  push?: Record<string, unknown>;
  enabled: boolean;
  channel_target: string;
  created_at: string;
  updated_at: string;
  last_run_at?: string;
  next_run_at?: string;
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

// ── Knowledge Base ────────────────────────────────────────────────────────────

export type KbEntry = {
  id: string;
  filename: string;
  /** 附件分类标签，例如 "Pdf" / "Image" / "Text" */
  kind: string;
  size: number;
  content_type?: string;
  channel: string;
  user_id: string;
  session_id: string;
  uploaded_at: string;
  original_path: string;
  parsed_path: string;
  /** "ok" | "failed" | "empty" | "skipped" */
  parse_status: string;
  parse_error?: string;
  /** 最近一次成功同步到知识表的时间（ISO 8601），undefined 表示从未同步 */
  analyzed_at?: string;
};

export type RelatedFileRef = {
  kb_id: string;
  filename: string;
  summary: string;
};

export type StockRow = {
  company_name: string;
  stock_code: string;
  related_files: RelatedFileRef[];
  /** 用户/AI 手动录入的重点知识条目 */
  key_knowledge: string[];
  updated_at: string;
};
