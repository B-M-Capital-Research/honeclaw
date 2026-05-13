import type {
  AgentSettings,
  AgentSettingsUpdateResult,
  AgentProvider,
  AuxiliarySettings,
  DesktopChannelSettings,
  DesktopChannelSettingsInput,
  FmpSettings,
  HoneCloudSettings,
  LlmProfileSettings,
  MetaInfo,
  TavilySettings,
} from "@/lib/types"

export type LanguageDraft = "zh" | "en"
export type SettingsTabKey = "agent" | "data" | "notify" | "channel" | "invite"
export type LlmProfileBindingKey = keyof Omit<LlmProfileSettings, "profiles">
export type InviteAction =
  | "disable"
  | "enable"
  | "reset"
  | "api-key"
  | "api-key-reset"

export const SETTINGS_TAB_KEYS: SettingsTabKey[] = [
  "agent",
  "data",
  "notify",
  "channel",
  "invite",
]
export const CHANNEL_CHAT_SCOPES = ["DM_ONLY", "GROUPCHAT_ONLY", "ALL"] as const

export function defaultLanguageDraft(meta?: MetaInfo | null): LanguageDraft {
  return meta?.language === "en" ? "en" : "zh"
}

export function defaultChannelDraft(): DesktopChannelSettingsInput {
  return {
    imessageEnabled: false,
    imessageTargetHandle: "",
    feishuEnabled: false,
    feishuAppId: "",
    feishuAppSecret: "",
    feishuChatScope: "DM_ONLY",
    feishuAllowEmails: [],
    feishuAllowMobiles: [],
    feishuAllowOpenIds: [],
    telegramEnabled: false,
    telegramBotToken: "",
    telegramChatScope: "DM_ONLY",
    telegramAllowFrom: [],
    discordEnabled: false,
    discordBotToken: "",
    discordChatScope: "DM_ONLY",
    discordAllowFrom: [],
  }
}

export function defaultAgentSettings(): AgentSettings {
  return {
    runner: "hone_cloud",
    codexModel: "",
    honeCloud: {
      baseUrl: "https://hone-claw.com",
      apiKey: "",
      model: "hone-cloud",
    },
    openaiUrl: "https://openrouter.ai/api/v1",
    openaiModel: "google/gemini-2.5-pro-preview",
    openaiApiKey: "",
    auxiliary: {
      baseUrl: "https://api.minimaxi.com/v1",
      apiKey: "",
      model: "MiniMax-M2.7-highspeed",
    },
    multiAgent: {
      search: {
        baseUrl: "https://api.minimaxi.com/v1",
        apiKey: "",
        model: "MiniMax-M2.7-highspeed",
        maxIterations: 8,
      },
      answer: {
        baseUrl: "https://openrouter.ai/api/v1",
        apiKey: "",
        model: "google/gemini-2.5-pro-preview",
        variant: "high",
        maxToolCalls: 3,
      },
    },
    llmProfiles: defaultLlmProfileSettings(),
  }
}

function defaultLlmProfileSettings(): LlmProfileSettings {
  return {
    defaultProfile: "main",
    auxiliaryProfile: "aux",
    polishProfile: "aux",
    newsClassifierProfile: "news_classifier",
    filingSummaryProfile: "filing_summary",
    earningsQualityProfile: "earnings_quality",
    digestPass1Profile: "digest_fast",
    digestPass2Profile: "digest_strong",
    digestEventDedupeProfile: "digest_strong",
    mainlineDistillProfile: "mainline_short",
    profiles: [
      {
        id: "main",
        provider: "openrouter",
        model: "moonshotai/kimi-k2.5",
        maxTokens: 32768,
        responseFormatJson: false,
      },
      {
        id: "aux",
        provider: "openrouter",
        model: "moonshotai/kimi-k2.5",
        maxTokens: 4096,
        responseFormatJson: false,
      },
      {
        id: "news_classifier",
        provider: "openrouter",
        model: "x-ai/grok-4.1-fast",
        maxTokens: 64,
        temperature: 0,
        responseFormatJson: false,
      },
      {
        id: "filing_summary",
        provider: "openrouter",
        model: "x-ai/grok-4.1-fast",
        maxTokens: 800,
        temperature: 0.2,
        responseFormatJson: true,
      },
      {
        id: "earnings_quality",
        provider: "openrouter",
        model: "x-ai/grok-4.1-fast",
        maxTokens: 1800,
        temperature: 0.2,
        responseFormatJson: true,
      },
      {
        id: "digest_fast",
        provider: "openrouter",
        model: "x-ai/grok-4.1-fast",
        maxTokens: 1200,
        temperature: 0.2,
        responseFormatJson: false,
      },
      {
        id: "digest_strong",
        provider: "openrouter",
        model: "x-ai/grok-4.1-fast",
        maxTokens: 1600,
        temperature: 0.2,
        reasoningEffort: "low",
        responseFormatJson: false,
      },
      {
        id: "mainline_short",
        provider: "openrouter",
        model: "x-ai/grok-4.1-fast",
        maxTokens: 1200,
        temperature: 0.2,
        responseFormatJson: false,
      },
    ],
  }
}

