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
  /** RFC3339 时间戳，用于聊天记录分组与日期分隔。 */
  at?: string;
  subtype?:
    "compact_boundary" | "compact_summary" | "compact_skill_snapshot" | string;
  synthetic?: boolean;
  transcript_only?: boolean;
  attachments: HistoryAttachment[];
  scheduled_push?: HistoryScheduledPush;
  finance_calendar?: HistoryFinanceCalendar;
};

export type HistoryFinanceCalendar = {
  month: string;
  image_path: string;
  variant: "desktop" | "mobile" | string;
};

export type PublicHistoryPageResponse = {
  messages: HistoryMsg[];
  history_start: number;
  next_before?: number | null;
};

/** Server-authoritative state for one public-chat run that is still active. */
export type PublicChatActiveRun = {
  run_id: string;
  started_at_ms: number;
  phase: "thinking" | "running";
  status_text: string;
  updated_at_ms: number;
  /** Stages this run already passed through, newest last. */
  steps?: string[];
};

export type PublicChatBootstrapResponse = PublicHistoryPageResponse & {
  user: PublicAuthUserInfo;
  active_run?: PublicChatActiveRun | null;
  interrupted_run?: boolean;
};

export type HistoryScheduledPush = {
  push_id?: string;
  title: string;
  summary: string;
  fallback_content?: string;
};

export type PublicPushListItem = {
  push_id: string;
  job_id: string;
  title: string;
  summary: string;
  created_at: string;
};

export type PublicPushDetail = PublicPushListItem & {
  content: string;
};

export type PublicPushListResponse = {
  items: PublicPushListItem[];
  unread_count: number;
  next_before?: string | null;
};

export type PublicPushOpenResponse = {
  push: PublicPushDetail;
  unread_count: number;
};

export type PublicCommunityResource = {
  resource_id: number;
  ordinal: number;
  resource_kind: "image" | "file" | string;
  display_name?: string | null;
  content_type?: string | null;
  byte_size?: number | null;
  version?: string | null;
  /** Same-origin private edge path; legacy API responses intentionally omit it. */
  delivery_path?: string | null;
  access_state: "stored" | "protected_in_app" | "metadata_only" | string;
};

export type PublicCommunityContent = {
  content_id: number;
  author_name: string;
  published_at?: string | null;
  published_at_raw?: string | null;
  content_type: string;
  body_text: string;
  body_blocks: unknown[];
  crawl_status: "complete" | "partial" | string;
  resources: PublicCommunityResource[];
};

export type PublicCommunityPage = {
  community: { id: string; name: string };
  items: PublicCommunityContent[];
  next_before?: number | null;
  unread: boolean;
  latest_content_id?: number | null;
};

export type CommunityForumAttachment = {
  id: string;
  filename: string;
  content_type: string;
  byte_size: number;
  sha256: string;
};

export type CommunityForumComment = {
  id: string;
  author_label: string;
  body: string;
  created_at: string;
  moderation_status: "visible" | "deleted" | string;
  can_delete: boolean;
};

export type CommunityForumPost = {
  id: string;
  author_label: string;
  title: string;
  body: string;
  tickers: string[];
  topics: string[];
  source_url?: string | null;
  created_at: string;
  updated_at: string;
  moderation_status: "visible" | "pending_review" | "hidden" | "deleted" | string;
  attachment?: CommunityForumAttachment | null;
  like_count: number;
  liked_by_me: boolean;
  report_count?: number | null;
  can_delete: boolean;
  comments: CommunityForumComment[];
};

export type CommunityForumPage = {
  items: CommunityForumPost[];
  is_admin: boolean;
  policy: {
    forum_content_is_research: false;
    attachment_max_bytes: number;
    auto_hide_report_count: number;
  };
};

export type HistoryAttachment = {
  path: string;
  name: string;
  kind: "image" | "pdf" | "file" | string;
};

export type FinanceCalendarEvent = {
  date: string;
  title: string;
  kind: "macro" | "earnings" | string;
  ticker?: string;
  subtitle?: string;
  source: string;
};

