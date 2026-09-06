import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { Dynamic } from "solid-js/web";
import { useNavigate, useSearchParams } from "@solidjs/router";

import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import { PublicLoginForm } from "@/components/public-login-form";
import { PublicAdminUsagePanel } from "@/components/public-admin-usage-panel";
import { PublicAdminWhitelistPanel } from "@/components/public-admin-whitelist-panel";
import { ResearchState } from "@/components/research/research-state";
import { DailySignalPanel } from "@/components/daily-signal-dashboard";
import { CompanyRatingPanel } from "@/components/company-rating-dashboard";
import { PortfolioNewsPanel } from "@/components/portfolio-news-dashboard";
import { PositionManagementPanel } from "@/components/position-management-dashboard";
import { InfluencerDigestPanel } from "@/components/influencer-digest-dashboard";
import { WeeklyBriefPanel } from "@/components/weekly-brief-dashboard";
import { KeyEventChainPanel } from "@/components/key-event-chain-dashboard";
import { getPublicAuthMe, getPublicResearchOverview } from "@/lib/api";
import { workspaceUserName } from "@/lib/public-agent-workspace";
import {
  cachedPublicUser,
  hasCachedPublicUser,
  setCachedPublicUser,
} from "@/lib/public-session-cache";
import type {
  PublicAuthUserInfo,
  ResearchOverviewCard,
} from "@/lib/types";
import "./public-research.css";

/**
 * The research desk: every daily research product in one place.
 *
 * The grid reads like an app launcher rather than a wall of equal cards —
 * sections are grouped, a section that produced something today leads with
 * its finding, and one that has not is visibly dimmed with the time it
 * refreshes so nobody has to open it to discover there is nothing inside.
 * Opening a section writes `?panel=` so the view is shareable and the back
 * button closes it; the full snapshot is fetched only at that point.
 *
 * Every section also says, in one quiet line, what it is and how to read
 * it, and carries a 「问 HONE」 that hands its context to the assistant — the
 * desk used to assume the reader already knew what a "关键事件链" was for.
 *
 * Administrators see two views. Their own has the unreleased sections and
 * the admin panels; 「用户视角」 is exactly the page a reader gets, so what
 * has and has not shipped is never a guess.
 */

type PanelProps = { onClose: () => void };

type SectionDef = {
  key: string;
  title: string;
  kicker: string;
  group: GroupKey;
  /** Static one-liner shown until (or in place of) overview data. */
  blurb: string;
  /** What this section is, for a reader who has never opened it. */
  what: string;
  /** How to read it, in one sentence. */
  howTo: string;
  /** The question 「问 HONE」 hands to the assistant. */
  question?: string;
  /** When this section refreshes, shown while it is still waiting. */
  refreshAt: string;
  panel?: (props: PanelProps) => ReturnType<typeof DailySignalPanel>;
  /** Sections that are full pages navigate instead of opening a panel. */
  href?: string;
  /** Hidden entirely from non-administrators. */
  adminOnly?: boolean;
};

type GroupKey = "signal" | "industry" | "voices" | "admin";

/**
 * The macro light, the industry explorer and the commentator digest are
 * released to everyone. Everything else is still being polished, so it sits
 * behind 管理 where administrators can use and review it without users
 * meeting a half-finished product.
 */
const GROUPS: { key: GroupKey | "all"; label: string; adminOnly?: boolean }[] = [
  { key: "all", label: "全部" },
  { key: "signal", label: "大盘信号" },
  { key: "industry", label: "产业研究" },
  { key: "voices", label: "大V观点" },
  { key: "admin", label: "管理", adminOnly: true },
];

