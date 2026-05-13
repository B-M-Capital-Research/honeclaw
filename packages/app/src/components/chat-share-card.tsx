// Offscreen-rendered card that gets fed into html2canvas to produce the
// shareable image. Layout is fully self-contained — no Tailwind, no
// inherited chat-page CSS — so a screenshot from any device looks the same.

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
  /** Mounted as visible (`false`) vs. hidden offscreen (`true`). */
  hidden?: boolean;
  /** Refs the wrapper element so the caller can hand it to html2canvas. */
  registerRef?: (el: HTMLDivElement) => void;
};

const CARD_WIDTH = 600;

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
    line-height: 1.6;
    white-space: pre-wrap;
    word-break: break-word;
    overflow-wrap: anywhere;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
  }
  .hf-share-card-md .hf-markdown-code code {
    background: transparent !important;
    padding: 0;
    font-family: inherit;
  }
  .hf-share-card-md :not(pre) > code {
    background: rgba(15, 23, 42, 0.06);
    border-radius: 4px;
    padding: 1px 6px;
    font-size: 0.92em;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
  }
`;

export function ChatShareCard(props: ChatShareCardProps) {
  const [qrDataUrl, setQrDataUrl] = createSignal<string>("");

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
            gap: "12px",
            padding: "28px 32px 20px 32px",
            "border-bottom": "1px solid #f1f5f9",
          }}
        >
          <img
            src="/logo.svg"
            alt=""
            crossorigin="anonymous"
            style={{ width: "36px", height: "36px" }}
          />
          <div style={{ display: "flex", "flex-direction": "column" }}>
            <span
              style={{
                "font-size": "18px",
                "font-weight": "800",
                "letter-spacing": "0.02em",
                color: "#0f172a",
              }}
            >
              {props.brandName}
            </span>
            <span style={{ "font-size": "12px", color: "#64748b", "margin-top": "2px" }}>
              {props.brandTagline}
            </span>
          </div>
        </div>

        {/* Messages */}
        <div
          style={{
            padding: "24px 32px",
            display: "flex",
            "flex-direction": "column",
            gap: "16px",
          }}
        >
          <For each={props.messages}>
            {(msg) => (
              <Show when={msg.role === "user"} fallback={<AssistantRow content={msg.content} />}>
                <UserRow content={msg.content} />
              </Show>
            )}
          </For>
        </div>

        {/* Footer */}
        <div
          style={{
            display: "flex",
            "align-items": "center",
            "justify-content": "space-between",
            gap: "20px",
            padding: "20px 32px 28px 32px",
            "border-top": "1px solid #f1f5f9",
            background: "#fafbfc",
          }}
        >
          <div style={{ display: "flex", "flex-direction": "column", gap: "6px" }}>
            <div style={{ display: "flex", "align-items": "center", gap: "10px" }}>
              <img
                src="/logo.svg"
                alt=""
                crossorigin="anonymous"
                style={{ width: "28px", height: "28px" }}
              />
              <span
                style={{
                  "font-size": "16px",
                  "font-weight": "800",
                  color: "#0f172a",
                  "letter-spacing": "0.02em",
                }}
              >
                {props.brandName}
              </span>
            </div>
            <span style={{ "font-size": "12px", color: "#64748b", "max-width": "360px" }}>
              {props.qrCaption}
            </span>
          </div>
          <Show when={qrDataUrl()}>
            <img
              src={qrDataUrl()}
              alt=""
              style={{
                width: "84px",
                height: "84px",
                "border-radius": "10px",
                background: "#fff",
                padding: "6px",
                border: "1px solid #e2e8f0",
              }}
            />
          </Show>
        </div>
      </div>
    </div>
  );
}

function UserRow(props: { content: string }) {
  const cleaned = () => stripAttachmentMarkers(props.content);
  return (
    <div style={{ display: "flex", "justify-content": "flex-end" }}>
      <div
        style={{
          "max-width": "82%",
          background: "#0f172a",
          color: "#f8fafc",
          padding: "12px 16px",
          "border-radius": "14px 14px 4px 14px",
          "font-size": "14.5px",
          "line-height": "1.6",
          "white-space": "pre-wrap",
          "word-break": "break-word",
        }}
      >
        {cleaned()}
      </div>
    </div>
  );
}

function AssistantRow(props: { content: string }) {
  return (
    <div style={{ display: "flex", "justify-content": "flex-start" }}>
      <div
        style={{
          "max-width": "92%",
          background: "#ffffff",
          color: "#1e293b",
          padding: "14px 18px",
          "border-radius": "4px 14px 14px 14px",
          border: "1px solid #e2e8f0",
          "font-size": "14.5px",
          "line-height": "1.65",
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
