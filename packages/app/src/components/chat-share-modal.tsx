// Bottom-sheet on mobile / centered dialog on desktop. User first picks from
// the latest messages, then previews the rendered image before exporting.

import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { Portal } from "solid-js/web";
import type { PublicChatMessage } from "@/lib/public-chat";
import { stripAttachmentMarkers } from "@/lib/public-chat";
import { ChatShareCard } from "./chat-share-card";
import {
  ShareRenderError,
  canvasToPngBlob,
  canSharePngFile,
  defaultShareMessageId,
  isLikelyIOSPlatform,
  isShareAbortError,
  isShareRenderError,
  recentShareMessages,
} from "./chat-share-export";

export type ChatShareModalProps = {
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
  };
  onClose: () => void;
};

type Toast =
  | { kind: "success"; text: string }
  | { kind: "error"; text: string }
  | null;

type ShareStep = "select" | "preview";

export function ChatShareModal(props: ChatShareModalProps) {
  const [selected, setSelected] = createSignal<Set<string>>(new Set());
  const [toast, setToast] = createSignal<Toast>(null);
  const [busy, setBusy] = createSignal(false);
  const [step, setStep] = createSignal<ShareStep>("select");
  const [previewUrl, setPreviewUrl] = createSignal<string | null>(null);
  let cardEl: HTMLDivElement | undefined;
  let listEl: HTMLUListElement | undefined;
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

  const revokePreviewUrl = () => {
    const url = previewUrl();
    if (url) URL.revokeObjectURL(url);
    setPreviewUrl(null);
  };

  const showToast = (t: Toast) => {
    setToast(t);
    if (toastTimer) window.clearTimeout(toastTimer);
    if (t) {
      toastTimer = window.setTimeout(() => setToast(null), 1800);
    }
  };

  // Reset selection to the newest recent message whenever the modal transitions from
  // closed to open — the parent keeps this component mounted across opens.
  createEffect(() => {
    if (props.open && !wasOpen) {
      const recent = recentMessages();
      const defaultId = defaultShareMessageId(recent);
      setSelected(defaultId ? new Set([defaultId]) : new Set<string>());
      setStep("select");
      revokePreviewUrl();
      setBusy(false);
      setToast(null);
      window.requestAnimationFrame(() => {
        const item = listEl?.querySelector<HTMLElement>(
          ".pub-share-item[data-selected='true']",
        );
        if (!item || !listEl) return;
        listEl.scrollTop =
          item.offsetTop - listEl.clientHeight / 2 + item.clientHeight / 2;
      });
    }
    wasOpen = props.open;
  });

  const selectionKey = () => selectedMessages().map((m) => m.id).join("|");

  createEffect(() => {
    const key = selectionKey();
    if (!props.open || !key) {
      renderKey = "";
      cachedBlob = null;
      renderPromise = null;
      revokePreviewUrl();
      return;
    }
    if (renderKey !== key) {
      renderKey = key;
      cachedBlob = null;
      renderPromise = null;
      revokePreviewUrl();
      setStep("select");
    }
    const timer = window.setTimeout(() => {
      void renderPngBlob().catch(() => {
        // Export handlers surface render failures to the user; background
        // warm-up only exists to keep iOS share inside the tap gesture budget.
      });
    }, 80);
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
      revokePreviewUrl();
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
    if (!cardEl) throw new ShareRenderError("Share card is not ready");
    const { default: html2canvas } = await import("html2canvas");
    try {
      return await html2canvas(cardEl, {
        scale: window.devicePixelRatio >= 2 ? 2 : 1.5,
        backgroundColor: "#ffffff",
        useCORS: true,
        logging: false,
      });
    } catch {
      throw new ShareRenderError("Share image rendering failed");
    }
  };

  const renderPngBlob = async () => {
    const key = selectionKey();
    if (key && renderKey === key && cachedBlob) return cachedBlob;
    if (key && renderKey === key && renderPromise) return renderPromise;
    renderKey = key;
    const canvas = await renderCanvas();
    renderPromise = canvasToPngBlob(canvas).then((blob) => {
      if (renderKey === key) cachedBlob = blob;
      return blob;
    }).finally(() => {
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

  const showPreview = async () => {
    const blob = await renderPngBlob();
    revokePreviewUrl();
    setPreviewUrl(URL.createObjectURL(blob));
    setStep("preview");
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

  const handleGenerateImage = () =>
    withBusy(async () => {
      try {
        await showPreview();
      } catch (error) {
        showExportError("download", error, props.strings.error_render);
      }
    });

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
        const text = selectedMessages()
          .map((m) => {
            const label =
              m.role === "user" ? props.strings.role_user : props.strings.role_assistant;
            return `【${label}】\n${stripAttachmentMarkers(m.content).trim()}`;
          })
          .join("\n\n");
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

  const hasSelection = () => selectedMessages().length > 0;
  const previewLabel = (m: PublicChatMessage) => {
    const text = stripAttachmentMarkers(m.content).replace(/\s+/g, " ").trim();
    return text.length > 80 ? `${text.slice(0, 80)}…` : text || "—";
  };

  return (
    <Show when={props.open}>
      <Portal>
        <div class="pub-share-overlay" onClick={props.onClose} role="presentation">
          <style>{MODAL_CSS}</style>
          <div
            class="pub-share-panel"
            onClick={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-label={props.strings.title}
          >
            <div class="pub-share-header">
              <div>
                <div class="pub-share-title">{props.strings.title}</div>
                <div class="pub-share-subtitle">
                  {step() === "preview"
                    ? props.strings.preview_subtitle
                    : props.strings.subtitle}
                </div>
              </div>
              <button
                type="button"
                class="pub-share-close"
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
            </div>

            <Show
              when={step() === "preview"}
              fallback={
                <>
                  <div class="pub-share-body">
                    <div class="pub-share-list-head">
                      <span>
                        {selectedMessages().length} / {recentMessages().length}
                      </span>
                    </div>
                    <ul class="pub-share-list" ref={listEl}>
                      <For each={recentMessages()}>
                        {(m) => (
                          <li
                            class="pub-share-item"
                            data-selected={selected().has(m.id) ? "true" : undefined}
                            data-role={m.role}
                          >
                            <label class="pub-share-item-label">
                              <input
                                type="checkbox"
                                checked={selected().has(m.id)}
                                onChange={() => toggle(m.id)}
                              />
                              <span class="pub-share-item-role">
                                {m.role === "user"
                                  ? props.strings.role_user
                                  : props.strings.role_assistant}
                              </span>
                              <span class="pub-share-item-preview">
                                {previewLabel(m)}
                              </span>
                            </label>
                          </li>
                        )}
                      </For>
                    </ul>
                  </div>

                  <div class="pub-share-actions pub-share-actions--single">
                    <button
                      type="button"
                      class="pub-share-action pub-share-action--primary"
                      disabled={!hasSelection() || busy()}
                      onClick={handleGenerateImage}
                    >
                      <ActionIcon name="image" />
                      <span>{props.strings.generate_image}</span>
                    </button>
                  </div>
                </>
              }
            >
              <div class="pub-share-preview-body">
                <Show when={previewUrl()}>
                  {(url) => (
                    <div class="pub-share-preview-frame">
                      <img src={url()} alt={props.strings.title} />
                    </div>
                  )}
                </Show>
                <button
                  type="button"
                  class="pub-share-back"
                  onClick={() => setStep("select")}
                >
                  {props.strings.back_to_select}
                </button>
              </div>

              <div class="pub-share-actions">
                <button
                  type="button"
                  class="pub-share-action"
                  disabled={!hasSelection() || busy()}
                  onClick={handleSaveImage}
                >
                  <ActionIcon name="download" />
                  <span>{props.strings.save_image}</span>
                </button>
                <button
                  type="button"
                  class="pub-share-action"
                  disabled={!hasSelection() || busy()}
                  onClick={handleCopyImage}
                >
                  <ActionIcon name="image" />
                  <span>{props.strings.copy_image}</span>
                </button>
                <button
                  type="button"
                  class="pub-share-action"
                  disabled={!hasSelection() || busy()}
                  onClick={handleCopyText}
                >
                  <ActionIcon name="text" />
                  <span>{props.strings.copy_text}</span>
                </button>
                <Show when={supportsSystemShare()}>
                  <button
                    type="button"
                    class="pub-share-action"
                    disabled={!hasSelection() || busy()}
                    onClick={handleSystemShare}
                  >
                    <ActionIcon name="share" />
                    <span>{props.strings.share_other_app}</span>
                  </button>
                </Show>
              </div>
            </Show>

            <Show when={busy()}>
              <div class="pub-share-busy">{props.strings.rendering}</div>
            </Show>

            <Show when={toast()}>
              <div class="pub-share-toast" data-kind={toast()!.kind}>
                {toast()!.text}
              </div>
            </Show>
          </div>

          <Show when={hasSelection()}>
            <ChatShareCard
              messages={selectedMessages()}
              brandName={props.brandName}
              brandTagline={props.brandTagline}
              qrUrl={props.qrUrl}
              qrCaption={props.qrCaption}
              hidden
              registerRef={(el) => (cardEl = el)}
            />
          </Show>
        </div>
      </Portal>
    </Show>
  );
}

function ActionIcon(props: { name: "download" | "image" | "text" | "share" }) {
  const common = {
    width: "18",
    height: "18",
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    "stroke-width": "2",
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

const MODAL_CSS = `
  .pub-share-overlay {
    position: fixed; inset: 0;
    background: rgba(15, 23, 42, 0.42);
    z-index: 1000;
    display: flex; align-items: center; justify-content: center;
    padding: 24px;
    -webkit-backdrop-filter: blur(2px);
    backdrop-filter: blur(2px);
    animation: pub-share-fade 0.18s ease-out;
  }
  @keyframes pub-share-fade { from { opacity: 0; } to { opacity: 1; } }
  .pub-share-panel {
    position: relative;
    width: 100%; max-width: 460px;
    background: #fff;
    border-radius: 18px;
    box-shadow: 0 24px 60px rgba(15,23,42,0.25);
    display: flex; flex-direction: column;
    max-height: calc(100vh - 48px);
    overflow: hidden;
    animation: pub-share-pop 0.22s cubic-bezier(0.16, 1, 0.3, 1);
  }
  @keyframes pub-share-pop {
    from { transform: translateY(12px) scale(0.97); opacity: 0; }
    to { transform: none; opacity: 1; }
  }
  .pub-share-header {
    display: flex; align-items: flex-start; justify-content: space-between;
    gap: 16px;
    padding: 20px 22px 14px 22px;
    border-bottom: 1px solid #f1f5f9;
  }
  .pub-share-title { font-size: 17px; font-weight: 800; color: #0f172a; }
  .pub-share-subtitle { margin-top: 2px; font-size: 12.5px; color: #64748b; }
  .pub-share-close {
    width: 32px; height: 32px;
    border-radius: 999px; border: 0;
    background: #f1f5f9;
    color: #475569;
    display: inline-flex; align-items: center; justify-content: center;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }
  .pub-share-close:hover { background: #e2e8f0; color: #0f172a; }
  .pub-share-body { padding: 14px 22px 10px 22px; overflow-y: auto; flex: 1; min-height: 0; }
  .pub-share-preview-body {
    padding: 16px 22px 12px 22px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    background: linear-gradient(180deg, #f8fafc 0%, #ffffff 100%);
  }
  .pub-share-preview-frame {
    width: min(100%, 320px);
    max-height: min(54vh, 560px);
    margin: 0 auto;
    border-radius: 16px;
    overflow: auto;
    background: #fff;
    border: 1px solid #e2e8f0;
    box-shadow: 0 18px 50px rgba(15,23,42,0.16);
  }
  .pub-share-preview-frame img {
    display: block;
    width: 100%;
    height: auto;
    user-select: none;
    -webkit-user-select: none;
    -webkit-touch-callout: default;
  }
  .pub-share-back {
    display: block;
    margin: 12px auto 0;
    background: none;
    border: 0;
    color: #2563eb;
    font-size: 12.5px;
    font-weight: 700;
    cursor: pointer;
  }
  .pub-share-back:hover { text-decoration: underline; }
  .pub-share-list-head {
    display: flex; align-items: center; justify-content: space-between;
    font-size: 12.5px; color: #64748b;
    margin-bottom: 10px;
  }
  .pub-share-toggle-all {
    background: none; border: 0;
    color: #2563eb; font-weight: 600; font-size: 12.5px;
    cursor: pointer; padding: 4px 0;
  }
  .pub-share-toggle-all:hover { text-decoration: underline; }
  .pub-share-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 6px; }
  .pub-share-item { border-radius: 10px; transition: background 0.12s; }
  .pub-share-item[data-selected="true"] { background: #f1f5f9; }
  .pub-share-item-label {
    display: flex; align-items: flex-start; gap: 10px;
    padding: 9px 12px;
    cursor: pointer;
    line-height: 1.45;
  }
  .pub-share-item-label input { margin-top: 3px; flex: none; cursor: pointer; }
  .pub-share-item-role {
    flex: none;
    font-size: 11px; font-weight: 700;
    letter-spacing: 0.04em; text-transform: uppercase;
    color: #94a3b8;
    padding-top: 1px;
  }
  .pub-share-item[data-role="assistant"] .pub-share-item-role { color: #f59e0b; }
  .pub-share-item-preview {
    flex: 1; min-width: 0;
    font-size: 13px; color: #1e293b;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }
  .pub-share-actions {
    display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px;
    padding: 12px 22px 18px 22px;
    border-top: 1px solid #f1f5f9;
    background: #fafbfc;
  }
  .pub-share-actions--single { grid-template-columns: 1fr; }
  .pub-share-action {
    display: inline-flex; align-items: center; justify-content: center;
    gap: 8px;
    padding: 10px 14px;
    background: #fff;
    color: #0f172a;
    border: 1px solid #e2e8f0;
    border-radius: 12px;
    font-size: 13.5px; font-weight: 600;
    cursor: pointer;
    transition: background 0.12s, border-color 0.12s, transform 0.06s;
  }
  .pub-share-action:hover:not(:disabled) { background: #f8fafc; border-color: #cbd5e1; }
  .pub-share-action--primary {
    background: #0f172a;
    color: #fff;
    border-color: #0f172a;
  }
  .pub-share-action--primary:hover:not(:disabled) {
    background: #111827;
    border-color: #111827;
  }
  .pub-share-action:active:not(:disabled) { transform: scale(0.98); }
  .pub-share-action:disabled { opacity: 0.45; cursor: not-allowed; }
  .pub-share-busy {
    position: absolute; inset: 0;
    background: rgba(255,255,255,0.82);
    display: flex; align-items: center; justify-content: center;
    font-size: 13px; color: #475569;
    pointer-events: none;
  }
  .pub-share-toast {
    position: absolute; left: 50%; bottom: 78px;
    transform: translateX(-50%);
    background: #0f172a; color: #fff;
    padding: 7px 14px; border-radius: 999px;
    font-size: 12.5px;
    pointer-events: none;
    animation: pub-share-toast-in 0.2s ease-out;
  }
  .pub-share-toast[data-kind="error"] { background: #b91c1c; }
  @keyframes pub-share-toast-in {
    from { opacity: 0; transform: translate(-50%, 6px); }
    to { opacity: 1; transform: translate(-50%, 0); }
  }
  @media (max-width: 600px) {
    .pub-share-overlay { padding: 0; align-items: flex-end; }
    .pub-share-panel {
      max-width: none; width: 100%;
      border-radius: 18px 18px 0 0;
      max-height: 92vh;
    }
    .pub-share-actions { grid-template-columns: repeat(2, 1fr); }
  }
`;