const SECTIONS: SectionDef[] = [
  {
    key: "daily-signal-macro",
    title: "宏观红绿灯",
    kicker: "领先周期判断",
    group: "signal",
    blurb: "收入 → 消费 → 生产 → 利润 → 资本开支",
    what: "按收入 → 消费 → 生产 → 利润 → 资本开支的领先顺序给今天的宏观周期打分并亮灯。",
    howTo: "先看颜色再看分数：绿灯偏机会、黄灯观望、红灯先控风险；点开看每一环的数据与变化。",
    question:
      "结合今天研究台的宏观红绿灯（分数与所处阶段），说明未来两周对我的持仓意味着什么，最该盯住的三个变量是什么。",
    refreshAt: "每日 20:00",
    panel: (props) => <DailySignalPanel kind="macro" {...props} />,
  },
  {
    key: "data-center",
    title: "3D 数据中心",
    kicker: "走进 AI 基础设施",
    group: "industry",
    blurb: "芯 · 存 · 光 · 电 · 冷 · AI 软件与云平台",
    what: "把 AI 数据中心拆成芯、存、光、电、冷与软件六层，点进去看每一层由谁供货。",
    howTo: "先看整机再点单层；每层的公司名可以直接拿去问投资助手。",
    refreshAt: "交互式产业地图",
    href: "/data-center",
  },
  {
    key: "industry-map",
    title: "行业分析",
    kicker: "产业链与关键变量",
    group: "industry",
    blurb: "行业树 · 上游信号 · 估值逻辑 · 相关公司",
    what: "AI 数据中心八条产业线的行业树、上游信号与估值逻辑，以及每条线上的公司。",
    howTo: "先看这一行的传导链和该盯的变量，再看它的成员公司；管理员可以在线改。",
    refreshAt: "研究底稿维护",
    href: "/industry-map",
  },
  {
    key: "influencer-digest",
    title: "大V速报",
    kicker: "观点不等于事实",
    group: "voices",
    blurb: "Serenity（白毛）等作者的推文时间线",
    what: "Serenity（白毛）等注册作者的公开推文，每 15 分钟同步成中文时间线；HONE 标出观点 / 事实与反方。",
    howTo: "按作者与类型筛选，先读原文再看解读；对某条有疑问，点那条下面的「问 HONE」带着原话追问。",
    question:
      "整理大V速报里 Serenity（白毛，@aleabitoreddit）最近三天推文的观点：她在关注哪些公司与瓶颈，哪些是事实、哪些是判断，与我的持仓有什么交集，以及需要验证的数据点。",
    refreshAt: "每 15 分钟",
    panel: (props) => <InfluencerDigestPanel {...props} />,
  },
  {
    key: "daily-signal-ai",
    title: "AI 红绿灯",
    kicker: "AI 增长可持续性",
    group: "admin",
    blurb: "需求旁证 · 商业化 · 融资 · 资本开支",
    what: "AI 增长可持续性打分：需求旁证、商业化、融资、资本开支。",
    howTo: "与宏观灯一起看：宏观红、AI 绿，说明结构性机会在但整体风险高。",
    question:
      "结合今天研究台的 AI 红绿灯，判断 AI 基础设施需求的可持续性，以及对我持仓里 AI 相关公司的影响。",
    refreshAt: "每日 20:00",
    panel: (props) => <DailySignalPanel kind="ai" {...props} />,
    adminOnly: true,
  },
  {
    key: "company-ratings",
    title: "公司评级",
    kicker: "52 家研究基线",
    group: "admin",
    blurb: "研究结构分与因子覆盖，缺数据时明示",
    what: "52 家覆盖公司的研究结构分与因子覆盖，缺数据时明示。",
    howTo: "找到你的持仓，看它在哪些因子上还缺证据。",
    question:
      "从今天研究台的公司评级里挑出我持仓中评分变化最大的公司，说明变化来自哪个因子、是否需要复核。",
    refreshAt: "每日 19:30",
    panel: (props) => <CompanyRatingPanel {...props} />,
    adminOnly: true,
  },
  {
    key: "valuation-lab",
    title: "估值实验室",
    kicker: "三情景估值",
    // 先停在管理分类里，等模型稳定后再对全部用户开放。
    group: "admin",
    blurb: "悲观 / 基准 / 乐观情景与关键假设",
    what: "悲观 / 基准 / 乐观三情景估值与关键假设，每日更新。",
    howTo: "看现价落在哪两个情景之间，再点开看假设是否还成立。",
    refreshAt: "每日 19:20",
    href: "/valuation-lab",
    adminOnly: true,
  },
  {
    key: "portfolio-news",
    title: "持仓重点新闻",
    kicker: "按你的持仓筛选",
    group: "admin",
    blurb: "每日新闻的持仓影响分析",
    what: "按你的持仓筛选当天新闻，并标注对每只持仓的影响。",
    howTo: "只看标了高影响的，再决定要不要追问。",
    question:
      "请复核今天研究台持仓重点新闻里对我持仓影响最大的三条，说明它们是否改变原判断。",
    refreshAt: "每日 20:00",
    panel: (props) => <PortfolioNewsPanel {...props} />,
    adminOnly: true,
  },
  {
    key: "position-management",
    title: "仓位管理",
    kicker: "评分 × 宏观 × 新闻",
    group: "admin",
    blurb: "结合评分与信号的每日仓位建议",
    what: "评分 × 宏观 × 新闻叠加后的每日仓位提示。",
    howTo: "它是提示不是指令：先看触发了哪条规则，再决定动不动。",
    question:
      "结合今天研究台的仓位管理提示，说明哪只持仓触发了调整信号、依据是什么、下一步的验证条件。",
    refreshAt: "每日 20:00",
    panel: (props) => <PositionManagementPanel {...props} />,
    adminOnly: true,
  },
  {
    key: "key-event-chain",
    title: "关键事件链",
    kicker: "第一性证据链",
    group: "admin",
    blurb: "AI 主题的里程碑与线索",
    what: "AI 主题十二层（模型、数据中心、HBM、光模块、CPO…）的里程碑与线索，分一手确认与待核实。",
    howTo: "先看「一手确认」再看线索；线索只能用来提问，不能当结论。",
    question:
      "从研究台关键事件链最近的一手确认里，找出对 AI 基础设施供需影响最大的三条变化，说明上下游影响与受益公司。",
    refreshAt: "每日 19:55",
    panel: (props) => <KeyEventChainPanel {...props} />,
    adminOnly: true,
  },
  {
    key: "weekly-brief",
    title: "周度简报",
    kicker: "回顾与前瞻",
    group: "admin",
    blurb: "上周回顾 · 下周日历 · 30 日展望",
    what: "上周回顾、下周日历与 30 日展望。",
    howTo: "周一早上先看它，再决定本周要盯什么。",
    question: "根据研究台的周度简报，列出下周最重要的三个事件以及它们对我持仓的潜在影响。",
    refreshAt: "每日 19:10",
    panel: (props) => <WeeklyBriefPanel {...props} />,
    adminOnly: true,
  },
  {
    key: "research-library",
    title: "研究文库",
    kicker: "知识源与投稿核验",
    group: "admin",
    blurb: "上传资料、核验投稿，注入研究对话",
    what: "上传资料、核验投稿，注入研究对话。",
    howTo: "上传后才会被引用；投稿要先核验。",
    refreshAt: "手动维护",
    href: "/research-library",
    adminOnly: true,
  },
];

