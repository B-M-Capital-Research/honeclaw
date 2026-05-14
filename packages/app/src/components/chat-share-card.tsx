// Render-only card used both for the modal preview and as the html2canvas
// source for exported images. Layout stays self-contained through inline and
// scoped styles so the output does not inherit chat-page CSS.

import { Markdown } from "@hone-financial/ui/markdown";
import { For, Show, createEffect, createSignal } from "solid-js";
import QRCode from "qrcode";
import type { PublicChatMessage } from "@/lib/public-chat";
import { stripAttachmentMarkers } from "@/lib/public-chat";

export type ChatShareCardProps = {
  messages: PublicChatMessage[];
  brandName: string;
  brandTagline: string;
  qrUrl: string;
  qrCaption: string;
  messageFontSize?: number;
  /** When true, position offscreen for capture; otherwise render inline. */
  hidden?: boolean;
  /** Refs the wrapper element so the caller can hand it to html2canvas. */
  registerRef?: (el: HTMLDivElement) => void;
};

// Portrait phone-screenshot width: narrower than a desktop card so long-form
// output keeps its mobile rhythm when people read or forward it inside IM.
const CARD_WIDTH = 420;

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
      style={{ display: "block" }}
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

// Scoped markdown styling so the screenshot looks identical regardless of
// viewport / chat-page CSS — only rules under .hf-share-card-md apply here.
const SHARE_CARD_CSS = `
  .hf-share-card-md p { margin: 0.5em 0; }
  .hf-share-card-md p:first-child { margin-top: 0; }
  .hf-share-card-md p:last-child { margin-bottom: 0; }
  .hf-share-card-md strong { color: #0f172a; font-weight: 700; }
  .hf-share-card-md ul,
  .hf-share-card-md ol { margin: 0.6em 0; padding-left: 1.25em; }
  .hf-share-card-md ul { list-style: disc; }
  .hf-share-card-md ol { list-style: decimal; }
  .hf-share-card-md li { margin: 0.2em 0; }
  .hf-share-card-md li::marker { color: #94a3b8; }
  .hf-share-card-md h1,
  .hf-share-card-md h2,
  .hf-share-card-md h3,
  .hf-share-card-md h4 {
    color: #0f172a;
    margin: 0.9em 0 0.3em;
    font-weight: 800;
    line-height: 1.35;
  }
  .hf-share-card-md h1 { font-size: 1.2em; }
  .hf-share-card-md h2 { font-size: 1.1em; }
  .hf-share-card-md h3 { font-size: 1.05em; }
  .hf-share-card-md h4 { font-size: 1em; }
  .hf-share-card-md blockquote {
    margin: 0.7em 0;
    padding: 0.1em 0 0.1em 0.9em;
    border-left: 3px solid #e2e8f0;
    color: #475569;
    font-style: italic;
  }
  .hf-share-card-md table {
    width: 100%;
    border-collapse: collapse;
    margin: 0.7em 0;
    font-size: 0.95em;
  }
  .hf-share-card-md th,
  .hf-share-card-md td {
    border: 1px solid #e2e8f0;
    padding: 6px 9px;
    text-align: left;
  }
  .hf-share-card-md th { background: #f8fafc; color: #0f172a; font-weight: 700; }
  .hf-share-card-md .hf-markdown-code { margin: 10px 0; }
  .hf-share-card-md .hf-markdown-code pre,
  .hf-share-card-md .hf-markdown-code pre.shiki {
    margin: 0;
    padding: 10px 12px;
    background: #f3f4f6 !important;
    border: 0;
    border-radius: 8px;
    font-size: 13px;
    line-height: 1.65;
    white-space: pre-wrap;
    word-break: break-word;
    overflow-wrap: anywhere;
    font-family: -apple-system, BlinkMacSystemFont, "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", "Helvetica Neue", Arial, sans-serif;
  }
  .hf-share-card-md .hf-markdown-code code {
    background: transparent !important;
    padding: 0;
    font-family: inherit;
  }
  .hf-share-card-md .hf-markdown-code code span {
    vertical-align: baseline;
    line-height: inherit;
  }
  .hf-share-card-md :not(pre) > code {
    background: rgba(15, 23, 42, 0.06);
    border-radius: 4px;
    padding: 1px 6px;
    font-size: 0.92em;
    font-family: -apple-system, BlinkMacSystemFont, "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", "Helvetica Neue", Arial, sans-serif;
  }
`;

