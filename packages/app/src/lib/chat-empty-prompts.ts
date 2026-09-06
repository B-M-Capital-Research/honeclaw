import type { FinanceCalendarEvent, InfluencerDigestSnapshot } from "./types";
import type { Locale } from "./i18n";

export type ChatStarterPrompt = {
  id: "macro" | "portfolio" | "calendar" | "influencer" | "ai-infra" | "industry" | "valuation";
  eyebrow: string;
  title: string;
  question: string;
};

type ChatStarterPromptInput = {
  holdings?: string[];
  events?: FinanceCalendarEvent[];
  today?: string;
  /** The latest commentator digest, so the Serenity hook can quote her newest post. */
  influencer?: InfluencerDigestSnapshot | null;
  locale?: Locale;
};

function cleanHoldings(holdings: string[]) {
  return [...new Set(holdings.map((item) => item.trim().toUpperCase()).filter(Boolean))]
    .slice(0, 3);
}

function nextCalendarEvent(events: FinanceCalendarEvent[], today: string) {
  return [...events]
    .filter((event) => event.date.slice(0, 10) >= today)
    .sort((left, right) => left.date.localeCompare(right.date))[0];
}

/**
 * The newest Serenity post in the digest, flattened to one line.
 *
 * Only Serenity is quoted by name on the home screen: she is the author the
 * digest exists for, and a hook that says "SemiAnalysis just wrote…" about an
 * essay title would not read as a question anyone wants to ask.
 */
function latestSerenityPost(snapshot: InfluencerDigestSnapshot | null | undefined) {
  const item = snapshot?.items?.find((entry) => entry.author_id === "serenity");
  if (!item) return undefined;
  const raw = (item.source_text_cn || item.source_text_en || item.source_excerpt || item.title || "")
    .replace(/https?:\/\/\S+/g, "")
    .trim();
  const text = raw.replace(/\s+/g, " ").trim();
  if (!text) return undefined;
  // The hook's title is the post's first sentence, not its first N
  // characters: a cut mid-clause reads as noise on a card.
  const lead = raw.split(/(?<=[。！？!?])|\n/).map((part) => part.trim()).find(Boolean) ?? text;
  return {
    when: item.published_at_local,
    text,
    lead,
    tickers: item.tickers ?? [],
  };
}

function clip(text: string, max: number) {
  const chars = [...text];
  return chars.length > max ? `${chars.slice(0, max).join("")}…` : text;
}

