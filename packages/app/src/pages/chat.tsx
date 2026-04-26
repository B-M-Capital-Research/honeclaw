// chat.tsx — Hone Public Site Chat (v4 - Styled to match Landing Page)

import { Logo } from "@hone-financial/ui/logo";
import { Markdown } from "@hone-financial/ui/markdown";
import {
  createMemo,
  createSignal,
  createEffect,
  createResource,
  For,
  Match,
  onCleanup,
  onMount,
  Show,
  Switch,
} from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import { useNavigate } from "@solidjs/router";
import { PasswordSetupGuard } from "@/components/password-setup-guard";
import { PublicLoginForm } from "@/components/public-login-form";
import { CONTENT } from "@/lib/public-content";
import { setLocale, useLocale } from "@/lib/i18n";
import "./public-site.css";
import {
  getPublicAuthMe,
  getPublicHistory,
  publicLogout,
  sendPublicChat,
  uploadPublicAttachments,
} from "@/lib/api";
import { buildApiUrl } from "@/lib/backend";
import { parseMessageContent, messageId } from "@/lib/messages";
import {
  nextVisibleMessageCount,
  selectVisibleRecentMessages,
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

// ── GitHub Star Fetching ─────────────────────────────────────────────────────
async function fetchGithubStars() {
  try {
    const res = await fetch("https://api.github.com/repos/B-M-Capital-Research/honeclaw")
    const data = await res.json()
    return data.stargazers_count || "..."
  } catch (e) {
    return "..."
  }
}

// ── Icons ────────────────────────────────────────────────────────────────────
const ICONS = {
  Chat: () => (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
  ),
  Github: () => (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
  ),
  Youtube: () => (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.016 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.016 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/></svg>
  ),
  Bilibili: () => (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M17.813 4.653h.854c1.51.054 2.769.578 3.773 1.574 1.004.995 1.524 2.249 1.56 3.76v7.36c-.036 1.51-.556 2.769-1.56 3.773s-2.262 1.524-3.773 1.56H5.333c-1.51-.036-2.769-.556-3.773-1.56S.036 18.883 0 17.373v-7.36c.036-1.51.556-2.765 1.56-3.76 1.004-.996 2.262-1.52 3.773-1.574h.774l-1.174-1.12a1.277 1.277 0 0 1-.388-.933c0-.346.138-.64.414-.88a1.277 1.277 0 0 1 .906-.36c.345 0 .647.127.906.38l2.227 2.12h4.72l2.227-2.12c.27-.253.57-.38.906-.38.365 0 .65.12.853.36.277.24.414.534.414.88 0 .346-.13.653-.387.92zm-12.48 5.387c-.331.03-.593.15-.786.36-.193.21-.29.473-.29.787v3.507c0 .313.097.576.29.786.193.21.455.33.786.36.331-.03.593-.15.786-.36.193-.21.29-.473.29-.786v-3.507c0-.314-.097-.577-.29-.787-.193-.21-.455-.33-.786-.36zm10.707 0c-.331.03-.593.15-.786.36-.193.21-.29.473-.29.787v3.507c0 .313.097.576.29.786.193.21.455.33.786.36.345-.03.607-.15.786-.36.193-.21.29-.473.29-.786v-3.507c0-.314-.097-.577-.29-.787-.193-.21-.455-.33-.786-.36zM18 19.04H6.013c-.113 0-.17.053-.17.16 0 .12.057.18.17.18H18c.113 0 .17-.06.17-.18 0-.107-.057-.16-.17-.16z"/></svg>
  )
}

const PUBLIC_IMAGE_ENDPOINT = "/api/public/image";
const MAX_ATTACHMENTS = 4;
const HISTORY_PAGE_SIZE = 24;

function AnimatedBackground() {
  return (
    <div class="animated-bg">
      <div class="circle circle-1"></div>
      <div class="circle circle-2"></div>
      <div class="circle circle-3"></div>
    </div>
  )
}

function Header() {
  const navigate = useNavigate()
  const [stars] = createResource(fetchGithubStars)
  const C = CONTENT.nav

  return (
    <header class="page-header">
      <div onClick={() => navigate("/")} class="header-logo">
        <img src="/logo.svg" alt="Hone" />
        <span>Hone</span>
      </div>

      <div class="header-actions">
        <div class="header-socials mobile-hide">
          <a href="https://www.youtube.com/@HoneFinancial" target="_blank" class="icon-btn-ghost"><ICONS.Youtube /></a>
          <a href="https://www.bilibili.com/video/BV1ByXNBGET5/" target="_blank" class="icon-btn-ghost"><ICONS.Bilibili /></a>
          <a href="https://github.com/B-M-Capital-Research/honeclaw" target="_blank" class="star-badge">
            <ICONS.Github />
            <span>{stars() || "..."}</span>
          </a>
        </div>

        <div class="divider-v mobile-hide" />

        <div class="lang-switch">
          <button onClick={() => setLocale("zh")} class={useLocale() === "zh" ? "active" : ""}>中</button>
          <button onClick={() => setLocale("en")} class={useLocale() === "en" ? "active" : ""}>EN</button>
        </div>

        <div style={{ display: "flex", gap: "10px" }}>
          <button onClick={() => navigate("/roadmap")} class="btn-roadmap-nav mobile-hide">
            {useLocale() === 'zh' ? '产品路线图' : 'Roadmap'}
          </button>
          <button onClick={() => navigate("/chat")} class="btn-chat-nav">{C.chat}</button>
        </div>
      </div>
    </header>
  )
}

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

function LoadingCard() {
  return (
    <div
      style={{
        "min-height": "100vh",
        display: "flex",
        "align-items": "center",
        "justify-content": "center",
      }}
    >
      <div
        style={{
          "max-width": "480px",
          width: "100%",
          padding: "0 24px",
          "text-align": "center",
          position: "relative",
          "z-index": "10",
        }}
      >
        <div
          style={{
            padding: "48px 32px",
            "border-radius": "24px",
            border: "1.5px solid #f1f5f9",
            background: "rgba(255, 255, 255, 0.9)",
            "backdrop-filter": "blur(10px)",
            "box-shadow": "0 20px 50px rgba(0,0,0,0.05)",
          }}
        >
          <div
            style={{
              width: "56px",
              height: "56px",
              "border-radius": "50%",
              background: "#000",
              display: "flex",
              "align-items": "center",
              "justify-content": "center",
              margin: "0 auto 24px",
            }}
          >
            <div class="h-6 w-6 animate-spin rounded-full border-2 border-white/20 border-t-white" />
          </div>
          <h1 style={{ "font-size": "22px", "font-weight": "800", color: "#0f172a", margin: "0 0 12px" }}>
            正在恢复对话
          </h1>
          <p style={{ "font-size": "15px", color: "#64748b", margin: "0", "line-height": "1.6" }}>
            正在校验当前会话并恢复聊天历史
          </p>
        </div>
      </div>
    </div>
  );
}

function assistantMarkdownClass(extra: string = "") {
  return [
    "break-words text-[16px] leading-[1.8] text-[#1e293b]",
    "[&_*]:max-w-full",
    "[&_p]:my-0 [&_p+*]:mt-3",
    "[&_strong]:text-[#0f172a] [&_strong]:font-bold",
    "[&_pre]:mt-4 [&_pre]:max-w-full [&_pre]:overflow-x-auto [&_pre]:rounded-2xl [&_pre]:border-0 [&_pre]:shadow-none",
    "[&_code]:rounded-lg [&_code]:bg-black/[0.05] [&_code]:px-2 [&_code]:py-1 [&_code]:text-[14px] [&_code]:font-[var(--font-mono,'JetBrains_Mono',monospace)]",
    "[&_ul]:my-3 [&_ol]:my-3 [&_li]:my-1",
    "[&_blockquote]:my-4 [&_blockquote]:border-l-4 [&_blockquote]:border-black/5 [&_blockquote]:pl-4 [&_blockquote]:text-[#64748b] [&_blockquote]:italic",
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
                class="hone-assistant-image mt-3 max-w-full cursor-zoom-in rounded-xl shadow-sm"
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
            "grid-template-columns": `repeat(2, 1fr)`,
            gap: "4px",
            "border-radius": "16px",
            overflow: "hidden",
            "max-width": "420px",
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
          "border-radius": "16px",
          overflow: "hidden",
          cursor: "zoom-in",
          "max-width": "420px",
          "line-height": "0",
          position: "relative",
        }}
      >
        <img
          src={publicAttachmentUrl(props.images[0]!)}
          alt={props.images[0]!.name}
          style={{ width: "100%", height: "auto", display: "block" }}
        />
      </div>
    </Show>
  );
}

