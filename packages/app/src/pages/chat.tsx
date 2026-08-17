// chat.tsx — HONE Public Site Chat (v4 - Styled to match Landing Page)

import { Markdown } from "@hone-financial/ui/markdown";
import {
  createMemo,
  createSignal,
  createEffect,
  batch,
  For,
  Match,
  onCleanup,
  onMount,
  Show,
  Switch,
} from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import { Portal } from "solid-js/web";
import { useNavigate, useSearchParams } from "@solidjs/router";
import { PublicLoginForm } from "@/components/public-login-form";
import { PublicNav } from "@/components/public-nav";
import { ChatShareModal } from "@/components/chat-share-modal";
import {
  AgentWorkspaceHistoryDrawer,
  AgentWorkspaceIcon,
  AgentWorkspaceLoadingState,
  AgentWorkspaceMobileHeader,
  AgentWorkspaceMobileNav,
  AgentWorkspaceSidebar,
  AgentWorkspaceTopbar,
} from "@/components/public-agent-workspace";
import { routePrefetchHandlers } from "@/lib/route-prefetch";
import { takeResearchAsk } from "@/lib/research-ask";
import { PublicPrefsButton } from "@/components/public-prefs-button";
import { canvasToPngBlob } from "@/components/chat-share-export";
import {
  FinanceCalendarCard,
  FINANCE_CALENDAR_CARD_HEIGHT,
  FINANCE_CALENDAR_CARD_WIDTH,
} from "@/components/finance-calendar-card";
import { FinanceCalendarMessageImage } from "@/components/finance-calendar-message";
import {
  PublicPushDetailDialog,
  PushNavIcon,
  PushUnreadDot,
  ScheduledPushCard,
  type ScheduledPushCardData,
} from "@/components/public-push-center";
import { renderFinanceCalendarMobilePng } from "@/lib/finance-calendar-mobile-renderer";
import { CONTENT } from "@/lib/public-content";
import {
  initPublicPrefs,
} from "@/lib/public-prefs";
import "./public-foundation.css";
import "./public-site.css";
import "./public-polish.css";
import "./public-chat.css";
import "./public-agent-workspace.css";
import "./public-chat-accessibility.css";
import {
  getPublicChatBootstrap,
  getPublicCommunity,
  getPublicFinanceCalendar,
  getPublicHistory,
  getPublicPushes,
  getPublicGeneratedFileBlob,
  connectPublicEvents,
  isUnauthorizedApiError,
  publicLogout,
  openPublicPush,
  sendPublicChat,
  sendPublicFinanceCalendar,
  uploadPublicAttachments,
} from "@/lib/api";
import { buildApiUrl } from "@/lib/backend";
import {
  defaultFinanceCalendarMonth,
  financeCalendarMessageMonth,
  financeCalendarStatusLabel,
  monthOptionsForSelection,
} from "@/lib/finance-calendar";
import { parseMessageContent, messageId } from "@/lib/messages";
import {
  applyPublicAssistantStreamEvent,
  canSendPublicChatMessage,
  findPendingPublicAssistantMessage,
  formatPublicAttachmentBytes,
  isPublicChatQuotaExhausted,
  PUBLIC_CHAT_CONTROLLED_PINCH_SELECTOR,
  PUBLIC_CHAT_VIEWPORT_CONTENT,
  PUBLIC_RESTORE_TIMEOUT_MS,
  publicAttachmentFileLabel,
  appendPublicChatProgressStep,
  publicChatRunEventPatch,
  publicChatRunStartedAtLabel,
  publicChatTerminalEventPatch,
  publicChatToolStatusText,
  publicRestoreRetryDelay,
  rekeyTrailingOptimisticIds,
  resolvePublicChatRecovery,
  resolvePublicChatStreamInterruption,
  isPublicChatBusy,
  isPublicChatRestoreCurrent,
  isPublicChatTerminalStreamEvent,
  mergePublicHistoryWindow,
  shouldPollPublicChatRecovery,
  shouldRecoverPublicChatAfterEof,
  shouldRetryPublicRestore,
  shouldRecoverPinnedBottom,
  shouldPreventPublicChatPinch,
  shouldSubmitPublicChatEnter,
  shouldLoadOlderPublicMessages,
  splitPublicChatAttachments,
  stripAttachmentMarkers,
  toPublicChatMessages,
  unreadCountAfterScheduledPush,
  publicEarningsWorkflowMessage,
} from "@/lib/public-chat";
import { parseSseChunks } from "@/lib/stream";
import {
  publicPushUnreadCount,
  setPublicPushUnreadCount,
} from "@/lib/public-push-unread";
import {
  daySeparatorLabel,
  workspaceUserName,
} from "@/lib/public-agent-workspace";
import { buildChatStarterPrompts } from "@/lib/chat-empty-prompts";
import { useLocale } from "@/lib/i18n";
import type {
  FinanceCalendarPayload,
  PublicCommunityContent,
  PublicAuthUserInfo,
  PublicPushDetail,
} from "@/lib/types";
import type {
  PublicChatAttachment,
  PublicChatAuthState as AuthState,
  PublicChatMessage as ChatMessage,
  PublicEarningsWorkflowKind,
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
// 侧栏/抽屉聊天记录上限:要能覆盖“加载更早”翻出来的历史,太小会让记录
// 看起来永远只有最近几条。
const SIDEBAR_HISTORY_LIMIT = 80;

function AnimatedBackground() {
  return (
    <div class="animated-bg">
      <div class="circle circle-1"></div>
      <div class="circle circle-2"></div>
      <div class="circle circle-3"></div>
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

function assistantMarkdownClass(white = false) {
  return [
    "public-chat-markdown",
    white ? "public-chat-markdown--white" : "",
  ].join(" ");
}

function ProgressiveMessageImage(props: {
  src: string;
  alt: string;
  testId?: string;
  aspectRatio?: string;
  objectFit?: "contain" | "cover";
  flush?: boolean;
}) {
  const [loaded, setLoaded] = createSignal(false);
  const [failed, setFailed] = createSignal(false);
  return (
    <span
      class="public-chat-media-frame"
      classList={{
        "is-loaded": loaded(),
        "is-failed": failed(),
        "is-flush": props.flush,
      }}
      style={{ "aspect-ratio": props.aspectRatio ?? "16 / 10" }}
    >
      <img
        data-testid={props.testId}
        src={props.src}
        alt={props.alt}
        loading="lazy"
        decoding="async"
        style={{ "object-fit": props.objectFit ?? "contain" }}
        onLoad={() => {
          setLoaded(true);
          setFailed(false);
        }}
        onError={() => setFailed(true)}
      />
      <Show when={failed()}>
        <small>{CONTENT.chat_page.composer.finance_calendar_image_failed}</small>
      </Show>
    </span>
  );
}

function AssistantBody(props: {
  content: string;
  financeCalendar?: ChatMessage["financeCalendar"];
  white?: boolean;
}) {
  const cleaned = createMemo(() => stripAttachmentMarkers(props.content));
  const parts = createMemo(() =>
    parseMessageContent(cleaned(), { imageEndpoint: PUBLIC_IMAGE_ENDPOINT }),
  );
  const hasImage = () => parts().some((part) => part.type === "image");
  const calendarMonth = createMemo(
    () =>
      props.financeCalendar?.month ?? financeCalendarMessageMonth(cleaned()),
  );
  const calendarImages = createMemo(() =>
    parts().filter(
      (part): part is { type: "image"; value: string } => part.type === "image",
    ),
  );
  const calendarText = createMemo(() =>
    parts()
      .filter((part) => part.type === "text")
      .map((part) => part.value)
      .join("")
      .trim(),
  );
  const calendarSource = createMemo(() => {
    const persistedPath = props.financeCalendar?.image_path;
    if (!persistedPath) return calendarImages()[0]?.value;
    const cleanPath = persistedPath.startsWith("file://")
      ? persistedPath.slice("file://".length)
      : persistedPath;
    return buildApiUrl(
      `${PUBLIC_IMAGE_ENDPOINT}?path=${encodeURIComponent(cleanPath)}`,
    );
  });
  const markdownClass = () => assistantMarkdownClass(props.white);

  return (
    <Show
      when={calendarMonth() && calendarSource()}
      fallback={
        <Show
          when={hasImage()}
          fallback={<Markdown text={cleaned()} class={markdownClass()} />}
        >
          <For each={parts()}>
            {(part) => (
              <Switch>
                <Match when={part.type === "image"}>
                  <ProgressiveMessageImage
                    testId="assistant-inline-image"
                    src={part.value}
                    alt=""
                  />
                </Match>
                <Match when={part.type === "text"}>
                  <Markdown text={part.value} class={markdownClass()} />
                </Match>
              </Switch>
            )}
          </For>
        </Show>
      }
    >
      <Show when={calendarText()}>
        {(text) => <Markdown text={text()} class={markdownClass()} />}
      </Show>
      <FinanceCalendarMessageImage
        src={calendarSource()!}
        month={calendarMonth()!}
        variant={props.financeCalendar?.variant}
      />
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
            "border-radius": "var(--hone-radius-md)",
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
                  background: "var(--hone-paper-200)",
                  ...(count() === 3 && index() === 0
                    ? { "grid-row": "span 2" }
                    : {}),
                }}
              >
                <ProgressiveMessageImage
                  testId="user-attachment-image"
                  src={publicAttachmentUrl(img)}
                  alt={img.name}
                  aspectRatio="1 / 1"
                  objectFit="cover"
                  flush
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
          "border-radius": "var(--hone-radius-md)",
          overflow: "hidden",
          cursor: "zoom-in",
          "max-width": "420px",
          "line-height": "0",
          position: "relative",
        }}
      >
        <ProgressiveMessageImage
          testId="user-attachment-image"
          src={publicAttachmentUrl(props.images[0]!)}
          alt={props.images[0]!.name}
          aspectRatio="4 / 3"
          flush
        />
      </div>
    </Show>
  );
}

