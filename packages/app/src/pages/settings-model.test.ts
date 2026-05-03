import { describe, expect, it } from "bun:test"

import {
  appendApiKey,
  appendMaskedKey,
  canSelectRunner,
  defaultAgentSettings,
  defaultLanguageDraft,
  hiddenApiKeys,
  isAgentSettingsRuntimeMismatch,
  mergeAgentSettings,
  normalizeApiKeys,
  removeApiKey,
  removeMaskedKey,
  toChannelDraft,
  toggleMaskedKey,
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
    expect(merged.multiAgent?.search.maxIterations).toBe(8)
  })

  it("normalizes key lists and visibility lists", () => {
    expect(normalizeApiKeys([])).toEqual([""])
    expect(hiddenApiKeys([])).toEqual([false])
  })

  it("updates and mutates api key lists immutably", () => {
    const updated = updateApiKeyList({ apiKeys: ["a", "b"] }, 1, "next")
    expect(updated.apiKeys).toEqual(["a", "next"])
    expect(appendApiKey(updated).apiKeys).toEqual(["a", "next", ""])
    expect(removeApiKey(updated, 0).apiKeys).toEqual(["next"])
    expect(removeApiKey({ apiKeys: ["only"] }, 0).apiKeys).toEqual([""])
  })

  it("updates masked key visibility consistently", () => {
    expect(toggleMaskedKey([false, true], 1)).toEqual([false, false])
    expect(appendMaskedKey([false])).toEqual([false, false])
    expect(removeMaskedKey([false, true], 0)).toEqual([true])
    expect(removeMaskedKey([false], 0)).toEqual([false])
  })

  it("converts persisted channel settings into editable draft", () => {
    expect(
      toChannelDraft({
        configPath: "/tmp/runtime.yaml",
        imessageEnabled: true,
        feishuEnabled: true,
        feishuAppId: "app",
        feishuAppSecret: "secret",
        telegramEnabled: false,
        telegramBotToken: undefined,
        discordEnabled: true,
        discordBotToken: "token",
      }),
    ).toEqual({
      imessageEnabled: true,
      feishuEnabled: true,
      feishuAppId: "app",
      feishuAppSecret: "secret",
      telegramEnabled: false,
      telegramBotToken: "",
      discordEnabled: true,
      discordBotToken: "token",
    })
  })

  it("skips runner auto-save when clicking the current runner", () => {
    expect(canSelectRunner("multi-agent", "multi-agent", false)).toBe(false)
  })

  it("skips runner auto-save while a runner switch is already saving", () => {
    expect(canSelectRunner("opencode_acp", "multi-agent", true)).toBe(false)
    expect(canSelectRunner("opencode_acp", "multi-agent", false)).toBe(true)
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
