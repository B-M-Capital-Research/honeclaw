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
  uploadPublicAttachments,
  type PublicChatAttachmentInput,
} from "@/lib/api";
import { buildApiUrl } from "@/lib/backend";
import { parseMessageContent, messageId } from "@/lib/messages";
import {
  normalizeInviteCode,
  normalizePhoneNumber,
  resolvePublicChatView,
  stripAttachmentMarkers,
  toPublicChatMessages,
} from "@/lib/public-chat";
import { parseSseChunks } from "@/lib/stream";
import type { PublicAuthUserInfo } from "@/lib/types";
import type {
  PublicChatAttachment,
  PublicChatAuthState as AuthState,
  PublicChatMessage as ChatMessage,
} from "@/lib/public-chat";

const PUBLIC_IMAGE_ENDPOINT = "/api/public/image";
const MAX_ATTACHMENTS = 4;

function publicAttachmentUrl(att: PublicChatAttachment): string {
  if (att.previewUrl) return att.previewUrl;
  return buildApiUrl(
    `${PUBLIC_IMAGE_ENDPOINT}?path=${encodeURIComponent(att.path)}`,
  );
}

function formatBytes(bytes?: number) {
  if (!bytes) return "";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function fileExtension(name: string) {
  const parts = name.split(".");
  return parts.length > 1
    ? parts[parts.length - 1]!.toUpperCase().slice(0, 4)
    : "FILE";
}

function classifyKind(file: File) {
  if (file.type.startsWith("image/")) return "image";
  if (file.type === "application/pdf") return "pdf";
  return "file";
}

function renamePasteFile(file: File) {
  const ext = file.type.split("/")[1]?.split(";")[0] || "bin";
  const stamp = new Date()
    .toISOString()
    .replace(/[:.]/g, "-")
    .replace("T", "_")
    .slice(0, 19);
  return new File([file], `pasted-${stamp}.${ext}`, {
    type: file.type,
    lastModified: file.lastModified,
  });
}

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
  const cleaned = createMemo(() => stripAttachmentMarkers(props.content));
  const parts = createMemo(() =>
    parseMessageContent(cleaned(), { imageEndpoint: PUBLIC_IMAGE_ENDPOINT }),
  );
  const hasImage = () => parts().some((part) => part.type === "image");
  const markdownClass = () =>
    assistantMarkdownClass(props.white ? "!text-white [&_*]:!text-white" : "");

  return (
    <Show
      when={hasImage()}
      fallback={<Markdown text={cleaned()} class={markdownClass()} />}
    >
      <For each={parts()}>
        {(part) => (
          <Switch>
            <Match when={part.type === "image"}>
              <img
                src={part.value}
                alt=""
                class="hone-assistant-image mt-2 max-w-full cursor-zoom-in rounded-lg"
                data-testid="assistant-inline-image"
              />
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

function ImageMosaic(props: {
  images: PublicChatAttachment[];
  onOpen: (index: number) => void;
  inUserBubble?: boolean;
}) {
  const count = () => props.images.length;

  return (
    <Show
      when={count() === 1}
      fallback={
        <div
          style={{
            display: "grid",
            "grid-template-columns": `repeat(${count() === 2 ? 2 : 2}, 1fr)`,
            gap: "2px",
            "border-radius": "12px",
            overflow: "hidden",
            "max-width": "380px",
            "aspect-ratio": count() === 2 ? "2 / 1" : "1 / 1",
          }}
        >
          <For each={props.images.slice(0, 4)}>
            {(img, index) => (
              <div
                onClick={() => props.onOpen(index())}
                style={{
                  position: "relative",
                  cursor: "zoom-in",
                  overflow: "hidden",
                  background: "#f1f5f9",
                  ...(count() === 3 && index() === 0
                    ? { "grid-row": "span 2" }
                    : {}),
                }}
              >
                <img
                  src={publicAttachmentUrl(img)}
                  alt={img.name}
                  style={{
                    width: "100%",
                    height: "100%",
                    "object-fit": "cover",
                    display: "block",
                  }}
                  data-testid="user-attachment-image"
                />
              </div>
            )}
          </For>
        </div>
      }
    >
      <div
        onClick={() => props.onOpen(0)}
        style={{
          "border-radius": "12px",
          overflow: "hidden",
          cursor: "zoom-in",
          "max-width": "380px",
          "line-height": "0",
          position: "relative",
        }}
      >
        <img
          src={publicAttachmentUrl(props.images[0]!)}
          alt={props.images[0]!.name}
          style={{ width: "100%", height: "auto", display: "block" }}
          data-testid="user-attachment-image"
        />
      </div>
    </Show>
  );
}

function FileCard(props: { file: PublicChatAttachment; inUserBubble?: boolean }) {
  const ext = () => fileExtension(props.file.name);
  const iconBg = () =>
    props.inUserBubble ? "rgba(255,255,255,0.22)" : "rgba(245,158,11,0.12)";
  const iconColor = () => (props.inUserBubble ? "#fff" : "#d97706");
  const textColor = () =>
    props.inUserBubble ? "rgba(255,255,255,0.95)" : "#0f172a";
  const subColor = () =>
    props.inUserBubble ? "rgba(255,255,255,0.75)" : "#64748b";
  return (
    <div
      style={{
        display: "flex",
        "align-items": "center",
        gap: "12px",
        padding: "8px 10px",
        background: props.inUserBubble
          ? "rgba(255,255,255,0.10)"
          : "rgba(0,0,0,0.02)",
        border: props.inUserBubble
          ? "1px solid rgba(255,255,255,0.15)"
          : "1px solid rgba(0,0,0,0.06)",
        "border-radius": "10px",
        "min-width": "220px",
      }}
      data-testid="attachment-file-card"
    >
      <div
        style={{
          width: "40px",
          height: "40px",
          "border-radius": "8px",
          background: iconBg(),
          display: "flex",
          "align-items": "center",
          "justify-content": "center",
          "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
          "font-size": "10px",
          "font-weight": "700",
          color: iconColor(),
          "letter-spacing": "0.04em",
          "flex-shrink": "0",
        }}
      >
        {ext()}
      </div>
      <div style={{ flex: "1", "min-width": "0" }}>
        <div
          style={{
            "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
            "font-size": "13px",
            "font-weight": "600",
            color: textColor(),
            "white-space": "nowrap",
            overflow: "hidden",
            "text-overflow": "ellipsis",
          }}
        >
          {props.file.name}
        </div>
        <Show when={props.file.size}>
          <div
            style={{
              "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
              "font-size": "11px",
              color: subColor(),
              "margin-top": "2px",
            }}
          >
            {formatBytes(props.file.size)}
          </div>
        </Show>
      </div>
    </div>
  );
}

function UserBubble(props: {
  content: string;
  attachments?: PublicChatAttachment[];
  onOpenImage: (images: PublicChatAttachment[], index: number) => void;
}) {
  const cleaned = createMemo(() => stripAttachmentMarkers(props.content));
  const images = createMemo(() =>
    (props.attachments ?? []).filter((a) => a.kind === "image"),
  );
  const files = createMemo(() =>
    (props.attachments ?? []).filter((a) => a.kind !== "image"),
  );
  const hasText = () => cleaned().length > 0;
  const hasAttach = () => images().length + files().length > 0;
  const imageOnly = () =>
    images().length > 0 && !hasText() && files().length === 0;

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
          padding: imageOnly() ? "4px" : "11px 16px",
          "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
          "font-size": "14px",
          "line-height": "1.65",
          "box-shadow": "0 2px 8px rgba(245,158,11,0.20)",
          "white-space": "pre-wrap",
          "word-break": "break-word",
        }}
      >
        <Show when={images().length > 0}>
          <div
            style={{
              "margin-bottom":
                hasText() || files().length > 0 ? "8px" : "0",
            }}
          >
            <ImageMosaic
              images={images()}
              inUserBubble
              onOpen={(index) => props.onOpenImage(images(), index)}
            />
          </div>
        </Show>
        <Show when={files().length > 0}>
          <div
            style={{
              display: "flex",
              "flex-direction": "column",
              gap: "6px",
              "margin-bottom": hasText() ? "8px" : "0",
            }}
          >
            <For each={files()}>
              {(file) => <FileCard file={file} inUserBubble />}
            </For>
          </div>
        </Show>
        <Show when={hasText()}>{cleaned()}</Show>
        <Show when={!hasAttach() && !hasText()}>{props.content}</Show>
      </div>
    </div>
  );
}

function AssistantBubble(props: {
  content: string;
  attachments?: PublicChatAttachment[];
}) {
  const nonImageAttachments = createMemo(() =>
    (props.attachments ?? []).filter((a) => a.kind !== "image"),
  );
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
        <Show when={nonImageAttachments().length > 0}>
          <div
            style={{
              display: "flex",
              "flex-direction": "column",
              gap: "6px",
              "margin-top": "12px",
            }}
          >
            <For each={nonImageAttachments()}>
              {(file) => <FileCard file={file} />}
            </For>
          </div>
        </Show>
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

function AttachPreview(props: {
  items: PublicChatAttachment[];
  onRemove: (index: number) => void;
}) {
  return (
    <Show when={props.items.length > 0}>
      <div
        style={{
          display: "flex",
          gap: "8px",
          padding: "10px 14px",
          "flex-wrap": "wrap",
          "border-bottom": "1px solid rgba(0,0,0,0.05)",
        }}
        data-testid="composer-attach-preview"
      >
        <For each={props.items}>
          {(item, index) => (
            <div style={{ position: "relative", "flex-shrink": "0" }}>
              <Show
                when={item.kind === "image"}
                fallback={
                  <div
                    style={{
                      width: "180px",
                      height: "64px",
                      padding: "0 10px",
                      display: "flex",
                      "align-items": "center",
                      gap: "10px",
                      "border-radius": "8px",
                      border: "1px solid rgba(0,0,0,0.08)",
                      background: "rgba(0,0,0,0.02)",
                    }}
                  >
                    <div
                      style={{
                        width: "36px",
                        height: "36px",
                        "border-radius": "6px",
                        background: "rgba(245,158,11,0.12)",
                        display: "flex",
                        "align-items": "center",
                        "justify-content": "center",
                        "font-family":
                          "var(--font-mono, 'JetBrains Mono', monospace)",
                        "font-size": "10px",
                        "font-weight": "700",
                        color: "#d97706",
                        "flex-shrink": "0",
                      }}
                    >
                      {fileExtension(item.name)}
                    </div>
                    <div style={{ flex: "1", "min-width": "0" }}>
                      <div
                        style={{
                          "font-size": "12px",
                          "font-weight": "600",
                          color: "#0f172a",
                          "white-space": "nowrap",
                          overflow: "hidden",
                          "text-overflow": "ellipsis",
                        }}
                      >
                        {item.name}
                      </div>
                      <div
                        style={{
                          "font-family":
                            "var(--font-mono, 'JetBrains Mono', monospace)",
                          "font-size": "10px",
                          color: "#64748b",
                        }}
                      >
                        {formatBytes(item.size)}
                      </div>
                    </div>
                  </div>
                }
              >
                <div
                  style={{
                    width: "64px",
                    height: "64px",
                    "border-radius": "8px",
                    overflow: "hidden",
                    border: "1px solid rgba(0,0,0,0.06)",
                  }}
                >
                  <img
                    src={publicAttachmentUrl(item)}
                    alt={item.name}
                    style={{
                      width: "100%",
                      height: "100%",
                      "object-fit": "cover",
                    }}
                  />
                </div>
              </Show>
              <button
                type="button"
                onClick={() => props.onRemove(index())}
                aria-label="移除附件"
                style={{
                  position: "absolute",
                  top: "-6px",
                  right: "-6px",
                  width: "20px",
                  height: "20px",
                  "border-radius": "10px",
                  background: "#0f172a",
                  color: "#fff",
                  border: "2px solid #fff",
                  cursor: "pointer",
                  "font-size": "11px",
                  "line-height": "1",
                  display: "flex",
                  "align-items": "center",
                  "justify-content": "center",
                  "box-shadow": "0 2px 6px rgba(0,0,0,0.2)",
                }}
              >
                ✕
              </button>
            </div>
          )}
        </For>
      </div>
    </Show>
  );
}

function AttachMenu(props: {
  open: boolean;
  onClose: () => void;
  onPickImage: () => void;
  onPickFile: () => void;
}) {
  return (
    <Show when={props.open}>
      <div class="pub-attach-backdrop" onClick={props.onClose} />
      <div class="pub-attach-menu" data-testid="composer-attach-menu">
        <div class="pub-attach-sheet-handle" aria-hidden="true" />
        <button
          type="button"
          class="pub-attach-item"
          data-testid="composer-pick-image"
          onClick={() => {
            props.onPickImage();
            props.onClose();
          }}
        >
          <span class="pub-attach-icon" aria-hidden="true">
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.7"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <rect x="3" y="5" width="18" height="14" rx="2.5" />
              <circle cx="8.5" cy="10" r="1.5" />
              <path d="M21 15l-5-5-8 9" />
            </svg>
          </span>
          <span class="pub-attach-label">
            <span class="pub-attach-label-title">图片</span>
            <span class="pub-attach-label-sub">照片与截图</span>
          </span>
        </button>
        <button
          type="button"
          class="pub-attach-item"
          data-testid="composer-pick-file"
          onClick={() => {
            props.onPickFile();
            props.onClose();
          }}
        >
          <span class="pub-attach-icon" aria-hidden="true">
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.7"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M14 3H7a2 2 0 00-2 2v14a2 2 0 002 2h10a2 2 0 002-2V8z" />
              <path d="M14 3v5h5" />
              <path d="M9 13h6M9 17h4" />
            </svg>
          </span>
          <span class="pub-attach-label">
            <span class="pub-attach-label-title">文件</span>
            <span class="pub-attach-label-sub">PDF · 文档 · 其他</span>
          </span>
        </button>
      </div>
    </Show>
  );
}

function Lightbox(props: {
  images: PublicChatAttachment[];
  index: number;
  onClose: () => void;
  onNav: (delta: number) => void;
}) {
  createEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.key === "Escape") props.onClose();
      if (event.key === "ArrowLeft") props.onNav(-1);
      if (event.key === "ArrowRight") props.onNav(1);
    };
    window.addEventListener("keydown", handler);
    onCleanup(() => window.removeEventListener("keydown", handler));
  });

  const image = () => props.images[props.index];

  return (
    <Show when={image()}>
      <div
        onClick={props.onClose}
        style={{
          position: "fixed",
          inset: "0",
          background: "rgba(15,23,42,0.92)",
          "z-index": "1000",
          display: "flex",
          "align-items": "center",
          "justify-content": "center",
          padding: "40px",
          cursor: "zoom-out",
        }}
        data-testid="chat-lightbox"
      >
        <button
          type="button"
          onClick={(event) => {
            event.stopPropagation();
            props.onClose();
          }}
          aria-label="关闭"
          style={{
            position: "absolute",
            top: "20px",
            right: "20px",
            width: "40px",
            height: "40px",
            "border-radius": "20px",
            background: "rgba(255,255,255,0.10)",
            border: "none",
            cursor: "pointer",
            color: "#fff",
            "font-size": "20px",
            display: "flex",
            "align-items": "center",
            "justify-content": "center",
          }}
        >
          ✕
        </button>
        <img
          src={publicAttachmentUrl(image()!)}
          alt={image()!.name}
          onClick={(event) => event.stopPropagation()}
          style={{
            "max-width": "90%",
            "max-height": "90%",
            "object-fit": "contain",
            "border-radius": "8px",
            "box-shadow": "0 10px 40px rgba(0,0,0,0.6)",
          }}
        />
      </div>
    </Show>
  );
}

