import { Logo } from "@hone-financial/ui/logo";
import { useNavigate } from "@solidjs/router";
import { For, Show, createResource, createSignal } from "solid-js";
import { loadDesktopAgentSettings } from "@/lib/backend";
import { useBackend } from "@/context/backend";
import { useSessions, ME_SESSION_ID } from "@/context/sessions";
import type { AgentProvider } from "@/lib/types";

type ChannelDef = {
  runner: AgentProvider;
  name: string;
  desc: string;
  icon: string;
};

const CHANNELS: ChannelDef[] = [
  {
    runner: "multi-agent",
    name: "Multi-Agent",
    desc: "MiniMax 搜索 + Gemini 回答",
    icon: "∞",
  },
  {
    runner: "opencode_acp",
    name: "自定义 OpenAI 协议",
    desc: "OpenAI compatible / 推荐 OpenRouter",
    icon: "⚡",
  },
  {
    runner: "gemini_cli",
    name: "Gemini CLI",
    desc: "复用本机 Gemini 命令行",
    icon: "✦",
  },
  {
    runner: "codex_cli",
    name: "Codex CLI",
    desc: "复用本机 Codex 命令行",
    icon: "◈",
  },
];

const RUNNER_LABEL: Record<AgentProvider, string> = {
  opencode_acp: "自定义 OpenAI 协议",
  "multi-agent": "Multi-Agent",
  gemini_cli: "Gemini CLI",
  gemini_acp: "Gemini ACP",
  codex_cli: "Codex CLI",
  codex_acp: "Codex ACP",
  function_calling: "Function Calling",
};

export default function StartPage() {
  const navigate = useNavigate();
  const backend = useBackend();
  const sessions = useSessions();

  const [input, setInput] = createSignal("");

  const [agentSettings] = createResource(
    () => backend.state.isDesktop,
    async (isDesktop) => {
      if (!isDesktop) return undefined;
      return loadDesktopAgentSettings();
    },
  );

  const activeRunner = () => agentSettings()?.runner ?? "opencode_acp";

  const handleSend = () => {
    const text = input().trim();
    if (!text) return;
    sessions.setPendingPrefill(text);
    navigate(`/sessions/${encodeURIComponent(ME_SESSION_ID)}`);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.isComposing) return;
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div class="flex h-full min-h-0 flex-col items-center justify-center overflow-y-auto px-6 py-10">
      {/* ── 品牌区 ── */}
      <div class="mb-10 flex flex-col items-center gap-4 text-center">
        <Logo class="mx-auto h-24 w-auto" />
        <p class="text-base text-[color:var(--text-muted)]">
          与 Hone 开启深度投研之旅
        </p>
      </div>

      {/* ── 三个渠道卡片 ── */}
      <div class="mb-10 flex w-full max-w-2xl gap-4">
        <For each={CHANNELS}>
          {(ch) => {
            const isActive = () => activeRunner() === ch.runner;
            return (
              <button
                type="button"
                class={[
                  "relative flex flex-1 flex-col items-center gap-2.5 rounded-2xl border px-5 py-6 text-center transition",
                  isActive()
                    ? "border-[color:var(--accent)] bg-[color:var(--accent-soft)]"
                    : "border-[color:var(--border)] bg-[color:var(--surface)] hover:border-[color:var(--accent)]/50 hover:bg-[color:var(--panel)]",
                  !backend.state.isDesktop ? "cursor-not-allowed opacity-40" : "cursor-pointer",
                ].join(" ")}
                disabled={!backend.state.isDesktop}
                onClick={() => navigate("/settings#agent-settings")}
                title={`前往设置配置 ${ch.name}`}
              >
                {/* 活跃徽章 */}
                <Show when={isActive()}>
                  <span class="absolute right-3 top-2.5 rounded-full bg-[color:var(--accent)] px-1.5 py-0.5 text-[9px] font-semibold uppercase tracking-wide text-white">
                    当前
                  </span>
                </Show>

                <span class="text-3xl">{ch.icon}</span>
                <span class="text-base font-semibold leading-tight text-[color:var(--text-primary)]">
                  {ch.name}
                </span>
                <span class="text-xs leading-tight text-[color:var(--text-muted)]">
                  {ch.desc}
                </span>
              </button>
            );
          }}
        </For>
      </div>

      {/* ── 输入框 ── */}
      <div class="w-full max-w-2xl">
        <div class="flex overflow-hidden rounded-2xl border border-[color:var(--border)] bg-[color:var(--surface)] shadow-sm transition focus-within:border-[color:var(--accent)] focus-within:shadow-md focus-within:shadow-[color:var(--accent)]/10">
          <textarea
            rows={3}
            placeholder="输入消息，按 Enter 发送…"
            class="min-h-0 flex-1 resize-none bg-transparent px-5 py-4 text-base text-[color:var(--text-primary)] outline-none placeholder:text-[color:var(--text-muted)]"
            value={input()}
            onInput={(e) => setInput(e.currentTarget.value)}
            onKeyDown={handleKeyDown}
          />
          <div class="flex items-end p-2.5">
            <button
              type="button"
              onClick={handleSend}
              disabled={!input().trim()}
              class="flex h-12 w-12 items-center justify-center rounded-xl bg-[color:var(--accent)] text-white transition hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-30"
            >
              <svg viewBox="0 0 20 20" fill="currentColor" class="h-6 w-6">
                <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
              </svg>
            </button>
          </div>
        </div>

        {/* 当前渠道提示 */}
        <div class="mt-3 flex items-center gap-1.5 text-sm text-[color:var(--text-muted)]">
          <Show
            when={agentSettings.loading}
            fallback={
              <>
                <span class="h-1.5 w-1.5 rounded-full bg-[color:var(--success)]" />
                <span>
                  当前渠道：
                  <span class="font-medium text-[color:var(--text-secondary)]">
                    {RUNNER_LABEL[activeRunner()]}
                  </span>
                </span>
                <span class="mx-1 text-[color:var(--border)]">·</span>
                <span>Shift+Enter 换行</span>
              </>
            }
          >
            <span class="animate-pulse">加载渠道配置中…</span>
          </Show>
        </div>
      </div>

      {/* ── 底部 Hone 品牌区 ── */}
      <div class="mt-16 w-full max-w-2xl">
        <div class="border-t border-[color:var(--border)] pt-8">
          <div class="flex items-center gap-4">
            <Logo class="h-8 w-auto opacity-80" />
            <div>
              <div class="text-sm font-medium text-[color:var(--text-secondary)]">
                磨砺认知、剔除噪音
              </div>
              <div class="text-xs text-[color:var(--text-muted)]">
                Open Financial Console
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
