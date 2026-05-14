// Dev-only preview route used to verify the rendered share card visually
// without going through the full chat flow. Mount under /__share-preview.

import { createSignal } from "solid-js";
import { ChatShareCard } from "@/components/chat-share-card";
import type { PublicChatMessage } from "@/lib/public-chat";

const SAMPLE_MESSAGES: PublicChatMessage[] = [
  {
    id: "u1",
    role: "user",
    content: "解释一下 PE 和 PB 的区别，最好给我一段公式示例。",
  },
  {
    id: "a1",
    role: "assistant",
    content: `**PE 与 PB 的核心区别**

- **PE（市盈率）**：股价相对于 **每股盈利** 的倍数，反映"赚钱能力估值"。
- **PB（市净率）**：股价相对于 **每股净资产** 的倍数，反映"资产价值估值"。

\`\`\`text
PE = 股价 / 每股盈利 (EPS)
PB = 股价 / 每股净资产 (BVPS)

示例：某公司股价 = 30 元
  EPS  = 2 元   →  PE = 30 / 2   = 15 倍
  BVPS = 20 元  →  PB = 30 / 20  = 1.5 倍
\`\`\`

- PE 更偏 "赚钱能力估值"，对盈利波动的公司（如周期股、亏损股）参考价值较低。
- PB 更偏 "资产价值估值"，常用于金融、地产等重资产行业的横向比较。`,
  },
];

export default function SharePreviewPage() {
  const [registered, setRegistered] = createSignal<HTMLDivElement | null>(null);
  const [pngUrl, setPngUrl] = createSignal<string>("");
  const [busy, setBusy] = createSignal(false);

  const exportPng = async () => {
    const el = registered();
    if (!el || busy()) return;
    setBusy(true);
    try {
      const { default: html2canvas } = await import("html2canvas");
      const canvas = await html2canvas(el, {
        scale: 2,
        backgroundColor: "#ffffff",
        useCORS: true,
        logging: false,
      });
      const dataUrl = canvas.toDataURL("image/png");
      setPngUrl(dataUrl);
      (window as any).__sharePngDataUrl = dataUrl;
    } finally {
      setBusy(false);
    }
  };

  (window as any).__exportSharePng = exportPng;

  return (
    <div style={{ padding: "24px", background: "#f1f5f9", "min-height": "100vh" }}>
      <h1 style={{ "font-family": "sans-serif", "font-size": "16px", "margin-bottom": "12px" }}>
        Share card preview
      </h1>
      <button
        onClick={exportPng}
        disabled={busy()}
        style={{ "margin-bottom": "16px", padding: "8px 16px" }}
      >
        {busy() ? "Rendering…" : "Export PNG"}
      </button>
      <ChatShareCard
        messages={SAMPLE_MESSAGES}
        brandName="Hone"
        brandTagline="Sharpen your edge."
        qrUrl="https://hone-claw.com/chat"
        qrCaption="Scan to try Hone Chat"
        registerRef={(el) => setRegistered(el)}
      />
      {pngUrl() && (
        <div style={{ "margin-top": "24px" }}>
          <div style={{ "font-family": "sans-serif", "font-size": "13px", "margin-bottom": "8px" }}>
            Rasterized output:
          </div>
          <img
            src={pngUrl()}
            alt="rendered share card"
            style={{ "max-width": "420px", border: "1px solid #cbd5e1" }}
          />
        </div>
      )}
    </div>
  );
}
