import { Logo } from "@hone-financial/ui/logo";
import { Markdown } from "@hone-financial/ui/markdown";
import {
  createSignal,
  createEffect,
  For,
  Match,
  onCleanup,
  onMount,
  Show,
  Switch,
} from "solid-js";
import {
  connectPublicEvents,
  getPublicAuthMe,
  getPublicHistory,
  publicInviteLogin,
  publicLogout,
  sendPublicChat,
} from "@/lib/api";
import {
  parseMessageContent,
  historyToTimeline,
  messageId,
} from "@/lib/messages";
import { parseSseChunks } from "@/lib/stream";

type AuthState = "loading" | "logged_out" | "logging_in" | "ready";
type AssistantPhase = "thinking" | "running" | "streaming" | "done" | "error";
type PublicChatView = "loading" | "login" | "chat";

type ChatMessage = {
  id: string;
  role: "user" | "assistant";
  content: string;
  phase?: AssistantPhase;
  statusText?: string;
  startedAt?: number;
  steps?: string[];
};

const GITHUB_REPO_URL = "https://github.com/B-M-Capital-Research/honeclaw";
const GITHUB_API_URL =
  "https://api.github.com/repos/B-M-Capital-Research/honeclaw";
const GITHUB_STAR_COUNT_FALLBACK = 221;

export function normalizeInviteCode(value: string) {
  return value.replace(/\s+/g, "").trim().toUpperCase();
}

export function normalizePhoneNumber(value: string) {
  const trimmed = value.trim();
  const hasLeadingPlus = trimmed.startsWith("+");
  const digits = trimmed.replace(/\D+/g, "");
  return hasLeadingPlus ? `+${digits}` : digits;
}

export function resolvePublicChatView(authState: AuthState): PublicChatView {
  if (authState === "ready") return "chat";
  if (authState === "loading") return "loading";
  return "login";
}

function formatElapsed(startedAt?: number) {
  if (!startedAt) return "0s";
  const seconds = Math.max(0, Math.floor((Date.now() - startedAt) / 1000));
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remain = seconds % 60;
  return `${minutes}m ${remain}s`;
}

function RepoLink(props: { stars: number }) {
  return (
    <a
      href={GITHUB_REPO_URL}
      target="_blank"
      rel="noreferrer"
      class="inline-flex items-center gap-2 rounded-full bg-white/80 px-3 py-2 text-xs text-black/48 transition hover:bg-white hover:text-black"
    >
      <span class="font-medium">GitHub</span>
      <span>★ {props.stars}</span>
    </a>
  );
}