export function ChatShareCard(props: ChatShareCardProps) {
  const [qrDataUrl, setQrDataUrl] = createSignal<string>("");
  const messageFontSize = () => props.messageFontSize ?? 16.5;

  createEffect(() => {
    let cancelled = false;
    QRCode.toDataURL(props.qrUrl, {
      errorCorrectionLevel: "M",
      margin: 1,
      width: 240,
      color: { dark: "#0f172a", light: "#ffffff" },
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
        width: `${CARD_WIDTH}px`,
      };
    }
    return { width: `${CARD_WIDTH}px`, margin: "0 auto" };
  };

  return (
    <div style={wrapperStyle()} aria-hidden={props.hidden ? "true" : undefined}>
      <style>{SHARE_CARD_CSS}</style>
      <div
        ref={(el) => props.registerRef?.(el)}
        style={{
          width: `${CARD_WIDTH}px`,
          background:
            "linear-gradient(180deg, #fffaf3 0%, #ffffff 12%, #ffffff 100%)",
          "font-family":
            "-apple-system, BlinkMacSystemFont, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif",
          color: "#0f172a",
          "box-sizing": "border-box",
        }}
      >
        {/* Header */}
        <div
          style={{
            display: "flex",
            "align-items": "center",
            gap: "10px",
            padding: "22px 20px 16px 20px",
            "border-bottom": "1px solid #f1f5f9",
          }}
        >
          <HoneLogo size={30} />
          <div style={{ display: "flex", "flex-direction": "column" }}>
            <span
              style={{
                "font-size": "16px",
                "font-weight": "800",
                "letter-spacing": "0.02em",
                color: "#0f172a",
              }}
            >
              {props.brandName}
            </span>
            <span style={{ "font-size": "11.5px", color: "#64748b", "margin-top": "1px" }}>
              {props.brandTagline}
            </span>
          </div>
        </div>

        {/* Messages */}
        <div
          style={{
            padding: "18px 20px 22px 20px",
            display: "flex",
            "flex-direction": "column",
            gap: "14px",
          }}
        >
          <For each={props.messages}>
            {(msg) => (
              <Show
                when={msg.role === "user"}
                fallback={
                  <AssistantRow
                    content={msg.content}
                    fontSize={messageFontSize()}
                  />
                }
              >
                <UserRow content={msg.content} fontSize={messageFontSize()} />
              </Show>
            )}
          </For>
        </div>

        {/* Footer */}
        <div
          style={{
            display: "grid",
            "grid-template-columns": "1fr auto",
            "align-items": "center",
            gap: "18px",
            padding: "16px 20px 22px 20px",
            "border-top": "1px solid #f1f5f9",
            background: "#fafbfc",
          }}
        >
          <div style={{ display: "flex", "flex-direction": "column", gap: "7px", "min-width": "0" }}>
            <div style={{ display: "flex", "align-items": "center", gap: "8px", "min-height": "28px" }}>
              <HoneLogo size={24} />
              <span
                style={{
                  "font-size": "14.5px",
                  "font-weight": "800",
                  color: "#0f172a",
                  "letter-spacing": "0.02em",
                }}
              >
                {props.brandName}
              </span>
            </div>
            <span style={{ "font-size": "11.5px", color: "#64748b", "line-height": "1.4" }}>
              {props.qrCaption}
            </span>
          </div>
          <Show when={qrDataUrl()}>
            <div
              style={{
                width: "88px",
                height: "88px",
                display: "flex",
                "align-items": "center",
                "justify-content": "center",
                "border-radius": "12px",
                background: "#fff",
                border: "1px solid #e2e8f0",
                "flex-shrink": "0",
                "box-sizing": "border-box",
              }}
            >
              <img
                src={qrDataUrl()}
                alt=""
                style={{
                  display: "block",
                  width: "76px",
                  height: "76px",
                }}
              />
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
}

function UserRow(props: { content: string; fontSize: number }) {
  const cleaned = () => stripAttachmentMarkers(props.content);
  return (
    <div style={{ display: "flex", "justify-content": "flex-end" }}>
      <div
        style={{
          "max-width": "86%",
          background: "#0f172a",
          color: "#f8fafc",
          padding: "10px 14px",
          "border-radius": "12px 12px 4px 12px",
          "font-size": `${props.fontSize}px`,
          "line-height": "1.55",
          "white-space": "pre-wrap",
          "word-break": "break-word",
        }}
      >
        {cleaned()}
      </div>
    </div>
  );
}

function AssistantRow(props: { content: string; fontSize: number }) {
  return (
    <div style={{ display: "flex", "justify-content": "flex-start" }}>
      <div
        style={{
          "max-width": "94%",
          background: "#ffffff",
          color: "#1e293b",
          padding: "12px 14px",
          "border-radius": "4px 12px 12px 12px",
          border: "1px solid #e2e8f0",
          "font-size": `${props.fontSize}px`,
          "line-height": "1.6",
        }}
      >
        <Markdown
          text={stripAttachmentMarkers(props.content)}
          class="hf-share-card-md break-words"
        />
      </div>
    </div>
  );
}