const SIGNAL_LABELS: Record<string, string> = {
  green: "绿灯",
  yellow: "黄灯",
  orange: "橙灯",
  red: "红灯",
};

/** Statuses that mean "the job ran, there is simply nothing material today".
 *  `source_only` is not one of them: the digest and the event chain still
 *  carry their sources when the model did not run. */
const EMPTY_STATUSES = new Set(["no_material_news", "baseline", "no_updates"]);
/** Statuses that mean "no usable snapshot yet". */
const WAITING_STATUSES = new Set(["waiting", "data_unavailable", "source_unconfigured", ""]);

type CardState = "ready" | "empty" | "waiting";

function cardState(card: ResearchOverviewCard | undefined): CardState {
  if (!card) return "waiting";
  if (WAITING_STATUSES.has(card.status)) return "waiting";
  if (EMPTY_STATUSES.has(card.status)) return "empty";
  return "ready";
}

/** Whole days between two `YYYY-MM-DD` strings, or undefined if unparseable. */
function daysBetween(from: string, to: string) {
  const a = Date.parse(`${from}T00:00:00Z`);
  const b = Date.parse(`${to}T00:00:00Z`);
  if (Number.isNaN(a) || Number.isNaN(b)) return undefined;
  return Math.round((b - a) / 86_400_000);
}