function FileCard(props: { file: PublicChatAttachment; inUserBubble?: boolean }) {
  const ext = () => fileExtension(props.file.name);
  const iconBg = () =>
    props.inUserBubble ? "rgba(255,255,255,0.2)" : "rgba(0,0,0,0.05)";
  const iconColor = () => (props.inUserBubble ? "#fff" : "#1e293b");
  const textColor = () =>
    props.inUserBubble ? "rgba(255,255,255,0.95)" : "#0f172a";
  const subColor = () =>
    props.inUserBubble ? "rgba(255,255,255,0.7)" : "#64748b";
  return (
    <div
      style={{
        display: "flex",
        "align-items": "center",
        gap: "14px",
        padding: "12px 14px",
        background: props.inUserBubble
          ? "rgba(255,255,255,0.12)"
          : "#fff",
        border: props.inUserBubble
          ? "1.5px solid rgba(255,255,255,0.2)"
          : "1.5px solid #f1f5f9",
        "border-radius": "16px",
        "min-width": "260px",
      }}
    >
      <div
        style={{
          width: "44px",
          height: "44px",
          "border-radius": "10px",
          background: iconBg(),
          display: "flex",
          "align-items": "center",
          "justify-content": "center",
          "font-family": "var(--font-mono, 'JetBrains Mono', monospace)",
          "font-size": "11px",
          "font-weight": "800",
          color: iconColor(),
          "letter-spacing": "0.05em",
          "flex-shrink": "0",
        }}
      >
        {ext()}
      </div>
      <div style={{ flex: "1", "min-width": "0" }}>
        <div
          style={{
            "font-size": "15px",
            "font-weight": "700",
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
              "font-size": "12px",
              color: subColor(),
              "margin-top": "3px",
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
        "margin-bottom": "20px",
      }}
    >
      <div
        style={{
          "max-width": "80%",
          background: "#000",
          color: "#fff",
          "border-radius": "24px 24px 4px 24px",
          padding: imageOnly() ? "6px" : "14px 20px",
          "font-size": "16px",
          "line-height": "1.7",
          "box-shadow": "0 10px 30px rgba(0,0,0,0.1)",
          "white-space": "pre-wrap",
          "word-break": "break-word",
        }}
      >
        <Show when={images().length > 0}>
          <div style={{ "margin-bottom": hasText() || files().length > 0 ? "10px" : "0" }}>
            <ImageMosaic
              images={images()}
              inUserBubble
              onOpen={(index) => props.onOpenImage(images(), index)}
            />
          </div>
        </Show>
        <Show when={files().length > 0}>
          <div style={{ display: "flex", "flex-direction": "column", gap: "8px", "margin-bottom": hasText() ? "10px" : "0" }}>
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
        "margin-bottom": "20px",
      }}
    >
      <div
        style={{
          "max-width": "85%",
          background: "rgba(255, 255, 255, 0.9)",
          "backdrop-filter": "blur(10px)",
          border: "1.5px solid #f1f5f9",
          "border-radius": "4px 24px 24px 24px",
          padding: "16px 20px",
          color: "#1e293b",
          "box-shadow": "0 4px 20px rgba(0,0,0,0.02)",
        }}
      >
        <div style={{ display: "flex", "align-items": "center", gap: "8px", "margin-bottom": "12px" }}>
          <span style={{ width: "8px", height: "8px", "border-radius": "50%", background: "#f59e0b", display: "inline-block" }} />
          <span style={{ "font-size": "13px", "font-weight": "800", "letter-spacing": "0.1em", "text-transform": "uppercase", color: "#64748b" }}>
            HONE
          </span>
        </div>
        <AssistantBody content={props.content} />
        <Show when={nonImageAttachments().length > 0}>
          <div style={{ display: "flex", "flex-direction": "column", gap: "8px", "margin-top": "16px" }}>
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
      case "error": return "HONE 出错了";
      case "streaming": return "HONE 输出中";
      case "running": return "HONE 执行中";
      default: return "HONE 思考中";
    }
  };
  const showDots = () => !props.message.content && !terminal();

  return (
    <div class="pub-msg-in" style={{ display: "flex", "justify-content": "flex-start", "margin-bottom": "20px" }}>
      <div
        style={{
          "max-width": "85%",
          "min-width": "240px",
          background: "#fff",
          border: terminal() ? "2px solid rgba(239,68,68,0.2)" : "1.5px solid #f1f5f9",
          "border-radius": "4px 24px 24px 24px",
          padding: "16px 20px",
          "box-shadow": "0 10px 30px rgba(0,0,0,0.03)",
        }}
      >
        <div style={{ display: "flex", "align-items": "center", "justify-content": "space-between", gap: "10px", "margin-bottom": props.message.content ? "12px" : "0" }}>
          <div style={{ display: "flex", "align-items": "center", gap: "8px" }}>
            <span class={props.message.phase === "thinking" || props.message.phase === "streaming" ? "pub-pulsedot" : ""}
              style={{ width: "8px", height: "8px", "border-radius": "50%", background: terminal() ? "#ef4444" : "#f59e0b" }} />
            <span style={{ "font-size": "13px", "font-weight": "800", "letter-spacing": "0.1em", "text-transform": "uppercase", color: "#64748b" }}>
              {labelText()}
            </span>
            <span style={{ "font-family": "var(--font-mono)", "font-size": "12px", color: "rgba(0,0,0,0.2)" }}>
              {elapsed()}s
            </span>
          </div>
          <Show when={!terminal()} fallback={<button onClick={props.onDismiss} style={{ background: "none", border: "none", cursor: "pointer", color: "#64748b", "font-size": "14px" }}>✕</button>}>
            <button onClick={props.onStop} class="btn-stop-thinking">停止</button>
          </Show>
        </div>

        <Show when={showDots()}>
          <div style={{ display: "flex", gap: "6px", padding: "8px 0" }}>
            <span class="pub-dot1" /><span class="pub-dot2" /><span class="pub-dot3" />
          </div>
        </Show>

        <Show when={(props.message.steps?.length ?? 0) > 0}>
          <ul style={{ margin: props.message.content ? "0 0 12px" : "8px 0 0", padding: "0", "list-style": "none", "font-size": "13px", "line-height": "1.8", color: "#64748b" }}>
            <For each={props.message.steps}>
              {(step) => (
                <li style={{ display: "flex", "align-items": "flex-start", gap: "8px" }}>
                  <span style={{ color: "#f59e0b" }}>•</span><span>{step}</span>
                </li>
              )}
            </For>
          </ul>
        </Show>

        <Show when={props.message.content}>
          <div style={{ "white-space": "pre-wrap" }}>
            <AssistantBody content={props.message.content} />
            <Show when={props.message.phase === "streaming"}><span class="pub-cursor" /></Show>
          </div>
        </Show>

        <Show when={terminal()}>
          <div style={{ "font-size": "14px", color: "#ef4444", "margin-top": "6px", "font-weight": "600" }}>
            {props.message.statusText || "请求出错，请重试。"}
          </div>
        </Show>
      </div>
    </div>
  );
}

