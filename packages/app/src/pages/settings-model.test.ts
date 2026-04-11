import { describe, expect, it } from "bun:test"

import {
  appendApiKey,
  appendMaskedKey,
  defaultAgentSettings,
  hiddenApiKeys,
  mergeAgentSettings,
  normalizeApiKeys,
  removeApiKey,
  removeMaskedKey,
  toChannelDraft,
  toggleMaskedKey,
  updateApiKeyList,
} from "./settings-model"

describe("settings-model", () => {
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
})
