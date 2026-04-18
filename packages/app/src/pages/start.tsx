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
    runner: "codex_acp",
    name: "Codex ACP",
    desc: "通过 codex-acp 驱动当前会话",
    icon: "⌘",
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
    <div class="flex h-full min-h-0 flex-col items-center overflow-y-auto px-5">
      {/* ── 控制整体垂直偏上的包裹层 ── */}
      <div class="mt-[15vh] flex w-full max-w-3xl flex-col items-center">
        {/* ── 品牌区 ── */}
        <div class="mb-10 flex flex-col items-center gap-4 text-center transition-transform hover:scale-[1.01]">
          <Logo class="mx-auto h-20 w-auto" />
          <h1 class="text-[22px] font-medium tracking-wide text-[color:var(--text-primary)]">
            开启深度投研之旅
          </h1>
        </div>

        {/* ── 输入区与渠道选择 ── */}
        <div class="flex w-full flex-col items-center gap-5">
          {/* 渠道标签栏 (Pills) */}
          <div class="flex w-full flex-wrap justify-center gap-2 md:gap-2.5">
          <For each={CHANNELS}>
            {(ch) => {
              const isActive = () => activeRunner() === ch.runner;
              return (
                <button
                  type="button"
                  class={[
                    "group flex items-center gap-1.5 rounded-full border px-3.5 py-1.5 text-[13px] backdrop-blur-sm transition-all duration-300",
                    isActive()
                      ? "border-[color:var(--accent)] bg-[color:var(--accent)]/10 text-[color:var(--accent)] shadow-sm shadow-[color:var(--accent)]/10"
                      : "border-[color:var(--border)] bg-[color:var(--surface)] text-[color:var(--text-secondary)] shadow-sm shadow-black/5 hover:-translate-y-0.5 hover:border-[color:var(--accent)]/50 hover:bg-[color:var(--panel)] hover:shadow-md",
                    !backend.state.isDesktop ? "cursor-not-allowed opacity-40" : "cursor-pointer",
                  ].join(" ")}
                  disabled={!backend.state.isDesktop}
                  onClick={() => navigate("/settings#agent-settings")}
                  title={`说明: ${ch.desc}\n点击前往配置`}
                >
                  <span
                    class={[
                      "text-base transition-transform duration-300",
                      isActive() ? "scale-110" : "group-hover:scale-110",
                    ].join(" ")}
                  >
                    {ch.icon}
                  </span>
                  <span class="font-medium tracking-wide">{ch.name}</span>
                </button>
              );
            }}
          </For>
        </div>

        {/* 聊天输入框主体 */}
        <div class="relative w-full overflow-hidden rounded-[24px] border border-[color:var(--border)] bg-[color:var(--surface)] shadow-lg shadow-black/5 transition-all duration-300 focus-within:border-[color:var(--accent)] focus-within:shadow-[color:var(--accent)]/20 focus-within:shadow-xl">
          <div class="flex">
            <textarea
              rows={3}
              placeholder="输入你想探索的投研问题，按 Enter 发送…"
              class="min-h-[120px] w-full resize-none bg-transparent px-6 pb-14 pt-5 text-[15px] leading-relaxed text-[color:var(--text-primary)] outline-none placeholder:text-[color:var(--text-muted)]/70"
              value={input()}
              onInput={(e) => setInput(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
            />
          </div>

          {/* 底部功能栏 */}
          <div class="absolute bottom-0 left-0 right-0 flex items-center justify-between bg-gradient-to-t from-[color:var(--surface)] via-[color:var(--surface)] to-transparent px-4 py-3">
            <div class="ml-2 flex items-center text-[12px] text-[color:var(--text-muted)] opacity-70">
              <Show
                when={agentSettings.loading}
                fallback={<span>Shift + Enter 换行</span>}
              >
                <div class="flex items-center gap-2">
                  <span class="relative flex h-1.5 w-1.5">
                    <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-[color:var(--text-muted)] opacity-75"></span>
                    <span class="relative inline-flex h-1.5 w-1.5 rounded-full bg-[color:var(--text-muted)]"></span>
                  </span>
                  <span class="animate-pulse">加载配置中…</span>
                </div>
              </Show>
            </div>
            <button
              type="button"
              onClick={handleSend}
              disabled={!input().trim()}
              class="flex h-10 w-10 items-center justify-center rounded-2xl bg-[color:var(--accent)] text-white transition-all hover:scale-105 hover:shadow-md disabled:cursor-not-allowed disabled:opacity-30 disabled:hover:scale-100 disabled:hover:shadow-none"
            >
              <svg viewBox="0 0 20 20" fill="currentColor" class="ml-0.5 h-5 w-5">
                <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
              </svg>
            </button>
          </div>
        </div>
        
        {/* 底部短语 */}
        <div class="mt-8 flex items-center gap-3 text-xs text-[color:var(--text-muted)] opacity-60">
          <span>磨砺认知、剔除噪音</span>
          <span class="h-1 w-1 rounded-full bg-[color:var(--border)]"></span>
          <span>Open Financial Console</span>
        </div>
      </div>
      </div>
    </div>
  );
}