function LoadingCard(props: { githubStars: number }) {
  return (
    <div class="relative min-h-screen overflow-x-hidden overflow-y-hidden bg-[#ffffff] px-4 py-6 text-[#111111] sm:px-6 sm:py-10">
      <div class="relative mx-auto flex min-h-[calc(100vh-3rem)] w-full max-w-5xl flex-col sm:min-h-[calc(100vh-5rem)]">
        <div class="flex flex-col gap-3 py-3 sm:flex-row sm:items-center sm:justify-between">
          <div class="flex items-center gap-3 text-[11px] uppercase tracking-[0.22em] text-black/45 sm:tracking-[0.26em]">
            <Logo class="h-8 w-auto" />
            <span>Hone Chat</span>
          </div>
          <RepoLink stars={props.githubStars} />
        </div>
        <div class="flex flex-1 items-center justify-center">
          <div class="w-full max-w-2xl rounded-[28px] border border-black/8 bg-white px-6 py-10 text-center shadow-[0_16px_48px_rgba(0,0,0,0.05)] sm:rounded-[32px] sm:px-8 sm:py-12">
            <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-black text-white shadow-[0_10px_28px_rgba(0,0,0,0.12)]">
              <div class="h-5 w-5 animate-spin rounded-full border-2 border-white/35 border-t-white" />
            </div>
            <h1 class="mt-6 text-[28px] font-medium tracking-[-0.04em] text-black sm:text-[34px]">
              正在恢复登录状态
            </h1>
            <p class="mx-auto mt-4 max-w-xl text-sm leading-7 text-black/52 sm:text-[15px]">
              已登录用户刷新页面后会先校验当前会话，再恢复聊天内容和长连接更新。
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

export function LoginCard(props: {
  inviteCode: string;
  phoneNumber: string;
  loading: boolean;
  error: string;
  githubStars: number;
  onInput: (value: string) => void;
  onPhoneInput: (value: string) => void;
  onSubmit: () => void;
}) {
  return (
    <div class="relative min-h-screen overflow-x-hidden overflow-y-hidden bg-[#ffffff] px-4 py-6 text-[#111111] sm:px-6 sm:py-10">
      <div class="relative mx-auto flex min-h-[calc(100vh-3rem)] w-full max-w-5xl flex-col sm:min-h-[calc(100vh-5rem)]">
        <div class="flex flex-col gap-3 py-3 sm:flex-row sm:items-center sm:justify-between">
          <div class="flex items-center gap-3 text-[11px] uppercase tracking-[0.22em] text-black/45 sm:tracking-[0.26em]">
            <Logo class="h-8 w-auto" />
            <span>Hone Chat</span>
          </div>
          <div class="flex flex-wrap items-center gap-2 sm:justify-end">
            <div class="rounded-full bg-white/80 px-3 py-2 text-[11px] text-black/45 sm:px-4 sm:text-xs">
              Invite access
            </div>
            <RepoLink stars={props.githubStars} />
          </div>
        </div>
        <div class="flex flex-1 items-center justify-center">
          <div class="w-full max-w-3xl pb-12 pt-6 text-center sm:pb-16 sm:pt-8">
            <div class="flex justify-center">
              <div class="rounded-[28px] bg-white/88 px-6 py-4 shadow-[0_18px_50px_rgba(0,0,0,0.05)] sm:rounded-[32px] sm:px-8 sm:py-5">
                <Logo class="h-12 w-auto sm:h-14 md:h-20" />
              </div>
            </div>
            <h1 class="mx-auto mt-6 max-w-3xl text-[28px] font-medium tracking-[-0.05em] text-black sm:text-[34px] md:text-[56px]">
              开启深度投研之旅
            </h1>
            <p class="mx-auto mt-4 max-w-2xl px-2 text-sm leading-7 text-black/55 md:px-0 md:text-[15px]">
              输入邀请码和手机号后进入单会话聊天界面。体验会保持简洁，没有侧边栏，历史消息直接向上滚动查看。
            </p>

            <form
              class="mx-auto mt-8 max-w-2xl rounded-[28px] border border-black/10 bg-white px-4 py-4 shadow-[0_12px_60px_rgba(0,0,0,0.06)] sm:mt-10 sm:rounded-[32px] sm:px-5"
              onSubmit={(event) => {
                event.preventDefault();
                props.onSubmit();
              }}
            >
              <input
                type="text"
                value={props.inviteCode}
                onInput={(event) =>
                  props.onInput(normalizeInviteCode(event.currentTarget.value))
                }
                placeholder="输入邀请码"
                autocomplete="off"
                autocapitalize="characters"
                spellcheck={false}
                class="h-13 w-full bg-transparent px-1 text-[16px] tracking-[0.01em] text-black outline-none placeholder:text-black/30 sm:h-14 sm:px-2"
              />
              <input
                type="tel"
                value={props.phoneNumber}
                onInput={(event) =>
                  props.onPhoneInput(
                    normalizePhoneNumber(event.currentTarget.value),
                  )
                }
                placeholder="输入手机号，按 Enter 登录"
                autocomplete="tel"
                spellcheck={false}
                class="mt-2 h-13 w-full border-t border-black/8 bg-transparent px-1 text-[16px] tracking-[0.01em] text-black outline-none placeholder:text-black/30 sm:h-14 sm:px-2"
              />
              <div class="mt-3 flex flex-col gap-3 border-t border-black/8 pt-3 sm:flex-row sm:items-center sm:justify-between sm:gap-4">
                <div class="text-left text-xs leading-6 text-black/45">
                  邀请码与手机号验证通过后会自动恢复你的单线程 Web 会话
                </div>
                <button
                  type="submit"
                  disabled={
                    props.loading ||
                    !props.inviteCode.trim() ||
                    !props.phoneNumber.trim()
                  }
                  class="inline-flex h-11 w-full shrink-0 items-center justify-center rounded-full bg-black px-5 text-sm font-medium text-white transition hover:bg-black/90 disabled:cursor-not-allowed disabled:bg-black/20 sm:w-auto"
                >
                  {props.loading ? "验证中…" : "开始对话"}
                </button>
              </div>
            </form>

            <Show when={props.error}>
              <div class="mx-auto mt-4 max-w-2xl rounded-2xl border border-rose-200 bg-rose-50 px-4 py-3 text-left text-sm text-rose-500">
                {props.error}
              </div>
            </Show>

            <div class="mt-8 flex flex-wrap items-center justify-center gap-3 text-xs text-black/42">
              <div class="rounded-full border border-black/10 bg-white/80 px-4 py-2">
                单会话
              </div>
              <div class="rounded-full border border-black/10 bg-white/80 px-4 py-2">
                长连接更新
              </div>
              <div class="rounded-full border border-black/10 bg-white/80 px-4 py-2">
                邀请码 + 手机号
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function AssistantCard(props: { message: ChatMessage }) {
  const [elapsed, setElapsed] = createSignal(
    formatElapsed(props.message.startedAt),
  );

  createEffect(() => {
    if (
      !props.message.startedAt ||
      props.message.phase === "done" ||
      props.message.phase === "error"
    ) {
      setElapsed(formatElapsed(props.message.startedAt));
      return;
    }
    const timer = setInterval(() => {
      setElapsed(formatElapsed(props.message.startedAt));
    }, 1000);
    onCleanup(() => clearInterval(timer));
  });

  const toneClass = () =>
    props.message.phase === "error" ? "bg-rose-50" : "bg-white/82";

  return (
    <div
      class={[
        "overflow-hidden rounded-[28px] px-5 py-4 shadow-[0_8px_32px_rgba(0,0,0,0.04)]",
        toneClass(),
      ].join(" ")}
    >
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-2">
          <span
            class={[
              "h-2.5 w-2.5 rounded-full",
              props.message.phase === "error" ? "bg-rose-400" : "bg-black",
            ].join(" ")}
          />
          <span class="text-[11px] font-semibold uppercase tracking-[0.22em] text-black/42">
            {props.message.phase === "done"
              ? "Hone 已回复"
              : props.message.phase === "error"
                ? "Hone 出错"
                : props.message.phase === "streaming"
                  ? "Hone 输出中"
                  : props.message.phase === "running"
                    ? "Hone 执行中"
                    : "Hone 思考中"}
          </span>
        </div>
        <div class="text-xs text-black/35">{elapsed()}</div>
      </div>

      <Show when={props.message.statusText}>
        <div class="mt-3 text-sm text-black/56">{props.message.statusText}</div>
      </Show>

      <Show when={props.message.steps && props.message.steps!.length > 0}>
        <div class="mt-4 space-y-2 rounded-2xl bg-[#f5f5f0] px-4 py-3">
          <For each={props.message.steps}>
            {(step) => (
              <div class="flex items-start gap-2 text-sm text-black/58">
                <span class="mt-[9px] h-1.5 w-1.5 rounded-full bg-black/70" />
                <span>{step}</span>
              </div>
            )}
          </For>
        </div>
      </Show>

      <Show when={props.message.content}>
        <div class="mt-4 text-[15px] leading-8 text-black">
          <For each={parseMessageContent(props.message.content)}>
            {(part) => (
              <Switch>
                <Match when={part.type === "image"}>
                  <img
                    src={part.value}
                    alt=""
                    class="mt-3 max-w-full rounded-2xl"
                  />
                </Match>
                <Match when={part.type === "text"}>
                  <Markdown
                    text={part.value}
                    class="break-words text-[15px] leading-8 text-black [&_*]:max-w-full [&_p]:my-0 [&_p+*]:mt-4 [&_pre]:mt-4 [&_pre]:max-w-full [&_pre]:overflow-x-auto [&_pre]:rounded-2xl [&_pre]:border-0 [&_pre]:shadow-none [&_code]:rounded [&_code]:bg-black/[0.04] [&_code]:px-1.5 [&_code]:py-0.5 [&_ul]:my-3 [&_ol]:my-3 [&_li]:my-1 [&_blockquote]:my-4 [&_blockquote]:border-l-2 [&_blockquote]:border-black/12 [&_blockquote]:pl-4 [&_blockquote]:text-black/58"
                  />
                </Match>
              </Switch>
            )}
          </For>
        </div>
      </Show>

      <Show when={!props.message.content && props.message.phase !== "error"}>
        <div class="mt-4 flex gap-1.5">
          <span
            class="h-2 w-2 animate-bounce rounded-full bg-black/28"
            style={{ "animation-delay": "0ms" }}
          />
          <span
            class="h-2 w-2 animate-bounce rounded-full bg-black/28"
            style={{ "animation-delay": "120ms" }}
          />
          <span
            class="h-2 w-2 animate-bounce rounded-full bg-black/28"
            style={{ "animation-delay": "240ms" }}
          />
        </div>
      </Show>
    </div>
  );
}

export default function PublicChatPage() {
  const [authState, setAuthState] = createSignal<AuthState>("loading");
  const [loginError, setLoginError] = createSignal("");
  const [inviteCode, setInviteCode] = createSignal("");
  const [phoneNumber, setPhoneNumber] = createSignal("");
  const [messages, setMessages] = createSignal<ChatMessage[]>([]);
  const [draft, setDraft] = createSignal("");
  const [sendError, setSendError] = createSignal("");
  const [isSending, setIsSending] = createSignal(false);
  const [githubStars, setGithubStars] = createSignal(
    GITHUB_STAR_COUNT_FALLBACK,
  );
  const [sessionInfo, setSessionInfo] = createSignal<{
    userId: string;
    remainingToday: number;
    dailyLimit: number;
  } | null>(null);
  let eventSource: EventSource | null = null;
  let activeController: AbortController | null = null;
  let scrollRef: HTMLDivElement | undefined;
  let sessionSyncGeneration = 0;

  const scrollToBottom = () => {
    requestAnimationFrame(() => {
      if (!scrollRef) return;
      scrollRef.scrollTop = scrollRef.scrollHeight;
    });
  };

  const syncSession = async () => {
    const generation = ++sessionSyncGeneration;
    try {
      const user = await getPublicAuthMe();
      if (generation !== sessionSyncGeneration) return;
      setSessionInfo({
        userId: user.user_id,
        remainingToday: user.remaining_today,
        dailyLimit: user.daily_limit,
      });
      setLoginError("");
      const history = await getPublicHistory();
      if (generation !== sessionSyncGeneration) return;
      const timeline = historyToTimeline(history)
        .filter(
          (message) => message.kind === "user" || message.kind === "assistant",
        )
        .map((message) => ({
          id: message.id,
          role: message.kind,
          content: message.content,
          phase: "done" as const,
          steps: [],
        }));
      setMessages(timeline);
      // Transition to "ready" only after history is loaded so the UI
      // doesn't flash an empty chat view while messages are still loading.
      setAuthState("ready");
      await connectPushEvents();
      if (generation !== sessionSyncGeneration) return;
      scrollToBottom();
    } catch (error) {
      if (generation !== sessionSyncGeneration) return;
      setSessionInfo(null);
      setMessages([]);
      setAuthState("logged_out");
      if (error instanceof Error && !/401|未登录|过期/.test(error.message)) {
        setLoginError(error.message);
      }
    }
  };

  const publicChatView = () => resolvePublicChatView(authState());

  const connectPushEvents = async () => {
    eventSource?.close();
    eventSource = null;
    try {
      eventSource = await connectPublicEvents();
      eventSource.addEventListener("scheduled_message", (event) => {
        const data = JSON.parse(event.data || "{}") as { text?: string };
        if (!data.text?.trim()) return;
        setMessages((current) => [
          ...current,
          {
            id: messageId(),
            role: "assistant",
            content: data.text ?? "",
            phase: "done",
            steps: [],
          },
        ]);
        scrollToBottom();
      });
      eventSource.addEventListener("push_message", (event) => {
        const data = JSON.parse(event.data || "{}") as { text?: string };
        if (!data.text?.trim()) return;
        setMessages((current) => [
          ...current,
          {
            id: messageId(),
            role: "assistant",
            content: data.text ?? "",
            phase: "done",
            steps: [],
          },
        ]);
        scrollToBottom();
      });
      eventSource.onerror = () => {
        eventSource?.close();
        eventSource = null;
      };
    } catch {
      eventSource?.close();
      eventSource = null;
    }
  };

  onMount(() => {
    void fetch(GITHUB_API_URL)
      .then((response) => (response.ok ? response.json() : null))
      .then((payload) => {
        const stars = payload?.stargazers_count;
        if (typeof stars === "number" && Number.isFinite(stars)) {
          setGithubStars(stars);
        }
      })
      .catch(() => {});
    void syncSession();
  });

  onCleanup(() => {
    eventSource?.close();
    activeController?.abort();
  });

  createEffect(() => {
    messages().length;
    scrollToBottom();
  });

  const handleLogin = async () => {
    const code = normalizeInviteCode(inviteCode());
    const phone = normalizePhoneNumber(phoneNumber());
    if (!code || !phone) return;
    setAuthState("logging_in");
    setLoginError("");
    setInviteCode(code);
    setPhoneNumber(phone);
    try {
      const user = await publicInviteLogin(code, phone);
      setSessionInfo({
        userId: user.user_id,
        remainingToday: user.remaining_today,
        dailyLimit: user.daily_limit,
      });
      setInviteCode("");
      setPhoneNumber("");
      await syncSession();
    } catch (error) {
      setAuthState("logged_out");
      setLoginError(error instanceof Error ? error.message : String(error));
    }
  };

  const handleLogout = async () => {
    activeController?.abort();
    activeController = null;
    eventSource?.close();
    eventSource = null;
    await publicLogout();
    setMessages([]);
    setDraft("");
    setSendError("");
    setSessionInfo(null);
    setAuthState("logged_out");
  };

  const appendAssistantStep = (messageIdValue: string, step: string) => {
    const normalized = step.trim();
    if (!normalized) return;
    setMessages((current) =>
      current.map((message) => {
        if (message.id !== messageIdValue) return message;
        const steps = message.steps ?? [];
        if (steps.includes(normalized)) return message;
        return {
          ...message,
          steps: [...steps, normalized],
        };
      }),
    );
  };

  const handleSend = async () => {
    const text = draft().trim();
    if (!text || authState() !== "ready" || activeController) return;
    const assistantId = messageId();
    setSendError("");
    setDraft("");
    setIsSending(true);
    setMessages((current) => [
      ...current,
      { id: messageId(), role: "user", content: text },
      {
        id: assistantId,
        role: "assistant",
        content: "",
        phase: "thinking",
        statusText: "Hone 思考中",
        startedAt: Date.now(),
        steps: [],
      },
    ]);
    scrollToBottom();

    const controller = new AbortController();
    activeController = controller;

    try {
      const stream = await sendPublicChat(text, controller.signal);
      const reader = stream.getReader();
      const decoder = new TextDecoder();
      let pending = "";

      while (true) {
        const chunk = await reader.read();
        if (chunk.done) break;
        pending += decoder.decode(chunk.value, { stream: true });
        const parsed = parseSseChunks(pending);
        pending = parsed.pending;

        for (const event of parsed.events) {
          if (event.event === "run_started") {
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? { ...message, phase: "thinking", statusText: "Hone 思考中" }
                  : message,
              ),
            );
          }

          if (event.event === "tool_call") {
            const step =
              (event.data.text ?? event.data.reasoning ?? "").trim() ||
              "处理中…";
            appendAssistantStep(assistantId, step);
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? {
                      ...message,
                      phase: "running",
                      statusText: step || "处理中…",
                    }
                  : message,
              ),
            );
          }

          if (event.event === "assistant_delta") {
            const content = event.data.content ?? "";
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? {
                      ...message,
                      phase: "streaming",
                      statusText: "整理正式回复",
                      content: message.content + content,
                    }
                  : message,
              ),
            );
          }

          if (event.event === "run_error" || event.event === "error") {
            const messageText =
              event.event === "run_error"
                ? (event.data.message ?? "处理失败，请重试")
                : (event.data.text ?? "处理失败，请重试");
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? {
                      ...message,
                      phase: "error",
                      statusText: messageText,
                    }
                  : message,
              ),
            );
          }

          if (event.event === "run_finished") {
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? {
                      ...message,
                      phase: event.data.success === false ? "error" : "done",
                      statusText:
                        event.data.success === false
                          ? message.statusText || "处理失败，请重试"
                          : message.content
                            ? ""
                            : "本轮没有返回正文",
                    }
                  : message,
              ),
            );
          }
        }
      }
    } catch (error) {
      const messageText =
        error instanceof Error && error.name === "AbortError"
          ? "本轮请求已停止"
          : error instanceof Error
            ? error.message
            : String(error);
      setSendError(messageText);
      setMessages((current) =>
        current.map((message) =>
          message.id === assistantId
            ? { ...message, phase: "error", statusText: messageText }
            : message,
        ),
      );
    } finally {
      activeController = null;
      setIsSending(false);
      void syncSession();
    }
  };

  return (
    <Switch>
      <Match when={publicChatView() === "loading"}>
        <LoadingCard githubStars={githubStars()} />
      </Match>
      <Match when={publicChatView() === "login"}>
        <LoginCard
          inviteCode={inviteCode()}
          phoneNumber={phoneNumber()}
          loading={authState() === "logging_in"}
          error={loginError()}
          githubStars={githubStars()}
          onInput={setInviteCode}
          onPhoneInput={setPhoneNumber}
          onSubmit={() => void handleLogin()}
        />
      </Match>
      <Match when={publicChatView() === "chat"}>
        <div class="flex h-[100dvh] max-h-[100dvh] min-w-0 flex-col overflow-x-hidden overflow-y-hidden bg-[#ffffff] text-[#111111]">
        <header class="shrink-0 bg-[#ffffff]/92 backdrop-blur">
          <div class="mx-auto flex w-full max-w-7xl items-center justify-between px-3 py-3 sm:px-6 sm:py-4">
            <div class="min-w-0 flex items-center gap-3 sm:gap-4">
              <Logo class="h-8 w-auto" />
              <div class="min-w-0">
                <div class="truncate text-sm font-semibold text-black">
                  Hone Chat
                </div>
                <div class="truncate text-[11px] text-black/45 sm:text-xs">
                  {sessionInfo()?.dailyLimit
                    ? `今日剩余 ${sessionInfo()?.remainingToday}/${sessionInfo()?.dailyLimit}`
                    : "当前实例未启用对话次数限制"}
                </div>
              </div>
            </div>
            <div class="flex items-center gap-2 sm:gap-3">
              <div class="hidden sm:block">
                <RepoLink stars={githubStars()} />
              </div>
              <div class="hidden rounded-full bg-white/70 px-3 py-1.5 text-xs text-black/48 md:block">
                {sessionInfo()?.userId}
              </div>
              <button
                type="button"
                onClick={() => void handleLogout()}
                class="rounded-full bg-white px-3 py-2 text-sm text-black transition hover:bg-black/[0.03] sm:px-4"
              >
                退出登录
              </button>
            </div>
          </div>
        </header>

        <main class="mx-auto flex min-h-0 min-w-0 w-full max-w-[1440px] flex-1 flex-col overflow-x-hidden overflow-y-hidden px-2 pb-3 pt-1 sm:px-3 md:px-6">
          <div
            ref={scrollRef}
            class="hf-scrollbar flex min-h-0 min-w-0 flex-1 flex-col gap-4 overflow-x-hidden overflow-y-auto px-1 py-3 sm:gap-5 sm:py-4 md:px-2"
          >
            <Show
              when={messages().length > 0}
              fallback={
                <div class="mx-auto flex w-full max-w-5xl flex-1 flex-col items-center justify-center px-3 py-12 text-center sm:py-16">
                  <div class="rounded-full bg-white/80 px-6 py-3.5 shadow-[0_8px_24px_rgba(0,0,0,0.04)]">
                    <Logo class="h-9 w-auto" />
                  </div>
                  <h1 class="mt-6 text-[30px] font-medium tracking-[-0.04em] text-black sm:text-[42px]">
                    问 Hone 一个问题
                  </h1>
                  <p class="mt-4 max-w-2xl text-sm leading-8 text-black/55">
                    这里没有侧边栏，也没有多会话切换。输入问题后，Hone
                    会先展示思考和工具执行过程，再把同一张回复卡片更新成最终答案。
                  </p>
                </div>
              }
            >
              <For each={messages()}>
                {(message) => (
                  <div
                    class={[
                      "flex w-full",
                      message.role === "user" ? "justify-end" : "justify-start",
                    ].join(" ")}
                  >
                    <div
                      class={
                        message.role === "user"
                          ? "min-w-0 max-w-[86%] sm:max-w-[72%]"
                          : "min-w-0 max-w-[100%] sm:max-w-[96%] md:max-w-[92%]"
                      }
                    >
                      <Show
                        when={message.role === "assistant"}
                        fallback={
                          <div class="overflow-hidden rounded-[20px] bg-black px-3.5 py-2.5 text-[15px] leading-6 text-white shadow-[0_12px_30px_rgba(0,0,0,0.12)] sm:rounded-[22px] sm:px-4">
                            <Markdown
                              text={message.content}
                              class="break-words text-[15px] leading-6 !text-white [&_*]:max-w-full [&_*]:!text-white [&_p]:my-0 [&_p+*]:mt-2.5 [&_pre]:mt-2.5 [&_pre]:max-w-full [&_pre]:overflow-x-auto [&_pre]:rounded-2xl [&_pre]:border-0 [&_pre]:shadow-none [&_code]:rounded [&_code]:bg-white/12 [&_code]:px-1.5 [&_code]:py-0.5 [&_ul]:my-2.5 [&_ol]:my-2.5 [&_li]:my-1 [&_blockquote]:my-3 [&_blockquote]:border-l-2 [&_blockquote]:border-white/18 [&_blockquote]:pl-4 [&_blockquote]:!text-white"
                            />
                          </div>
                        }
                      >
                        <AssistantCard message={message} />
                      </Show>
                    </div>
                  </div>
                )}
              </For>
            </Show>
          </div>

          <div
            class="mt-2 shrink-0 rounded-[22px] bg-white/92 px-3 py-2 shadow-[0_10px_28px_rgba(0,0,0,0.04)] sm:rounded-[24px] sm:px-4 sm:py-2.5"
            style={{
              "padding-bottom": "max(env(safe-area-inset-bottom), 0.5rem)",
            }}
          >
            <textarea
              rows={1}
              value={draft()}
              onInput={(event) => setDraft(event.currentTarget.value)}
              onKeyDown={(event) => {
                if (event.isComposing) return;
                if (event.key === "Enter" && !event.shiftKey) {
                  event.preventDefault();
                  void handleSend();
                }
              }}
              placeholder="输入你的问题，按 Enter 发送"
              class="min-h-[44px] max-h-[132px] w-full resize-none bg-transparent px-1 py-1.5 text-[15px] leading-6 text-black outline-none placeholder:text-black/28 sm:leading-7"
            />
            <div class="mt-1.5 flex flex-col gap-2 pt-1.5 sm:flex-row sm:items-center sm:justify-between sm:gap-4">
              <div class="text-xs text-black/42">
                <Show
                  when={sendError()}
                  fallback={<span>Shift + Enter 换行</span>}
                >
                  <span class="text-rose-400">{sendError()}</span>
                </Show>
              </div>
              <div class="flex items-center justify-end gap-2 sm:gap-3">
                <Show when={isSending()}>
                  <button
                    type="button"
                    onClick={() => activeController?.abort()}
                    class="rounded-full bg-rose-50 px-3 py-2 text-sm text-rose-500 transition hover:bg-rose-100 sm:px-4"
                  >
                    停止
                  </button>
                </Show>
                <button
                  type="button"
                  onClick={() => void handleSend()}
                  disabled={!draft().trim() || isSending()}
                  class="inline-flex h-11 items-center rounded-full bg-black px-4 text-sm font-medium text-white transition hover:bg-black/90 disabled:cursor-not-allowed disabled:bg-black/20 sm:px-5"
                >
                  发送
                </button>
              </div>
            </div>
          </div>
        </main>
        </div>
      </Match>
    </Switch>
  );
}
