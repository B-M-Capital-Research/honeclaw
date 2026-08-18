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
 */

type PanelProps = { onClose: () => void };

type SectionDef = {
  key: string;
  title: string;
  kicker: string;
  group: GroupKey;
  /** Static one-liner shown until (or in place of) overview data. */
  blurb: string;
  /** When this section refreshes, shown while it is still waiting. */
  refreshAt: string;
  panel?: (props: PanelProps) => ReturnType<typeof DailySignalPanel>;
  /** Sections that are full pages navigate instead of opening a panel. */
  href?: string;
  /** Hidden entirely from non-administrators. */
  adminOnly?: boolean;
};

type GroupKey = "signal" | "admin";

/**
 * Only the macro light is released to everyone for now. Everything else is
 * still being polished, so it sits behind 管理 where administrators can use
 * and review it without users meeting a half-finished product.
 */
const GROUPS: { key: GroupKey | "all"; label: string; adminOnly?: boolean }[] = [
  { key: "all", label: "全部" },
  { key: "signal", label: "大盘信号" },
  { key: "admin", label: "管理", adminOnly: true },
];

const SECTIONS: SectionDef[] = [
  {
    key: "daily-signal-macro",
    title: "宏观红绿灯",
    kicker: "领先周期判断",
    group: "signal",
    blurb: "收入 → 消费 → 生产 → 利润 → 资本开支",
    refreshAt: "每日 20:00",
    panel: (props) => <DailySignalPanel kind="macro" {...props} />,
  },
  {
    key: "daily-signal-ai",
    title: "AI 红绿灯",
    kicker: "AI 增长可持续性",
    group: "admin",
    blurb: "需求旁证 · 商业化 · 融资 · 资本开支",
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
    refreshAt: "每日 20:00",
    panel: (props) => <PositionManagementPanel {...props} />,
    adminOnly: true,
  },
  {
    key: "influencer-digest",
    title: "大V速报",
    kicker: "观点不等于事实",
    group: "admin",
    blurb: "注册作者的公开观点日报",
    refreshAt: "每日 19:50",
    panel: (props) => <InfluencerDigestPanel {...props} />,
    adminOnly: true,
  },
  {
    key: "key-event-chain",
    title: "关键事件链",
    kicker: "第一性证据链",
    group: "admin",
    blurb: "AI 主题的里程碑与线索",
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

/** Statuses that mean "the job ran, there is simply nothing material today". */
const EMPTY_STATUSES = new Set(["no_material_news", "source_only", "baseline"]);
/** Statuses that mean "no usable snapshot yet". */
const WAITING_STATUSES = new Set(["waiting", "data_unavailable", ""]);

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
 * How fresh this section actually is.
 *
 * The label used to read "今日已更新" for any snapshot that merely existed, so
 * a report written last night still claimed to be today's — the panel then
 * opened on visibly older content and the card looked like a lie. The day is
 * compared against the server's calendar day, and anything older says so.
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
  if (!reportDate || !today) return "已更新";
  const age = daysBetween(reportDate, today);
  if (age === undefined) return "已更新";
  if (age <= 0) return "今日已更新";
  if (age === 1) return "昨日更新";
  return `${age} 天前更新`;
}

/**
 * Today's date alone is not freshness: a section can be stamped today and
 * still have produced nothing usable. The green marker claims "there is
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

  const activeSection = createMemo(() => {
    const key = typeof searchParams.panel === "string" ? searchParams.panel : undefined;
    return key ? SECTIONS.find((section) => section.panel && section.key === key) : undefined;
  });

  const isAdmin = createMemo(() => user()?.is_admin === true);

  const visibleGroups = createMemo(() =>
    GROUPS.filter((item) => !item.adminOnly || isAdmin()),
  );

  const allowedSections = createMemo(() =>
    SECTIONS.filter((section) => !section.adminOnly || isAdmin()),
  );

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
    allowedSections().filter((section) => section.panel || section.href),
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
                <div>
                  <h1>研究台</h1>
                  <p>
                    每日研究产品的家。
                    <Show when={!overviewLoading()}>
                      <b>今日 {readyCount()} / {dailySections().length} 项已更新。</b>
                    </Show>
                    缺数据时明示，不用模拟值补位。
                  </p>
                </div>
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
                <div class="public-research-grid">
                  <For each={visibleSections()}>
                    {(section) => {
                      const card = () => cards().get(section.key);
                      const state = () => cardState(card());
                      return (
                        <button
                          type="button"
                          class="public-research-card"
                          classList={{
                            [`is-${card()?.signal ?? "none"}`]: true,
                            [`is-state-${state()}`]: true,
                            "is-fresh": isFreshToday(card(), reportToday(), state()),
                          }}
                          onClick={() => openSection(section)}
                        >
                          <span class="public-research-card__top">
                            <span class="public-research-card__kicker">
                              {section.kicker}
                              <Show when={section.adminOnly}>
                                <em class="public-research-card__gated">未发布</em>
                              </Show>
                            </span>
                            <span class="public-research-card__state">
                              <Show when={isFreshToday(card(), reportToday(), state())}>
                                <i class="public-research-card__dot" aria-hidden="true" />
                              </Show>
                              {stateLabel(state(), card(), reportToday())}
                            </span>
                          </span>

                          <span class="public-research-card__title">
                            <strong>{section.title}</strong>
                            <Show when={state() === "ready" && card()?.score != null}>
                              <b>{card()!.score!.toFixed(1)}</b>
                            </Show>
                            <Show when={state() === "ready" && card()?.signal}>
                              <em class="public-research-card__signal">
                                {SIGNAL_LABELS[card()!.signal!] ?? card()!.signal}
                              </em>
                            </Show>
                          </span>

                          <span class="public-research-card__summary">
                            {state() === "ready"
                              ? card()?.summary || section.blurb
                              : section.blurb}
                          </span>

                          <span class="public-research-card__foot">
                            <Show
                              when={state() === "ready"}
                              fallback={<span class="public-research-card__hint">{section.refreshAt}更新</span>}
                            >
                              <Show when={card()?.metric}>
                                <span class="public-research-card__metric">{card()!.metric}</span>
                              </Show>
                              <Show when={card()?.report_date}>
                                <time>{card()!.report_date}</time>
                              </Show>
                            </Show>
                            <i class="public-research-card__go" aria-hidden="true">›</i>
                          </span>
                        </button>
                      );
                    }}
                  </For>
                </div>
              </Show>

              <Show when={activeGroup() === "admin" && isAdmin()}>
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
