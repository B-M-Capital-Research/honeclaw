// A development-only conversation for reviewing the chat surface without a
// backend that can answer: `/chat?demo=1` on a `vite dev` build seeds every
// turn state at once (finished answer with its folded work trail, a run in
// progress, a streaming answer, a failed turn and a scheduled push card).
// The figures are typographic samples, not research; production builds
// never reach this module.

import type { PublicAuthUserInfo } from "./types";
import type { PublicChatMessage } from "./public-chat";

const DEMO_ANSWER = `**结论：营收与 EPS 双超预期，但市场先看 AI 服务器毛利率。** 盘后股价一度冲高后回落，说明分歧不在需求，而在"高投入能否变现"——本轮指引把 AI 服务器全年出货上修到 200 亿美元，却没有同步上修自由现金流。

## 已核验事实

| 指标 | 实际 | 一致预期 | 差异 | 口径 |
| --- | ---: | ---: | ---: | --- |
| 营收 | 297.8 亿 | 291.2 亿 | +2.3% | 美元 · GAAP |
| 调整后 EPS | 2.32 | 2.26 | +2.7% | non-GAAP |
| AI 服务器出货 | 82 亿 | 70 亿 | +17% | ISG 分部 |
| ISG 营业利润率 | 8.6% | 9.4% | −0.8pt | mix 拖累 |

## 改写了哪个长期变量

没有改写需求曲线，改写的是**单位经济**：AI 服务器每一美元营收带来的营业利润从 0.11 降到 0.086。若 FY27 仍无法回到 10% 以上，DCF 里的终值利润率假设要下调 1 个百分点，对应合理价区间从 \`$138–$176\` 收窄到 \`$131–$168\`。

## 防守与减仓触发条件

1. 若财报后冲高至 **190 美元** 以上但自由现金流指引未改善，顺势兑现部分浮盈；
2. 若跌破 **122 美元**（悲观锚点下沿），"高投入-难变现"循环成真，严格止损。

> 提示：以上分析基于公开财务报表与市场一致预期，不构成未经独立风险评估的投资操作保证。`;

const DEMO_STREAMING = `**结论：指引上修主要来自网络侧，光模块链受益顺序是 1.6T 先于 800G。** 数据中心营收指引 5,400 亿美元的隐含增速里，网络业务贡献了超过一半的增量，这意味着`;

export function buildPublicChatDemo(now = Date.now()): {
  user: PublicAuthUserInfo;
  messages: PublicChatMessage[];
} {
  const at = (minutesAgo: number) => new Date(now - minutesAgo * 60_000).toISOString();
  const user: PublicAuthUserInfo = {
    user_id: "demo-0001",
    created_at: at(60 * 24 * 30),
    daily_limit: 30,
    success_count: 18,
    in_flight: 0,
    remaining_today: 12,
    has_password: true,
    identity_kind: "domestic_invite",
    billing: {} as PublicAuthUserInfo["billing"],
    is_admin: false,
  };
  const messages: PublicChatMessage[] = [
    {
      id: "demo-u-1",
      role: "user",
      content: "戴尔这次财报超预期吗？AI 服务器订单怎么看",
      at: at(42),
      phase: "done",
    },
    {
      id: "demo-a-1",
      role: "assistant",
      content: DEMO_ANSWER,
      at: at(41),
      phase: "done",
      steps: [
        "核验实体：Dell Technologies（DELL）FY26 Q2",
        "读取营收 / EPS 与一致预期",
        "查询 AI 服务器订单、backlog 与盘后报价",
        "估值再锚定与情景推演",
      ],
      startedAt: now - 41 * 60_000 - 23_000,
      finishedAt: now - 41 * 60_000,
    },
    {
      id: "demo-a-push",
      role: "assistant",
      content: "",
      at: at(30),
      phase: "done",
      scheduledPush: {
        pushId: "demo-push",
        title: "持仓晨报 · 9 月 6 日",
        summary:
          "英伟达盘前 +1.8%，10 年期美债 4.27%；今晚 20:30 非农就业，持仓里 AVGO 财报在周四盘后。",
        fallbackContent: "示例推送内容。",
        createdAt: at(30),
      },
    },
    {
      id: "demo-u-2",
      role: "user",
      content: "英伟达 FY27Q2 指引对光模块链的影响？",
      at: at(3),
      phase: "done",
    },
    {
      id: "demo-a-2",
      role: "assistant",
      content: DEMO_STREAMING,
      at: at(2),
      phase: "streaming",
      statusText: "HONE 输出中",
      steps: ["核验实体：NVIDIA（NVDA）FY27 Q2", "读取数据中心分部指引", "整理光模块链上市公司名单"],
      startedAt: now - 48_000,
    },
    {
      id: "demo-u-3",
      role: "user",
      content: "10 年期美债破 4.3% 对成长股估值的传导",
      at: at(1),
      phase: "done",
    },
    {
      id: "demo-a-3",
      role: "assistant",
      content: "",
      at: at(1),
      phase: "running",
      statusText: "查询 10 年期与 30 年期美债收益率、VIX 与期限溢价",
      startedAt: now - 23_000,
      steps: [
        "核验问题范围：美债收益率 → 成长股估值",
        "读取政策利率与就业、通胀最新值",
        "查询 10 年期与 30 年期美债收益率、VIX 与期限溢价",
      ],
      reasoningLog:
        "先把收益率上行拆成实际利率和通胀预期两部分，再看哪一部分在驱动本轮；对成长股估值的传导要落到贴现率变化对 DCF 终值的敏感度上，而不是泛泛地说利率高压制估值。",
    },
    {
      id: "demo-a-4",
      role: "assistant",
      content: "",
      phase: "error",
      statusText: "请求出错，请重试。",
      steps: [],
    },
  ];
  return { user, messages };
}