function mergeLlmProfileSettings(settings: AgentSettings["llmProfiles"]) {
  const defaults = defaultLlmProfileSettings()
  if (!settings) return defaults
  const incomingProfiles = new Map(
    (settings.profiles ?? []).map((profile) => [profile.id, profile]),
  )
  const mergedProfiles = defaults.profiles.map((profile) => ({
    ...profile,
    ...(incomingProfiles.get(profile.id) ?? {}),
  }))
  for (const profile of settings.profiles ?? []) {
    if (
      !mergedProfiles.some((mergedProfile) => mergedProfile.id === profile.id)
    ) {
      mergedProfiles.push(profile)
    }
  }
  return {
    ...defaults,
    ...settings,
    profiles: mergedProfiles,
  }
}

export function mergeAgentSettings(settings?: AgentSettings): AgentSettings {
  const defaults = defaultAgentSettings()
  if (!settings) return defaults
  return {
    ...defaults,
    ...settings,
    auxiliary: settings.auxiliary ?? defaults.auxiliary,
    honeCloud: settings.honeCloud ?? defaults.honeCloud,
    multiAgent: settings.multiAgent ?? defaults.multiAgent,
    llmProfiles: mergeLlmProfileSettings(settings.llmProfiles),
  }
}

export function updateLlmProfileBinding(
  current: LlmProfileSettings,
  key: LlmProfileBindingKey,
  profileId: string,
): LlmProfileSettings {
  return {
    ...current,
    [key]: profileId,
  }
}

export function updateLlmProfileEntry(
  current: LlmProfileSettings,
  targetIndex: number,
  profilePatch: Partial<LlmProfileSettings["profiles"][number]>,
): LlmProfileSettings {
  return {
    ...current,
    profiles: current.profiles.map((profile, profileIndex) =>
      profileIndex === targetIndex ? { ...profile, ...profilePatch } : profile,
    ),
  }
}

export function canSelectRunner(
  currentRunner: AgentProvider,
  nextRunner: AgentProvider,
  isSaving: boolean,
): boolean {
  return !isSaving && currentRunner !== nextRunner
}

export function resolveSettingsTab(raw?: string | null): SettingsTabKey {
  return SETTINGS_TAB_KEYS.includes(raw as SettingsTabKey)
    ? (raw as SettingsTabKey)
    : "agent"
}

export function canShowSettingsTab(
  key: SettingsTabKey,
  hasWebInvites: boolean,
): boolean {
  return key !== "invite" || hasWebInvites
}

export function inviteActionKey(userId: string, action: InviteAction): string {
  return `${userId}:${action}`
}

export function isInviteActionRunning(
  currentKey: string,
  userId: string,
  action: InviteAction,
): boolean {
  return currentKey === inviteActionKey(userId, action)
}

export function mergeHoneCloudDraft(
  current: AgentSettings,
  patch: Partial<HoneCloudSettings>,
): AgentSettings {
  return {
    ...current,
    honeCloud: {
      ...defaultAgentSettings().honeCloud!,
      ...current.honeCloud,
      ...patch,
    },
  }
}

export function mergeAuxiliaryDraft(
  current: AgentSettings,
  patch: Partial<AuxiliarySettings>,
): AgentSettings {
  return {
    ...current,
    auxiliary: {
      ...defaultAgentSettings().auxiliary!,
      ...current.auxiliary,
      ...patch,
    },
  }
}

export function normalizePhoneNumber(value: string): string {
  const trimmed = value.trim()
  const hasLeadingPlus = trimmed.startsWith("+")
  const digits = trimmed.replace(/\D+/g, "")
  return hasLeadingPlus ? `+${digits}` : digits
}

export function formatCsv(values?: string[]): string {
  return (values ?? []).join(", ")
}

