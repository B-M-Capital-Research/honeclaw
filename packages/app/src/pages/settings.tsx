import { Index, Show, createEffect, createMemo, createResource, createSignal } from "solid-js"
import { useBackend } from "@/context/backend"
import {
  checkDesktopAgentCli,
  loadDesktopAgentSettings,
  testDesktopOpenAiChannel,
  loadDesktopFmpSettings,
  saveDesktopFmpSettings,
  loadDesktopTavilySettings,
  saveDesktopTavilySettings,
} from "@/lib/backend"
import type { AgentProvider, AgentSettings, BackendConfig, DesktopChannelSettingsInput, FmpSettings, TavilySettings } from "@/lib/types"
import {
  appendApiKey,
  appendMaskedKey,
  canSelectRunner,
  defaultAgentSettings,
  defaultChannelDraft,
  defaultFmpSettings,
  defaultTavilySettings,
  hiddenApiKeys,
  isAgentSettingsRuntimeMismatch,
  mergeAgentSettings,
  normalizeApiKeys,
  removeApiKey,
  removeMaskedKey,
  toChannelDraft,
  toggleMaskedKey,
  updateApiKeyList,
} from "@/pages/settings-model"

export default function SettingsPage() {
  const backend = useBackend()
  const [draft, setDraft] = createSignal<BackendConfig>(backend.state.config)
  const [channelDraft, setChannelDraft] = createSignal<DesktopChannelSettingsInput>(defaultChannelDraft())
  const [channelMessage, setChannelMessage] = createSignal("")
  const [channelError, setChannelError] = createSignal("")
  const capabilities = createMemo(() => backend.state.meta?.capabilities ?? [])
  const [desktopChannelSettings, { refetch: refetchDesktopChannelSettings, mutate: setDesktopChannelSettings }] =
    createResource(
      () => backend.state.isDesktop,
      async (isDesktop) => {
        if (!isDesktop) return undefined
        return backend.loadChannelSettings()
      },
    )

  // ── Agent 基础设置 ──────────────────────────────────────────────────────────
  const [agentDraft, setAgentDraft] = createSignal<AgentSettings>(defaultAgentSettings())
  const [agentSaving, setAgentSaving] = createSignal(false)
  const [agentMessage, setAgentMessage] = createSignal("")
  const [agentError, setAgentError] = createSignal("")

  // OpenAI 协议渠道测试状态
  const [openaiTestStatus, setOpenaiTestStatus] = createSignal<"idle" | "checking" | "ok" | "error">("idle")
  const [openaiTestMessage, setOpenaiTestMessage] = createSignal("")
  const [showOpenaiKey, setShowOpenaiKey] = createSignal(false)
  const [auxiliaryTestStatus, setAuxiliaryTestStatus] = createSignal<"idle" | "checking" | "ok" | "error">("idle")
  const [auxiliaryTestMessage, setAuxiliaryTestMessage] = createSignal("")
  const [showAuxiliaryKey, setShowAuxiliaryKey] = createSignal(false)
  const [showSearchKey, setShowSearchKey] = createSignal(false)
  const [showAnswerKey, setShowAnswerKey] = createSignal(false)
  const [showFeishuSecret, setShowFeishuSecret] = createSignal(false)
  const [showTelegramToken, setShowTelegramToken] = createSignal(false)
  const [showDiscordToken, setShowDiscordToken] = createSignal(false)

  // Gemini CLI 检测状态
  const [geminiCheckStatus, setGeminiCheckStatus] = createSignal<"idle" | "checking" | "ok" | "error">("idle")
  const [geminiCheckMessage, setGeminiCheckMessage] = createSignal("")

  // Codex CLI 检测状态
  const [codexCheckStatus, setCodexCheckStatus] = createSignal<"idle" | "checking" | "ok" | "error">("idle")
  const [codexCheckMessage, setCodexCheckMessage] = createSignal("")
  const [opencodeCheckStatus, setOpencodeCheckStatus] = createSignal<"idle" | "checking" | "ok" | "error">("idle")
  const [opencodeCheckMessage, setOpencodeCheckMessage] = createSignal("")
  const [searchTestStatus, setSearchTestStatus] = createSignal<"idle" | "checking" | "ok" | "error">("idle")
  const [searchTestMessage, setSearchTestMessage] = createSignal("")
  const [answerTestStatus, setAnswerTestStatus] = createSignal<"idle" | "checking" | "ok" | "error">("idle")
  const [answerTestMessage, setAnswerTestMessage] = createSignal("")

  const [agentSettingsRes] = createResource(
    () => backend.state.isDesktop,
    async (isDesktop) => {
      if (!isDesktop) return undefined
      return loadDesktopAgentSettings()
    },
  )

  createEffect(() => {
    const s = agentSettingsRes()
    if (s) {
      setAgentDraft(mergeAgentSettings(s))
    }
  })


  // ── FMP API Keys 设置 ───────────────────────────────────────────────────────
  const [fmpDraft, setFmpDraft] = createSignal<FmpSettings>(defaultFmpSettings())
  const [fmpSaving, setFmpSaving] = createSignal(false)
  const [fmpMessage, setFmpMessage] = createSignal("")
  const [fmpError, setFmpError] = createSignal("")
  const [showFmpKeys, setShowFmpKeys] = createSignal<boolean[]>([false])

  const [fmpSettingsRes] = createResource(
    () => backend.state.isDesktop,
    async (isDesktop) => {
      if (!isDesktop) return undefined
      return loadDesktopFmpSettings()
    },
  )

  createEffect(() => {
    const s = fmpSettingsRes()
    if (s) {
      const keys = normalizeApiKeys(s.apiKeys)
      setFmpDraft({ apiKeys: keys })
      setShowFmpKeys(hiddenApiKeys(keys))
    }
  })

  const submitFmpSettings = async (event: Event) => {
    event.preventDefault()
    setFmpSaving(true)
    setFmpMessage("")
    setFmpError("")
    try {
      await saveDesktopFmpSettings(fmpDraft())
      setFmpMessage("已保存 FMP API Keys，内置后端已重启生效")
    } catch (e) {
      setFmpError(e instanceof Error ? e.message : String(e))
    } finally {
      setFmpSaving(false)
    }
  }

  // ── Tavily API Keys 设置 ────────────────────────────────────────────────────
  const [tavilyDraft, setTavilyDraft] = createSignal<TavilySettings>(defaultTavilySettings())
  const [tavilySaving, setTavilySaving] = createSignal(false)
  const [tavilyMessage, setTavilyMessage] = createSignal("")
  const [tavilyError, setTavilyError] = createSignal("")
  const [showTavilyKeys, setShowTavilyKeys] = createSignal<boolean[]>([false])

  const [tavilySettingsRes] = createResource(
    () => backend.state.isDesktop,
    async (isDesktop) => {
      if (!isDesktop) return undefined
      return loadDesktopTavilySettings()
    },
  )

  createEffect(() => {
    const s = tavilySettingsRes()
    if (s) {
      const keys = normalizeApiKeys(s.apiKeys)
      setTavilyDraft({ apiKeys: keys })
      setShowTavilyKeys(hiddenApiKeys(keys))
    }
  })

  const submitTavilySettings = async (event: Event) => {
    event.preventDefault()
    setTavilySaving(true)
    setTavilyMessage("")
    setTavilyError("")
    try {
      await saveDesktopTavilySettings(tavilyDraft())
      setTavilyMessage("已保存 Tavily API Keys，内置后端已重启生效")
    } catch (e) {
      setTavilyError(e instanceof Error ? e.message : String(e))
    } finally {
      setTavilySaving(false)
    }
  }

  // ── 多 Key 输入辅助函数 ──────────────────────────────────────────────────────
  /** 更新指定索引的 key 值 */
  function updateKey<T extends { apiKeys: string[] }>(
    setter: (fn: (prev: T) => T) => void,
    index: number,
    value: string,
  ) {
    setter((prev) => updateApiKeyList(prev, index, value))
  }

  /** 追加一个空 key 输入行 */
  function addKey<T extends { apiKeys: string[] }>(
    setter: (fn: (prev: T) => T) => void,
    showSetter: (fn: (prev: boolean[]) => boolean[]) => void,
  ) {
    setter((prev) => appendApiKey(prev))
    showSetter((prev) => appendMaskedKey(prev))
  }

  /** 删除指定索引的 key */
  function removeKey<T extends { apiKeys: string[] }>(
    setter: (fn: (prev: T) => T) => void,
    showSetter: (fn: (prev: boolean[]) => boolean[]) => void,
    index: number,
  ) {
    setter((prev) => removeApiKey(prev, index))
    showSetter((prev) => removeMaskedKey(prev, index))
  }

  /** 切换指定索引的 key 显示/隐藏 */
  function toggleShowKey(
    showSetter: (fn: (prev: boolean[]) => boolean[]) => void,
    index: number,
  ) {
    showSetter((prev) => toggleMaskedKey(prev, index))
  }

  // ── OpenAI 协议渠道测试 ──────────────────────────────────────────────────────
  const handleTestOpenAi = async () => {
    setOpenaiTestStatus("checking")
    setOpenaiTestMessage("")
    try {
      const d = agentDraft()
      const result = await testDesktopOpenAiChannel(d.openaiUrl, d.openaiModel, d.openaiApiKey)
      setOpenaiTestStatus(result.ok ? "ok" : "error")
      setOpenaiTestMessage(result.message)
    } catch (e) {
      setOpenaiTestStatus("error")
      setOpenaiTestMessage(e instanceof Error ? e.message : String(e))
    }
  }

  const handleTestAuxiliary = async () => {
    setAuxiliaryTestStatus("checking")
    setAuxiliaryTestMessage("")
    try {
      const auxiliary = agentDraft().auxiliary
      const result = await testDesktopOpenAiChannel(
        auxiliary?.baseUrl ?? "",
        auxiliary?.model ?? "",
        auxiliary?.apiKey ?? "",
      )
      setAuxiliaryTestStatus(result.ok ? "ok" : "error")
      setAuxiliaryTestMessage(result.message)
    } catch (e) {
      setAuxiliaryTestStatus("error")
      setAuxiliaryTestMessage(e instanceof Error ? e.message : String(e))
    }
  }

  // ── Gemini CLI 检测 ──────────────────────────────────────────────────────────
  const handleCheckGemini = async () => {
    setGeminiCheckStatus("checking")
    setGeminiCheckMessage("")
    try {
      const result = await checkDesktopAgentCli("gemini_cli")
      setGeminiCheckStatus(result.ok ? "ok" : "error")
      setGeminiCheckMessage(result.message)
    } catch (e) {
      setGeminiCheckStatus("error")
      setGeminiCheckMessage(e instanceof Error ? e.message : String(e))
    }
  }

  // ── Codex CLI 检测 ──────────────────────────────────────────────────────────
  const handleCheckCodex = async () => {
    setCodexCheckStatus("checking")
    setCodexCheckMessage("")
    try {
      const result = await checkDesktopAgentCli("codex_cli")
      setCodexCheckStatus(result.ok ? "ok" : "error")
      setCodexCheckMessage(result.message)
    } catch (e) {
      setCodexCheckStatus("error")
      setCodexCheckMessage(e instanceof Error ? e.message : String(e))
    }
  }

  const handleCheckOpencode = async () => {
    setOpencodeCheckStatus("checking")
    setOpencodeCheckMessage("")
    try {
      const result = await checkDesktopAgentCli("opencode_acp")
      setOpencodeCheckStatus(result.ok ? "ok" : "error")
      setOpencodeCheckMessage(result.message)
    } catch (e) {
      setOpencodeCheckStatus("error")
      setOpencodeCheckMessage(e instanceof Error ? e.message : String(e))
    }
  }

  const handleTestMultiAgentSearch = async () => {
    setSearchTestStatus("checking")
    setSearchTestMessage("")
    try {
      const search = agentDraft().multiAgent?.search
      const result = await testDesktopOpenAiChannel(
        search?.baseUrl ?? "",
        search?.model ?? "",
        search?.apiKey ?? "",
      )
      setSearchTestStatus(result.ok ? "ok" : "error")
      setSearchTestMessage(result.message)
    } catch (e) {
      setSearchTestStatus("error")
      setSearchTestMessage(e instanceof Error ? e.message : String(e))
    }
  }

  const handleTestMultiAgentAnswer = async () => {
    setAnswerTestStatus("checking")
    setAnswerTestMessage("")
    try {
      const answer = agentDraft().multiAgent?.answer
      const result = await testDesktopOpenAiChannel(
        answer?.baseUrl ?? "",
        answer?.model ?? "",
        answer?.apiKey ?? "",
      )
      setAnswerTestStatus(result.ok ? "ok" : "error")
      setAnswerTestMessage(result.message)
    } catch (e) {
      setAnswerTestStatus("error")
      setAnswerTestMessage(e instanceof Error ? e.message : String(e))
    }
  }

  // ── 选中某个 runner 并立即保存 ───────────────────────────────────────────────
  const selectRunner = async (runner: AgentProvider) => {
    const previous = agentDraft()
    if (!canSelectRunner(previous.runner, runner, agentSaving())) return
    const next = { ...previous, runner }
    setAgentDraft(next)
    setAgentSaving(true)
    setAgentMessage("")
    setAgentError("")
    try {
      const result = await backend.saveAgentSettings(next)
      if (isAgentSettingsRuntimeMismatch(result)) {
        setAgentError(result.message)
      } else {
        setAgentMessage(result.message)
      }
    } catch (e) {
      setAgentDraft(previous)
      setAgentError(e instanceof Error ? e.message : String(e))
    } finally {
      setAgentSaving(false)
    }
  }

  const submitAgentSettings = async (event: Event) => {
    event.preventDefault()
    setAgentSaving(true)
    setAgentMessage("")
    setAgentError("")
    try {
      const result = await backend.saveAgentSettings(agentDraft())
      if (isAgentSettingsRuntimeMismatch(result)) {
        setAgentError(result.message)
      } else {
        setAgentMessage(result.message)
      }
    } catch (e) {
      setAgentError(e instanceof Error ? e.message : String(e))
    } finally {
      setAgentSaving(false)
    }
  }

  createEffect(() => {
    setDraft(backend.state.config)
  })

  createEffect(() => {
    const settings = desktopChannelSettings()
    if (!settings) return
    setChannelDraft(toChannelDraft(settings))
  })

  const submit = async (event: Event) => {
    event.preventDefault()
    await backend.saveConfig(draft())
  }

  const submitChannels = async (event: Event) => {
    event.preventDefault()
    setChannelMessage("")
    setChannelError("")
    try {
      const result = await backend.saveChannelSettings(channelDraft())
      setDesktopChannelSettings(result.settings)
      setChannelMessage(result.message)
    } catch (error) {
      setChannelError(error instanceof Error ? error.message : String(error))
    }
  }

  return (
    <div class="mx-auto flex h-full max-w-4xl flex-col gap-4 overflow-y-auto">
      {/* ── 基础设置 ── */}
      <div id="agent-settings" class="rounded-2xl border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm">
        <h1 class="text-xl font-semibold text-[color:var(--text-primary)]">基础设置</h1>
        <p class="mt-2 text-sm text-[color:var(--text-secondary)]">
          选择 Agent 引擎并配置相关参数，保存后立即写入运行时配置。
        </p>

        <fieldset disabled={!backend.state.isDesktop || agentSettingsRes.loading || agentSaving()} class="mt-6 space-y-4 disabled:opacity-60">

          {/* ── 卡片 0：Multi-Agent ── */}
          <div
            class={[
              "rounded-xl border p-5 transition cursor-pointer",
              agentDraft().runner === "multi-agent"
                ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                : "border-[color:var(--border)] bg-[color:var(--panel)] hover:border-[color:var(--accent)]/50",
            ].join(" ")}
            onClick={() => void selectRunner("multi-agent")}
          >
            <div class="flex items-start justify-between gap-3">
              <div>
                <div class="text-sm font-semibold text-[color:var(--text-primary)]">Multi-Agent</div>
                <div class="mt-0.5 text-xs text-[color:var(--text-secondary)]">
                  Search Agent 使用 MiniMax function calling，Answer Agent 使用 opencode ACP 收束回复
                </div>
              </div>
              <Show when={agentDraft().runner === "multi-agent"}>
                <span class="shrink-0 rounded-full border border-[color:var(--accent)] px-2 py-0.5 text-[10px] font-medium text-[color:var(--accent)]">当前</span>
              </Show>
            </div>

            <div class="mt-4 grid gap-4 md:grid-cols-2" onClick={(e) => e.stopPropagation()}>
              <div class="space-y-3 rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
                <div class="text-xs font-semibold text-[color:var(--text-primary)]">Search Agent (MiniMax / OpenAI-compatible)</div>
                <input
                  type="url"
                  placeholder="https://api.minimaxi.com/v1"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm"
                  value={agentDraft().multiAgent?.search.baseUrl ?? ""}
                  onInput={(e) => setAgentDraft((prev) => ({
                    ...prev,
                    multiAgent: {
                      ...prev.multiAgent!,
                      search: { ...prev.multiAgent!.search, baseUrl: e.currentTarget.value },
                    },
                  }))}
                />
                <input
                  type="text"
                  placeholder="MiniMax-M2.7-highspeed"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm"
                  value={agentDraft().multiAgent?.search.model ?? ""}
                  onInput={(e) => setAgentDraft((prev) => ({
                    ...prev,
                    multiAgent: {
                      ...prev.multiAgent!,
                      search: { ...prev.multiAgent!.search, model: e.currentTarget.value },
                    },
                  }))}
                />
                <div class="relative">
                  <input
                    type={showSearchKey() ? "text" : "password"}
                    placeholder="sk-cp-..."
                    class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 pr-16 text-sm"
                    value={agentDraft().multiAgent?.search.apiKey ?? ""}
                    onInput={(e) => setAgentDraft((prev) => ({
                      ...prev,
                      multiAgent: {
                        ...prev.multiAgent!,
                        search: { ...prev.multiAgent!.search, apiKey: e.currentTarget.value },
                      },
                    }))}
                  />
                  <button
                    type="button"
                    class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-2 py-0.5 text-xs text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                    onClick={() => setShowSearchKey((v) => !v)}
                  >
                    {showSearchKey() ? "隐藏" : "显示"}
                  </button>
                </div>
                <input
                  type="number"
                  min="1"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm"
                  value={agentDraft().multiAgent?.search.maxIterations ?? 8}
                  onInput={(e) => setAgentDraft((prev) => ({
                    ...prev,
                    multiAgent: {
                      ...prev.multiAgent!,
                      search: { ...prev.multiAgent!.search, maxIterations: Number(e.currentTarget.value || 0) },
                    },
                  }))}
                />
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs"
                  disabled={searchTestStatus() === "checking"}
                  onClick={() => void handleTestMultiAgentSearch()}
                >
                  {searchTestStatus() === "checking" ? "测试中…" : "测试 Search Agent"}
                </button>
                <Show when={searchTestStatus() !== "idle"}>
                  <div class="text-xs text-[color:var(--text-secondary)]">{searchTestMessage()}</div>
                </Show>
              </div>

              <div class="space-y-3 rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-4">
                <div class="text-xs font-semibold text-[color:var(--text-primary)]">Answer Agent (OpenAI-compatible via opencode ACP)</div>
                <input
                  type="url"
                  placeholder="https://openrouter.ai/api/v1"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm"
                  value={agentDraft().multiAgent?.answer.baseUrl ?? ""}
                  onInput={(e) => setAgentDraft((prev) => ({
                    ...prev,
                    multiAgent: {
                      ...prev.multiAgent!,
                      answer: { ...prev.multiAgent!.answer, baseUrl: e.currentTarget.value },
                    },
                  }))}
                />
                <input
                  type="text"
                  placeholder="google/gemini-3.1-pro-preview"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm"
                  value={agentDraft().multiAgent?.answer.model ?? ""}
                  onInput={(e) => setAgentDraft((prev) => ({
                    ...prev,
                    multiAgent: {
                      ...prev.multiAgent!,
                      answer: { ...prev.multiAgent!.answer, model: e.currentTarget.value },
                    },
                  }))}
                />
                <input
                  type="text"
                  placeholder="high"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm"
                  value={agentDraft().multiAgent?.answer.variant ?? ""}
                  onInput={(e) => setAgentDraft((prev) => ({
                    ...prev,
                    multiAgent: {
                      ...prev.multiAgent!,
                      answer: { ...prev.multiAgent!.answer, variant: e.currentTarget.value },
                    },
                  }))}
                />
                <div class="relative">
                  <input
                    type={showAnswerKey() ? "text" : "password"}
                    placeholder="sk-or-..."
                    class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 pr-16 text-sm"
                    value={agentDraft().multiAgent?.answer.apiKey ?? ""}
                    onInput={(e) => setAgentDraft((prev) => ({
                      ...prev,
                      multiAgent: {
                        ...prev.multiAgent!,
                        answer: { ...prev.multiAgent!.answer, apiKey: e.currentTarget.value },
                      },
                    }))}
                  />
                  <button
                    type="button"
                    class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-2 py-0.5 text-xs text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                    onClick={() => setShowAnswerKey((v) => !v)}
                  >
                    {showAnswerKey() ? "隐藏" : "显示"}
                  </button>
                </div>
                <input
                  type="number"
                  min="0"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm"
                  value={agentDraft().multiAgent?.answer.maxToolCalls ?? 1}
                  onInput={(e) => setAgentDraft((prev) => ({
                    ...prev,
                    multiAgent: {
                      ...prev.multiAgent!,
                      answer: { ...prev.multiAgent!.answer, maxToolCalls: Number(e.currentTarget.value || 0) },
                    },
                  }))}
                />
                <div class="flex gap-2">
                  <button
                    type="button"
                    class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs"
                    disabled={answerTestStatus() === "checking"}
                    onClick={() => void handleTestMultiAgentAnswer()}
                  >
                    {answerTestStatus() === "checking" ? "测试中…" : "测试 Answer Agent"}
                  </button>
                  <button
                    type="button"
                    class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs"
                    disabled={opencodeCheckStatus() === "checking"}
                    onClick={() => void handleCheckOpencode()}
                  >
                    {opencodeCheckStatus() === "checking" ? "检测中…" : "检查 opencode"}
                  </button>
                </div>
                <Show when={answerTestStatus() !== "idle"}>
                  <div class="text-xs text-[color:var(--text-secondary)]">{answerTestMessage()}</div>
                </Show>
                <Show when={opencodeCheckStatus() !== "idle"}>
                  <div class="text-xs text-[color:var(--text-secondary)]">{opencodeCheckMessage()}</div>
                </Show>
              </div>
            </div>
          </div>

          {/* ── 卡片 1：OpenAI 协议渠道 ── */}
          <div
            class={[
              "rounded-xl border p-5 transition cursor-pointer",
              agentDraft().runner === "opencode_acp"
                ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                : "border-[color:var(--border)] bg-[color:var(--panel)] hover:border-[color:var(--accent)]/50",
            ].join(" ")}
            onClick={() => void selectRunner("opencode_acp")}
          >
            <div class="flex items-start justify-between gap-3">
              <div>
                <div class="text-sm font-semibold text-[color:var(--text-primary)]">OpenAI 协议渠道</div>
                <div class="mt-0.5 text-xs text-[color:var(--text-secondary)]">
                  兼容 OpenRouter、OpenAI 及任意 OpenAI-compatible 端点（通过 opencode acp 驱动）
                </div>
              </div>
              <Show when={agentDraft().runner === "opencode_acp"}>
                <span class="shrink-0 rounded-full border border-[color:var(--accent)] px-2 py-0.5 text-[10px] font-medium text-[color:var(--accent)]">当前</span>
              </Show>
            </div>

            {/* 配置字段区（点击卡片内部不触发 selectRunner） */}
            <div class="mt-4 space-y-3" onClick={(e) => e.stopPropagation()}>
              {/* Base URL */}
              <div>
                <label class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]" for="openai-url">
                  Base URL
                </label>
                <input
                  id="openai-url"
                  type="url"
                  placeholder="https://openrouter.ai/api/v1"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                  value={agentDraft().openaiUrl}
                  onInput={(e) => setAgentDraft((prev) => ({ ...prev, openaiUrl: e.currentTarget.value }))}
                />
              </div>

              {/* Model */}
              <div>
                <label class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]" for="openai-model">
                  主模型
                </label>
                <input
                  id="openai-model"
                  type="text"
                  placeholder="google/gemini-2.5-pro-preview"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                  value={agentDraft().openaiModel}
                  onInput={(e) => setAgentDraft((prev) => ({ ...prev, openaiModel: e.currentTarget.value }))}
                />
              </div>

              <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface-soft)] p-3">
                <p class="text-xs font-medium text-[color:var(--text-primary)]">Auxiliary 子模型链路</p>
                <p class="mt-1 text-[11px] text-[color:var(--text-muted)]">
                  用于心跳检测、会话压缩等后台辅助任务，支持独立的 OpenAI-compatible Base URL / API Key / Model。
                </p>
                <div class="mt-3 space-y-3">
                  <div>
                    <label class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]" for="auxiliary-url">
                      Auxiliary Base URL
                    </label>
                    <input
                      id="auxiliary-url"
                      type="url"
                      placeholder="https://api.minimaxi.com/v1"
                      class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                      value={agentDraft().auxiliary?.baseUrl ?? ""}
                      onInput={(e) => setAgentDraft((prev) => ({
                        ...prev,
                        auxiliary: { ...(prev.auxiliary ?? { baseUrl: "", apiKey: "", model: "" }), baseUrl: e.currentTarget.value },
                      }))}
                    />
                  </div>
                  <div>
                    <label class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]" for="auxiliary-model">
                      Auxiliary Model
                    </label>
                    <input
                      id="auxiliary-model"
                      type="text"
                      placeholder="MiniMax-M2.7-highspeed"
                      class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                      value={agentDraft().auxiliary?.model ?? ""}
                      onInput={(e) => setAgentDraft((prev) => ({
                        ...prev,
                        auxiliary: { ...(prev.auxiliary ?? { baseUrl: "", apiKey: "", model: "" }), model: e.currentTarget.value },
                      }))}
                    />
                  </div>
                  <div>
                    <label class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]" for="auxiliary-apikey">
                      Auxiliary API Key
                    </label>
                    <div class="relative">
                      <input
                        id="auxiliary-apikey"
                        type={showAuxiliaryKey() ? "text" : "password"}
                        placeholder="sk-cp-..."
                        class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 pr-16 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                        value={agentDraft().auxiliary?.apiKey ?? ""}
                        onInput={(e) => setAgentDraft((prev) => ({
                          ...prev,
                          auxiliary: { ...(prev.auxiliary ?? { baseUrl: "", apiKey: "", model: "" }), apiKey: e.currentTarget.value },
                        }))}
                      />
                      <button
                        type="button"
                        class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-2 py-0.5 text-xs text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                        onClick={() => setShowAuxiliaryKey((v) => !v)}
                      >
                        {showAuxiliaryKey() ? "隐藏" : "显示"}
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              {/* API Key */}
              <div>
                <label class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]" for="openai-apikey">
                  API Key
                </label>
                <div class="relative">
                  <input
                    id="openai-apikey"
                    type={showOpenaiKey() ? "text" : "password"}
                    placeholder="sk-or-..."
                    class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 pr-16 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                    value={agentDraft().openaiApiKey}
                    onInput={(e) => setAgentDraft((prev) => ({ ...prev, openaiApiKey: e.currentTarget.value }))}
                  />
                  <button
                    type="button"
                    class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-2 py-0.5 text-xs text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                    onClick={() => setShowOpenaiKey((v) => !v)}
                  >
                    {showOpenaiKey() ? "隐藏" : "显示"}
                  </button>
                </div>
              </div>

              {/* 测试联通状态 */}
              <Show when={openaiTestStatus() !== "idle"}>
                <div
                  class={[
                    "flex items-center gap-2 rounded-lg border px-3 py-2 text-xs",
                    openaiTestStatus() === "checking"
                      ? "border-amber-300/40 bg-amber-500/10 text-amber-300"
                      : openaiTestStatus() === "ok"
                        ? "border-emerald-300/40 bg-emerald-500/10 text-emerald-300"
                        : "border-rose-300/40 bg-rose-500/10 text-rose-300",
                  ].join(" ")}
                >
                  <Show when={openaiTestStatus() === "checking"}>
                    <svg class="h-3.5 w-3.5 shrink-0 animate-spin" viewBox="0 0 24 24" fill="none">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 22 6.477 22 12h-4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                    </svg>
                  </Show>
                  <Show when={openaiTestStatus() === "ok"}>
                    <svg class="h-3.5 w-3.5 shrink-0" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                    </svg>
                  </Show>
                  <Show when={openaiTestStatus() === "error"}>
                    <svg class="h-3.5 w-3.5 shrink-0" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                    </svg>
                  </Show>
                  <span>
                    {openaiTestStatus() === "checking" ? "连通测试中，请稍候…" : openaiTestMessage()}
                  </span>
                </div>
              </Show>

              <Show when={auxiliaryTestStatus() !== "idle"}>
                <div
                  class={[
                    "flex items-center gap-2 rounded-lg border px-3 py-2 text-xs",
                    auxiliaryTestStatus() === "checking"
                      ? "border-amber-300/40 bg-amber-500/10 text-amber-300"
                      : auxiliaryTestStatus() === "ok"
                        ? "border-emerald-300/40 bg-emerald-500/10 text-emerald-300"
                        : "border-rose-300/40 bg-rose-500/10 text-rose-300",
                  ].join(" ")}
                >
                  <span>
                    {auxiliaryTestStatus() === "checking" ? "Auxiliary 连通测试中，请稍候…" : auxiliaryTestMessage()}
                  </span>
                </div>
              </Show>

              {/* 反馈 */}
              {agentMessage() ? (
                <div class="rounded-md border border-emerald-300/40 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-300">
                  {agentMessage()}
                </div>
              ) : null}
              {agentError() ? (
                <div class="rounded-md border border-rose-300/40 bg-rose-500/10 px-3 py-2 text-xs text-rose-300">
                  {agentError()}
                </div>
              ) : null}

              {/* 操作按钮 */}
              <div class="flex gap-2 pt-1">
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs text-[color:var(--text-primary)] transition hover:border-[color:var(--accent)]/60 disabled:opacity-50"
                  disabled={openaiTestStatus() === "checking"}
                  onClick={() => void handleTestOpenAi()}
                >
                  {openaiTestStatus() === "checking" ? "测试中…" : "测试联通"}
                </button>
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs text-[color:var(--text-primary)] transition hover:border-[color:var(--accent)]/60 disabled:opacity-50"
                  disabled={auxiliaryTestStatus() === "checking"}
                  onClick={() => void handleTestAuxiliary()}
                >
                  {auxiliaryTestStatus() === "checking" ? "测试中…" : "测试 Auxiliary"}
                </button>
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--accent)] bg-[color:var(--accent-soft)] px-3 py-1.5 text-xs font-medium text-[color:var(--text-primary)] transition hover:opacity-90 disabled:opacity-50"
                  disabled={agentSaving()}
                  onClick={(e) => void submitAgentSettings(e)}
                >
                  {agentSaving() ? "保存中…" : "保存"}
                </button>
              </div>
            </div>
          </div>

          {/* ── 卡片 2：Gemini CLI ── */}
          <div
            class={[
              "rounded-xl border p-5 transition cursor-pointer",
              agentDraft().runner === "gemini_cli"
                ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                : "border-[color:var(--border)] bg-[color:var(--panel)] hover:border-[color:var(--accent)]/50",
            ].join(" ")}
            onClick={() => void selectRunner("gemini_cli")}
          >
            <div class="flex items-start justify-between gap-3">
              <div>
                <div class="text-sm font-semibold text-[color:var(--text-primary)]">Gemini CLI</div>
                <div class="mt-0.5 text-xs text-[color:var(--text-secondary)]">
                  使用本机安装的 <code class="rounded bg-black/20 px-1">gemini</code> 命令行 Agent
                </div>
              </div>
              <Show when={agentDraft().runner === "gemini_cli"}>
                <span class="shrink-0 rounded-full border border-[color:var(--accent)] px-2 py-0.5 text-[10px] font-medium text-[color:var(--accent)]">当前</span>
              </Show>
            </div>

            <div class="mt-4 space-y-3" onClick={(e) => e.stopPropagation()}>
              {/* 检测状态 */}
              <Show when={geminiCheckStatus() !== "idle"}>
                <div
                  class={[
                    "flex items-center gap-2 rounded-lg border px-3 py-2 text-xs",
                    geminiCheckStatus() === "checking"
                      ? "border-amber-300/40 bg-amber-500/10 text-amber-300"
                      : geminiCheckStatus() === "ok"
                        ? "border-emerald-300/40 bg-emerald-500/10 text-emerald-300"
                        : "border-rose-300/40 bg-rose-500/10 text-rose-300",
                  ].join(" ")}
                >
                  <Show when={geminiCheckStatus() === "checking"}>
                    <svg class="h-3.5 w-3.5 shrink-0 animate-spin" viewBox="0 0 24 24" fill="none">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 22 6.477 22 12h-4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                    </svg>
                  </Show>
                  <Show when={geminiCheckStatus() === "ok"}>
                    <svg class="h-3.5 w-3.5 shrink-0" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                    </svg>
                  </Show>
                  <Show when={geminiCheckStatus() === "error"}>
                    <svg class="h-3.5 w-3.5 shrink-0" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                    </svg>
                  </Show>
                  <span>
                    {geminiCheckStatus() === "checking" ? "检测中，请稍候…" : geminiCheckMessage()}
                  </span>
                </div>
              </Show>

              <div class="flex gap-2 pt-1">
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs text-[color:var(--text-primary)] transition hover:border-[color:var(--accent)]/60 disabled:opacity-50"
                  disabled={geminiCheckStatus() === "checking"}
                  onClick={() => void handleCheckGemini()}
                >
                  {geminiCheckStatus() === "checking" ? "检测中…" : "测试联通"}
                </button>
              </div>
            </div>
          </div>

          {/* ── 卡片 3：Codex ── */}
          <div
            class={[
              "rounded-xl border p-5 transition cursor-pointer",
              agentDraft().runner === "codex_cli"
                ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                : "border-[color:var(--border)] bg-[color:var(--panel)] hover:border-[color:var(--accent)]/50",
            ].join(" ")}
            onClick={() => void selectRunner("codex_cli")}
          >
            <div class="flex items-start justify-between gap-3">
              <div>
                <div class="text-sm font-semibold text-[color:var(--text-primary)]">Codex</div>
                <div class="mt-0.5 text-xs text-[color:var(--text-secondary)]">
                  使用本机安装的 <code class="rounded bg-black/20 px-1">codex</code> 命令行 Agent
                </div>
              </div>
              <Show when={agentDraft().runner === "codex_cli"}>
                <span class="shrink-0 rounded-full border border-[color:var(--accent)] px-2 py-0.5 text-[10px] font-medium text-[color:var(--accent)]">当前</span>
              </Show>
            </div>

            <div class="mt-4 space-y-3" onClick={(e) => e.stopPropagation()}>
              {/* 检测状态 */}
              <Show when={codexCheckStatus() !== "idle"}>
                <div
                  class={[
                    "flex items-center gap-2 rounded-lg border px-3 py-2 text-xs",
                    codexCheckStatus() === "checking"
                      ? "border-amber-300/40 bg-amber-500/10 text-amber-300"
                      : codexCheckStatus() === "ok"
                        ? "border-emerald-300/40 bg-emerald-500/10 text-emerald-300"
                        : "border-rose-300/40 bg-rose-500/10 text-rose-300",
                  ].join(" ")}
                >
                  <Show when={codexCheckStatus() === "checking"}>
                    <svg class="h-3.5 w-3.5 shrink-0 animate-spin" viewBox="0 0 24 24" fill="none">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 22 6.477 22 12h-4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                    </svg>
                  </Show>
                  <Show when={codexCheckStatus() === "ok"}>
                    <svg class="h-3.5 w-3.5 shrink-0" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                    </svg>
                  </Show>
                  <Show when={codexCheckStatus() === "error"}>
                    <svg class="h-3.5 w-3.5 shrink-0" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                    </svg>
                  </Show>
                  <span>
                    {codexCheckStatus() === "checking" ? "检测中，请稍候…" : codexCheckMessage()}
                  </span>
                </div>
              </Show>

              <div class="flex gap-2 pt-1">
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs text-[color:var(--text-primary)] transition hover:border-[color:var(--accent)]/60 disabled:opacity-50"
                  disabled={codexCheckStatus() === "checking"}
                  onClick={() => void handleCheckCodex()}
                >
                  {codexCheckStatus() === "checking" ? "检测中…" : "测试联通"}
                </button>
              </div>
            </div>
          </div>

        </fieldset>
      </div>

      {/* ── 2. API 配置 ── */}
      <div id="api-settings" class="rounded-2xl border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm">
        <div class="flex items-center gap-3">
          <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-amber-500/10 text-amber-500 font-bold">
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z" />
              <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
              <line x1="12" y1="22.08" x2="12" y2="12" />
            </svg>
          </div>
          <h1 class="text-xl font-bold text-[color:var(--text-primary)]">API 配置</h1>
        </div>
        <p class="mt-2 text-sm text-[color:var(--text-secondary)]">配置各类数据源和搜索服务的密钥。支持多 Key 轮换及自动重试。</p>

        <div class="mt-8 space-y-6">
          {/* FMP Subsection */}
          <div class="rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="flex h-6 w-6 items-center justify-center rounded bg-emerald-500/10 text-emerald-500 font-extrabold text-[10px]">FMP</div>
                <div>
                  <div class="text-sm font-bold text-[color:var(--text-primary)]">金融数据 API (Financial Modeling Prep)</div>
                  <div class="mt-0.5 text-[10px] text-[color:var(--text-secondary)]">用于获取实时股票、报表等金融核心数据</div>
                </div>
              </div>
              <input type="checkbox" checked={true} disabled class="h-3.5 w-3.5 rounded border-[color:var(--border)] text-[color:var(--accent)]" />
            </div>
            <form class="mt-4 space-y-4" onSubmit={(event) => void submitFmpSettings(event)}>
              <fieldset disabled={!backend.state.isDesktop || fmpSettingsRes.loading} class="space-y-3">
                <Index each={fmpDraft().apiKeys}>
                  {(key, index) => (
                    <div class="flex items-center gap-2">
                       <div class="relative flex-1">
                        <input
                          type={showFmpKeys()[index] ? "text" : "password"}
                          placeholder="FMP API Key"
                          class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                          value={key()}
                          onInput={(e) => updateKey(setFmpDraft, index, e.currentTarget.value)}
                        />
                        <button
                          type="button"
                          class="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                          onClick={() => toggleShowKey(setShowFmpKeys, index)}
                        >
                          <Show when={showFmpKeys()[index]} fallback={<svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>}>
                            <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.046m4.596-4.596A9.964 9.964 0 0112 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>
                          </Show>
                        </button>
                      </div>
                      <Show when={fmpDraft().apiKeys.length > 1}>
                        <button type="button" class="text-xs text-rose-500 px-2 font-medium" onClick={() => removeKey(setFmpDraft, setShowFmpKeys, index)}>删除</button>
                      </Show>
                    </div>
                  )}
                </Index>
                <div class="flex items-center justify-between pt-1">
                  <button type="button" class="text-[10px] font-bold text-[color:var(--accent)]" onClick={() => addKey(setFmpDraft, setShowFmpKeys)}>+ 添加 Key</button>
                  <button type="submit" class="rounded bg-[color:var(--accent)] px-3 py-1 text-xs font-bold text-white shadow-sm" disabled={fmpSaving()}>{fmpSaving() ? "保存中..." : "保存 FMP"}</button>
                </div>
              </fieldset>
            </form>
          </div>

          {/* Tavily Subsection */}
          <div class="rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="flex h-6 w-6 items-center justify-center rounded bg-blue-500/10 text-blue-500 font-extrabold text-[10px]">TAV</div>
                <div>
                  <div class="text-sm font-bold text-[color:var(--text-primary)]">搜索 API (Tavily)</div>
                  <div class="mt-0.5 text-[10px] text-[color:var(--text-secondary)]">用于联网获取最新信息、文章、网页内容</div>
                </div>
              </div>
              <input type="checkbox" checked={true} disabled class="h-3.5 w-3.5 rounded border-[color:var(--border)] text-[color:var(--accent)]" />
            </div>
            <form class="mt-4 space-y-4" onSubmit={(event) => void submitTavilySettings(event)}>
              <fieldset disabled={!backend.state.isDesktop || tavilySettingsRes.loading} class="space-y-3">
                <Index each={tavilyDraft().apiKeys}>
                  {(key, index) => (
                    <div class="flex items-center gap-2">
                       <div class="relative flex-1">
                        <input
                          type={showTavilyKeys()[index] ? "text" : "password"}
                          placeholder="tvly-..."
                          class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                          value={key()}
                          onInput={(e) => updateKey(setTavilyDraft, index, e.currentTarget.value)}
                        />
                        <button
                          type="button"
                          class="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                          onClick={() => toggleShowKey(setShowTavilyKeys, index)}
                        >
                          <Show when={showTavilyKeys()[index]} fallback={<svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>}>
                            <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.046m4.596-4.596A9.964 9.964 0 0112 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>
                          </Show>
                        </button>
                      </div>
                      <Show when={tavilyDraft().apiKeys.length > 1}>
                        <button type="button" class="text-xs text-rose-500 px-2 font-medium" onClick={() => removeKey(setTavilyDraft, setShowTavilyKeys, index)}>删除</button>
                      </Show>
                    </div>
                  )}
                </Index>
                <div class="flex items-center justify-between pt-1">
                  <button type="button" class="text-[10px] font-bold text-[color:var(--accent)]" onClick={() => addKey(setTavilyDraft, setShowTavilyKeys)}>+ 添加 Key</button>
                  <button type="submit" class="rounded bg-[color:var(--accent)] px-3 py-1 text-xs font-bold text-white shadow-sm" disabled={tavilySaving()}>{tavilySaving() ? "保存中..." : "保存 Tavily"}</button>
                </div>
              </fieldset>
            </form>
          </div>
        </div>
      </div>


      {/* ── 3. 渠道设置 ── */}
      <div id="channel-settings" class="rounded-2xl border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm">
        <form onSubmit={(event) => void submitChannels(event)}>
          <fieldset disabled={!backend.state.isDesktop || desktopChannelSettings.loading} class="space-y-6 disabled:opacity-60">
            <div class="flex items-start justify-between gap-4">
              <div class="flex items-center gap-3">
                <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-sky-500/10 text-sky-500 font-bold">
                  <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" />
                  </svg>
                </div>
                <div>
                  <h1 class="text-xl font-bold text-[color:var(--text-primary)]">渠道设置</h1>
                  <p class="mt-1 text-sm text-[color:var(--text-secondary)]">开启后 Hone 会通过对应渠道监听消息并进行 Agent 响应。</p>
                </div>
              </div>
              <button
                type="button"
                class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs text-[color:var(--text-primary)] transition hover:bg-black/5"
                onClick={() => void refetchDesktopChannelSettings()}
              >
                刷新配置
              </button>
            </div>

            <div class="grid gap-6 md:grid-cols-2">
              {/* Feishu */}
              <div class="space-y-4 rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-3">
                    <div class="flex h-10 w-10 items-center justify-center rounded-full bg-[#3370ff]/10 text-[#3370ff]">
                      <svg class="h-6 w-6" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.5 9h-9c-.28 0-.5-.22-.5-.5s.22-.5.5-.5h9c.28 0 .5.22.5.5s-.22.5-.5.5zm0 3h-9c-.28 0-.5-.22-.5-.5s.22-.5.5-.5h9c.28 0 .5.22.5.5s-.22.5-.5.5z" />
                      </svg>
                    </div>
                    <div class="font-bold text-[color:var(--text-primary)]">飞书 (Feishu)</div>
                  </div>
                  <label class="relative inline-flex cursor-pointer items-center">
                    <input
                      type="checkbox"
                      class="peer sr-only"
                      checked={channelDraft().feishuEnabled}
                      onChange={(e) => setChannelDraft(p => ({ ...p, feishuEnabled: e.currentTarget.checked }))}
                    />
                    <div class="peer h-5 w-9 rounded-full bg-gray-200 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-all after:content-[''] peer-checked:bg-[color:var(--accent)] peer-checked:after:translate-x-full dark:bg-gray-700"></div>
                  </label>
                </div>
                <Show when={channelDraft().feishuEnabled}>
                  <div class="space-y-3 pt-2">
                    <div class="space-y-1">
                      <label class="text-[10px] font-bold uppercase tracking-wider text-[color:var(--text-secondary)]">App ID</label>
                      <input
                        type="text"
                        placeholder="cli_..."
                        class="w-full rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1.5 text-xs text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                        value={channelDraft().feishuAppId || ""}
                        onInput={(e) => setChannelDraft(p => ({ ...p, feishuAppId: e.currentTarget.value }))}
                      />
                    </div>
                    <div class="space-y-1">
                      <label class="text-[10px] font-bold uppercase tracking-wider text-[color:var(--text-secondary)]">App Secret</label>
                      <div class="relative">
                        <input
                          type={showFeishuSecret() ? "text" : "password"}
                          placeholder="Secret"
                          class="w-full rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1.5 pr-14 text-xs text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                          value={channelDraft().feishuAppSecret || ""}
                          onInput={(e) => setChannelDraft(p => ({ ...p, feishuAppSecret: e.currentTarget.value }))}
                        />
                        <button
                          type="button"
                          class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-1.5 py-0.5 text-[10px] text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                          onClick={() => setShowFeishuSecret((v) => !v)}
                        >
                          {showFeishuSecret() ? "隐藏" : "显示"}
                        </button>
                      </div>
                    </div>
                  </div>
                </Show>
              </div>

              {/* Discord */}
              <div class="space-y-4 rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-3">
                    <div class="flex h-10 w-10 items-center justify-center rounded-full bg-[#5865F2]/10 text-[#5865F2]">
                      <svg class="h-6 w-6" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M20.317 4.37c-1.215-.503-2.546-.882-3.932-1.057a.06.06 0 00-.063.03c-.157.28-.344.66-.464.945-1.497-.225-2.991-.225-4.463 0-.12-.285-.312-.665-.472-.945a.061.061 0 00-.063-.03 15.343 15.343 0 00-3.931 1.056.052.052 0 00-.024.02C4.195 7.42 2.91 10.375 3.328 13.25a.066.066 0 00.026.046 15.485 15.485 0 004.757 2.413.064.064 0 00.069-.022c.36-.492.684-1.02.954-1.574a.062.062 0 00-.034-.085c-.504-.19-1.002-.42-1.468-.69a.065.065 0 01-.006-.109c.097-.074.196-.15.291-.228a.063.063 0 01.066-.009 11.2 11.2 0 009.11 0 .063.063 0 01.067.01c.094.077.193.153.29.227a.065.065 0 01-.006.11c-.465.269-.963.499-1.467.689a.061.061 0 00-.034.086c.27.554.594 1.082.955 1.574a.063.063 0 00.068.022 15.441 15.441 0 004.759-2.413.06.06 0 00.026-.046c.491-3.415-.843-6.33-3.11-8.86a.052.052 0 00-.023-.02zM8.02 11.08c-.908 0-1.657-.84-1.657-1.87 0-1.03.731-1.87 1.657-1.87.935 0 1.666.84 1.657 1.87 0 1.03-.731 1.87-1.657 1.87zm7.96 0c-.908 0-1.657-.84-1.657-1.87 0-1.03.731-1.87 1.657-1.87.935 0 1.666.84 1.657 1.87 0 1.03-.732 1.87-1.657 1.87z" />
                      </svg>
                    </div>
                    <div class="font-bold text-[color:var(--text-primary)]">Discord</div>
                  </div>
                  <label class="relative inline-flex cursor-pointer items-center">
                    <input
                      type="checkbox"
                      class="peer sr-only"
                      checked={channelDraft().discordEnabled}
                      onChange={(e) => setChannelDraft(p => ({ ...p, discordEnabled: e.currentTarget.checked }))}
                    />
                    <div class="peer h-5 w-9 rounded-full bg-gray-200 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-all after:content-[''] peer-checked:bg-[color:var(--accent)] peer-checked:after:translate-x-full dark:bg-gray-700"></div>
                  </label>
                </div>
                <Show when={channelDraft().discordEnabled}>
                  <div class="space-y-1 pt-2">
                    <label class="text-[10px] font-bold uppercase tracking-wider text-[color:var(--text-secondary)]">Bot Token</label>
                    <div class="relative">
                      <input
                        type={showDiscordToken() ? "text" : "password"}
                        placeholder="Discord Bot Token"
                        class="w-full rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1.5 pr-14 text-xs text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                        value={channelDraft().discordBotToken || ""}
                        onInput={(e) => setChannelDraft(p => ({ ...p, discordBotToken: e.currentTarget.value }))}
                      />
                      <button
                        type="button"
                        class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-1.5 py-0.5 text-[10px] text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                        onClick={() => setShowDiscordToken((v) => !v)}
                      >
                        {showDiscordToken() ? "隐藏" : "显示"}
                      </button>
                    </div>
                  </div>
                </Show>
              </div>

              {/* Telegram */}
              <div class="space-y-4 rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-3">
                    <div class="flex h-10 w-10 items-center justify-center rounded-full bg-[#0088cc]/10 text-[#0088cc]">
                      <svg class="h-6 w-6" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.64 6.8c-.15 1.58-.8 5.42-1.13 7.19-.14.75-.42 1-.68 1.03-.58.05-1.02-.38-1.58-.75-.88-.58-1.38-.94-2.23-1.5-.99-.65-.35-1.01.22-1.59.15-.15 2.71-2.48 2.76-2.69.01-.03.01-.14-.07-.2-.08-.06-.19-.04-.27-.02-.12.02-1.96 1.25-5.54 3.69-.52.36-1 .54-1.43.53-.48-.01-1.4-.27-2.09-.49-.84-.27-1.51-.42-1.45-.88.03-.24.37-.48 1.02-.73 4-1.74 6.67-2.89 8.01-3.44 3.82-1.58 4.61-1.85 5.13-1.86.11 0 .37.03.54.17.14.12.18.28.2.45-.02.07-.02.13-.03.19z" />
                      </svg>
                    </div>
                    <div class="font-bold text-[color:var(--text-primary)]">Telegram</div>
                  </div>
                  <label class="relative inline-flex cursor-pointer items-center">
                    <input
                      type="checkbox"
                      class="peer sr-only"
                      checked={channelDraft().telegramEnabled}
                      onChange={(e) => setChannelDraft(p => ({ ...p, telegramEnabled: e.currentTarget.checked }))}
                    />
                    <div class="peer h-5 w-9 rounded-full bg-gray-200 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-all after:content-[''] peer-checked:bg-[color:var(--accent)] peer-checked:after:translate-x-full dark:bg-gray-700"></div>
                  </label>
                </div>
                <Show when={channelDraft().telegramEnabled}>
                  <div class="space-y-1 pt-2">
                    <label class="text-[10px] font-bold uppercase tracking-wider text-[color:var(--text-secondary)]">Bot Token</label>
                    <div class="relative">
                      <input
                        type={showTelegramToken() ? "text" : "password"}
                        placeholder="Token"
                        class="w-full rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1.5 pr-14 text-xs text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                        value={channelDraft().telegramBotToken || ""}
                        onInput={(e) => setChannelDraft(p => ({ ...p, telegramBotToken: e.currentTarget.value }))}
                      />
                      <button
                        type="button"
                        class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-1.5 py-0.5 text-[10px] text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                        onClick={() => setShowTelegramToken((v) => !v)}
                      >
                        {showTelegramToken() ? "隐藏" : "显示"}
                      </button>
                    </div>
                  </div>
                </Show>
              </div>

              {/* iMessage */}
              <div class="group relative overflow-hidden rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-5 transition-all">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-3">
                    <div class="flex h-10 w-10 items-center justify-center rounded-full bg-[#34C759]/10 text-[#34C759]">
                      <svg class="h-6 w-6" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 2C6.48 2 2 5.92 2 10.74c0 2.22 1.03 4.25 2.74 5.75-.12.44-.74 2.1-1.74 3.5 0 0 2.13 0 4.14-1.22.88.24 1.83.37 2.86.37 5.52 0 10-3.92 10-8.74S17.52 2 12 2z" />
                      </svg>
                    </div>
                    <div>
                      <div class="font-bold text-[color:var(--text-primary)]">iMessage</div>
                      <div class="text-[10px] font-bold text-amber-600">⚠️ Needs Full Disk Access</div>
                    </div>
                  </div>
                  <label class="relative inline-flex cursor-pointer items-center">
                    <input
                      type="checkbox"
                      class="peer sr-only"
                      checked={channelDraft().imessageEnabled}
                      onChange={(e) => setChannelDraft(p => ({ ...p, imessageEnabled: e.currentTarget.checked }))}
                    />
                    <div class="peer h-5 w-9 rounded-full bg-gray-200 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-all after:content-[''] peer-checked:bg-[color:var(--accent)] peer-checked:after:translate-x-full dark:bg-gray-700"></div>
                  </label>
                </div>
              </div>
            </div>

            <div class="mt-8 flex items-center justify-between border-t border-[color:var(--border)] pt-6">
              <div class="text-xs text-[color:var(--text-secondary)]">同步设置将立即生效。</div>
              <button
                type="submit"
                class="rounded-md bg-[color:var(--accent)] px-6 py-2 text-sm font-bold text-white shadow-sm transition-all hover:opacity-90 active:scale-95 disabled:opacity-50"
                disabled={backend.state.saving}
              >
                {backend.state.saving ? "同步中..." : "同步全部渠道"}
              </button>
            </div>
          </fieldset>
        </form>
      </div>
    </div>
  )
}