export type FinanceCalendarMonth = {
  value: string;
  label: string;
};

export type FinanceCalendarPayload = {
  today: string;
  month: string;
  months: FinanceCalendarMonth[];
  holdings: string[];
  events: FinanceCalendarEvent[];
  earnings_status: string;
  errors: string[];
};

export type WebInviteInfo = {
  user_id: string;
  invite_code: string;
  phone_number: string;
  created_at: string;
  last_login_at?: string;
  revoked_at?: string;
  api_key_prefix?: string;
  api_key_created_at?: string;
  api_key_last_used_at?: string;
  api_key?: string;
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
  has_password: boolean;
  tos_accepted_at?: string;
  tos_version?: string;
  identity_kind: "domestic_invite" | "international_email" | string;
  email_hint?: string;
  billing: PublicBillingSummary;
  is_admin: boolean;
};

export type PublicBillingEntitlement = {
  entitlement_id: string;
  provider: "stripe" | "domestic_invite" | string;
  entitlement_kind:
    | "recurring_subscription"
    | "fixed_term_purchase"
    | "domestic_invite"
    | string;
  raw_status: string;
  access_state: "pending" | "active" | "grace" | "inactive" | string;
  grants_access: boolean;
  current_period_start?: string;
  current_period_end?: string;
  cancel_at_period_end: boolean;
  manage_url?: string;
  grace_expires_at?: string;
};

export type PublicBillingSummary = {
  access_granted: boolean;
  entitlements: PublicBillingEntitlement[];
  has_duplicate_active_subscriptions: boolean;
};

export type PublicBillingConfig = {
  stripe: {
    subscription: PublicBillingOfferConfig;
    fixed_term: PublicBillingOfferConfig;
  };
  purchases_allowed_on_this_client: boolean;
  management_allowed_on_this_client: boolean;
};

export type PublicBillingOfferConfig = {
  enabled: boolean;
  amount_minor: number;
  currency: string;
  term_months: number;
  auto_renews: boolean;
  advertised_payment_methods: {
    card: boolean;
    alipay: boolean;
    wechat_pay: boolean;
  };
};

export type PublicBillingStatus = {
  billing: PublicBillingSummary;
  config: PublicBillingConfig;
};

export type PublicAdminInviteInfo = {
  user_id: string;
  phone_number: string;
  created_at: string;
  last_login_at?: string;
  enabled: boolean;
  can_disable: boolean;
};

export type PublicAdminInviteList = {
  invites: PublicAdminInviteInfo[];
  daily_create_limit: number;
  created_today: number;
  remaining_today: number;
};

export type PublicAdminInviteMutation = {
  invite: PublicAdminInviteInfo;
  daily_create_limit: number;
  created_today: number;
  remaining_today: number;
  cleared_session_count: number;
  message: string;
};

export type PublicAdminUsageQuestion = {
  asked_at: string;
  text: string;
};

export type PublicAdminUsageRow = {
  date: string;
  channel: "web" | "feishu" | "telegram" | "discord" | "imessage" | string;
  user_id: string;
  user_label: string;
  question_count: number;
  questions: PublicAdminUsageQuestion[];
  scheduled_run_count: number;
  delivered_push_count: number;
  failed_delivery_count: number;
  latest_activity_at: string;
};

export type PublicAdminUsageReport = {
  generated_at: string;
  period_days: number;
  period_start: string;
  period_end: string;
  summary: {
    today: string;
    today_active_users: number;
    today_question_count: number;
    today_delivered_push_count: number;
    last_week_same_day_active_users: number;
    active_user_change: number;
    leading_decline_user_label?: string;
    leading_decline_question_delta: number;
    text: string;
  };
  rows: PublicAdminUsageRow[];
};

