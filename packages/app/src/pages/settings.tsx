import {
  For,
  Index,
  Show,
  createEffect,
  createMemo,
  createResource,
  createSignal,
} from "solid-js";
import { useSearchParams } from "@solidjs/router";
import { useBackend } from "@/context/backend";
import {
  checkDesktopAgentCli,
  loadDesktopAgentSettings,
  testDesktopOpenAiChannel,
  loadDesktopFmpSettings,
  saveDesktopFmpSettings,
  loadDesktopTavilySettings,
  saveDesktopTavilySettings,
} from "@/lib/backend";
import {
  createWebInvite,
  disableWebInvite,
  enableWebInvite,
  getWebInvites,
  getWebInviteApiKey,
  resetWebInvite,
  resetWebInviteApiKey,
} from "@/lib/api";
import { NotificationPreferencesCard } from "@/components/notification-preferences-card";
import type {
  AgentProvider,
  AgentSettings,
  BackendConfig,
  DesktopChannelSettingsInput,
  FmpSettings,
  TavilySettings,
  WebInviteInfo,
} from "@/lib/types";
import {
  appendApiKey,
  appendMaskedKey,
  canSelectRunner,
  defaultAgentSettings,
  defaultChannelDraft,
  defaultFmpSettings,
  defaultLanguageDraft,
  defaultTavilySettings,
  hiddenApiKeys,
  isAgentSettingsRuntimeMismatch,
  mergeAgentSettings,
  normalizeApiKeys,
  removeApiKey,
  removeMaskedKey,
  resolveHoneCloudOpenAiBaseUrl,
  toChannelDraft,
  toggleMaskedKey,
  updateApiKeyList,
  type LanguageDraft,
} from "@/pages/settings-model";
import { SETTINGS } from "@/lib/admin-content/settings";
import { tpl } from "@/lib/i18n";

function normalizePhoneNumber(value: string) {
  const trimmed = value.trim();
  const hasLeadingPlus = trimmed.startsWith("+");
  const digits = trimmed.replace(/\D+/g, "");
  return hasLeadingPlus ? `+${digits}` : digits;
}

