// Render-only card used both for the live preview inside the share dialog and
// as the html2canvas source for the exported image. It is styled by the
// scoped rules in chat-share.css and by the light tokens pinned on its root,
// so the output is the same whichever page or theme mounted it.

import { Markdown } from "@hone-financial/ui/markdown";
import { For, Show, createEffect, createMemo, createSignal } from "solid-js";
import QRCode from "qrcode";
import type { PublicChatMessage } from "@/lib/public-chat";
import { stripAttachmentMarkers } from "@/lib/public-chat";
import { shareDateLabel, shareQuestionStyle } from "./chat-share-export";
import { reflowShareTables, shareTableCapacityEm } from "./chat-share-tables";
// The card renders offscreen and standalone; the foundation sheet keeps the
// --hone-* font tokens resolving regardless of which page mounted us.
import "@/pages/public-foundation.css";
import "./chat-share.css";

type ChatShareCardProps = {
  messages: PublicChatMessage[];
  brandName: string;
  brandTagline: string;
  qrUrl: string;
  qrCaption: string;
  disclaimer: string;
  messageFontSize?: number;
  /**
   * "auto" (default) reshapes tables that cannot fit the card's width —
   * rotated when they are short, one block per row when they are tall.
   * "table" leaves every table as the conversation rendered it.
   */
  tableLayout?: "auto" | "table";
  /** When true, position offscreen for capture; otherwise render inline. */
  hidden?: boolean;
  /** Refs the card element so the caller can hand it to html2canvas. */
  registerRef?: (el: HTMLDivElement) => void;
};

// Portrait phone-screenshot width: narrower than a desktop card so long-form
// output keeps its mobile rhythm when people read or forward it inside IM.
export const SHARE_CARD_WIDTH = 420;

// Inline SVG version of /logo.svg — html2canvas can't reliably rasterize
// external SVG <img src="…"> sources (CORS / referrer / async-load races
// all bite), so the brand mark has to live in the DOM as real SVG nodes.
function HoneLogo(props: { size: number }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="145 90 220 210"
      width={props.size}
      height={props.size}
      aria-hidden="true"
      style={{ display: "block", "flex-shrink": "0" }}
    >
      <defs>
        <linearGradient id="hone-share-stone-top" x1="0%" y1="100%" x2="100%" y2="0%">
          <stop offset="0%" stop-color="#ffaf45" />
          <stop offset="100%" stop-color="#ff6b00" />
        </linearGradient>
        <linearGradient id="hone-share-stone-left" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stop-color="#e85d04" />
          <stop offset="100%" stop-color="#9d3c00" />
        </linearGradient>
        <linearGradient id="hone-share-stone-right" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stop-color="#3a4047" />
          <stop offset="100%" stop-color="#1d2024" />
        </linearGradient>
        <linearGradient id="hone-share-knife-top" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#ffb703" />
          <stop offset="100%" stop-color="#f46000" />
        </linearGradient>
        <linearGradient id="hone-share-knife-blade" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#5a636e" />
          <stop offset="50%" stop-color="#2c3136" />
          <stop offset="100%" stop-color="#16191d" />
        </linearGradient>
      </defs>
      <g>
        <path d="M 175 220 L 265 110 L 325 140 L 235 250 Z" fill="url(#hone-share-stone-top)" />
        <path d="M 175 220 L 175 250 L 235 280 L 235 250 Z" fill="url(#hone-share-stone-left)" />
        <path d="M 235 250 L 235 280 L 325 170 L 325 140 Z" fill="url(#hone-share-stone-right)" />
        <path d="M 175 140 L 335 210 L 325 220 L 165 150 Z" fill="url(#hone-share-knife-top)" />
        <path d="M 165 150 L 325 220 L 325 245 L 165 190 Z" fill="url(#hone-share-knife-blade)" />
      </g>
    </svg>
  );
}