export type MetaInfo = {
  name: string;
  version: string;
  channel: string;
  supportsImessage: boolean;
  apiVersion: string;
  capabilities: string[];
  deploymentMode: "local" | "remote";
  /** Admin/console default UI language. "zh" or "en". Optional for backwards compat with older backends. */
  language?: "zh" | "en";
  build?: {
    git_sha?: string | null;
    build_timestamp?: string | null;
    profile: "debug" | "release" | string;
    source?: "workspace" | "direct_source_runtime" | "unknown";
    binary_sha256?: string | null;
  };
  acp_profiles?: Array<{
    runner: string;
    adapter: string;
    detected_version: string;
    baseline_version: string;
    dialect: string;
    compatibility: "validated" | "compatible_newer" | string;
    companion_versions: Record<string, string>;
    detected_at: string;
    build_git_sha?: string | null;
  }>;
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
  imessageTargetHandle?: string;
  feishuEnabled: boolean;
  feishuAppId?: string;
  feishuAppSecret?: string;
  feishuChatScope?: string;
  feishuAllowEmails?: string[];
  feishuAllowMobiles?: string[];
  feishuAllowOpenIds?: string[];
  telegramEnabled: boolean;
  telegramBotToken?: string;
  telegramChatScope?: string;
  telegramAllowFrom?: string[];
  discordEnabled: boolean;
  discordBotToken?: string;
  discordChatScope?: string;
  discordAllowFrom?: string[];
};

/** agent.runner values accepted by config/admin surfaces; gemini_acp is legacy/parseable but disabled at runtime. */
export type AgentProvider =
  | "gemini_cli"
  | "gemini_acp"
  | "codex_cli"
  | "codex_acp"
  | "opencode_acp"
  | "hone_cloud";

export type AuxiliarySettings = {
  baseUrl: string;
  apiKey: string;
  model: string;
};

export type HoneCloudSettings = {
  baseUrl: string;
  apiKey: string;
  model: string;
};

export type LlmProfileEntrySettings = {
  id: string;
  provider: string;
  model: string;
  maxTokens?: number;
  temperature?: number;
  topP?: number;
  reasoningEffort?: string;
  reasoningMaxTokens?: number;
  responseFormatJson: boolean;
};

export type LlmProfileSettings = {
  defaultProfile: string;
  auxiliaryProfile: string;
  polishProfile: string;
  newsClassifierProfile: string;
  filingSummaryProfile: string;
  earningsQualityProfile: string;
  digestPass1Profile: string;
  digestPass2Profile: string;
  digestEventDedupeProfile: string;
  mainlineDistillProfile: string;
  profiles: LlmProfileEntrySettings[];
};