export default function SettingsPage() {
  const backend = useBackend();
  const [draft, setDraft] = createSignal<BackendConfig>(backend.state.config);
  const [channelDraft, setChannelDraft] =
    createSignal<DesktopChannelSettingsInput>(defaultChannelDraft());
  const [channelMessage, setChannelMessage] = createSignal("");
  const [channelError, setChannelError] = createSignal("");
  const capabilities = createMemo(() => backend.state.meta?.capabilities ?? []);
  const [
    desktopChannelSettings,
    {
      refetch: refetchDesktopChannelSettings,
      mutate: setDesktopChannelSettings,
    },
  ] = createResource(
    () => backend.state.isDesktop,
    async (isDesktop) => {
      if (!isDesktop) return undefined;
      return backend.loadChannelSettings();
    },
  );

  // ── 界面语言 ────────────────────────────────────────────────────────────────
  const [languageDraft, setLanguageDraft] = createSignal<LanguageDraft>(
    defaultLanguageDraft(backend.state.meta),
  );
  const [languageSaving, setLanguageSaving] = createSignal(false);
  const [languageMessage, setLanguageMessage] = createSignal("");
  const [languageError, setLanguageError] = createSignal("");
  createEffect(() => {
    // Re-sync draft whenever the canonical meta language changes (e.g. after
    // a save round-trip or another device pushed an update on reconnect).
    setLanguageDraft(defaultLanguageDraft(backend.state.meta));
  });
  const languageDirty = createMemo(
    () => languageDraft() !== defaultLanguageDraft(backend.state.meta),
  );
  const submitLanguage = async (event: Event) => {
    event.preventDefault();
    setLanguageSaving(true);
    setLanguageMessage("");
    setLanguageError("");
    try {
      await backend.saveLanguage(languageDraft());
      setLanguageMessage(SETTINGS.language.saved);
    } catch (e) {
      setLanguageError(e instanceof Error ? e.message : String(e));
    } finally {
      setLanguageSaving(false);
    }
  };

  // ── Agent 基础设置 ──────────────────────────────────────────────────────────
  const [agentDraft, setAgentDraft] = createSignal<AgentSettings>(
    defaultAgentSettings(),
  );
  const [agentSaving, setAgentSaving] = createSignal(false);
  const [agentMessage, setAgentMessage] = createSignal("");
  const [agentError, setAgentError] = createSignal("");

  // OpenAI 协议渠道测试状态
  const [openaiTestStatus, setOpenaiTestStatus] = createSignal<
    "idle" | "checking" | "ok" | "error"
  >("idle");
  const [openaiTestMessage, setOpenaiTestMessage] = createSignal("");
  const [honeCloudTestStatus, setHoneCloudTestStatus] = createSignal<
    "idle" | "checking" | "ok" | "error"
  >("idle");
  const [honeCloudTestMessage, setHoneCloudTestMessage] = createSignal("");
  const [showHoneCloudKey, setShowHoneCloudKey] = createSignal(false);
  const [showOpenaiKey, setShowOpenaiKey] = createSignal(false);
  const [auxiliaryTestStatus, setAuxiliaryTestStatus] = createSignal<
    "idle" | "checking" | "ok" | "error"
  >("idle");
  const [auxiliaryTestMessage, setAuxiliaryTestMessage] = createSignal("");
  const [showAuxiliaryKey, setShowAuxiliaryKey] = createSignal(false);
  const [showFeishuSecret, setShowFeishuSecret] = createSignal(false);
  const [showTelegramToken, setShowTelegramToken] = createSignal(false);
  const [showDiscordToken, setShowDiscordToken] = createSignal(false);
  const [inviteMessage, setInviteMessage] = createSignal("");
  const [inviteError, setInviteError] = createSignal("");
  const [inviteCreating, setInviteCreating] = createSignal(false);
  const [inviteActionKey, setInviteActionKey] = createSignal("");
  const [invitePhoneNumber, setInvitePhoneNumber] = createSignal("");

  const [webInvites, { refetch: refetchWebInvites, mutate: setWebInvites }] =
    createResource(
      () => backend.state.connected && backend.hasCapability("web_invites"),
      async (enabled) => {
        if (!enabled) return [];
        return getWebInvites();
      },
    );

  // Gemini CLI 检测状态
  const [geminiCheckStatus, setGeminiCheckStatus] = createSignal<
    "idle" | "checking" | "ok" | "error"
  >("idle");
  const [geminiCheckMessage, setGeminiCheckMessage] = createSignal("");

  const [codexAcpCheckStatus, setCodexAcpCheckStatus] = createSignal<
    "idle" | "checking" | "ok" | "error"
  >("idle");
  const [codexAcpCheckMessage, setCodexAcpCheckMessage] = createSignal("");

  const [agentSettingsRes] = createResource(
    () => backend.state.isDesktop,
    async (isDesktop) => {
      if (!isDesktop) return undefined;
      return loadDesktopAgentSettings();
    },
  );

  createEffect(() => {
    const s = agentSettingsRes();
    if (s) {
      setAgentDraft(mergeAgentSettings(s));
    }
  });

  // ── FMP API Keys 设置 ───────────────────────────────────────────────────────
  const [fmpDraft, setFmpDraft] =
    createSignal<FmpSettings>(defaultFmpSettings());
  const [fmpSaving, setFmpSaving] = createSignal(false);
  const [fmpMessage, setFmpMessage] = createSignal("");
  const [fmpError, setFmpError] = createSignal("");
  const [showFmpKeys, setShowFmpKeys] = createSignal<boolean[]>([false]);

  const [fmpSettingsRes] = createResource(
    () => backend.state.isDesktop,
    async (isDesktop) => {
      if (!isDesktop) return undefined;
      return loadDesktopFmpSettings();
    },
  );

  createEffect(() => {
    const s = fmpSettingsRes();
    if (s) {
      const keys = normalizeApiKeys(s.apiKeys);
      setFmpDraft({ apiKeys: keys });
      setShowFmpKeys(hiddenApiKeys(keys));
    }
  });

  const submitFmpSettings = async (event: Event) => {
    event.preventDefault();
    setFmpSaving(true);
    setFmpMessage("");
    setFmpError("");
    try {
      await saveDesktopFmpSettings(fmpDraft());
      setFmpMessage(SETTINGS.data.fmp.saved);
    } catch (e) {
      setFmpError(e instanceof Error ? e.message : String(e));
    } finally {
      setFmpSaving(false);
    }
  };

  // ── Tavily API Keys 设置 ────────────────────────────────────────────────────
  const [tavilyDraft, setTavilyDraft] = createSignal<TavilySettings>(
    defaultTavilySettings(),
  );
  const [tavilySaving, setTavilySaving] = createSignal(false);
  const [tavilyMessage, setTavilyMessage] = createSignal("");
  const [tavilyError, setTavilyError] = createSignal("");
  const [showTavilyKeys, setShowTavilyKeys] = createSignal<boolean[]>([false]);

  const [tavilySettingsRes] = createResource(
    () => backend.state.isDesktop,
    async (isDesktop) => {
      if (!isDesktop) return undefined;
      return loadDesktopTavilySettings();
    },
  );

  createEffect(() => {
    const s = tavilySettingsRes();
    if (s) {
      const keys = normalizeApiKeys(s.apiKeys);
      setTavilyDraft({ apiKeys: keys });
      setShowTavilyKeys(hiddenApiKeys(keys));
    }
  });

  const submitTavilySettings = async (event: Event) => {
    event.preventDefault();
    setTavilySaving(true);
    setTavilyMessage("");
    setTavilyError("");
    try {
      await saveDesktopTavilySettings(tavilyDraft());
      setTavilyMessage(SETTINGS.data.tavily.saved);
    } catch (e) {
      setTavilyError(e instanceof Error ? e.message : String(e));
    } finally {
      setTavilySaving(false);
    }
  };

  // ── 多 Key 输入辅助函数 ──────────────────────────────────────────────────────
  /** 更新指定索引的 key 值 */
  function updateKey<T extends { apiKeys: string[] }>(
    setter: (fn: (prev: T) => T) => void,
    index: number,
    value: string,
  ) {
    setter((prev) => updateApiKeyList(prev, index, value));
  }

  /** 追加一个空 key 输入行 */
  function addKey<T extends { apiKeys: string[] }>(
    setter: (fn: (prev: T) => T) => void,
    showSetter: (fn: (prev: boolean[]) => boolean[]) => void,
  ) {
    setter((prev) => appendApiKey(prev));
    showSetter((prev) => appendMaskedKey(prev));
  }

  /** 删除指定索引的 key */
  function removeKey<T extends { apiKeys: string[] }>(
    setter: (fn: (prev: T) => T) => void,
    showSetter: (fn: (prev: boolean[]) => boolean[]) => void,
    index: number,
  ) {
    setter((prev) => removeApiKey(prev, index));
    showSetter((prev) => removeMaskedKey(prev, index));
  }

  /** 切换指定索引的 key 显示/隐藏 */
  function toggleShowKey(
    showSetter: (fn: (prev: boolean[]) => boolean[]) => void,
    index: number,
  ) {
    showSetter((prev) => toggleMaskedKey(prev, index));
  }

  // ── OpenAI 协议渠道测试 ──────────────────────────────────────────────────────
  const handleTestOpenAi = async () => {
    setOpenaiTestStatus("checking");
    setOpenaiTestMessage("");
    try {
      const d = agentDraft();
      const result = await testDesktopOpenAiChannel(
        d.openaiUrl,
        d.openaiModel,
        d.openaiApiKey,
      );
      setOpenaiTestStatus(result.ok ? "ok" : "error");
      setOpenaiTestMessage(result.message);
    } catch (e) {
      setOpenaiTestStatus("error");
      setOpenaiTestMessage(e instanceof Error ? e.message : String(e));
    }
  };

  const handleTestHoneCloud = async () => {
    setHoneCloudTestStatus("checking");
    setHoneCloudTestMessage("");
    try {
      const d = agentDraft().honeCloud;
      const result = await testDesktopOpenAiChannel(
        resolveHoneCloudOpenAiBaseUrl(d?.baseUrl),
        d?.model || "hone-cloud",
        d?.apiKey ?? "",
      );
      setHoneCloudTestStatus(result.ok ? "ok" : "error");
      setHoneCloudTestMessage(result.message);
    } catch (e) {
      setHoneCloudTestStatus("error");
      setHoneCloudTestMessage(e instanceof Error ? e.message : String(e));
    }
  };

  const handleTestAuxiliary = async () => {
    setAuxiliaryTestStatus("checking");
    setAuxiliaryTestMessage("");
    try {
      const auxiliary = agentDraft().auxiliary;
      const result = await testDesktopOpenAiChannel(
        auxiliary?.baseUrl ?? "",
        auxiliary?.model ?? "",
        auxiliary?.apiKey ?? "",
      );
      setAuxiliaryTestStatus(result.ok ? "ok" : "error");
      setAuxiliaryTestMessage(result.message);
    } catch (e) {
      setAuxiliaryTestStatus("error");
      setAuxiliaryTestMessage(e instanceof Error ? e.message : String(e));
    }
  };

  // ── Gemini CLI 检测 ──────────────────────────────────────────────────────────
  const handleCheckGemini = async () => {
    setGeminiCheckStatus("checking");
    setGeminiCheckMessage("");
    try {
      const result = await checkDesktopAgentCli("gemini_cli");
      setGeminiCheckStatus(result.ok ? "ok" : "error");
      setGeminiCheckMessage(result.message);
    } catch (e) {
      setGeminiCheckStatus("error");
      setGeminiCheckMessage(e instanceof Error ? e.message : String(e));
    }
  };

  const handleCheckCodexAcp = async () => {
    setCodexAcpCheckStatus("checking");
    setCodexAcpCheckMessage("");
    try {
      const result = await checkDesktopAgentCli("codex_acp");
      setCodexAcpCheckStatus(result.ok ? "ok" : "error");
      setCodexAcpCheckMessage(result.message);
    } catch (e) {
      setCodexAcpCheckStatus("error");
      setCodexAcpCheckMessage(e instanceof Error ? e.message : String(e));
    }
  };

  // ── 选中某个 runner 并立即保存 ───────────────────────────────────────────────
  const selectRunner = async (runner: AgentProvider) => {
    const previous = agentDraft();
    if (!canSelectRunner(previous.runner, runner, agentSaving())) return;
    const next = { ...previous, runner };
    setAgentDraft(next);
    setAgentSaving(true);
    setAgentMessage("");
    setAgentError("");
    try {
      const result = await backend.saveAgentSettings(next);
      if (isAgentSettingsRuntimeMismatch(result)) {
        setAgentError(result.message);
      } else {
        setAgentMessage(result.message);
      }
    } catch (e) {
      setAgentDraft(previous);
      setAgentError(e instanceof Error ? e.message : String(e));
    } finally {
      setAgentSaving(false);
    }
  };

  const submitAgentSettings = async (event: Event) => {
    event.preventDefault();
    setAgentSaving(true);
    setAgentMessage("");
    setAgentError("");
    try {
      const result = await backend.saveAgentSettings(agentDraft());
      if (isAgentSettingsRuntimeMismatch(result)) {
        setAgentError(result.message);
      } else {
        setAgentMessage(result.message);
      }
    } catch (e) {
      setAgentError(e instanceof Error ? e.message : String(e));
    } finally {
      setAgentSaving(false);
    }
  };

  createEffect(() => {
    setDraft(backend.state.config);
  });

  createEffect(() => {
    const settings = desktopChannelSettings();
    if (!settings) return;
    setChannelDraft(toChannelDraft(settings));
  });

  const submit = async (event: Event) => {
    event.preventDefault();
    await backend.saveConfig(draft());
  };

  const submitChannels = async (event: Event) => {
    event.preventDefault();
    setChannelMessage("");
    setChannelError("");
    try {
      const result = await backend.saveChannelSettings(channelDraft());
      setDesktopChannelSettings(result.settings);
      setChannelMessage(result.message);
    } catch (error) {
      setChannelError(error instanceof Error ? error.message : String(error));
    }
  };

  const handleCreateInvite = async () => {
    const phoneNumber = normalizePhoneNumber(invitePhoneNumber());
    if (!phoneNumber) {
      setInviteMessage("");
      setInviteError(SETTINGS.invite.phone_required);
      return;
    }
    setInviteCreating(true);
    setInviteMessage("");
    setInviteError("");
    try {
      const created = await createWebInvite(phoneNumber);
      setWebInvites((current = []) => [created, ...current]);
      setInvitePhoneNumber("");
      setInviteMessage(
        tpl(created.api_key ? SETTINGS.invite.created_with_api_key : SETTINGS.invite.created, {
          phone: created.phone_number,
          code: created.invite_code,
          apiKey: created.api_key ?? "",
        }),
      );
      if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(
          created.api_key
            ? `Invite: ${created.invite_code}\nAPI Key: ${created.api_key}`
            : created.invite_code,
        );
        setInviteMessage(
          tpl(
            created.api_key
              ? SETTINGS.invite.created_with_api_key_copied
              : SETTINGS.invite.created_copied,
            {
            phone: created.phone_number,
            code: created.invite_code,
              apiKey: created.api_key ?? "",
            },
          ),
        );
      }
    } catch (error) {
      setInviteError(error instanceof Error ? error.message : String(error));
    } finally {
      setInviteCreating(false);
    }
  };

  const copyInviteCode = async (code: string) => {
    setInviteMessage("");
    setInviteError("");
    try {
      if (!navigator.clipboard?.writeText) {
        throw new Error(SETTINGS.invite.copy_unsupported);
      }
      await navigator.clipboard.writeText(code);
      setInviteMessage(tpl(SETTINGS.invite.copied, { code }));
    } catch (error) {
      setInviteError(error instanceof Error ? error.message : String(error));
    }
  };

  const replaceInvite = (next: WebInviteInfo) => {
    setWebInvites((current = []) =>
      current.map((invite) =>
        invite.user_id === next.user_id ? next : invite,
      ),
    );
  };

  const isInviteActionRunning = (
    userId: string,
    action: "disable" | "enable" | "reset" | "api-key" | "api-key-reset",
  ) => inviteActionKey() === `${userId}:${action}`;

  const handleDisableInvite = async (invite: WebInviteInfo) => {
    if (typeof window !== "undefined") {
      const confirmed = window.confirm(
        tpl(SETTINGS.invite.disable_confirm, { userId: invite.user_id }),
      );
      if (!confirmed) return;
    }
    setInviteMessage("");
    setInviteError("");
    setInviteActionKey(`${invite.user_id}:disable`);
    try {
      const result = await disableWebInvite(invite.user_id);
      replaceInvite(result.invite);
      setInviteMessage(result.message);
    } catch (error) {
      setInviteError(error instanceof Error ? error.message : String(error));
    } finally {
      setInviteActionKey("");
    }
  };

  const handleEnableInvite = async (invite: WebInviteInfo) => {
    setInviteMessage("");
    setInviteError("");
    setInviteActionKey(`${invite.user_id}:enable`);
    try {
      const result = await enableWebInvite(invite.user_id);
      replaceInvite(result.invite);
      setInviteMessage(result.message);
    } catch (error) {
      setInviteError(error instanceof Error ? error.message : String(error));
    } finally {
      setInviteActionKey("");
    }
  };

  const handleResetInvite = async (invite: WebInviteInfo) => {
    if (typeof window !== "undefined") {
      const confirmed = window.confirm(
        tpl(SETTINGS.invite.reset_confirm, { userId: invite.user_id }),
      );
      if (!confirmed) return;
    }
    setInviteMessage("");
    setInviteError("");
    setInviteActionKey(`${invite.user_id}:reset`);
    try {
      const result = await resetWebInvite(invite.user_id);
      replaceInvite(result.invite);
      setInviteMessage(result.message);
      if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(result.invite.invite_code);
        setInviteMessage(
          tpl(SETTINGS.invite.reset_copied_suffix, { message: result.message }),
        );
      }
    } catch (error) {
      setInviteError(error instanceof Error ? error.message : String(error));
    } finally {
      setInviteActionKey("");
    }
  };

  const copyInviteApiKey = async (apiKey: string) => {
    setInviteMessage("");
    setInviteError("");
    try {
      if (!navigator.clipboard?.writeText) {
        throw new Error(SETTINGS.invite.copy_unsupported);
      }
      await navigator.clipboard.writeText(apiKey);
      setInviteMessage(SETTINGS.invite.api_key_copied);
    } catch (error) {
      setInviteError(error instanceof Error ? error.message : String(error));
    }
  };

  const handleGetInviteApiKey = async (invite: WebInviteInfo) => {
    setInviteMessage("");
    setInviteError("");
    setInviteActionKey(`${invite.user_id}:api-key`);
    try {
      const result = await getWebInviteApiKey(invite.user_id);
      replaceInvite(result.invite);
      setInviteMessage(result.message);
      if (result.invite.api_key) {
        await copyInviteApiKey(result.invite.api_key);
      }
    } catch (error) {
      setInviteError(error instanceof Error ? error.message : String(error));
    } finally {
      setInviteActionKey("");
    }
  };

  const handleResetInviteApiKey = async (invite: WebInviteInfo) => {
    if (typeof window !== "undefined") {
      const confirmed = window.confirm(
        tpl(SETTINGS.invite.api_key_reset_confirm, { userId: invite.user_id }),
      );
      if (!confirmed) return;
    }
    setInviteMessage("");
    setInviteError("");
    setInviteActionKey(`${invite.user_id}:api-key-reset`);
    try {
      const result = await resetWebInviteApiKey(invite.user_id);
      replaceInvite(result.invite);
      setInviteMessage(result.message);
      if (result.invite.api_key) {
        await copyInviteApiKey(result.invite.api_key);
      }
    } catch (error) {
      setInviteError(error instanceof Error ? error.message : String(error));
    } finally {
      setInviteActionKey("");
    }
  };

  type TabKey = "agent" | "data" | "notify" | "channel" | "invite";
  const TAB_KEYS: TabKey[] = ["agent", "data", "notify", "channel", "invite"];
  const tabLabel = (key: TabKey): string => SETTINGS.tabs[key];
  const [searchParams, setSearchParams] = useSearchParams<{ tab?: string }>();
  const activeTab = (): TabKey => {
    const raw = searchParams.tab;
    return (TAB_KEYS as string[]).includes(raw ?? "")
      ? (raw as TabKey)
      : "agent";
  };
  const selectTab = (key: TabKey) => setSearchParams({ tab: key });
  let contentRef: HTMLDivElement | undefined;
  createEffect(() => {
    // track active tab and reset scroll on change
    activeTab();
    if (contentRef) contentRef.scrollTop = 0;
  });
  const isTab = (key: TabKey) => activeTab() === key;

  return (
    <div class="mx-auto flex h-full max-w-4xl flex-col">
      <form
        onSubmit={submitLanguage}
        class="mb-3 rounded-2xl border border-[color:var(--border)] bg-[color:var(--surface)] p-5 shadow-sm"
      >
        <div class="flex items-start justify-between gap-4">
          <div class="min-w-0">
            <h2 class="text-base font-semibold text-[color:var(--text-primary)]">
              {SETTINGS.language.title}
            </h2>
            <p class="mt-1 text-xs text-[color:var(--text-secondary)]">
              {SETTINGS.language.subtitle}
            </p>
          </div>
          <button
            type="submit"
            disabled={!languageDirty() || languageSaving()}
            class="shrink-0 rounded-md border border-[color:var(--accent)] bg-[color:var(--accent)] px-3 py-1.5 text-xs font-medium text-white transition disabled:cursor-not-allowed disabled:border-[color:var(--border)] disabled:bg-transparent disabled:text-[color:var(--text-muted)]"
          >
            {languageSaving()
              ? SETTINGS.language.saving
              : SETTINGS.language.save}
          </button>
        </div>
        <div class="mt-3 flex flex-wrap gap-3">
          <For each={["zh", "en"] as const}>
            {(code) => (
              <label
                class={[
                  "flex items-center gap-2 rounded-md border px-3 py-1.5 text-sm cursor-pointer",
                  languageDraft() === code
                    ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)] text-[color:var(--text-primary)]"
                    : "border-[color:var(--border)] bg-[color:var(--panel)] text-[color:var(--text-secondary)] hover:border-[color:var(--accent)]/50",
                ].join(" ")}
              >
                <input
                  type="radio"
                  name="settings-language"
                  value={code}
                  checked={languageDraft() === code}
                  onChange={() => setLanguageDraft(code)}
                  class="h-3.5 w-3.5"
                />
                {code === "zh"
                  ? SETTINGS.language.option_zh
                  : SETTINGS.language.option_en}
              </label>
            )}
          </For>
        </div>
        <p class="mt-2 text-[11px] text-[color:var(--text-muted)]">
          {SETTINGS.language.note}
        </p>
        <Show when={languageMessage()}>
          <p class="mt-2 text-xs text-[color:var(--accent)]">
            {languageMessage()}
          </p>
        </Show>
        <Show when={languageError()}>
          <p class="mt-2 text-xs text-red-500">
            {SETTINGS.language.save_failed}: {languageError()}
          </p>
        </Show>
      </form>
      <nav class="sticky top-0 z-10 -mx-1 flex gap-1 overflow-x-auto border-b border-[color:var(--border)] bg-[color:var(--surface)]/95 px-1 py-2 backdrop-blur">
        <For each={TAB_KEYS}>
          {(key) => (
            <Show
              when={key !== "invite" || backend.hasCapability("web_invites")}
            >
              <button
                type="button"
                onClick={() => selectTab(key)}
                class={[
                  "shrink-0 rounded-md px-4 py-2 text-sm font-medium transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]",
                  isTab(key)
                    ? "bg-[color:var(--accent-soft)] text-[color:var(--text-primary)]"
                    : "text-[color:var(--text-secondary)] hover:bg-black/5 hover:text-[color:var(--text-primary)]",
                ].join(" ")}
              >
                {tabLabel(key)}
              </button>
            </Show>
          )}
        </For>
      </nav>
      <div
        ref={contentRef}
        class="flex flex-1 flex-col gap-4 overflow-y-auto py-4"
      >
      {/* ── 基础设置 ── */}
      <div
        id="agent-settings"
        classList={{ hidden: !isTab("agent") }}
        class="rounded-2xl border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm"
      >
        <h1 class="text-xl font-semibold text-[color:var(--text-primary)]">
          {SETTINGS.agent.title}
        </h1>
        <p class="mt-2 text-sm text-[color:var(--text-secondary)]">
          {SETTINGS.agent.subtitle}
        </p>

        <fieldset
          disabled={
            !backend.state.isDesktop ||
            agentSettingsRes.loading ||
            agentSaving()
          }
          class="mt-6 space-y-4 disabled:opacity-60"
        >
          {/* ── 卡片 0：Hone Cloud ── */}
          <div
            class={[
              "rounded-xl border p-5 transition cursor-pointer",
              agentDraft().runner === "hone_cloud"
                ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                : "border-[color:var(--border)] bg-[color:var(--panel)] hover:border-[color:var(--accent)]/50",
            ].join(" ")}
            onClick={() => void selectRunner("hone_cloud")}
          >
            <div class="flex items-start justify-between gap-3">
              <div>
                <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                  {SETTINGS.agent.hone_cloud.name}
                </div>
                <div class="mt-0.5 text-xs text-[color:var(--text-secondary)]">
                  {SETTINGS.agent.hone_cloud.description}
                </div>
              </div>
              <Show when={agentDraft().runner === "hone_cloud"}>
                <span class="shrink-0 rounded-full border border-[color:var(--accent)] px-2 py-0.5 text-[10px] font-medium text-[color:var(--accent)]">
                  {SETTINGS.agent.current_badge}
                </span>
              </Show>
            </div>

            <div class="mt-4 space-y-3" onClick={(e) => e.stopPropagation()}>
              <div>
                <label
                  class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]"
                  for="hone-cloud-url"
                >
                  {SETTINGS.agent.hone_cloud.base_url_label}
                </label>
                <input
                  id="hone-cloud-url"
                  type="url"
                  placeholder="https://hone-claw.com"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                  value={agentDraft().honeCloud?.baseUrl ?? ""}
                  onInput={(e) =>
                    setAgentDraft((prev) => ({
                      ...prev,
                      honeCloud: {
                        ...(prev.honeCloud ?? {
                          baseUrl: "https://hone-claw.com",
                          apiKey: "",
                          model: "hone-cloud",
                        }),
                        baseUrl: e.currentTarget.value,
                      },
                    }))
                  }
                />
              </div>
              <div>
                <label
                  class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]"
                  for="hone-cloud-model"
                >
                  {SETTINGS.agent.hone_cloud.model_label}
                </label>
                <input
                  id="hone-cloud-model"
                  type="text"
                  placeholder="hone-cloud"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                  value={agentDraft().honeCloud?.model ?? ""}
                  onInput={(e) =>
                    setAgentDraft((prev) => ({
                      ...prev,
                      honeCloud: {
                        ...(prev.honeCloud ?? {
                          baseUrl: "https://hone-claw.com",
                          apiKey: "",
                          model: "hone-cloud",
                        }),
                        model: e.currentTarget.value,
                      },
                    }))
                  }
                />
              </div>
              <div>
                <label
                  class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]"
                  for="hone-cloud-key"
                >
                  {SETTINGS.agent.hone_cloud.api_key_label}
                </label>
                <div class="relative">
                  <input
                    id="hone-cloud-key"
                    type={showHoneCloudKey() ? "text" : "password"}
                    placeholder="hck_..."
                    class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 pr-16 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                    value={agentDraft().honeCloud?.apiKey ?? ""}
                    onInput={(e) =>
                      setAgentDraft((prev) => ({
                        ...prev,
                        honeCloud: {
                          ...(prev.honeCloud ?? {
                            baseUrl: "https://hone-claw.com",
                            apiKey: "",
                            model: "hone-cloud",
                          }),
                          apiKey: e.currentTarget.value,
                        },
                      }))
                    }
                  />
                  <button
                    type="button"
                    class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-2 py-0.5 text-xs text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                    onClick={() => setShowHoneCloudKey((v) => !v)}
                  >
                    {showHoneCloudKey()
                      ? SETTINGS.agent.hone_cloud.hide
                      : SETTINGS.agent.hone_cloud.show}
                  </button>
                </div>
              </div>
              <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface-soft)] p-3 text-xs text-[color:var(--text-secondary)]">
                {SETTINGS.agent.hone_cloud.contact_note}
              </div>
              <Show when={honeCloudTestStatus() !== "idle"}>
                <div
                  class={[
                    "flex items-center gap-2 rounded-lg border px-3 py-2 text-xs",
                    honeCloudTestStatus() === "checking"
                      ? "border-amber-300/40 bg-amber-500/10 text-amber-300"
                      : honeCloudTestStatus() === "ok"
                        ? "border-emerald-300/40 bg-emerald-500/10 text-emerald-300"
                        : "border-rose-300/40 bg-rose-500/10 text-rose-300",
                  ].join(" ")}
                >
                  <span>
                    {honeCloudTestStatus() === "checking"
                      ? SETTINGS.agent.hone_cloud.connection_testing_status
                      : honeCloudTestMessage()}
                  </span>
                </div>
              </Show>
              <div class="flex gap-2 pt-1">
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs text-[color:var(--text-primary)] transition hover:border-[color:var(--accent)]/60 disabled:opacity-50"
                  disabled={honeCloudTestStatus() === "checking"}
                  onClick={() => void handleTestHoneCloud()}
                >
                  {honeCloudTestStatus() === "checking"
                    ? SETTINGS.agent.hone_cloud.testing
                    : SETTINGS.agent.hone_cloud.test_connection}
                </button>
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--accent)] bg-[color:var(--accent-soft)] px-3 py-1.5 text-xs font-medium text-[color:var(--text-primary)] transition hover:opacity-90 disabled:opacity-50"
                  disabled={agentSaving()}
                  onClick={(e) => void submitAgentSettings(e)}
                >
                  {agentSaving()
                    ? SETTINGS.agent.hone_cloud.saving
                    : SETTINGS.agent.hone_cloud.save}
                </button>
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
                <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                  {SETTINGS.agent.openai.name}
                </div>
                <div class="mt-0.5 text-xs text-[color:var(--text-secondary)]">
                  {SETTINGS.agent.openai.description}
                </div>
              </div>
              <Show when={agentDraft().runner === "opencode_acp"}>
                <span class="shrink-0 rounded-full border border-[color:var(--accent)] px-2 py-0.5 text-[10px] font-medium text-[color:var(--accent)]">
                  {SETTINGS.agent.current_badge}
                </span>
              </Show>
            </div>

            {/* 配置字段区（点击卡片内部不触发 selectRunner） */}
            <div class="mt-4 space-y-3" onClick={(e) => e.stopPropagation()}>
              {/* Base URL */}
              <div>
                <label
                  class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]"
                  for="openai-url"
                >
                  {SETTINGS.agent.openai.base_url_label}
                </label>
                <input
                  id="openai-url"
                  type="url"
                  placeholder="https://openrouter.ai/api/v1"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                  value={agentDraft().openaiUrl}
                  onInput={(e) =>
                    setAgentDraft((prev) => ({
                      ...prev,
                      openaiUrl: e.currentTarget.value,
                    }))
                  }
                />
              </div>

              {/* Model */}
              <div>
                <label
                  class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]"
                  for="openai-model"
                >
                  {SETTINGS.agent.openai.model_label}
                </label>
                <input
                  id="openai-model"
                  type="text"
                  placeholder="google/gemini-2.5-pro-preview"
                  class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                  value={agentDraft().openaiModel}
                  onInput={(e) =>
                    setAgentDraft((prev) => ({
                      ...prev,
                      openaiModel: e.currentTarget.value,
                    }))
                  }
                />
              </div>

              <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface-soft)] p-3">
                <p class="text-xs font-medium text-[color:var(--text-primary)]">
                  {SETTINGS.agent.openai.auxiliary_title}
                </p>
                <p class="mt-1 text-[11px] text-[color:var(--text-muted)]">
                  {SETTINGS.agent.openai.auxiliary_subtitle}
                </p>
                <div class="mt-3 space-y-3">
                  <div>
                    <label
                      class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]"
                      for="auxiliary-url"
                    >
                      {SETTINGS.agent.openai.auxiliary_url_label}
                    </label>
                    <input
                      id="auxiliary-url"
                      type="url"
                      placeholder="https://api.minimaxi.com/v1"
                      class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                      value={agentDraft().auxiliary?.baseUrl ?? ""}
                      onInput={(e) =>
                        setAgentDraft((prev) => ({
                          ...prev,
                          auxiliary: {
                            ...(prev.auxiliary ?? {
                              baseUrl: "",
                              apiKey: "",
                              model: "",
                            }),
                            baseUrl: e.currentTarget.value,
                          },
                        }))
                      }
                    />
                  </div>
                  <div>
                    <label
                      class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]"
                      for="auxiliary-model"
                    >
                      {SETTINGS.agent.openai.auxiliary_model_label}
                    </label>
                    <input
                      id="auxiliary-model"
                      type="text"
                      placeholder="MiniMax-M2.7-highspeed"
                      class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                      value={agentDraft().auxiliary?.model ?? ""}
                      onInput={(e) =>
                        setAgentDraft((prev) => ({
                          ...prev,
                          auxiliary: {
                            ...(prev.auxiliary ?? {
                              baseUrl: "",
                              apiKey: "",
                              model: "",
                            }),
                            model: e.currentTarget.value,
                          },
                        }))
                      }
                    />
                  </div>
                  <div>
                    <label
                      class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]"
                      for="auxiliary-apikey"
                    >
                      {SETTINGS.agent.openai.auxiliary_apikey_label}
                    </label>
                    <div class="relative">
                      <input
                        id="auxiliary-apikey"
                        type={showAuxiliaryKey() ? "text" : "password"}
                        placeholder="sk-cp-..."
                        class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 pr-16 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                        value={agentDraft().auxiliary?.apiKey ?? ""}
                        onInput={(e) =>
                          setAgentDraft((prev) => ({
                            ...prev,
                            auxiliary: {
                              ...(prev.auxiliary ?? {
                                baseUrl: "",
                                apiKey: "",
                                model: "",
                              }),
                              apiKey: e.currentTarget.value,
                            },
                          }))
                        }
                      />
                      <button
                        type="button"
                        class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-2 py-0.5 text-xs text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                        onClick={() => setShowAuxiliaryKey((v) => !v)}
                      >
                        {showAuxiliaryKey()
                          ? SETTINGS.agent.openai.hide
                          : SETTINGS.agent.openai.show}
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              {/* API Key */}
              <div>
                <label
                  class="mb-1 block text-xs font-medium text-[color:var(--text-primary)]"
                  for="openai-apikey"
                >
                  {SETTINGS.agent.openai.api_key_label}
                </label>
                <div class="relative">
                  <input
                    id="openai-apikey"
                    type={showOpenaiKey() ? "text" : "password"}
                    placeholder="sk-or-..."
                    class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 pr-16 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                    value={agentDraft().openaiApiKey}
                    onInput={(e) =>
                      setAgentDraft((prev) => ({
                        ...prev,
                        openaiApiKey: e.currentTarget.value,
                      }))
                    }
                  />
                  <button
                    type="button"
                    class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-2 py-0.5 text-xs text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                    onClick={() => setShowOpenaiKey((v) => !v)}
                  >
                    {showOpenaiKey()
                      ? SETTINGS.agent.openai.hide
                      : SETTINGS.agent.openai.show}
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
                    <svg
                      class="h-3.5 w-3.5 shrink-0 animate-spin"
                      viewBox="0 0 24 24"
                      fill="none"
                    >
                      <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                      />
                      <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 22 6.477 22 12h-4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                      />
                    </svg>
                  </Show>
                  <Show when={openaiTestStatus() === "ok"}>
                    <svg
                      class="h-3.5 w-3.5 shrink-0"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  </Show>
                  <Show when={openaiTestStatus() === "error"}>
                    <svg
                      class="h-3.5 w-3.5 shrink-0"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  </Show>
                  <span>
                    {openaiTestStatus() === "checking"
                      ? SETTINGS.agent.openai.connection_testing_status
                      : openaiTestMessage()}
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
                    {auxiliaryTestStatus() === "checking"
                      ? SETTINGS.agent.openai.auxiliary_testing_status
                      : auxiliaryTestMessage()}
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
                  {openaiTestStatus() === "checking"
                    ? SETTINGS.agent.openai.testing
                    : SETTINGS.agent.openai.test_connection}
                </button>
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs text-[color:var(--text-primary)] transition hover:border-[color:var(--accent)]/60 disabled:opacity-50"
                  disabled={auxiliaryTestStatus() === "checking"}
                  onClick={() => void handleTestAuxiliary()}
                >
                  {auxiliaryTestStatus() === "checking"
                    ? SETTINGS.agent.openai.testing
                    : SETTINGS.agent.openai.test_auxiliary}
                </button>
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--accent)] bg-[color:var(--accent-soft)] px-3 py-1.5 text-xs font-medium text-[color:var(--text-primary)] transition hover:opacity-90 disabled:opacity-50"
                  disabled={agentSaving()}
                  onClick={(e) => void submitAgentSettings(e)}
                >
                  {agentSaving()
                    ? SETTINGS.agent.openai.saving
                    : SETTINGS.agent.openai.save}
                </button>
              </div>
            </div>
          </div>

          {/* ── 卡片 2：Codex ACP ── */}
          <div
            class={[
              "rounded-xl border p-5 transition cursor-pointer",
              agentDraft().runner === "codex_acp"
                ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                : "border-[color:var(--border)] bg-[color:var(--panel)] hover:border-[color:var(--accent)]/50",
            ].join(" ")}
            onClick={() => void selectRunner("codex_acp")}
          >
            <div class="flex items-start justify-between gap-3">
              <div>
                <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                  {SETTINGS.agent.codex_acp.name}
                </div>
                <div class="mt-0.5 text-xs text-[color:var(--text-secondary)]">
                  {SETTINGS.agent.codex_acp.description_prefix}
                  <code class="rounded bg-black/20 px-1">
                    {SETTINGS.agent.codex_acp.description_code}
                  </code>
                  {SETTINGS.agent.codex_acp.description_suffix}
                </div>
              </div>
              <Show when={agentDraft().runner === "codex_acp"}>
                <span class="shrink-0 rounded-full border border-[color:var(--accent)] px-2 py-0.5 text-[10px] font-medium text-[color:var(--accent)]">
                  {SETTINGS.agent.current_badge}
                </span>
              </Show>
            </div>

            <div class="mt-4 space-y-3" onClick={(e) => e.stopPropagation()}>
              <Show when={codexAcpCheckStatus() !== "idle"}>
                <div
                  class={[
                    "flex items-center gap-2 rounded-lg border px-3 py-2 text-xs",
                    codexAcpCheckStatus() === "checking"
                      ? "border-amber-300/40 bg-amber-500/10 text-amber-300"
                      : codexAcpCheckStatus() === "ok"
                        ? "border-emerald-300/40 bg-emerald-500/10 text-emerald-300"
                        : "border-rose-300/40 bg-rose-500/10 text-rose-300",
                  ].join(" ")}
                >
                  <Show when={codexAcpCheckStatus() === "checking"}>
                    <svg
                      class="h-3.5 w-3.5 shrink-0 animate-spin"
                      viewBox="0 0 24 24"
                      fill="none"
                    >
                      <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                      />
                      <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 22 6.477 22 12h-4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                      />
                    </svg>
                  </Show>
                  <Show when={codexAcpCheckStatus() === "ok"}>
                    <svg
                      class="h-3.5 w-3.5 shrink-0"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  </Show>
                  <Show when={codexAcpCheckStatus() === "error"}>
                    <svg
                      class="h-3.5 w-3.5 shrink-0"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  </Show>
                  <span>
                    {codexAcpCheckStatus() === "checking"
                      ? SETTINGS.agent.codex_acp.checking_status
                      : codexAcpCheckMessage()}
                  </span>
                </div>
              </Show>

              <div class="rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] p-3 text-xs text-[color:var(--text-secondary)]">
                {SETTINGS.agent.codex_acp.runtime_note}
              </div>

              <div class="flex gap-2 pt-1">
                <button
                  type="button"
                  class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs text-[color:var(--text-primary)] transition hover:border-[color:var(--accent)]/60 disabled:opacity-50"
                  disabled={codexAcpCheckStatus() === "checking"}
                  onClick={() => void handleCheckCodexAcp()}
                >
                  {codexAcpCheckStatus() === "checking"
                    ? SETTINGS.agent.codex_acp.checking
                    : SETTINGS.agent.codex_acp.test_connection}
                </button>
              </div>
            </div>
          </div>

          {/* ── 卡片 3：Gemini CLI ── */}
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
                <div class="text-sm font-semibold text-[color:var(--text-primary)]">
                  {SETTINGS.agent.gemini_cli.name}
                </div>
                <div class="mt-0.5 text-xs text-[color:var(--text-secondary)]">
                  {SETTINGS.agent.gemini_cli.description_prefix}
                  <code class="rounded bg-black/20 px-1">
                    {SETTINGS.agent.gemini_cli.description_code}
                  </code>
                  {SETTINGS.agent.gemini_cli.description_suffix}
                </div>
              </div>
              <Show when={agentDraft().runner === "gemini_cli"}>
                <span class="shrink-0 rounded-full border border-[color:var(--accent)] px-2 py-0.5 text-[10px] font-medium text-[color:var(--accent)]">
                  {SETTINGS.agent.current_badge}
                </span>
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
                    <svg
                      class="h-3.5 w-3.5 shrink-0 animate-spin"
                      viewBox="0 0 24 24"
                      fill="none"
                    >
                      <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                      />
                      <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 22 6.477 22 12h-4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                      />
                    </svg>
                  </Show>
                  <Show when={geminiCheckStatus() === "ok"}>
                    <svg
                      class="h-3.5 w-3.5 shrink-0"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  </Show>
                  <Show when={geminiCheckStatus() === "error"}>
                    <svg
                      class="h-3.5 w-3.5 shrink-0"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  </Show>
                  <span>
                    {geminiCheckStatus() === "checking"
                      ? SETTINGS.agent.gemini_cli.checking_status
                      : geminiCheckMessage()}
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
                  {geminiCheckStatus() === "checking"
                    ? SETTINGS.agent.gemini_cli.checking
                    : SETTINGS.agent.gemini_cli.test_connection}
                </button>
              </div>
            </div>
          </div>
        </fieldset>
      </div>

      <Show when={backend.hasCapability("web_invites")}>
        <div
          id="web-invite-settings"
          classList={{ hidden: !isTab("invite") }}
          class="rounded-2xl border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm"
        >
          <div class="flex items-start justify-between gap-4">
            <div>
              <h1 class="text-xl font-semibold text-[color:var(--text-primary)]">
                {SETTINGS.invite.title}
              </h1>
              <p class="mt-2 text-sm text-[color:var(--text-secondary)]">
                {SETTINGS.invite.subtitle}
              </p>
            </div>
            <div class="flex gap-2">
              <button
                type="button"
                class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs text-[color:var(--text-primary)] transition hover:bg-black/5"
                onClick={() => void refetchWebInvites()}
              >
                {SETTINGS.invite.refresh}
              </button>
            </div>
          </div>

          <div class="mt-4 flex flex-col gap-3 rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-4 lg:flex-row lg:items-end">
            <label class="flex-1">
              <div class="text-xs font-medium uppercase tracking-[0.14em] text-[color:var(--text-muted)]">
                {SETTINGS.invite.phone_label}
              </div>
              <input
                type="tel"
                value={invitePhoneNumber()}
                onInput={(event) =>
                  setInvitePhoneNumber(
                    normalizePhoneNumber(event.currentTarget.value),
                  )
                }
                placeholder={SETTINGS.invite.phone_placeholder}
                autocomplete="tel"
                class="mt-2 w-full rounded-lg border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm text-[color:var(--text-primary)] outline-none transition focus:border-[color:var(--accent)]"
              />
            </label>
            <button
              type="button"
              class="rounded-md bg-[color:var(--accent)] px-3 py-2 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50"
              disabled={inviteCreating() || !invitePhoneNumber().trim()}
              onClick={() => void handleCreateInvite()}
            >
              {inviteCreating() ? SETTINGS.invite.creating : SETTINGS.invite.create}
            </button>
          </div>

          <Show when={inviteMessage()}>
            <div class="mt-4 rounded-md border border-emerald-300/40 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-300">
              {inviteMessage()}
            </div>
          </Show>
          <Show when={inviteError()}>
            <div class="mt-4 rounded-md border border-rose-300/40 bg-rose-500/10 px-3 py-2 text-xs text-rose-300">
              {inviteError()}
            </div>
          </Show>

          <div class="mt-6 overflow-hidden rounded-xl border border-[color:var(--border)]">
            <div class="grid grid-cols-[1.2fr_1fr_1.1fr_0.8fr_0.8fr_0.7fr_0.9fr_1fr_auto] gap-3 bg-[color:var(--panel)] px-4 py-3 text-[11px] font-semibold uppercase tracking-[0.18em] text-[color:var(--text-muted)]">
              <div>{SETTINGS.invite.table.code}</div>
              <div>{SETTINGS.invite.table.phone}</div>
              <div>{SETTINGS.invite.table.web_user}</div>
              <div>{SETTINGS.invite.table.status}</div>
              <div>{SETTINGS.invite.table.api_key}</div>
              <div>{SETTINGS.invite.table.sessions}</div>
              <div>{SETTINGS.invite.table.remaining}</div>
              <div>{SETTINGS.invite.table.last_login}</div>
              <div>{SETTINGS.invite.table.actions}</div>
            </div>
            <Show
              when={(webInvites() ?? []).length > 0}
              fallback={
                <div class="px-4 py-8 text-sm text-[color:var(--text-secondary)]">
                  {SETTINGS.invite.table.empty}
                </div>
              }
            >
              <div class="divide-y divide-[color:var(--border)]">
                <For each={webInvites() ?? []}>
                  {(invite) => (
                    <div class="grid grid-cols-[1.2fr_1fr_1.1fr_0.8fr_0.8fr_0.7fr_0.9fr_1fr_auto] items-center gap-3 px-4 py-3 text-sm">
                      <div class="font-mono text-[color:var(--text-primary)]">
                        {invite.invite_code}
                      </div>
                      <div class="font-mono text-[color:var(--text-secondary)]">
                        {invite.phone_number || SETTINGS.invite.table.phone_unbound}
                      </div>
                      <div class="text-[color:var(--text-secondary)]">
                        {invite.user_id}
                      </div>
                      <div>
                        <span
                          class={[
                            "inline-flex rounded-full px-2.5 py-1 text-xs font-medium",
                            invite.enabled
                              ? "bg-emerald-500/10 text-emerald-300"
                              : "bg-rose-500/10 text-rose-300",
                          ].join(" ")}
                        >
                          {invite.enabled
                            ? SETTINGS.invite.table.enabled
                            : SETTINGS.invite.table.disabled}
                        </span>
                      </div>
                      <div class="font-mono text-xs text-[color:var(--text-secondary)]">
                        {invite.api_key_prefix || SETTINGS.invite.table.api_key_missing}
                      </div>
                      <div class="text-[color:var(--text-secondary)]">
                        {invite.active_session_count}
                      </div>
                      <div class="text-[color:var(--text-secondary)]">
                        {invite.daily_limit === 0
                          ? SETTINGS.invite.table.unlimited
                          : `${invite.remaining_today}/${invite.daily_limit}`}
                      </div>
                      <div class="text-[color:var(--text-secondary)]">
                        {invite.last_login_at
                          ? new Date(invite.last_login_at).toLocaleString()
                          : SETTINGS.invite.table.never_logged_in}
                      </div>
                      <div class="flex flex-wrap justify-end gap-2">
                        <button
                          type="button"
                          class="rounded-md border border-[color:var(--border)] px-2.5 py-1 text-xs text-[color:var(--text-primary)] transition hover:border-[color:var(--accent)]/60"
                          onClick={() =>
                            void copyInviteCode(invite.invite_code)
                          }
                        >
                          {SETTINGS.invite.table.copy}
                        </button>
                        <button
                          type="button"
                          class="rounded-md border border-[color:var(--border)] px-2.5 py-1 text-xs text-[color:var(--text-primary)] transition hover:border-[color:var(--accent)]/60 disabled:opacity-50"
                          disabled={isInviteActionRunning(
                            invite.user_id,
                            "reset",
                          )}
                          onClick={() => void handleResetInvite(invite)}
                        >
                          {isInviteActionRunning(invite.user_id, "reset")
                            ? SETTINGS.invite.table.resetting
                            : SETTINGS.invite.table.reset}
                        </button>
                        <button
                          type="button"
                          class="rounded-md border border-[color:var(--border)] px-2.5 py-1 text-xs text-[color:var(--text-primary)] transition hover:border-[color:var(--accent)]/60 disabled:opacity-50"
                          disabled={isInviteActionRunning(
                            invite.user_id,
                            "api-key",
                          )}
                          onClick={() => void handleGetInviteApiKey(invite)}
                        >
                          {isInviteActionRunning(invite.user_id, "api-key")
                            ? SETTINGS.invite.table.api_key_getting
                            : SETTINGS.invite.table.api_key_get}
                        </button>
                        <button
                          type="button"
                          class="rounded-md border border-[color:var(--border)] px-2.5 py-1 text-xs text-[color:var(--text-primary)] transition hover:border-[color:var(--accent)]/60 disabled:opacity-50"
                          disabled={isInviteActionRunning(
                            invite.user_id,
                            "api-key-reset",
                          )}
                          onClick={() => void handleResetInviteApiKey(invite)}
                        >
                          {isInviteActionRunning(invite.user_id, "api-key-reset")
                            ? SETTINGS.invite.table.api_key_resetting
                            : SETTINGS.invite.table.api_key_reset}
                        </button>
                        <Show
                          when={invite.enabled}
                          fallback={
                            <button
                              type="button"
                              class="rounded-md border border-emerald-500/30 px-2.5 py-1 text-xs text-emerald-300 transition hover:border-emerald-400 disabled:opacity-50"
                              disabled={isInviteActionRunning(
                                invite.user_id,
                                "enable",
                              )}
                              onClick={() => void handleEnableInvite(invite)}
                            >
                              {isInviteActionRunning(invite.user_id, "enable")
                                ? SETTINGS.invite.table.enabling
                                : SETTINGS.invite.table.enable}
                            </button>
                          }
                        >
                          <button
                            type="button"
                            class="rounded-md border border-rose-500/30 px-2.5 py-1 text-xs text-rose-300 transition hover:border-rose-400 disabled:opacity-50"
                            disabled={isInviteActionRunning(
                              invite.user_id,
                              "disable",
                            )}
                            onClick={() => void handleDisableInvite(invite)}
                          >
                            {isInviteActionRunning(invite.user_id, "disable")
                              ? SETTINGS.invite.table.disabling
                              : SETTINGS.invite.table.disable}
                          </button>
                        </Show>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </div>
      </Show>

      {/* ── 2. API 配置 ── */}
      <div
        id="api-settings"
        classList={{ hidden: !isTab("data") }}
        class="rounded-2xl border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm"
      >
        <div class="flex items-center gap-3">
          <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-amber-500/10 text-amber-500 font-bold">
            <svg
              class="h-5 w-5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z" />
              <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
              <line x1="12" y1="22.08" x2="12" y2="12" />
            </svg>
          </div>
          <h1 class="text-xl font-bold text-[color:var(--text-primary)]">
            {SETTINGS.data.title}
          </h1>
        </div>
        <p class="mt-2 text-sm text-[color:var(--text-secondary)]">
          {SETTINGS.data.subtitle}
        </p>

        <div class="mt-8 space-y-6">
          {/* FMP Subsection */}
          <div class="rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="flex h-6 w-6 items-center justify-center rounded bg-emerald-500/10 text-emerald-500 font-extrabold text-[10px]">
                  FMP
                </div>
                <div>
                  <div class="text-sm font-bold text-[color:var(--text-primary)]">
                    {SETTINGS.data.fmp.name}
                  </div>
                  <div class="mt-0.5 text-[10px] text-[color:var(--text-secondary)]">
                    {SETTINGS.data.fmp.description}
                  </div>
                </div>
              </div>
              <input
                type="checkbox"
                checked={true}
                disabled
                class="h-3.5 w-3.5 rounded border-[color:var(--border)] text-[color:var(--accent)]"
              />
            </div>
            <form
              class="mt-4 space-y-4"
              onSubmit={(event) => void submitFmpSettings(event)}
            >
              <fieldset
                disabled={!backend.state.isDesktop || fmpSettingsRes.loading}
                class="space-y-3"
              >
                <Index each={fmpDraft().apiKeys}>
                  {(key, index) => (
                    <div class="flex items-center gap-2">
                      <div class="relative flex-1">
                        <input
                          type={showFmpKeys()[index] ? "text" : "password"}
                          placeholder={SETTINGS.data.fmp.key_placeholder}
                          class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                          value={key()}
                          onInput={(e) =>
                            updateKey(setFmpDraft, index, e.currentTarget.value)
                          }
                        />
                        <button
                          type="button"
                          class="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                          onClick={() => toggleShowKey(setShowFmpKeys, index)}
                        >
                          <Show
                            when={showFmpKeys()[index]}
                            fallback={
                              <svg
                                class="h-3.5 w-3.5"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                              >
                                <path
                                  stroke-linecap="round"
                                  stroke-linejoin="round"
                                  stroke-width="2"
                                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                                />
                                <path
                                  stroke-linecap="round"
                                  stroke-linejoin="round"
                                  stroke-width="2"
                                  d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                                />
                              </svg>
                            }
                          >
                            <svg
                              class="h-3.5 w-3.5"
                              fill="none"
                              viewBox="0 0 24 24"
                              stroke="currentColor"
                            >
                              <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.046m4.596-4.596A9.964 9.964 0 0112 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                              />
                            </svg>
                          </Show>
                        </button>
                      </div>
                      <Show when={fmpDraft().apiKeys.length > 1}>
                        <button
                          type="button"
                          class="text-xs text-rose-500 px-2 font-medium"
                          onClick={() =>
                            removeKey(setFmpDraft, setShowFmpKeys, index)
                          }
                        >
                          {SETTINGS.data.fmp.remove}
                        </button>
                      </Show>
                    </div>
                  )}
                </Index>
                <div class="flex items-center justify-between pt-1">
                  <button
                    type="button"
                    class="text-[10px] font-bold text-[color:var(--accent)]"
                    onClick={() => addKey(setFmpDraft, setShowFmpKeys)}
                  >
                    {SETTINGS.data.fmp.add_key}
                  </button>
                  <button
                    type="submit"
                    class="rounded bg-[color:var(--accent)] px-3 py-1 text-xs font-bold text-white shadow-sm"
                    disabled={fmpSaving()}
                  >
                    {fmpSaving() ? SETTINGS.data.fmp.saving : SETTINGS.data.fmp.save}
                  </button>
                </div>
              </fieldset>
            </form>
          </div>

          {/* Tavily Subsection */}
          <div class="rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="flex h-6 w-6 items-center justify-center rounded bg-blue-500/10 text-blue-500 font-extrabold text-[10px]">
                  TAV
                </div>
                <div>
                  <div class="text-sm font-bold text-[color:var(--text-primary)]">
                    {SETTINGS.data.tavily.name}
                  </div>
                  <div class="mt-0.5 text-[10px] text-[color:var(--text-secondary)]">
                    {SETTINGS.data.tavily.description}
                  </div>
                </div>
              </div>
              <input
                type="checkbox"
                checked={true}
                disabled
                class="h-3.5 w-3.5 rounded border-[color:var(--border)] text-[color:var(--accent)]"
              />
            </div>
            <form
              class="mt-4 space-y-4"
              onSubmit={(event) => void submitTavilySettings(event)}
            >
              <fieldset
                disabled={!backend.state.isDesktop || tavilySettingsRes.loading}
                class="space-y-3"
              >
                <Index each={tavilyDraft().apiKeys}>
                  {(key, index) => (
                    <div class="flex items-center gap-2">
                      <div class="relative flex-1">
                        <input
                          type={showTavilyKeys()[index] ? "text" : "password"}
                          placeholder="tvly-..."
                          class="w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-sm text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                          value={key()}
                          onInput={(e) =>
                            updateKey(
                              setTavilyDraft,
                              index,
                              e.currentTarget.value,
                            )
                          }
                        />
                        <button
                          type="button"
                          class="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                          onClick={() =>
                            toggleShowKey(setShowTavilyKeys, index)
                          }
                        >
                          <Show
                            when={showTavilyKeys()[index]}
                            fallback={
                              <svg
                                class="h-3.5 w-3.5"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                              >
                                <path
                                  stroke-linecap="round"
                                  stroke-linejoin="round"
                                  stroke-width="2"
                                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                                />
                                <path
                                  stroke-linecap="round"
                                  stroke-linejoin="round"
                                  stroke-width="2"
                                  d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                                />
                              </svg>
                            }
                          >
                            <svg
                              class="h-3.5 w-3.5"
                              fill="none"
                              viewBox="0 0 24 24"
                              stroke="currentColor"
                            >
                              <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.046m4.596-4.596A9.964 9.964 0 0112 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                              />
                            </svg>
                          </Show>
                        </button>
                      </div>
                      <Show when={tavilyDraft().apiKeys.length > 1}>
                        <button
                          type="button"
                          class="text-xs text-rose-500 px-2 font-medium"
                          onClick={() =>
                            removeKey(setTavilyDraft, setShowTavilyKeys, index)
                          }
                        >
                          {SETTINGS.data.tavily.remove}
                        </button>
                      </Show>
                    </div>
                  )}
                </Index>
                <div class="flex items-center justify-between pt-1">
                  <button
                    type="button"
                    class="text-[10px] font-bold text-[color:var(--accent)]"
                    onClick={() => addKey(setTavilyDraft, setShowTavilyKeys)}
                  >
                    {SETTINGS.data.tavily.add_key}
                  </button>
                  <button
                    type="submit"
                    class="rounded bg-[color:var(--accent)] px-3 py-1 text-xs font-bold text-white shadow-sm"
                    disabled={tavilySaving()}
                  >
                    {tavilySaving()
                      ? SETTINGS.data.tavily.saving
                      : SETTINGS.data.tavily.save}
                  </button>
                </div>
              </fieldset>
            </form>
          </div>
        </div>
      </div>

      {/* ── 2.5. 通知推送偏好 ── */}
      <div
        id="notification-prefs"
        classList={{ hidden: !isTab("notify") }}
        class="rounded-2xl border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm"
      >
        <div class="flex items-center gap-3">
          <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-violet-500/10 text-violet-500 font-bold">
            <svg
              class="h-5 w-5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9" />
              <path d="M13.73 21a2 2 0 01-3.46 0" />
            </svg>
          </div>
          <div>
            <h1 class="text-xl font-bold text-[color:var(--text-primary)]">
              {SETTINGS.notify.title}
            </h1>
            <p class="mt-1 text-sm text-[color:var(--text-secondary)]">
              {SETTINGS.notify.subtitle}
            </p>
          </div>
        </div>
        <div class="mt-6">
          <NotificationPreferencesCard />
        </div>
      </div>

      {/* ── 3. 渠道设置 ── */}
      <div
        id="channel-settings"
        classList={{ hidden: !isTab("channel") }}
        class="rounded-2xl border border-[color:var(--border)] bg-[color:var(--surface)] p-6 shadow-sm"
      >
        <form onSubmit={(event) => void submitChannels(event)}>
          <fieldset
            disabled={
              !backend.state.isDesktop || desktopChannelSettings.loading
            }
            class="space-y-6 disabled:opacity-60"
          >
            <div class="flex items-start justify-between gap-4">
              <div class="flex items-center gap-3">
                <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-sky-500/10 text-sky-500 font-bold">
                  <svg
                    class="h-5 w-5"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                  >
                    <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" />
                  </svg>
                </div>
                <div>
                  <h1 class="text-xl font-bold text-[color:var(--text-primary)]">
                    {SETTINGS.channel.title}
                  </h1>
                  <p class="mt-1 text-sm text-[color:var(--text-secondary)]">
                    {SETTINGS.channel.subtitle}
                  </p>
                </div>
              </div>
              <button
                type="button"
                class="rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-1.5 text-xs text-[color:var(--text-primary)] transition hover:bg-black/5"
                onClick={() => void refetchDesktopChannelSettings()}
              >
                {SETTINGS.channel.refresh}
              </button>
            </div>

            <div class="grid gap-6 md:grid-cols-2">
              {/* Feishu */}
              <div class="space-y-4 rounded-xl border border-[color:var(--border)] bg-[color:var(--panel)] p-5">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-3">
                    <div class="flex h-10 w-10 items-center justify-center rounded-full bg-[#3370ff]/10 text-[#3370ff]">
                      <svg
                        class="h-6 w-6"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                      >
                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.5 9h-9c-.28 0-.5-.22-.5-.5s.22-.5.5-.5h9c.28 0 .5.22.5.5s-.22.5-.5.5zm0 3h-9c-.28 0-.5-.22-.5-.5s.22-.5.5-.5h9c.28 0 .5.22.5.5s-.22.5-.5.5z" />
                      </svg>
                    </div>
                    <div class="font-bold text-[color:var(--text-primary)]">
                      {SETTINGS.channel.feishu.name}
                    </div>
                  </div>
                  <label class="relative inline-flex cursor-pointer items-center">
                    <input
                      type="checkbox"
                      class="peer sr-only"
                      checked={channelDraft().feishuEnabled}
                      onChange={(e) =>
                        setChannelDraft((p) => ({
                          ...p,
                          feishuEnabled: e.currentTarget.checked,
                        }))
                      }
                    />
                    <div class="peer h-5 w-9 rounded-full bg-gray-200 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-all after:content-[''] peer-checked:bg-[color:var(--accent)] peer-checked:after:translate-x-full dark:bg-gray-700"></div>
                  </label>
                </div>
                <Show when={channelDraft().feishuEnabled}>
                  <div class="space-y-3 pt-2">
                    <div class="space-y-1">
                      <label class="text-[10px] font-bold uppercase tracking-wider text-[color:var(--text-secondary)]">
                        {SETTINGS.channel.feishu.app_id_label}
                      </label>
                      <input
                        type="text"
                        placeholder={SETTINGS.channel.feishu.app_id_placeholder}
                        class="w-full rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1.5 text-xs text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                        value={channelDraft().feishuAppId || ""}
                        onInput={(e) =>
                          setChannelDraft((p) => ({
                            ...p,
                            feishuAppId: e.currentTarget.value,
                          }))
                        }
                      />
                    </div>
                    <div class="space-y-1">
                      <label class="text-[10px] font-bold uppercase tracking-wider text-[color:var(--text-secondary)]">
                        {SETTINGS.channel.feishu.app_secret_label}
                      </label>
                      <div class="relative">
                        <input
                          type={showFeishuSecret() ? "text" : "password"}
                          placeholder={SETTINGS.channel.feishu.app_secret_placeholder}
                          class="w-full rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1.5 pr-14 text-xs text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                          value={channelDraft().feishuAppSecret || ""}
                          onInput={(e) =>
                            setChannelDraft((p) => ({
                              ...p,
                              feishuAppSecret: e.currentTarget.value,
                            }))
                          }
                        />
                        <button
                          type="button"
                          class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-1.5 py-0.5 text-[10px] text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                          onClick={() => setShowFeishuSecret((v) => !v)}
                        >
                          {showFeishuSecret()
                            ? SETTINGS.channel.feishu.hide
                            : SETTINGS.channel.feishu.show}
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
                      <svg
                        class="h-6 w-6"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                      >
                        <path d="M20.317 4.37c-1.215-.503-2.546-.882-3.932-1.057a.06.06 0 00-.063.03c-.157.28-.344.66-.464.945-1.497-.225-2.991-.225-4.463 0-.12-.285-.312-.665-.472-.945a.061.061 0 00-.063-.03 15.343 15.343 0 00-3.931 1.056.052.052 0 00-.024.02C4.195 7.42 2.91 10.375 3.328 13.25a.066.066 0 00.026.046 15.485 15.485 0 004.757 2.413.064.064 0 00.069-.022c.36-.492.684-1.02.954-1.574a.062.062 0 00-.034-.085c-.504-.19-1.002-.42-1.468-.69a.065.065 0 01-.006-.109c.097-.074.196-.15.291-.228a.063.063 0 01.066-.009 11.2 11.2 0 009.11 0 .063.063 0 01.067.01c.094.077.193.153.29.227a.065.065 0 01-.006.11c-.465.269-.963.499-1.467.689a.061.061 0 00-.034.086c.27.554.594 1.082.955 1.574a.063.063 0 00.068.022 15.441 15.441 0 004.759-2.413.06.06 0 00.026-.046c.491-3.415-.843-6.33-3.11-8.86a.052.052 0 00-.023-.02zM8.02 11.08c-.908 0-1.657-.84-1.657-1.87 0-1.03.731-1.87 1.657-1.87.935 0 1.666.84 1.657 1.87 0 1.03-.731 1.87-1.657 1.87zm7.96 0c-.908 0-1.657-.84-1.657-1.87 0-1.03.731-1.87 1.657-1.87.935 0 1.666.84 1.657 1.87 0 1.03-.732 1.87-1.657 1.87z" />
                      </svg>
                    </div>
                    <div class="font-bold text-[color:var(--text-primary)]">
                      {SETTINGS.channel.discord.name}
                    </div>
                  </div>
                  <label class="relative inline-flex cursor-pointer items-center">
                    <input
                      type="checkbox"
                      class="peer sr-only"
                      checked={channelDraft().discordEnabled}
                      onChange={(e) =>
                        setChannelDraft((p) => ({
                          ...p,
                          discordEnabled: e.currentTarget.checked,
                        }))
                      }
                    />
                    <div class="peer h-5 w-9 rounded-full bg-gray-200 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-all after:content-[''] peer-checked:bg-[color:var(--accent)] peer-checked:after:translate-x-full dark:bg-gray-700"></div>
                  </label>
                </div>
                <Show when={channelDraft().discordEnabled}>
                  <div class="space-y-1 pt-2">
                    <label class="text-[10px] font-bold uppercase tracking-wider text-[color:var(--text-secondary)]">
                      {SETTINGS.channel.discord.bot_token_label}
                    </label>
                    <div class="relative">
                      <input
                        type={showDiscordToken() ? "text" : "password"}
                        placeholder={SETTINGS.channel.discord.bot_token_placeholder}
                        class="w-full rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1.5 pr-14 text-xs text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                        value={channelDraft().discordBotToken || ""}
                        onInput={(e) =>
                          setChannelDraft((p) => ({
                            ...p,
                            discordBotToken: e.currentTarget.value,
                          }))
                        }
                      />
                      <button
                        type="button"
                        class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-1.5 py-0.5 text-[10px] text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                        onClick={() => setShowDiscordToken((v) => !v)}
                      >
                        {showDiscordToken()
                          ? SETTINGS.channel.discord.hide
                          : SETTINGS.channel.discord.show}
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
                      <svg
                        class="h-6 w-6"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                      >
                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.64 6.8c-.15 1.58-.8 5.42-1.13 7.19-.14.75-.42 1-.68 1.03-.58.05-1.02-.38-1.58-.75-.88-.58-1.38-.94-2.23-1.5-.99-.65-.35-1.01.22-1.59.15-.15 2.71-2.48 2.76-2.69.01-.03.01-.14-.07-.2-.08-.06-.19-.04-.27-.02-.12.02-1.96 1.25-5.54 3.69-.52.36-1 .54-1.43.53-.48-.01-1.4-.27-2.09-.49-.84-.27-1.51-.42-1.45-.88.03-.24.37-.48 1.02-.73 4-1.74 6.67-2.89 8.01-3.44 3.82-1.58 4.61-1.85 5.13-1.86.11 0 .37.03.54.17.14.12.18.28.2.45-.02.07-.02.13-.03.19z" />
                      </svg>
                    </div>
                    <div class="font-bold text-[color:var(--text-primary)]">
                      {SETTINGS.channel.telegram.name}
                    </div>
                  </div>
                  <label class="relative inline-flex cursor-pointer items-center">
                    <input
                      type="checkbox"
                      class="peer sr-only"
                      checked={channelDraft().telegramEnabled}
                      onChange={(e) =>
                        setChannelDraft((p) => ({
                          ...p,
                          telegramEnabled: e.currentTarget.checked,
                        }))
                      }
                    />
                    <div class="peer h-5 w-9 rounded-full bg-gray-200 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-all after:content-[''] peer-checked:bg-[color:var(--accent)] peer-checked:after:translate-x-full dark:bg-gray-700"></div>
                  </label>
                </div>
                <Show when={channelDraft().telegramEnabled}>
                  <div class="space-y-1 pt-2">
                    <label class="text-[10px] font-bold uppercase tracking-wider text-[color:var(--text-secondary)]">
                      {SETTINGS.channel.telegram.bot_token_label}
                    </label>
                    <div class="relative">
                      <input
                        type={showTelegramToken() ? "text" : "password"}
                        placeholder={SETTINGS.channel.telegram.bot_token_placeholder}
                        class="w-full rounded border border-[color:var(--border)] bg-[color:var(--surface)] px-2.5 py-1.5 pr-14 text-xs text-[color:var(--text-primary)] outline-none focus:border-[color:var(--accent)]"
                        value={channelDraft().telegramBotToken || ""}
                        onInput={(e) =>
                          setChannelDraft((p) => ({
                            ...p,
                            telegramBotToken: e.currentTarget.value,
                          }))
                        }
                      />
                      <button
                        type="button"
                        class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-1.5 py-0.5 text-[10px] text-[color:var(--text-secondary)] hover:text-[color:var(--text-primary)]"
                        onClick={() => setShowTelegramToken((v) => !v)}
                      >
                        {showTelegramToken()
                          ? SETTINGS.channel.telegram.hide
                          : SETTINGS.channel.telegram.show}
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
                      <svg
                        class="h-6 w-6"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                      >
                        <path d="M12 2C6.48 2 2 5.92 2 10.74c0 2.22 1.03 4.25 2.74 5.75-.12.44-.74 2.1-1.74 3.5 0 0 2.13 0 4.14-1.22.88.24 1.83.37 2.86.37 5.52 0 10-3.92 10-8.74S17.52 2 12 2z" />
                      </svg>
                    </div>
                    <div>
                      <div class="font-bold text-[color:var(--text-primary)]">
                        {SETTINGS.channel.imessage.name}
                      </div>
                      <div class="text-[10px] font-bold text-amber-600">
                        {SETTINGS.channel.imessage.warning}
                      </div>
                    </div>
                  </div>
                  <label class="relative inline-flex cursor-pointer items-center">
                    <input
                      type="checkbox"
                      class="peer sr-only"
                      checked={channelDraft().imessageEnabled}
                      onChange={(e) =>
                        setChannelDraft((p) => ({
                          ...p,
                          imessageEnabled: e.currentTarget.checked,
                        }))
                      }
                    />
                    <div class="peer h-5 w-9 rounded-full bg-gray-200 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:bg-white after:transition-all after:content-[''] peer-checked:bg-[color:var(--accent)] peer-checked:after:translate-x-full dark:bg-gray-700"></div>
                  </label>
                </div>
              </div>
            </div>

            <div class="mt-8 flex items-center justify-between border-t border-[color:var(--border)] pt-6">
              <div class="text-xs text-[color:var(--text-secondary)]">
                {SETTINGS.channel.sync_note}
              </div>
              <button
                type="submit"
                class="rounded-md bg-[color:var(--accent)] px-6 py-2 text-sm font-bold text-white shadow-sm transition-all hover:opacity-90 active:scale-95 disabled:opacity-50"
                disabled={backend.state.saving}
              >
                {backend.state.saving
                  ? SETTINGS.channel.saving
                  : SETTINGS.channel.save}
              </button>
            </div>
          </fieldset>
        </form>
      </div>
      </div>
    </div>
  );
}
