// One-screen share dialog: pick messages and a text size on the left, see the
// real card on the right, act from one row of buttons. On a phone it is a
// sheet — preview first, picker beneath, actions pinned above the safe area.
// The PNG is only rasterised when an action asks for it, and warmed in the
// background so an iOS share can still happen inside the tap gesture.

import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { Portal } from "solid-js/web";
import type { PublicChatMessage } from "@/lib/public-chat";
import { stripAttachmentMarkers } from "@/lib/public-chat";
import { ChatShareCard, SHARE_CARD_WIDTH } from "./chat-share-card";
import {
  ShareRenderError,
  canvasToPngBlob,
  canSharePngFile,
  defaultShareMessageId,
  isLikelyIOSPlatform,
  isShareAbortError,
  isShareRenderError,
  recentShareMessages,
  sharePickerPreview,
  shareTextForClipboard,
} from "./chat-share-export";
import "./chat-share.css";

type ChatShareModalProps = {
  open: boolean;
  messages: PublicChatMessage[];
  seedIndex: number;
  brandName: string;
  brandTagline: string;
  qrUrl: string;
  qrCaption: string;
  strings: {
    title: string;
    subtitle: string;
    preview_subtitle: string;
    preview_scroll_hint: string;
    generate_image: string;
    back_to_select: string;
    download: string;
    save_image: string;
    copy_image: string;
    copy_text: string;
    share: string;
    share_other_app: string;
    close_aria: string;
    success_download: string;
    success_copy_image: string;
    success_copy_text: string;
    success_share: string;
    save_image_hint: string;
    error_download: string;
    error_copy_image: string;
    error_copy_text: string;
    error_render: string;
    error_share: string;
    error_system_share: string;
    role_user: string;
    role_assistant: string;
    nothing_selected: string;
    rendering: string;
    included: string;
    picker_hint: string;
    font_size: string;
    font_s: string;
    font_m: string;
    font_l: string;
    font_xl: string;
    preview_label: string;
    card_disclaimer: string;
    text_footer: string;
  };
  onClose: () => void;
};

type Toast =
  | { kind: "success"; text: string }
  | { kind: "error"; text: string }
  | null;

/** Card text sizes: the conversation's 15px is the middle of the range. */
const SHARE_FONT_SIZES = [14, 15, 16.5, 18] as const;
const DEFAULT_SHARE_FONT_INDEX = 1;

