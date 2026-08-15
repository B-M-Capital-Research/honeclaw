import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import { Dynamic } from "solid-js/web";
import { useNavigate, useSearchParams } from "@solidjs/router";

import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import { PublicLoginForm } from "@/components/public-login-form";
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
import { stashResearchAsk } from "@/lib/research-ask";
import type {
  PublicAuthUserInfo,
  ResearchOverviewCard,
} from "@/lib/types";
import "./public-research.css";

/**
 * The research desk: one grid, every daily research product.
 *
 * These sections used to live as launcher chips crammed into a horizontal
 * rail above the chat composer — no URLs, no back button, nine eager fetches
 * the user might never look at. Here each section is a card fed by a single
 * compact overview call; the full snapshot loads only when its panel opens,
 * and the open panel lives in `?panel=` so it can be shared and dismissed
 * with the browser's back button.
 */

type PanelProps = { onClose: () => void; onAsk?: (message: string) => void };

type SectionDef = {
  key: string;
  title: string;
  kicker: string;
  /** Static one-liner shown until (or in place of) overview data. */
  blurb: string;
  panel?: (props: PanelProps) => ReturnType<typeof DailySignalPanel>;
  /** Sections that are full pages navigate instead of opening a panel. */
  href?: string;
};

const SECTIONS: SectionDef[] = [
  {
    key: "daily-signal-macro",
    title: "宏观红绿灯",
    kicker: "领先周期判断",
    blurb: "收入 → 消费 → 生产 → 利润 → 资本开支",
    panel: (props) => <DailySignalPanel kind="macro" {...props} />,
  },
  {
    key: "daily-signal-ai",
    title: "AI 红绿灯",
    kicker: "AI 增长可持续性",
    blurb: "需求旁证 · 商业化 · 融资 · 资本开支",
    panel: (props) => <DailySignalPanel kind="ai" {...props} />,
  },
  {
    key: "company-ratings",
    title: "公司评级",
    kicker: "52 家研究基线",
    blurb: "研究结构分与因子覆盖，缺数据时明示",
    panel: (props) => <CompanyRatingPanel {...props} />,
  },
  {
    key: "valuation-lab",
    title: "估值实验室",
    kicker: "三情景估值",
    blurb: "悲观 / 基准 / 乐观情景与关键假设",
    href: "/valuation-lab",
  },
  {
    key: "portfolio-news",
    title: "持仓重点新闻",
    kicker: "按你的持仓筛选",
    blurb: "每日新闻的持仓影响分析",
    panel: (props) => <PortfolioNewsPanel {...props} />,
  },
  {
    key: "position-management",
    title: "仓位管理",
    kicker: "评分 × 宏观 × 新闻",
    blurb: "结合评分与信号的每日仓位建议",
    panel: (props) => <PositionManagementPanel {...props} />,
  },
  {
    key: "influencer-digest",
    title: "大V速报",
    kicker: "观点不等于事实",
    blurb: "注册作者的公开观点日报",
    panel: (props) => <InfluencerDigestPanel {...props} />,
  },
  {
    key: "weekly-brief",
    title: "周度简报",
    kicker: "回顾与前瞻",
    blurb: "上周回顾 · 下周日历 · 30 日展望",
    panel: (props) => <WeeklyBriefPanel {...props} />,
  },
  {
    key: "key-event-chain",
    title: "关键事件链",
    kicker: "第一性证据链",
    blurb: "14 个 AI 主题的里程碑与线索",
    panel: (props) => <KeyEventChainPanel {...props} />,
  },
  {
    key: "research-library",
    title: "研究文库",
    kicker: "你的知识源",
    blurb: "上传资料，注入研究对话",
    href: "/research-library",
  },
];

const SIGNAL_LABELS: Record<string, string> = {
  green: "绿灯",
  yellow: "黄灯",
  orange: "橙灯",
  red: "红灯",
};

const STATUS_LABELS: Record<string, string> = {
  live: "数据完整",
  partial: "部分数据",
  stale: "沿用上次快照",
  waiting: "等待数据源",
  baseline: "研究基线",
  data_unavailable: "暂无数据",
  source_only: "仅收录原文",
  ready: "数据完整",
};

export default function PublicResearchPage() {
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const [user, setUser] = createSignal<PublicAuthUserInfo | null>(cachedPublicUser());
  const [authLoading, setAuthLoading] = createSignal(!hasCachedPublicUser());
  const [cards, setCards] = createSignal<Map<string, ResearchOverviewCard>>(new Map());
  const [overviewLoading, setOverviewLoading] = createSignal(true);
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

  const openSection = (section: SectionDef) => {
    if (section.href) {
      navigate(section.href);
      return;
    }
    // A plain push, so the browser back button closes the panel.
    setSearchParams({ panel: section.key });
  };

  const closePanel = () => setSearchParams({ panel: undefined });

  const askInChat = (message: string) => {
    stashResearchAsk(message);
    navigate("/chat?ask=research");
  };

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
                <h1>研究台</h1>
                <p>每日研究产品的家。评分与信号来自已保存的每日快照，缺数据时明示，不用模拟值补位。</p>
              </header>
              <Show when={!overviewLoading() || cards().size > 0} fallback={<ResearchState kind="loading" message="正在读取今日研究总览…" />}>
                <div class="public-research-grid">
                  <For each={SECTIONS}>
                    {(section) => {
                      const card = () => cards().get(section.key);
                      return (
                        <button
                          type="button"
                          class="public-research-card"
                          classList={{ [`is-${card()?.signal ?? "none"}`]: true }}
                          onClick={() => openSection(section)}
                        >
                          <span class="public-research-card__kicker">{section.kicker}</span>
                          <span class="public-research-card__title">
                            <Show when={card()?.signal}>
                              <i class="public-research-card__light" aria-hidden="true" />
                            </Show>
                            <strong>{section.title}</strong>
                            <Show when={card()?.score != null}>
                              <b>{card()!.score!.toFixed(1)}</b>
                            </Show>
                          </span>
                          <span class="public-research-card__summary">
                            {card()?.summary || section.blurb}
                          </span>
                          <span class="public-research-card__meta">
                            <Show when={card()} fallback={<em>{section.href ? "打开页面 ›" : "打开面板 ›"}</em>}>
                              {(value) => (
                                <>
                                  <Show when={value().signal}>
                                    <span class="public-research-card__chip is-signal">
                                      {SIGNAL_LABELS[value().signal!] ?? value().signal}
                                    </span>
                                  </Show>
                                  <span class="public-research-card__chip">
                                    {STATUS_LABELS[value().status] ?? value().status}
                                  </span>
                                  <Show when={value().metric}>
                                    <span class="public-research-card__chip">{value().metric}</span>
                                  </Show>
                                  <Show when={value().report_date}>
                                    <time>{value().report_date}</time>
                                  </Show>
                                </>
                              )}
                            </Show>
                          </span>
                        </button>
                      );
                    }}
                  </For>
                </div>
              </Show>
              <Show when={activeSection()}>
                {(section) => (
                  <Dynamic
                    component={section().panel!}
                    onClose={closePanel}
                    onAsk={askInChat}
                  />
                )}
              </Show>
            </main>
          </PublicWorkspaceShell>
        )}
      </Show>
    </Show>
  );
}
