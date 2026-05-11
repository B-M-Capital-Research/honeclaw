import { describe, expect, it } from "bun:test"

import {
  appendApiKey,
  appendApiKeyVisibility,
  canSelectRunner,
  defaultAgentSettings,
  defaultLanguageDraft,
  resolveHoneCloudOpenAiBaseUrl,
  initialApiKeyVisibility,
  isAgentSettingsRuntimeMismatch,
  mergeAgentSettings,
  normalizeApiKeys,
  removeApiKey,
  removeApiKeyVisibility,
  toChannelDraft,
  toggleApiKeyVisibility,
  updateApiKeyList,
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

describe("settings-model", () => {
  it("defaults multi-agent answer tool limit to three", () => {
    expect(defaultAgentSettings().multiAgent?.answer.maxToolCalls).toBe(3)
  })

  it("merges partial agent settings onto defaults", () => {
    const merged = mergeAgentSettings({
      ...defaultAgentSettings(),
      runner: "multi-agent",
      auxiliary: undefined,
      multiAgent: undefined,
    })

    expect(merged.runner).toBe("multi-agent")
    expect(merged.auxiliary?.baseUrl).toBe("https://api.minimaxi.com/v1")
    expect(merged.honeCloud?.baseUrl).toBe("https://hone-claw.com")
    expect(merged.multiAgent?.search.maxIterations).toBe(8)
  })

  it("defaults to Hone Cloud runner settings", () => {
    expect(defaultAgentSettings().runner).toBe("hone_cloud")
    expect(defaultAgentSettings().honeCloud?.model).toBe("hone-cloud")
  })

  it("defaults and merges LLM profile settings", () => {
    const defaults = defaultAgentSettings().llmProfiles
    expect(defaults?.defaultProfile).toBe("main")
    expect(defaults?.digestPass2Profile).toBe("digest_strong")
    expect(
      defaults?.profiles.find((profile) => profile.id === "digest_strong")
        ?.reasoningEffort,
    ).toBe("low")

    const merged = mergeAgentSettings({
      ...defaultAgentSettings(),
      llmProfiles: {
        ...defaultAgentSettings().llmProfiles!,
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

    expect(merged.llmProfiles?.defaultProfile).toBe("custom_main")
    expect(
      merged.llmProfiles?.profiles.find((profile) => profile.id === "main")
        ?.model,
    ).toBe("openai/gpt-5.4")
    expect(
      merged.llmProfiles?.profiles.find(
        (profile) => profile.id === "digest_strong",
      )?.model,
    ).toBe("x-ai/grok-4.1-fast")
  })

  it("normalizes empty key lists and derives matching visibility state", () => {
    expect(normalizeApiKeys([])).toEqual([""])
    expect(normalizeApiKeys(["a", "b"])).toEqual(["a", "b"])
    expect(initialApiKeyVisibility([])).toEqual([false])
    expect(initialApiKeyVisibility(["a", "b"])).toEqual([false, false])
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
        message: "已保存 Agent 设置，但当前 runtime 尚未生效",
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
