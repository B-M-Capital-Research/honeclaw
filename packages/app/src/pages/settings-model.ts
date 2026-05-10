import type {
  AgentSettings,
  AgentSettingsUpdateResult,
  AgentProvider,
  DesktopChannelSettings,
  DesktopChannelSettingsInput,
  FmpSettings,
  MetaInfo,
  TavilySettings,
} from "@/lib/types"

export type LanguageDraft = "zh" | "en"

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
  }
}

export function canSelectRunner(
  currentRunner: AgentProvider,
  nextRunner: AgentProvider,
  isSaving: boolean,
): boolean {
  return !isSaving && currentRunner !== nextRunner
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

export function normalizeApiKeys(keys?: string[]): string[] {
  return keys && keys.length > 0 ? keys : [""]
}

export function hiddenApiKeys(keys?: string[]): boolean[] {
  return normalizeApiKeys(keys).map(() => false)
}

export function updateApiKeyList<T extends { apiKeys: string[] }>(
  prev: T,
  index: number,
  value: string,
): T {
  const keys = [...prev.apiKeys]
  keys[index] = value
  return { ...prev, apiKeys: keys }
}

export function appendApiKey<T extends { apiKeys: string[] }>(prev: T): T {
  return { ...prev, apiKeys: [...prev.apiKeys, ""] }
}

export function removeApiKey<T extends { apiKeys: string[] }>(prev: T, index: number): T {
  const keys = prev.apiKeys.filter((_, i) => i !== index)
  return { ...prev, apiKeys: keys.length > 0 ? keys : [""] }
}

export function toggleMaskedKey(prev: boolean[], index: number): boolean[] {
  return prev.map((value, currentIndex) => (currentIndex === index ? !value : value))
}

export function removeMaskedKey(prev: boolean[], index: number): boolean[] {
  const next = prev.filter((_, currentIndex) => currentIndex !== index)
  return next.length > 0 ? next : [false]
}

export function appendMaskedKey(prev: boolean[]): boolean[] {
  return [...prev, false]
}

export function toChannelDraft(settings: DesktopChannelSettings): DesktopChannelSettingsInput {
  return {
    imessageEnabled: settings.imessageEnabled,
    imessageTargetHandle: settings.imessageTargetHandle || "",
    feishuEnabled: settings.feishuEnabled,
    feishuAppId: settings.feishuAppId || "",
    feishuAppSecret: settings.feishuAppSecret || "",
    feishuChatScope: settings.feishuChatScope || "DM_ONLY",
    feishuAllowEmails: settings.feishuAllowEmails || [],
    feishuAllowMobiles: settings.feishuAllowMobiles || [],
    feishuAllowOpenIds: settings.feishuAllowOpenIds || [],
    telegramEnabled: settings.telegramEnabled,
    telegramBotToken: settings.telegramBotToken || "",
    telegramChatScope: settings.telegramChatScope || "DM_ONLY",
    telegramAllowFrom: settings.telegramAllowFrom || [],
    discordEnabled: settings.discordEnabled,
    discordBotToken: settings.discordBotToken || "",
    discordChatScope: settings.discordChatScope || "DM_ONLY",
    discordAllowFrom: settings.discordAllowFrom || [],
  }
}
