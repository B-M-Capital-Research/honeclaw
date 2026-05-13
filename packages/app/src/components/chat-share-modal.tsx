// Bottom-sheet on mobile / centered dialog on desktop. User picks which
// messages to include, previews the rendered card, and exports via one of
// four channels: download / copy-image / copy-text / system share.

import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { Portal } from "solid-js/web";
import type { PublicChatMessage } from "@/lib/public-chat";
import { stripAttachmentMarkers } from "@/lib/public-chat";
import { ChatShareCard } from "./chat-share-card";

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
    select_all: string;
    deselect_all: string;
    download: string;
    copy_image: string;
    copy_text: string;
    share: string;
    close_aria: string;
    success_download: string;
    success_copy_image: string;
    success_copy_text: string;
    success_share: string;
    error_copy_image: string;
    error_share: string;
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

export function ChatShareModal(props: ChatShareModalProps) {
  const [selected, setSelected] = createSignal<Set<string>>(new Set());
  const [toast, setToast] = createSignal<Toast>(null);
  const [busy, setBusy] = createSignal(false);
  let cardEl: HTMLDivElement | undefined;
  let toastTimer: number | undefined;
  let wasOpen = false;

  const showToast = (t: Toast) => {
    setToast(t);
    if (toastTimer) window.clearTimeout(toastTimer);
    if (t) {
      toastTimer = window.setTimeout(() => setToast(null), 1800);
    }
  };

  // Reset selection to just the seed whenever the modal transitions from
  // closed to open — the parent keeps this component mounted across opens.
  createEffect(() => {
    if (props.open && !wasOpen) {
      const seedId = props.messages[props.seedIndex]?.id;
      setSelected(seedId ? new Set([seedId]) : new Set<string>());
      setBusy(false);
      setToast(null);
    }
    wasOpen = props.open;
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

  const allSelected = () => selected().size === props.messages.length;
  const selectedMessages = createMemo<PublicChatMessage[]>(() =>
    props.messages.filter((m) => selected().has(m.id)),
  );

  const supportsClipboardImage = () =>
    typeof window !== "undefined" &&
    "clipboard" in navigator &&
    "write" in navigator.clipboard &&
    typeof window.ClipboardItem !== "undefined";

  const supportsSystemShare = () =>
    typeof window !== "undefined" && "share" in navigator;

  const renderCanvas = async () => {
    if (!cardEl) return null;
    const { default: html2canvas } = await import("html2canvas");
    return html2canvas(cardEl, {
      scale: window.devicePixelRatio >= 2 ? 2 : 1.5,
      backgroundColor: "#ffffff",
      useCORS: true,
      logging: false,
    });
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

  const handleDownload = () =>
    withBusy(async () => {
      const canvas = await renderCanvas();
      if (!canvas) return;
      const blob: Blob | null = await new Promise((resolve) =>
        canvas.toBlob((b) => resolve(b), "image/png"),
      );
      if (!blob) return;
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `hone-share-${Date.now()}.png`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      window.setTimeout(() => URL.revokeObjectURL(url), 2000);
      showToast({ kind: "success", text: props.strings.success_download });
    });

  const handleCopyImage = () =>
    withBusy(async () => {
      const canvas = await renderCanvas();
      if (!canvas) return;
      const blob: Blob | null = await new Promise((resolve) =>
        canvas.toBlob((b) => resolve(b), "image/png"),
      );
      if (!blob) return;
      try {
        await navigator.clipboard.write([
          new ClipboardItem({ "image/png": blob }),
        ]);
        showToast({ kind: "success", text: props.strings.success_copy_image });
      } catch {
        showToast({ kind: "error", text: props.strings.error_copy_image });
      }
    });

  const handleCopyText = () =>
    withBusy(async () => {
      const text = selectedMessages()
        .map((m) => {
          const label =
            m.role === "user" ? props.strings.role_user : props.strings.role_assistant;
          return `【${label}】\n${stripAttachmentMarkers(m.content).trim()}`;
        })
        .join("\n\n");
      await navigator.clipboard.writeText(text);
      showToast({ kind: "success", text: props.strings.success_copy_text });
    });

  const handleSystemShare = () =>
    withBusy(async () => {
      const canvas = await renderCanvas();
      if (!canvas) return;
      const blob: Blob | null = await new Promise((resolve) =>
        canvas.toBlob((b) => resolve(b), "image/png"),
      );
      if (!blob) return;
      const file = new File([blob], `hone-share-${Date.now()}.png`, {
        type: "image/png",
      });
      try {
        if (
          "canShare" in navigator &&
          (navigator as Navigator & { canShare: (d: ShareData) => boolean }).canShare({
            files: [file],
          })
        ) {
          await navigator.share({ files: [file], title: props.brandName });
          showToast({ kind: "success", text: props.strings.success_share });
          return;
        }
        await navigator.share({ title: props.brandName, url: props.qrUrl });
        showToast({ kind: "success", text: props.strings.success_share });
      } catch {
        showToast({ kind: "error", text: props.strings.error_share });
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
                <div class="pub-share-subtitle">{props.strings.subtitle}</div>
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

            <div class="pub-share-body">
              <div class="pub-share-list-head">
                <span>
                  {selectedMessages().length} / {props.messages.length}
                </span>
                <button
                  type="button"
                  class="pub-share-toggle-all"
                  onClick={() => {
                    if (allSelected()) {
                      const seedId = props.messages[props.seedIndex]?.id;
                      setSelected(seedId ? new Set([seedId]) : new Set<string>());
                    } else {
                      setSelected(new Set(props.messages.map((m) => m.id)));
                    }
                  }}
                >
                  {allSelected()
                    ? props.strings.deselect_all
                    : props.strings.select_all}
                </button>
              </div>
              <ul class="pub-share-list">
                <For each={props.messages}>
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
                        <span class="pub-share-item-preview">{previewLabel(m)}</span>
                      </label>
                    </li>
                  )}
                </For>
              </ul>
            </div>

            <div class="pub-share-actions">
              <button
                type="button"
                class="pub-share-action"
                disabled={!hasSelection() || busy()}
                onClick={handleDownload}
              >
                <ActionIcon name="download" />
                <span>{props.strings.download}</span>
              </button>
              <Show when={supportsClipboardImage()}>
                <button
                  type="button"
                  class="pub-share-action"
                  disabled={!hasSelection() || busy()}
                  onClick={handleCopyImage}
                >
                  <ActionIcon name="image" />
                  <span>{props.strings.copy_image}</span>
                </button>
              </Show>
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
                  <span>{props.strings.share}</span>
                </button>
              </Show>
            </div>

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