export function buildChatStarterPrompts({
  holdings = [],
  events = [],
  today = new Date().toISOString().slice(0, 10),
  influencer,
  locale = "zh",
}: ChatStarterPromptInput): ChatStarterPrompt[] {
  const symbols = cleanHoldings(holdings);
  const holdingsLabel = symbols.length > 0 ? symbols.join("、") : "我的持仓";
  const firstHolding = symbols[0];
  const nextEvent = nextCalendarEvent(events, today);
  const latest = latestSerenityPost(influencer);

  if (locale === "en") {
    const holdingsLabelEn = symbols.length > 0 ? symbols.join(", ") : "my holdings";
    return [
      {
        id: "macro",
        eyebrow: "Macro",
        title: "What will markets price most heavily over the next two weeks?",
        question:
          "Using the latest policy rate, 10-year and 30-year Treasury yields, employment, inflation and VIX, decide whether the next two weeks are an opportunity, hold or risk regime and name the three changes that matter most.",
      },
      {
        id: "portfolio",
        eyebrow: "Portfolio",
        title: symbols.length > 0
          ? `Which of ${holdingsLabelEn} needs review first today?`
          : "What should I check first in my portfolio today?",
        question: symbols.length > 0
          ? `Review my holdings ${holdingsLabelEn} using current news, earnings, valuation and key risks. Identify the one name that needs review first and give short-, medium- and long-term conclusions.`
          : "Using my actual holdings, current news and valuation, identify the three names that need review first today and state the reason, risk and next validation condition.",
      },
      {
        id: "calendar",
        eyebrow: "Calendar",
        title: nextEvent
          ? `Next key event: ${nextEvent.title}`
          : "Which earnings and macro events matter over the next week?",
        question: nextEvent
          ? `The calendar shows “${nextEvent.title}” on ${nextEvent.date.slice(0, 10)}. Verify the latest official time and explain how it could affect my holdings; do not guess unreleased results.`
          : "List the most important macro releases, AI-company earnings and industry conferences over the next week and explain how they could affect my holdings. Use official dates and do not invent unconfirmed events.",
      },
      {
        id: "influencer",
        eyebrow: "Serenity",
        title: latest
          ? `Serenity, ${latest.when}: ${clip(latest.lead, 40)}`
          : "What is Serenity (@aleabitoreddit) focused on right now?",
        question: latest
          ? `Serenity (@aleabitoreddit) posted on ${latest.when}: “${clip(latest.text, 220)}”. Using the commentator digest and verifiable market, filing and news evidence, explain what she is saying, separate fact from her judgment, state what it means for ${latest.tickers.length ? latest.tickers.map((ticker) => `$${ticker}`).join(", ") : "the companies involved"} and my holdings, and list what still needs verification.`
          : "Summarize Serenity's (@aleabitoreddit) views from the last three days in the commentator digest: which companies and bottlenecks she is watching, what is fact versus her judgment, where it overlaps with my holdings, and what still needs verification.",
      },
      {
        id: "ai-infra",
        eyebrow: "AI infrastructure",
        title: "Where is the AI infrastructure chain tightest right now?",
        question:
          "Walk the AI data-center chain (GPU/ASIC → HBM and storage → optical modules and CPO → power and cooling → hyperscaler capex). Using the latest quarterly results, guidance and capacity signals from each layer's leaders, identify the tightest bottleneck, the layer that is loosening, the companies that benefit or suffer, and the next data points to verify.",
      },
      {
        id: "industry",
        eyebrow: "Event chains",
        title: "Which recent Rubin, HBM or CPO change matters most?",
        question:
          "Across Rubin, HBM/HBF, CPO/NPO, 800G/1.6T, data centers and AI models, identify the three most important recently confirmed changes and explain the upstream/downstream impact, beneficiaries and next milestone to verify.",
      },
      {
        id: "valuation",
        eyebrow: "Valuation",
        title: firstHolding
          ? `Where does ${firstHolding}'s current price sit in its valuation range?`
          : "Which priority companies offer more opportunity than risk today?",
        question: firstHolding
          ? `Value ${firstHolding} with multiple methods suited to its business model. Give bear, base and bull cases, the current-price position and reverse valuation, then classify it as opportunity, hold or risk.`
          : "Select three HONE priority companies where opportunity currently exceeds risk. Use current data and multiple valuation methods, with bear, base and bull cases.",
      },
    ];
  }

  return [
    {
      id: "macro",
      eyebrow: "宏观",
      title: "未来两周，市场最大的定价变量是什么？",
      question:
        "结合最新的利率、10年与30年美债、就业、通胀和 VIX，直接判断未来两周宏观环境是机会区、持有区还是风险区，并说明最需要盯住的三个变化。",
    },
    {
      id: "portfolio",
      eyebrow: "持仓",
      title:
        symbols.length > 0
          ? `${holdingsLabel}，今天哪个需要优先复核？`
          : "我的持仓里，今天最该先检查什么？",
      question:
        symbols.length > 0
          ? `请检查我的持仓 ${holdingsLabel}：结合最新新闻、财报、估值与关键风险，直接指出今天最需要优先复核的一只，并给出短期、中期和长期判断。`
          : "请结合我的真实持仓、最新新闻和估值，列出今天最需要优先复核的三只股票，并明确原因、风险和下一步观察条件。",
    },
    {
      id: "calendar",
      eyebrow: "重要日程",
      title: nextEvent
        ? `下个关键事件：${nextEvent.title}`
        : "未来一周有哪些财报和宏观事件？",
      question: nextEvent
        ? `财经日历显示下一个重要事件是“${nextEvent.title}”（${nextEvent.date.slice(0, 10)}）。请核对最新官方时间，分析它可能如何影响我的持仓；尚未公布的结果不要猜。`
        : "请整理未来一周最重要的宏观数据、AI 公司财报和产业会议，并分析它们可能如何影响我的持仓。日期必须引用官方来源，未确认的不要补造。",
    },
    {
      id: "influencer",
      eyebrow: "大V速报 · Serenity",
      title: latest
        ? `白毛 ${latest.when}：${clip(latest.lead, 30)}`
        : "Serenity（白毛）最近在关注什么？",
      question: latest
        ? `Serenity（白毛，@aleabitoreddit）${latest.when} 发推：“${clip(latest.text, 220)}”。请结合大V速报和本轮可核验的行情、财报与新闻说明：这条在讲什么，哪些是事实、哪些是作者判断，对 ${latest.tickers.length ? latest.tickers.map((ticker) => `$${ticker}`).join("、") : "相关公司"} 和我的持仓意味着什么，以及接下来需要验证的数据点。`
        : "整理大V速报里 Serenity（白毛，@aleabitoreddit）最近三天推文的观点：她在关注哪些公司与瓶颈，哪些是事实、哪些是判断，与我的持仓有什么交集，以及需要验证的数据点。",
    },
    {
      id: "ai-infra",
      eyebrow: "AI 基础设施",
      title: "AI 基础设施这条链，现在最紧的是哪一环？",
      question:
        "沿 AI 数据中心产业链（GPU/ASIC → HBM 与存储 → 光模块与 CPO → 电力与散热 → 云厂资本开支），结合各环节龙头最新一季财报、指引与产能信号，判断当前最紧的瓶颈环节、正在松动的环节、对应受益与承压的公司，并给出下一步要验证的数据点。",
    },
    {
      id: "industry",
      eyebrow: "关键事件链",
      title: "Rubin、HBM、CPO，最近哪条变化最关键？",
      question:
        "从 Rubin、HBM/HBF、CPO/NPO、800G/1.6T、数据中心和 AI 模型中，找出最近最重要的三条已确认变化，说明上下游影响、受益公司和需要继续验证的里程碑。",
    },
    {
      id: "valuation",
      eyebrow: "估值",
      title: firstHolding
        ? `${firstHolding} 当前价格处在什么估值位置？`
        : "重点公司里，今天谁的机会大于风险？",
      question: firstHolding
        ? `请用多种适合其商业模式的估值方法评估 ${firstHolding}，给出悲观、基准、乐观三种情景、当前价格位置和反向估值，并明确现在属于机会区、持有区还是风险区。`
        : "从 HONE 重点公司中筛选三家当前机会大于风险的公司。估值必须使用最新数据和多方法交叉验证，并给出悲观、基准、乐观情景。",
    },
  ];
}