export function ChatShareModal(props: ChatShareModalProps) {
  const [selected, setSelected] = createSignal<Set<string>>(new Set());
  const [toast, setToast] = createSignal<Toast>(null);
  const [busy, setBusy] = createSignal(false);
  const [fontIndex, setFontIndex] = createSignal(DEFAULT_SHARE_FONT_INDEX);
  let cardEl: HTMLDivElement | undefined;
  let closeRef: HTMLButtonElement | undefined;
  let toastTimer: number | undefined;
  let wasOpen = false;
  let renderKey = "";
  let cachedBlob: Blob | null = null;
  let renderPromise: Promise<Blob> | null = null;

  const recentMessages = createMemo<PublicChatMessage[]>(() =>
    recentShareMessages(props.messages, 4, props.seedIndex),
  );
  const selectedMessages = createMemo<PublicChatMessage[]>(() =>
    recentMessages().filter((m) => selected().has(m.id)),
  );
  const hasSelection = () => selectedMessages().length > 0;
  const shareFontSize = () =>
    SHARE_FONT_SIZES[fontIndex()] ?? SHARE_FONT_SIZES[DEFAULT_SHARE_FONT_INDEX];
  const fontLabels = () => [
    props.strings.font_s,
    props.strings.font_m,
    props.strings.font_l,
    props.strings.font_xl,
  ];

  const showToast = (t: Toast) => {
    setToast(t);
    if (toastTimer) window.clearTimeout(toastTimer);
    if (t) {
      toastTimer = window.setTimeout(() => setToast(null), 1800);
    }
  };

  // Reset to the clicked message whenever the dialog transitions from closed
  // to open; the parent keeps this component mounted across opens.
  createEffect(() => {
    if (props.open && !wasOpen) {
      const defaultId = defaultShareMessageId(recentMessages());
      setSelected(defaultId ? new Set([defaultId]) : new Set<string>());
      setFontIndex(DEFAULT_SHARE_FONT_INDEX);
      setBusy(false);
      setToast(null);
      cachedBlob = null;
      renderPromise = null;
      renderKey = "";
      window.requestAnimationFrame(() => closeRef?.focus());
    }
    wasOpen = props.open;
  });

  const renderSignature = () =>
    `${selectedMessages().map((m) => m.id).join("|")}::font:${shareFontSize()}`;

  // Warm the PNG for the current selection so a later share on iOS can hand
  // the file over inside the tap gesture budget.
  createEffect(() => {
    const key = renderSignature();
    if (!props.open || !hasSelection()) {
      cachedBlob = null;
      renderPromise = null;
      renderKey = "";
      return;
    }
    if (renderKey !== key) {
      cachedBlob = null;
      renderPromise = null;
      renderKey = "";
    }
    const timer = window.setTimeout(() => {
      void renderPngBlob().catch(() => {
        // Export handlers surface render failures to the user.
      });
    }, 300);
    onCleanup(() => window.clearTimeout(timer));
  });

  onMount(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && props.open) props.onClose();
    };
    window.addEventListener("keydown", onKey);
    onCleanup(() => {
      window.removeEventListener("keydown", onKey);
      if (toastTimer) window.clearTimeout(toastTimer);
    });
  });

  const toggle = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const supportsSystemShare = () =>
    typeof window !== "undefined" && "share" in navigator;

  const isIOS = () =>
    typeof navigator !== "undefined" &&
    isLikelyIOSPlatform(navigator.platform, navigator.maxTouchPoints || 0);

  const renderCanvas = async () => {
    if (!cardEl) {
      await new Promise<void>((resolve) => window.requestAnimationFrame(() => resolve()));
    }
    if (!cardEl) throw new ShareRenderError("Share card is not ready");
    const { default: html2canvas } = await import("html2canvas");
    try {
      return await html2canvas(cardEl, {
        scale: window.devicePixelRatio >= 2 ? 2 : 1.5,
        backgroundColor: "#fffdf8",
        useCORS: true,
        logging: false,
      });
    } catch {
      throw new ShareRenderError("Share image rendering failed");
    }
  };

  const renderPngBlob = async () => {
    const key = renderSignature();
    if (key && renderKey === key && cachedBlob) return cachedBlob;
    if (key && renderKey === key && renderPromise) return renderPromise;
    renderKey = key;
    const canvas = await renderCanvas();
    renderPromise = canvasToPngBlob(canvas)
      .then((blob) => {
        if (renderKey === key) cachedBlob = blob;
        return blob;
      })
      .finally(() => {
        if (renderKey === key) renderPromise = null;
      });
    return renderPromise;
  };

  const makePngFile = (blob: Blob) =>
    new File([blob], `hone-share-${Date.now()}.png`, { type: "image/png" });

  const sharePngFile = async (blob: Blob) => {
    if (!supportsSystemShare()) return false;
    const file = makePngFile(blob);
    if (!canSharePngFile(navigator, file)) return false;
    await navigator.share({ files: [file], title: props.brandName });
    return true;
  };

  const openImageForSave = (blob: Blob) => {
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.target = "_blank";
    link.rel = "noopener";
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.setTimeout(() => URL.revokeObjectURL(url), 60_000);
    showToast({ kind: "success", text: props.strings.save_image_hint });
  };

  const withBusy = async (fn: () => Promise<void>) => {
    if (busy()) return;
    setBusy(true);
    try {
      await fn();
    } finally {
      setBusy(false);
    }
  };

  const showExportError = (
    action: "download" | "copy_image" | "copy_text" | "system_share",
    error: unknown,
    fallbackText: string,
  ) => {
    if (isShareAbortError(error)) {
      showToast({ kind: "error", text: props.strings.error_share });
      return;
    }
    const detail = error instanceof Error ? `${error.name}: ${error.message}` : String(error);
    console.warn(`[ChatShareModal] ${action} failed: ${detail}`);
    showToast({
      kind: "error",
      text: isShareRenderError(error) ? props.strings.error_render : fallbackText,
    });
  };

  const handleSaveImage = () =>
    withBusy(async () => {
      try {
        const blob = await renderPngBlob();
        if (isIOS()) {
          try {
            if (await sharePngFile(blob)) {
              showToast({ kind: "success", text: props.strings.save_image_hint });
              return;
            }
          } catch (error) {
            if (isShareAbortError(error)) throw error;
          }
          openImageForSave(blob);
          return;
        }
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `hone-share-${Date.now()}.png`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        window.setTimeout(() => URL.revokeObjectURL(url), 2000);
        showToast({ kind: "success", text: props.strings.success_download });
      } catch (error) {
        showExportError("download", error, props.strings.error_download);
      }
    });

  const handleCopyImage = () =>
    withBusy(async () => {
      let blob: Blob | undefined;
      try {
        blob = await renderPngBlob();
        await navigator.clipboard.write([
          new ClipboardItem({ "image/png": blob }),
        ]);
        showToast({ kind: "success", text: props.strings.success_copy_image });
      } catch (error) {
        if (blob && isIOS() && !isShareRenderError(error) && !isShareAbortError(error)) {
          openImageForSave(blob);
          return;
        }
        showExportError("copy_image", error, props.strings.error_copy_image);
      }
    });

  const handleCopyText = () =>
    withBusy(async () => {
      try {
        const text = shareTextForClipboard(
          selectedMessages().map((m) => ({
            role: m.role,
            content: stripAttachmentMarkers(m.content),
          })),
          { user: props.strings.role_user, assistant: props.strings.role_assistant },
          props.strings.text_footer,
        );
        await navigator.clipboard.writeText(text);
        showToast({ kind: "success", text: props.strings.success_copy_text });
      } catch (error) {
        showExportError("copy_text", error, props.strings.error_copy_text);
      }
    });

  const handleSystemShare = () =>
    withBusy(async () => {
      try {
        const blob = await renderPngBlob();
        if (await sharePngFile(blob)) {
          showToast({ kind: "success", text: props.strings.success_share });
          return;
        }
        await navigator.share({ title: props.brandName, url: props.qrUrl });
        showToast({ kind: "success", text: props.strings.success_share });
      } catch (error) {
        if (isIOS() && !isShareRenderError(error) && !isShareAbortError(error)) {
          try {
            const blob = await renderPngBlob();
            openImageForSave(blob);
            return;
          } catch {
            // Fall through to the original error toast.
          }
        }
        showExportError("system_share", error, props.strings.error_system_share);
      }
    });

  const previewLabel = (m: PublicChatMessage) =>
    sharePickerPreview(stripAttachmentMarkers(m.content));

  return (
    <Show when={props.open}>
      <Portal>
        <div class="hc-share" onClick={props.onClose} role="presentation">
          <div
            class="hc-share__panel"
            onClick={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-label={props.strings.title}
          >
            <header class="hc-share__head">
              <div>
                <strong>{props.strings.title}</strong>
                <small>{props.strings.subtitle}</small>
              </div>
              <button
                ref={closeRef}
                type="button"
                class="hc-share__close"
                aria-label={props.strings.close_aria}
                onClick={props.onClose}
              >
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </header>

            <div class="hc-share__grid">
              <aside class="hc-share__side">
                <section class="hc-share__section">
                  <div class="hc-share__section-head">
                    <b>{props.strings.included}</b>
                    <span>
                      {selectedMessages().length} / {recentMessages().length}
                    </span>
                  </div>
                  <ul class="hc-share__list">
                    <For each={recentMessages()}>
                      {(m) => (
                        <li>
                          <button
                            type="button"
                            class="hc-share__row"
                            aria-pressed={selected().has(m.id)}
                            onClick={() => toggle(m.id)}
                          >
                            <span class="hc-share__check" aria-hidden="true">
                              <svg
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="3"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                              >
                                <path d="M20 6L9 17l-5-5" />
                              </svg>
                            </span>
                            <span class="hc-share__row-copy">
                              <small data-role={m.role}>
                                {m.role === "user"
                                  ? props.strings.role_user
                                  : props.strings.role_assistant}
                              </small>
                              <span>{previewLabel(m)}</span>
                            </span>
                          </button>
                        </li>
                      )}
                    </For>
                  </ul>
                </section>

                <section class="hc-share__section">
                  <div class="hc-share__section-head">
                    <b>{props.strings.font_size}</b>
                  </div>
                  <div class="hc-share__seg" role="radiogroup" aria-label={props.strings.font_size}>
                    <For each={SHARE_FONT_SIZES}>
                      {(size, i) => (
                        <button
                          type="button"
                          role="radio"
                          aria-checked={i() === fontIndex()}
                          aria-label={`${fontLabels()[i()]} ${size}px`}
                          onClick={() => setFontIndex(i())}
                        >
                          {fontLabels()[i()]}
                        </button>
                      )}
                    </For>
                  </div>
                </section>

                <div class="hc-share__actions">
                  <button
                    type="button"
                    class="hc-share__action is-primary"
                    disabled={!hasSelection() || busy()}
                    onClick={handleSaveImage}
                  >
                    <ActionIcon name="download" />
                    <span>{props.strings.save_image}</span>
                  </button>
                  <button
                    type="button"
                    class="hc-share__action"
                    disabled={!hasSelection() || busy()}
                    onClick={handleCopyImage}
                  >
                    <ActionIcon name="image" />
                    <span>{props.strings.copy_image}</span>
                  </button>
                  <button
                    type="button"
                    class="hc-share__action"
                    disabled={!hasSelection() || busy()}
                    onClick={handleCopyText}
                  >
                    <ActionIcon name="text" />
                    <span>{props.strings.copy_text}</span>
                  </button>
                  <Show when={supportsSystemShare()}>
                    <button
                      type="button"
                      class="hc-share__action"
                      disabled={!hasSelection() || busy()}
                      onClick={handleSystemShare}
                    >
                      <ActionIcon name="share" />
                      <span>{props.strings.share_other_app}</span>
                    </button>
                  </Show>
                </div>
              </aside>

              <div class="hc-share__preview" aria-label={props.strings.preview_label}>
                <Show
                  when={hasSelection()}
                  fallback={<p class="hc-share__empty">{props.strings.nothing_selected}</p>}
                >
                  <ScaledCard>
                    <ChatShareCard
                      messages={selectedMessages()}
                      brandName={props.brandName}
                      brandTagline={props.brandTagline}
                      qrUrl={props.qrUrl}
                      qrCaption={props.qrCaption}
                      disclaimer={props.strings.card_disclaimer}
                      messageFontSize={shareFontSize()}
                    />
                  </ScaledCard>
                </Show>
              </div>
            </div>

            <Show when={busy()}>
              <div class="hc-share__busy">{props.strings.rendering}</div>
            </Show>

            <Show when={toast()}>
              <div class="hc-share__toast" data-kind={toast()!.kind}>
                {toast()!.text}
              </div>
            </Show>
          </div>

          {/* Offscreen twin of the preview: the element html2canvas captures. */}
          <Show when={hasSelection()}>
            <ChatShareCard
              messages={selectedMessages()}
              brandName={props.brandName}
              brandTagline={props.brandTagline}
              qrUrl={props.qrUrl}
              qrCaption={props.qrCaption}
              disclaimer={props.strings.card_disclaimer}
              messageFontSize={shareFontSize()}
              hidden
              registerRef={(el) => (cardEl = el)}
            />
          </Show>
        </div>
      </Portal>
    </Show>
  );
}