/**
 * Only a section that deviates from today gets a label, and it says how.
 *
 * Two separate corrections meet here. The badge used to read "今日已更新" for
 * any snapshot that merely existed, so a report written last night still
 * claimed to be today's and the panel then opened on visibly older content.
 * And it was worn by six of eight cards at once — a mark repeating on almost
 * every card carries no information while costing a fixed slot in each.
 *
 * So freshness is measured against the server's calendar day, and a section
 * that is genuinely current says nothing at all: the header already prints
 * the "N / M 项已更新" tally, and the per-card slot is spent only on the ones
 * that deviate from it.
 */
function stateLabel(
  state: CardState,
  card: ResearchOverviewCard | undefined,
  today: string | undefined,
) {
  if (state === "waiting") return "等待数据";
  if (state === "empty") return "今日无新增";
  if (card?.status === "stale") return "沿用上次快照";
  const reportDate = card?.report_date ?? undefined;
  if (!reportDate || !today) return "";
  const age = daysBetween(reportDate, today);
  if (age === undefined || age <= 0) return "";
  return age === 1 ? "昨日更新" : `${age} 天前更新`;
}

/**
 * Today's date alone is not freshness: a section can be stamped today and
 * still have produced nothing usable. The header tally claims "there is
 * something here, and it is today's", so it requires both.
 */
function isFreshToday(
  card: ResearchOverviewCard | undefined,
  today: string | undefined,
  state?: CardState,
) {
  if (!card?.report_date || !today) return false;
  if (state && state !== "ready") return false;
  return (daysBetween(card.report_date, today) ?? 1) <= 0;
}

/**
 * The card blurb is one sentence, cut on a sentence boundary.
 *
 * A two-line `-webkit-line-clamp` alone truncates mid-clause — the AI card
 * read "…本版只使…" and the second line was spent saying nothing. Cutting at
 * the first terminator keeps the line whole; the clamp stays as the backstop
 * for a first sentence that is itself too long.
 */
function leadSentence(text: string) {
  const trimmed = text.trim();
  const end = trimmed.search(/[。；！？](?![）」』】])/);
  if (end === -1 || end > 56) return trimmed;
  const cut = trimmed.slice(0, end + 1);
  // 分号只是分句，留在句末会读成话还没说完。
  return cut.endsWith("；") ? cut.slice(0, -1) : cut;
}

/** Per-device memory of two small choices: which view, and whether the
 *  how-to strip is folded. Neither is worth a server round trip. */
const VIEW_STORAGE_KEY = "hone.research.view";
const GUIDE_STORAGE_KEY = "hone.research.guide";

function readStored(key: string) {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function writeStored(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    // Private mode or a blocked store: the choice simply lasts one visit.
  }
}