function AttachPreview(props: { items: PublicChatAttachment[]; onRemove: (index: number) => void; }) {
  return (
    <Show when={props.items.length > 0}>
      <div style={{ display: "flex", gap: "10px", padding: "12px 16px", "flex-wrap": "wrap", "border-bottom": "1.5px solid #f8fafc" }}>
        <For each={props.items}>
          {(item, index) => (
            <div style={{ position: "relative" }}>
              <Show when={item.kind === "image"} fallback={
                <div style={{ width: "200px", height: "72px", padding: "0 12px", display: "flex", "align-items": "center", gap: "12px", "border-radius": "12px", border: "1.5px solid #f1f5f9", background: "#fcfdfe" }}>
                  <div style={{ width: "40px", height: "40px", "border-radius": "8px", background: "rgba(245,158,11,0.1)", display: "flex", "align-items": "center", "justify-content": "center", "font-family": "var(--font-mono)", "font-size": "11px", "font-weight": "800", color: "#d97706" }}>{fileExtension(item.name)}</div>
                  <div style={{ flex: "1", "min-width": "0" }}>
                    <div style={{ "font-size": "13px", "font-weight": "700", color: "#0f172a", overflow: "hidden", "text-overflow": "ellipsis", "white-space": "nowrap" }}>{item.name}</div>
                    <div style={{ "font-family": "var(--font-mono)", "font-size": "11px", color: "#94a3b8" }}>{formatBytes(item.size)}</div>
                  </div>
                </div>
              }>
                <div style={{ width: "72px", height: "72px", "border-radius": "12px", overflow: "hidden", border: "1.5px solid #f1f5f9" }}>
                  <img src={publicAttachmentUrl(item)} alt={item.name} style={{ width: "100%", height: "100%", "object-fit": "cover" }} />
                </div>
              </Show>
              <button onClick={() => props.onRemove(index())} style={{ position: "absolute", top: "-8px", right: "-8px", width: "24px", height: "24px", "border-radius": "12px", background: "#000", color: "#fff", border: "2.5px solid #fff", cursor: "pointer", "font-size": "12px", display: "flex", "align-items": "center", "justify-content": "center", "box-shadow": "0 4px 10px rgba(0,0,0,0.2)" }}>✕</button>
            </div>
          )}
        </For>
      </div>
    </Show>
  );
}