function FileCard(props: {
  file: PublicChatAttachment;
  inUserBubble?: boolean;
}) {
  const [downloadState, setDownloadState] = createSignal<
    "idle" | "working" | "done" | "error"
  >("idle");
  const [downloadError, setDownloadError] = createSignal("");
  const ext = () => publicAttachmentFileLabel(props.file.name);
  const iconBg = () =>
    props.inUserBubble ? "rgba(255,255,255,0.2)" : "rgba(23, 32, 31, 0.05)";
  const iconColor = () => (props.inUserBubble ? "#fff" : "var(--hone-ink-800)");
  const textColor = () =>
    props.inUserBubble ? "rgba(255,255,255,0.95)" : "var(--hone-ink-950)";
  const subColor = () =>
    props.inUserBubble ? "rgba(255,255,255,0.7)" : "var(--hone-ink-600)";
  const downloadStatus = () => {
    if (downloadState() === "working") {
      return CONTENT.chat_page.attachments.downloading;
    }
    if (downloadState() === "done") {
      return CONTENT.chat_page.attachments.downloaded;
    }
    if (downloadState() === "error") {
      return downloadError() || CONTENT.chat_page.attachments.download_failed;
    }
    return CONTENT.chat_page.attachments.click_download;
  };
  const download = async () => {
    if (downloadState() === "working") return;
    setDownloadError("");
    setDownloadState("working");
    try {
      const blob = await getPublicGeneratedFileBlob(props.file.path);
      const objectUrl = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = objectUrl;
      anchor.download = props.file.name;
      anchor.rel = "noopener";
      document.body.appendChild(anchor);
      anchor.click();
      anchor.remove();
      window.setTimeout(() => URL.revokeObjectURL(objectUrl), 1_000);
      setDownloadState("done");
    } catch (cause) {
      setDownloadError(
        cause instanceof Error
          ? cause.message
          : CONTENT.chat_page.attachments.download_failed,
      );
      setDownloadState("error");
    }
  };
  const card = (
    <div
      style={{
        display: "flex",
        "align-items": "center",
        gap: "14px",
        padding: "12px 14px",
        background: props.inUserBubble ? "rgba(255,255,255,0.12)" : "#fff",
        border: props.inUserBubble
          ? "1.5px solid rgba(255,255,255,0.2)"
          : "1.5px solid var(--hone-paper-200)",
        "border-radius": "var(--hone-radius-md)",
        "min-width": "260px",
      }}
    >
      <div
        style={{
          width: "44px",
          height: "44px",
          "border-radius": "var(--hone-radius-sm)",
          background: iconBg(),
          display: "flex",
          "align-items": "center",
          "justify-content": "center",
          "font-family": "var(--hone-font-label)",
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
        <div
          role={downloadState() === "error" ? "alert" : "status"}
          aria-live="polite"
          style={{
            "font-family": "var(--hone-font-label)",
            "font-size": "12px",
            color:
              downloadState() === "error" ? "var(--hone-error-600)" : subColor(),
            "margin-top": "3px",
          }}
        >
          <Show when={props.file.size}>
            {formatPublicAttachmentBytes(props.file.size)} · {" "}
          </Show>
          {downloadStatus()}
        </div>
      </div>
    </div>
  );
  if (props.file.kind === "image") return card;
  return (
    <button
      type="button"
      aria-label={`${CONTENT.chat_page.attachments.click_download} ${props.file.name}`}
      aria-busy={downloadState() === "working"}
      disabled={downloadState() === "working"}
      onClick={() => void download()}
      style={{
        display: "block",
        width: "100%",
        padding: "0",
        border: "0",
        background: "transparent",
        color: "inherit",
        "text-align": "left",
        cursor: downloadState() === "working" ? "wait" : "pointer",
      }}
    >
      {card}
    </button>
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
          background: "var(--hone-ink-950)",
          color: "#fff",
          "border-radius": "24px 24px 4px 24px",
          padding: imageOnly() ? "6px" : "14px 20px",
          "font-size": "16px",
          "line-height": "1.7",
          "box-shadow": "0 10px 30px rgba(23, 32, 31, 0.1)",
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
  message: ChatMessage;
  isContinuation?: boolean;
  onShare?: () => void;
  onStop?: () => void;
  onDismiss?: () => void;
}) {
  const runStartedAtLabel = createMemo(() =>
    publicChatRunStartedAtLabel(
      props.message.runId ? props.message.startedAt : undefined,
    ),
  );
  const nonImageAttachments = createMemo(() =>
    (props.message.attachments ?? []).filter((a) => a.kind !== "image"),
  );
  const isCalendarMessage = createMemo(
    () =>
      !!props.message.financeCalendar ||
      financeCalendarMessageMonth(stripAttachmentMarkers(props.message.content)) !==
        null,
  );
  const pending = () =>
    props.message.phase !== "done" && props.message.phase !== "error";
  const terminal = () => props.message.phase === "error";
  const hasContent = () => !!props.message.content.trim();
  const [elapsed, setElapsed] = createSignal(0);
  const [copied, setCopied] = createSignal(false);

  createEffect(() => {
    if (!props.message.startedAt || !pending()) {
      if (!props.message.startedAt) setElapsed(0);
      return;
    }
    const tick = () =>
      setElapsed(
        Math.max(
          0,
          Math.floor((Date.now() - (props.message.startedAt ?? 0)) / 1000),
        ),
      );
    tick();
    const timer = window.setInterval(tick, 1000);
    onCleanup(() => window.clearInterval(timer));
  });

  const statusLabel = () => {
    if (terminal()) return CONTENT.chat_page.status.error;
    if (hasContent()) return "HONE";
    const currentStatus = props.message.statusText?.trim();
    if (currentStatus) return currentStatus;
    switch (props.message.phase) {
      case "running":
        return CONTENT.chat_page.status.running;
      case "streaming":
        return CONTENT.chat_page.status.streaming;
      default:
        return CONTENT.chat_page.status.thinking;
    }
  };
  const handleCopy = () => {
    const text = stripAttachmentMarkers(props.message.content);
    void navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1400);
    });
  };
  return (
    <div
      class="pub-msg-in pub-msg-row"
      data-testid="assistant-turn"
      data-phase={props.message.phase ?? "done"}
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
          border: "1.5px solid var(--hone-line)",
          "border-radius": "4px 24px 24px 24px",
          padding: "16px 20px",
          color: "var(--hone-ink-800)",
          "box-shadow": "0 4px 20px rgba(23, 32, 31, 0.04)",
          position: "relative",
        }}
      >
        <Show when={!props.isContinuation || pending() || terminal()}>
          <div
            class="pub-msg-bubble__brand pub-assistant-turn-status"
            classList={{
              "is-thinking": pending() && !hasContent(),
              "is-error": terminal(),
            }}
          >
            <span class="pub-assistant-turn-dot" />
            <span class="pub-assistant-turn-label">{statusLabel()}</span>
            <Show when={pending() && !hasContent()}>
              <span class="pub-assistant-turn-time">{elapsed()}s</span>
            </Show>
            <span class="pub-assistant-turn-spacer" />
            <Show when={pending() && props.onStop}>
              <button
                type="button"
                class="pub-assistant-turn-stop"
                onClick={() => props.onStop?.()}
              >
                {CONTENT.chat_page.status.stop}
              </button>
            </Show>
            <Show when={terminal() && props.onDismiss}>
              <button
                type="button"
                class="pub-assistant-turn-dismiss"
                aria-label={CONTENT.chat_page.actions.dismiss_aria}
                onClick={() => props.onDismiss?.()}
              >
                ×
              </button>
            </Show>
          </div>
        </Show>
        <Show when={pending() && !hasContent() && runStartedAtLabel()}>
          <div class="pub-assistant-turn-started-at">
            {runStartedAtLabel()}
          </div>
        </Show>
        <Show when={pending() && !hasContent()}>
          <div class="pub-assistant-thinking-body" aria-hidden="true">
            <i /><i /><i />
          </div>
        </Show>
        <Show when={(props.message.steps?.length ?? 0) > 0 && !hasContent()}>
          <ul class="pub-assistant-turn-steps">
            <For each={props.message.steps}>
              {(step) => <li>{step}</li>}
            </For>
          </ul>
        </Show>
        <Show when={hasContent()}>
          <div class="pub-assistant-turn-content">
            <AssistantBody
              content={props.message.content}
              financeCalendar={props.message.financeCalendar}
            />
            <Show when={pending()}>
              <span class="pub-cursor" />
            </Show>
          </div>
        </Show>
        <Show when={terminal()}>
          <p class="pub-assistant-turn-error">
            {props.message.statusText || CONTENT.chat_page.status.fallback_error}
          </p>
        </Show>
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
        <Show when={props.message.phase === "done" && !isCalendarMessage()}>
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
                  stroke-width="2"
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
          "border-bottom": "1.5px solid var(--hone-paper-100)",
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
                      "border-radius": "var(--hone-radius-md)",
                      border: "1.5px solid var(--hone-paper-200)",
                      background: "var(--hone-paper-100)",
                    }}
                  >
                    <div
                      style={{
                        width: "40px",
                        height: "40px",
                        "border-radius": "var(--hone-radius-sm)",
                        background:
                          "color-mix(in srgb, var(--hone-coral-500) 10%, transparent)",
                        display: "flex",
                        "align-items": "center",
                        "justify-content": "center",
                        "font-family": "var(--hone-font-label)",
                        "font-size": "11px",
                        "font-weight": "800",
                        color: "var(--hone-coral-600)",
                      }}
                    >
                      {publicAttachmentFileLabel(item.name)}
                    </div>
                    <div style={{ flex: "1", "min-width": "0" }}>
                      <div
                        style={{
                          "font-size": "13px",
                          "font-weight": "700",
                          color: "var(--hone-ink-950)",
                          overflow: "hidden",
                          "text-overflow": "ellipsis",
                          "white-space": "nowrap",
                        }}
                      >
                        {item.name}
                      </div>
                      <div
                        style={{
                          "font-family": "var(--hone-font-label)",
                          "font-size": "11px",
                          color: "var(--hone-ink-400)",
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
                    "border-radius": "var(--hone-radius-md)",
                    overflow: "hidden",
                    border: "1.5px solid var(--hone-paper-200)",
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
                  background: "var(--hone-ink-950)",
                  color: "#fff",
                  border: "2.5px solid #fff",
                  cursor: "pointer",
                  "font-size": "12px",
                  display: "flex",
                  "align-items": "center",
                  "justify-content": "center",
                  "box-shadow": "0 4px 10px rgba(23, 32, 31, 0.2)",
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
          "border-radius": "var(--hone-radius-lg)",
          padding: "8px",
          "min-width": "240px",
          bottom: "80px",
          "box-shadow": "0 20px 50px rgba(23, 32, 31, 0.15)",
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
          <span class="pub-attach-icon" style={{ background: "var(--hone-paper-200)" }}>
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
          <span class="pub-attach-icon" style={{ background: "var(--hone-paper-200)" }}>
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

function ProactiveModeTips(props: { openRequest?: number }) {
  const [open, setOpen] = createSignal(false);
  const [copiedExample, setCopiedExample] = createSignal<number | null>(null);
  let copiedTimer: number | undefined;
  let handledOpenRequest = props.openRequest ?? 0;

  createEffect(() => {
    const request = props.openRequest ?? 0;
    if (request <= handledOpenRequest) return;
    handledOpenRequest = request;
    setOpen(true);
  });

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

type EarningsWorkflowStart = {
  kind: PublicEarningsWorkflowKind;
  company: string;
  files: File[];
};

function EarningsResearchQuickAction(props: {
  kind: PublicEarningsWorkflowKind;
  disabled: boolean;
  onStart: (input: EarningsWorkflowStart) => Promise<void>;
}) {
  const [open, setOpen] = createSignal(false);
  const [company, setCompany] = createSignal("");
  const [files, setFiles] = createSignal<File[]>([]);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string>();
  let fileInputRef: HTMLInputElement | undefined;
  const isPreview = () => props.kind === "preview";
  const label = () =>
    isPreview()
      ? CONTENT.chat_page.earnings.preview_label
      : CONTENT.chat_page.earnings.analysis_label;

  const close = () => {
    if (busy()) return;
    setOpen(false);
    setError(undefined);
  };

  const submit = async () => {
    const normalizedCompany = company().trim();
    if (!normalizedCompany) {
      setError(CONTENT.chat_page.earnings.company_required);
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      await props.onStart({
        kind: props.kind,
        company: normalizedCompany,
        files: files(),
      });
      setCompany("");
      setFiles([]);
      setOpen(false);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : CONTENT.chat_page.earnings.start_failed);
    } finally {
      setBusy(false);
    }
  };

  createEffect(() => {
    if (!open()) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") close();
    };
    document.addEventListener("keydown", onKey);
    onCleanup(() => document.removeEventListener("keydown", onKey));
  });

  return (
    <>
      <button
        type="button"
        class="public-chat-proactive-tip public-chat-earnings-action"
        aria-haspopup="dialog"
        aria-expanded={open()}
        disabled={props.disabled}
        onClick={() => setOpen(true)}
      >
        <svg
          class="public-chat-proactive-tip-icon"
          width="15"
          height="15"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.1"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M4 19V9M10 19V5M16 19v-7M22 19H2" />
          <path d={isPreview() ? "m4 6 5-3 5 4 6-4" : "m3 8 5 4 5-6 7 3"} />
        </svg>
        <span>{label()}</span>
      </button>
      <Portal>
        <Show when={open()}>
          <div
            class="public-chat-proactive-modal-backdrop public-chat-earnings-backdrop"
            role="presentation"
            onClick={close}
          >
            <form
              class="public-chat-proactive-modal public-chat-earnings-modal"
              role="dialog"
              aria-modal="true"
              aria-labelledby={`public-chat-earnings-${props.kind}-title`}
              onSubmit={(event) => {
                event.preventDefault();
                void submit();
              }}
              onClick={(event) => event.stopPropagation()}
            >
              <button
                type="button"
                class="public-chat-proactive-close"
                aria-label={CONTENT.chat_page.actions.dismiss_aria}
                disabled={busy()}
                onClick={close}
              >
                ×
              </button>
              <span class="public-chat-earnings-kicker">管理员研究工作流</span>
              <h2 id={`public-chat-earnings-${props.kind}-title`}>{label()}</h2>
              <p class="public-chat-proactive-intro">
                {isPreview()
                  ? CONTENT.chat_page.earnings.preview_hint
                  : CONTENT.chat_page.earnings.analysis_hint}
              </p>
              <label class="public-chat-earnings-field">
                <span>公司名称或股票代码</span>
                <input
                  data-testid={`earnings-${props.kind}-company`}
                  value={company()}
                  maxlength={120}
                  autofocus
                  disabled={busy()}
                  placeholder={CONTENT.chat_page.earnings.company_placeholder}
                  onInput={(event) => setCompany(event.currentTarget.value)}
                />
              </label>
              <Show when={!isPreview()}>
                <label class="public-chat-earnings-field">
                  <span>财报材料（可选）</span>
                  <input
                    ref={fileInputRef}
                    type="file"
                    multiple
                    hidden
                    accept=".pdf,.doc,.docx,.xls,.xlsx,.csv,.txt,.md,image/*"
                    disabled={busy()}
                    onChange={(event) => {
                      setFiles(event.currentTarget.files ? Array.from(event.currentTarget.files) : []);
                      event.currentTarget.value = "";
                    }}
                  />
                  <button
                    type="button"
                    class="public-chat-earnings-file-picker"
                    disabled={busy()}
                    onClick={() => fileInputRef?.click()}
                  >
                    <span>{files().length
                        ? CONTENT.chat_page.earnings.selected_files.replace(
                            "{count}",
                            String(files().length),
                          )
                        : CONTENT.chat_page.earnings.pick_files}</span>
                    <small>{files().length ? files().map((file) => file.name).join("、") : CONTENT.chat_page.earnings.file_hint}</small>
                  </button>
                </label>
              </Show>
              <Show when={error()}>
                {(message) => <p class="public-chat-earnings-error" role="alert">{message()}</p>}
              </Show>
              <div class="public-chat-earnings-actions">
                <button type="button" onClick={close} disabled={busy()}>取消</button>
                <button
                  data-testid={`earnings-${props.kind}-start`}
                  type="submit"
                  class="public-chat-proactive-primary"
                  disabled={busy() || !company().trim()}
                >
                  {busy()
                    ? CONTENT.chat_page.earnings.starting
                    : CONTENT.chat_page.earnings.start_action.replace("{label}", label())}
                </button>
              </div>
            </form>
          </div>
        </Show>
      </Portal>
    </>
  );
}

function FinanceCalendarQuickAction(props: {
  onSent: () => void;
  openRequest?: number;
}) {
  const [open, setOpen] = createSignal(false);
  const [selectedMonth, setSelectedMonth] = createSignal(
    defaultFinanceCalendarMonth(),
  );
  const [busy, setBusy] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [payload, setPayload] = createSignal<FinanceCalendarPayload | null>(
    null,
  );
  const [largePreviewOpen, setLargePreviewOpen] = createSignal(false);
  const [largePreviewScale, setLargePreviewScale] = createSignal(1);
  const [largePreviewFit, setLargePreviewFit] = createSignal(true);
  const monthOptions = createMemo(() =>
    monthOptionsForSelection(selectedMonth()),
  );
  const selectedMonthIndex = createMemo(() =>
    monthOptions().findIndex((month) => month.value === selectedMonth()),
  );
  const selectedMonthLabel = createMemo(
    () => monthOptions()[selectedMonthIndex()]?.label ?? selectedMonth(),
  );
  const macroCount = createMemo(
    () => payload()?.events.filter((event) => event.kind === "macro").length ?? 0,
  );
  const earningsCount = createMemo(
    () => payload()?.events.filter((event) => event.kind === "earnings").length ?? 0,
  );
  let cardEl: HTMLDivElement | undefined;
  let requestId = 0;
  let handledOpenRequest = props.openRequest ?? 0;

  const fitLargePreview = () => {
    const width = Math.max(240, window.innerWidth - 24);
    const height = Math.max(320, window.innerHeight - 82);
    setLargePreviewScale(
      Math.min(
        1,
        width / FINANCE_CALENDAR_CARD_WIDTH,
        height / FINANCE_CALENDAR_CARD_HEIGHT,
      ),
    );
    setLargePreviewFit(true);
  };

  const openLargePreview = () => {
    fitLargePreview();
    setLargePreviewOpen(true);
  };

  const zoomLargePreview = (delta: number) => {
    setLargePreviewFit(false);
    setLargePreviewScale((current) =>
      Math.min(1.5, Math.max(0.2, Math.round((current + delta) * 100) / 100)),
    );
  };

  const loadMonth = async (month: string) => {
    const currentRequest = ++requestId;
    setLoading(true);
    setError(null);
    try {
      const data = await getPublicFinanceCalendar(month);
      if (currentRequest !== requestId) return;
      setPayload(data);
    } catch (err) {
      if (currentRequest !== requestId) return;
      setPayload(null);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      if (currentRequest === requestId) setLoading(false);
    }
  };

  const selectMonth = (month: string) => {
    if (month === selectedMonth() && payload()?.month === month) return;
    setSelectedMonth(month);
    void loadMonth(month);
  };

  const shiftMonth = (delta: number) => {
    const option = monthOptions()[selectedMonthIndex() + delta];
    if (option) selectMonth(option.value);
  };

  const openCalendar = () => {
    const currentMonth = defaultFinanceCalendarMonth();
    setOpen(true);
    setSelectedMonth(currentMonth);
    setPayload(null);
    void loadMonth(currentMonth);
  };

  createEffect(() => {
    const request = props.openRequest ?? 0;
    if (request <= handledOpenRequest) return;
    handledOpenRequest = request;
    openCalendar();
  });

  const close = () => {
    if (busy()) return;
    requestId += 1;
    setOpen(false);
    setLoading(false);
    setError(null);
    setPayload(null);
    setLargePreviewOpen(false);
  };

  const waitForCard = async () => {
    await new Promise<void>((resolve) =>
      requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
    );
    const fonts = (
      document as Document & { fonts?: { ready: Promise<unknown> } }
    ).fonts;
    await fonts?.ready.catch(() => undefined);
    if (!cardEl) {
      throw new Error(CONTENT.chat_page.composer.finance_calendar_render_error);
    }
  };

  const renderPngBlobs = async (data: FinanceCalendarPayload) => {
    if (payload()?.month !== data.month) setPayload(data);
    await waitForCard();
    const { default: html2canvas } = await import("html2canvas");
    const desktopCanvas = await html2canvas(cardEl!, {
      scale: 2,
      backgroundColor: "#eef2f3",
      useCORS: true,
      logging: false,
    });
    return Promise.all([
      canvasToPngBlob(desktopCanvas),
      renderFinanceCalendarMobilePng(data),
    ]);
  };

  const sendCalendar = async () => {
    if (busy()) return;
    setBusy(true);
    setError(null);
    try {
      const data =
        payload()?.month === selectedMonth()
          ? payload()!
          : await getPublicFinanceCalendar(selectedMonth());
      const [desktopBlob, mobileBlob] = await renderPngBlobs(data);
      const uploaded = await uploadPublicAttachments([
        new File([desktopBlob], `hone-finance-calendar-${data.month}.png`, {
          type: "image/png",
        }),
        new File([mobileBlob], `hone-finance-calendar-${data.month}-mobile-v4.png`, {
          type: "image/png",
        }),
      ]);
      const desktopImage = uploaded[0];
      const mobileImage = uploaded[1];
      if (!desktopImage?.path || !mobileImage?.path) {
        throw new Error(CONTENT.chat_page.composer.finance_calendar_upload_error);
      }
      await sendPublicFinanceCalendar({
        path: desktopImage.path,
        mobile_path: mobileImage.path,
        month: data.month,
      });
      props.onSent();
      setOpen(false);
      setPayload(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  createEffect(() => {
    if (!open()) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      if (largePreviewOpen()) {
        setLargePreviewOpen(false);
      } else {
        close();
      }
    };
    document.addEventListener("keydown", onKey);
    onCleanup(() => document.removeEventListener("keydown", onKey));
  });

  createEffect(() => {
    if (!largePreviewOpen() || !largePreviewFit()) return;
    const onResize = () => fitLargePreview();
    window.addEventListener("resize", onResize);
    onCleanup(() => window.removeEventListener("resize", onResize));
  });

  return (
    <>
      <button
        type="button"
        class="public-chat-proactive-tip"
        aria-haspopup="dialog"
        aria-expanded={open()}
        disabled={busy()}
        onClick={openCalendar}
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
          <path d="M8 2v4" />
          <path d="M16 2v4" />
          <rect x="3" y="4" width="18" height="18" rx="3" />
          <path d="M3 10h18" />
          <path d="m9 16 2 2 4-5" />
        </svg>
        <span>{CONTENT.chat_page.composer.finance_calendar_tip}</span>
      </button>
      <Show when={open()}>
        <Portal>
          <div
            class="public-chat-proactive-modal-backdrop"
            role="presentation"
            onClick={close}
          >
          <div
            class="public-chat-proactive-modal public-chat-calendar-modal"
            role="dialog"
            aria-modal="true"
            aria-labelledby="public-chat-calendar-title"
            onClick={(e) => e.stopPropagation()}
          >
            <div class="public-chat-calendar-modal-header">
              <span class="public-chat-calendar-modal-mark" aria-hidden="true">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.1"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path d="M8 2v4M16 2v4M3 10h18" />
                  <rect x="3" y="4" width="18" height="18" rx="2" />
                  <path d="m9 16 2 2 4-5" />
                </svg>
              </span>
              <div>
                <h2 id="public-chat-calendar-title">
                  {CONTENT.chat_page.composer.finance_calendar_title}
                </h2>
                <p>
                  {selectedMonthLabel()} · {macroCount()} {CONTENT.chat_page.composer.finance_calendar_macro_label} ·{" "}
                  {earningsCount()} {CONTENT.chat_page.composer.finance_calendar_earnings_label}
                </p>
              </div>
              <button
                type="button"
                class="public-chat-proactive-close"
                aria-label={CONTENT.chat_page.composer.finance_calendar_close_aria}
                onClick={close}
                disabled={busy()}
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
            </div>

            <div class="public-chat-calendar-modal-body">
              <div class="public-chat-calendar-preview-pane">
                <Switch>
                  <Match when={loading()}>
                    <div
                      class="public-chat-calendar-preview-loading"
                      aria-live="polite"
                    >
                      <span class="public-chat-calendar-loading-ring" />
                      <strong>{CONTENT.chat_page.composer.finance_calendar_loading}</strong>
                    </div>
                  </Match>
                  <Match when={payload()}>
                    {(data) => (
                      <button
                        type="button"
                        class="public-chat-calendar-preview-frame"
                        aria-label={
                          CONTENT.chat_page.composer.finance_calendar_preview_open
                        }
                        onClick={openLargePreview}
                      >
                        <div class="public-chat-calendar-preview-artboard">
                          <FinanceCalendarCard payload={data()} />
                        </div>
                        <span class="public-chat-calendar-preview-hint">
                          <svg
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
                            <circle cx="11" cy="11" r="7" />
                            <path d="m20 20-3.8-3.8M11 8v6M8 11h6" />
                          </svg>
                          {CONTENT.chat_page.composer.finance_calendar_preview_hint}
                        </span>
                      </button>
                    )}
                  </Match>
                  <Match when={error()}>
                    <div class="public-chat-calendar-preview-loading public-chat-calendar-preview-failed">
                      <span aria-hidden="true">!</span>
                      <strong>{CONTENT.chat_page.composer.finance_calendar_error}</strong>
                    </div>
                  </Match>
                </Switch>
              </div>

              <aside class="public-chat-calendar-controls">
                <div class="public-chat-calendar-month-label">
                  {CONTENT.chat_page.composer.finance_calendar_months_label}
                </div>
                <div class="public-chat-calendar-month-nav">
                  <button
                    type="button"
                    aria-label={CONTENT.chat_page.composer.finance_calendar_previous_aria}
                    title={CONTENT.chat_page.composer.finance_calendar_previous_aria}
                    disabled={busy() || loading() || selectedMonthIndex() <= 0}
                    onClick={() => shiftMonth(-1)}
                  >
                    ‹
                  </button>
                  <select
                    value={selectedMonth()}
                    disabled={busy() || loading()}
                    onChange={(event) => selectMonth(event.currentTarget.value)}
                  >
                    <For each={monthOptions()}>
                      {(month) => (
                        <option value={month.value}>{month.label}</option>
                      )}
                    </For>
                  </select>
                  <button
                    type="button"
                    aria-label={CONTENT.chat_page.composer.finance_calendar_next_aria}
                    title={CONTENT.chat_page.composer.finance_calendar_next_aria}
                    disabled={
                      busy() ||
                      loading() ||
                      selectedMonthIndex() >= monthOptions().length - 1
                    }
                    onClick={() => shiftMonth(1)}
                  >
                    ›
                  </button>
                </div>
                <button
                  type="button"
                  class="public-chat-calendar-current-month"
                  disabled={
                    busy() ||
                    loading() ||
                    selectedMonth() === defaultFinanceCalendarMonth()
                  }
                  onClick={() => selectMonth(defaultFinanceCalendarMonth())}
                >
                  {CONTENT.chat_page.composer.finance_calendar_current_month}
                </button>

                <div class="public-chat-calendar-stat-list">
                  <div>
                    <span>{CONTENT.chat_page.composer.finance_calendar_macro_label}</span>
                    <strong>{macroCount()}</strong>
                  </div>
                  <div>
                    <span>{CONTENT.chat_page.composer.finance_calendar_earnings_label}</span>
                    <strong>{earningsCount()}</strong>
                  </div>
                  <div>
                    <span>{CONTENT.chat_page.composer.finance_calendar_holdings_label}</span>
                    <strong>{payload()?.holdings.length ?? 0}</strong>
                  </div>
                </div>

                <Show when={payload()}>
                  {(data) => (
                    <div
                      class="public-chat-calendar-data-state"
                      data-degraded={data().earnings_status !== "ok"}
                    >
                      <span class="public-chat-calendar-state-dot" />
                      <div>
                        <strong>{financeCalendarStatusLabel(data().earnings_status)}</strong>
                        <span>
                          {data().errors[0] ?? CONTENT.chat_page.composer.finance_calendar_sources}
                        </span>
                      </div>
                    </div>
                  )}
                </Show>

                <Show when={error()}>
                  {(message) => (
                    <div class="public-chat-calendar-error" role="alert">
                      <strong>{CONTENT.chat_page.composer.finance_calendar_error}</strong>
                      <span>{message()}</span>
                    </div>
                  )}
                </Show>

                <button
                  type="button"
                  class="public-chat-proactive-primary public-chat-calendar-send"
                  onClick={() => void sendCalendar()}
                  disabled={busy() || loading() || !payload()}
                >
                  {busy()
                    ? CONTENT.chat_page.composer.finance_calendar_sending
                    : CONTENT.chat_page.composer.finance_calendar_send}
                </button>
              </aside>
            </div>
          </div>
          </div>
        </Portal>
      </Show>
      <Portal>
        <Show when={largePreviewOpen() && payload()}>
          <div
            class="public-chat-calendar-large-preview"
            role="dialog"
            aria-modal="true"
            aria-label={CONTENT.chat_page.composer.finance_calendar_preview_aria}
          >
            <header>
              <strong>{selectedMonthLabel()}</strong>
              <div class="public-chat-calendar-zoom-controls">
                <button
                  type="button"
                  aria-label={CONTENT.chat_page.composer.finance_calendar_zoom_out}
                  disabled={largePreviewScale() <= 0.2}
                  onClick={() => zoomLargePreview(-0.15)}
                >
                  −
                </button>
                <span>{Math.round(largePreviewScale() * 100)}%</span>
                <button
                  type="button"
                  aria-label={CONTENT.chat_page.composer.finance_calendar_zoom_in}
                  disabled={largePreviewScale() >= 1.5}
                  onClick={() => zoomLargePreview(0.15)}
                >
                  +
                </button>
                <button
                  type="button"
                  class="is-fit"
                  aria-label={CONTENT.chat_page.composer.finance_calendar_zoom_fit}
                  onClick={fitLargePreview}
                >
                  {CONTENT.chat_page.composer.finance_calendar_zoom_fit}
                </button>
              </div>
              <button
                type="button"
                class="public-chat-calendar-large-close"
                aria-label={CONTENT.chat_page.composer.finance_calendar_preview_close}
                onClick={() => setLargePreviewOpen(false)}
              >
                ×
              </button>
            </header>
            <div class="public-chat-calendar-large-viewport">
              <div
                class="public-chat-calendar-large-canvas-shell"
                style={{
                  width: `${FINANCE_CALENDAR_CARD_WIDTH * largePreviewScale()}px`,
                  height: `${FINANCE_CALENDAR_CARD_HEIGHT * largePreviewScale()}px`,
                }}
              >
                <div
                  class="public-chat-calendar-large-canvas"
                  style={{ transform: `scale(${largePreviewScale()})` }}
                >
                  <FinanceCalendarCard payload={payload()!} />
                </div>
              </div>
            </div>
          </div>
        </Show>
      </Portal>
      <Show when={payload()}>
        {(data) => (
          <>
            <FinanceCalendarCard
              payload={data()}
              hidden
              registerRef={(el) => {
                cardEl = el;
              }}
            />
          </>
        )}
      </Show>
    </>
  );
}

/**
 * The composer's single tool affordance.
 *
 * The dock used to carry two visually identical chip rows — one that
 * navigated away, one that acted on the conversation — so neither read as
 * primary. Destinations now collapse into this menu and the row beside it
 * keeps only actions that stay in the chat.
 */
/**
 * The composer's research-tools menu.
 *
 * Rendered through a Portal on purpose: the chip row it sits in is a
 * horizontally scrollable strip (`overflow-x: auto; overflow-y: hidden`), and
 * an absolutely positioned child of a scroll container gets clipped by it —
 * on phones the menu was being laid out correctly and then cut away entirely,
 * so tapping 工具 looked like nothing happened.
 */
function ChatToolsMenu(props: { isAdmin: boolean }) {
  const navigate = useNavigate();
  const [open, setOpen] = createSignal(false);
  const [anchor, setAnchor] = createSignal<DOMRect>();
  let triggerRef: HTMLButtonElement | undefined;

  const isPhone = () => window.matchMedia("(max-width: 760px)").matches;

  createEffect(() => {
    if (!open()) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    const reposition = () => {
      if (triggerRef) setAnchor(triggerRef.getBoundingClientRect());
    };
    document.addEventListener("keydown", onKeyDown);
    window.addEventListener("resize", reposition);
    onCleanup(() => {
      document.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("resize", reposition);
    });
  });

  const toggle = () => {
    if (open()) {
      setOpen(false);
      return;
    }
    if (triggerRef) setAnchor(triggerRef.getBoundingClientRect());
    setOpen(true);
  };

  const go = (href: string) => {
    setOpen(false);
    navigate(href);
  };

  // Each daily research product is addressable, so the menu can drop the
  // reader at the exact panel instead of only at the desk's front door.
  const copy = () => CONTENT.chat_page.workspace;
  const groups = () => [
    {
      label: copy().tools_group_daily,
      items: [
        { href: "/research", title: copy().research_desk_entry, desc: copy().tools_research_desc },
        { href: "/research?panel=daily-signal-macro", title: copy().tools_macro_title, desc: copy().tools_macro_desc },
        { href: "/research?panel=daily-signal-ai", title: copy().tools_ai_title, desc: copy().tools_ai_desc },
        { href: "/research?panel=company-ratings", title: copy().tools_ratings_title, desc: copy().tools_ratings_desc },
      ],
    },
    {
      label: copy().tools_group_intel,
      items: [
        { href: "/research?panel=influencer-digest", title: copy().tools_influencer_title, desc: copy().tools_influencer_desc },
        { href: "/research?panel=key-event-chain", title: copy().tools_chain_title, desc: copy().tools_chain_desc },
        { href: "/research?panel=weekly-brief", title: copy().tools_weekly_title, desc: copy().tools_weekly_desc },
      ],
    },
    {
      label: copy().tools_group_holdings,
      items: [
        { href: "/research?panel=portfolio-news", title: copy().tools_news_title, desc: copy().tools_news_desc },
        { href: "/research?panel=position-management", title: copy().tools_position_title, desc: copy().tools_position_desc },
      ],
    },
    ...(props.isAdmin
      ? [
          {
            label: copy().tools_group_admin,
            items: [
              { href: "/valuation-lab", title: copy().valuation_lab_title, desc: copy().tools_valuation_desc },
              { href: "/research?group=admin", title: copy().research_library_title, desc: copy().tools_library_desc },
            ],
          },
        ]
      : []),
  ];

  return (
    <>
      <button
        ref={triggerRef}
        type="button"
        class="chat-tools__trigger"
        aria-haspopup="menu"
        aria-expanded={open()}
        {...routePrefetchHandlers("research")}
        onClick={toggle}
      >
        <AgentWorkspaceIcon name="research" size={15} />
        <span>{CONTENT.chat_page.workspace.tools_label}</span>
      </button>
      <Show when={open()}>
        <Portal>
          <div
            class="chat-tools__backdrop"
            classList={{ "is-sheet-backdrop": isPhone() }}
            onClick={() => setOpen(false)}
          >
            <div
              class="chat-tools__menu"
              classList={{ "is-sheet": isPhone() }}
              role="menu"
              style={
                isPhone()
                  ? undefined
                  : {
                      left: `${Math.round(anchor()?.left ?? 16)}px`,
                      bottom: `${Math.round(window.innerHeight - (anchor()?.top ?? 0) + 8)}px`,
                    }
              }
              onClick={(event) => event.stopPropagation()}
            >
              <Show when={isPhone()}>
                <div class="chat-tools__sheet-head">
                  <strong>{CONTENT.chat_page.workspace.tools_group_research}</strong>
                  <button type="button" onClick={() => setOpen(false)}>
                    {CONTENT.chat_page.workspace.tools_close}
                  </button>
                </div>
              </Show>
              <For each={groups()}>
                {(group) => (
                  <>
                    <p class="chat-tools__group">{group.label}</p>
                    <For each={group.items}>
                      {(item) => (
                        <button type="button" role="menuitem" onClick={() => go(item.href)}>
                          <b>{item.title}</b>
                          <small>{item.desc}</small>
                        </button>
                      )}
                    </For>
                  </>
                )}
              </For>
            </div>
          </div>
        </Portal>
      </Show>
    </>
  );
}

function CommunityQuickAction(props: { unread: boolean; onOpen: () => void }) {
  return (
    <button
      type="button"
      class="public-chat-proactive-tip public-chat-community-action"
      onClick={props.onOpen}
      aria-label={props.unread ? CONTENT.chat_page.community.open_aria_unread : CONTENT.chat_page.community.open_aria}
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
        <path d="M21 11.5a8.4 8.4 0 0 1-9 8.4 8.4 8.4 0 0 1-9-8.4 8.4 8.4 0 0 1 9-8.4 8.4 8.4 0 0 1 9 8.4Z" />
        <path d="M8 11h8M8 15h5" />
      </svg>
      <span>查看社区动态</span>
      <Show when={props.unread}>
        <i class="public-chat-community-unread" aria-hidden="true" />
      </Show>
    </button>
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
  onCalendarSent: () => void;
  communityUnread: boolean;
  onOpenCommunity: () => void;
  isSending: boolean;
  remaining: number | undefined;
  dailyLimit: number | undefined;
  trackingOpenRequest: number;
  calendarOpenRequest: number;
  isAdmin: boolean;
  onStartEarnings: (input: EarningsWorkflowStart) => Promise<void>;
}) {
  const [focused, setFocused] = createSignal(false);
  const [menuOpen, setMenuOpen] = createSignal(false);
  let taRef: HTMLTextAreaElement | undefined;
  let imgInputRef: HTMLInputElement | undefined;
  let fileInputRef: HTMLInputElement | undefined;
  let compositionActive = false;
  let suppressEnterUntil = 0;

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
      <div class="public-chat-proactive-tip-wrap">
        <ChatToolsMenu isAdmin={props.isAdmin} />
        <ProactiveModeTips openRequest={props.trackingOpenRequest} />
        <Show when={props.isAdmin}>
          <EarningsResearchQuickAction
            kind="preview"
            disabled={props.isSending || props.uploading}
            onStart={props.onStartEarnings}
          />
          <EarningsResearchQuickAction
            kind="analysis"
            disabled={props.isSending || props.uploading}
            onStart={props.onStartEarnings}
          />
        </Show>
        <FinanceCalendarQuickAction
          onSent={props.onCalendarSent}
          openRequest={props.calendarOpenRequest}
        />
        <CommunityQuickAction
          unread={props.communityUnread}
          onOpen={props.onOpenCommunity}
        />
      </div>
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

      <div class="public-chat-composer-frame">
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
            "border-radius": "var(--hone-radius-lg)",
            border: focused() ? "2px solid var(--hone-ink-950)" : "2px solid var(--hone-paper-200)",
            background: "#fff",
            "box-shadow": focused()
              ? "0 20px 60px rgba(23, 32, 31, 0.08)"
              : "0 10px 30px rgba(23, 32, 31, 0.03)",
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
            aria-label={CONTENT.chat_page.recovery.attach_aria}
            title={CONTENT.chat_page.recovery.attach_aria}
            aria-haspopup="menu"
            aria-expanded={menuOpen()}
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
            onCompositionStart={() => {
              compositionActive = true;
            }}
            onCompositionEnd={() => {
              compositionActive = false;
              // Safari can report isComposing=false on the Enter keydown that
              // commits a Chinese candidate. Ignore that same keystroke.
              suppressEnterUntil = Date.now() + 120;
            }}
            onKeyDown={(e) => {
              const shouldSubmit = shouldSubmitPublicChatEnter({
                key: e.key,
                shiftKey: e.shiftKey,
                eventIsComposing: e.isComposing,
                compositionActive,
                keyCode: e.keyCode,
                now: Date.now(),
                suppressEnterUntil,
              });
              if (shouldSubmit) {
                e.preventDefault();
                if (canSend()) props.onSend();
              } else if (
                e.key === "Enter" &&
                !e.shiftKey &&
                !e.isComposing &&
                !compositionActive &&
                e.keyCode !== 229 &&
                Date.now() < suppressEnterUntil
              ) {
                e.preventDefault();
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
              color: "var(--hone-ink-950)",
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
              "border-radius": "var(--hone-radius-md)",
              background: canSend() ? "var(--hone-ink-950)" : "var(--hone-paper-200)",
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
  const [historyStart, setHistoryStart] = createSignal(0);
  const [historyNextBefore, setHistoryNextBefore] = createSignal<number>();
  const [loadingOlderMessages, setLoadingOlderMessages] = createSignal(false);
  const pushUnreadCount = publicPushUnreadCount;
  const setPushUnreadCount = setPublicPushUnreadCount;
  const [pushDetailOpen, setPushDetailOpen] = createSignal(false);
  const [pushDetailLoading, setPushDetailLoading] = createSignal(false);
  const [pushDetailError, setPushDetailError] = createSignal<string>();
  const [pushDetail, setPushDetail] = createSignal<PublicPushDetail>();
  const [communityUnread, setCommunityUnread] = createSignal(false);
  const [historyDrawerOpen, setHistoryDrawerOpen] = createSignal(false);
  const [workspaceCommunity, setWorkspaceCommunity] = createSignal<
    PublicCommunityContent[]
  >([]);
  const [workspaceCalendar, setWorkspaceCalendar] =
    createSignal<FinanceCalendarPayload>();
  const [trackingOpenRequest, setTrackingOpenRequest] = createSignal(0);
  const [calendarOpenRequest, setCalendarOpenRequest] = createSignal(0);
  const [conversationStartIndex, setConversationStartIndex] = createSignal<number | null>(null);
  // True when the user has scrolled up far enough to lose track of the latest
  // reply — drives the floating scroll-to-bottom affordance above the composer.
  const [awayFromBottom, setAwayFromBottom] = createSignal(false);
  // When set, the server has authoritatively reported an active assistant run
  // for which this tab has no streaming context, usually after a refresh.
  // Poll bootstrap until that run reaches a persisted terminal answer.
  const [backgroundPending, setBackgroundPending] = createSignal<{
    runId: string;
  } | null>(null);
  const [restoreStatus, setRestoreStatus] =
    createSignal<RestoreSessionStatus | null>({ attempt: 1, mode: "loading" });
  let activeController: AbortController | null = null;
  let restoreController: AbortController | null = null;
  let restoreRetryTimer: number | undefined;
  let scrollRef: HTMLDivElement | undefined;
  let messagesInnerRef: HTMLDivElement | undefined;
  let sessionSyncGeneration = 0;
  let localSendGeneration = 0;
  let stickToBottom = true;
  let lastScrollTop = 0;
  let suppressScrollUntil = 0;
  let pinBottomUntil = 0;
  let shareReturnScrollTop: number | null = null;
  let shareReturnAtBottom = true;
  let pushUserId: string | undefined;
  let workspaceLoadedFor: string | undefined;
  let initialBottomPending = true;

  const refreshCommunityUnread = async () => {
    if (authState() !== "ready" || !currentUser()) return;
    try {
      const community = await getPublicCommunity({ limit: 1 });
      setCommunityUnread(community.unread);
    } catch {
      // Community availability must not interrupt the primary chat flow.
    }
  };

  createEffect(() => {
    const user = currentUser();
    if (authState() !== "ready" || !user || workspaceLoadedFor === user.user_id) {
      return;
    }
    workspaceLoadedFor = user.user_id;
    void Promise.allSettled([
      getPublicCommunity({ limit: 3 }).then((community) => {
        setWorkspaceCommunity(community.items);
        setCommunityUnread(community.unread);
      }),
      getPublicFinanceCalendar(defaultFinanceCalendarMonth()).then(
        setWorkspaceCalendar,
      ),
    ]);
  });

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
    if (!messages.some((message) => message.id === id)) return;
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
  const visibleMessages = createMemo(() => {
    const start = conversationStartIndex();
    return start === null ? messages : messages.slice(Math.min(start, messages.length));
  });
  const sidebarHistoryMessages = createMemo(() =>
    messages
      .filter((message) => message.role === "user")
      .slice(-SIDEBAR_HISTORY_LIMIT)
      .reverse(),
  );
  const workspaceResearch = createMemo(() =>
    sidebarHistoryMessages().map((message) => ({
      id: message.id,
      at: message.at,
      title:
        stripAttachmentMarkers(message.content).replace(/\s+/g, " ").trim() ||
        CONTENT.chat_page.sidebar.history_attachment,
    })),
  );
  const workspaceDisplayName = createMemo(() =>
    workspaceUserName(currentUser()?.user_id ?? ""),
  );
  const starterPrompts = createMemo(() =>
    buildChatStarterPrompts({
      holdings: workspaceCalendar()?.holdings ?? [],
      events: workspaceCalendar()?.events ?? [],
      today: workspaceCalendar()?.today,
      locale: useLocale(),
    }),
  );
  const hasOlderMessages = () => historyNextBefore() !== undefined;
  const pendingAssistantMessage = createMemo(() => {
    return findPendingPublicAssistantMessage(messages);
  });
  const isSendingOrStreaming = () =>
    isPublicChatBusy({
      isSending: isSending(),
      hasPendingAssistant: !!pendingAssistantMessage(),
      hasBackgroundPending: !!backgroundPending(),
    });

  createEffect(() => {
    const ready = authState() === "ready";
    const messageCount = messages.length;
    if (!ready || messageCount === 0 || !initialBottomPending) return;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        if (!scrollRef) return;
        initialBottomPending = false;
        pinToBottom(1800);
      });
    });
  });

  const refreshPushUnread = async () => {
    try {
      const payload = await getPublicPushes(undefined, 1);
      setPushUnreadCount(payload.unread_count);
    } catch {
      // Badge availability must not interrupt the primary chat flow.
    }
  };

  const openPushCenter = () => {
    // The destination owns read-through and only clears the badge after the
    // server confirms it. Navigating must never optimistically mutate unread.
    navigate("/pushes");
  };

  const openScheduledPush = async (push: ScheduledPushCardData) => {
    setPushDetailOpen(true);
    setPushDetailError(undefined);
    if (!push.pushId) {
      setPushDetailLoading(false);
      setPushDetail({
        push_id: "legacy",
        job_id: "legacy",
        title: push.title,
        summary: push.summary,
        content: push.fallbackContent ?? push.summary,
        created_at: push.createdAt ?? "",
      });
      return;
    }

    setPushDetailLoading(true);
    setPushDetail(undefined);
    try {
      const payload = await openPublicPush(push.pushId);
      setPushDetail(payload.push);
      setPushUnreadCount(payload.unread_count);
    } catch (error) {
      if (push.fallbackContent) {
        setPushDetail({
          push_id: push.pushId,
          job_id: "fallback",
          title: push.title,
          summary: push.summary,
          content: push.fallbackContent,
          created_at: push.createdAt ?? "",
        });
      } else {
        setPushDetailError(
          error instanceof Error ? error.message : String(error),
        );
      }
    } finally {
      setPushDetailLoading(false);
    }
  };

  const loadOlderMessages = async () => {
    if (!hasOlderMessages() || loadingOlderMessages()) return;
    const before = historyNextBefore();
    if (before === undefined) return;
    const previousScrollHeight = scrollRef?.scrollHeight;
    const previousScrollTop = scrollRef?.scrollTop;
    setLoadingOlderMessages(true);
    try {
      const page = await getPublicHistory(before);
      const older = toPublicChatMessages(page.messages ?? [], page.history_start);
      batch(() => {
        setMessages(reconcile([...older, ...messages], { key: "id" }));
        setHistoryStart(page.history_start);
        setHistoryNextBefore(page.next_before ?? undefined);
      });
      requestAnimationFrame(() => {
        if (scrollRef && previousScrollHeight !== undefined && previousScrollTop !== undefined) {
          suppressScrollUntil = Date.now() + 180;
          scrollRef.scrollTop =
            previousScrollTop + (scrollRef.scrollHeight - previousScrollHeight);
          lastScrollTop = scrollRef.scrollTop;
        }
      });
    } catch {
      // Keep the current window intact; the next upward gesture can retry.
    } finally {
      setLoadingOlderMessages(false);
    }
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
      void loadOlderMessages();
    }
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
  };

  const logoutPublicChat = () => {
    void publicLogout();
    pushUserId = undefined;
    setPushDetailOpen(false);
    setPushUnreadCount(0);
    setCommunityUnread(false);
    setCurrentUser(null);
    setSessionInfo(null);
    setAuthState("logged_out");
  };

  const focusWorkspaceComposer = () => {
    requestAnimationFrame(() => {
      document
        .querySelector<HTMLTextAreaElement>(".public-chat-composer-input")
        ?.focus();
    });
  };

  /* 从「我的 · 持仓」等入口跳来时用 ?q= 预填提问，填好后立即从地址栏擦掉，
     免得刷新或分享链接又把同一句问题灌回输入框。 */
  const [searchParams, setSearchParams] = useSearchParams();
  /** A question an entry point already decided, waiting only for a ready session. */
  const [pendingAutoSend, setPendingAutoSend] = createSignal<string | undefined>();
  createEffect(() => {
    const prefill = searchParams.q;
    if (typeof prefill !== "string" || !prefill.trim()) return;
    const autoSend = searchParams.send === "1";
    setSearchParams({ q: undefined, send: undefined }, { replace: true });
    if (autoSend) {
      // An entry point that already knows the question must not hand back a
      // filled box and ask the user to press send again. Sending cannot happen
      // here: the session may still be loading, so it is queued and dispatched
      // once the turn can actually start.
      setPendingAutoSend(prefill);
      return;
    }
    setDraft(prefill);
    focusWorkspaceComposer();
  });

  /* 研究台面板的「发送到对话」：正文可达数 KB，不走 URL，改经 sessionStorage
     转交（见 research-ask.ts），这里只认一个一次性的 ?ask=research 标记。 */
  createEffect(() => {
    if (searchParams.ask !== "research") return;
    setSearchParams({ ask: undefined }, { replace: true });
    const message = takeResearchAsk();
    if (message) setPendingAutoSend(message);
  });

  /* 「新对话」在当前服务端会话内建立一个新的可见分段。旧消息仍能从左侧
     对话记录恢复，但新页面先显示和当前持仓、日历相关的提问入口。 */
  const startNewConversation = () => {
    if (isSendingOrStreaming()) {
      settleAtBottom();
      return;
    }
    setDraft("");
    setConversationStartIndex(messages.length);
    setAwayFromBottom(false);
    requestAnimationFrame(() => {
      if (scrollRef) scrollRef.scrollTop = 0;
    });
    focusWorkspaceComposer();
  };

  const openWorkspaceResearch = (id: string) => {
    setHistoryDrawerOpen(false);
    setConversationStartIndex(null);
    requestAnimationFrame(() => scrollToMessage(id));
  };

  createEffect(() => {
    if (authState() !== "ready") return;
    const shouldStartNew = searchParams.new === "1";
    const researchParam = searchParams.research;
    const researchId = Array.isArray(researchParam)
      ? researchParam[0]
      : researchParam;
    const shouldOpenPushes = searchParams.panel === "pushes";
    if (!shouldStartNew && !researchId && !shouldOpenPushes) return;
    if (shouldStartNew) startNewConversation();
    if (researchId) openWorkspaceResearch(researchId);
    if (shouldOpenPushes) openPushCenter();
    setSearchParams(
      { new: undefined, research: undefined, panel: undefined },
      { replace: true },
    );
  });

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
      onExhausted?: (message: string) => void;
    } = {},
  ) => {
    clearRestoreRetry();
    const generation = ++sessionSyncGeneration;
    const sendGeneration = localSendGeneration;
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
      const bootstrap = await getPublicChatBootstrap(controller.signal);
      if (
        !isPublicChatRestoreCurrent({
          requestedSyncGeneration: generation,
          currentSyncGeneration: sessionSyncGeneration,
          requestedSendGeneration: sendGeneration,
          currentSendGeneration: localSendGeneration,
        })
      )
        return;
      const user = bootstrap.user;
      const history = bootstrap.messages ?? [];
      const latest = toPublicChatMessages(history, bootstrap.history_start);
      if (options.resetWindow) initialBottomPending = true;
      const merged = options.resetWindow
        ? { messages: latest, start: bootstrap.history_start }
        : mergePublicHistoryWindow(
            messages,
            historyStart(),
            latest,
            bootstrap.history_start,
          );
      const recovery = !isSending()
        ? resolvePublicChatRecovery({
            activeRun: bootstrap.active_run,
            interruptedRun: bootstrap.interrupted_run,
            thinkingText: CONTENT.chat_page.status.thinking,
            interruptedText: CONTENT.chat_page.recovery.interrupted,
          })
        : {};
      if (recovery.message) {
        merged.messages.push(recovery.message);
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
      rekeyTrailingOptimisticIds(messages, merged.messages);
      batch(() => {
        applyPublicUser(user);
        setMessages(reconcile(merged.messages, { key: "id" }));
        setHistoryStart(merged.start);
        setHistoryNextBefore(
          merged.start > 0 ? merged.start : undefined,
        );
        setAuthState("ready");
        setRestoreStatus(null);
        if (options.resetWindow) {
        }
      });
      void refreshCommunityUnread();
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
      // Keep polling when this tab did not start the run. The placeholder is
      // part of the timeline, so the eventual server reply can re-use its id
      // and update the same assistant card in place.
      setBackgroundPending((current) => {
        const next = recovery.activeRunId
          ? { runId: recovery.activeRunId }
          : null;
        return current?.runId === next?.runId ? current : next;
      });
    } catch (error) {
      if (
        !isPublicChatRestoreCurrent({
          requestedSyncGeneration: generation,
          currentSyncGeneration: sessionSyncGeneration,
          requestedSendGeneration: sendGeneration,
          currentSendGeneration: localSendGeneration,
        })
      )
        return;
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
      } else {
        options.onExhausted?.(message);
      }
    } finally {
      window.clearTimeout(timeoutId);
      if (restoreController === controller) restoreController = null;
    }
  };

  // Poll while the server still owes us an answer we can't stream locally.
  // Schedule the next poll only after the previous one settles. A fixed
  // interval used to abort every bootstrap request slower than three seconds,
  // leaving the recovered card in a permanent thinking state.
  createEffect(() => {
    const pendingRunId = backgroundPending()?.runId;
    if (!pendingRunId || isSending()) return;

    let cancelled = false;
    let timerId: number | undefined;
    const scheduleNext = () => {
      if (cancelled) return;
      timerId = window.setTimeout(() => {
        timerId = undefined;
        void poll();
      }, 3000);
    };
    const poll = async () => {
      if (cancelled) return;
      const sameRun = backgroundPending()?.runId === pendingRunId;
      if (
        !shouldPollPublicChatRecovery({
          hasBackgroundPending: sameRun,
          isSending: isSending(),
          restoreInFlight:
            restoreController !== null || restoreRetryTimer !== undefined,
        })
      ) {
        if (sameRun && !isSending()) scheduleNext();
        return;
      }
      await restoreSession();
      if (
        !cancelled &&
        backgroundPending()?.runId === pendingRunId &&
        !isSending()
      ) {
        scheduleNext();
      }
    };

    scheduleNext();
    onCleanup(() => {
      cancelled = true;
      if (timerId !== undefined) window.clearTimeout(timerId);
    });
  });

  createEffect(() => {
    if (authState() !== "ready" || !currentUser()) return;
    const refreshWhenVisible = () => {
      if (document.visibilityState !== "visible") return;
      void refreshCommunityUnread();
      void refreshPushUnread();
    };
    const intervalId = window.setInterval(refreshWhenVisible, 60_000);
    window.addEventListener("focus", refreshWhenVisible);
    document.addEventListener("visibilitychange", refreshWhenVisible);
    onCleanup(() => {
      window.clearInterval(intervalId);
      window.removeEventListener("focus", refreshWhenVisible);
      document.removeEventListener("visibilitychange", refreshWhenVisible);
    });
  });

  createEffect(() => {
    const userId = currentUser()?.user_id;
    if (authState() !== "ready" || !userId) return;
    if (pushUserId === userId) return;
    pushUserId = userId;
    setPushUnreadCount(0);
    void refreshPushUnread();
  });

  onMount(() => {
    initPublicPrefs();
    const viewportMeta = document.querySelector<HTMLMetaElement>(
      'meta[name="viewport"]',
    );
    const previousViewport = viewportMeta?.getAttribute("content") ?? null;
    viewportMeta?.setAttribute("content", PUBLIC_CHAT_VIEWPORT_CONTENT);
    const insideControlledPinchSurface = (target: EventTarget | null) =>
      target instanceof Element &&
      target.closest(PUBLIC_CHAT_CONTROLLED_PINCH_SELECTOR) !== null;
    const preventGesture = (event: Event) => {
      if (!insideControlledPinchSurface(event.target)) event.preventDefault();
    };
    const preventPagePinch = (event: TouchEvent) => {
      if (
        shouldPreventPublicChatPinch({
          touchCount: event.touches.length,
          insideControlledSurface: insideControlledPinchSurface(event.target),
        })
      ) {
        event.preventDefault();
      }
    };
    document.addEventListener("gesturestart", preventGesture);
    document.addEventListener("gesturechange", preventGesture);
    document.addEventListener("gestureend", preventGesture);
    document.addEventListener("touchmove", preventPagePinch, {
      passive: false,
      capture: true,
    });
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
      document.removeEventListener("touchmove", preventPagePinch, true);
    });
  });
  createEffect(() => {
    const locked = authState() === "ready";
    document.documentElement.classList.toggle("public-chat-scroll-lock", locked);
    document.body.classList.toggle("public-chat-scroll-lock", locked);
  });
  createEffect(() => {
    if (authState() !== "ready" || !currentUser()) return;

    let closed = false;
    let source: EventSource | undefined;
    const appendServerPush = (event: MessageEvent) => {
      const payload = JSON.parse(event.data || "{}") as {
        text?: string;
      };
      const text = (payload.text ?? "").trim();
      if (!text) return;
      const shouldStayAtBottom =
        stickToBottom || isBottomPinned() || distanceFromBottom() < 160;
      setMessages(messages.length, {
        id: messageId(),
        role: "assistant",
        content: text,
        phase: "done",
      });
      if (shouldStayAtBottom) pinToBottom(1200);
    };

    const appendScheduledPush = (event: MessageEvent) => {
      const payload = JSON.parse(event.data || "{}") as {
        push_id?: string;
        job_id?: string;
        job_name?: string;
        summary?: string;
        created_at?: string;
        unread_count?: number;
        content?: string;
        text?: string;
      };
      const title = (payload.job_name ?? CONTENT.chat_page.pushes.fallback_title).trim();
      const summary = (
        payload.summary ??
        payload.text ??
        CONTENT.chat_page.pushes.fallback_summary
      ).trim();
      if (!summary) return;
      const shouldStayAtBottom =
        stickToBottom || isBottomPinned() || distanceFromBottom() < 160;
      setMessages(messages.length, {
        id: messageId(),
        role: "assistant",
        content: "",
        phase: "done",
        scheduledPush: {
          pushId: payload.push_id,
          title,
          summary,
          fallbackContent: payload.content ?? payload.text,
          createdAt: payload.created_at,
        },
      });
      setPushUnreadCount((current) =>
        unreadCountAfterScheduledPush(current, payload.unread_count),
      );
      if (shouldStayAtBottom) pinToBottom(1200);
    };

    void connectPublicEvents()
      .then((eventSource) => {
        if (closed) {
          eventSource.close();
          return;
        }
        source = eventSource;
        eventSource.addEventListener("push_message", appendServerPush);
        eventSource.addEventListener("scheduled_message", appendScheduledPush);
      })
      .catch(() => {
        // History restore remains the fallback when the live event stream is down.
      });

    onCleanup(() => {
      closed = true;
      source?.close();
    });
  });
  onCleanup(() => {
    sessionSyncGeneration += 1;
    activeController?.abort();
    restoreController?.abort();
    clearRestoreRetry();
    document.documentElement.classList.remove("public-chat-scroll-lock");
    document.body.classList.remove("public-chat-scroll-lock");
  });

  const sendChatTurn = async (input?: {
    text: string;
    attachments: PublicChatAttachment[];
    earningsWorkflow?: {
      kind: PublicEarningsWorkflowKind;
      company: string;
    };
  }) => {
    const text = input?.text.trim() ?? draft().trim();
    const atts = input?.attachments ?? [...pendingAttachments];
    if (
      (!text && atts.length === 0) ||
      authState() !== "ready" ||
      isSendingOrStreaming() ||
      uploading()
    )
      return;

    // A bootstrap begun for the preceding turn must never reconcile over the
    // optimistic pair created below. Fence and abort every older restore
    // before the new local send becomes visible; this also prevents its retry
    // timer from reviving after the POST has started.
    localSendGeneration += 1;
    sessionSyncGeneration += 1;
    restoreController?.abort();
    clearRestoreRetry();

    const assistantId = messageId();
    if (!input) setDraft("");
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
      steps: input?.earningsWorkflow
        ? [
            input.earningsWorkflow.kind === "analysis"
              ? CONTENT.chat_page.earnings.loading_analysis_skill
              : CONTENT.chat_page.earnings.loading_preview_skill,
          ]
        : undefined,
    });
    if (!input) setPendingAttachments(reconcile([], { key: "path" }));
    scrollToBottom();

    const controller = new AbortController();
    activeController = controller;
    let reachedStreamEof = false;
    let sawTerminalEvent = false;
    let recoverAfterDisconnect = false;
    let lastRunErrorMessage: string | undefined;
    try {
      const stream = await sendPublicChat(
        text,
        atts.map((a) => ({ path: a.path, name: a.name })),
        controller.signal,
        input?.earningsWorkflow,
      );
      const reader = stream.getReader();
      const decoder = new TextDecoder();
      let pendingSse = "";
      let pendingAssistantDelta = "";
      let deltaFrame: number | undefined;
      const flushAssistantDelta = () => {
        deltaFrame = undefined;
        if (!pendingAssistantDelta) return;
        const delta = pendingAssistantDelta;
        pendingAssistantDelta = "";
        const index = messages.findIndex((message) => message.id === assistantId);
        if (index >= 0) {
          setMessages(index, {
            content: applyPublicAssistantStreamEvent(
              messages[index].content,
              "assistant_delta",
              delta,
            ),
            phase: "streaming",
          });
        }
        if (stickToBottom) scrollToBottom();
      };
      const queueAssistantDelta = (delta: string) => {
        if (!delta) return;
        pendingAssistantDelta += delta;
        deltaFrame ??= requestAnimationFrame(flushAssistantDelta);
      };
      const resetAssistantDelta = () => {
        if (deltaFrame !== undefined) cancelAnimationFrame(deltaFrame);
        deltaFrame = undefined;
        pendingAssistantDelta = "";
        const index = messages.findIndex((message) => message.id === assistantId);
        if (index >= 0) {
          setMessages(index, {
            content: applyPublicAssistantStreamEvent(
              messages[index].content,
              "assistant_reset",
            ),
            phase: "thinking",
          });
        }
      };
      streamLoop: while (true) {
        const { done, value } = await reader.read();
        if (done) {
          reachedStreamEof = true;
          break;
        }
        pendingSse += decoder.decode(value, { stream: true });
        const parsed = parseSseChunks(pendingSse);
        pendingSse = parsed.pending;
        for (const ev of parsed.events) {
          if (ev.event === "run_started" || ev.event === "run_progress") {
            const index = messages.findIndex(
              (message) => message.id === assistantId,
            );
            if (index >= 0) {
              const patch = publicChatRunEventPatch(
                messages[index],
                ev.data,
                ev.event === "run_progress"
                  ? CONTENT.chat_page.status.running
                  : CONTENT.chat_page.status.thinking,
              );
              if (patch) setMessages(index, patch);
            }
          }
          if (ev.event === "tool_call") {
            const index = messages.findIndex(
              (message) => message.id === assistantId,
            );
            if (index >= 0) {
              setMessages(index, {
                phase: "running",
                statusText: publicChatToolStatusText(
                  ev.data,
                  CONTENT.chat_page.status.running,
                ),
                ...(messages[index].steps
                  ? {
                      steps: appendPublicChatProgressStep(
                        messages[index].steps,
                        publicChatToolStatusText(
                          ev.data,
                          CONTENT.chat_page.status.running,
                        ),
                      ),
                    }
                  : {}),
              });
            }
          }
          if (ev.event === "assistant_delta") {
            queueAssistantDelta(ev.data.content ?? "");
          }
          if (ev.event === "assistant_reset") {
            resetAssistantDelta();
          }
          if (ev.event === "run_error") {
            lastRunErrorMessage =
              ev.data.message ?? CONTENT.chat_page.status.fallback_error;
            if (deltaFrame !== undefined) cancelAnimationFrame(deltaFrame);
            flushAssistantDelta();
            const index = messages.findIndex((m) => m.id === assistantId);
            if (index >= 0) {
              setMessages(index, {
                // A run error can be attempt-local (for example a context
                // recovery). Keep the turn pending until run_finished says
                // whether the whole server-owned run actually failed.
                phase: "running",
                statusText: CONTENT.chat_page.status.running,
              });
            }
          }
          if (ev.event === "run_finished") {
            if (deltaFrame !== undefined) cancelAnimationFrame(deltaFrame);
            flushAssistantDelta();
            const index = messages.findIndex((m) => m.id === assistantId);
            if (index >= 0) {
              setMessages(
                index,
                publicChatTerminalEventPatch(
                  ev.data,
                  lastRunErrorMessage,
                  CONTENT.chat_page.status.fallback_error,
                ),
              );
            }
            pinToBottom(1400);
          }
          if (ev.event === "error") {
            const index = messages.findIndex((m) => m.id === assistantId);
            if (index >= 0) {
              setMessages(index, {
                phase: "error",
                statusText:
                  ev.data.text ?? CONTENT.chat_page.status.fallback_error,
              });
            }
          }
          if (ev.event === "done") {
            const index = messages.findIndex((m) => m.id === assistantId);
            if (index >= 0 && messages[index].phase !== "error") {
              setMessages(index, { phase: "done", statusText: undefined });
            }
          }
          if (isPublicChatTerminalStreamEvent(ev.event)) {
            // A terminal frame is authoritative. Stop consuming immediately
            // so a queued reset/progress/delta can never make a completed card
            // flash back into a second apparent response round.
            sawTerminalEvent = true;
            break streamLoop;
          }
        }
      }
      if (deltaFrame !== undefined) cancelAnimationFrame(deltaFrame);
      flushAssistantDelta();
      recoverAfterDisconnect = shouldRecoverPublicChatAfterEof({
        reachedEof: reachedStreamEof,
        sawTerminalEvent,
      });
      if (recoverAfterDisconnect) {
        const index = messages.findIndex((m) => m.id === assistantId);
        if (index >= 0) {
          setMessages(index, {
            phase: "thinking",
            statusText: CONTENT.chat_page.recovery.reconnecting,
          });
        }
      }
    } catch (e) {
      const index = messages.findIndex((m) => m.id === assistantId);
      const interrupted = resolvePublicChatStreamInterruption({
        aborted:
          controller.signal.aborted ||
          (e instanceof Error && e.name === "AbortError"),
        recoveringText: CONTENT.chat_page.recovery.reconnecting,
        stoppedText: CONTENT.chat_page.status.stopped,
      });
      recoverAfterDisconnect = interrupted.shouldRecover;
      if (index >= 0) {
        setMessages(index, interrupted.patch);
      }
    } finally {
      const shouldStayAtBottom =
        stickToBottom || isBottomPinned() || distanceFromBottom() < 160;
      if (shouldStayAtBottom) pinToBottom(1600);
      setIsSending(false);
      void restoreSession({
        keepAtBottom: shouldStayAtBottom,
        retryOnFailure: recoverAfterDisconnect,
        onExhausted: recoverAfterDisconnect
          ? () => {
              const index = messages.findIndex((m) => m.id === assistantId);
              if (
                index >= 0 &&
                messages[index].phase !== "done" &&
                messages[index].phase !== "error"
              ) {
                setMessages(index, {
                  phase: "error",
                  statusText:
                    CONTENT.chat_page.recovery.reconnect_failed,
                });
              }
            }
          : undefined,
      });
    }
  };

  const handleSend = () => {
    void sendChatTurn();
  };

  // Dispatch a queued question the moment the session can accept one. Guarding
  // here rather than at the call site keeps a question from being dropped
  // silently while auth or a previous turn is still settling.
  createEffect(() => {
    const text = pendingAutoSend();
    if (!text) return;
    if (authState() !== "ready" || isSendingOrStreaming() || uploading()) return;
    setPendingAutoSend(undefined);
    setDraft("");
    void sendChatTurn({ text, attachments: [] });
  });

  const startEarningsWorkflow = async (input: EarningsWorkflowStart) => {
    if (
      authState() !== "ready" ||
      !currentUser()?.is_admin ||
      isSendingOrStreaming() ||
      uploading()
    ) {
      throw new Error(CONTENT.chat_page.earnings.busy);
    }

    let attachments: PublicChatAttachment[] = [];
    if (input.files.length) {
      setUploading(true);
      try {
        const uploaded = await uploadPublicAttachments(input.files);
        attachments = uploaded.map((item) => ({
          ...item,
          kind: item.kind,
        }));
      } finally {
        setUploading(false);
      }
    }

    const text = publicEarningsWorkflowMessage(
      input.kind,
      input.company,
      attachments.length > 0,
    );
    void sendChatTurn({
      text,
      attachments,
      earningsWorkflow: {
        kind: input.kind,
        company: input.company,
      },
    });
  };

  const handleCalendarSent = () => {
    const shouldStayAtBottom =
      stickToBottom || isBottomPinned() || distanceFromBottom() < 160;
    if (shouldStayAtBottom) pinToBottom(1400);
    void restoreSession({ keepAtBottom: shouldStayAtBottom });
  };

  return (
    <div
      class={`hone-landing-v4 public-chat-page public-chat-page--${authState()} ${authState() !== "logged_out" ? "public-chat-page--ready" : ""}`}
      style={{ height: "100dvh", display: "flex", "flex-direction": "column" }}
    >
      <AnimatedBackground />
      <Show when={authState() === "logged_out"}>
        <PublicNav
          chatMode
          mobileLabel={CONTENT.chat_page.header.subtitle}
          communityUnread={communityUnread()}
          mobileAction={
            <Show when={authState() === "ready"}>
              <button
                type="button"
                class="public-chat-account-trigger public-push-nav-button"
                aria-label={CONTENT.chat_page.pushes.open_aria}
                title={CONTENT.chat_page.pushes.nav}
                onClick={openPushCenter}
              >
                <PushNavIcon />
                <PushUnreadDot count={pushUnreadCount()} />
              </button>
            </Show>
          }
          extraActions={
            <>
              <Show when={authState() === "ready"}>
                <button
                  type="button"
                  class="public-chat-account-trigger public-push-nav-button"
                  aria-label={CONTENT.chat_page.pushes.open_aria}
                  title={CONTENT.chat_page.pushes.nav}
                  onClick={openPushCenter}
                >
                  <PushNavIcon />
                  <PushUnreadDot count={pushUnreadCount()} />
                </button>
              </Show>
              <PublicPrefsButton />
              <AccountButton user={currentUser()} onLogout={logoutPublicChat} />
            </>
          }
        />
      </Show>

      <Switch>
        <Match when={authState() === "logged_out"}>
          <PublicLoginForm
            onLogin={() => restoreSession({ resetWindow: true })}
          />
        </Match>
        <Match when={authState() !== "logged_out"}>
              <>
                <AgentWorkspaceSidebar
                  userName={workspaceDisplayName()}
                  research={workspaceResearch()}
                  researchLoading={authState() === "loading"}
                  activeMode="conversation"
                  activeSection="agent"
                  communityUnread={communityUnread()}
                  hasOlder={hasOlderMessages()}
                  loadingOlder={loadingOlderMessages()}
                  onLoadOlder={() => void loadOlderMessages()}
                  onNewResearch={startNewConversation}
                  onSelectResearch={openWorkspaceResearch}
                  /* 当前区块的导航项被再次点击时只回到对话底部；开新分段
                     是「新对话」按钮的职责，误触不该截断正在看的记录。 */
                  onHome={settleAtBottom}
                  onResearchDesk={() => navigate("/research")}
                  onInsights={() => navigate("/community")}
                  onPushes={() => navigate("/pushes")}
                  unreadPushCount={pushUnreadCount()}
                  onAccount={() => navigate("/me")}
                  onLogout={logoutPublicChat}
                />
                <div class="agent-workspace-stage">
                  <AgentWorkspaceTopbar
                    query=""
                    unreadPushCount={pushUnreadCount()}
                    showSearch={false}
                    onQueryChange={() => {}}
                    preferences={<PublicPrefsButton />}
                    onPushes={openPushCenter}
                  />
                  <AgentWorkspaceMobileHeader
                    userName={workspaceDisplayName()}
                    unreadPushCount={pushUnreadCount()}
                    historyCount={workspaceResearch().length}
                    preferences={<PublicPrefsButton />}
                    onMenu={() => setHistoryDrawerOpen(true)}
                    onPushes={openPushCenter}
                    onAccount={() => navigate("/me")}
                  />
                  <Show when={restoreStatus()?.mode === "failed"}>
                    <div class="agent-workspace-restore-notice" role="status">
                      <span>会话暂时未同步，你仍可查看当前页面。</span>
                      <button type="button" onClick={() => restoreSession({ resetWindow: true, retryOnFailure: true, attempt: 1 })}>重新连接</button>
                    </div>
                  </Show>
                  <Show
                    when={
                      authState() === "loading" ||
                      (restoreStatus()?.mode === "retrying" && !currentUser())
                    }
                  >
                    <AgentWorkspaceLoadingState
                      retrying={restoreStatus()?.mode === "retrying"}
                      attempt={restoreStatus()?.attempt}
                    />
                  </Show>
                  <div class="agent-workspace-body">
                    <div
                      class="public-chat-shell is-conversation"
                      style={{
                        flex: "1",
                        display: "flex",
                        "flex-direction": "column",
                        position: "relative",
                        "z-index": "10",
                        overflow: "hidden",
                      }}
                    >
                      <div
                            ref={scrollRef}
                            class="public-chat-messages"
                            onScroll={handleMessagesScroll}
                            style={{ flex: "1", "overflow-y": "auto", padding: "20px 0" }}
                          >
                            <div
                              ref={messagesInnerRef}
                              style={{ "max-width": "900px", margin: "0 auto", padding: "0 24px" }}
                            >
                              <Show when={authState() === "ready" && visibleMessages().length === 0}>
                                <section class="chat-empty-prompts" aria-label={CONTENT.chat_page.workspace.starter_aria}>
                                  <header>
                                    <span>{CONTENT.chat_page.workspace.starter_kicker}</span>
                                    <h2>{CONTENT.chat_page.workspace.starter_title}</h2>
                                    <p>{CONTENT.chat_page.workspace.starter_desc}</p>
                                  </header>
                                  <div>
                                    <For each={starterPrompts()}>
                                      {(prompt) => (
                                        <button
                                          type="button"
                                          data-prompt-kind={prompt.id}
                                          onClick={() => setPendingAutoSend(prompt.question)}
                                        >
                                          <small>{prompt.eyebrow}</small>
                                          <strong>{prompt.title}</strong>
                                          <i aria-hidden="true">↗</i>
                                        </button>
                                      )}
                                    </For>
                                  </div>
                                </section>
                              </Show>
                              <Show when={hasOlderMessages()}>
                                <div class="public-chat-history-status">
                                  {loadingOlderMessages()
                                    ? CONTENT.chat_page.history.loading_older
                                    : CONTENT.chat_page.history.load_older}
                                </div>
                              </Show>
                              <For each={visibleMessages()}>
                                {(msg, i) => (
                                  <div id={`public-chat-message-${msg.id}`}>
                                    <Show when={daySeparatorLabel(i() > 0 ? visibleMessages()[i() - 1]?.at : undefined, msg.at)}>
                                      {(label) => <div class="public-chat-date-chip"><span>{label()}</span></div>}
                                    </Show>
                                    <Switch>
                                      <Match when={msg.role === "user"}>
                                        <UserBubble content={msg.content} attachments={msg.attachments} onOpenImage={(imgs, index) => setLightbox({ images: imgs, index })} />
                                      </Match>
                                      <Match when={msg.role === "assistant" && msg.scheduledPush}>
                                        <ScheduledPushCard push={msg.scheduledPush!} onOpen={openScheduledPush} />
                                      </Match>
                                      <Match when={msg.role === "assistant" && !msg.scheduledPush}>
                                        <AssistantBubble
                                          message={msg}
                                          isContinuation={i() > 0 && visibleMessages()[i() - 1]?.role === "assistant"}
                                          onShare={() => openShareModal(i())}
                                          onStop={msg.id === "_background" ? undefined : () => activeController?.abort()}
                                          onDismiss={() => setMessages(reconcile(messages.filter((item) => item.id !== msg.id), { key: "id" }))}
                                        />
                                      </Match>
                                    </Switch>
                                  </div>
                                )}
                              </For>
                            </div>
                          </div>
                      <div class="public-chat-composer-dock" style={{ position: "relative" }}>
                        <Show when={awayFromBottom()}>
                          <button type="button" class="public-chat-scroll-down" aria-label={CONTENT.chat_page.actions.scroll_to_bottom_aria} title={CONTENT.chat_page.actions.scroll_to_bottom_aria} onClick={settleAtBottom}>
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 5v14M19 12l-7 7-7-7" /></svg>
                          </button>
                        </Show>
                        <Composer
                          draft={draft()}
                          onDraftChange={setDraft}
                          attachments={pendingAttachments}
                          onRemoveAttachment={(i) => setPendingAttachments(pendingAttachments.filter((_, j) => j !== i))}
                          onPickFiles={async (files) => {
                            setUploading(true);
                            try {
                              const uploaded = await uploadPublicAttachments(files);
                              setPendingAttachments([...pendingAttachments, ...uploaded.map((item) => ({ ...item, kind: item.kind as any }))]);
                            } finally {
                              setUploading(false);
                            }
                          }}
                          uploading={uploading()}
                          onSend={handleSend}
                          onCalendarSent={handleCalendarSent}
                          communityUnread={communityUnread()}
                          onOpenCommunity={() => navigate("/community")}
                          isSending={isSendingOrStreaming()}
                          remaining={sessionInfo()?.remainingToday}
                          dailyLimit={sessionInfo()?.dailyLimit}
                          trackingOpenRequest={trackingOpenRequest()}
                          calendarOpenRequest={calendarOpenRequest()}
                          isAdmin={currentUser()?.is_admin === true}
                          onStartEarnings={startEarningsWorkflow}
                        />
                        <p class="public-chat-disclaimer">HONE 可能出错。内容仅供研究参考，不构成投资建议。</p>
                      </div>
                    </div>
                  </div>
                </div>
                <AgentWorkspaceMobileNav
                  activeMode="conversation"
                  activeSection="agent"
                  communityUnread={communityUnread()}
                  unreadPushCount={pushUnreadCount()}
                  onHome={settleAtBottom}
                  onInsights={() => navigate("/community")}
                  onAgent={settleAtBottom}
                  onResearchDesk={() => navigate("/research")}
                  onPushesTab={() => navigate("/pushes")}
                  onAccount={() => navigate("/me")}
                />
                <AgentWorkspaceHistoryDrawer
                  open={historyDrawerOpen()}
                  userName={workspaceDisplayName()}
                  research={workspaceResearch()}
                  hasOlder={hasOlderMessages()}
                  loadingOlder={loadingOlderMessages()}
                  communityUnread={communityUnread()}
                  onOpen={() => setHistoryDrawerOpen(true)}
                  onClose={() => setHistoryDrawerOpen(false)}
                  onSelectResearch={openWorkspaceResearch}
                  onLoadOlder={() => void loadOlderMessages()}
                  onNewResearch={() => {
                    setHistoryDrawerOpen(false);
                    startNewConversation();
                  }}
                  onHome={() => {
                    setHistoryDrawerOpen(false);
                    settleAtBottom();
                  }}
                  onResearchDesk={() => navigate("/research")}
                  onInsights={() => navigate("/community")}
                  onAccount={() => navigate("/me")}
                />
              </>
        </Match>
      </Switch>

      <PublicPushDetailDialog
        open={pushDetailOpen()}
        detail={pushDetail()}
        loading={pushDetailLoading()}
        error={pushDetailError()}
        onClose={() => setPushDetailOpen(false)}
      />

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

    </div>
  );
}
