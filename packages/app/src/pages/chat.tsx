import { Logo } from "@hone-financial/ui/logo";
import { Markdown } from "@hone-financial/ui/markdown";
import {
  createMemo,
  createSignal,
  createEffect,
  For,
  Match,
  onCleanup,
  onMount,
  Show,
  Switch,
} from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import { useNavigate } from "@solidjs/router";
import { PublicNav } from "@/components/public-nav";
import "./public-site.css";
import {
  connectPublicEvents,
  getPublicAuthMe,
  getPublicHistory,
  publicInviteLogin,
  publicLogout,
  sendPublicChat,
} from "@/lib/api";
import { parseMessageContent, messageId } from "@/lib/messages";
import {
  normalizeInviteCode,
  normalizePhoneNumber,
  resolvePublicChatView,
  toPublicChatMessages,
} from "@/lib/public-chat";
import { parseSseChunks } from "@/lib/stream";
import type { PublicAuthUserInfo } from "@/lib/types";
import type {
  PublicChatAuthState as AuthState,
  PublicChatMessage as ChatMessage,
} from "@/lib/public-chat";

function formatElapsed(startedAt?: number) {
  if (!startedAt) return "0s";
  const seconds = Math.max(0, Math.floor((Date.now() - startedAt) / 1000));
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remain = seconds % 60;
  return `${minutes}m ${remain}s`;
}