export type AgentSettings = {
  /** AgentProvider value; gemini_acp is legacy/disabled at runtime. */
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
  /** HONE Cloud 用户端服务配置 */
  honeCloud?: HoneCloudSettings;
  /** Named LLM profiles used by runtime subsystems */
  llmProfiles?: LlmProfileSettings;
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

/** FMP API Key 设置（保存在 canonical config.yaml 的 fmp.api_keys，支持多 Key fallback） */
export type FmpSettings = {
  /** 多 Key 列表，按顺序 fallback */
  apiKeys: string[];
};

/** Tavily API Key 设置（保存在 canonical config.yaml 的 search.api_keys，支持多 Key fallback） */
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
  | {
      event: "run_started";
      data: Partial<PublicChatActiveRun> & { runner?: string; text?: string };
    }
  | {
      event: "run_progress";
      data: Partial<PublicChatActiveRun> & { text?: string };
    }
  | { event: "assistant_delta"; data: { content?: string } }
  | { event: "reasoning_delta"; data: { content?: string } }
  | { event: "assistant_reset"; data: Record<string, never> }
  | {
      event: "tool_call";
      data: {
        public_status_text?: string;
        tool?: string;
        status?: string;
        text?: string;
        reasoning?: string;
      };
    }
  | { event: "run_error"; data: { message?: string } }
  | {
      event: "run_finished";
      data: { success?: boolean; partial?: boolean };
    }
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

export type TimelineMessage =
  | {
      id: string;
      kind: "user";
      content: string;
      at?: string;
      subtype?: string;
      synthetic?: boolean;
      transcriptOnly?: boolean;
      attachments?: HistoryAttachment[];
      scheduledPush?: HistoryScheduledPush;
      financeCalendar?: HistoryFinanceCalendar;
    }
  | {
      id: string;
      kind: "assistant";
      content: string;
      at?: string;
      subtype?: string;
      synthetic?: boolean;
      transcriptOnly?: boolean;
      attachments?: HistoryAttachment[];
      scheduledPush?: HistoryScheduledPush;
      financeCalendar?: HistoryFinanceCalendar;
    }
  | {
      id: string;
      kind: "system";
      content: string;
      at?: string;
      subtype?: string;
      synthetic?: boolean;
      transcriptOnly?: boolean;
      attachments?: HistoryAttachment[];
    }
  | {
      id: string;
      kind: "scheduled";
      content: string;
      jobName?: string;
      synthetic?: boolean;
      transcriptOnly?: boolean;
      attachments?: HistoryAttachment[];
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
  tracking_only?: boolean;
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
  watchlist_count?: number;
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
  tracking_only?: boolean;
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

export type CompanyProfileConflictDecision = "skip" | "replace";

export type CompanyProfileImportMode =
  "keep_existing" | "replace_all" | "interactive";

export type CompanyProfileImportProfileSummary = {
  profile_id: string;
  company_name: string;
  stock_code: string;
  updated_at: string;
  event_count: number;
  mainline_excerpt: string;
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

/** One of the caller's own scheduled pushes, as the push page shows it. */
export type PublicSubscription = {
  job_id: string;
  name: string;
  task_prompt: string;
  enabled: boolean;
  channel: string;
  schedule: {
    hour: number;
    minute: number;
    repeat: string;
    weekday?: number | null;
    date?: string | null;
  };
  created_at?: string | null;
  last_run_at?: string | null;
};

// ── Daily company ratings ──────────────────────────────────────────────────

export type CompanyRatingLight = "green" | "yellow" | "red" | "unknown";
export type CompanyRatingDataStatus =
  "live" | "partial" | "transcript_only" | "stale" | "simulation";

export type CompanyRatingDimensions = {
  moat: number;
  scarcity: number;
  fundamentals: number;
  visibility: number;
  growth_quality?: number | null;
  pricing_power?: number | null;
  financial_quality?: number | null;
  valuation?: number | null;
  market_confirmation: number;
  timing?: number | null;
};

export type CompanyRatingMetrics = {
  revenue_growth_percent?: number | null;
  forward_revenue_growth_percent?: number | null;
  gross_margin_percent?: number | null;
  gross_margin_change_pp?: number | null;
  ebit_margin_percent?: number | null;
  fcf_margin_percent?: number | null;
  net_cash_to_revenue_percent?: number | null;
  financial_as_of?: string | null;
  forward_metric_label?: string | null;
  forward_metric_value?: string | null;
  forward_metric_growth_percent?: number | null;
  forward_metric_as_of?: string | null;
  forward_metric_source_url?: string | null;
};

export type CompanyDailyValuation = {
  as_of: string;
  generated_at_local: string;
  currency: string;
  bear_case: number;
  base_case: number;
  bull_case: number;
  current_price: number;
  probability_weighted_value: number;
  expected_upside_percent: number;
  method_count: number;
  confidence: "high" | "medium" | "low" | string;
  current_position: string;
  position_percent: number;
  method: string;
  assumptions: string[];
  sources: string[];
};

export type CompanyRating = {
  name: string;
  symbol: string;
  market_scope: string;
  theme: string;
  value_chain: string;
  score: number;
  light: CompanyRatingLight;
  confidence: "high" | "medium" | "low";
  data_status: CompanyRatingDataStatus;
  price?: number | null;
  change_percent?: number | null;
  market_as_of?: string | null;
  financial_as_of?: string | null;
  thesis_summary: string;
  business_model: string;
  moat: string;
  valuation_method: string;
  valuation?: CompanyDailyValuation | null;
  valuation_unavailable_reason: string;
  dimensions: CompanyRatingDimensions;
  metrics?: CompanyRatingMetrics;
  score_cap_reason?: string;
  factor_coverage?: number;
  watch_items: string[];
  risks: string[];
  falsifiers: string[];
  research_updated_at: string;
  data_sources: string[];
};

export type CompanyRatingSnapshot = {
  generated_at: string;
  generated_at_local: string;
  next_refresh_at: string;
  timezone: string;
  data_status: CompanyRatingDataStatus;
  methodology_version: string;
  simulation_note?: string;
  coverage: {
    companies: number;
    quotes: number;
    financials: number;
    valuations: number;
  };
  disclaimer: string;
  items: CompanyRating[];
};

// ── Daily valuation lab ──────────────────────────────────────────────────

export type ValuationEvidence = {
  label: string;
  display_value: string;
  as_of: string;
  source: string;
  source_url: string;
};

export type ValuationScenario = {
  id: "bear" | "base" | "bull" | string;
  label: string;
  probability: number;
  initial_growth_rate: number;
  discount_rate: number;
  terminal_growth_rate: number;
  dcf_value?: number | null;
  multiple_value?: number | null;
  methods: Array<{
    id: string;
    label: string;
    value: number;
    weight: number;
    metric: string;
    assumption: string;
  }>;
  fair_value: number;
};

export type ValuationLabItem = {
  symbol: string;
  name: string;
  market_scope: string;
  theme: string;
  status: "ready" | "review_required" | "unavailable" | "stale" | string;
  confidence: "high" | "medium" | "low" | string;
  eligible_for_rating: boolean;
  unavailable_reason: string;
  currency: string;
  current_price?: number | null;
  market_as_of?: string | null;
  financial_as_of?: string | null;
  normalized_fcf?: number | null;
  normalized_fcf_per_share?: number | null;
  net_cash_per_share?: number | null;
  historical_fcf_growth_rate?: number | null;
  current_position: string;
  position_percent?: number | null;
  method: string;
  valuation_profile: string;
  company_method_hint: string;
  scenarios: ValuationScenario[];
  probability_weighted_value?: number | null;
  expected_upside_percent?: number | null;
  reverse_dcf?: {
    status: string;
    implied_growth_rate?: number | null;
    implied_forward_eps?: number | null;
    implied_forward_pe?: number | null;
    explanation: string;
  } | null;
  cross_check?: {
    status: string;
    method_count: number;
    dispersion_percent?: number | null;
    forward_eps?: number | null;
    forward_pe?: number | null;
    pe_value?: number | null;
    dcf_value?: number | null;
    gap_percent?: number | null;
    explanation: string;
  } | null;
  assumptions: string[];
  evidence: ValuationEvidence[];
};

export type ValuationLabSnapshot = {
  report_date: string;
  generated_at: string;
  generated_at_local: string;
  next_refresh_at: string;
  timezone: string;
  methodology_version: string;
  status: "live" | "partial" | "data_unavailable" | "stale" | string;
  coverage: {
    companies: number;
    calculated: number;
    cross_checked: number;
    eligible_for_rating: number;
  };
  summary: string;
  methodology_note: string;
  items: ValuationLabItem[];
  disclaimer: string;
};

// ── Daily actor-scoped portfolio news ─────────────────────────────────────

export type PortfolioNewsImpact =
  "positive" | "neutral" | "negative" | "unassessed";

export type PortfolioNewsItem = {
  id: string;
  symbol: string;
  title: string;
  published_at: string;
  published_at_local: string;
  source: string;
  source_url: string;
  source_summary: string;
  severity: "high" | "medium" | "low";
  impact: PortfolioNewsImpact;
  horizon: "short" | "medium" | "long" | "unknown";
  thesis_effect: "strengthens" | "unchanged" | "weakens" | "unassessed";
  summary: string;
  why_it_matters: string;
  attention: "立即复核" | "持续观察" | "无需动作";
  confidence: "high" | "medium" | "low";
  analysis_status: "model_analyzed" | "source_only";
  priority_score: number;
};

export type PortfolioNewsSnapshot = {
  report_date: string;
  generated_at: string;
  generated_at_local: string;
  next_refresh_at: string;
  timezone: string;
  model_version: string;
  status:
    | "live"
    | "partial"
    | "source_only"
    | "no_material_news"
    | "data_unavailable"
    | "no_portfolio"
    | "waiting_refresh"
    | "portfolio_changed"
    | "stale"
    | string;
  source_status: string;
  model_status: string;
  portfolio_updated_at: string;
  holdings_count: number;
  lookback_hours: number;
  covered_symbols: string[];
  missing_symbols: string[];
  summary: string;
  counts: {
    total: number;
    positive: number;
    neutral: number;
    negative: number;
    immediate_review: number;
  };
  items: PortfolioNewsItem[];
  disclaimer: string;
};

// ── Daily actor-scoped position management ───────────────────────────────

export type PositionManagementAction =
  "increase_candidate" | "hold" | "reduce" | "review" | "insufficient_data";

export type PositionAdviceItem = {
  symbol: string;
  name: string;
  theme: string;
  weight: number;
  current_price?: number | null;
  avg_cost?: number | null;
  unrealized_return_percent?: number | null;
  rating_score?: number | null;
  rating_light: "green" | "yellow" | "red" | "unknown" | string;
  rating_status: string;
  valuation_position: string;
  news_impact: string;
  news_attention: string;
  action: PositionManagementAction;
  action_label: string;
  confidence: "high" | "medium" | "low" | string;
  rationale: string[];
  risks: string[];
  falsifiers: string[];
  framework_logic: string[];
  evidence_as_of: string[];
  evidence_sources: string[];
  priority_score: number;
};

export type PositionManagementSnapshot = {
  report_date: string;
  generated_at: string;
  generated_at_local: string;
  next_refresh_at: string;
  timezone: string;
  model_version: string;
  framework_version: string;
  status: string;
  portfolio_updated_at: string;
  holdings_count: number;
  total_weight: number;
  unallocated_weight: number;
  concentration: {
    level: "balanced" | "elevated" | "high" | "unknown" | string;
    largest_symbol: string;
    largest_weight: number;
    top_three_weight: number;
    theme_exposures: Array<{ theme: string; weight: number }>;
  };
  macro_context: {
    signal: string;
    score?: number | null;
    phase: string;
    report_date: string;
    status: string;
  };
  counts: Record<PositionManagementAction, number>;
  summary: string;
  items: PositionAdviceItem[];
  methodology_note: string;
  disclaimer: string;
};

export type InfluencerDigestSnapshot = {
  report_date: string;
  generated_at: string;
  generated_at_local: string;
  timezone: string;
  next_refresh_at: string;
  lookback_hours: number;
  model_version: string;
  status: string;
  summary: string;
  coverage: {
    authors: number;
    configured: number;
    succeeded: number;
    items: number;
    analyzed: number;
  };
  authors: Array<{
    id: string;
    name: string;
    public_handle: string;
    focus: string;
    configured: boolean;
    source_status: string;
    item_count: number;
    last_published_at?: string | null;
  }>;
  items: InfluencerDigestItem[];
  disclaimer: string;
};

export type InfluencerDigestItem = {
  id: string;
  author_id: string;
  author_name: string;
  public_handle: string;
  title: string;
  published_at: string;
  published_at_local: string;
  source_url: string;
  aggregation_source?: string | null;
  aggregation_url?: string | null;
  post_kind: string;
  /** Short form kept for compatibility; the panel renders the full text below. */
  source_excerpt: string;
  /** Untruncated author text. Chinese translation first, English original next. */
  source_text_cn?: string;
  source_text_en?: string;
  media_urls?: string[];
  reply_context?: { author: string; text: string } | null;
  metrics?: { views: number; likes: number };
  summary: string;
  stance: string;
  horizon: string;
  content_type: string;
  topics: string[];
  tickers: string[];
  counterpoint: string;
  analysis_status: string;
};

export type KeyEventChainSnapshot = {
  report_date: string;
  generated_at: string;
  generated_at_local: string;
  next_refresh_at: string;
  timezone: string;
  lookback_days: number;
  model_version: string;
  status: string;
  summary: string;
  topics: Array<{
    id: string;
    name: string;
    layer: string;
    description: string;
    first_principle: string;
    priority: number;
    status: string;
    event_count: number;
    confirmed_count: number;
    clue_count: number;
    last_event_at?: string | null;
    latest_change: string;
    events: Array<{
      id: string;
      topic_id: string;
      published_at: string;
      published_at_local: string;
      source_name: string;
      source_url: string;
      source_tier: string;
      verification_status: string;
      verification_note: string;
      title: string;
      excerpt: string;
      change_type: string;
      direction: string;
      impact: string;
      next_watch: string;
      tickers: string[];
      analysis_status: string;
    }>;
  }>;
  /** Legacy storage-only payload; no longer exposed by the public dashboard API. */
  ten_day_brief?: {
    review_start: string;
    review_end: string;
    outlook_start: string;
    outlook_end: string;
    previous_generated_at_local?: string | null;
    status: string;
    summary: string;
    version_summary: string;
    review: Array<{
      topic_id: string;
      topic_name: string;
      event_count: number;
      confirmed_count: number;
      clue_count: number;
      new_since_previous: number;
      direction_summary: string;
      latest_change: string;
      evidence_event_ids: string[];
    }>;
    questions: Array<{
      id: string;
      topic_id: string;
      topic_name: string;
      question: string;
      why_it_matters: string;
      status: string;
      review_by: string;
      evidence_event_ids: string[];
    }>;
    methodology_note: string;
  };
  disclaimer: string;
};

export type WeeklyBriefItem = {
  id: string;
  date: string;
  weekday: string;
  phase: "last_week" | "next_week" | "ai_outlook" | string;
  category:
    | "policy"
    | "inflation"
    | "labor"
    | "growth"
    | "macro"
    | "earnings"
    | "industry"
    | "ai_conference"
    | string;
  importance: "high" | "medium" | string;
  title: string;
  subtitle: string;
  ticker?: string | null;
  source_name: string;
  source_url?: string | null;
  evidence_status:
    | "confirmed"
    | "schedule_passed"
    | "scheduled"
    | "official_schedule"
    | string;
  evidence_note: string;
  analysis: string;
  attention: string;
};

export type WeeklyBriefPayload = {
  report_date: string;
  generated_at_local: string;
  timezone: string;
  status: "live" | "partial" | "empty" | string;
  summary: string;
  previous_week: { start: string; end: string; label: string };
  next_week: { start: string; end: string; label: string };
  ai_outlook: { start: string; end: string; label: string };
  last_week_items: WeeklyBriefItem[];
  next_week_items: WeeklyBriefItem[];
  ai_outlook_items: WeeklyBriefItem[];
  earnings_status: string;
  earnings_scope_count: number;
  holdings: string[];
  errors: string[];
  methodology_note: string;
  disclaimer: string;
};

// ── Unified research library ──────────────────────────────────────────────

export type ResearchLibraryUse = "chat" | "key_event_chain" | "portfolio_news";
export type ResearchLibraryScope =
  "personal" | "community_candidate" | "hone_global";
export type ResearchLibraryReviewStatus =
  "not_required" | "pending" | "approved" | "rejected";
export type ResearchLibrarySourceType =
  "manual_upload" | "zsxq_export" | "ima_export" | "authorized_connector";

export type ResearchLibraryItem = {
  id: string;
  scope: ResearchLibraryScope;
  submitted_by?: string | null;
  title: string;
  filename: string;
  content_type: string;
  size: number;
  sha256: string;
  source_type: ResearchLibrarySourceType;
  source_name: string;
  source_url?: string | null;
  source_date: string;
  uploaded_at: string;
  updated_at: string;
  parse_status: "ready" | "stored" | "error" | string;
  excerpt: string;
  tickers: string[];
  topics: string[];
  uses: ResearchLibraryUse[];
  review_status: ResearchLibraryReviewStatus;
  review_note?: string | null;
  reviewed_at?: string | null;
  download_url: string;
};

export type ResearchConnectorStatus = {
  status: "available_via_import" | string;
  mode: "official_skill_export" | "file_export" | string;
  read_only: boolean;
  automatic_sync: boolean;
  guide_url?: string;
  note: string;
};

export type ResearchLibraryBundle = {
  items: ResearchLibraryItem[];
  is_admin: boolean;
  connector_status: {
    zsxq: ResearchConnectorStatus;
    ima: ResearchConnectorStatus;
  };
};

// ── Daily macro / AI signals ──────────────────────────────────────────────

export type DailySignalKind = "macro" | "ai";
export type DailySignalLight =
  "green" | "yellow" | "orange" | "red" | "unknown";
export type DailySignalStatus =
  "live" | "partial" | "framework_only" | "stale" | string;

export type DailySignalTrendPoint = { period: string; value: number };
export type DailySignalEvidence = {
  label: string;
  value?: number | null;
  display_value: string;
  unit: string;
  period?: string | null;
  released_at?: string | null;
  fetched_at: string;
  source: string;
  source_url: string;
  provenance: "reported_fact" | "model_inference" | "unavailable" | string;
};

export type DailySignalDimension = {
  id: string;
  label: string;
  role: string;
  score?: number | null;
  signal: DailySignalLight;
  trend_label: string;
  reason: string;
  threshold: string;
  trend: DailySignalTrendPoint[];
  evidence: DailySignalEvidence[];
};

export type DailySignalMetric = {
  id: string;
  label: string;
  score?: number | null;
  display_value: string;
  reason: string;
};

export type DailySignalCompanyScore = {
  symbol: string;
  name: string;
  score?: number | null;
  signal: DailySignalLight;
  capex?: number | null;
  capex_growth?: number | null;
  capex_peak_status: string;
  coverage: number;
  metric_total: number;
  metrics: DailySignalMetric[];
};

export type DailyHardwareSignal = {
  symbol: string;
  segment: string;
  signal: DailySignalLight;
  score?: number | null;
  price?: number | null;
  change_percent?: number | null;
  reason: string;
};

export type DailySignalReport = {
  kind: DailySignalKind;
  title: string;
  report_date: string;
  market_date?: string | null;
  data_cutoff?: string | null;
  generated_at: string;
  generated_at_local: string;
  timezone: string;
  next_refresh_at: string;
  model_version: string;
  status: DailySignalStatus;
  score?: number | null;
  raw_score?: number | null;
  signal: DailySignalLight;
  phase: string;
  summary: string;
  comparison_yesterday?: number | null;
  comparison_week?: number | null;
  changes: { label: string; direction: string; detail: string }[];
  dimensions: DailySignalDimension[];
  company_scores: DailySignalCompanyScore[];
  hardware_signals: DailyHardwareSignal[];
  alerts: string[];
  evidence: DailySignalEvidence[];
  sources: { label: string; url: string; source_type: string }[];
  full_report: string;
  stale: boolean;
  disclaimer: string;
};

export type DailySignalHistoryItem = Pick<
  DailySignalReport,
  | "report_date"
  | "generated_at_local"
  | "status"
  | "score"
  | "raw_score"
  | "signal"
  | "phase"
  | "summary"
>;

/** One compact card on the research overview grid (`/api/public/research-overview`). */
export type ResearchOverviewCard = {
  /** Stable panel key, e.g. `daily-signal-macro`, `company-ratings`. */
  key: string;
  title: string;
  kicker: string;
  report_date?: string | null;
  /** live | partial | stale | waiting | baseline — same vocabulary the panels use. */
  status: string;
  /** Traffic-light signal when the section has one. */
  signal?: string | null;
  score?: number | null;
  /** Short headline metric, e.g. `52 家覆盖` or `3 条提醒`. */
  metric?: string | null;
  summary?: string | null;
};

export type ResearchOverviewPayload = {
  generated_at?: string | null;
  /** The runtime timezone's calendar day, so freshness is judged by the
   *  schedule that produced the snapshots rather than the reader's clock. */
  report_today?: string | null;
  cards: ResearchOverviewCard[];
};