function AttachMenu(props: { open: boolean; onClose: () => void; onPickImage: () => void; onPickFile: () => void; }) {
  return (
    <Show when={props.open}>
      <div class="pub-attach-backdrop" onClick={props.onClose} />
      <div class="pub-attach-menu" style={{ "border-radius": "20px", padding: "8px", "min-width": "240px", bottom: "80px", "box-shadow": "0 20px 50px rgba(0,0,0,0.15)" }}>
        <button type="button" class="pub-attach-item" onClick={() => { props.onPickImage(); props.onClose(); }}>
          <span class="pub-attach-icon" style={{ "background": "#f1f5f9" }}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="5" width="18" height="14" rx="2.5" /><circle cx="8.5" cy="10" r="1.5" /><path d="M21 15l-5-5-8 9" /></svg>
          </span>
          <span class="pub-attach-label"><span class="pub-attach-label-title" style={{ "font-size": "15px" }}>图片</span><span class="pub-attach-label-sub">照片与截图</span></span>
        </button>
        <button type="button" class="pub-attach-item" onClick={() => { props.onPickFile(); props.onClose(); }}>
          <span class="pub-attach-icon" style={{ "background": "#f1f5f9" }}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 3H7a2 2 0 00-2 2v14a2 2 0 002 2h10a2 2 0 002-2V8z" /><path d="M14 3v5h5" /><path d="M9 13h6M9 17h4" /></svg>
          </span>
          <span class="pub-attach-label"><span class="pub-attach-label-title" style={{ "font-size": "15px" }}>文件</span><span class="pub-attach-label-sub">PDF · 文档 · 其他</span></span>
        </button>
      </div>
    </Show>
  );
}

function ComposerStatus(props: { message: ChatMessage | undefined; onStop: () => void; justFinished: boolean }) {
  const [elapsed, setElapsed] = createSignal(0);

  createEffect(() => {
    const m = props.message;
    if (!m || !m.startedAt) {
      setElapsed(0);
      return;
    }
    const tick = () => setElapsed(Math.max(0, Math.floor((Date.now() - (m.startedAt ?? 0)) / 1000)));
    tick();
    const timer = setInterval(tick, 1000);
    onCleanup(() => clearInterval(timer));
  });

  const labelText = (m: ChatMessage) => {
    switch (m.phase) {
      case "streaming": return "HONE 输出中";
      case "running": return "HONE 执行中";
      default: return "HONE 思考中";
    }
  };

  return (
    <Show when={props.message || props.justFinished}>
      <div
        class={"public-chat-composer-status" + (props.justFinished ? " is-done" : "")}
        role="status"
        aria-live="polite"
      >
        <Show when={props.message} fallback={
          <>
            <span class="public-chat-composer-status-dot done" />
            <span class="public-chat-composer-status-label">本轮已完成</span>
          </>
        }>
          {(m) => (
            <>
              <span class="public-chat-composer-status-dot pulsing" />
              <span class="public-chat-composer-status-label">{labelText(m())}</span>
              <span class="public-chat-composer-status-time">{elapsed()}s</span>
              <button type="button" class="public-chat-composer-status-stop" onClick={props.onStop}>停止</button>
            </>
          )}
        </Show>
      </div>
    </Show>
  );
}

