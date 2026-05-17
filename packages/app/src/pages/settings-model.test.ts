import { describe, expect, it } from "bun:test"

import {
  appendApiKey,
  appendApiKeyVisibility,
  canSelectRunner,
  canShowSettingsTab,
  CHANNEL_CHAT_SCOPES,
  defaultAgentSettings,
  defaultLanguageDraft,
  formatCsv,
  inviteActionKey,
  resolveHoneCloudOpenAiBaseUrl,
  initialApiKeyVisibility,
  isAgentSettingsRuntimeMismatch,
  isInviteActionRunning,
  mergeAuxiliaryDraft,
  mergeAgentSettings,
  mergeHoneCloudDraft,
  normalizePhoneNumber,
  normalizeApiKeys,
  optionalNumber,
  parseCsv,
  removeApiKey,
  removeApiKeyVisibility,
  resolveSettingsTab,
  SETTINGS_TAB_KEYS,
  toChannelDraft,
  toggleApiKeyVisibility,
  updateApiKeyList,
  updateLlmProfileBinding,
  updateLlmProfileEntry,
} from "./settings-model"
import type { MetaInfo } from "@/lib/types"

function metaWithLanguage(language?: "zh" | "en"): MetaInfo {
  return {
    name: "Hone",
    version: "0.0.0-test",
    channel: "imessage",
    supportsImessage: false,
    apiVersion: "desktop-v1",
    capabilities: [],
    deploymentMode: "local",
    language,
  }
}

function requireValue<T>(value: T | null | undefined, label: string): T {
  if (value == null) {
    throw new Error(`${label} was not found`)
  }
  return value
}

function defaultLlmProfiles() {
  return requireValue(defaultAgentSettings().llmProfiles, "default LLM profiles")
}

function profileById(
  profiles: ReturnType<typeof defaultLlmProfiles>["profiles"],
  id: string,
) {
  return requireValue(
    profiles.find((profile) => profile.id === id),
    `${id} profile`,
  )
}