export function ChatShareCard(props: ChatShareCardProps) {
  const [qrDataUrl, setQrDataUrl] = createSignal<string>("");
  const messageFontSize = () => props.messageFontSize ?? 15;
  // Memoised so the Markdown resource re-runs only when the size (and with it
  // the room a table has) actually changes, not on every render.
  const reflowTables = createMemo(() => {
    if (props.tableLayout === "table") return undefined;
    const capacityEm = shareTableCapacityEm(messageFontSize());
    return (html: string) => reflowShareTables(html, capacityEm);
  });

  createEffect(() => {
    let cancelled = false;
    QRCode.toDataURL(props.qrUrl, {
      errorCorrectionLevel: "M",
      margin: 1,
      width: 216,
      color: { dark: "#17201f", light: "#ffffff" },
    })
      .then((url) => {
        if (!cancelled) setQrDataUrl(url);
      })
      .catch(() => {
        if (!cancelled) setQrDataUrl("");
      });
    return () => {
      cancelled = true;
    };
  });

  const wrapperStyle = () => {
    if (props.hidden) {
      return {
        position: "fixed" as const,
        left: "-99999px",
        top: "0",
        "pointer-events": "none" as const,
        width: `${SHARE_CARD_WIDTH}px`,
      };
    }
    return { width: `${SHARE_CARD_WIDTH}px` };
  };

  return (
    <div style={wrapperStyle()} aria-hidden={props.hidden ? "true" : undefined}>
      <div
        ref={(el) => props.registerRef?.(el)}
        class="hf-share-card"
        style={{
          // Exported and previewed cards are intentionally light artifacts.
          // Pin every token they consume so a dark application theme cannot
          // turn the card into a low-contrast white-on-white surface.
          "--hone-ink-950": "#17201f",
          "--hone-ink-800": "#2d3735",
          "--hone-ink-600": "#606c68",
          "--hone-ink-400": "#68736f",
          "--hone-paper-50": "#fffdf8",
          "--hone-paper-100": "#f8f4ec",
          "--hone-paper-200": "#eee8dc",
          "--hone-line": "rgba(23, 32, 31, 0.11)",
          "--hone-line-strong": "rgba(23, 32, 31, 0.18)",
          "--text-primary": "#17201f",
          "--text-secondary": "#2d3735",
          "--accent": "#a83c2c",
          "--border": "rgba(23, 32, 31, 0.11)",
          "--shadow": "none",
          "font-size": `${messageFontSize()}px`,
        }}
      >
        <header class="hf-share-card__head">
          <HoneLogo size={22} />
          <span class="hf-share-card__brand">{props.brandName}</span>
          <span class="hf-share-card__date">{shareDateLabel()}</span>
        </header>

        <div class="hf-share-card__body">
          <For each={props.messages}>
            {(msg) => (
              <Show
                when={msg.role === "user"}
                fallback={
                  <div class="hf-share-card__answer">
                    <div class="hf-share-card__byline">
                      <i aria-hidden="true" />
                      {props.brandName}
                    </div>
                    <Markdown
                      text={stripAttachmentMarkers(msg.content)}
                      class="hf-share-card-md"
                      transform={reflowTables()}
                    />
                  </div>
                }
              >
                <div style={shareQuestionStyle(messageFontSize())}>
                  {stripAttachmentMarkers(msg.content)}
                </div>
              </Show>
            )}
          </For>
        </div>

        <footer class="hf-share-card__foot">
          <div class="hf-share-card__foot-copy">
            <div class="hf-share-card__foot-brand">
              <HoneLogo size={18} />
              <b>{props.brandName}</b>
              <span>{props.brandTagline}</span>
            </div>
            <small>{props.qrCaption}</small>
            <small class="is-legal">{props.disclaimer}</small>
          </div>
          <Show when={qrDataUrl()}>
            <div class="hf-share-card__qr">
              <img src={qrDataUrl()} alt="" />
            </div>
          </Show>
        </footer>
      </div>
    </div>
  );
}
