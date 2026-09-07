// Dev-only preview route used to verify the rendered share card visually
// without going through the full chat flow. Routed at /__share-preview.
//
// Two samples: the prose + code card the route has always shown, and a
// tables card rendered twice — as the conversation lays it out and as the
// card reshapes it — so the table reflow can be judged side by side.

import { For, createSignal } from "solid-js";
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
- PB 更偏 "资产价值估值"，常用于金融、地产等重资产行业的横向比较。

据公开报道与发布信息，OpenAI 于 2026 年 9 月 3 日正式发布 GPT-6 Astra（内部代号 \`gpt-6-astra\`），并在 9 月 4 日进入分阶段推送，其前代为 \`gpt-5.6-sol\`；接口层面 \`security-listing-evidence-status=active-listing-with-quote-fallback\` 为新增字段。`,
  },
];

const TABLE_MESSAGES: PublicChatMessage[] = [
  {
    id: "u2",
    role: "user",
    content: "GPT-6 Astra 发布后，主要 AI 基础设施公司的事件影响帮我列个表。",
  },
  {
    id: "a2",
    role: "assistant",
    content: `**三、主要上市公司事件与影响矩阵**

以下按照 GPT-6 Astra 发布后一周内的公开信息整理，置信度为主观判断。

| 公司 | 事件 | 时间 | 类型 | 影响 | 期限 | 来源 | 置信度 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| NVDA | GPT-6 Astra 训练集群确认采用 GB300 NVL72，推理侧新增 Rubin CPX 订单 | 2026-09-03 | 订单 / 产品 | 正面：推理算力需求上修，FY27 数据中心营收指引上调 | 6-12 个月 | OpenAI 发布会、NVDA 8-K | 高 |
| MSFT | Azure 独家承载 GPT-6 Astra API 首发，Copilot 全线升级 | 2026-09-04 | 渠道 / 分成 | 正面：Azure AI 收入贡献占比抬升，资本开支同步上修 | 3-6 个月 | MSFT 官方博客 | 高 |
| AVGO | 为 OpenAI 定制 XPU 进入量产，2027 财年出货指引 | 2026-09-05 | 定制芯片 | 正面：ASIC 收入进入兑现期，估值锚由 Wi-Fi/存储切换至 AI | 12 个月以上 | AVGO 财报电话会 | 中 |
| AMZN | Anthropic 追加 Trainium 3 采购，对冲 OpenAI 生态 | 2026-09-05 | 反向对冲 | 中性偏正：AWS 自研芯片渗透率抬升，但对 GPT-6 生态暴露度低 | 6-12 个月 | AWS re:Invent 预告 | 中 |

**四、两家 GPU 供应商的季度对比**

| 公司 | 营收（亿美元） | 同比 | 净利润（亿美元） | EPS | PE（TTM） | 目标价 |
| --- | --- | --- | --- | --- | --- | --- |
| NVDA | 468.2 | +56% | 264.4 | 1.08 | 42.6 | $215 |
| AMD | 92.4 | +32% | 12.1 | 0.74 | 58.3 | $198 |

**五、关键指标口径**

| 指标 | 数值 | 说明 |
| --- | --- | --- |
| 数据中心营收 | 412 亿 | 含网络业务 |
| 毛利率 | 74.8% | Non-GAAP |
| 资本开支 | 38 亿 | 季度值 |`,
  },
];

type Sample = {
  label: string;
  messages: PublicChatMessage[];
  tableLayout: "auto" | "table";
};

const SAMPLES: Sample[] = [
  { label: "Prose + code", messages: SAMPLE_MESSAGES, tableLayout: "auto" },
  { label: "Tables — as the conversation lays them out", messages: TABLE_MESSAGES, tableLayout: "table" },
  { label: "Tables — reshaped for the card", messages: TABLE_MESSAGES, tableLayout: "auto" },
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
        backgroundColor: "#fffdf8",
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
        {busy() ? "Rendering…" : "Export PNG (reshaped tables card)"}
      </button>
      <div style={{ display: "flex", "flex-wrap": "wrap", gap: "24px", "align-items": "flex-start" }}>
        <For each={SAMPLES}>
          {(sample) => (
            <div data-share-sample={sample.tableLayout}>
              <div style={{ "font-family": "sans-serif", "font-size": "12px", color: "#475569", "margin-bottom": "8px" }}>
                {sample.label}
              </div>
              <ChatShareCard
                messages={sample.messages}
                brandName="HONE"
                brandTagline="Sharpen your edge."
                qrUrl="https://hone-claw.com/chat"
                qrCaption="Scan to try HONE Chat"
                disclaimer="Research reference only; not investment advice"
                tableLayout={sample.tableLayout}
                registerRef={(el) => {
                  if (sample.tableLayout === "auto" && sample.messages === TABLE_MESSAGES) {
                    setRegistered(el);
                  }
                }}
              />
            </div>
          )}
        </For>
      </div>
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