/**
 * Shows the 420px card at the width available to it. The card keeps its
 * export width and is scaled down as a whole, so the preview is the image.
 */
function ScaledCard(props: { children: any }) {
  const [scale, setScale] = createSignal(1);
  const [height, setHeight] = createSignal<number>();
  let stageRef: HTMLDivElement | undefined;
  let innerRef: HTMLDivElement | undefined;

  onMount(() => {
    if (typeof ResizeObserver === "undefined" || !stageRef || !innerRef) return;
    const stage = stageRef;
    const inner = innerRef;
    const apply = () => {
      const next = Math.min(1, stage.clientWidth / SHARE_CARD_WIDTH);
      setScale(next);
      setHeight(Math.ceil(inner.offsetHeight * next));
    };
    const ro = new ResizeObserver(apply);
    ro.observe(stage);
    ro.observe(inner);
    apply();
    onCleanup(() => ro.disconnect());
  });

  return (
    <div
      ref={stageRef}
      class="hc-share__stage"
      style={{ height: height() !== undefined ? `${height()}px` : undefined }}
    >
      <div
        ref={innerRef}
        class="hc-share__stage-inner"
        style={{ transform: `scale(${scale()})` }}
      >
        {props.children}
      </div>
    </div>
  );
}

function ActionIcon(props: { name: "download" | "image" | "text" | "share" }) {
  const common = {
    width: "16",
    height: "16",
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    "stroke-width": "1.8",
    "stroke-linecap": "round" as const,
    "stroke-linejoin": "round" as const,
    "aria-hidden": true,
  };
  switch (props.name) {
    case "download":
      return (
        <svg {...common}>
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="7 10 12 15 17 10" />
          <line x1="12" y1="15" x2="12" y2="3" />
        </svg>
      );
    case "image":
      return (
        <svg {...common}>
          <rect x="3" y="3" width="18" height="18" rx="3" />
          <circle cx="8.5" cy="9" r="1.5" />
          <path d="M21 16l-5-5-9 9" />
        </svg>
      );
    case "text":
      return (
        <svg {...common}>
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
        </svg>
      );
    case "share":
      return (
        <svg {...common}>
          <circle cx="18" cy="5" r="3" />
          <circle cx="6" cy="12" r="3" />
          <circle cx="18" cy="19" r="3" />
          <line x1="8.59" y1="13.51" x2="15.42" y2="17.49" />
          <line x1="15.41" y1="6.51" x2="8.59" y2="10.49" />
        </svg>
      );
  }
}