export default function PublicResearchPage() {
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const [user, setUser] = createSignal<PublicAuthUserInfo | null>(cachedPublicUser());
  const [authLoading, setAuthLoading] = createSignal(!hasCachedPublicUser());
  const [cards, setCards] = createSignal<Map<string, ResearchOverviewCard>>(new Map());
  const [reportToday, setReportToday] = createSignal<string>();
  const [overviewLoading, setOverviewLoading] = createSignal(true);
  const initialGroup = typeof searchParams.group === "string" ? searchParams.group : "";
  const [group, setGroup] = createSignal<GroupKey | "all">(
    GROUPS.some((item) => item.key === initialGroup)
      ? (initialGroup as GroupKey | "all")
      : "all",
  );
  // 管理员可以切到「用户视角」：看到的就是普通用户此刻看到的页面。
  const [viewAsUser, setViewAsUser] = createSignal(readStored(VIEW_STORAGE_KEY) === "user");
  const [guideOpen, setGuideOpen] = createSignal(readStored(GUIDE_STORAGE_KEY) !== "closed");
  let controller: AbortController | undefined;

  const loadAuth = async () => {
    try {
      setUser(await getPublicAuthMe());
    } catch {
      setUser(null);
      setCachedPublicUser(null);
    } finally {
      setAuthLoading(false);
    }
  };

  const loadOverview = async () => {
    controller?.abort();
    controller = new AbortController();
    setOverviewLoading(true);
    try {
      const payload = await getPublicResearchOverview(controller.signal);
      setCards(new Map(payload.cards.map((card) => [card.key, card])));
      setReportToday(payload.report_today ?? undefined);
    } catch {
      // The grid is a navigation surface first: cards fall back to their
      // static blurbs, every panel stays reachable.
    } finally {
      setOverviewLoading(false);
    }
  };

  onMount(() => {
    void loadAuth();
    void loadOverview();
  });
  onCleanup(() => controller?.abort());

  const isAdmin = createMemo(() => user()?.is_admin === true);

  /** Whether the page is drawn with the administrator's extras. */
  const adminView = createMemo(() => isAdmin() && !viewAsUser());

  const chooseView = (asUser: boolean) => {
    setViewAsUser(asUser);
    writeStored(VIEW_STORAGE_KEY, asUser ? "user" : "admin");
  };

  const toggleGuide = (open: boolean) => {
    setGuideOpen(open);
    writeStored(GUIDE_STORAGE_KEY, open ? "open" : "closed");
  };

  const visibleGroups = createMemo(() =>
    GROUPS.filter((item) => !item.adminOnly || adminView()),
  );

  const allowedSections = createMemo(() =>
    SECTIONS.filter((section) => !section.adminOnly || adminView()),
  );

  const activeSection = createMemo(() => {
    const key = typeof searchParams.panel === "string" ? searchParams.panel : undefined;
    return key
      ? allowedSections().find((section) => section.panel && section.key === key)
      : undefined;
  });

  // 非管理员即使拿到 ?group=admin 的链接也回落到「全部」，而不是看到空网格。
  const activeGroup = createMemo(() => {
    const current = group();
    return visibleGroups().some((item) => item.key === current) ? current : "all";
  });

  const visibleSections = createMemo(() => {
    const active = activeGroup();
    return active === "all"
      ? allowedSections()
      : allowedSections().filter((section) => section.group === active);
  });

  const dailySections = createMemo(() =>
    allowedSections().filter((section) => section.group !== "industry" && (section.panel || section.href)),
  );
  const industrySections = createMemo(() =>
    visibleSections().filter((section) => section.group === "industry"),
  );

  /**
   * Three tiers, outermost first.
   *
   * The desk used to open as eight identical cards — every section a closed
   * container you had to click to learn anything, including the two that ARE
   * the day's verdict. Now the outer layer states conclusions and the detail
   * stays behind the panel: the traffic lights lead with their score and
   * phase, sections that produced a finding print that finding as a line of
   * prose, and sections still waiting collapse into a single quiet row.
   */
  const verdictSections = createMemo(() =>
    visibleSections().filter((section) => section.group !== "industry" && cards().get(section.key)?.signal),
  );

  const findingSections = createMemo(() =>
    visibleSections().filter((section) => {
      const card = cards().get(section.key);
      return section.group !== "industry" && !card?.signal && cardState(card) === "ready";
    }),
  );

  const pendingSections = createMemo(() =>
    visibleSections().filter((section) => section.group !== "industry" && cardState(cards().get(section.key)) !== "ready"),
  );

  const readyCount = createMemo(
    () =>
      dailySections().filter((section) => {
        const card = cards().get(section.key);
        return isFreshToday(card, reportToday(), cardState(card));
      }).length,
  );

  const openSection = (section: SectionDef) => {
    if (section.href) {
      navigate(section.href);
      return;
    }
    // A plain push, so the browser back button closes the panel.
    setSearchParams({ panel: section.key });
  };

  const closePanel = () => setSearchParams({ panel: undefined });

  /** Hands the section's question to the assistant, which sends it at once. */
  const askHone = (section: SectionDef) => {
    if (!section.question) return;
    navigate(`/chat?q=${encodeURIComponent(section.question)}&send=1`);
  };

  const rowActions = (section: SectionDef) => (
    <span class="public-research-row__actions">
      <button type="button" onClick={() => openSection(section)}>
        {section.href ? "打开" : "看详情"}
      </button>
      <Show when={section.question}>
        <button type="button" class="public-research-tochat" onClick={() => askHone(section)}>
          问 HONE
        </button>
      </Show>
    </span>
  );

  return (
    <Show
      when={!authLoading() || user()}
      fallback={<div class="public-research-loading" role="status">正在进入研究台…</div>}
    >
      <Show when={user()} fallback={<PublicLoginForm onLogin={() => void loadAuth()} />}>
        {(currentUser) => (
          <PublicWorkspaceShell
            active="research"
            userName={workspaceUserName(currentUser().user_id)}
          >
            <main class="public-research-main">
              <header class="public-research-header">
                <div class="public-research-header__row">
                  <div>
                    <h1>研究台</h1>
                    <p>
                      HONE 每天自动跑的研究产品都在这里：先看结论，再点开看证据，有疑问直接问投资助手。
                      <Show when={!overviewLoading()}>
                        {" "}
                        <b>今日 {readyCount()} / {dailySections().length} 项已更新。</b>
                      </Show>
                      {" "}缺数据时明示，不用模拟值补位。
                    </p>
                  </div>
                  <Show when={isAdmin()}>
                    <div class="public-research-view" role="group" aria-label="视角">
                      <button
                        type="button"
                        classList={{ "is-active": !viewAsUser() }}
                        onClick={() => chooseView(false)}
                      >
                        管理员视角
                      </button>
                      <button
                        type="button"
                        classList={{ "is-active": viewAsUser() }}
                        onClick={() => chooseView(true)}
                      >
                        用户视角
                      </button>
                    </div>
                  </Show>
                </div>
                <Show when={isAdmin()}>
                  <p class="public-research-view__hint">
                    {viewAsUser()
                      ? "现在看到的就是普通用户看到的页面：只有已发布的板块。"
                      : "管理员视角多出未发布板块（标「未发布」）与管理面板；切到用户视角可以核对对外的样子。"}
                  </p>
                </Show>

                {/* 三步说明：研究台是什么、怎么用。折叠状态记在本机。 */}
                <details
                  class="public-research-guide"
                  open={guideOpen()}
                  onToggle={(event) => toggleGuide(event.currentTarget.open)}
                >
                  <summary>研究台是什么、怎么用</summary>
                  <ol>
                    <li>
                      <b>先看灯。</b>宏观红绿灯每天 20:00 给周期打分：绿灯偏机会、黄灯观望、红灯先控风险。
                    </li>
                    <li>
                      <b>再看人。</b>大V速报每 15 分钟同步 Serenity（白毛）等作者的推文，中文时间线，HONE 标出观点 / 事实与反方。
                    </li>
                    <li>
                      <b>有疑问就问。</b>每个板块都有「问 HONE」，会把板块的上下文带到投资助手接着追问；点「看详情」看完整证据。
                    </li>
                  </ol>
                </details>
              </header>

              <nav class="public-research-tabs" aria-label="研究分类">
                <For each={visibleGroups()}>
                  {(item) => (
                    <button
                      type="button"
                      classList={{ "is-active": activeGroup() === item.key }}
                      onClick={() => setGroup(item.key)}
                    >
                      {item.label}
                    </button>
                  )}
                </For>
              </nav>

              <Show
                when={!overviewLoading() || cards().size > 0}
                fallback={<ResearchState kind="loading" message="正在读取今日研究总览…" />}
              >
                <div class="public-research-layers">
                  <Show when={industrySections().length}>
                    <section class="public-research-findings" aria-label="产业研究入口">
                      <h2>探索 AI 基础设施</h2>
                      <For each={industrySections()}>
                        {(section) => (
                          <button type="button" onClick={() => openSection(section)}>
                            <span class="public-research-findings__label">{section.title}</span>
                            <span class="public-research-findings__text">{section.blurb}</span>
                            <span class="public-research-findings__metric" aria-hidden="true">↗</span>
                          </button>
                        )}
                      </For>
                    </section>
                  </Show>
                  {/* 判断层：红绿灯本身就是结论，不该藏在卡片后面。 */}
                  <Show when={verdictSections().length}>
                    <section class="public-research-verdicts">
                      <For each={verdictSections()}>
                        {(section) => {
                          const card = () => cards().get(section.key);
                          return (
                            <div
                              class="public-research-verdict"
                              classList={{ [`is-${card()?.signal ?? "none"}`]: true }}
                            >
                              <button
                                type="button"
                                class="public-research-verdict__open"
                                onClick={() => openSection(section)}
                              >
                                <span class="public-research-verdict__top">
                                  <strong>{section.title}</strong>
                                  <Show when={section.adminOnly}>
                                    <i class="public-research-gated">未发布</i>
                                  </Show>
                                  <b>{card()?.score?.toFixed(1)}</b>
                                  <em>{SIGNAL_LABELS[card()!.signal!] ?? card()!.signal}</em>
                                </span>
                                <span class="public-research-verdict__note">
                                  {leadSentence(card()?.summary || section.blurb)}
                                </span>
                                <span class="public-research-what">{section.howTo}</span>
                                <Show when={stateLabel(cardState(card()), card(), reportToday())}>
                                  {(age) => <span class="public-research-age">{age()}</span>}
                                </Show>
                              </button>
                              {rowActions(section)}
                            </div>
                          );
                        }}
                      </For>
                    </section>
                  </Show>

                  {/* 洞察层：一行一个结论，而不是一格一个入口。 */}
                  <Show when={findingSections().length}>
                    <section class="public-research-findings">
                      <h2>今日要点</h2>
                      <For each={findingSections()}>
                        {(section) => {
                          const card = () => cards().get(section.key);
                          return (
                            <div class="public-research-finding">
                              <button
                                type="button"
                                class="public-research-finding__open"
                                onClick={() => openSection(section)}
                              >
                                <span class="public-research-findings__label">
                                  {section.title}
                                  <Show when={section.adminOnly}>
                                    <i class="public-research-gated">未发布</i>
                                  </Show>
                                </span>
                                <span class="public-research-findings__text">
                                  {leadSentence(card()?.summary || section.blurb)}
                                  <small class="public-research-what">{section.what}</small>
                                </span>
                                <span class="public-research-findings__metric">
                                  <Show when={stateLabel(cardState(card()), card(), reportToday())}>
                                    {(age) => <b class="public-research-age">{age()}</b>}
                                  </Show>
                                  <Show when={card()?.metric}>{card()!.metric}</Show>
                                </span>
                              </button>
                              {rowActions(section)}
                            </div>
                          );
                        }}
                      </For>
                    </section>
                  </Show>

                  {/* 还没有内容的模块不配占一整格，一行说清楚它是什么、什么时候有。 */}
                  <Show when={pendingSections().length}>
                    <section class="public-research-pending">
                      <For each={pendingSections()}>
                        {(section) => {
                          const card = () => cards().get(section.key);
                          return (
                            <div class="public-research-pending__row">
                              <button type="button" onClick={() => openSection(section)}>
                                {section.title}
                                <Show when={section.adminOnly}>
                                  <i class="public-research-gated">未发布</i>
                                </Show>
                                <small>
                                  {stateLabel(cardState(card()), card(), reportToday()) || "今日无新增"} ·{" "}
                                  {section.refreshAt}
                                </small>
                              </button>
                              <span class="public-research-what">{section.what}</span>
                            </div>
                          );
                        }}
                      </For>
                    </section>
                  </Show>
                </div>
              </Show>

              <Show when={activeGroup() === "admin" && adminView()}>
                {/* 管理模块是宽表格，内联堆叠比塞进弹层可用得多。 */}
                <div class="public-research-admin">
                  <PublicAdminUsagePanel />
                  <PublicAdminWhitelistPanel />
                </div>
              </Show>

              <Show when={activeSection()}>
                {(section) => (
                  <Dynamic component={section().panel!} onClose={closePanel} />
                )}
              </Show>
            </main>
          </PublicWorkspaceShell>
        )}
      </Show>
    </Show>
  );
}