function Composer(props: {
  draft: string; onDraftChange: (v: string) => void;
  attachments: PublicChatAttachment[]; onRemoveAttachment: (index: number) => void;
  onPickFiles: (files: File[], kind: "image" | "file") => void;
  uploadError: string; onDismissUploadError: () => void;
  uploading: boolean; onSend: () => void; onStop: () => void;
  isSending: boolean; remaining: number | undefined;
  pendingMessage: ChatMessage | undefined; justFinished: boolean;
}) {
  const [focused, setFocused] = createSignal(false);
  const [menuOpen, setMenuOpen] = createSignal(false);
  let taRef: HTMLTextAreaElement | undefined;
  let imgInputRef: HTMLInputElement | undefined;
  let fileInputRef: HTMLInputElement | undefined;

  const canSend = () => !props.isSending && !props.uploading && (!!props.draft.trim() || props.attachments.length > 0) && (props.remaining === undefined || props.remaining > 0);

  createEffect(() => { if (!props.isSending && taRef) taRef.focus(); });

  return (
    <div class="public-chat-composer" style={{ padding: "16px 24px 32px", background: "transparent", "flex-shrink": "0", position: "relative", "z-index": "20" }}>
      <ComposerStatus message={props.pendingMessage} onStop={props.onStop} justFinished={props.justFinished} />
      <input ref={imgInputRef} type="file" accept="image/*" multiple style={{ display: "none" }} onChange={(e) => { const files = e.currentTarget.files ? Array.from(e.currentTarget.files) : []; e.currentTarget.value = ""; if (files.length) props.onPickFiles(files, "image"); }} />
      <input ref={fileInputRef} type="file" multiple style={{ display: "none" }} onChange={(e) => { const files = e.currentTarget.files ? Array.from(e.currentTarget.files) : []; e.currentTarget.value = ""; if (files.length) props.onPickFiles(files, "file"); }} />

      <AttachMenu open={menuOpen()} onClose={() => setMenuOpen(false)} onPickImage={() => imgInputRef?.click()} onPickFile={() => fileInputRef?.click()} />

      <div class="public-chat-composer-box" style={{ position: "relative", "max-width": "900px", margin: "0 auto", "border-radius": "28px", border: focused() ? "2px solid #000" : "2px solid #f1f5f9", background: "#fff", "box-shadow": focused() ? "0 20px 60px rgba(0,0,0,0.08)" : "0 10px 30px rgba(0,0,0,0.03)", transition: "all 0.3s cubic-bezier(0.16, 1, 0.3, 1)", overflow: "hidden" }}>
        <AttachPreview items={props.attachments} onRemove={props.onRemoveAttachment} />
        <div style={{ display: "flex", "align-items": "flex-end", gap: "8px", padding: "8px 12px" }}>
          <button type="button" class="pub-attach-btn" style={{ width: "48px", height: "48px" }} onClick={() => setMenuOpen(!menuOpen())}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.44 11.05l-9.19 9.19a6 6 0 11-8.49-8.49l9.19-9.19a4 4 0 115.66 5.66l-9.2 9.19a2 2 0 11-2.83-2.83l8.49-8.48" /></svg>
          </button>
          <textarea ref={taRef} class="public-chat-composer-input" rows={1} placeholder={props.remaining === 0 ? "今日额度已用完" : "输入问题，开始投研纪律对话..."} value={props.draft} disabled={props.isSending} onInput={(e) => props.onDraftChange(e.currentTarget.value)}
            onKeyDown={(e) => { if (!e.isComposing && e.key === "Enter" && !e.shiftKey) { e.preventDefault(); if (canSend()) props.onSend(); } }}
            onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}
            style={{ flex: "1", resize: "none", border: "none", outline: "none", background: "transparent", padding: "12px 8px", "font-size": "16px", "font-weight": "600", "line-height": "1.6", color: "#0f172a", "max-height": "200px", "min-height": "48px" }} />
          <button type="button" class="public-chat-send-button" onClick={() => canSend() && props.onSend()} disabled={!canSend()} style={{ width: "48px", height: "48px", "border-radius": "16px", background: canSend() ? "#000" : "#f1f5f9", border: "none", cursor: canSend() ? "pointer" : "default", display: "flex", "align-items": "center", "justify-content": "center", transition: "all 0.2s" }}>
            <svg viewBox="0 0 20 20" width="20" height="20" fill={canSend() ? "white" : "#94a3b8"}><path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" /></svg>
          </button>
        </div>
      </div>
    </div>
  );
}