function Composer(props: {
  draft: string;
  onDraftChange: (v: string) => void;
  attachments: PublicChatAttachment[];
  onRemoveAttachment: (index: number) => void;
  onPickFiles: (files: File[], kind: "image" | "file") => void;
  uploadError: string;
  onDismissUploadError: () => void;
  uploading: boolean;
  onSend: () => void;
  onStop: () => void;
  isSending: boolean;
  remaining: number | undefined;
}) {
  const [focused, setFocused] = createSignal(false);
  const [menuOpen, setMenuOpen] = createSignal(false);
  let taRef: HTMLTextAreaElement | undefined;
  let imgInputRef: HTMLInputElement | undefined;
  let fileInputRef: HTMLInputElement | undefined;

  const canSend = () =>
    !props.isSending &&
    !props.uploading &&
    (!!props.draft.trim() || props.attachments.length > 0) &&
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

  const onPaste = (e: ClipboardEvent) => {
    const items = e.clipboardData?.items;
    if (!items || items.length === 0) return;
    const images: File[] = [];
    const others: File[] = [];
    for (let i = 0; i < items.length; i++) {
      const item = items[i]!;
      if (item.kind !== "file") continue;
      const file = item.getAsFile();
      if (!file) continue;
      // Pasted screenshots often come through with an empty or generic
      // filename; stamp a readable one so history/preview/upload are clear.
      const named =
        file.name && file.name !== "image.png"
          ? file
          : renamePasteFile(file);
      if (named.type.startsWith("image/")) images.push(named);
      else others.push(named);
    }
    if (images.length === 0 && others.length === 0) return;
    e.preventDefault();
    if (images.length > 0) props.onPickFiles(images, "image");
    if (others.length > 0) props.onPickFiles(others, "file");
  };

  return (
    <div
      style={{
        padding: "8px 20px 16px",
        "border-top": "1px solid rgba(0,0,0,0.08)",
        background: "#fff",
        "flex-shrink": "0",
        position: "relative",
      }}
    >
      <input
        ref={imgInputRef}
        type="file"
        accept="image/*"
        multiple
        style={{ display: "none" }}
        data-testid="composer-image-input"
        onChange={(event) => {
          const files = event.currentTarget.files
            ? Array.from(event.currentTarget.files)
            : [];
          event.currentTarget.value = "";
          if (files.length) props.onPickFiles(files, "image");
        }}
      />
      <input
        ref={fileInputRef}
        type="file"
        multiple
        style={{ display: "none" }}
        data-testid="composer-file-input"
        onChange={(event) => {
          const files = event.currentTarget.files
            ? Array.from(event.currentTarget.files)
            : [];
          event.currentTarget.value = "";
          if (files.length) props.onPickFiles(files, "file");
        }}
      />

      <AttachMenu
        open={menuOpen()}
        onClose={() => setMenuOpen(false)}
        onPickImage={() => imgInputRef?.click()}
        onPickFile={() => fileInputRef?.click()}
      />

      <Show when={props.uploadError}>
        <div
          style={{
            margin: "0 0 6px",
            padding: "8px 12px",
            "border-radius": "8px",
            border: "1px solid rgba(239,68,68,0.20)",
            background: "rgba(239,68,68,0.05)",
            "font-size": "12px",
            color: "#ef4444",
            display: "flex",
            "justify-content": "space-between",
            "align-items": "center",
            gap: "12px",
          }}
          data-testid="composer-upload-error"
        >
          <span>{props.uploadError}</span>
          <button
            type="button"
            onClick={props.onDismissUploadError}
            aria-label="关闭错误"
            style={{
              background: "none",
              border: "none",
              color: "#ef4444",
              cursor: "pointer",
              "font-size": "12px",
            }}
          >
            ✕
          </button>
        </div>
      </Show>

      <div
        style={{
          position: "relative",
          "border-radius": "20px",
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
        <AttachPreview
          items={props.attachments}
          onRemove={props.onRemoveAttachment}
        />

        <div
          style={{
            display: "flex",
            "align-items": "flex-end",
            gap: "4px",
            padding: "4px 6px 4px 4px",
          }}
        >
          <button
            type="button"
            class="pub-attach-btn"
            onClick={() => setMenuOpen(!menuOpen())}
            aria-label="附件"
            aria-haspopup="menu"
            aria-expanded={menuOpen()}
            data-open={menuOpen() ? "true" : "false"}
            data-testid="composer-attach-button"
          >
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.6"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M21.44 11.05l-9.19 9.19a6 6 0 11-8.49-8.49l9.19-9.19a4 4 0 115.66 5.66l-9.2 9.19a2 2 0 11-2.83-2.83l8.49-8.48" />
            </svg>
          </button>

          <textarea
            ref={taRef}
            rows={1}
            placeholder={
              props.remaining === 0 ? "今日额度已用完" : "消息..."
            }
            value={props.draft}
            disabled={props.isSending}
            onInput={(e) => props.onDraftChange(e.currentTarget.value)}
            onKeyDown={onKey}
            onPaste={onPaste}
            onFocus={() => setFocused(true)}
            onBlur={() => setFocused(false)}
            style={{
              flex: "1",
              resize: "none",
              border: "none",
              outline: "none",
              background: "transparent",
              padding: "10px 6px",
              "font-family":
                "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
              "font-size": "14px",
              "line-height": "1.55",
              color: "#0f172a",
              "max-height": "160px",
              "min-height": "38px",
            }}
          />

          <button
            type="button"
            onClick={() => canSend() && props.onSend()}
            disabled={!canSend()}
            aria-label="发送"
            data-testid="composer-send-button"
            style={{
              width: "38px",
              height: "38px",
              "border-radius": "10px",
              background: canSend() ? "#f59e0b" : "rgba(0,0,0,0.07)",
              border: "none",
              cursor: canSend() ? "pointer" : "default",
              display: "flex",
              "align-items": "center",
              "justify-content": "center",
              transition: "all 0.2s",
              "flex-shrink": "0",
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

      <div
        style={{
          display: "flex",
          "justify-content": "space-between",
          "margin-top": "6px",
          padding: "0 6px",
        }}
      >
        <span style={{ "font-size": "10px", color: "rgba(0,0,0,0.30)" }}>
          Shift + Enter 换行 · 拖拽或粘贴图片上传
        </span>
        <Show when={props.uploading}>
          <span style={{ "font-size": "10px", color: "#f59e0b" }}>
            正在上传附件…
          </span>
        </Show>
        <Show when={props.isSending}>
          <button
            type="button"
            onClick={props.onStop}
            style={{
              padding: "2px 8px",
              "border-radius": "5px",
              border: "1px solid rgba(0,0,0,0.10)",
              background: "#fff",
              "font-size": "10px",
              color: "#64748b",
              cursor: "pointer",
            }}
          >
            停止
          </button>
        </Show>
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
  const [pendingAttachments, setPendingAttachments] = createStore<
    PublicChatAttachment[]
  >([]);
  const [uploadError, setUploadError] = createSignal("");
  const [uploading, setUploading] = createSignal(false);
  const [lightbox, setLightbox] = createSignal<{
    images: PublicChatAttachment[];
    index: number;
  } | null>(null);
  const [dragActive, setDragActive] = createSignal(false);
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
    if ("attachments" in patch && patch.attachments !== current.attachments) {
      setMessages(index, "attachments", patch.attachments);
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
      // Transition to "ready" only after history is loaded so the UI
      // doesn't flash an empty chat view while messages are still loading.
      setAuthState("ready");
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

  // Called after a send finishes (success OR error OR abort). We intentionally
  // DO NOT re-fetch history here: the SSE stream already delivered the full
  // assistant content into local state, and history IDs (hash-based, derived
  // from index+role+content) don't match the UUIDs on the locally-appended
  // user/assistant pair. A reconcile(history) would blow away the bubbles the
  // user just saw — which is exactly the "占位符突然消失" bug that showed up
  // when the backend hadn't finished persisting the round yet.
  //
  // Only refresh the quota/session strip. If the backend pushes or schedules
  // a later message, ensurePushEvents() handles it. A hard refresh via page
  // reload still goes through restoreSession() which DOES re-pull history.
  const refreshSessionAfterSend = async () => {
    const generation = ++sessionSyncGeneration;
    try {
      const user = await getPublicAuthMe();
      if (generation !== sessionSyncGeneration) return;
      applySessionInfo(user);
      if (!eventSource) {
        await ensurePushEvents();
      }
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

  const revokePreviewUrl = (att: PublicChatAttachment) => {
    if (att.previewUrl) {
      try {
        URL.revokeObjectURL(att.previewUrl);
      } catch {
        /* noop */
      }
    }
  };

  const clearPendingAttachments = () => {
    for (const att of pendingAttachments) revokePreviewUrl(att);
    setPendingAttachments(reconcile([], { key: "path" }));
  };

  const handlePickFiles = async (files: File[], _kind: "image" | "file") => {
    if (!files.length) return;
    setUploadError("");
    const room = MAX_ATTACHMENTS - pendingAttachments.length;
    if (room <= 0) {
      setUploadError(`最多只能附加 ${MAX_ATTACHMENTS} 个文件`);
      return;
    }
    const accepted = files.slice(0, room);
    if (files.length > room) {
      setUploadError(`最多只能附加 ${MAX_ATTACHMENTS} 个文件，已截断多余项`);
    }

    const previewStart = pendingAttachments.length;
    const previews: PublicChatAttachment[] = accepted.map((file) => ({
      path: `__uploading__${crypto.randomUUID?.() ?? `${Date.now()}-${Math.random()}`}`,
      name: file.name,
      kind: classifyKind(file),
      size: file.size,
      previewUrl: file.type.startsWith("image/")
        ? URL.createObjectURL(file)
        : undefined,
    }));
    previews.forEach((preview, offset) => {
      setPendingAttachments(previewStart + offset, preview);
    });

    setUploading(true);
    try {
      const uploaded = await uploadPublicAttachments(accepted);
      uploaded.forEach((item, offset) => {
        const existing = pendingAttachments[previewStart + offset];
        setPendingAttachments(previewStart + offset, {
          path: item.path,
          name: item.name,
          kind: item.kind,
          size: item.size,
          previewUrl: existing?.previewUrl,
        });
      });
      if (uploaded.length < accepted.length) {
        // Trim trailing previews whose upload was rejected silently.
        for (let i = accepted.length - 1; i >= uploaded.length; i--) {
          const idx = previewStart + i;
          const stale = pendingAttachments[idx];
          if (stale) revokePreviewUrl(stale);
          setPendingAttachments(
            reconcile(
              [...pendingAttachments].filter((_, j) => j !== idx),
              { key: "path" },
            ),
          );
        }
      }
    } catch (error) {
      for (let i = accepted.length - 1; i >= 0; i--) {
        const idx = previewStart + i;
        const stale = pendingAttachments[idx];
        if (stale) revokePreviewUrl(stale);
      }
      setPendingAttachments(
        reconcile(
          pendingAttachments.slice(0, previewStart),
          { key: "path" },
        ),
      );
      setUploadError(
        error instanceof Error ? error.message : "上传失败，请重试",
      );
    } finally {
      setUploading(false);
    }
  };

  const handleRemoveAttachment = (index: number) => {
    const target = pendingAttachments[index];
    if (!target) return;
    revokePreviewUrl(target);
    setPendingAttachments(
      reconcile(
        pendingAttachments.filter((_, i) => i !== index),
        { key: "path" },
      ),
    );
  };

  const handleOpenLightbox = (
    images: PublicChatAttachment[],
    index: number,
  ) => {
    if (!images.length) return;
    setLightbox({ images, index });
  };

  const handleNavLightbox = (delta: number) => {
    const current = lightbox();
    if (!current) return;
    const next =
      (current.index + delta + current.images.length) % current.images.length;
    setLightbox({ images: current.images, index: next });
  };

  const handleDragEnter = (event: DragEvent) => {
    if (!event.dataTransfer?.types?.includes("Files")) return;
    event.preventDefault();
    setDragActive(true);
  };

  const handleDragOver = (event: DragEvent) => {
    if (!event.dataTransfer?.types?.includes("Files")) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = "copy";
  };

  const handleDragLeave = (event: DragEvent) => {
    if (event.currentTarget === event.target) {
      setDragActive(false);
    }
  };

  const handleDrop = (event: DragEvent) => {
    if (!event.dataTransfer?.files?.length) return;
    event.preventDefault();
    setDragActive(false);
    const files = Array.from(event.dataTransfer.files);
    const kind = files.every((f) => f.type.startsWith("image/"))
      ? "image"
      : "file";
    void handlePickFiles(files, kind);
  };

  const handleSend = async () => {
    const text = draft().trim();
    const atts = [...pendingAttachments];
    const hasReadyAtts = atts.every((att) => !att.path.startsWith("__uploading__"));
    if (
      (!text && atts.length === 0) ||
      authState() !== "ready" ||
      activeController ||
      uploading() ||
      !hasReadyAtts
    )
      return;
    const assistantId = messageId();
    setSendError("");
    setDraft("");
    setIsSending(true);
    appendMessages(
      {
        id: messageId(),
        role: "user",
        content: text,
        attachments: atts.map((a) => ({
          path: a.path,
          name: a.name,
          kind: a.kind,
          size: a.size,
          previewUrl: a.previewUrl,
        })),
      },
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
    clearPendingAttachments();
    scrollToBottom();

    const controller = new AbortController();
    activeController = controller;

    const attachmentInputs: PublicChatAttachmentInput[] = atts.map((a) => ({
      path: a.path,
      name: a.name,
    }));

    try {
      const stream = await sendPublicChat(
        text,
        attachmentInputs,
        controller.signal,
      );
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
            onDragEnter={handleDragEnter}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
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
                            <AssistantBubble
                              content={message.content}
                              attachments={message.attachments}
                            />
                          </Show>
                        }
                      >
                        <UserBubble
                          content={message.content}
                          attachments={message.attachments}
                          onOpenImage={handleOpenLightbox}
                        />
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
                attachments={[...pendingAttachments]}
                onRemoveAttachment={handleRemoveAttachment}
                onPickFiles={(files, kind) => void handlePickFiles(files, kind)}
                uploadError={uploadError()}
                onDismissUploadError={() => setUploadError("")}
                uploading={uploading()}
                onSend={() => void handleSend()}
                onStop={() => activeController?.abort()}
                isSending={isSending()}
                remaining={sessionInfo()?.remainingToday}
              />
            </div>

            <Show when={lightbox()}>
              <Lightbox
                images={lightbox()!.images}
                index={lightbox()!.index}
                onClose={() => setLightbox(null)}
                onNav={handleNavLightbox}
              />
            </Show>

            <Show when={dragActive()}>
              <div
                style={{
                  position: "absolute",
                  inset: "0",
                  "pointer-events": "none",
                  background: "rgba(245,158,11,0.08)",
                  border: "2px dashed rgba(245,158,11,0.45)",
                  "z-index": "40",
                  display: "flex",
                  "align-items": "center",
                  "justify-content": "center",
                  "font-family":
                    "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                  "font-size": "14px",
                  "font-weight": "600",
                  color: "#d97706",
                }}
                data-testid="chat-drop-overlay"
              >
                释放文件以上传
              </div>
            </Show>
          </div>
        </Match>
      </Switch>
    </>
  );
}