describe("settings-model", () => {
  it("defaults multi-agent answer tool limit to three", () => {
    expect(
      requireValue(defaultAgentSettings().multiAgent, "default multi-agent")
        .answer.maxToolCalls,
    ).toBe(3)
  })

  it("merges partial agent settings onto defaults", () => {
    const merged = mergeAgentSettings({
      ...defaultAgentSettings(),
      runner: "multi-agent",
      auxiliary: undefined,
      multiAgent: undefined,
    })

    expect(merged.runner).toBe("multi-agent")
    expect(requireValue(merged.auxiliary, "auxiliary defaults").baseUrl).toBe(
      "https://api.minimaxi.com/v1",
    )
    expect(requireValue(merged.honeCloud, "Hone Cloud defaults").baseUrl).toBe(
      "https://hone-claw.com",
    )
    expect(
      requireValue(merged.multiAgent, "multi-agent defaults").search
        .maxIterations,
    ).toBe(8)
  })

  it("defaults to Hone Cloud runner settings", () => {
    expect(defaultAgentSettings().runner).toBe("hone_cloud")
    expect(
      requireValue(defaultAgentSettings().honeCloud, "default Hone Cloud").model,
    ).toBe("hone-cloud")
  })

  it("defaults and merges LLM profile settings", () => {
    const defaults = defaultLlmProfiles()
    expect(defaults.defaultProfile).toBe("main")
    expect(defaults.digestPass2Profile).toBe("digest_strong")
    expect(profileById(defaults.profiles, "digest_strong").reasoningEffort).toBe(
      "low",
    )

    const merged = mergeAgentSettings({
      ...defaultAgentSettings(),
      llmProfiles: {
        ...defaultLlmProfiles(),
        defaultProfile: "custom_main",
        profiles: [
          {
            id: "main",
            provider: "openrouter",
            model: "openai/gpt-5.4",
            maxTokens: 2048,
            responseFormatJson: false,
          },
        ],
      },
    })

    const mergedProfiles = requireValue(
      merged.llmProfiles,
      "merged LLM profiles",
    )
    expect(mergedProfiles.defaultProfile).toBe("custom_main")
    expect(profileById(mergedProfiles.profiles, "main").model).toBe(
      "openai/gpt-5.4",
    )
    expect(
      profileById(mergedProfiles.profiles, "digest_strong").model,
    ).toBe("x-ai/grok-4.3")
  })

  it("normalizes empty key lists and derives matching visibility state", () => {
    expect(normalizeApiKeys([])).toEqual([""])
    expect(normalizeApiKeys(["a", "b"])).toEqual(["a", "b"])
    expect(initialApiKeyVisibility([])).toEqual([false])
    expect(initialApiKeyVisibility(["a", "b"])).toEqual([false, false])
  })

  it("normalizes settings form draft inputs", () => {
    expect(normalizePhoneNumber(" +1 (555) 123-4567 ")).toBe("+15551234567")
    expect(normalizePhoneNumber(" 555-123-4567 ")).toBe("5551234567")

    expect(formatCsv(["alice@example.com", "bob@example.com"])).toBe(
      "alice@example.com, bob@example.com",
    )
    expect(formatCsv(undefined)).toBe("")
    expect(parseCsv(" alice@example.com, , bob@example.com ")).toEqual([
      "alice@example.com",
      "bob@example.com",
    ])

    expect(optionalNumber(" 42 ")).toBe(42)
    expect(optionalNumber("")).toBeUndefined()
    expect(optionalNumber("not-a-number")).toBeUndefined()
  })

  it("updates api key lists without mutating the previous settings", () => {
    const previous = { apiKeys: ["a", "b"], label: "fmp" }
    const updated = updateApiKeyList(previous, 1, "next")
    expect(updated.apiKeys).toEqual(["a", "next"])
    expect(updated.label).toBe("fmp")
    expect(previous.apiKeys).toEqual(["a", "b"])
    expect(updated).not.toBe(previous)
    expect(updated.apiKeys).not.toBe(previous.apiKeys)

    const appended = appendApiKey(updated)
    expect(appended.apiKeys).toEqual(["a", "next", ""])
    expect(appended).not.toBe(updated)
    expect(appended.apiKeys).not.toBe(updated.apiKeys)
    expect(updated.apiKeys).toEqual(["a", "next"])

    const removed = removeApiKey(updated, 0)
    expect(removed.apiKeys).toEqual(["next"])
    expect(removed).not.toBe(updated)
    expect(removed.apiKeys).not.toBe(updated.apiKeys)
    expect(updated.apiKeys).toEqual(["a", "next"])

    expect(removeApiKey({ apiKeys: ["only"] }, 0).apiKeys).toEqual([""])
  })

  it("updates api key visibility without mutating the previous list", () => {
    const visibility = [false, true]
    const toggled = toggleApiKeyVisibility(visibility, 1)
    expect(toggled).toEqual([false, false])
    expect(visibility).toEqual([false, true])
    expect(toggled).not.toBe(visibility)

    const appended = appendApiKeyVisibility(visibility)
    expect(appended).toEqual([false, true, false])
    expect(appended).not.toBe(visibility)

    const removed = removeApiKeyVisibility(visibility, 0)
    expect(removed).toEqual([true])
    expect(removed).not.toBe(visibility)
    expect(removeApiKeyVisibility([false], 0)).toEqual([false])
  })

  it("converts persisted channel settings into editable draft", () => {
    expect(
      toChannelDraft({
        configPath: "/tmp/runtime.yaml",
        imessageEnabled: true,
        imessageTargetHandle: "+15551234567",
        feishuEnabled: true,
        feishuAppId: "app",
        feishuAppSecret: "secret",
        feishuChatScope: "ALL",
        feishuAllowEmails: ["admin@example.com"],
        telegramEnabled: false,
        telegramBotToken: undefined,
        discordEnabled: true,
        discordBotToken: "token",
        discordAllowFrom: ["42"],
      }),
    ).toEqual({
      imessageEnabled: true,
      imessageTargetHandle: "+15551234567",
      feishuEnabled: true,
      feishuAppId: "app",
      feishuAppSecret: "secret",
      feishuChatScope: "ALL",
      feishuAllowEmails: ["admin@example.com"],
      feishuAllowMobiles: [],
      feishuAllowOpenIds: [],
      telegramEnabled: false,
      telegramBotToken: "",
      telegramChatScope: "DM_ONLY",
      telegramAllowFrom: [],
      discordEnabled: true,
      discordBotToken: "token",
      discordChatScope: "DM_ONLY",
      discordAllowFrom: ["42"],
    })
  })

  it("skips runner auto-save when clicking the current runner", () => {
    expect(canSelectRunner("multi-agent", "multi-agent", false)).toBe(false)
  })

  it("skips runner auto-save while a runner switch is already saving", () => {
    expect(canSelectRunner("opencode_acp", "multi-agent", true)).toBe(false)
    expect(canSelectRunner("opencode_acp", "multi-agent", false)).toBe(true)
  })

  it("keeps settings navigation state rules in the model layer", () => {
    expect(SETTINGS_TAB_KEYS).toEqual(["agent", "data", "notify", "channel", "invite"])
    expect(resolveSettingsTab("channel")).toBe("channel")
    expect(resolveSettingsTab("unknown")).toBe("agent")
    expect(resolveSettingsTab(undefined)).toBe("agent")
    expect(canShowSettingsTab("agent", false)).toBe(true)
    expect(canShowSettingsTab("invite", false)).toBe(false)
    expect(canShowSettingsTab("invite", true)).toBe(true)
  })

  it("derives invite action keys outside the settings page component", () => {
    expect(inviteActionKey("user-1", "api-key-reset")).toBe("user-1:api-key-reset")
    expect(isInviteActionRunning("user-1:disable", "user-1", "disable")).toBe(true)
    expect(isInviteActionRunning("user-1:disable", "user-1", "enable")).toBe(false)
  })

  it("keeps channel scope options centralized", () => {
    expect(CHANNEL_CHAT_SCOPES).toEqual(["DM_ONLY", "GROUPCHAT_ONLY", "ALL"])
  })

  it("merges agent sub-drafts without dropping default fields", () => {
    const withoutNestedDefaults = {
      ...defaultAgentSettings(),
      honeCloud: undefined,
      auxiliary: undefined,
    }

    expect(
      mergeHoneCloudDraft(withoutNestedDefaults, { apiKey: "hck_test" }).honeCloud,
    ).toEqual({
      baseUrl: "https://hone-claw.com",
      apiKey: "hck_test",
      model: "hone-cloud",
    })
    expect(
      mergeAuxiliaryDraft(withoutNestedDefaults, { model: "next-model" })
        .auxiliary,
    ).toEqual({
      baseUrl: "https://api.minimaxi.com/v1",
      apiKey: "",
      model: "next-model",
    })
  })

  it("updates LLM profile draft slices immutably", () => {
    const current = defaultLlmProfiles()
    const withBinding = updateLlmProfileBinding(current, "defaultProfile", "aux")
    expect(withBinding.defaultProfile).toBe("aux")
    expect(current.defaultProfile).toBe("main")
    expect(withBinding.profiles).toBe(current.profiles)

    const withEntry = updateLlmProfileEntry(current, 0, {
      model: "openai/gpt-5.4",
      maxTokens: 2048,
    })
    expect(withEntry.profiles[0].model).toBe("openai/gpt-5.4")
    expect(withEntry.profiles[0].maxTokens).toBe(2048)
    expect(current.profiles[0].model).toBe("moonshotai/kimi-k2.5")
    expect(withEntry.profiles).not.toBe(current.profiles)
    expect(withEntry.profiles[1]).toBe(current.profiles[1])
  })

  it("resolves Hone Cloud URLs as OpenAI-compatible bases", () => {
    expect(resolveHoneCloudOpenAiBaseUrl("https://hone-claw.com")).toBe(
      "https://hone-claw.com/api/public/v1",
    )
    expect(resolveHoneCloudOpenAiBaseUrl("https://hone-claw.com/api/public/v1")).toBe(
      "https://hone-claw.com/api/public/v1",
    )
    expect(
      resolveHoneCloudOpenAiBaseUrl(
        "https://hone-claw.com/api/public/v1/chat/completions",
      ),
    ).toBe("https://hone-claw.com/api/public/v1")
  })

  it("derives language draft from meta with zh fallback", () => {
    expect(defaultLanguageDraft(undefined)).toBe("zh")
    expect(defaultLanguageDraft(null)).toBe("zh")
    expect(defaultLanguageDraft(metaWithLanguage(undefined))).toBe("zh")
    expect(defaultLanguageDraft(metaWithLanguage("zh"))).toBe("zh")
    expect(defaultLanguageDraft(metaWithLanguage("en"))).toBe("en")
  })

  it("marks agent save result as runtime mismatch when bundled backend is disconnected", () => {
    expect(
      isAgentSettingsRuntimeMismatch({
        settings: defaultAgentSettings(),
        restartedBundledBackend: true,
        message: "已保存 Agent 设置，但当前运行时尚未生效",
        backendStatus: {
          config: {
            mode: "bundled",
            baseUrl: "",
            bearerToken: "",
          },
          connected: false,
        },
      }),
    ).toBe(true)

    expect(
      isAgentSettingsRuntimeMismatch({
        settings: defaultAgentSettings(),
        restartedBundledBackend: false,
        message: "已保存 Agent 设置",
      }),
    ).toBe(false)
  })
})