function LoadingCard() {
  return (
    <div
      style={{
        background: "#f8fafc",
        "min-height": "100vh",
        "padding-top": "56px",
        display: "flex",
        "align-items": "center",
        "justify-content": "center",
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
      }}
    >
      <div
        style={{
          "max-width": "480px",
          width: "100%",
          padding: "0 24px",
          "text-align": "center",
        }}
      >
        <div
          style={{
            padding: "40px 28px",
            "border-radius": "16px",
            border: "1px solid rgba(0,0,0,0.08)",
            background: "#fff",
            "box-shadow": "0 4px 24px rgba(0,0,0,0.06)",
          }}
        >
          <div
            style={{
              width: "44px",
              height: "44px",
              "border-radius": "50%",
              background: "#f59e0b",
              "box-shadow": "0 6px 20px rgba(245,158,11,0.28)",
              display: "flex",
              "align-items": "center",
              "justify-content": "center",
              margin: "0 auto 20px",
            }}
          >
            <div class="h-5 w-5 animate-spin rounded-full border-2 border-white/35 border-t-white" />
          </div>
          <h1
            style={{
              "font-size": "18px",
              "font-weight": "700",
              color: "#0f172a",
              margin: "0 0 8px",
            }}
          >
            正在恢复登录状态
          </h1>
          <p style={{ "font-size": "13px", color: "#94a3b8", margin: "0", "line-height": "1.7" }}>
            校验当前会话，恢复聊天内容和长连接更新
          </p>
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
  onInput: (value: string) => void;
  onPhoneInput: (value: string) => void;
  onSubmit: () => void;
}) {
  return (
    <div
      style={{
        background: "#f8fafc",
        "min-height": "100vh",
        "padding-top": "56px",
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
        display: "flex",
        "align-items": "center",
        "justify-content": "center",
      }}
    >
      <div
        style={{
          "max-width": "480px",
          width: "100%",
          padding: "0 24px 64px",
          "text-align": "center",
        }}
      >
        {/* Heading */}
        <div style={{ "margin-bottom": "28px" }}>
          <h1
            style={{
              "font-size": "26px",
              "font-weight": "700",
              color: "#0f172a",
              margin: "0 0 10px",
              "letter-spacing": "-0.02em",
            }}
          >
            开启深度投研之旅
          </h1>
          <p style={{ "font-size": "14px", color: "#64748b", margin: "0", "line-height": "1.7" }}>
            输入邀请码和手机号，进入单会话聊天界面
          </p>
        </div>

        {/* Form card */}
        <div
          style={{
            padding: "28px",
            "border-radius": "16px",
            border: "1px solid rgba(0,0,0,0.08)",
            background: "#fff",
            "box-shadow": "0 4px 24px rgba(0,0,0,0.06)",
          }}
        >
          <form
            onSubmit={(event) => {
              event.preventDefault();
              props.onSubmit();
            }}
          >
            <div
              style={{
                "border-radius": "10px",
                border: "1px solid rgba(0,0,0,0.10)",
                overflow: "hidden",
                "margin-bottom": "16px",
              }}
            >
              <input
                type="text"
                value={props.inviteCode}
                onInput={(event) =>
                  props.onInput(normalizeInviteCode(event.currentTarget.value))
                }
                placeholder="邀请码"
                autocomplete="off"
                autocapitalize="characters"
                spellcheck={false}
                style={{
                  display: "block",
                  width: "100%",
                  padding: "13px 14px",
                  "font-size": "15px",
                  "font-family": "inherit",
                  color: "#0f172a",
                  background: "#fafafa",
                  border: "none",
                  "border-bottom": "1px solid rgba(0,0,0,0.08)",
                  outline: "none",
                  "box-sizing": "border-box",
                }}
              />
              <input
                type="tel"
                value={props.phoneNumber}
                onInput={(event) =>
                  props.onPhoneInput(normalizePhoneNumber(event.currentTarget.value))
                }
                placeholder="手机号"
                autocomplete="tel"
                spellcheck={false}
                style={{
                  display: "block",
                  width: "100%",
                  padding: "13px 14px",
                  "font-size": "15px",
                  "font-family": "inherit",
                  color: "#0f172a",
                  background: "#fafafa",
                  border: "none",
                  outline: "none",
                  "box-sizing": "border-box",
                }}
              />
            </div>

            <button
              type="submit"
              disabled={
                props.loading ||
                !props.inviteCode.trim() ||
                !props.phoneNumber.trim()
              }
              style={{
                display: "block",
                width: "100%",
                padding: "13px 24px",
                "border-radius": "8px",
                background:
                  props.loading || !props.inviteCode.trim() || !props.phoneNumber.trim()
                    ? "rgba(245,158,11,0.35)"
                    : "#f59e0b",
                border: "none",
                cursor:
                  props.loading || !props.inviteCode.trim() || !props.phoneNumber.trim()
                    ? "not-allowed"
                    : "pointer",
                "font-family": "inherit",
                "font-size": "15px",
                "font-weight": "700",
                color: "#fff",
                "box-shadow":
                  props.loading || !props.inviteCode.trim() || !props.phoneNumber.trim()
                    ? "none"
                    : "0 4px 16px rgba(245,158,11,0.30)",
                transition: "background 0.2s",
                "margin-bottom": "12px",
              }}
            >
              {props.loading ? "验证中…" : "开始对话"}
            </button>

            <p style={{ "font-size": "12px", color: "#94a3b8", margin: "0", "line-height": "1.6" }}>
              验证通过后自动恢复你的单线程 Web 会话
            </p>
          </form>
        </div>

        <Show when={props.error}>
          <div
            style={{
              "margin-top": "12px",
              padding: "12px 16px",
              "border-radius": "10px",
              border: "1px solid rgba(239,68,68,0.20)",
              background: "rgba(239,68,68,0.05)",
              "font-size": "13px",
              color: "#ef4444",
              "text-align": "left",
            }}
          >
            {props.error}
          </div>
        </Show>

        <div
          style={{
            "margin-top": "20px",
            display: "flex",
            "justify-content": "center",
            "flex-wrap": "wrap",
            gap: "8px",
          }}
        >
          {["单会话", "长连接更新", "邀请码 + 手机号"].map((label) => (
            <span
              style={{
                padding: "4px 12px",
                "border-radius": "999px",
                border: "1px solid rgba(0,0,0,0.08)",
                "font-size": "11px",
                color: "#94a3b8",
                background: "#fff",
              }}
            >
              {label}
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}

function assistantMarkdownClass(extra: string = "") {
  return [
    "break-words text-[14px] leading-[1.75] text-[#0f172a]",
    "[&_*]:max-w-full",
    "[&_p]:my-0 [&_p+*]:mt-2",
    "[&_strong]:text-[#0f172a] [&_strong]:font-semibold",
    "[&_pre]:mt-3 [&_pre]:max-w-full [&_pre]:overflow-x-auto [&_pre]:rounded-xl [&_pre]:border-0 [&_pre]:shadow-none",
    "[&_code]:rounded [&_code]:bg-black/[0.06] [&_code]:px-1.5 [&_code]:py-0.5 [&_code]:text-[12px] [&_code]:font-[var(--font-mono,'JetBrains_Mono',monospace)]",
    "[&_ul]:my-2 [&_ol]:my-2 [&_li]:my-0.5",
    "[&_blockquote]:my-3 [&_blockquote]:border-l-2 [&_blockquote]:border-black/12 [&_blockquote]:pl-3 [&_blockquote]:text-[#64748b]",
    extra,
  ].join(" ");
}

function AssistantBody(props: { content: string; white?: boolean }) {
  const parts = createMemo(() => parseMessageContent(props.content));
  const hasImage = () => parts().some((part) => part.type === "image");
  const markdownClass = () =>
    assistantMarkdownClass(props.white ? "!text-white [&_*]:!text-white" : "");

  return (
    <Show
      when={hasImage()}
      fallback={<Markdown text={props.content} class={markdownClass()} />}
    >
      <For each={parts()}>
        {(part) => (
          <Switch>
            <Match when={part.type === "image"}>
              <img src={part.value} alt="" class="mt-2 max-w-full rounded-lg" />
            </Match>
            <Match when={part.type === "text"}>
              <Markdown text={part.value} class={markdownClass()} />
            </Match>
          </Switch>
        )}
      </For>
    </Show>
  );
}

function UserBubble(props: { content: string }) {
  return (
    <div
      class="pub-msg-in"
      style={{
        display: "flex",
        "justify-content": "flex-end",
        "margin-bottom": "12px",
      }}
    >
      <div
        style={{
          "max-width": "72%",
          background: "#f59e0b",
          color: "#fff",
          "border-radius": "18px 18px 4px 18px",
          padding: "11px 16px",
          "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
          "font-size": "14px",
          "line-height": "1.65",
          "box-shadow": "0 2px 8px rgba(245,158,11,0.20)",
          "white-space": "pre-wrap",
          "word-break": "break-word",
        }}
      >
        {props.content}
      </div>
    </div>
  );
}

function AssistantBubble(props: { content: string }) {
  return (
    <div
      class="pub-msg-in"
      style={{
        display: "flex",
        "justify-content": "flex-start",
        "margin-bottom": "12px",
      }}
    >
      <div
        style={{
          "max-width": "78%",
          background: "#fff",
          border: "1px solid rgba(0,0,0,0.09)",
          "border-radius": "4px 18px 18px 18px",
          padding: "12px 16px",
          "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
          color: "#0f172a",
          "box-shadow": "0 1px 4px rgba(0,0,0,0.05)",
        }}
      >
        <div
          style={{
            display: "flex",
            "align-items": "center",
            gap: "6px",
            "margin-bottom": "9px",
          }}
        >
          <span
            style={{
              width: "7px",
              height: "7px",
              "border-radius": "50%",
              background: "#f59e0b",
              display: "inline-block",
              "flex-shrink": "0",
            }}
          />
          <span
            style={{
              "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
              "font-size": "11px",
              "font-weight": "600",
              "letter-spacing": "0.15em",
              "text-transform": "uppercase",
              color: "#64748b",
            }}
          >
            HONE
          </span>
        </div>
        <AssistantBody content={props.content} />
      </div>
    </div>
  );
}

function PendingBubble(props: {
  message: ChatMessage;
  onStop: () => void;
  onDismiss: () => void;
}) {
  const [elapsed, setElapsed] = createSignal(0);

  createEffect(() => {
    if (!props.message.startedAt) {
      setElapsed(0);
      return;
    }
    const tick = () => {
      const seconds = Math.max(
        0,
        Math.floor((Date.now() - (props.message.startedAt ?? 0)) / 1000),
      );
      setElapsed(seconds);
    };
    tick();
    if (props.message.phase === "done" || props.message.phase === "error") return;
    const timer = setInterval(tick, 1000);
    onCleanup(() => clearInterval(timer));
  });

  const terminal = () => props.message.phase === "error";
  const labelText = () => {
    switch (props.message.phase) {
      case "error":
        return "HONE 出错了";
      case "streaming":
        return "HONE 输出中";
      case "running":
        return "HONE 执行中";
      default:
        return "HONE 思考中";
    }
  };
  const showDots = () => !props.message.content && !terminal();

  return (
    <div
      class="pub-msg-in"
      style={{
        display: "flex",
        "justify-content": "flex-start",
        "margin-bottom": "12px",
      }}
    >
      <div
        style={{
          "max-width": "78%",
          "min-width": "200px",
          background: "#fff",
          border: terminal()
            ? "1px solid rgba(239,68,68,0.22)"
            : "1px solid rgba(0,0,0,0.09)",
          "border-radius": "4px 18px 18px 18px",
          padding: "12px 16px",
          "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
          color: "#0f172a",
          "box-shadow": "0 1px 4px rgba(0,0,0,0.05)",
        }}
      >
        <div
          style={{
            display: "flex",
            "align-items": "center",
            "justify-content": "space-between",
            gap: "8px",
            "margin-bottom": props.message.content ? "10px" : "0",
          }}
        >
          <div style={{ display: "flex", "align-items": "center", gap: "6px" }}>
            <span
              class={
                props.message.phase === "thinking" ||
                props.message.phase === "streaming"
                  ? "pub-pulsedot"
                  : ""
              }
              style={{
                width: "7px",
                height: "7px",
                "border-radius": "50%",
                display: "inline-block",
                "flex-shrink": "0",
                background: terminal() ? "#ef4444" : "#f59e0b",
              }}
            />
            <span
              style={{
                "font-size": "11px",
                "font-weight": "600",
                "letter-spacing": "0.15em",
                "text-transform": "uppercase",
                color: "#64748b",
              }}
            >
              {labelText()}
            </span>
            <span
              style={{
                "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                "font-size": "10px",
                color: "rgba(0,0,0,0.20)",
              }}
            >
              {elapsed()}s
            </span>
          </div>
          <Show
            when={!terminal()}
            fallback={
              <button
                type="button"
                onClick={props.onDismiss}
                style={{
                  background: "none",
                  border: "none",
                  cursor: "pointer",
                  color: "#64748b",
                  "font-size": "12px",
                  padding: "0 2px",
                }}
              >
                ✕
              </button>
            }
          >
            <button
              type="button"
              onClick={props.onStop}
              style={{
                display: "inline-flex",
                "align-items": "center",
                gap: "4px",
                background: "none",
                border: "none",
                cursor: "pointer",
                padding: "2px 6px",
                "border-radius": "4px",
                "font-size": "11px",
                color: "#64748b",
                transition: "all 0.15s",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = "rgba(239,68,68,0.08)";
                e.currentTarget.style.color = "#ef4444";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = "none";
                e.currentTarget.style.color = "#64748b";
              }}
            >
              <span
                style={{
                  width: "6px",
                  height: "6px",
                  "border-radius": "1px",
                  background: "currentColor",
                  display: "inline-block",
                }}
              />
              停止
            </button>
          </Show>
        </div>

        <Show when={showDots()}>
          <div style={{ display: "flex", gap: "4px", padding: "4px 0" }}>
            <span class="pub-dot1" />
            <span class="pub-dot2" />
            <span class="pub-dot3" />
          </div>
        </Show>

        <Show when={(props.message.steps?.length ?? 0) > 0}>
          <ul
            style={{
              margin: props.message.content ? "0 0 8px" : "4px 0 0",
              padding: "0",
              "list-style": "none",
              "font-size": "12px",
              "line-height": "1.7",
              color: "#64748b",
            }}
          >
            <For each={props.message.steps}>
              {(step) => (
                <li
                  style={{
                    display: "flex",
                    "align-items": "flex-start",
                    gap: "6px",
                    "word-break": "break-word",
                  }}
                >
                  <span style={{ color: "#f59e0b", "flex-shrink": "0" }}>•</span>
                  <span>{step}</span>
                </li>
              )}
            </For>
          </ul>
        </Show>

        <Show when={props.message.content}>
          <div style={{ "white-space": "pre-wrap", "word-break": "break-word" }}>
            <AssistantBody content={props.message.content} />
            <Show when={props.message.phase === "streaming"}>
              <span class="pub-cursor" />
            </Show>
          </div>
        </Show>

        <Show when={terminal()}>
          <div
            style={{
              "font-size": "13px",
              color: "#ef4444",
              "margin-top": "4px",
            }}
          >
            {props.message.statusText || "请求出错，请重试。"}
          </div>
        </Show>
      </div>
    </div>
  );
}

function Composer(props: {
  draft: string;
  onDraftChange: (v: string) => void;
  onSend: () => void;
  onStop: () => void;
  isSending: boolean;
  remaining: number | undefined;
}) {
  const [focused, setFocused] = createSignal(false);
  let taRef: HTMLTextAreaElement | undefined;

  const canSend = () =>
    !props.isSending &&
    !!props.draft.trim() &&
    (props.remaining === undefined || props.remaining > 0);

  createEffect(() => {
    if (!props.isSending && taRef) taRef.focus();
  });

  const onKey = (e: KeyboardEvent) => {
    if (e.isComposing) return;
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (canSend()) props.onSend();
    }
  };

  return (
    <div
      style={{
        padding: "8px 20px 16px",
        "border-top": "1px solid rgba(0,0,0,0.08)",
        background: "#fff",
        "flex-shrink": "0",
      }}
    >
      <div
        style={{
          position: "relative",
          "border-radius": "24px",
          border: focused()
            ? "1px solid #f59e0b"
            : "1px solid rgba(0,0,0,0.10)",
          background: "#fff",
          "box-shadow": focused()
            ? "0 4px 20px rgba(245,158,11,0.12)"
            : "0 2px 8px rgba(0,0,0,0.05)",
          transition: "border-color 0.2s, box-shadow 0.2s",
          overflow: "hidden",
        }}
      >
        <textarea
          ref={taRef}
          rows={3}
          placeholder={
            props.remaining === 0
              ? "今日额度已用完"
              : "输入你的问题，按 Enter 发送"
          }
          value={props.draft}
          disabled={props.isSending}
          onInput={(e) => props.onDraftChange(e.currentTarget.value)}
          onKeyDown={onKey}
          onFocus={() => setFocused(true)}
          onBlur={() => setFocused(false)}
          style={{
            width: "100%",
            "min-height": "90px",
            resize: "none",
            border: "none",
            outline: "none",
            background: "transparent",
            padding: "16px 20px 44px",
            "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
            "font-size": "14px",
            "line-height": "1.65",
            color: "#0f172a",
          }}
        />
        <div
          style={{
            position: "absolute",
            bottom: "0",
            left: "0",
            right: "0",
            display: "flex",
            "align-items": "center",
            "justify-content": "space-between",
            padding: "8px 14px 10px",
            background: "linear-gradient(to top,#fff 60%,transparent)",
          }}
        >
          <span style={{ "font-size": "11px", color: "rgba(0,0,0,0.28)" }}>
            Shift + Enter 换行
          </span>
          <div style={{ display: "flex", gap: "6px", "align-items": "center" }}>
            <Show when={props.isSending}>
              <button
                type="button"
                onClick={props.onStop}
                style={{
                  padding: "5px 13px",
                  "border-radius": "7px",
                  border: "1px solid rgba(0,0,0,0.10)",
                  background: "#fff",
                  "font-size": "12px",
                  "font-weight": "600",
                  color: "#64748b",
                  cursor: "pointer",
                  transition: "all 0.15s",
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.borderColor = "#ef4444";
                  e.currentTarget.style.color = "#ef4444";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.borderColor = "rgba(0,0,0,0.10)";
                  e.currentTarget.style.color = "#64748b";
                }}
              >
                停止
              </button>
            </Show>
            <button
              type="button"
              onClick={() => canSend() && props.onSend()}
              disabled={!canSend()}
              style={{
                width: "34px",
                height: "34px",
                "border-radius": "10px",
                background: canSend() ? "#f59e0b" : "rgba(0,0,0,0.07)",
                border: "none",
                cursor: canSend() ? "pointer" : "default",
                display: "flex",
                "align-items": "center",
                "justify-content": "center",
                transition: "all 0.2s",
              }}
              onMouseEnter={(e) => {
                if (canSend()) e.currentTarget.style.transform = "scale(1.06)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.transform = "scale(1)";
              }}
            >
              <svg
                viewBox="0 0 20 20"
                width="16"
                height="16"
                fill={canSend() ? "white" : "rgba(0,0,0,0.25)"}
              >
                <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function PublicChatPage() {
  const navigate = useNavigate();
  const [authState, setAuthState] = createSignal<AuthState>("loading");
  const [loginError, setLoginError] = createSignal("");
  const [inviteCode, setInviteCode] = createSignal("");
  const [phoneNumber, setPhoneNumber] = createSignal("");
  const [messages, setMessages] = createStore<ChatMessage[]>([]);
  const [draft, setDraft] = createSignal("");
  const [sendError, setSendError] = createSignal("");
  const [isSending, setIsSending] = createSignal(false);
  const [sessionInfo, setSessionInfo] = createSignal<{
    userId: string;
    remainingToday: number;
    dailyLimit: number;
  } | null>(null);
  let eventSource: EventSource | null = null;
  let activeController: AbortController | null = null;
  let scrollRef: HTMLDivElement | undefined;
  let sessionSyncGeneration = 0;

  const isAuthExpiredError = (error: unknown) =>
    error instanceof Error && /401|未登录|过期/.test(error.message);

  const scrollToBottom = () => {
    requestAnimationFrame(() => {
      if (!scrollRef) return;
      scrollRef.scrollTop = scrollRef.scrollHeight;
    });
  };

  const findMessageIndex = (id: string) =>
    messages.findIndex((message) => message.id === id);

  const clearMessages = () => {
    setMessages(reconcile([], { key: "id" }));
  };

  const appendMessage = (message: ChatMessage) => {
    setMessages(messages.length, message);
  };

  const appendMessages = (...nextMessages: ChatMessage[]) => {
    const start = messages.length;
    nextMessages.forEach((message, offset) => {
      setMessages(start + offset, message);
    });
  };

  const replaceHistoryMessages = (nextMessages: ChatMessage[]) => {
    setMessages(reconcile(nextMessages, { key: "id" }));
  };

  const removeMessageById = (id: string) => {
    setMessages(
      reconcile(
        messages.filter((message) => message.id !== id),
        { key: "id" },
      ),
    );
  };

  const patchMessageAtIndex = (index: number, patch: Partial<ChatMessage>) => {
    const current = messages[index];
    if (!current) return;

    if ("content" in patch && patch.content !== current.content) {
      setMessages(index, "content", patch.content ?? "");
    }
    if ("phase" in patch && patch.phase !== current.phase) {
      setMessages(index, "phase", patch.phase);
    }
    if ("statusText" in patch && patch.statusText !== current.statusText) {
      setMessages(index, "statusText", patch.statusText);
    }
    if ("startedAt" in patch && patch.startedAt !== current.startedAt) {
      setMessages(index, "startedAt", patch.startedAt);
    }
    if ("steps" in patch && patch.steps !== current.steps) {
      setMessages(index, "steps", patch.steps);
    }
  };

  const patchMessageById = (id: string, patch: Partial<ChatMessage>) => {
    const index = findMessageIndex(id);
    if (index < 0) return;
    patchMessageAtIndex(index, patch);
  };

  const applySessionInfo = (user: PublicAuthUserInfo) => {
    setSessionInfo({
      userId: user.user_id,
      remainingToday: user.remaining_today,
      dailyLimit: user.daily_limit,
    });
    setAuthState("ready");
    setLoginError("");
  };

  const restoreSession = async () => {
    const generation = ++sessionSyncGeneration;
    try {
      const user = await getPublicAuthMe();
      if (generation !== sessionSyncGeneration) return;
      applySessionInfo(user);
      const history = await getPublicHistory();
      if (generation !== sessionSyncGeneration) return;
      replaceHistoryMessages(toPublicChatMessages(history));
      await ensurePushEvents();
      if (generation !== sessionSyncGeneration) return;
      scrollToBottom();
    } catch (error) {
      if (generation !== sessionSyncGeneration) return;
      setSessionInfo(null);
      clearMessages();
      setAuthState("logged_out");
      if (error instanceof Error && !isAuthExpiredError(error)) {
        setLoginError(error.message);
      }
    }
  };

  const publicChatView = () => resolvePublicChatView(authState());

  const ensurePushEvents = async () => {
    if (eventSource) return;
    let nextEventSource: EventSource | null = null;
    try {
      nextEventSource = await connectPublicEvents();
      nextEventSource.addEventListener("scheduled_message", (event) => {
        const data = JSON.parse(event.data || "{}") as { text?: string };
        if (!data.text?.trim()) return;
        appendMessage({
          id: messageId(),
          role: "assistant",
          content: data.text ?? "",
          phase: "done",
          steps: [],
        });
        scrollToBottom();
      });
      nextEventSource.addEventListener("push_message", (event) => {
        const data = JSON.parse(event.data || "{}") as { text?: string };
        if (!data.text?.trim()) return;
        appendMessage({
          id: messageId(),
          role: "assistant",
          content: data.text ?? "",
          phase: "done",
          steps: [],
        });
        scrollToBottom();
      });
      nextEventSource.onerror = () => {
        nextEventSource?.close();
        if (eventSource === nextEventSource) {
          eventSource = null;
        }
      };
      eventSource = nextEventSource;
    } catch {
      nextEventSource?.close();
      if (eventSource === nextEventSource) {
        eventSource = null;
      }
    }
  };

  const refreshSessionAfterSend = async () => {
    const generation = ++sessionSyncGeneration;
    try {
      const user = await getPublicAuthMe();
      if (generation !== sessionSyncGeneration) return;
      applySessionInfo(user);
      const history = await getPublicHistory();
      if (generation !== sessionSyncGeneration) return;
      replaceHistoryMessages(toPublicChatMessages(history));
      if (!eventSource) {
        await ensurePushEvents();
      }
      if (generation !== sessionSyncGeneration) return;
      scrollToBottom();
    } catch (error) {
      if (generation !== sessionSyncGeneration) return;
      if (isAuthExpiredError(error)) {
        setSessionInfo(null);
        clearMessages();
        setAuthState("logged_out");
        return;
      }
      if (error instanceof Error) {
        setSendError(error.message);
      }
    }
  };

  onMount(() => {
    void restoreSession();
  });

  onCleanup(() => {
    eventSource?.close();
    activeController?.abort();
  });

  createEffect(() => {
    messages.length;
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
      await restoreSession();
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
    clearMessages();
    setDraft("");
    setSendError("");
    setSessionInfo(null);
    setAuthState("logged_out");
  };

  const appendAssistantStep = (messageIdValue: string, step: string) => {
    const normalized = step.trim();
    if (!normalized) return;
    const index = findMessageIndex(messageIdValue);
    if (index < 0) return;
    const steps = messages[index]?.steps ?? [];
    if (steps.includes(normalized)) return;
    patchMessageAtIndex(index, { steps: [...steps, normalized] });
  };

  const handleSend = async () => {
    const text = draft().trim();
    if (!text || authState() !== "ready" || activeController) return;
    const assistantId = messageId();
    setSendError("");
    setDraft("");
    setIsSending(true);
    appendMessages(
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
    );
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
            patchMessageById(assistantId, {
              phase: "thinking",
              statusText: "Hone 思考中",
            });
          }

          if (event.event === "tool_call") {
            if (event.data.status === "start") {
              const tool = event.data.tool?.trim() ?? "";
              const detail = (event.data.text ?? event.data.reasoning ?? "").trim();
              const step = tool
                ? detail
                  ? `正在调用 Tool: ${tool} · ${detail}`
                  : `正在调用 Tool: ${tool}`
                : detail || "处理中…";
              appendAssistantStep(assistantId, step);
              patchMessageById(assistantId, {
                phase: "running",
                statusText: step,
              });
            }
          }

          if (event.event === "assistant_delta") {
            const content = event.data.content ?? "";
            const index = findMessageIndex(assistantId);
            if (index >= 0) {
              patchMessageAtIndex(index, {
                phase: "streaming",
                statusText: "整理正式回复",
                content: messages[index].content + content,
              });
            }
          }

          if (event.event === "run_error" || event.event === "error") {
            const messageText =
              event.event === "run_error"
                ? (event.data.message ?? "处理失败，请重试")
                : (event.data.text ?? "处理失败，请重试");
            patchMessageById(assistantId, {
              phase: "error",
              statusText: messageText,
            });
          }

          if (event.event === "run_finished") {
            const index = findMessageIndex(assistantId);
            if (index >= 0) {
              const message = messages[index];
              patchMessageAtIndex(index, {
                phase: event.data.success === false ? "error" : "done",
                statusText:
                  event.data.success === false
                    ? message.statusText || "处理失败，请重试"
                    : message.content
                      ? ""
                      : "本轮没有返回正文",
              });
            }
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
      patchMessageById(assistantId, {
        phase: "error",
        statusText: messageText,
      });
    } finally {
      activeController = null;
      setIsSending(false);
      void refreshSessionAfterSend();
    }
  };

  return (
    <>
      <PublicNav />
      <Switch>
        <Match when={publicChatView() === "loading"}>
          <LoadingCard />
        </Match>
        <Match when={publicChatView() === "login"}>
          <LoginCard
            inviteCode={inviteCode()}
            phoneNumber={phoneNumber()}
            loading={authState() === "logging_in"}
            error={loginError()}
            onInput={setInviteCode}
            onPhoneInput={setPhoneNumber}
            onSubmit={() => void handleLogin()}
          />
        </Match>
        <Match when={publicChatView() === "chat"}>
          {/* Fixed container below the 56px PublicNav */}
          <div
            style={{
              position: "fixed",
              top: "56px",
              left: 0,
              right: 0,
              bottom: 0,
              display: "flex",
              "flex-direction": "column",
              background: "#fff",
            }}
          >
            {/* Session strip — 34px */}
            <div
              style={{
                height: "34px",
                display: "flex",
                "align-items": "center",
                "justify-content": "space-between",
                padding: "0 20px",
                "border-top": "1px solid rgba(0,0,0,0.06)",
                "border-bottom": "1px solid rgba(0,0,0,0.08)",
                background: "#fafbfc",
                "flex-shrink": "0",
              }}
            >
              <span style={{ "font-size": "12px", color: "#64748b" }}>
                {sessionInfo()?.dailyLimit
                  ? `今日剩余 ${sessionInfo()?.remainingToday}/${sessionInfo()?.dailyLimit}`
                  : "当前实例未启用次数限制"}
              </span>
              <div style={{ display: "flex", "align-items": "center", gap: "10px" }}>
                <button
                  type="button"
                  onClick={() => navigate("/me")}
                  style={{
                    "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
                    "font-size": "11px",
                    color: "#64748b",
                    background: "none",
                    border: "none",
                    cursor: "pointer",
                    padding: "0",
                  }}
                >
                  {sessionInfo()?.userId ?? "个人"}
                </button>
                <button
                  type="button"
                  onClick={() => void handleLogout()}
                  style={{
                    padding: "3px 10px",
                    "border-radius": "5px",
                    border: "1px solid rgba(0,0,0,0.09)",
                    background: "#fff",
                    "font-size": "11px",
                    cursor: "pointer",
                    color: "#64748b",
                  }}
                >
                  退出登录
                </button>
              </div>
            </div>

            {/* Messages list */}
            <div
              ref={scrollRef}
              style={{
                flex: "1",
                "overflow-y": "auto",
                padding: "20px 0",
                "scroll-behavior": "smooth",
              }}
            >
              <div style={{ "max-width": "760px", margin: "0 auto", padding: "0 24px" }}>
                <Show
                  when={messages.length > 0}
                  fallback={
                    <div
                      style={{
                        display: "flex",
                        "flex-direction": "column",
                        "align-items": "center",
                        "justify-content": "center",
                        "text-align": "center",
                        padding: "48px 16px",
                      }}
                    >
                      <div
                        style={{
                          padding: "12px 24px",
                          "border-radius": "999px",
                          background: "rgba(245,158,11,0.08)",
                          border: "1px solid rgba(245,158,11,0.18)",
                        }}
                      >
                        <Logo class="h-8 w-auto" />
                      </div>
                      <h1
                        style={{
                          "font-size": "28px",
                          "font-weight": "700",
                          color: "#0f172a",
                          margin: "20px 0 10px",
                          "letter-spacing": "-0.02em",
                        }}
                      >
                        问 Hone 一个问题
                      </h1>
                      <p
                        style={{
                          "font-size": "14px",
                          "line-height": "1.75",
                          color: "#64748b",
                          "max-width": "480px",
                          margin: "0",
                        }}
                      >
                        这里没有侧边栏，也没有多会话切换。输入问题后，Hone
                        会先展示思考和工具执行过程，再把同一张回复卡片更新成最终答案。
                      </p>
                    </div>
                  }
                >
                  <For each={messages}>
                    {(message) => (
                      <Show
                        when={message.role === "user"}
                        fallback={
                          <Show
                            when={
                              !message.phase ||
                              message.phase === "done"
                            }
                            fallback={
                              <PendingBubble
                                message={message}
                                onStop={() => activeController?.abort()}
                                onDismiss={() => {
                                  removeMessageById(message.id);
                                }}
                              />
                            }
                          >
                            <AssistantBubble content={message.content} />
                          </Show>
                        }
                      >
                        <UserBubble content={message.content} />
                      </Show>
                    )}
                  </For>
                </Show>
                <div style={{ height: "8px" }} />
              </div>
            </div>

            {/* Composer */}
            <div style={{ "max-width": "760px", width: "100%", margin: "0 auto", padding: "0 4px" }}>
              <Show when={sendError()}>
                <div
                  style={{
                    margin: "0 24px 4px",
                    padding: "8px 12px",
                    "border-radius": "8px",
                    border: "1px solid rgba(239,68,68,0.20)",
                    background: "rgba(239,68,68,0.05)",
                    "font-size": "12px",
                    color: "#ef4444",
                  }}
                >
                  {sendError()}
                </div>
              </Show>
              <Composer
                draft={draft()}
                onDraftChange={setDraft}
                onSend={() => void handleSend()}
                onStop={() => activeController?.abort()}
                isSending={isSending()}
                remaining={sessionInfo()?.remainingToday}
              />
            </div>
          </div>
        </Match>
      </Switch>
    </>
  );
}
