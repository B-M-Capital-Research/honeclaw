// chat.tsx — Hone Public Site Chat (v4 - Styled to match Landing Page)

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
import { PublicLoginForm } from "@/components/public-login-form";
import { PublicNav } from "@/components/public-nav";
import { ChatShareModal } from "@/components/chat-share-modal";
import { displayGithubStars, fetchGithubStars } from "@/lib/github-stars";
import { CONTENT } from "@/lib/public-content";
import { setLocale, useLocale } from "@/lib/i18n";
import {
  initPublicPrefs,
  publicFontScale,
  publicTheme,
  setPublicFontScale,
  setPublicTheme,
  type PublicTheme,
} from "@/lib/public-prefs";
import "./public-site.css";
import {
  getPublicAuthMe,
  getPublicHistory,
  isUnauthorizedApiError,
  publicLogout,
  sendPublicChat,
  uploadPublicAttachments,
} from "@/lib/api";
import { buildApiUrl } from "@/lib/backend";
import { parseMessageContent, messageId } from "@/lib/messages";
import {
  canSendPublicChatMessage,
  findPendingPublicAssistantMessage,
  formatPublicAttachmentBytes,
  isPublicChatQuotaExhausted,
  nextVisibleMessageCount,
  PUBLIC_RESTORE_TIMEOUT_MS,
  publicComposerPendingMessage,
  publicAttachmentFileLabel,
  publicRestoreRetryDelay,
  rekeyTrailingOptimisticIds,
  selectVisibleRecentMessages,
  shouldRetryPublicRestore,
  shouldRecoverPinnedBottom,
  shouldLoadOlderPublicMessages,
  splitPublicChatAttachments,
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

// ── Icons ────────────────────────────────────────────────────────────────────
const ICONS = {
  Chat: () => (
    <svg
      width="22"
      height="22"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
    </svg>
  ),
  Github: () => (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
    </svg>
  ),
  Youtube: () => (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
      <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.016 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.016 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z" />
    </svg>
  ),
  Bilibili: () => (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
      <path d="M17.813 4.653h.854c1.51.054 2.769.578 3.773 1.574 1.004.995 1.524 2.249 1.56 3.76v7.36c-.036 1.51-.556 2.769-1.56 3.773s-2.262 1.524-3.773 1.56H5.333c-1.51-.036-2.769-.556-3.773-1.56S.036 18.883 0 17.373v-7.36c.036-1.51.556-2.765 1.56-3.76 1.004-.996 2.262-1.52 3.773-1.574h.774l-1.174-1.12a1.277 1.277 0 0 1-.388-.933c0-.346.138-.64.414-.88a1.277 1.277 0 0 1 .906-.36c.345 0 .647.127.906.38l2.227 2.12h4.72l2.227-2.12c.27-.253.57-.38.906-.38.365 0 .65.12.853.36.277.24.414.534.414.88 0 .346-.13.653-.387.92zm-12.48 5.387c-.331.03-.593.15-.786.36-.193.21-.29.473-.29.787v3.507c0 .313.097.576.29.786.193.21.455.33.786.36.331-.03.593-.15.786-.36.193-.21.29-.473.29-.786v-3.507c0-.314-.097-.577-.29-.787-.193-.21-.455-.33-.786-.36zm10.707 0c-.331.03-.593.15-.786.36-.193.21-.29.473-.29.787v3.507c0 .313.097.576.29.786.193.21.455.33.786.36.345-.03.607-.15.786-.36.193-.21.29-.473.29-.786v-3.507c0-.314-.097-.577-.29-.787-.193-.21-.455-.33-.786-.36zM18 19.04H6.013c-.113 0-.17.053-.17.16 0 .12.057.18.17.18H18c.113 0 .17-.06.17-.18 0-.107-.057-.16-.17-.16z" />
    </svg>
  ),
};

const PUBLIC_IMAGE_ENDPOINT = "/api/public/image";
const HISTORY_PAGE_SIZE = 24;
const SIDEBAR_HISTORY_LIMIT = 6;

function AnimatedBackground() {
  return (
    <div class="animated-bg">
      <div class="circle circle-1"></div>
      <div class="circle circle-2"></div>
      <div class="circle circle-3"></div>
    </div>
  );
}

function PrefsButton() {
  const [open, setOpen] = createSignal(false);
  const themeOptions = createMemo<{ value: PublicTheme; label: string }[]>(() => [
    { value: "auto", label: CONTENT.chat_page.prefs.theme_auto },
    { value: "light", label: CONTENT.chat_page.prefs.theme_light },
    { value: "dark", label: CONTENT.chat_page.prefs.theme_dark },
  ]);
  const close = () => setOpen(false);
  let rootRef: HTMLDivElement | undefined;

  // Close on outside click + Esc. Document-level listeners avoid the
  // stacking-context pitfalls of a transparent backdrop sitting under a
  // position:fixed header.
  createEffect(() => {
    if (!open()) return;
    const onPointer = (e: PointerEvent) => {
      if (rootRef && !rootRef.contains(e.target as Node)) close();
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") close();
    };
    document.addEventListener("pointerdown", onPointer, true);
    document.addEventListener("keydown", onKey);
    onCleanup(() => {
      document.removeEventListener("pointerdown", onPointer, true);
      document.removeEventListener("keydown", onKey);
    });
  });

  return (
    <div class="hone-prefs" ref={rootRef}>
      <button
        type="button"
        class="hone-prefs-trigger"
        aria-label={CONTENT.chat_page.prefs.aria_label}
        aria-expanded={open()}
        onClick={() => setOpen((v) => !v)}
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M4 19l5.5-13 5.5 13M6.5 14h6M16 19h4M16 13h4M16 7h4" />
        </svg>
      </button>
      <Show when={open()}>
        <div class="hone-prefs-panel" role="dialog">
          <div class="hone-prefs-row">
            <span class="hone-prefs-label">
              {CONTENT.chat_page.prefs.font_size}
            </span>
            <div class="hone-prefs-segmented">
              <For each={["s", "m", "l", "xl"] as const}>
                {(size) => (
                  <button
                    type="button"
                    class={
                      "hone-prefs-seg" +
                      (publicFontScale() === size ? " is-active" : "")
                    }
                    data-size={size}
                    onClick={() => setPublicFontScale(size)}
                  >
                    A
                  </button>
                )}
              </For>
            </div>
          </div>
          <div class="hone-prefs-row">
            <span class="hone-prefs-label">
              {CONTENT.chat_page.prefs.theme}
            </span>
            <div class="hone-prefs-segmented">
              <For each={themeOptions()}>
                {(opt) => (
                  <button
                    type="button"
                    class={
                      "hone-prefs-seg hone-prefs-seg--text" +
                      (publicTheme() === opt.value ? " is-active" : "")
                    }
                    onClick={() => setPublicTheme(opt.value)}
                  >
                    {opt.label}
                  </button>
                )}
              </For>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}

function AccountButton(props: {
  user?: PublicAuthUserInfo | null;
  onLogout?: () => void;
}) {
  const navigate = useNavigate();
  const [open, setOpen] = createSignal(false);
  let rootRef: HTMLDivElement | undefined;

  createEffect(() => {
    if (!open()) return;
    const onPointer = (e: PointerEvent) => {
      if (rootRef && !rootRef.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", onPointer, true);
    document.addEventListener("keydown", onKey);
    onCleanup(() => {
      document.removeEventListener("pointerdown", onPointer, true);
      document.removeEventListener("keydown", onKey);
    });
  });

  const openAccountCenter = () => {
    setOpen(false);
    navigate("/me");
  };

  const logout = () => {
    setOpen(false);
    props.onLogout?.();
  };

  return (
    <Show when={props.user}>
      {(user) => (
        <div class="public-chat-account" ref={rootRef}>
          <button
            type="button"
            class="public-chat-account-trigger"
            aria-label={CONTENT.chat_page.sidebar.account_center}
            aria-expanded={open()}
            onClick={() => setOpen((value) => !value)}
          >
            <svg
              width="17"
              height="17"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <path d="M20 21a8 8 0 0 0-16 0" />
              <circle cx="12" cy="8" r="4" />
            </svg>
          </button>
          <Show when={open()}>
            <div class="public-chat-account-panel" role="dialog">
              <div class="public-chat-account-card">
                <span class="public-chat-account-avatar">H</span>
                <span>
                  <strong>{CONTENT.chat_page.sidebar.signed_in}</strong>
                  <small>{user().user_id}</small>
                </span>
              </div>
              <div class="public-chat-account-meta">
                <span>{CONTENT.me.fields.remaining}</span>
                <strong>
                  {user().remaining_today} / {user().daily_limit}
                </strong>
              </div>
              <button
                type="button"
                class="public-chat-account-center"
                onClick={openAccountCenter}
              >
                {CONTENT.chat_page.sidebar.account_center}
              </button>
              <button
                type="button"
                class="public-chat-account-logout"
                onClick={logout}
              >
                {CONTENT.chat_page.actions.logout}
              </button>
            </div>
          </Show>
        </div>
      )}
    </Show>
  );
}

function ChatSidebar(props: {
  user: PublicAuthUserInfo;
  collapsed: boolean;
  recentMessages: ChatMessage[];
  onToggle: () => void;
  onSelectMessage: (id: string) => void;
  onLogout: () => void;
}) {
  const navigate = useNavigate();
  const [stars] = createResource(fetchGithubStars);
  const messagePreview = (message: ChatMessage) => {
    const text = stripAttachmentMarkers(message.content)
      .replace(/\s+/g, " ")
      .trim();
    if (text) return text.length > 44 ? `${text.slice(0, 44)}...` : text;
    if ((message.attachments?.length ?? 0) > 0) {
      return CONTENT.chat_page.sidebar.history_attachment;
    }
    return CONTENT.chat_page.sidebar.history_empty_item;
  };

  return (
    <aside
      class={"public-chat-sidebar" + (props.collapsed ? " is-collapsed" : "")}
      aria-label={CONTENT.chat_page.sidebar.label}
    >
      <div class="public-chat-sidebar-brand">
        <button
          type="button"
          class="public-chat-sidebar-logo"
          onClick={() => navigate("/")}
          aria-label="Hone"
        >
          <img src="/logo.svg" alt="Hone" />
          <span>Hone</span>
        </button>
        <button
          type="button"
          class="public-chat-sidebar-toggle"
          onClick={props.onToggle}
          aria-label={
            props.collapsed
              ? CONTENT.chat_page.sidebar.expand
              : CONTENT.chat_page.sidebar.collapse
          }
          title={
            props.collapsed
              ? CONTENT.chat_page.sidebar.expand
              : CONTENT.chat_page.sidebar.collapse
          }
        >
          <svg
            width="17"
            height="17"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path
              d={props.collapsed ? "M9 18l6-6-6-6" : "M15 18l-6-6 6-6"}
            />
          </svg>
        </button>
      </div>

      <nav class="public-chat-sidebar-nav">
        <button type="button" class="is-active" title={CONTENT.nav.chat}>
          <ICONS.Chat />
          <span>{CONTENT.nav.chat}</span>
        </button>
        <button
          type="button"
          onClick={() => navigate("/roadmap")}
          title={CONTENT.nav.roadmap}
        >
          <span class="public-chat-sidebar-icon">R</span>
          <span>{CONTENT.nav.roadmap}</span>
        </button>
      </nav>

      <div class="public-chat-sidebar-socials">
        <a
          href={CONTENT.nav.github_url}
          target="_blank"
          rel="noopener noreferrer"
          class="public-chat-sidebar-star"
          title="GitHub"
        >
          <ICONS.Github />
          <span>{displayGithubStars(stars())}</span>
        </a>
      </div>

      <section class="public-chat-sidebar-history">
        <div class="public-chat-sidebar-section-title">
          {CONTENT.chat_page.sidebar.history_title}
        </div>
        <Show
          when={props.recentMessages.length > 0}
          fallback={
            <div class="public-chat-sidebar-history-empty">
              {CONTENT.chat_page.sidebar.history_empty}
            </div>
          }
        >
          <div class="public-chat-sidebar-history-list">
            <For each={props.recentMessages}>
              {(message, index) => (
                <button
                  type="button"
                  class="public-chat-sidebar-history-item"
                  onClick={() => props.onSelectMessage(message.id)}
                  title={messagePreview(message)}
                >
                  <span class="public-chat-sidebar-history-index">
                    {index() + 1}
                  </span>
                  <span class="public-chat-sidebar-history-text">
                    {messagePreview(message)}
                  </span>
                </button>
              )}
            </For>
          </div>
        </Show>
      </section>

      <div class="public-chat-sidebar-footer">
        <button
          type="button"
          class="public-chat-sidebar-user"
          title={props.user.user_id}
          onClick={() => navigate("/me")}
        >
          <span class="public-chat-sidebar-avatar">H</span>
          <span>
            <strong>{CONTENT.chat_page.sidebar.signed_in}</strong>
            <small>{CONTENT.chat_page.sidebar.account_center}</small>
          </span>
        </button>
        <div class="public-chat-sidebar-footer-actions">
          <button
            type="button"
            class="public-chat-sidebar-lang"
            onClick={() => setLocale(useLocale() === "zh" ? "en" : "zh")}
            title={
              useLocale() === "zh" ? CONTENT.nav.locale_en : CONTENT.nav.locale_zh
            }
          >
            {useLocale() === "zh" ? "中" : "EN"}
          </button>
          <button
            type="button"
            class="public-chat-sidebar-logout"
            onClick={props.onLogout}
            title={CONTENT.chat_page.actions.logout}
          >
            <span>{CONTENT.chat_page.actions.logout}</span>
          </button>
        </div>
      </div>
    </aside>
  );
}

function publicAttachmentUrl(att: PublicChatAttachment): string {
  if (att.previewUrl) return att.previewUrl;
  return buildApiUrl(
    `${PUBLIC_IMAGE_ENDPOINT}?path=${encodeURIComponent(att.path)}`,
  );
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

type RestoreSessionStatus = {
  attempt: number;
  mode: "loading" | "retrying" | "failed";
  message?: string;
};

function restoreErrorMessage(error: unknown) {
  if (error instanceof DOMException && error.name === "AbortError") {
    return CONTENT.chat_page.restoring.timeout_reason;
  }
  if (error instanceof Error && error.message.trim()) return error.message;
  return CONTENT.chat_page.restoring.generic_reason;
}

function LoadingCard(props: {
  status?: RestoreSessionStatus | null;
  onRetry?: () => void;
}) {
  const title = () =>
    props.status?.mode === "failed"
      ? CONTENT.chat_page.restoring.failed_title
      : CONTENT.chat_page.restoring.title;
  const desc = () => {
    const status = props.status;
    if (!status || status.mode === "loading") {
      return CONTENT.chat_page.restoring.desc;
    }
    if (status.mode === "retrying") {
      return CONTENT.chat_page.restoring.retrying.replace(
        "{attempt}",
        String(status.attempt),
      );
    }
    return CONTENT.chat_page.restoring.failed_desc;
  };

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
            <Show
              when={props.status?.mode === "failed"}
              fallback={
                <div class="h-6 w-6 animate-spin rounded-full border-2 border-white/20 border-t-white" />
              }
            >
              <span
                style={{
                  color: "#fff",
                  "font-size": "24px",
                  "font-weight": "900",
                }}
              >
                !
              </span>
            </Show>
          </div>
          <h1
            style={{
              "font-size": "22px",
              "font-weight": "800",
              color: "#0f172a",
              margin: "0 0 12px",
            }}
          >
            {title()}
          </h1>
          <p
            style={{
              "font-size": "15px",
              color: "#64748b",
              margin: "0",
              "line-height": "1.6",
            }}
          >
            {desc()}
          </p>
          <Show when={props.status?.mode === "failed" && props.status.message}>
            <p
              style={{
                "font-size": "13px",
                color: "#94a3b8",
                margin: "12px 0 0",
                "line-height": "1.6",
                "word-break": "break-word",
              }}
            >
              {CONTENT.chat_page.restoring.reason_prefix.replace(
                "{message}",
                props.status?.message ?? "",
              )}
            </p>
          </Show>
          <Show when={props.status?.mode === "failed"}>
            <button
              type="button"
              onClick={props.onRetry}
              style={{
                margin: "22px auto 0",
                height: "44px",
                padding: "0 22px",
                "border-radius": "999px",
                border: "0",
                background: "#0f172a",
                color: "#fff",
                "font-weight": "800",
                cursor: "pointer",
              }}
            >
              {CONTENT.chat_page.restoring.retry_button}
            </button>
          </Show>
        </div>
      </div>
    </div>
  );
}

function assistantMarkdownClass(white = false) {
  return [
    "public-chat-markdown",
    white ? "public-chat-markdown--white" : "",
  ].join(" ");
}

function AssistantBody(props: { content: string; white?: boolean }) {
  const cleaned = createMemo(() => stripAttachmentMarkers(props.content));
  const parts = createMemo(() =>
    parseMessageContent(cleaned(), { imageEndpoint: PUBLIC_IMAGE_ENDPOINT }),
  );
  const hasImage = () => parts().some((part) => part.type === "image");
  const markdownClass = () => assistantMarkdownClass(props.white);

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
                data-testid="assistant-inline-image"
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
                  data-testid="user-attachment-image"
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
          data-testid="user-attachment-image"
          src={publicAttachmentUrl(props.images[0]!)}
          alt={props.images[0]!.name}
          style={{ width: "100%", height: "auto", display: "block" }}
        />
      </div>
    </Show>
  );
}

function FileCard(props: {
  file: PublicChatAttachment;
  inUserBubble?: boolean;
}) {
  const ext = () => publicAttachmentFileLabel(props.file.name);
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
        background: props.inUserBubble ? "rgba(255,255,255,0.12)" : "#fff",
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
            {formatPublicAttachmentBytes(props.file.size)}
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
  const attachmentGroups = createMemo(() =>
    splitPublicChatAttachments(props.attachments),
  );
  const images = () => attachmentGroups().images;
  const files = () => attachmentGroups().files;
  const hasText = () => cleaned().length > 0;
  const hasAttach = () => images().length + files().length > 0;
  const imageOnly = () =>
    images().length > 0 && !hasText() && files().length === 0;

  return (
    <div
      class="pub-msg-in pub-msg-row"
      style={{
        display: "flex",
        "justify-content": "flex-end",
        "margin-bottom": "20px",
      }}
    >
      <div
        class="pub-msg-bubble pub-msg-bubble--user"
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
          <div
            style={{
              "margin-bottom": hasText() || files().length > 0 ? "10px" : "0",
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
              gap: "8px",
              "margin-bottom": hasText() ? "10px" : "0",
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
  isContinuation?: boolean;
  onShare?: () => void;
}) {
  const nonImageAttachments = createMemo(() =>
    (props.attachments ?? []).filter((a) => a.kind !== "image"),
  );
  const [copied, setCopied] = createSignal(false);
  const handleCopy = () => {
    const text = stripAttachmentMarkers(props.content);
    void navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1400);
    });
  };
  return (
    <div
      class="pub-msg-in pub-msg-row"
      style={{
        display: "flex",
        "justify-content": "flex-start",
        "margin-bottom": "20px",
      }}
    >
      <div
        class="pub-msg-bubble pub-msg-bubble--assistant"
        style={{
          "max-width": "85%",
          background: "rgba(255, 255, 255, 0.9)",
          "backdrop-filter": "blur(10px)",
          border: "1.5px solid #e2e8f0",
          "border-radius": "4px 24px 24px 24px",
          padding: "16px 20px",
          color: "#1e293b",
          "box-shadow": "0 4px 20px rgba(15,23,42,0.04)",
          position: "relative",
        }}
      >
        <Show when={!props.isContinuation}>
        <div
          class="pub-msg-bubble__brand"
          style={{
            display: "flex",
            "align-items": "center",
            gap: "8px",
            "margin-bottom": "12px",
          }}
        >
          <span
            style={{
              width: "8px",
              height: "8px",
              "border-radius": "50%",
              background: "#f59e0b",
              display: "inline-block",
            }}
          />
          <span
            style={{
              "font-size": "13px",
              "font-weight": "800",
              "letter-spacing": "0.1em",
              "text-transform": "uppercase",
              color: "#64748b",
            }}
          >
            HONE
          </span>
        </div>
        </Show>
        <AssistantBody content={props.content} />
        <Show when={nonImageAttachments().length > 0}>
          <div
            style={{
              display: "flex",
              "flex-direction": "column",
              gap: "8px",
              "margin-top": "16px",
            }}
          >
            <For each={nonImageAttachments()}>
              {(file) => <FileCard file={file} />}
            </For>
          </div>
        </Show>
        <div class="pub-msg-actions">
          <button
            type="button"
            class="pub-msg-action"
            aria-label={CONTENT.chat_page.actions.copy_aria}
            title={
              copied()
                ? CONTENT.chat_page.actions.copied
                : CONTENT.chat_page.actions.copy_aria
            }
            onClick={handleCopy}
            data-copied={copied() ? "true" : undefined}
          >
            <Show
              when={copied()}
              fallback={
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                </svg>
              }
            >
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.4"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
              >
                <path d="M20 6L9 17l-5-5" />
              </svg>
            </Show>
          </button>
          <Show when={props.onShare}>
            <button
              type="button"
              class="pub-msg-action"
              aria-label={CONTENT.chat_page.actions.share_aria}
              title={CONTENT.chat_page.actions.share_aria}
              onClick={() => props.onShare?.()}
            >
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
              >
                <circle cx="18" cy="5" r="3" />
                <circle cx="6" cy="12" r="3" />
                <circle cx="18" cy="19" r="3" />
                <line x1="8.59" y1="13.51" x2="15.42" y2="17.49" />
                <line x1="15.41" y1="6.51" x2="8.59" y2="10.49" />
              </svg>
            </button>
          </Show>
        </div>
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
    if (props.message.phase === "done" || props.message.phase === "error")
      return;
    const timer = setInterval(tick, 1000);
    onCleanup(() => clearInterval(timer));
  });

  const terminal = () => props.message.phase === "error";
  const labelText = () => {
    switch (props.message.phase) {
      case "error":
        return CONTENT.chat_page.status.error;
      case "streaming":
        return CONTENT.chat_page.status.streaming;
      case "running":
        return CONTENT.chat_page.status.running;
      default:
        return CONTENT.chat_page.status.thinking;
    }
  };
  return (
    <div
      class="pub-msg-in pub-msg-row"
      style={{
        display: "flex",
        "justify-content": "flex-start",
        "margin-bottom": "20px",
      }}
    >
      <div
        class="pub-msg-bubble pub-msg-bubble--assistant"
        style={{
          "max-width": "85%",
          background: "#fff",
          border: terminal()
            ? "2px solid rgba(239,68,68,0.2)"
            : "1.5px solid #e2e8f0",
          "border-radius": "4px 24px 24px 24px",
          padding: "16px 20px",
          "box-shadow": "0 10px 30px rgba(0,0,0,0.03)",
        }}
      >
        {/* The header status row only shows in error state — for the
            normal thinking/streaming flow the composer-side status strip
            is the single source of truth (avoids duplicate "HONE 思考中"). */}
        <Show when={terminal()}>
          <div
            style={{
              display: "flex",
              "align-items": "center",
              "justify-content": "space-between",
              gap: "10px",
              "margin-bottom": props.message.content ? "12px" : "0",
            }}
          >
            <div
              style={{ display: "flex", "align-items": "center", gap: "8px" }}
            >
              <span
                style={{
                  width: "8px",
                  height: "8px",
                  "border-radius": "50%",
                  background: "#ef4444",
                }}
              />
              <span
                style={{
                  "font-size": "13px",
                  "font-weight": "800",
                  "letter-spacing": "0.1em",
                  "text-transform": "uppercase",
                  color: "#64748b",
                }}
              >
                {labelText()}
              </span>
              <span
                style={{
                  "font-family": "var(--font-mono)",
                  "font-size": "12px",
                  color: "rgba(0,0,0,0.35)",
                }}
              >
                {elapsed()}s
              </span>
            </div>
            <button
              onClick={props.onDismiss}
              style={{
                background: "none",
                border: "none",
                cursor: "pointer",
                color: "#64748b",
                "font-size": "14px",
              }}
            >
              ✕
            </button>
          </div>
        </Show>

        <Show when={(props.message.steps?.length ?? 0) > 0}>
          <ul
            style={{
              margin: props.message.content ? "0 0 12px" : "8px 0 0",
              padding: "0",
              "list-style": "none",
              "font-size": "13px",
              "line-height": "1.8",
              color: "#64748b",
            }}
          >
            <For each={props.message.steps}>
              {(step) => (
                <li
                  style={{
                    display: "flex",
                    "align-items": "flex-start",
                    gap: "8px",
                  }}
                >
                  <span style={{ color: "#f59e0b" }}>•</span>
                  <span>{step}</span>
                </li>
              )}
            </For>
          </ul>
        </Show>

        <Show when={props.message.content}>
          <div style={{ "white-space": "pre-wrap" }}>
            <AssistantBody content={props.message.content} />
            <Show when={props.message.phase === "streaming"}>
              <span class="pub-cursor" />
            </Show>
          </div>
        </Show>

        <Show when={terminal()}>
          <div
            style={{
              "font-size": "14px",
              color: "#ef4444",
              "margin-top": "6px",
              "font-weight": "600",
            }}
          >
            {props.message.statusText || CONTENT.chat_page.status.fallback_error}
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
        data-testid="composer-attach-preview"
        style={{
          display: "flex",
          gap: "10px",
          padding: "12px 16px",
          "flex-wrap": "wrap",
          "border-bottom": "1.5px solid #f8fafc",
        }}
      >
        <For each={props.items}>
          {(item, index) => (
            <div style={{ position: "relative" }}>
              <Show
                when={item.kind === "image"}
                fallback={
                  <div
                    style={{
                      width: "200px",
                      height: "72px",
                      padding: "0 12px",
                      display: "flex",
                      "align-items": "center",
                      gap: "12px",
                      "border-radius": "12px",
                      border: "1.5px solid #f1f5f9",
                      background: "#fcfdfe",
                    }}
                  >
                    <div
                      style={{
                        width: "40px",
                        height: "40px",
                        "border-radius": "8px",
                        background: "rgba(245,158,11,0.1)",
                        display: "flex",
                        "align-items": "center",
                        "justify-content": "center",
                        "font-family": "var(--font-mono)",
                        "font-size": "11px",
                        "font-weight": "800",
                        color: "#d97706",
                      }}
                    >
                      {publicAttachmentFileLabel(item.name)}
                    </div>
                    <div style={{ flex: "1", "min-width": "0" }}>
                      <div
                        style={{
                          "font-size": "13px",
                          "font-weight": "700",
                          color: "#0f172a",
                          overflow: "hidden",
                          "text-overflow": "ellipsis",
                          "white-space": "nowrap",
                        }}
                      >
                        {item.name}
                      </div>
                      <div
                        style={{
                          "font-family": "var(--font-mono)",
                          "font-size": "11px",
                          color: "#94a3b8",
                        }}
                      >
                        {formatPublicAttachmentBytes(item.size)}
                      </div>
                    </div>
                  </div>
                }
              >
                <div
                  style={{
                    width: "72px",
                    height: "72px",
                    "border-radius": "12px",
                    overflow: "hidden",
                    border: "1.5px solid #f1f5f9",
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
                onClick={() => props.onRemove(index())}
                style={{
                  position: "absolute",
                  top: "-8px",
                  right: "-8px",
                  width: "24px",
                  height: "24px",
                  "border-radius": "12px",
                  background: "#000",
                  color: "#fff",
                  border: "2.5px solid #fff",
                  cursor: "pointer",
                  "font-size": "12px",
                  display: "flex",
                  "align-items": "center",
                  "justify-content": "center",
                  "box-shadow": "0 4px 10px rgba(0,0,0,0.2)",
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
      <div
        class="pub-attach-menu"
        style={{
          "border-radius": "20px",
          padding: "8px",
          "min-width": "240px",
          bottom: "80px",
          "box-shadow": "0 20px 50px rgba(0,0,0,0.15)",
        }}
      >
        <button
          type="button"
          class="pub-attach-item"
          onClick={() => {
            props.onPickImage();
            props.onClose();
          }}
        >
          <span class="pub-attach-icon" style={{ background: "#f1f5f9" }}>
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <rect x="3" y="5" width="18" height="14" rx="2.5" />
              <circle cx="8.5" cy="10" r="1.5" />
              <path d="M21 15l-5-5-8 9" />
            </svg>
          </span>
          <span class="pub-attach-label">
            <span
              class="pub-attach-label-title"
              style={{ "font-size": "15px" }}
            >
              {CONTENT.chat_page.attachments.image_title}
            </span>
            <span class="pub-attach-label-sub">
              {CONTENT.chat_page.attachments.image_subtitle}
            </span>
          </span>
        </button>
        <button
          type="button"
          class="pub-attach-item"
          onClick={() => {
            props.onPickFile();
            props.onClose();
          }}
        >
          <span class="pub-attach-icon" style={{ background: "#f1f5f9" }}>
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M14 3H7a2 2 0 00-2 2v14a2 2 0 002 2h10a2 2 0 002-2V8z" />
              <path d="M14 3v5h5" />
              <path d="M9 13h6M9 17h4" />
            </svg>
          </span>
          <span class="pub-attach-label">
            <span
              class="pub-attach-label-title"
              style={{ "font-size": "15px" }}
            >
              {CONTENT.chat_page.attachments.file_title}
            </span>
            <span class="pub-attach-label-sub">
              {CONTENT.chat_page.attachments.file_subtitle}
            </span>
          </span>
        </button>
      </div>
    </Show>
  );
}

function ComposerStatus(props: {
  message: ChatMessage | undefined;
  onStop: () => void;
  justFinished: boolean;
}) {
  const [elapsed, setElapsed] = createSignal(0);

  createEffect(() => {
    const m = props.message;
    if (!m || !m.startedAt) {
      setElapsed(0);
      return;
    }
    const tick = () =>
      setElapsed(
        Math.max(0, Math.floor((Date.now() - (m.startedAt ?? 0)) / 1000)),
      );
    tick();
    const timer = setInterval(tick, 1000);
    onCleanup(() => clearInterval(timer));
  });

  const labelText = (m: ChatMessage) => {
    switch (m.phase) {
      case "streaming":
        return CONTENT.chat_page.status.streaming;
      case "running":
        return CONTENT.chat_page.status.running;
      default:
        return CONTENT.chat_page.status.thinking;
    }
  };

  return (
    <Show when={props.message || props.justFinished}>
      <div
        class={
          "public-chat-composer-status" + (props.justFinished ? " is-done" : "")
        }
        role="status"
        aria-live="polite"
      >
        <Show
          when={props.message}
          fallback={
            <>
              <span class="public-chat-composer-status-dot done" />
              <span class="public-chat-composer-status-label">
                {CONTENT.chat_page.status.done}
              </span>
            </>
          }
        >
          {(m) => (
            <>
              <span class="public-chat-composer-status-dot pulsing" />
              <span class="public-chat-composer-status-label">
                {labelText(m())}
              </span>
              <span class="public-chat-composer-status-time">{elapsed()}s</span>
              <button
                type="button"
                class="public-chat-composer-status-stop"
                onClick={props.onStop}
              >
                {CONTENT.chat_page.status.stop}
              </button>
            </>
          )}
        </Show>
      </div>
    </Show>
  );
}

function ProactiveModeTips() {
  const [open, setOpen] = createSignal(false);
  const [copiedExample, setCopiedExample] = createSignal<number | null>(null);
  let copiedTimer: number | undefined;

  const copyText = async (text: string) => {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return;
    }
    const textarea = document.createElement("textarea");
    textarea.value = text;
    textarea.setAttribute("readonly", "");
    textarea.style.position = "fixed";
    textarea.style.left = "-9999px";
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand("copy");
    textarea.remove();
  };

  const copyExample = async (text: string, index: number) => {
    await copyText(text);
    setCopiedExample(index);
    if (copiedTimer) window.clearTimeout(copiedTimer);
    copiedTimer = window.setTimeout(() => setCopiedExample(null), 1200);
  };

  createEffect(() => {
    if (!open()) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("keydown", onKey);
    onCleanup(() => document.removeEventListener("keydown", onKey));
  });

  onCleanup(() => {
    if (copiedTimer) window.clearTimeout(copiedTimer);
  });

  return (
    <>
      <div class="public-chat-proactive-tip-wrap">
        <button
          type="button"
          class="public-chat-proactive-tip"
          aria-haspopup="dialog"
          aria-expanded={open()}
          onClick={() => setOpen(true)}
        >
          <svg
            class="public-chat-proactive-tip-icon"
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path d="M9 6V5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v1" />
            <path d="M5 6h14a2 2 0 0 1 2 2v10.5a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2Z" />
            <path d="M3 12h18" />
            <path d="M12 10.5v3" />
          </svg>
          <span>{CONTENT.chat_page.composer.proactive_tip}</span>
        </button>
      </div>
      <Show when={open()}>
        <div
          class="public-chat-proactive-modal-backdrop"
          role="presentation"
          onClick={() => setOpen(false)}
        >
          <div
            class="public-chat-proactive-modal"
            role="dialog"
            aria-modal="true"
            aria-labelledby="public-chat-proactive-title"
            onClick={(e) => e.stopPropagation()}
          >
            <button
              type="button"
              class="public-chat-proactive-close"
              aria-label={CONTENT.chat_page.composer.proactive_close_aria}
              onClick={() => setOpen(false)}
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.4"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
              >
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
            <h2 id="public-chat-proactive-title">
              {CONTENT.chat_page.composer.proactive_title}
            </h2>
            <p class="public-chat-proactive-intro">
              {CONTENT.chat_page.composer.proactive_intro}
            </p>
            <div class="public-chat-proactive-list">
              <For each={CONTENT.chat_page.composer.proactive_items}>
                {(item) => (
                  <div class="public-chat-proactive-item">
                    <span class="public-chat-proactive-item-mark" />
                    <span>
                      <strong>{item.title}</strong>
                      <small>{item.body}</small>
                    </span>
                  </div>
                )}
              </For>
            </div>
            <div class="public-chat-proactive-examples">
              <div>{CONTENT.chat_page.composer.proactive_examples_title}</div>
              <For each={CONTENT.chat_page.composer.proactive_examples}>
                {(example, index) => (
                  <span class="public-chat-proactive-example-row">
                    <button
                      type="button"
                      class="public-chat-proactive-copy"
                      aria-label={CONTENT.chat_page.actions.copy_aria}
                      title={
                        copiedExample() === index()
                          ? CONTENT.chat_page.actions.copied
                          : CONTENT.chat_page.actions.copy_aria
                      }
                      onClick={() => void copyExample(example, index())}
                    >
                      <Show
                        when={copiedExample() === index()}
                        fallback={
                          <svg
                            width="13"
                            height="13"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            aria-hidden="true"
                          >
                            <rect
                              x="9"
                              y="9"
                              width="13"
                              height="13"
                              rx="2"
                              ry="2"
                            />
                            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                          </svg>
                        }
                      >
                        <svg
                          width="13"
                          height="13"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2.2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          aria-hidden="true"
                        >
                          <path d="M20 6 9 17l-5-5" />
                        </svg>
                      </Show>
                    </button>
                    <span>{example}</span>
                  </span>
                )}
              </For>
            </div>
            <button
              type="button"
              class="public-chat-proactive-primary"
              onClick={() => setOpen(false)}
            >
              {CONTENT.chat_page.composer.proactive_got_it}
            </button>
          </div>
        </div>
      </Show>
    </>
  );
}

function Composer(props: {
  draft: string;
  onDraftChange: (v: string) => void;
  attachments: PublicChatAttachment[];
  onRemoveAttachment: (index: number) => void;
  onPickFiles: (files: File[]) => void;
  uploading: boolean;
  onSend: () => void;
  onStop: () => void;
  isSending: boolean;
  remaining: number | undefined;
  dailyLimit: number | undefined;
  pendingMessage: ChatMessage | undefined;
  justFinished: boolean;
}) {
  const [focused, setFocused] = createSignal(false);
  const [menuOpen, setMenuOpen] = createSignal(false);
  let taRef: HTMLTextAreaElement | undefined;
  let imgInputRef: HTMLInputElement | undefined;
  let fileInputRef: HTMLInputElement | undefined;

  const quotaExhausted = () =>
    isPublicChatQuotaExhausted({
      remaining: props.remaining,
      dailyLimit: props.dailyLimit,
    });
  const canSend = () =>
    canSendPublicChatMessage({
      draft: props.draft,
      attachmentCount: props.attachments.length,
      isSending: props.isSending,
      uploading: props.uploading,
      remaining: props.remaining,
      dailyLimit: props.dailyLimit,
    });
  const isMobileViewport = () =>
    typeof window !== "undefined" &&
    window.matchMedia("(max-width: 768px)").matches;
  const syncTextareaHeight = () => {
    if (!taRef) return;
    const maxHeight = isMobileViewport() ? 132 : 180;
    taRef.style.height = "auto";
    const nextHeight = Math.min(taRef.scrollHeight, maxHeight);
    taRef.style.height = `${nextHeight}px`;
    taRef.style.overflowY = taRef.scrollHeight > maxHeight ? "auto" : "hidden";
  };
  const handlePaste = (e: ClipboardEvent) => {
    const items = Array.from(e.clipboardData?.items ?? []);
    const files = items
      .filter((item) => item.kind === "file" && item.type.startsWith("image/"))
      .map((item) => item.getAsFile())
      .filter((file): file is File => !!file)
      .map(renamePasteFile);
    if (files.length === 0) return;
    e.preventDefault();
    props.onPickFiles(files);
  };

  createEffect(() => {
    props.draft;
    queueMicrotask(syncTextareaHeight);
  });

  createEffect(() => {
    if (!props.isSending && taRef && !isMobileViewport()) {
      taRef.focus();
      syncTextareaHeight();
    }
  });

  return (
    <div
      class="public-chat-composer"
      style={{
        padding: "16px 24px 32px",
        background: "transparent",
        "flex-shrink": "0",
        position: "relative",
        "z-index": "20",
      }}
    >
      <ComposerStatus
        message={props.pendingMessage}
        onStop={props.onStop}
        justFinished={props.justFinished}
      />
      <ProactiveModeTips />
      <input
        data-testid="composer-image-input"
        ref={imgInputRef}
        type="file"
        accept="image/*"
        multiple
        style={{ display: "none" }}
        onChange={(e) => {
          const files = e.currentTarget.files
            ? Array.from(e.currentTarget.files)
            : [];
          e.currentTarget.value = "";
          if (files.length) props.onPickFiles(files);
        }}
      />
      <input
        ref={fileInputRef}
        type="file"
        multiple
        style={{ display: "none" }}
        onChange={(e) => {
          const files = e.currentTarget.files
            ? Array.from(e.currentTarget.files)
            : [];
          e.currentTarget.value = "";
          if (files.length) props.onPickFiles(files);
        }}
      />

      <AttachMenu
        open={menuOpen()}
        onClose={() => setMenuOpen(false)}
        onPickImage={() => imgInputRef?.click()}
        onPickFile={() => fileInputRef?.click()}
      />

      <div
        class="public-chat-composer-box"
        style={{
          position: "relative",
          "max-width": "900px",
          margin: "0 auto",
          "border-radius": "22px",
          border: focused() ? "2px solid #000" : "2px solid #f1f5f9",
          background: "#fff",
          "box-shadow": focused()
            ? "0 20px 60px rgba(0,0,0,0.08)"
            : "0 10px 30px rgba(0,0,0,0.03)",
          transition: "all 0.3s cubic-bezier(0.16, 1, 0.3, 1)",
          overflow: "hidden",
        }}
      >
        <AttachPreview
          items={props.attachments}
          onRemove={props.onRemoveAttachment}
        />
        <div
          class="public-chat-composer-row"
          style={{
            display: "flex",
            "align-items": "center",
            gap: "6px",
            padding: "6px 10px",
          }}
        >
          <button
            data-testid="composer-attach-button"
            type="button"
            class="pub-attach-btn"
            data-open={menuOpen() ? "true" : undefined}
            style={{ width: "36px", height: "36px", "flex-shrink": "0" }}
            onClick={() => setMenuOpen(!menuOpen())}
          >
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M21.44 11.05l-9.19 9.19a6 6 0 11-8.49-8.49l9.19-9.19a4 4 0 115.66 5.66l-9.2 9.19a2 2 0 11-2.83-2.83l8.49-8.48" />
            </svg>
          </button>
          <textarea
            ref={taRef}
            class="public-chat-composer-input"
            rows={1}
            placeholder={
              quotaExhausted()
                ? CONTENT.chat_page.composer.quota_exhausted
                : CONTENT.chat_page.composer.placeholder
            }
            value={props.draft}
            disabled={props.isSending}
            onInput={(e) => {
              props.onDraftChange(e.currentTarget.value);
              requestAnimationFrame(syncTextareaHeight);
            }}
            onKeyDown={(e) => {
              if (!e.isComposing && e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                if (canSend()) props.onSend();
              }
            }}
            onPaste={handlePaste}
            onFocus={() => setFocused(true)}
            onBlur={() => setFocused(false)}
            style={{
              flex: "1",
              resize: "none",
              border: "none",
              outline: "none",
              background: "transparent",
              padding: "6px 6px",
              "font-size": "16px",
              "font-weight": "500",
              "line-height": "1.5",
              color: "#0f172a",
              "max-height": "180px",
              "min-height": "32px",
              overflow: "hidden auto",
            }}
          />
          <button
            data-testid="composer-send-button"
            type="button"
            class="public-chat-send-button"
            aria-label={CONTENT.chat_page.composer.send_aria}
            title={CONTENT.chat_page.composer.send_aria}
            onClick={() => canSend() && props.onSend()}
            disabled={!canSend()}
            style={{
              width: "36px",
              height: "36px",
              "border-radius": "12px",
              background: canSend() ? "#000" : "#f1f5f9",
              border: "none",
              cursor: canSend() ? "pointer" : "default",
              display: "flex",
              "align-items": "center",
              "justify-content": "center",
              "flex-shrink": "0",
              transition: "all 0.2s",
            }}
          >
            <svg
              viewBox="0 0 20 20"
              width="16"
              height="16"
              fill={canSend() ? "white" : "#94a3b8"}
            >
              <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}

export default function PublicChatPage() {
  const navigate = useNavigate();
  const [authState, setAuthState] = createSignal<AuthState>("loading");
  const [currentUser, setCurrentUser] = createSignal<PublicAuthUserInfo | null>(
    null,
  );
  const [messages, setMessages] = createStore<ChatMessage[]>([]);
  const [draft, setDraft] = createSignal("");
  const [isSending, setIsSending] = createSignal(false);
  const [pendingAttachments, setPendingAttachments] = createStore<
    PublicChatAttachment[]
  >([]);
  const [uploading, setUploading] = createSignal(false);
  const [lightbox, setLightbox] = createSignal<{
    images: PublicChatAttachment[];
    index: number;
  } | null>(null);
  // When set, the share modal is open and `seedIndex` is the index of the
  // message the user clicked 分享 from (within visibleMessages()). The modal
  // pre-selects that one and lets the user toggle the rest.
  const [shareSeed, setShareSeed] = createSignal<number | null>(null);
  const [sessionInfo, setSessionInfo] = createSignal<{
    userId: string;
    remainingToday: number;
    dailyLimit: number;
  } | null>(null);
  const [visibleMessageCount, setVisibleMessageCount] =
    createSignal(HISTORY_PAGE_SIZE);
  const [loadingOlderMessages, setLoadingOlderMessages] = createSignal(false);
  const [justFinished, setJustFinished] = createSignal(false);
  const [sidebarCollapsed, setSidebarCollapsed] = createSignal(false);
  // True when the user has scrolled up far enough to lose track of the latest
  // reply — drives the floating scroll-to-bottom affordance above the composer.
  const [awayFromBottom, setAwayFromBottom] = createSignal(false);
  // When set, the server has an in-flight assistant run for which we have
  // no local streaming context — typically because the page was refreshed
  // mid-response. Until the answer arrives, show the same "思考中" status
  // and poll history so the reply lands without manual refresh.
  const [backgroundPending, setBackgroundPending] = createSignal<{
    since: number;
  } | null>(null);
  const [restoreStatus, setRestoreStatus] =
    createSignal<RestoreSessionStatus | null>({ attempt: 1, mode: "loading" });
  let activeController: AbortController | null = null;
  let restoreController: AbortController | null = null;
  let restoreRetryTimer: number | undefined;
  let scrollRef: HTMLDivElement | undefined;
  let messagesInnerRef: HTMLDivElement | undefined;
  let sessionSyncGeneration = 0;
  let stickToBottom = true;
  let lastScrollTop = 0;
  let suppressScrollUntil = 0;
  let pinBottomUntil = 0;
  let shareReturnScrollTop: number | null = null;
  let shareReturnAtBottom = true;
  let justFinishedTimer: number | undefined;

  const scrollToBottom = () => {
    requestAnimationFrame(() => {
      if (!scrollRef) return;
      suppressScrollUntil = Math.max(suppressScrollUntil, Date.now() + 180);
      scrollRef.scrollTop = Math.max(
        0,
        scrollRef.scrollHeight - scrollRef.clientHeight,
      );
      lastScrollTop = scrollRef.scrollTop;
    });
  };
  const scrollToMessage = (id: string) => {
    const index = messages.findIndex((message) => message.id === id);
    if (index < 0) return;
    const neededVisibleCount = messages.length - index;
    setVisibleMessageCount((current) =>
      Math.max(current, neededVisibleCount, HISTORY_PAGE_SIZE),
    );
    requestAnimationFrame(() => {
      document
        .getElementById(`public-chat-message-${id}`)
        ?.scrollIntoView({ block: "center", behavior: "smooth" });
    });
  };
  const pinToBottom = (durationMs = 900) => {
    stickToBottom = true;
    const until = Date.now() + durationMs;
    pinBottomUntil = Math.max(pinBottomUntil, until);
    suppressScrollUntil = Math.max(suppressScrollUntil, until);
    setAwayFromBottom(false);
    scrollToBottom();
    requestAnimationFrame(scrollToBottom);
    window.setTimeout(scrollToBottom, 40);
    window.setTimeout(scrollToBottom, 90);
    window.setTimeout(scrollToBottom, 180);
    window.setTimeout(scrollToBottom, 360);
  };
  const settleAtBottom = () => pinToBottom(900);
  const isBottomPinned = () => Date.now() < pinBottomUntil;
  const distanceFromBottom = () =>
    scrollRef
      ? scrollRef.scrollHeight - scrollRef.scrollTop - scrollRef.clientHeight
      : 0;
  const visibleMessages = createMemo(() =>
    selectVisibleRecentMessages(messages, visibleMessageCount()),
  );
  const sidebarHistoryMessages = createMemo(() =>
    messages
      .filter((message) => message.role === "user")
      .slice(-SIDEBAR_HISTORY_LIMIT)
      .reverse(),
  );
  const hasOlderMessages = () => visibleMessageCount() < messages.length;
  const isSendingOrStreaming = () =>
    isSending() || !!pendingAssistantMessage() || !!backgroundPending();
  const pendingAssistantMessage = createMemo(() => {
    return findPendingPublicAssistantMessage(messages);
  });
  const composerPendingMessage = createMemo<ChatMessage | undefined>(() => {
    return publicComposerPendingMessage({
      local: pendingAssistantMessage(),
      background: backgroundPending(),
    });
  });

  const loadOlderMessages = () => {
    if (!scrollRef || !hasOlderMessages() || loadingOlderMessages()) return;
    const previousScrollHeight = scrollRef.scrollHeight;
    const previousScrollTop = scrollRef.scrollTop;
    setLoadingOlderMessages(true);
    setVisibleMessageCount((current) =>
      nextVisibleMessageCount(messages.length, current, HISTORY_PAGE_SIZE),
    );
    requestAnimationFrame(() => {
      if (scrollRef) {
        suppressScrollUntil = Date.now() + 180;
        scrollRef.scrollTop =
          previousScrollTop + (scrollRef.scrollHeight - previousScrollHeight);
        lastScrollTop = scrollRef.scrollTop;
      }
      setLoadingOlderMessages(false);
    });
  };

  const handleMessagesScroll = () => {
    if (!scrollRef) return;
    const top = scrollRef.scrollTop;
    const dist = distanceFromBottom();
    // Ignore scroll events produced by our own bottom pinning/history
    // compensation. Mobile browsers can emit these while keyboard/layout
    // metrics settle; treating them as user scrolls can jump to older messages.
    if (Date.now() < suppressScrollUntil || isBottomPinned()) {
      if (
        shouldRecoverPinnedBottom({
          scrollTop: top,
          distanceFromBottom: dist,
          pinnedToBottom: stickToBottom || isBottomPinned(),
        })
      ) {
        scrollToBottom();
        return;
      }
      if (!isBottomPinned()) lastScrollTop = top;
      return;
    }
    suppressScrollUntil = 0;
    const sendingOrStreaming = isSendingOrStreaming();
    if (
      sendingOrStreaming &&
      stickToBottom &&
      top <= 24 &&
      top < lastScrollTop - 2 &&
      dist > 120
    ) {
      scrollToBottom();
      return;
    }
    if (top < lastScrollTop - 2) {
      // user-initiated scroll up
      stickToBottom = dist < 80;
    } else if (dist < 80) {
      stickToBottom = true;
    }
    setAwayFromBottom(dist > 240);
    const previousTop = lastScrollTop;
    lastScrollTop = top;
    if (
      shouldLoadOlderPublicMessages({
        scrollTop: top,
        previousScrollTop: previousTop,
        distanceFromBottom: dist,
        hasOlderMessages: hasOlderMessages(),
        loadingOlderMessages: loadingOlderMessages(),
        sendingOrStreaming,
      })
    ) {
      loadOlderMessages();
    }
  };

  const flashJustFinished = () => {
    setJustFinished(true);
    if (justFinishedTimer !== undefined) window.clearTimeout(justFinishedTimer);
    justFinishedTimer = window.setTimeout(() => setJustFinished(false), 2400);
  };

  const openShareModal = (seedIndex: number) => {
    shareReturnScrollTop = scrollRef?.scrollTop ?? null;
    shareReturnAtBottom = stickToBottom || distanceFromBottom() < 160;
    if (shareReturnAtBottom) pinToBottom(900);
    setShareSeed(seedIndex);
  };

  const closeShareModal = () => {
    setShareSeed(null);
    if (shareReturnAtBottom) {
      pinToBottom(1200);
    } else if (scrollRef && shareReturnScrollTop !== null) {
      const nextTop = shareReturnScrollTop;
      suppressScrollUntil = Date.now() + 260;
      requestAnimationFrame(() => {
        if (!scrollRef) return;
        scrollRef.scrollTop = nextTop;
        lastScrollTop = scrollRef.scrollTop;
      });
    }
    shareReturnScrollTop = null;
  };

  // When the inner messages content grows (streaming, new message), keep the
  // viewport glued to the bottom unless the user has explicitly scrolled away.
  createEffect(() => {
    if (!messagesInnerRef || typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver(() => {
      if (stickToBottom) scrollToBottom();
    });
    ro.observe(messagesInnerRef);
    onCleanup(() => ro.disconnect());
  });

  const applyPublicUser = (user: PublicAuthUserInfo) => {
    setSessionInfo({
      userId: user.user_id,
      remainingToday: user.remaining_today,
      dailyLimit: user.daily_limit,
    });
    setCurrentUser(user);
    setAuthState("ready");
  };

  const logoutPublicChat = () => {
    void publicLogout();
    setCurrentUser(null);
    setSessionInfo(null);
    setAuthState("logged_out");
  };

  const clearRestoreRetry = () => {
    if (restoreRetryTimer !== undefined) {
      window.clearTimeout(restoreRetryTimer);
      restoreRetryTimer = undefined;
    }
  };

  const restoreSession = async (
    options: {
      resetWindow?: boolean;
      keepAtBottom?: boolean;
      retryOnFailure?: boolean;
      attempt?: number;
    } = {},
  ) => {
    clearRestoreRetry();
    const generation = ++sessionSyncGeneration;
    const attempt = options.attempt ?? 1;
    const retryOnFailure =
      options.retryOnFailure ??
      (authState() === "loading" || options.resetWindow);
    restoreController?.abort();
    const controller = new AbortController();
    restoreController = controller;
    const timeoutId = window.setTimeout(
      () => controller.abort(),
      PUBLIC_RESTORE_TIMEOUT_MS,
    );
    if (authState() === "loading" || !currentUser()) {
      setRestoreStatus({
        attempt,
        mode: attempt > 1 ? "retrying" : "loading",
      });
    }
    try {
      const user = await getPublicAuthMe(controller.signal);
      if (generation !== sessionSyncGeneration) return;
      applyPublicUser(user);
      const history = await getPublicHistory(controller.signal);
      if (generation !== sessionSyncGeneration) return;
      const next = toPublicChatMessages(history);
      if (options.resetWindow) {
        setVisibleMessageCount(HISTORY_PAGE_SIZE);
      } else {
        // Preserve user's current viewing window; never shrink it on a sync.
        setVisibleMessageCount((c) =>
          Math.max(c, Math.min(next.length, HISTORY_PAGE_SIZE)),
        );
      }
      const previousScrollTop = scrollRef?.scrollTop;
      const shouldKeepBottom =
        options.resetWindow ||
        options.keepAtBottom ||
        stickToBottom ||
        distanceFromBottom() < 120;
      if (shouldKeepBottom) pinToBottom(1200);
      // Keep optimistic UUIDs on the just-sent pair so reconcile patches the
      // bubbles in place instead of swapping the DOM nodes for the next
      // history-derived stable ids — the swap collapses scrollHeight long
      // enough for the browser to clamp scrollTop "to the top of the
      // conversation" before settleAtBottom can pull it back.
      rekeyTrailingOptimisticIds(messages, next);
      setMessages(reconcile(next, { key: "id" }));
      if (shouldKeepBottom) {
        pinToBottom(1200);
      } else if (previousScrollTop !== undefined) {
        requestAnimationFrame(() => {
          if (scrollRef) {
            scrollRef.scrollTop = previousScrollTop;
            lastScrollTop = scrollRef.scrollTop;
          }
        });
      }
      // If the server has a run in flight and we're not the one streaming
      // it (e.g. page was just refreshed mid-answer), surface a "思考中"
      // status until the reply lands.
      const lastIsUser =
        next.length > 0 && next[next.length - 1]!.role === "user";
      if (user.in_flight > 0 && lastIsUser && !isSending()) {
        setBackgroundPending((prev) => prev ?? { since: Date.now() });
      } else {
        setBackgroundPending(null);
      }
      setRestoreStatus(null);
    } catch (error) {
      if (generation !== sessionSyncGeneration) return;
      if (isUnauthorizedApiError(error)) {
        setRestoreStatus(null);
        setAuthState("logged_out");
        return;
      }
      const message = restoreErrorMessage(error);
      if (retryOnFailure && shouldRetryPublicRestore(attempt)) {
        const nextAttempt = attempt + 1;
        const retryDelay = publicRestoreRetryDelay(attempt);
        setRestoreStatus({
          attempt: nextAttempt,
          mode: "retrying",
          message,
        });
        restoreRetryTimer = window.setTimeout(() => {
          restoreRetryTimer = undefined;
          void restoreSession({
            ...options,
            attempt: nextAttempt,
            retryOnFailure: true,
          });
        }, retryDelay);
        return;
      }
      if (authState() === "loading" || !currentUser()) {
        setRestoreStatus({ attempt, mode: "failed", message });
      }
    } finally {
      window.clearTimeout(timeoutId);
      if (restoreController === controller) restoreController = null;
    }
  };

  // Poll while the server still owes us an answer we can't stream locally.
  createEffect(() => {
    if (!backgroundPending() || isSending()) return;
    const id = window.setInterval(() => {
      void restoreSession();
    }, 3000);
    onCleanup(() => clearInterval(id));
  });

  // Flash "本轮已完成" when a background-pending run resolves.
  let bgPrev = false;
  createEffect(() => {
    const cur = !!backgroundPending();
    if (bgPrev && !cur && !isSending()) flashJustFinished();
    bgPrev = cur;
  });

  onMount(() => {
    initPublicPrefs();
    const viewportMeta = document.querySelector<HTMLMetaElement>(
      'meta[name="viewport"]',
    );
    const previousViewport = viewportMeta?.getAttribute("content") ?? null;
    viewportMeta?.setAttribute(
      "content",
      "width=device-width, initial-scale=1, viewport-fit=cover, interactive-widget=resizes-content",
    );
    const preventGesture = (event: Event) => event.preventDefault();
    document.addEventListener("gesturestart", preventGesture);
    document.addEventListener("gesturechange", preventGesture);
    document.addEventListener("gestureend", preventGesture);
    void restoreSession({ resetWindow: true });
    onCleanup(() => {
      if (viewportMeta) {
        if (previousViewport === null) {
          viewportMeta.removeAttribute("content");
        } else {
          viewportMeta.setAttribute("content", previousViewport);
        }
      }
      document.removeEventListener("gesturestart", preventGesture);
      document.removeEventListener("gesturechange", preventGesture);
      document.removeEventListener("gestureend", preventGesture);
    });
  });
  createEffect(() => {
    const locked = authState() === "ready";
    document.documentElement.classList.toggle("public-chat-scroll-lock", locked);
    document.body.classList.toggle("public-chat-scroll-lock", locked);
  });
  onCleanup(() => {
    sessionSyncGeneration += 1;
    activeController?.abort();
    restoreController?.abort();
    clearRestoreRetry();
    if (justFinishedTimer !== undefined) window.clearTimeout(justFinishedTimer);
    document.documentElement.classList.remove("public-chat-scroll-lock");
    document.body.classList.remove("public-chat-scroll-lock");
  });

  const handleSend = async () => {
    const text = draft().trim();
    const atts = [...pendingAttachments];
    if (
      (!text && atts.length === 0) ||
      authState() !== "ready" ||
      isSending() ||
      uploading()
    )
      return;

    const assistantId = messageId();
    setDraft("");
    setIsSending(true);
    // Send action is an explicit user intent to follow the new content.
    stickToBottom = true;
    setMessages(messages.length, {
      id: messageId(),
      role: "user",
      content: text,
      attachments: atts,
    });
    setMessages(messages.length, {
      id: assistantId,
      role: "assistant",
      content: "",
      phase: "thinking",
      statusText: CONTENT.chat_page.status.thinking,
      startedAt: Date.now(),
      steps: [],
    });
    // Keep all existing + new messages in view; never shrink the visible window.
    setVisibleMessageCount((c) => Math.max(c + 2, HISTORY_PAGE_SIZE));
    setPendingAttachments(reconcile([], { key: "path" }));
    scrollToBottom();

    const controller = new AbortController();
    activeController = controller;
    try {
      const stream = await sendPublicChat(
        text,
        atts.map((a) => ({ path: a.path, name: a.name })),
        controller.signal,
      );
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
            const index = messages.findIndex((m) => m.id === assistantId);
            if (index >= 0) {
              setMessages(index, {
                content: messages[index].content + (ev.data.content ?? ""),
                phase: "streaming",
              });
            }
            if (stickToBottom) scrollToBottom();
          }
          if (ev.event === "run_finished") {
            const index = messages.findIndex((m) => m.id === assistantId);
            if (index >= 0) setMessages(index, "phase", "done");
            pinToBottom(1400);
            flashJustFinished();
          }
        }
      }
    } catch (e) {
      const index = messages.findIndex((m) => m.id === assistantId);
      if (index >= 0)
        setMessages(index, { phase: "error", statusText: String(e) });
    } finally {
      const shouldStayAtBottom =
        stickToBottom || isBottomPinned() || distanceFromBottom() < 160;
      if (shouldStayAtBottom) pinToBottom(1600);
      setIsSending(false);
      flashJustFinished();
      void restoreSession({ keepAtBottom: shouldStayAtBottom });
    }
  };

  return (
    <div
      class={`hone-landing-v4 public-chat-page public-chat-page--${authState()}`}
      style={{ height: "100dvh", display: "flex", "flex-direction": "column" }}
    >
      <AnimatedBackground />
      <PublicNav
        extraActions={
          <>
            <PrefsButton />
            <AccountButton user={currentUser()} onLogout={logoutPublicChat} />
          </>
        }
      />

      <Switch>
        <Match when={authState() === "loading"}>
          <LoadingCard
            status={restoreStatus()}
            onRetry={() =>
              restoreSession({
                resetWindow: true,
                retryOnFailure: true,
                attempt: 1,
              })
            }
          />
        </Match>
        <Match when={authState() === "logged_out"}>
          <PublicLoginForm
            onLogin={() => restoreSession({ resetWindow: true })}
          />
        </Match>
        <Match when={authState() === "ready"}>
          <Show
            when={currentUser()}
            fallback={
              <LoadingCard
                status={restoreStatus()}
                onRetry={() =>
                  restoreSession({
                    resetWindow: true,
                    retryOnFailure: true,
                    attempt: 1,
                  })
                }
              />
            }
          >
            {(user) => (
              <>
                <ChatSidebar
                  user={user()}
                  collapsed={sidebarCollapsed()}
                  recentMessages={sidebarHistoryMessages()}
                  onToggle={() => setSidebarCollapsed((value) => !value)}
                  onSelectMessage={scrollToMessage}
                  onLogout={logoutPublicChat}
                />
                <div
                  class="public-chat-shell"
                  style={{
                    flex: "1",
                    display: "flex",
                    "flex-direction": "column",
                    "padding-top": "80px",
                    position: "relative",
                    "z-index": "10",
                    overflow: "hidden",
                  }}
                >
                {/* Session actions */}
                <div
                  class="public-chat-session-strip"
                  style={{
                    display: "flex",
                    "justify-content": "center",
                    padding: "12px",
                  }}
                >
                  <div
                    style={{
                      background: "rgba(255,255,255,0.7)",
                      "backdrop-filter": "blur(10px)",
                      padding: "6px 20px",
                      "border-radius": "100px",
                      border: "1.5px solid #f1f5f9",
                      display: "flex",
                      gap: "14px",
                      "align-items": "center",
                      "font-size": "13px",
                      "font-weight": "700",
                    }}
                  >
                    <button
                      onClick={() => navigate("/me")}
                      style={{
                        border: "none",
                        background: "none",
                        cursor: "pointer",
                        color: "#000",
                        "font-weight": "800",
                      }}
                    >
                      {sessionInfo()?.userId}
                    </button>
                    <button
                      onClick={logoutPublicChat}
                      style={{
                        border: "none",
                        background: "none",
                        cursor: "pointer",
                        color: "#ef4444",
                      }}
                    >
                      {CONTENT.chat_page.actions.logout}
                    </button>
                  </div>
                </div>

                {/* Message List */}
                <div
                  ref={scrollRef}
                  class="public-chat-messages"
                  onScroll={handleMessagesScroll}
                  style={{ flex: "1", "overflow-y": "auto", padding: "20px 0" }}
                >
                  <div
                    ref={messagesInnerRef}
                    style={{
                      "max-width": "900px",
                      margin: "0 auto",
                      padding: "0 24px",
                    }}
                  >
                    <Show when={hasOlderMessages()}>
                      <div
                        style={{
                          "text-align": "center",
                          color: "#94a3b8",
                          "font-size": "12px",
                          "font-weight": "700",
                          padding: "4px 0 18px",
                        }}
                      >
                        {loadingOlderMessages()
                          ? CONTENT.chat_page.history.loading_older
                          : CONTENT.chat_page.history.load_older}
                      </div>
                    </Show>
                    <For each={visibleMessages()}>
                      {(msg, i) => (
                        <div id={`public-chat-message-${msg.id}`}>
                          <Switch>
                            <Match when={msg.role === "user"}>
                              <UserBubble
                                content={msg.content}
                                attachments={msg.attachments}
                                onOpenImage={(imgs, i) =>
                                  setLightbox({ images: imgs, index: i })
                                }
                              />
                            </Match>
                            <Match
                              when={
                                msg.role === "assistant" && msg.phase === "done"
                              }
                            >
                              <AssistantBubble
                                content={msg.content}
                                attachments={msg.attachments}
                                isContinuation={
                                  i() > 0 &&
                                  visibleMessages()[i() - 1]?.role ===
                                    "assistant"
                                }
                                onShare={() => openShareModal(i())}
                              />
                            </Match>
                            <Match
                              when={
                                msg.role === "assistant" &&
                                msg.phase !== "done" &&
                                (msg.content || msg.phase === "error")
                              }
                            >
                              <PendingBubble
                                message={msg}
                                onStop={() => activeController?.abort()}
                                onDismiss={() => {}}
                              />
                            </Match>
                          </Switch>
                        </div>
                      )}
                    </For>
                  </div>
                </div>

                <div style={{ position: "relative" }}>
                  <Show when={awayFromBottom()}>
                    <button
                      type="button"
                      class="public-chat-scroll-down"
                      aria-label={CONTENT.chat_page.actions.scroll_to_bottom_aria}
                      title={CONTENT.chat_page.actions.scroll_to_bottom_aria}
                      onClick={settleAtBottom}
                    >
                      <svg
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2.4"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        aria-hidden="true"
                      >
                        <path d="M12 5v14M19 12l-7 7-7-7" />
                      </svg>
                    </button>
                  </Show>
                  <Composer
                    draft={draft()}
                    onDraftChange={setDraft}
                    attachments={pendingAttachments}
                    onRemoveAttachment={(i) =>
                      setPendingAttachments(
                        pendingAttachments.filter((_, j) => j !== i),
                      )
                    }
                    onPickFiles={async (files) => {
                      setUploading(true);
                      try {
                        const uploaded = await uploadPublicAttachments(files);
                        setPendingAttachments([
                          ...pendingAttachments,
                          ...uploaded.map((u) => ({
                            ...u,
                            kind: u.kind as any,
                          })),
                        ]);
                      } finally {
                        setUploading(false);
                      }
                    }}
                    uploading={uploading()}
                    onSend={handleSend}
                    onStop={() => activeController?.abort()}
                    isSending={isSending()}
                    remaining={sessionInfo()?.remainingToday}
                    dailyLimit={sessionInfo()?.dailyLimit}
                    pendingMessage={composerPendingMessage()}
                    justFinished={justFinished()}
                  />
                </div>
              </div>
              </>
            )}
          </Show>
        </Match>
      </Switch>

      <Show when={lightbox()}>
        <div class="lightbox-overlay" onClick={() => setLightbox(null)}>
          <img
            src={publicAttachmentUrl(lightbox()!.images[lightbox()!.index]!)}
            class="lightbox-img"
          />
          <button class="lightbox-close">×</button>
        </div>
      </Show>

      <ChatShareModal
        open={shareSeed() !== null}
        messages={visibleMessages()}
        seedIndex={shareSeed() ?? 0}
        brandName={CONTENT.chat_page.share.brand_name}
        brandTagline={CONTENT.chat_page.share.brand_tagline}
        qrUrl="https://hone-claw.com/chat"
        qrCaption={CONTENT.chat_page.share.qr_caption}
        strings={CONTENT.chat_page.share.strings}
        onClose={closeShareModal}
      />

      <style>{`
        html.public-chat-scroll-lock,
        body.public-chat-scroll-lock,
        body.public-chat-scroll-lock #root {
          width: 100% !important;
          height: 100dvh !important;
          min-height: 100dvh !important;
          max-height: 100dvh !important;
          overflow: hidden !important;
          overscroll-behavior: none;
          -webkit-text-size-adjust: 100%;
          text-size-adjust: 100%;
        }
        body.public-chat-scroll-lock {
          position: fixed;
          inset: 0;
        }
        body.public-chat-scroll-lock #root {
          position: relative;
        }
        .public-chat-page {
          --font-sans: "Plus Jakarta Sans", "Inter", -apple-system, BlinkMacSystemFont, "PingFang SC", "Microsoft YaHei", "Segoe UI", sans-serif;
          width: 100vw;
          height: 100dvh !important;
          max-height: 100dvh;
          overflow: hidden;
          overflow-anchor: none;
          overscroll-behavior: none;
          font-family: var(--font-sans);
          -webkit-font-smoothing: antialiased;
          text-rendering: optimizeLegibility;
        }
        .public-chat-page button,
        .public-chat-page textarea,
        .public-chat-page input {
          font-family: inherit;
        }
        .public-chat-page--logged_out,
        .public-chat-page--loading {
          height: auto !important;
          min-height: 100dvh !important;
          max-height: none !important;
          overflow-x: hidden !important;
          overflow-y: auto !important;
          overscroll-behavior-y: contain;
          -webkit-overflow-scrolling: touch;
        }
        .public-chat-shell {
          height: 100%;
          min-height: 0;
          overflow-anchor: none;
        }
        .public-chat-header-brand {
          display: flex;
          align-items: center;
          gap: 8px;
          min-width: 0;
        }
        .public-chat-account {
          position: relative;
          display: inline-flex;
        }
        .public-chat-account-trigger {
          width: 32px;
          height: 32px;
          border: 1px solid rgba(15,23,42,0.10);
          border-radius: 999px;
          background: rgba(255,255,255,0.82);
          color: #334155;
          display: inline-flex;
          align-items: center;
          justify-content: center;
          cursor: pointer;
          box-shadow: 0 1px 3px rgba(15,23,42,0.08);
          transition: background 0.16s ease, border-color 0.16s ease, color 0.16s ease, transform 0.06s ease;
        }
        .public-chat-account-trigger:active {
          transform: scale(0.96);
        }
        .public-chat-account-panel {
          position: fixed;
          top: calc(54px + env(safe-area-inset-top, 0px));
          left: 12px;
          z-index: 220;
          width: min(280px, calc(100vw - 24px));
          padding: 12px;
          border: 1px solid rgba(15,23,42,0.10);
          border-radius: 16px;
          background: rgba(255,255,255,0.98);
          box-shadow: 0 20px 52px rgba(15,23,42,0.18);
          backdrop-filter: blur(18px);
          -webkit-backdrop-filter: blur(18px);
        }
        .pub-nav-extra-actions .public-chat-account-panel {
          position: absolute;
          top: calc(100% + 10px);
          right: 0;
          left: auto;
        }
        .public-chat-account-card {
          display: flex;
          align-items: center;
          gap: 10px;
          min-width: 0;
        }
        .public-chat-account-avatar {
          width: 38px;
          height: 38px;
          flex: 0 0 38px;
          border-radius: 12px;
          background: #0f172a;
          color: #fff;
          display: inline-flex;
          align-items: center;
          justify-content: center;
          font-size: 15px;
          font-weight: 850;
        }
        .public-chat-account-card strong,
        .public-chat-account-card small {
          display: block;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
        .public-chat-account-card strong {
          color: #0f172a;
          font-size: 13px;
          font-weight: 850;
        }
        .public-chat-account-card small {
          margin-top: 2px;
          color: #64748b;
          font-size: 12px;
          font-weight: 650;
        }
        .public-chat-account-meta {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 12px;
          margin-top: 12px;
          padding: 10px 12px;
          border-radius: 12px;
          background: #f8fafc;
          color: #64748b;
          font-size: 12px;
          font-weight: 700;
        }
        .public-chat-account-meta strong {
          color: #0f172a;
          font-size: 13px;
          font-weight: 850;
        }
        .public-chat-account-center,
        .public-chat-account-logout {
          width: 100%;
          min-height: 40px;
          margin-top: 10px;
          border-radius: 12px;
          cursor: pointer;
          font-size: 13px;
          font-weight: 850;
          font-family: inherit;
        }
        .public-chat-account-center {
          border: 1px solid rgba(15,23,42,0.10);
          background: #fff;
          color: #0f172a;
        }
        .public-chat-account-logout {
          border: 1px solid rgba(225,29,72,0.14);
          background: #fff1f2;
          color: #e11d48;
        }
        @media (min-width: 769px) {
          .public-chat-page--ready {
            flex-direction: row !important;
            background: #f8fafc;
          }
          .public-chat-page--ready > .page-header {
            display: none !important;
          }
          .public-chat-page--ready .public-chat-shell {
            height: 100dvh !important;
            padding-top: 56px !important;
            background: rgba(248,250,252,0.72);
          }
          .public-chat-page--ready .public-chat-session-strip {
            display: none !important;
          }
          .public-chat-page--ready .public-chat-messages {
            padding-top: 24px !important;
          }
          .public-chat-sidebar {
            position: relative;
            z-index: 20;
            padding-top: 56px;
            width: 292px;
            height: 100dvh;
            flex: 0 0 292px;
            display: flex;
            flex-direction: column;
            gap: 14px;
            padding: 14px 12px;
            background: rgba(255,255,255,0.88);
            border-right: 1px solid rgba(15,23,42,0.08);
            box-shadow: 10px 0 34px rgba(15,23,42,0.05);
            backdrop-filter: blur(20px);
            -webkit-backdrop-filter: blur(20px);
            transition: width 0.18s ease, flex-basis 0.18s ease;
            overflow: hidden;
          }
          .public-chat-sidebar.is-collapsed {
            width: 72px;
            flex-basis: 72px;
            align-items: center;
            padding-right: 10px;
            padding-left: 10px;
          }
          .public-chat-sidebar-brand {
            display: flex;
            align-items: center;
            justify-content: space-between;
            min-height: 62px;
            gap: 8px;
          }
          .public-chat-sidebar-logo {
            min-width: 0;
            min-height: 58px;
            border: 1px solid rgba(15,23,42,0.08);
            border-radius: 14px;
            background: #fff;
            display: inline-flex;
            align-items: center;
            gap: 12px;
            cursor: pointer;
            padding: 6px 10px;
            color: #0f172a;
            font-size: 21px;
            font-weight: 900;
            letter-spacing: 0;
            box-shadow: 0 8px 22px rgba(15,23,42,0.06);
            transition: border-color 0.18s ease, transform 0.06s ease, box-shadow 0.18s ease;
          }
          .public-chat-sidebar-logo:hover {
            border-color: rgba(15,23,42,0.16);
            box-shadow: 0 10px 26px rgba(15,23,42,0.08);
          }
          .public-chat-sidebar-logo:active {
            transform: scale(0.99);
          }
          .public-chat-sidebar-logo img {
            width: 44px;
            height: 44px;
            border-radius: 10px;
            flex: 0 0 44px;
          }
          .public-chat-sidebar-toggle,
          .public-chat-sidebar-lang {
            width: 36px;
            height: 36px;
            min-width: 36px;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            border: 1px solid rgba(15,23,42,0.10);
            border-radius: 10px;
            background: rgba(255,255,255,0.78);
            color: #475569;
            cursor: pointer;
            box-shadow: none;
            transition: background 0.18s ease, border-color 0.18s ease, color 0.18s ease, transform 0.06s ease;
          }
          .public-chat-sidebar-toggle:hover,
          .public-chat-sidebar-lang:hover {
            color: #0f172a;
            background: #f8fafc;
            border-color: #cbd5e1;
          }
          .public-chat-sidebar-nav {
            display: grid;
            gap: 4px;
          }
          .public-chat-sidebar-nav button,
          .public-chat-sidebar-star {
            width: 100%;
            min-height: 40px;
            display: flex;
            align-items: center;
            gap: 10px;
            border: 1px solid transparent;
            border-radius: 10px;
            background: transparent;
            color: #475569;
            cursor: pointer;
            padding: 0 10px;
            font-size: 13px;
            font-weight: 750;
            text-decoration: none;
            letter-spacing: 0;
            transition: background 0.18s ease, border-color 0.18s ease, color 0.18s ease, transform 0.06s ease;
          }
          .public-chat-sidebar-nav button:hover,
          .public-chat-sidebar-star:hover {
            background: #f1f5f9;
            border-color: rgba(15,23,42,0.08);
            color: #0f172a;
          }
          .public-chat-sidebar-nav button.is-active {
            background: #0f172a;
            border-color: #0f172a;
            color: #fff;
          }
          .public-chat-sidebar-nav svg,
          .public-chat-sidebar-star svg,
          .public-chat-sidebar-icon {
            width: 22px;
            height: 22px;
            flex: 0 0 22px;
          }
          .public-chat-sidebar-icon {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            border-radius: 6px;
            background: rgba(15,23,42,0.06);
            font-size: 12px;
            font-weight: 850;
          }
          .public-chat-sidebar-history {
            min-height: 0;
            overflow: auto;
            padding: 6px 2px 0;
            flex: 1 1 auto;
          }
          .public-chat-sidebar-section-title {
            margin: 0 6px 8px;
            color: #94a3b8;
            font-size: 12px;
            font-weight: 850;
            letter-spacing: 0.08em;
            text-transform: uppercase;
          }
          .public-chat-sidebar-history-empty {
            margin: 0 6px;
            padding: 12px 10px;
            border: 1px dashed rgba(15,23,42,0.12);
            border-radius: 12px;
            color: #94a3b8;
            font-size: 12px;
            font-weight: 650;
            line-height: 1.5;
          }
          .public-chat-sidebar-history-list {
            display: grid;
            gap: 7px;
          }
          .public-chat-sidebar-history-item {
            width: 100%;
            min-height: 42px;
            display: flex;
            align-items: center;
            gap: 9px;
            padding: 8px 10px;
            border: 1px solid rgba(15,23,42,0.06);
            border-radius: 10px;
            background: #f8fafc;
            color: #334155;
            cursor: pointer;
            text-align: left;
            transition: background 0.18s ease, border-color 0.18s ease, color 0.18s ease, transform 0.06s ease;
          }
          .public-chat-sidebar-history-item:hover {
            background: #fff;
            border-color: rgba(245,158,11,0.28);
            color: #0f172a;
          }
          .public-chat-sidebar-history-item:active {
            transform: scale(0.99);
          }
          .public-chat-sidebar-history-index {
            width: 22px;
            height: 22px;
            flex: 0 0 22px;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            border-radius: 7px;
            background: rgba(245,158,11,0.12);
            color: #b45309;
            font-size: 11px;
            font-weight: 850;
            font-variant-numeric: tabular-nums;
          }
          .public-chat-sidebar-history-text {
            min-width: 0;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            font-size: 12.5px;
            font-weight: 700;
            line-height: 1.3;
          }
          .public-chat-sidebar-footer {
            margin-top: auto;
            display: grid;
            gap: 10px;
            padding-top: 12px;
            border-top: 1px solid rgba(15,23,42,0.08);
          }
          .public-chat-sidebar-user {
            min-width: 0;
            display: flex;
            align-items: center;
            gap: 10px;
            width: 100%;
            padding: 0;
            border: 0;
            background: transparent;
            cursor: pointer;
            text-align: left;
          }
          .public-chat-sidebar-avatar {
            width: 36px;
            height: 36px;
            flex: 0 0 36px;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            border-radius: 10px;
            background: #0f172a;
            color: #fff;
            font-size: 14px;
            font-weight: 850;
          }
          .public-chat-sidebar-user strong,
          .public-chat-sidebar-user small {
            display: block;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
          }
          .public-chat-sidebar-user strong {
            color: #0f172a;
            font-size: 13px;
            font-weight: 850;
          }
          .public-chat-sidebar-user small {
            margin-top: 2px;
            color: #64748b;
            font-size: 12px;
            font-weight: 650;
          }
          .public-chat-sidebar-footer-actions {
            display: flex;
            align-items: center;
            gap: 8px;
          }
          .public-chat-sidebar-logout {
            flex: 1;
            min-height: 36px;
            border: 1px solid rgba(225,29,72,0.12);
            border-radius: 10px;
            background: #fff1f2;
            color: #e11d48;
            cursor: pointer;
            font-size: 13px;
            font-weight: 800;
            transition: background 0.18s ease, border-color 0.18s ease, transform 0.06s ease;
          }
          .public-chat-sidebar-logout:hover {
            background: #ffe4e6;
            border-color: rgba(225,29,72,0.22);
          }
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-logo span,
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-nav span:not(.public-chat-sidebar-icon),
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-star span,
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-history,
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-section-title,
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-user span:not(.public-chat-sidebar-avatar),
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-logout span {
            display: none !important;
          }
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-brand,
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-footer-actions {
            flex-direction: column;
          }
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-nav button,
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-star {
            width: 42px;
            justify-content: center;
            padding: 0;
          }
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-logo {
            width: 42px;
            min-height: 42px;
            justify-content: center;
            padding: 0;
            border-radius: 12px;
          }
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-logo img {
            width: 32px;
            height: 32px;
            flex-basis: 32px;
          }
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-footer {
            width: 42px;
          }
          .public-chat-sidebar.is-collapsed .public-chat-sidebar-logout {
            flex: 0 0 36px;
            width: 36px;
          }
        }
        @media (max-width: 768px) {
          .public-chat-sidebar {
            display: none !important;
          }
        }
        .public-chat-messages {
          min-height: 0;
          overscroll-behavior: contain;
          -webkit-overflow-scrolling: touch;
          overflow-anchor: none;
          touch-action: pan-y;
        }
        .public-chat-composer {
          touch-action: manipulation;
        }
        .public-chat-composer *,
        .public-chat-composer-input {
          touch-action: manipulation;
        }
        .public-chat-composer-input {
          touch-action: pan-y;
          overflow-x: hidden;
        }
        @media (max-width: 768px) {
          .public-chat-page--logged_out .public-login-screen {
            min-height: 100svh !important;
            height: auto !important;
            align-items: flex-start !important;
            justify-content: flex-start !important;
            padding: 58px 0 max(18px, env(safe-area-inset-bottom)) !important;
            overflow: visible !important;
          }
          .public-chat-page--logged_out .public-login-card-wrap {
            max-width: none !important;
            padding: 0 14px !important;
          }
          .public-chat-page--logged_out .public-login-card {
            padding: 18px !important;
            border-radius: 12px !important;
          }
          .public-chat-page--logged_out .public-login-code-row {
            gap: 8px !important;
          }
          .public-chat-page--logged_out .public-login-code-button {
            width: 104px !important;
            font-size: 12px !important;
          }
          .public-chat-page input,
          .public-chat-page textarea,
          .public-chat-page select {
            font-size: 16px !important;
          }
        }
        .public-chat-composer-status {
          max-width: 900px;
          margin: 0 auto 8px;
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 6px 14px;
          background: rgba(255,255,255,0.94);
          backdrop-filter: blur(10px);
          border: 1.5px solid #e2e8f0;
          border-radius: 14px;
          box-shadow: 0 6px 18px rgba(15,23,42,0.06);
          font-size: 13px;
          font-weight: 700;
          color: #334155;
        }
        .public-chat-composer-status.is-done {
          color: #047857;
          border-color: rgba(16,185,129,0.3);
          background: rgba(236,253,245,0.96);
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
        .public-chat-composer-status-time {
          margin-left: auto;
          font-family: var(--font-mono, 'JetBrains Mono', monospace);
          font-size: 12px;
          color: #64748b;
          font-variant-numeric: tabular-nums;
        }
        .public-chat-composer-status-stop {
          background: #0f172a;
          color: #fff;
          border: none;
          padding: 4px 12px;
          border-radius: 999px;
          font-size: 11px;
          font-weight: 700;
          cursor: pointer;
          transition: background 0.2s;
        }
        .public-chat-composer-status-stop:hover { background: #ef4444; }

        .public-chat-proactive-tip-wrap {
          max-width: 900px;
          margin: 0 auto 8px;
          display: flex;
          align-items: center;
        }
        .public-chat-proactive-tip {
          display: inline-flex;
          align-items: center;
          gap: 7px;
          min-height: 28px;
          max-width: 100%;
          padding: 5px 10px;
          border: 1px solid rgba(15,23,42,0.08);
          border-radius: 999px;
          background: rgba(255,255,255,0.82);
          color: #475569;
          box-shadow: 0 6px 18px rgba(15,23,42,0.05);
          backdrop-filter: blur(12px);
          -webkit-backdrop-filter: blur(12px);
          cursor: pointer;
          font-size: 12.5px;
          font-weight: 700;
          line-height: 1.2;
          letter-spacing: 0;
          transition: background 0.18s, color 0.18s, border-color 0.18s, box-shadow 0.18s;
        }
        .public-chat-proactive-tip:hover,
        .public-chat-proactive-tip[aria-expanded="true"] {
          background: #fff;
          color: #0f172a;
          border-color: rgba(245,158,11,0.35);
          box-shadow: 0 8px 22px rgba(15,23,42,0.08);
        }
        .public-chat-proactive-tip-icon {
          width: 15px;
          height: 15px;
          flex: 0 0 15px;
          color: #d97706;
          stroke-width: 2.2;
        }
        .public-chat-proactive-modal-backdrop {
          position: fixed;
          inset: 0;
          z-index: 1000;
          display: flex;
          align-items: center;
          justify-content: center;
          padding: 20px;
          background: rgba(15,23,42,0.22);
          backdrop-filter: blur(10px);
          -webkit-backdrop-filter: blur(10px);
          animation: hone-proactive-backdrop 0.14s ease-out;
        }
        @keyframes hone-proactive-backdrop {
          from { opacity: 0; }
          to   { opacity: 1; }
        }
        .public-chat-proactive-modal {
          position: relative;
          width: min(440px, calc(100vw - 32px));
          max-height: min(720px, calc(100dvh - 40px));
          overflow: auto;
          padding: 22px;
          border: 1px solid rgba(15,23,42,0.08);
          border-radius: 18px;
          background: rgba(255,255,255,0.98);
          color: #0f172a;
          box-shadow: 0 26px 80px rgba(15,23,42,0.22);
          animation: hone-proactive-pop 0.16s ease-out;
        }
        @keyframes hone-proactive-pop {
          from { opacity: 0; transform: translateY(8px) scale(0.98); }
          to   { opacity: 1; transform: translateY(0) scale(1); }
        }
        .public-chat-proactive-modal h2 {
          margin: 0 36px 8px 0;
          font-size: 22px;
          line-height: 1.25;
          letter-spacing: 0;
          color: #0f172a;
        }
        .public-chat-proactive-intro {
          margin: 0 0 16px;
          color: #475569;
          font-size: 14px;
          line-height: 1.65;
        }
        .public-chat-proactive-close {
          position: absolute;
          top: 14px;
          right: 14px;
          width: 30px;
          height: 30px;
          display: inline-flex;
          align-items: center;
          justify-content: center;
          border: none;
          border-radius: 999px;
          background: #f8fafc;
          color: #64748b;
          cursor: pointer;
          transition: background 0.16s, color 0.16s;
        }
        .public-chat-proactive-close:hover { background: #f1f5f9; color: #0f172a; }
        .public-chat-proactive-list {
          display: grid;
          gap: 12px;
          margin: 0 0 16px;
        }
        .public-chat-proactive-item {
          display: grid;
          grid-template-columns: 8px 1fr;
          gap: 10px;
          align-items: start;
        }
        .public-chat-proactive-item-mark {
          width: 8px;
          height: 8px;
          margin-top: 7px;
          border-radius: 999px;
          background: #f59e0b;
        }
        .public-chat-proactive-item strong {
          display: block;
          margin: 0 0 3px;
          font-size: 14px;
          line-height: 1.35;
          color: #0f172a;
        }
        .public-chat-proactive-item small {
          display: block;
          color: #64748b;
          font-size: 13px;
          line-height: 1.55;
        }
        .public-chat-proactive-examples {
          display: grid;
          gap: 7px;
          margin: 0 0 18px;
          padding: 12px;
          border-radius: 12px;
          background: #f8fafc;
        }
        .public-chat-proactive-examples div {
          color: #475569;
          font-size: 12px;
          font-weight: 800;
          line-height: 1.3;
        }
        .public-chat-proactive-example-row {
          display: grid;
          grid-template-columns: 24px 1fr;
          align-items: start;
          gap: 7px;
          color: #334155;
          font-size: 12.5px;
          line-height: 1.45;
        }
        .public-chat-proactive-example-row span {
          color: #334155;
          font-size: 12.5px;
          line-height: 1.45;
        }
        .public-chat-proactive-copy {
          width: 22px;
          height: 22px;
          display: inline-flex;
          align-items: center;
          justify-content: center;
          border: 1px solid rgba(15,23,42,0.08);
          border-radius: 7px;
          background: #fff;
          color: #64748b;
          cursor: pointer;
          transition: background 0.16s ease, border-color 0.16s ease, color 0.16s ease, transform 0.06s ease;
        }
        .public-chat-proactive-copy:hover {
          background: rgba(245,158,11,0.10);
          border-color: rgba(245,158,11,0.24);
          color: #b45309;
        }
        .public-chat-proactive-copy:active {
          transform: scale(0.96);
        }
        .public-chat-proactive-primary {
          width: 100%;
          min-height: 40px;
          border: none;
          border-radius: 12px;
          background: #0f172a;
          color: #fff;
          cursor: pointer;
          font-size: 14px;
          font-weight: 800;
          letter-spacing: 0;
        }
        /* Tone down the homepage's animated background blobs on the chat
           page — three near-white surfaces (page bg + bubble + ticker chip)
           competing for attention reads as visual noise, so the gradient
           goes to a near-invisible tint. */
        .public-chat-page .animated-bg .circle { opacity: 0.08; filter: blur(80px); }
        .public-chat-messages .public-chat-markdown {
          color: #1e293b;
          font-size: 16px;
          line-height: 1.75;
          white-space: normal;
        }
        .public-chat-messages .public-chat-markdown * {
          max-width: 100%;
        }
        .public-chat-messages .public-chat-markdown h1 {
          font-size: 1.35em;
          line-height: 1.35;
          margin: 1.2em 0 0.45em;
        }
        .public-chat-messages .public-chat-markdown h2 {
          font-size: 1.18em;
          line-height: 1.4;
          margin: 1.15em 0 0.45em;
        }
        .public-chat-messages .public-chat-markdown h3,
        .public-chat-messages .public-chat-markdown h4 {
          font-size: 1.05em;
          line-height: 1.45;
          margin: 1em 0 0.35em;
        }
        .public-chat-messages .public-chat-markdown p {
          margin: 0.72em 0;
        }
        .public-chat-messages .public-chat-markdown strong {
          color: #0f172a;
          font-weight: 800;
        }
        .public-chat-messages .public-chat-markdown ul,
        .public-chat-messages .public-chat-markdown ol {
          margin: 0.72em 0;
          padding-left: 1.45em;
          list-style-position: outside;
        }
        .public-chat-messages .public-chat-markdown ul {
          list-style-type: disc;
        }
        .public-chat-messages .public-chat-markdown ol {
          list-style-type: decimal;
        }
        .public-chat-messages .public-chat-markdown ul ul {
          list-style-type: circle;
        }
        .public-chat-messages .public-chat-markdown ul ul ul {
          list-style-type: square;
        }
        .public-chat-messages .public-chat-markdown li {
          margin: 0.32em 0;
          padding-left: 0.12em;
        }
        .public-chat-messages .public-chat-markdown li > p {
          margin: 0.35em 0;
        }
        .public-chat-messages .public-chat-markdown li > ul,
        .public-chat-messages .public-chat-markdown li > ol {
          margin: 0.35em 0 0.5em;
        }
        .public-chat-messages .public-chat-markdown blockquote {
          margin: 1em 0;
          border-left: 4px solid rgba(15,23,42,0.12);
          padding-left: 1em;
          color: #64748b;
        }
        .public-chat-messages .public-chat-markdown :not(pre) > code {
          border-radius: 6px;
          background: rgba(15,23,42,0.06);
          padding: 0.12em 0.36em;
          font-size: 0.92em;
          font-family: var(--font-mono, "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace);
        }
        .public-chat-messages .public-chat-markdown > :first-child {
          margin-top: 0;
        }
        .public-chat-messages .public-chat-markdown > :last-child {
          margin-bottom: 0;
        }
        .public-chat-messages .public-chat-markdown--white,
        .public-chat-messages .public-chat-markdown--white * {
          color: #fff !important;
        }
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
        /* Shiki code block: a single flat surface — the wrapper div is
           invisible, the visible chip is the <pre> itself. Soft gray-50
           background, no border (lets it sit gently against the bubble
           without looking like a stacked card), small radius, wrapped lines
           so long PE-style formulas don't spawn nested scrollbars. */
        .public-chat-messages .hf-markdown .hf-markdown-code {
          margin: 10px 0;
        }
        .public-chat-messages .hf-markdown .hf-markdown-code pre,
        .public-chat-messages .hf-markdown .hf-markdown-code pre.shiki {
          margin: 0 !important;
          padding: 10px 12px !important;
          background: #f3f4f6 !important;
          border: 0 !important;
          border-radius: 8px !important;
          font-size: 13.5px;
          line-height: 1.6;
          white-space: pre-wrap;
          word-break: break-word;
          overflow-wrap: anywhere;
        }
        .public-chat-messages .hf-markdown .hf-markdown-code code {
          background: transparent !important;
          padding: 0 !important;
          font-size: inherit !important;
          font-family: var(--font-mono, "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace);
        }
        [data-theme="dark"] .public-chat-messages .hf-markdown .hf-markdown-code pre,
        [data-theme="dark"] .public-chat-messages .hf-markdown .hf-markdown-code pre.shiki {
          background: #111827 !important;
        }
        [data-theme="dark"] .public-chat-messages .hf-markdown .hf-markdown-code code,
        [data-theme="dark"] .public-chat-messages .hf-markdown .hf-markdown-code span {
          color: #e5e7eb !important;
        }
        /* Action row (copy + share) sits in the bottom-right of an
           assistant bubble without overlaying the answer text. Desktop:
           faded until hover/focus. Mobile: visible at low-key opacity so
           long answers can be lifted out with one tap. */
        .pub-msg-actions {
          display: inline-flex;
          width: 100%;
          justify-content: flex-end;
          gap: 4px;
          margin-top: 10px;
          opacity: 0;
          pointer-events: none;
          transition: opacity 0.18s;
        }
        .pub-msg-bubble--assistant:hover .pub-msg-actions,
        .pub-msg-bubble--assistant:focus-within .pub-msg-actions {
          opacity: 1;
          pointer-events: auto;
        }
        .pub-msg-action {
          width: 28px;
          height: 28px;
          display: inline-flex;
          align-items: center;
          justify-content: center;
          border: none;
          border-radius: 999px;
          background: rgba(15, 23, 42, 0.04);
          color: #64748b;
          cursor: pointer;
          transition: background 0.18s, color 0.18s;
        }
        .pub-msg-action:hover { background: rgba(15, 23, 42, 0.08); color: #0f172a; }
        .pub-msg-action[data-copied="true"] {
          background: rgba(16, 185, 129, 0.12);
          color: #059669;
        }
        @media (hover: none), (max-width: 768px) {
          .pub-msg-actions {
            gap: 6px;
            opacity: 0.78;
            pointer-events: auto;
          }
          .pub-msg-action {
            width: 36px;
            height: 36px;
            background: rgba(15, 23, 42, 0.07);
          }
          .pub-msg-action svg {
            width: 18px;
            height: 18px;
          }
          .pub-msg-actions:active { opacity: 1; }
        }
        /* Scroll-to-bottom: floats above the composer when the user has
           scrolled up enough to lose track of the latest reply. */
        .public-chat-scroll-down {
          position: absolute;
          right: 16px;
          bottom: calc(100% + 8px);
          width: 36px;
          height: 36px;
          display: inline-flex;
          align-items: center;
          justify-content: center;
          border: none;
          border-radius: 999px;
          background: #0f172a;
          color: #fff;
          cursor: pointer;
          box-shadow: 0 6px 18px rgba(15, 23, 42, 0.22);
          z-index: 5;
          animation: hone-scroll-down-pop 0.16s ease-out;
        }
        @keyframes hone-scroll-down-pop {
          from { opacity: 0; transform: translateY(6px); }
          to   { opacity: 1; transform: translateY(0); }
        }
        .public-chat-composer-input::placeholder { color: #94a3b8; font-size: 14px; font-weight: 500; }
        /* Header right side: equalize visual heights so the lang pill and the
           对话 button look truly center-aligned, and trim some vertical bulk. */
        .public-chat-page .lang-switch { padding: 2px; }
        .public-chat-page .lang-switch button { min-height: 30px; }
        .public-chat-page .btn-chat-nav,
        .public-chat-page .btn-roadmap-nav { min-height: 36px; padding: 0 16px; }

        /* ── Prefs trigger + popover ─────────────────────────────────── */
        .hone-prefs { position: relative; display: inline-flex; }
        .hone-prefs-trigger {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          width: 36px; height: 36px;
          border-radius: 999px;
          border: 1px solid rgba(15,23,42,0.10);
          background: rgba(255,255,255,0.72);
          color: #64748b;
          cursor: pointer;
          transition: background 0.18s, border-color 0.18s, color 0.18s, transform 0.06s;
        }
        .hone-prefs-trigger:hover,
        .hone-prefs-trigger[aria-expanded="true"] { background: #f8fafc; border-color: #cbd5e1; color: #0f172a; }
        .hone-prefs-panel {
          position: absolute;
          right: 0; top: calc(100% + 10px);
          z-index: 999;
          width: 240px;
          padding: 10px 12px;
          background: rgba(255,255,255,0.96);
          backdrop-filter: blur(18px);
          -webkit-backdrop-filter: blur(18px);
          border: 1px solid rgba(15,23,42,0.08);
          border-radius: 14px;
          box-shadow: 0 12px 36px rgba(15,23,42,0.14);
          animation: hone-prefs-pop 0.14s ease-out;
        }
        @keyframes hone-prefs-pop {
          from { opacity: 0; transform: translateY(-4px) scale(0.98); }
          to   { opacity: 1; transform: translateY(0) scale(1); }
        }
        .hone-prefs-row {
          display: grid;
          grid-template-columns: 36px 1fr;
          align-items: center;
          gap: 10px;
          padding: 4px 0;
        }
        .hone-prefs-row + .hone-prefs-row { border-top: 1px solid rgba(15,23,42,0.06); }
        .hone-prefs-label {
          font-size: 12px;
          color: #64748b;
          font-weight: 600;
        }
        .hone-prefs-segmented {
          display: grid;
          grid-auto-flow: column;
          grid-auto-columns: 1fr;
          gap: 2px;
          padding: 2px;
          background: #f1f5f9;
          border-radius: 9px;
        }
        .hone-prefs-seg {
          border: none;
          background: transparent;
          color: #64748b;
          cursor: pointer;
          border-radius: 7px;
          padding: 5px 0;
          font-weight: 700;
          letter-spacing: 0;
          line-height: 1;
          display: inline-flex;
          align-items: center;
          justify-content: center;
          transition: background 0.15s, color 0.15s, box-shadow 0.15s;
        }
        .hone-prefs-seg.is-active {
          background: #fff;
          color: #0f172a;
          box-shadow: 0 1px 3px rgba(15,23,42,0.1), 0 1px 1px rgba(15,23,42,0.04);
        }
        .hone-prefs-seg[data-size="s"]  { font-size: 11px; }
        .hone-prefs-seg[data-size="m"]  { font-size: 13px; }
        .hone-prefs-seg[data-size="l"]  { font-size: 16px; }
        .hone-prefs-seg[data-size="xl"] { font-size: 19px; }
        .hone-prefs-seg--text { font-size: 12px; padding: 6px 0; }

        /* ── Font scale variants ─────────────────────────────────────── */
        /* Desktop baselines from the central markdown CSS are ~16px; the
           mobile @media block resets to 13.5px. We override both. */
        [data-chat-fs="s"]  .public-chat-messages .hf-markdown { font-size: 14.5px; }
        [data-chat-fs="m"]  .public-chat-messages .hf-markdown { font-size: 16px; }
        [data-chat-fs="l"]  .public-chat-messages .hf-markdown { font-size: 18.5px; line-height: 1.7; }
        [data-chat-fs="xl"] .public-chat-messages .hf-markdown { font-size: 22px; line-height: 1.75; }
        [data-chat-fs="l"]  .public-chat-messages .pub-msg-bubble--user { font-size: 18px; }
        [data-chat-fs="xl"] .public-chat-messages .pub-msg-bubble--user { font-size: 21px; }

        /* ── Dark theme ──────────────────────────────────────────────── */
        [data-theme="dark"] .public-chat-page { background: #0a0e16; }
        [data-theme="dark"] .public-chat-page .animated-bg .circle { opacity: 0.06 !important; }
        [data-theme="dark"] .page-header {
          background: rgba(10,14,22,0.85) !important;
          border-bottom-color: rgba(255,255,255,0.06) !important;
        }
        [data-theme="dark"] .header-logo span { color: #e5e7eb !important; }
        [data-theme="dark"] .public-chat-account-trigger {
          background: rgba(15,23,42,0.86);
          border-color: rgba(148,163,184,0.20);
          color: #cbd5e1;
        }
        [data-theme="dark"] .public-chat-account-panel {
          background: rgba(15,23,42,0.98);
          border-color: rgba(148,163,184,0.18);
        }
        [data-theme="dark"] .public-chat-account-card strong,
        [data-theme="dark"] .public-chat-account-meta strong {
          color: #f8fafc;
        }
        [data-theme="dark"] .public-chat-account-card small,
        [data-theme="dark"] .public-chat-account-meta {
          color: #94a3b8;
        }
        [data-theme="dark"] .public-chat-account-meta {
          background: rgba(30,41,59,0.82);
        }
        [data-theme="dark"] .public-chat-account-center {
          background: rgba(30,41,59,0.86);
          border-color: rgba(148,163,184,0.18);
          color: #f8fafc;
        }
        [data-theme="dark"] .public-chat-account-logout {
          background: rgba(127,29,29,0.28);
          border-color: rgba(248,113,113,0.22);
          color: #fda4af;
        }
        [data-theme="dark"] .lang-switch {
          background: rgba(255,255,255,0.04) !important;
          border-color: rgba(255,255,255,0.08) !important;
        }
        [data-theme="dark"] .lang-switch button { color: #94a3b8; }
        [data-theme="dark"] .lang-switch button.active { background: #f1f5f9 !important; color: #0a0e16 !important; }
        [data-theme="dark"] .btn-chat-nav { background: #f1f5f9 !important; color: #0a0e16 !important; }
        [data-theme="dark"] .btn-roadmap-nav { color: #94a3b8 !important; border-color: #1f2937 !important; }
        [data-theme="dark"] .icon-btn-ghost { color: #94a3b8; }
        [data-theme="dark"] .icon-btn-ghost:hover { background: rgba(255,255,255,0.06); color: #e5e7eb; }
        [data-theme="dark"] .star-badge { background: rgba(255,255,255,0.06); color: #cbd5e1; }
        [data-theme="dark"] .divider-v { background: rgba(255,255,255,0.08); }

        [data-theme="dark"] .hone-prefs-trigger { color: #94a3b8; }
        [data-theme="dark"] .hone-prefs-trigger:hover,
        [data-theme="dark"] .hone-prefs-trigger[aria-expanded="true"] {
          background: rgba(255,255,255,0.06); color: #fff;
        }
        [data-theme="dark"] .hone-prefs-panel {
          background: rgba(19,27,44,0.95);
          border-color: rgba(255,255,255,0.08);
          box-shadow: 0 16px 40px rgba(0,0,0,0.5);
        }
        [data-theme="dark"] .hone-prefs-row + .hone-prefs-row { border-top-color: rgba(255,255,255,0.06); }
        [data-theme="dark"] .hone-prefs-label { color: #94a3b8; }
        [data-theme="dark"] .hone-prefs-segmented { background: rgba(255,255,255,0.06); }
        [data-theme="dark"] .hone-prefs-seg { color: #94a3b8; }
        [data-theme="dark"] .hone-prefs-seg.is-active {
          background: #1f2937; color: #fff; box-shadow: none;
        }

        [data-theme="dark"] .public-chat-session-strip > div {
          background: rgba(19,27,44,0.7) !important;
          border-color: #1f2937 !important;
        }
        [data-theme="dark"] .public-chat-session-strip span,
        [data-theme="dark"] .public-chat-session-strip button { color: #cbd5e1 !important; }
        [data-theme="dark"] .public-chat-session-strip button[style*="ef4444"] { color: #f87171 !important; }

        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant {
          background: #131b2c !important;
          border-color: #1f2937 !important;
          box-shadow: 0 1px 6px rgba(0,0,0,0.25) !important;
        }
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant,
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant .hf-markdown,
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant .hf-markdown * { color: #e5e7eb !important; }
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant .hf-markdown strong,
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant .hf-markdown h1,
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant .hf-markdown h2,
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant .hf-markdown h3,
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant .hf-markdown h4 { color: #f8fafc !important; }
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant .hf-markdown a { color: #60a5fa !important; }
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant .hf-markdown code {
          background: rgba(255,255,255,0.08) !important; color: #f8fafc !important;
        }
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant .hf-markdown table { border-color: #1f2937 !important; }
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant .hf-markdown th,
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--assistant .hf-markdown td { border-color: #1f2937 !important; }
        [data-theme="dark"] .public-chat-messages .pub-msg-bubble--user {
          background: #1e293b !important;
          box-shadow: 0 1px 6px rgba(0,0,0,0.3) !important;
        }

        [data-theme="dark"] .public-chat-composer-box {
          background: #131b2c !important;
          border-color: #1f2937 !important;
          box-shadow: 0 4px 18px rgba(0,0,0,0.3) !important;
        }
        [data-theme="dark"] .public-chat-composer-input { color: #e5e7eb !important; }
        [data-theme="dark"] .public-chat-composer-input::placeholder { color: #64748b !important; }
        [data-theme="dark"] .pub-attach-btn { color: #94a3b8 !important; }
        [data-theme="dark"] .pub-attach-btn:hover { color: #e5e7eb !important; background: rgba(255,255,255,0.06) !important; }
        [data-theme="dark"] .public-chat-send-button[disabled] { background: #1f2937 !important; }
        [data-theme="dark"] .public-chat-send-button[disabled] svg { fill: #475569 !important; }

        [data-theme="dark"] .public-chat-composer-status {
          background: rgba(19,27,44,0.95) !important;
          border-color: #1f2937 !important;
          color: #cbd5e1 !important;
        }
        [data-theme="dark"] .public-chat-composer-status.is-done {
          background: rgba(6,78,59,0.45) !important;
          border-color: rgba(16,185,129,0.4) !important;
          color: #6ee7b7 !important;
        }
        [data-theme="dark"] .public-chat-composer-status-time { color: #94a3b8 !important; }
        [data-theme="dark"] .public-chat-composer-status-stop { background: #f1f5f9 !important; color: #0a0e16 !important; }
        [data-theme="dark"] .public-chat-composer-status-stop:hover { background: #ef4444 !important; color: #fff !important; }
        [data-theme="dark"] .public-chat-proactive-tip {
          background: rgba(19,27,44,0.82) !important;
          border-color: rgba(255,255,255,0.08) !important;
          color: #cbd5e1 !important;
          box-shadow: 0 8px 22px rgba(0,0,0,0.28) !important;
        }
        [data-theme="dark"] .public-chat-proactive-tip:hover,
        [data-theme="dark"] .public-chat-proactive-tip[aria-expanded="true"] {
          background: #131b2c !important;
          border-color: rgba(245,158,11,0.38) !important;
          color: #f8fafc !important;
        }
        [data-theme="dark"] .public-chat-proactive-modal-backdrop {
          background: rgba(2,6,23,0.52) !important;
        }
        [data-theme="dark"] .public-chat-proactive-modal {
          background: rgba(19,27,44,0.98) !important;
          border-color: rgba(255,255,255,0.08) !important;
          color: #e5e7eb !important;
          box-shadow: 0 28px 90px rgba(0,0,0,0.58) !important;
        }
        [data-theme="dark"] .public-chat-proactive-modal h2,
        [data-theme="dark"] .public-chat-proactive-item strong { color: #f8fafc !important; }
        [data-theme="dark"] .public-chat-proactive-intro,
        [data-theme="dark"] .public-chat-proactive-item small,
        [data-theme="dark"] .public-chat-proactive-examples div,
        [data-theme="dark"] .public-chat-proactive-example-row,
        [data-theme="dark"] .public-chat-proactive-example-row span { color: #cbd5e1 !important; }
        [data-theme="dark"] .public-chat-proactive-close {
          background: rgba(255,255,255,0.06) !important;
          color: #cbd5e1 !important;
        }
        [data-theme="dark"] .public-chat-proactive-close:hover {
          background: rgba(255,255,255,0.1) !important;
          color: #fff !important;
        }
        [data-theme="dark"] .public-chat-proactive-examples {
          background: rgba(255,255,255,0.05) !important;
        }
        [data-theme="dark"] .public-chat-proactive-copy {
          background: rgba(255,255,255,0.06) !important;
          border-color: rgba(255,255,255,0.08) !important;
          color: #cbd5e1 !important;
        }
        [data-theme="dark"] .public-chat-proactive-copy:hover {
          background: rgba(245,158,11,0.16) !important;
          border-color: rgba(245,158,11,0.32) !important;
          color: #fbbf24 !important;
        }
        [data-theme="dark"] .public-chat-proactive-primary {
          background: #f8fafc !important;
          color: #0a0e16 !important;
        }
        [data-theme="dark"] .public-chat-sidebar {
          background: rgba(10,14,22,0.9) !important;
          border-right-color: rgba(255,255,255,0.08) !important;
        }
        [data-theme="dark"] .public-chat-sidebar-logo {
          background: rgba(19,27,44,0.9) !important;
          border-color: rgba(255,255,255,0.08) !important;
          color: #f8fafc !important;
          box-shadow: 0 8px 22px rgba(0,0,0,0.24) !important;
        }
        [data-theme="dark"] .public-chat-sidebar-nav button,
        [data-theme="dark"] .public-chat-sidebar-star,
        [data-theme="dark"] .public-chat-sidebar-lang {
          color: #cbd5e1 !important;
        }
        [data-theme="dark"] .public-chat-sidebar-nav button:hover,
        [data-theme="dark"] .public-chat-sidebar-star:hover,
        [data-theme="dark"] .public-chat-sidebar-lang:hover,
        [data-theme="dark"] .public-chat-sidebar-history-item:hover {
          background: rgba(255,255,255,0.06) !important;
          border-color: rgba(255,255,255,0.12) !important;
          color: #f8fafc !important;
        }
        [data-theme="dark"] .public-chat-sidebar-nav button.is-active {
          background: #f8fafc !important;
          border-color: #f8fafc !important;
          color: #0a0e16 !important;
        }
        [data-theme="dark"] .public-chat-sidebar-history-empty,
        [data-theme="dark"] .public-chat-sidebar-history-item {
          background: rgba(19,27,44,0.72) !important;
          border-color: rgba(255,255,255,0.08) !important;
          color: #cbd5e1 !important;
        }
        [data-theme="dark"] .public-chat-sidebar-footer {
          border-top-color: rgba(255,255,255,0.08) !important;
        }
        [data-theme="dark"] .public-chat-sidebar-user strong {
          color: #f8fafc !important;
        }
        [data-theme="dark"] .public-chat-sidebar-user small {
          color: #94a3b8 !important;
        }

        @media (max-width: 768px) {
          .public-chat-composer-status { border-radius: 11px; }
          /* Density target: WeChat-feel but with enough breathing room that
             long answers stay scannable. Bubbles, font, and inter-turn gap
             all calibrated together — bumping one without the others looks
             off. */
          .public-chat-messages .hf-markdown {
            font-size: 14.5px;
            line-height: 1.55;
          }
          .public-chat-messages .hf-markdown p,
          .public-chat-messages .hf-markdown ul,
          .public-chat-messages .hf-markdown ol,
          .public-chat-messages .hf-markdown table,
          .public-chat-messages .hf-markdown pre,
          .public-chat-messages .hf-markdown blockquote {
            margin: 0.55rem 0;
          }
          .public-chat-messages .hf-markdown th,
          .public-chat-messages .hf-markdown td {
            padding: 0.32rem 0.5rem;
            font-size: 12px;
          }
          .public-chat-messages .pub-msg-row {
            margin-bottom: 12px !important;
          }
          .public-chat-messages .pub-msg-bubble {
            max-width: 92% !important;
            border-radius: 14px !important;
            box-shadow: 0 1px 6px rgba(15,23,42,0.05) !important;
          }
          .public-chat-messages .pub-msg-bubble--assistant {
            padding: 10px 14px !important;
            border: 1.5px solid #e2e8f0 !important;
            border-radius: 4px 14px 14px 14px !important;
          }
          .public-chat-messages .pub-msg-bubble--user {
            padding: 10px 14px !important;
            font-size: 14.5px !important;
            line-height: 1.55 !important;
            border-radius: 14px 14px 4px 14px !important;
          }
          /* The HONE brand row inside each assistant bubble is redundant
             on mobile (the bubble shape already tells you it's HONE) and
             eats 30+ px of vertical space per turn. */
          .public-chat-messages .pub-msg-bubble__brand { display: none !important; }
          .public-chat-page .page-header { height: 50px !important; padding: 0 12px !important; }
          .public-chat-page .public-chat-header-brand {
            gap: 7px !important;
            flex: 1 1 auto !important;
            min-width: 0 !important;
          }
          .public-chat-page .header-logo img { height: 28px !important; }
          .public-chat-page .header-logo span { font-size: 18px !important; }
          .public-chat-page--ready .public-chat-account {
            display: inline-flex !important;
            flex: 0 0 auto !important;
          }
          .public-chat-account-trigger {
            width: 30px !important;
            height: 30px !important;
          }
          .public-chat-account-trigger svg {
            width: 16px !important;
            height: 16px !important;
          }
          .public-chat-page .lang-switch { padding: 1px !important; }
          .public-chat-page .lang-switch button { min-height: 22px !important; min-width: 26px !important; padding: 0 6px !important; font-size: 11px !important; }
          .public-chat-page .btn-chat-nav { min-height: 28px !important; padding: 0 12px !important; font-size: 12px !important; }
          .hone-prefs-trigger { width: 28px !important; height: 28px !important; }
          .hone-prefs-trigger svg { width: 14px !important; height: 14px !important; }
          /* Pin to viewport so the popover can't slide off-screen behind
             other header items, regardless of where the trigger sits. */
          .hone-prefs-panel {
            position: fixed !important;
            top: 50px !important;
            right: 8px !important;
            left: auto !important;
            width: auto !important;
            min-width: 220px !important;
            max-width: calc(100vw - 16px) !important;
            padding: 8px 10px !important;
          }
          .hone-prefs-row { grid-template-columns: 32px 1fr !important; gap: 8px !important; padding: 3px 0 !important; }
          .hone-prefs-label { font-size: 11px !important; }
          .hone-prefs-seg { padding: 4px 0 !important; }
          .hone-prefs-seg--text { padding: 5px 0 !important; }
          .public-chat-shell {
            padding-top: 50px !important;
          }
          .public-chat-messages {
            padding-top: 6px !important;
            padding-bottom: 2px !important;
          }
          .public-chat-messages > div {
            padding-right: 12px !important;
            padding-left: 12px !important;
          }
          .public-chat-composer {
            padding: 10px 12px calc(12px + env(safe-area-inset-bottom)) !important;
            background:
              linear-gradient(180deg, rgba(248,250,252,0), rgba(248,250,252,0.92) 28%, rgba(248,250,252,0.98)) !important;
          }
          .public-chat-proactive-tip-wrap {
            margin: 0 4px 7px !important;
          }
          .public-chat-proactive-tip {
            min-height: 26px !important;
            padding: 5px 9px !important;
            font-size: 12px !important;
          }
          .public-chat-proactive-modal-backdrop {
            align-items: flex-end !important;
            padding: 12px !important;
          }
          .public-chat-proactive-modal {
            width: 100% !important;
            max-height: calc(100dvh - 24px) !important;
            padding: 20px !important;
            border-radius: 18px !important;
          }
          .public-chat-proactive-modal h2 {
            font-size: 20px !important;
          }
          .public-chat-composer-box {
            width: 100% !important;
            max-width: none !important;
            border-radius: 24px !important;
            border: 1px solid rgba(15,23,42,0.08) !important;
            background: rgba(255,255,255,0.96) !important;
            box-shadow: 0 18px 44px rgba(15,23,42,0.14), 0 2px 8px rgba(15,23,42,0.06) !important;
          }
          .public-chat-composer-box:focus-within {
            border-color: rgba(245,158,11,0.45) !important;
            box-shadow: 0 20px 52px rgba(15,23,42,0.16), 0 0 0 4px rgba(245,158,11,0.12) !important;
          }
          .public-chat-composer-row {
            align-items: flex-end !important;
            padding: 8px !important;
            gap: 8px !important;
          }
          .public-chat-composer .pub-attach-btn,
          .public-chat-send-button {
            width: 40px !important;
            height: 40px !important;
            border-radius: 15px !important;
            flex: 0 0 40px !important;
            align-self: flex-end !important;
          }
          .public-chat-composer .pub-attach-btn {
            background: #f8fafc !important;
            border: 1px solid #e2e8f0 !important;
            color: #475569 !important;
          }
          .public-chat-composer .pub-attach-btn[data-open="true"],
          .public-chat-composer .pub-attach-btn:active {
            background: rgba(245,158,11,0.12) !important;
            border-color: rgba(245,158,11,0.35) !important;
            color: #d97706 !important;
          }
          .public-chat-send-button {
            box-shadow: 0 8px 18px rgba(15,23,42,0.18) !important;
          }
          .public-chat-send-button[disabled] {
            box-shadow: none !important;
          }
          .public-chat-composer .pub-attach-btn svg,
          .public-chat-send-button svg { width: 18px !important; height: 18px !important; }
          .public-chat-composer-input {
            flex: 1 1 auto !important;
            min-width: 0 !important;
            min-height: 40px !important;
            max-height: 132px !important;
            padding: 8px 2px 7px !important;
            font-size: 16px !important;
            line-height: 1.48 !important;
            font-weight: 500 !important;
            align-self: stretch !important;
          }
          .public-chat-composer-input::placeholder {
            font-size: 15px !important;
            line-height: 1.48 !important;
          }
          .public-chat-composer-status {
            font-size: 11.5px !important;
            padding: 6px 12px !important;
            margin: 0 4px 7px !important;
            gap: 8px !important;
            border-radius: 14px !important;
          }
          .public-chat-composer-status-label { font-size: 11px !important; }
          .public-chat-composer-status-time { font-size: 11px !important; }
          .public-chat-composer-status-stop { padding: 3px 9px !important; font-size: 10.5px !important; }
          .public-chat-session-strip { display: none !important; }
        }
      `}</style>
    </div>
  );
}