export default function PublicChatPage() {
  const navigate = useNavigate();
  const [authState, setAuthState] = createSignal<AuthState>("loading");
  const [currentUser, setCurrentUser] = createSignal<PublicAuthUserInfo | null>(null);
  const [messages, setMessages] = createStore<ChatMessage[]>([]);
  const [draft, setDraft] = createSignal("");
  const [isSending, setIsSending] = createSignal(false);
  const [pendingAttachments, setPendingAttachments] = createStore<PublicChatAttachment[]>([]);
  const [uploadError, setUploadError] = createSignal("");
  const [uploading, setUploading] = createSignal(false);
  const [lightbox, setLightbox] = createSignal<{ images: PublicChatAttachment[]; index: number; } | null>(null);
  const [sessionInfo, setSessionInfo] = createSignal<{ userId: string; remainingToday: number; dailyLimit: number; } | null>(null);
  const [visibleMessageCount, setVisibleMessageCount] = createSignal(HISTORY_PAGE_SIZE);
  const [loadingOlderMessages, setLoadingOlderMessages] = createSignal(false);
  const [justFinished, setJustFinished] = createSignal(false);
  let activeController: AbortController | null = null;
  let scrollRef: HTMLDivElement | undefined;
  let messagesInnerRef: HTMLDivElement | undefined;
  let sessionSyncGeneration = 0;
  let stickToBottom = true;
  let lastScrollTop = 0;
  let suppressScrollDetect = 0;
  let justFinishedTimer: number | undefined;

  const scrollToBottom = () => {
    requestAnimationFrame(() => {
      if (!scrollRef) return;
      suppressScrollDetect = scrollRef.scrollHeight;
      scrollRef.scrollTop = scrollRef.scrollHeight;
      lastScrollTop = scrollRef.scrollTop;
    });
  };
  const distanceFromBottom = () => scrollRef ? scrollRef.scrollHeight - scrollRef.scrollTop - scrollRef.clientHeight : 0;
  const visibleMessages = createMemo(() => selectVisibleRecentMessages(messages, visibleMessageCount()));
  const hasOlderMessages = () => visibleMessageCount() < messages.length;
  const pendingAssistantMessage = createMemo(() => {
    for (let i = messages.length - 1; i >= 0; i--) {
      const m = messages[i];
      if (m.role === "assistant" && m.phase && m.phase !== "done" && m.phase !== "error") return m;
    }
    return undefined;
  });

  const loadOlderMessages = () => {
    if (!scrollRef || !hasOlderMessages() || loadingOlderMessages()) return;
    const previousScrollHeight = scrollRef.scrollHeight;
    const previousScrollTop = scrollRef.scrollTop;
    setLoadingOlderMessages(true);
    setVisibleMessageCount((current) => nextVisibleMessageCount(messages.length, current, HISTORY_PAGE_SIZE));
    requestAnimationFrame(() => {
      if (scrollRef) {
        scrollRef.scrollTop = previousScrollTop + (scrollRef.scrollHeight - previousScrollHeight);
        lastScrollTop = scrollRef.scrollTop;
      }
      setLoadingOlderMessages(false);
    });
  };

  const handleMessagesScroll = () => {
    if (!scrollRef) return;
    const top = scrollRef.scrollTop;
    const dist = distanceFromBottom();
    // Skip the synthetic scroll event we just produced via scrollToBottom.
    if (suppressScrollDetect && Math.abs(scrollRef.scrollHeight - suppressScrollDetect) < 2 && dist < 4) {
      lastScrollTop = top;
      suppressScrollDetect = 0;
      return;
    }
    suppressScrollDetect = 0;
    if (top < lastScrollTop - 2) {
      // user-initiated scroll up
      stickToBottom = dist < 80;
    } else if (dist < 80) {
      stickToBottom = true;
    }
    lastScrollTop = top;
    if (top <= 24) loadOlderMessages();
  };

  const flashJustFinished = () => {
    setJustFinished(true);
    if (justFinishedTimer !== undefined) window.clearTimeout(justFinishedTimer);
    justFinishedTimer = window.setTimeout(() => setJustFinished(false), 2400);
  };

  // Keep the composer pinned to the visible viewport when the user pinch-zooms
  // or the soft keyboard opens, so it remains reachable without zooming back out.
  createEffect(() => {
    const vv = typeof window !== "undefined" ? window.visualViewport : undefined;
    if (!vv) return;
    const update = () => {
      const delta = (vv.offsetTop + vv.height) - window.innerHeight;
      document.documentElement.style.setProperty("--composer-vv-shift", `${Math.min(0, delta)}px`);
    };
    update();
    vv.addEventListener("scroll", update);
    vv.addEventListener("resize", update);
    onCleanup(() => {
      vv.removeEventListener("scroll", update);
      vv.removeEventListener("resize", update);
      document.documentElement.style.removeProperty("--composer-vv-shift");
    });
  });

  // When the inner messages content grows (streaming, new message), keep the
  // viewport glued to the bottom unless the user has explicitly scrolled away.
  createEffect(() => {
    if (!messagesInnerRef || typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver(() => { if (stickToBottom) scrollToBottom(); });
    ro.observe(messagesInnerRef);
    onCleanup(() => ro.disconnect());
  });

  const applyPublicUser = (user: PublicAuthUserInfo) => {
    setSessionInfo({ userId: user.user_id, remainingToday: user.remaining_today, dailyLimit: user.daily_limit });
    setCurrentUser(user);
    setAuthState(user.has_password ? "ready" : "needs_password");
  };

  const restoreSession = async (options: { resetWindow?: boolean } = {}) => {
    const generation = ++sessionSyncGeneration;
    try {
      const user = await getPublicAuthMe();
      if (generation !== sessionSyncGeneration) return;
      applyPublicUser(user);
      const history = await getPublicHistory();
      if (generation !== sessionSyncGeneration) return;
      const next = toPublicChatMessages(history);
      if (options.resetWindow) {
        setVisibleMessageCount(HISTORY_PAGE_SIZE);
      } else {
        // Preserve user's current viewing window; never shrink it on a sync.
        setVisibleMessageCount((c) => Math.max(c, Math.min(next.length, HISTORY_PAGE_SIZE)));
      }
      setMessages(reconcile(next, { key: "id" }));
      if (options.resetWindow) scrollToBottom();
    } catch {
      setAuthState("logged_out");
    }
  };

  onMount(() => {
    document.documentElement.classList.add("public-chat-scroll-lock");
    document.body.classList.add("public-chat-scroll-lock");
    void restoreSession({ resetWindow: true });
  });
  onCleanup(() => {
    activeController?.abort();
    if (justFinishedTimer !== undefined) window.clearTimeout(justFinishedTimer);
    document.documentElement.classList.remove("public-chat-scroll-lock");
    document.body.classList.remove("public-chat-scroll-lock");
  });

  const handleSend = async () => {
    const text = draft().trim();
    const atts = [...pendingAttachments];
    if ((!text && atts.length === 0) || authState() !== "ready" || isSending() || uploading()) return;
    
    const assistantId = messageId();
    setDraft("");
    setIsSending(true);
    // Send action is an explicit user intent to follow the new content.
    stickToBottom = true;
    setMessages(messages.length, { id: messageId(), role: "user", content: text, attachments: atts });
    setMessages(messages.length, { id: assistantId, role: "assistant", content: "", phase: "thinking", statusText: "Hone 思考中", startedAt: Date.now(), steps: [] });
    // Keep all existing + new messages in view; never shrink the visible window.
    setVisibleMessageCount((c) => Math.max(c + 2, HISTORY_PAGE_SIZE));
    setPendingAttachments(reconcile([], { key: "path" }));
    scrollToBottom();

    const controller = new AbortController();
    activeController = controller;
    try {
      const stream = await sendPublicChat(text, atts.map(a => ({ path: a.path, name: a.name })), controller.signal);
      const reader = stream.getReader();
      const decoder = new TextDecoder();
      let pendingSse = "";
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        pendingSse += decoder.decode(value, { stream: true });
        const parsed = parseSseChunks(pendingSse);
        pendingSse = parsed.pending;
        for (const ev of parsed.events) {
          if (ev.event === "assistant_delta") {
            const index = messages.findIndex(m => m.id === assistantId);
            if (index >= 0) {
              setMessages(index, { content: messages[index].content + (ev.data.content ?? ""), phase: "streaming" });
            }
            if (stickToBottom) scrollToBottom();
          }
          if (ev.event === "run_finished") {
            const index = messages.findIndex(m => m.id === assistantId);
            if (index >= 0) setMessages(index, "phase", "done");
            flashJustFinished();
          }
        }
      }
    } catch (e) {
      const index = messages.findIndex(m => m.id === assistantId);
      if (index >= 0) setMessages(index, { phase: "error", statusText: String(e) });
    } finally {
      setIsSending(false);
      flashJustFinished();
      void restoreSession();
    }
  };

  return (
    <div class="hone-landing-v4 public-chat-page" style={{ height: "100vh", display: "flex", "flex-direction": "column" }}>
      <AnimatedBackground />
      <Header />

      <Switch>
        <Match when={authState() === "loading"}>
          <LoadingCard />
        </Match>
        <Match when={authState() === "logged_out"}>
          <PublicLoginForm onLogin={() => restoreSession({ resetWindow: true })} />
        </Match>
        <Match when={authState() === "ready" || authState() === "needs_password"}>
          <Show when={currentUser()} fallback={<LoadingCard />}>
            {(user) => (
              <PasswordSetupGuard user={user()} onPasswordSet={applyPublicUser}>
                <div class="public-chat-shell" style={{ flex: "1", display: "flex", "flex-direction": "column", "padding-top": "80px", position: "relative", "z-index": "10", overflow: "hidden" }}>
            
            {/* Session Strip */}
            <div class="public-chat-session-strip" style={{ display: "flex", "justify-content": "center", padding: "12px" }}>
               <div style={{ background: "rgba(255,255,255,0.7)", "backdrop-filter": "blur(10px)", padding: "6px 20px", "border-radius": "100px", border: "1.5px solid #f1f5f9", display: "flex", gap: "20px", "align-items": "center", "font-size": "13px", "font-weight": "700" }}>
                 <span style={{ color: "#64748b" }}>{sessionInfo()?.dailyLimit ? `今日剩余 ${sessionInfo()?.remainingToday}/${sessionInfo()?.dailyLimit}` : "无限额度"}</span>
                 <div style={{ width: "1px", height: "12px", background: "#e2e8f0" }} />
                 <button onClick={() => navigate("/me")} style={{ border: "none", background: "none", cursor: "pointer", color: "#000", "font-weight": "800" }}>{sessionInfo()?.userId}</button>
                 <button onClick={() => { publicLogout(); setAuthState("logged_out"); }} style={{ border: "none", background: "none", cursor: "pointer", color: "#ef4444" }}>退出</button>
               </div>
            </div>

            {/* Message List */}
            <div ref={scrollRef} class="public-chat-messages" onScroll={handleMessagesScroll} style={{ flex: "1", "overflow-y": "auto", padding: "20px 0" }}>
              <div ref={messagesInnerRef} style={{ "max-width": "900px", margin: "0 auto", padding: "0 24px" }}>
                <Show when={hasOlderMessages()}>
                  <div style={{ "text-align": "center", color: "#94a3b8", "font-size": "12px", "font-weight": "700", padding: "4px 0 18px" }}>
                    {loadingOlderMessages() ? "加载中..." : "上滑加载更早消息"}
                  </div>
                </Show>
                <For each={visibleMessages()}>
                  {(msg) => (
                    <Switch>
                      <Match when={msg.role === "user"}>
                        <UserBubble content={msg.content} attachments={msg.attachments} onOpenImage={(imgs, i) => setLightbox({ images: imgs, index: i })} />
                      </Match>
                      <Match when={msg.role === "assistant" && msg.phase === "done"}>
                        <AssistantBubble content={msg.content} attachments={msg.attachments} />
                      </Match>
                      <Match when={msg.role === "assistant" && msg.phase !== "done"}>
                        <PendingBubble message={msg} onStop={() => activeController?.abort()} onDismiss={() => {}} />
                      </Match>
                    </Switch>
                  )}
                </For>
              </div>
            </div>

                  <Composer
                    draft={draft()} onDraftChange={setDraft}
                    attachments={pendingAttachments} onRemoveAttachment={(i) => setPendingAttachments(pendingAttachments.filter((_, j) => j !== i))}
                    onPickFiles={async (files) => {
                      setUploading(true);
                      try {
                        const uploaded = await uploadPublicAttachments(files);
                        setPendingAttachments([...pendingAttachments, ...uploaded.map(u => ({ ...u, kind: u.kind as any }))]);
                      } finally { setUploading(false); }
                    }}
                    uploadError={uploadError()} onDismissUploadError={() => setUploadError("")}
                    uploading={uploading()} onSend={handleSend} onStop={() => activeController?.abort()}
                    isSending={isSending()} remaining={sessionInfo()?.remainingToday}
                    pendingMessage={pendingAssistantMessage()} justFinished={justFinished()}
                  />
                </div>
              </PasswordSetupGuard>
            )}
          </Show>
        </Match>
      </Switch>

      <Show when={lightbox()}>
        <div class="lightbox-overlay" onClick={() => setLightbox(null)}>
          <img src={publicAttachmentUrl(lightbox()!.images[lightbox()!.index]!)} class="lightbox-img" />
          <button class="lightbox-close">×</button>
        </div>
      </Show>

      <style>{`
        html.public-chat-scroll-lock,
        body.public-chat-scroll-lock,
        body.public-chat-scroll-lock #root {
          height: 100dvh !important;
          min-height: 100dvh !important;
          overflow: hidden !important;
          overscroll-behavior: none;
        }
        .public-chat-page {
          height: 100dvh !important;
          max-height: 100dvh;
          overflow: hidden;
        }
        .public-chat-shell {
          min-height: 0;
        }
        .public-chat-messages {
          min-height: 0;
          overscroll-behavior: contain;
          -webkit-overflow-scrolling: touch;
        }
        /* Keep the composer pinned to the visible viewport when the user
           pinch-zooms or the soft keyboard pushes the layout viewport up. */
        .public-chat-composer {
          transform: translateY(var(--composer-vv-shift, 0px));
          transition: transform 0.12s ease-out;
          will-change: transform;
        }
        .public-chat-composer-status {
          max-width: 900px;
          margin: 0 auto 8px;
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 6px 14px;
          background: rgba(255,255,255,0.92);
          backdrop-filter: blur(10px);
          border: 1.5px solid #f1f5f9;
          border-radius: 100px;
          box-shadow: 0 6px 18px rgba(15,23,42,0.06);
          font-size: 13px;
          font-weight: 700;
          color: #475569;
          width: fit-content;
        }
        .public-chat-composer-status.is-done {
          color: #047857;
          border-color: rgba(16,185,129,0.25);
          background: rgba(236,253,245,0.95);
        }
        .public-chat-composer-status-dot {
          width: 8px;
          height: 8px;
          border-radius: 50%;
          background: #f59e0b;
          flex-shrink: 0;
        }
        .public-chat-composer-status-dot.done { background: #10b981; }
        .public-chat-composer-status-dot.pulsing { animation: hone-status-pulse 1.4s ease-in-out infinite; }
        @keyframes hone-status-pulse {
          0%, 100% { transform: scale(1); opacity: 1; }
          50% { transform: scale(1.35); opacity: 0.55; }
        }
        .public-chat-composer-status-label { letter-spacing: 0.06em; text-transform: uppercase; font-size: 12px; }
        .public-chat-composer-status-time { font-family: var(--font-mono, 'JetBrains Mono', monospace); font-size: 12px; color: #94a3b8; }
        .public-chat-composer-status-stop {
          margin-left: 6px;
          background: #f1f5f9;
          color: #475569;
          border: none;
          padding: 3px 10px;
          border-radius: 999px;
          font-size: 11px;
          font-weight: 700;
          cursor: pointer;
          transition: background 0.2s, color 0.2s;
        }
        .public-chat-composer-status-stop:hover { background: #fee2e2; color: #ef4444; }
        /* Markdown tables: keep them inside the bubble on narrow screens. */
        .public-chat-messages .hf-markdown table {
          display: block;
          max-width: 100%;
          overflow-x: auto;
          -webkit-overflow-scrolling: touch;
        }
        .public-chat-messages .hf-markdown th,
        .public-chat-messages .hf-markdown td {
          white-space: nowrap;
        }
        .public-chat-composer-input::placeholder { color: #94a3b8; font-size: 10px; font-weight: 500; }
        @media (max-width: 768px) {
          .public-chat-composer-status {
            font-size: 12px;
            padding: 5px 12px;
            margin-bottom: 6px;
          }
          .public-chat-messages .hf-markdown {
            font-size: 15px;
          }
          .public-chat-messages .hf-markdown th,
          .public-chat-messages .hf-markdown td {
            padding: 0.55rem 0.7rem;
            font-size: 13px;
          }
          .public-chat-shell {
            padding-top: 64px !important;
          }
          .public-chat-messages {
            padding-top: 12px !important;
            padding-bottom: 8px !important;
          }
          .public-chat-messages > div {
            padding-right: 14px !important;
            padding-left: 14px !important;
          }
          .public-chat-composer {
            padding: 8px 12px calc(10px + env(safe-area-inset-bottom)) !important;
          }
          .public-chat-composer-box {
            border-radius: 20px !important;
          }
          .public-chat-composer .pub-attach-btn,
          .public-chat-send-button {
            width: 42px !important;
            height: 42px !important;
            border-radius: 14px !important;
            flex: 0 0 42px;
          }
          .public-chat-composer-input {
            min-height: 42px !important;
            padding-top: 8px !important;
            padding-bottom: 8px !important;
          }
          .public-chat-session-strip { display: none !important; }
        }
        .btn-stop-thinking { background: #f1f5f9; color: #64748b; border: none; padding: 4px 12px; border-radius: 8px; font-size: 12px; font-weight: 700; cursor: pointer; transition: all 0.2s; }
        .btn-stop-thinking:hover { background: #fee2e2; color: #ef4444; }
      `}</style>
    </div>
  );
}