export function parseCsv(value: string): string[] {
  return value
    .split(",")
    .map((csvEntry) => csvEntry.trim())
    .filter(Boolean)
}

export function optionalNumber(value: string): number | undefined {
  const trimmed = value.trim()
  if (!trimmed) return undefined
  const parsed = Number(trimmed)
  return Number.isFinite(parsed) ? parsed : undefined
}

export function resolveHoneCloudOpenAiBaseUrl(baseUrl?: string): string {
  const trimmed = (baseUrl ?? "").trim().replace(/\/+$/, "") || "https://hone-claw.com"
  if (trimmed.endsWith("/chat/completions")) {
    return trimmed.slice(0, -"/chat/completions".length)
  }
  if (trimmed.endsWith("/v1")) {
    return trimmed
  }
  return `${trimmed}/api/public/v1`
}

export function isAgentSettingsRuntimeMismatch(result: AgentSettingsUpdateResult): boolean {
  return Boolean(result.backendStatus && !result.backendStatus.connected)
}

export function defaultFmpSettings(): FmpSettings {
  return { apiKeys: [""] }
}

export function defaultTavilySettings(): TavilySettings {
  return { apiKeys: [""] }
}

export function normalizeApiKeys(apiKeys?: string[]): string[] {
  return apiKeys && apiKeys.length > 0 ? apiKeys : [""]
}

export function initialApiKeyVisibility(apiKeys?: string[]): boolean[] {
  return normalizeApiKeys(apiKeys).map(() => false)
}

export function updateApiKeyList<T extends { apiKeys: string[] }>(
  currentSettings: T,
  targetIndex: number,
  apiKey: string,
): T {
  const nextApiKeys = [...currentSettings.apiKeys]
  nextApiKeys[targetIndex] = apiKey
  return { ...currentSettings, apiKeys: nextApiKeys }
}

export function appendApiKey<T extends { apiKeys: string[] }>(
  currentSettings: T,
): T {
  return { ...currentSettings, apiKeys: [...currentSettings.apiKeys, ""] }
}

export function removeApiKey<T extends { apiKeys: string[] }>(
  currentSettings: T,
  targetIndex: number,
): T {
  const nextApiKeys = currentSettings.apiKeys.filter(
    (_, apiKeyIndex) => apiKeyIndex !== targetIndex,
  )
  return {
    ...currentSettings,
    apiKeys: nextApiKeys.length > 0 ? nextApiKeys : [""],
  }
}

export function toggleApiKeyVisibility(
  currentVisibility: boolean[],
  targetIndex: number,
): boolean[] {
  return currentVisibility.map((isVisible, visibilityIndex) =>
    visibilityIndex === targetIndex ? !isVisible : isVisible,
  )
}

export function removeApiKeyVisibility(
  currentVisibility: boolean[],
  targetIndex: number,
): boolean[] {
  const nextVisibility = currentVisibility.filter(
    (_, visibilityIndex) => visibilityIndex !== targetIndex,
  )
  return nextVisibility.length > 0 ? nextVisibility : [false]
}

export function appendApiKeyVisibility(currentVisibility: boolean[]): boolean[] {
  return [...currentVisibility, false]
}

export function toChannelDraft(
  persistedSettings: DesktopChannelSettings,
): DesktopChannelSettingsInput {
  return {
    imessageEnabled: persistedSettings.imessageEnabled,
    imessageTargetHandle: persistedSettings.imessageTargetHandle || "",
    feishuEnabled: persistedSettings.feishuEnabled,
    feishuAppId: persistedSettings.feishuAppId || "",
    feishuAppSecret: persistedSettings.feishuAppSecret || "",
    feishuChatScope: persistedSettings.feishuChatScope || "DM_ONLY",
    feishuAllowEmails: persistedSettings.feishuAllowEmails || [],
    feishuAllowMobiles: persistedSettings.feishuAllowMobiles || [],
    feishuAllowOpenIds: persistedSettings.feishuAllowOpenIds || [],
    telegramEnabled: persistedSettings.telegramEnabled,
    telegramBotToken: persistedSettings.telegramBotToken || "",
    telegramChatScope: persistedSettings.telegramChatScope || "DM_ONLY",
    telegramAllowFrom: persistedSettings.telegramAllowFrom || [],
    discordEnabled: persistedSettings.discordEnabled,
    discordBotToken: persistedSettings.discordBotToken || "",
    discordChatScope: persistedSettings.discordChatScope || "DM_ONLY",
    discordAllowFrom: persistedSettings.discordAllowFrom || [],
  }
}
