import {
  createEffect,
  createMemo,
  createSignal,
  For,
  Match,
  onCleanup,
  Show,
  Switch,
  type JSX,
} from "solid-js";
import { HoneBrand } from "@/components/hone-brand";
import { CONTENT } from "@/lib/public-content"
import { routePrefetchHandlers } from "@/lib/route-prefetch";
import { groupResearchByDate } from "@/lib/public-agent-workspace";
import type {
  AgentWorkspaceEvent,
  AgentWorkspaceInsight,
} from "@/lib/public-agent-workspace";

type ResearchItem = { id: string; title: string; at?: string };

type IconName =
  | "agent"
  | "arrow"
  | "bell"
  | "briefcase"
  | "calendar"
  | "compare"
  | "history"
  | "home"
  | "insight"
  | "invest"
  | "me"
  | "menu"
  | "new"
  | "paper"
  | "research"
  | "search"
  | "send"
  | "track";

export function AgentWorkspaceIcon(props: {
  name: IconName;
  size?: number;
}) {
  const size = () => props.size ?? 20;
  return (
    <svg
      width={size()}
      height={size()}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <Switch>
        <Match when={props.name === "agent"}>
          <path d="M12 2.8 13.7 8l5.3 1.7-5.3 1.7L12 16.6l-1.7-5.2L5 9.7 10.3 8 12 2.8Z" />
          <path d="m18.2 14 .8 2.4 2.4.8-2.4.8-.8 2.4-.8-2.4-2.4-.8 2.4-.8.8-2.4Z" />
        </Match>
        <Match when={props.name === "arrow"}><path d="M5 12h14M14 7l5 5-5 5" /></Match>
        <Match when={props.name === "bell"}><path d="M18 8a6 6 0 0 0-12 0c0 7-3 7-3 9h18c0-2-3-2-3-9M10 21h4" /></Match>
        <Match when={props.name === "briefcase"}><rect x="3" y="7" width="18" height="13" rx="2" /><path d="M8 7V5a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M3 12h18M10 12v2h4v-2" /></Match>
        <Match when={props.name === "calendar"}><rect x="3" y="5" width="18" height="16" rx="2" /><path d="M16 3v4M8 3v4M3 10h18M8 14h.01M12 14h.01M16 14h.01" /></Match>
        <Match when={props.name === "compare"}><path d="M4 20V10h5v10M10 20V4h5v16M16 20v-7h4v7M2 20h20" /></Match>
        <Match when={props.name === "history"}><path d="M3 12a9 9 0 1 0 3-6.7L3 8M3 3v5h5M12 7v5l3 2" /></Match>
        <Match when={props.name === "home"}><path d="m4 10 8-6 8 6v9a1 1 0 0 1-1 1h-5v-6h-4v6H5a1 1 0 0 1-1-1Z" /></Match>
        <Match when={props.name === "insight"}><path d="M9 18h6M10 22h4M8.6 14.8A7 7 0 1 1 15.4 14.8C14.5 15.5 14 16.5 14 18h-4c0-1.5-.5-2.5-1.4-3.2Z" /></Match>
        <Match when={props.name === "invest"}><path d="M4 19V9M10 19V5M16 19v-7M22 19V3M2 19h22" /></Match>
        <Match when={props.name === "me"}><circle cx="12" cy="8" r="4" /><path d="M4 21a8 8 0 0 1 16 0" /></Match>
        <Match when={props.name === "menu"}><path d="M4 7h16M4 12h16M4 17h16" /></Match>
        <Match when={props.name === "new"}><path d="M12 5v14M5 12h14" /></Match>
        <Match when={props.name === "paper"}><path d="M6 2h9l4 4v16H6zM14 2v5h5M9 12h7M9 16h7" /></Match>
        <Match when={props.name === "research"}><rect x="3" y="3" width="7.5" height="9.5" rx="1.6" /><rect x="13.5" y="3" width="7.5" height="5.5" rx="1.6" /><rect x="13.5" y="11.5" width="7.5" height="9.5" rx="1.6" /><rect x="3" y="15.5" width="7.5" height="5.5" rx="1.6" /></Match>
        <Match when={props.name === "search"}><circle cx="11" cy="11" r="7" /><path d="m20 20-4-4" /></Match>
        <Match when={props.name === "send"}><path d="m3 11 18-8-8 18-2-8-8-2Z" /><path d="m11 13 5-5" /></Match>
        <Match when={props.name === "track"}><circle cx="12" cy="12" r="8" /><circle cx="12" cy="12" r="3" /><path d="M12 2v3M22 12h-3" /></Match>
      </Switch>
    </svg>
  );
}

export function AgentWorkspaceSidebar(props: {
  userName: string;
  research: ResearchItem[];
  activeMode: "overview" | "conversation";
  activeSection?: "agent" | "research" | "pushes" | "insights" | "me";
  /** Optional so a surface that has not wired the section simply omits it
      rather than failing to compile. */
  onPushes?: () => void;
  unreadPushCount?: number;
  communityUnread: boolean;
  hasOlder?: boolean;
  loadingOlder?: boolean;
  onLoadOlder?: () => void;
  researchLoading?: boolean;
  onNewResearch: () => void;
  onSelectResearch: (id: string) => void;
  onHome: () => void;
  onResearchDesk?: () => void;
  onInsights: () => void;
  onAccount: () => void;
  onLogout: () => void;
}) {
  const avatar = () =>
    props.userName === CONTENT.chat_page.workspace.default_user || props.userName.startsWith(CONTENT.chat_page.workspace.user_prefix)
      ? "H"
      : props.userName.slice(-1);
  const [query, setQuery] = createSignal("");
  const filteredResearch = createMemo(() => {
    const normalized = query().trim().toLowerCase();
    if (!normalized) return props.research;
    return props.research.filter((item) => item.title.toLowerCase().includes(normalized));
  });
  createEffect(() => {
    if (props.activeMode === "overview") setQuery("");
  });
  return (
    <aside class="agent-workspace-sidebar" aria-label={CONTENT.chat_page.workspace.brand_aria}>
      <button type="button" class="agent-workspace-brand" onClick={props.onNewResearch} aria-label={CONTENT.chat_page.workspace.brand_aria}>
        <HoneBrand />
      </button>
      <nav class="agent-workspace-nav">
        <button type="button" classList={{ "is-active": props.activeSection === "agent" }} onClick={props.onHome}><AgentWorkspaceIcon name="agent" /><span>{CONTENT.chat_page.workspace.assistant_nav}</span></button>
        <Show when={props.onResearchDesk}>{(onResearchDesk) => <button type="button" {...routePrefetchHandlers("research")} classList={{ "is-active": props.activeSection === "research" }} onClick={onResearchDesk()}><AgentWorkspaceIcon name="research" /><span>{CONTENT.chat_page.workspace.research}</span></button>}</Show>
        <Show when={props.onPushes}>{(onPushes) => <button type="button" {...routePrefetchHandlers("pushes")} classList={{ "is-active": props.activeSection === "pushes" }} onClick={onPushes()} class="agent-workspace-nav-with-dot"><AgentWorkspaceIcon name="bell" /><span>{CONTENT.chat_page.workspace.pushes_tab}</span><Show when={(props.unreadPushCount ?? 0) > 0}><i /></Show></button>}</Show>
        <button type="button" {...routePrefetchHandlers("community")} onClick={props.onInsights} class="agent-workspace-nav-with-dot" classList={{ "is-active": props.activeSection === "insights" }}><AgentWorkspaceIcon name="insight" /><span>{CONTENT.chat_page.workspace.insights}</span><Show when={props.communityUnread}><i /></Show></button>
        <button type="button" {...routePrefetchHandlers("me")} classList={{ "is-active": props.activeSection === "me" }} onClick={props.onAccount}><AgentWorkspaceIcon name="me" /><span>{CONTENT.chat_page.workspace.me}</span></button>
      </nav>
      <div class="agent-workspace-sidebar-rule" />
      <div class="agent-workspace-nav-label">{CONTENT.chat_page.workspace.history_label}</div>
      <button type="button" class={`agent-workspace-new ${props.activeSection === "agent" && props.activeMode === "overview" ? "is-active" : ""}`} onClick={props.onNewResearch}>
        <AgentWorkspaceIcon name="new" /><span>{CONTENT.chat_page.workspace.new_chat}</span>
      </button>
      <label class="agent-workspace-history-search">
        <AgentWorkspaceIcon name="search" size={16} />
        <input value={query()} onInput={(event) => setQuery(event.currentTarget.value)} placeholder={CONTENT.chat_page.workspace.search_history} />
      </label>
      <section class="agent-workspace-history">
        <Show
          when={filteredResearch().length > 0}
          fallback={
            <>
              <div class="agent-workspace-history-label">{CONTENT.chat_page.workspace.recent}</div>
              <p role="status">
                {props.researchLoading
                  ? CONTENT.chat_page.workspace.syncing_history
                  : query().trim()
                    ? CONTENT.chat_page.workspace.no_match
                    : CONTENT.chat_page.workspace.history_empty}
              </p>
            </>
          }
        >
          <For each={groupResearchByDate(filteredResearch())}>{(group) => (
            <>
              <div class="agent-workspace-history-label">{group.label}</div>
              <For each={group.items}>{(item) => (
                <button type="button" onClick={() => props.onSelectResearch(item.id)}>{item.title}</button>
              )}</For>
            </>
          )}</For>
        </Show>
        <Show when={props.hasOlder && props.onLoadOlder && !query().trim()}>
          <button type="button" class="agent-workspace-history-older" disabled={props.loadingOlder} onClick={() => props.onLoadOlder?.()}>
            {props.loadingOlder ? CONTENT.chat_page.workspace.loading : CONTENT.chat_page.workspace.load_older}
          </button>
        </Show>
      </section>
      <div class="agent-workspace-user">
        <button type="button" class="agent-workspace-user-main" {...routePrefetchHandlers("me")} classList={{ "is-active": props.activeSection === "me" }} onClick={props.onAccount}>
          <span class="agent-workspace-avatar">{avatar()}</span>
          <span><strong>{props.userName}</strong><small>{CONTENT.chat_page.workspace.personal_space}</small></span>
        </button>
        <button type="button" class="agent-workspace-logout" onClick={props.onLogout}>{CONTENT.chat_page.workspace.logout}</button>
      </div>
    </aside>
  );
}

export function AgentWorkspaceTopbar(props: {
  query: string;
  unreadPushCount: number;
  label?: string;
  placeholder?: string;
  showSearch?: boolean;
  preferences?: JSX.Element;
  onQueryChange: (value: string) => void;
  onPushes: () => void;
}) {
  return (
    <header class="agent-workspace-topbar">
      <span>{props.label ?? CONTENT.chat_page.workspace.agent_tagline}</span>
      <div class="agent-workspace-topbar-actions">
        <Show when={props.showSearch !== false}><label><AgentWorkspaceIcon name="search" size={17} /><input value={props.query} onInput={(event) => props.onQueryChange(event.currentTarget.value)} placeholder={props.placeholder ?? CONTENT.chat_page.workspace.search_all} /></label></Show>
        {props.preferences}
        <button type="button" onClick={props.onPushes} aria-label={CONTENT.chat_page.workspace.open_pushes}>
          <AgentWorkspaceIcon name="bell" />
          <Show when={props.unreadPushCount > 0}><i /></Show>
        </button>
      </div>
    </header>
  );
}

export function AgentWorkspaceLoadingState(props: {
  retrying?: boolean;
  attempt?: number;
}) {
  return (
    <div class="agent-workspace-loading" role="status" aria-live="polite">
      <div class="agent-workspace-loading-copy">
        <span class="agent-workspace-loading-mark" aria-hidden="true">
          <AgentWorkspaceIcon name="agent" size={24} />
        </span>
        <div>
          <strong>{props.retrying ? CONTENT.chat_page.workspace.reconnecting : CONTENT.chat_page.workspace.restoring}</strong>
          <p>
            {props.retrying
              ? CONTENT.chat_page.workspace.sync_attempt.replace("{attempt}", String(props.attempt ?? 2))
              : CONTENT.chat_page.workspace.sync_detail}
          </p>
        </div>
      </div>
      <div class="agent-workspace-loading-skeleton" aria-hidden="true">
        <i />
        <i />
        <i />
      </div>
    </div>
  );
}

type QuickStart = {
  icon: IconName;
  title: string;
  summary: string;
  meta: string;
  prompt: string;
  action?: "tracking";
};

/// Built per call so the locale proxy is read at render time; a module-level
/// const would freeze whichever language was active at import.
const quickStarts = (): QuickStart[] => [
  { icon: "invest", title: CONTENT.chat_page.workspace.qa_moves_title, summary: CONTENT.chat_page.workspace.qa_moves_summary, meta: CONTENT.chat_page.workspace.qa_moves_meta, prompt: CONTENT.chat_page.workspace.qa_moves_prompt },
  { icon: "compare", title: CONTENT.chat_page.workspace.qa_compare_title, summary: CONTENT.chat_page.workspace.qa_compare_summary, meta: CONTENT.chat_page.workspace.qa_compare_meta, prompt: CONTENT.chat_page.workspace.qa_compare_prompt },
  { icon: "paper", title: CONTENT.chat_page.workspace.qa_filing_title, summary: CONTENT.chat_page.workspace.qa_filing_summary, meta: CONTENT.chat_page.workspace.qa_filing_meta, prompt: CONTENT.chat_page.workspace.qa_filing_prompt },
  { icon: "track", title: CONTENT.chat_page.workspace.qa_track_title, summary: CONTENT.chat_page.workspace.qa_track_summary, meta: CONTENT.chat_page.workspace.qa_track_meta, prompt: CONTENT.chat_page.workspace.qa_track_prompt, action: "tracking" },
];

export function AgentWorkspaceOverview(props: {
  greeting: string;
  insights: AgentWorkspaceInsight[];
  events: AgentWorkspaceEvent[];
  insightCount: number;
  searchQuery: string;
  onPrompt: (prompt: string) => void;
  onTracking: () => void;
  onInsights: () => void;
  onCalendar: () => void;
}) {
  const fallbackInsights = (): AgentWorkspaceInsight[] => [
    { id: "portfolio", eyebrow: CONTENT.chat_page.workspace.seed_portfolio_eyebrow, title: CONTENT.chat_page.workspace.seed_portfolio_title, summary: CONTENT.chat_page.workspace.seed_portfolio_summary },
    { id: "event", eyebrow: CONTENT.chat_page.workspace.seed_event_eyebrow, title: CONTENT.chat_page.workspace.seed_event_title, summary: CONTENT.chat_page.workspace.seed_event_summary },
    { id: "research", eyebrow: CONTENT.chat_page.workspace.seed_research_eyebrow, title: CONTENT.chat_page.workspace.seed_research_title, summary: CONTENT.chat_page.workspace.seed_research_summary },
  ];
  const visibleInsights = createMemo(() => {
    const source = props.insights.length ? props.insights : fallbackInsights();
    const query = props.searchQuery.trim().toLowerCase();
    if (!query) return source;
    return source.filter((item) => `${item.title} ${item.summary}`.toLowerCase().includes(query));
  });
  const promptForInsight = (item: AgentWorkspaceInsight) =>
    CONTENT.chat_page.workspace.insight_prompt
      .replace("{title}", item.title)
      .replace("{summary}", item.summary);
  return (
    <main class="agent-workspace-overview">
      <div class="agent-workspace-title-row">
        <div><h1>投资助手</h1><div class="agent-workspace-context">{CONTENT.chat_page.workspace.context_prefix}<span>{CONTENT.chat_page.workspace.context_portfolio}</span><span>{CONTENT.chat_page.workspace.context_events}</span></div></div>
      </div>
      <section class="agent-workspace-greeting">
        <span class="agent-workspace-agent-mark"><AgentWorkspaceIcon name="agent" size={25} /></span>
        <div><h2>{props.greeting}</h2><p>{CONTENT.chat_page.workspace.insight_count.replace("{count}", String(props.insightCount))}</p></div>
      </section>
      <section class="agent-workspace-section">
        <div class="agent-workspace-section-heading"><h2>{CONTENT.chat_page.workspace.quick_start}</h2><span>{CONTENT.chat_page.workspace.quick_start_hint}</span></div>
        <div class="agent-workspace-quick-grid">
          <For each={quickStarts()}>{(item) => (
            <button type="button" onClick={() => item.action === "tracking" ? props.onTracking() : props.onPrompt(item.prompt)}>
              <AgentWorkspaceIcon name={item.icon} />
              <strong>{item.title}</strong><span>{item.summary}</span><small>{item.meta}</small>
            </button>
          )}</For>
        </div>
      </section>
      <section class="agent-workspace-section agent-workspace-insights">
        <div class="agent-workspace-section-heading"><h2>{CONTENT.chat_page.workspace.today_insights}</h2><button type="button" onClick={props.onInsights}>{CONTENT.chat_page.workspace.browse_community} <AgentWorkspaceIcon name="arrow" size={16} /></button></div>
        <div class="agent-workspace-insight-list">
          <Show when={visibleInsights().length > 0} fallback={<div class="agent-workspace-empty">{CONTENT.chat_page.workspace.no_insight_match}</div>}>
            <For each={visibleInsights()}>{(item) => (
              <button type="button" onClick={() => props.onPrompt(promptForInsight(item))}>
                <i /><span><small>{item.eyebrow}</small><strong>{item.title}</strong><em>{item.summary}</em></span><AgentWorkspaceIcon name="arrow" />
              </button>
            )}</For>
          </Show>
        </div>
      </section>
      <section class="agent-workspace-section agent-workspace-mobile-events">
        <div class="agent-workspace-section-heading">
          <h2>{CONTENT.chat_page.workspace.key_events}</h2>
          <button type="button" onClick={props.onCalendar}>
            {CONTENT.chat_page.workspace.finance_calendar} <AgentWorkspaceIcon name="arrow" size={16} />
          </button>
        </div>
        <button type="button" onClick={props.onCalendar}>
          <span class="agent-workspace-mobile-event-icon">
            <AgentWorkspaceIcon name="calendar" />
          </span>
          <span>
            <strong>{props.events[0]?.title ?? CONTENT.chat_page.workspace.open_my_calendar}</strong>
            <small>
              {props.events[0]
                ? `${props.events[0].date} ${props.events[0].time}`.trim()
                : CONTENT.chat_page.workspace.calendar_summary}
            </small>
          </span>
          <AgentWorkspaceIcon name="arrow" />
        </button>
      </section>
    </main>
  );
}

export function AgentWorkspaceRightRail(props: {
  events: AgentWorkspaceEvent[];
  research: ResearchItem[];
  onCalendar: () => void;
  onSelectResearch: (id: string) => void;
}) {
  return (
    <aside class="agent-workspace-rail">
      <section><div class="agent-workspace-rail-heading"><h2>{CONTENT.chat_page.workspace.upcoming_events}</h2><button type="button" onClick={props.onCalendar}>{CONTENT.chat_page.workspace.finance_calendar}</button></div>
        <div class="agent-workspace-event-list">
          <Show when={props.events.length > 0} fallback={<button type="button" onClick={props.onCalendar} class="agent-workspace-rail-empty"><AgentWorkspaceIcon name="calendar" /><span>{CONTENT.chat_page.workspace.open_your_calendar}</span></button>}>
            <For each={props.events}>{(event) => <button type="button" onClick={props.onCalendar}><span><strong>{event.title}</strong><small>{event.date}{event.time ? ` ${event.time}` : ""}</small><em>{event.summary}</em></span><AgentWorkspaceIcon name="arrow" size={15} /></button>}</For>
          </Show>
        </div>
      </section>
      <section><div class="agent-workspace-rail-heading"><h2>{CONTENT.chat_page.workspace.recent_research}</h2></div>
        <div class="agent-workspace-saved-list">
          <Show when={props.research.length > 0} fallback={<p>{CONTENT.chat_page.workspace.research_empty}</p>}>
            <For each={props.research.slice(0, 3)}>{(item) => <button type="button" onClick={() => props.onSelectResearch(item.id)}><strong>{item.title}</strong><small>{CONTENT.chat_page.workspace.continue_research}</small></button>}</For>
          </Show>
        </div>
      </section>
    </aside>
  );
}

export function AgentWorkspaceMobileHeader(props: {
  userName: string;
  unreadPushCount: number;
  historyCount?: number;
  preferences?: JSX.Element;
  onPushes: () => void;
  onHistory?: () => void;
  onMenu?: () => void;
  onAccount: () => void;
}) {
  const avatar = () =>
    props.userName === CONTENT.chat_page.workspace.default_user || props.userName.startsWith(CONTENT.chat_page.workspace.user_prefix)
      ? "H"
      : props.userName.slice(-1);
  return <header class="agent-workspace-mobile-header"><div class="agent-workspace-mobile-header-left"><Show when={props.onMenu}>{(onMenu) => <button type="button" onClick={onMenu()} aria-label={CONTENT.chat_page.workspace.open_menu} class="agent-workspace-mobile-menu-trigger"><AgentWorkspaceIcon name="menu" /></button>}</Show><HoneBrand /></div><div><Show when={props.onMenu ? undefined : props.onHistory}>{(onHistory) => <button type="button" onClick={onHistory()} aria-label={CONTENT.chat_page.workspace.history_title} class="agent-workspace-mobile-history-trigger"><AgentWorkspaceIcon name="history" /><Show when={(props.historyCount ?? 0) > 0}><span>{Math.min(props.historyCount ?? 0, 99)}</span></Show></button>}</Show>{props.preferences}<button type="button" onClick={props.onPushes} aria-label={CONTENT.chat_page.workspace.pushes}><AgentWorkspaceIcon name="bell" /><Show when={props.unreadPushCount > 0}><i /></Show></button><button type="button" {...routePrefetchHandlers("me")} onClick={props.onAccount} class="agent-workspace-mobile-avatar" aria-label={CONTENT.chat_page.workspace.open_account.replace("{name}", props.userName)}>{avatar()}</button></div></header>;
}

/**
 * 移动端左侧抽屉（ChatGPT 式）：左上角菜单按钮或从屏幕左缘右滑拉出，
 * 上半部分是主要菜单，下半部分是聊天记录；点背景、左滑或 Esc 关闭。
 */
export function AgentWorkspaceHistoryDrawer(props: {
  open: boolean;
  userName: string;
  research: ResearchItem[];
  hasOlder: boolean;
  loadingOlder: boolean;
  communityUnread: boolean;
  researchLoading?: boolean;
  onOpen: () => void;
  onClose: () => void;
  onSelectResearch: (id: string) => void;
  onLoadOlder: () => void;
  onNewResearch: () => void;
  onHome: () => void;
  onResearchDesk?: () => void;
  onInsights: () => void;
  onAccount: () => void;
}) {
  const [query, setQuery] = createSignal("");
  const avatar = () =>
    props.userName === CONTENT.chat_page.workspace.default_user || props.userName.startsWith(CONTENT.chat_page.workspace.user_prefix)
      ? "H"
      : props.userName.slice(-1);
  const filteredResearch = createMemo(() => {
    const normalized = query().trim().toLowerCase();
    if (!normalized) return props.research;
    return props.research.filter((item) => item.title.toLowerCase().includes(normalized));
  });

  createEffect(() => {
    if (!props.open) return;
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") props.onClose();
    };
    document.addEventListener("keydown", closeOnEscape);
    onCleanup(() => document.removeEventListener("keydown", closeOnEscape));
  });

  // 左缘右滑打开 / 抽屉上左滑关闭（仅移动端布局生效）。
  createEffect(() => {
    const isMobileLayout = () => window.matchMedia("(max-width: 820px)").matches;
    let startX = 0;
    let startY = 0;
    let tracking: "open" | "close" | null = null;
    const onTouchStart = (event: TouchEvent) => {
      const touch = event.touches[0];
      if (!touch || event.touches.length > 1 || !isMobileLayout()) {
        tracking = null;
        return;
      }
      if (!props.open && touch.clientX <= 26) {
        tracking = "open";
      } else if (props.open) {
        tracking = "close";
      } else {
        tracking = null;
        return;
      }
      startX = touch.clientX;
      startY = touch.clientY;
    };
    const onTouchMove = (event: TouchEvent) => {
      if (!tracking) return;
      const touch = event.touches[0];
      if (!touch) return;
      const deltaX = touch.clientX - startX;
      const deltaY = touch.clientY - startY;
      if (Math.abs(deltaY) > 70) {
        tracking = null;
        return;
      }
      if (tracking === "open" && deltaX > 48) {
        tracking = null;
        props.onOpen();
      } else if (tracking === "close" && deltaX < -48) {
        tracking = null;
        props.onClose();
      }
    };
    const onTouchEnd = () => {
      tracking = null;
    };
    document.addEventListener("touchstart", onTouchStart, { passive: true });
    document.addEventListener("touchmove", onTouchMove, { passive: true });
    document.addEventListener("touchend", onTouchEnd, { passive: true });
    onCleanup(() => {
      document.removeEventListener("touchstart", onTouchStart);
      document.removeEventListener("touchmove", onTouchMove);
      document.removeEventListener("touchend", onTouchEnd);
    });
  });

  return (
    <Show when={props.open}>
      <div class="agent-workspace-history-backdrop" onClick={props.onClose} />
      <aside class="agent-workspace-history-drawer" aria-label={CONTENT.chat_page.workspace.drawer_aria} aria-modal="true" role="dialog">
        <header>
          <HoneBrand />
          <button type="button" onClick={props.onClose} aria-label={CONTENT.chat_page.workspace.close_menu}>×</button>
        </header>
        <nav class="agent-workspace-drawer-nav" aria-label={CONTENT.chat_page.workspace.main_menu}>
          <button type="button" class="agent-workspace-drawer-new" onClick={props.onNewResearch}><AgentWorkspaceIcon name="new" /><span>{CONTENT.chat_page.workspace.new_chat}</span></button>
          <Show when={props.onResearchDesk}>{(onResearchDesk) => <button type="button" {...routePrefetchHandlers("research")} onClick={onResearchDesk()}><AgentWorkspaceIcon name="research" /><span>{CONTENT.chat_page.workspace.research}</span></button>}</Show>
          <button type="button" {...routePrefetchHandlers("community")} onClick={props.onInsights} class="agent-workspace-drawer-with-dot"><AgentWorkspaceIcon name="insight" /><span>{CONTENT.chat_page.workspace.insights}</span><Show when={props.communityUnread}><i /></Show></button>
          <button type="button" {...routePrefetchHandlers("me")} onClick={props.onAccount}><AgentWorkspaceIcon name="me" /><span>{CONTENT.chat_page.workspace.me}</span></button>
        </nav>
        <label class="agent-workspace-history-search agent-workspace-drawer-search">
          <AgentWorkspaceIcon name="search" size={16} />
          <input value={query()} onInput={(event) => setQuery(event.currentTarget.value)} placeholder={CONTENT.chat_page.workspace.search_chats} />
        </label>
        <div class="agent-workspace-history-drawer-label">{CONTENT.chat_page.workspace.chat_records}</div>
        <div class="agent-workspace-history-drawer-list">
          <Show
            when={filteredResearch().length > 0}
            fallback={
              <p role="status">
                {props.researchLoading
                  ? CONTENT.chat_page.workspace.syncing_history
                  : query().trim()
                    ? CONTENT.chat_page.workspace.no_match
                    : CONTENT.chat_page.workspace.drawer_history_empty}
              </p>
            }
          >
            <For each={groupResearchByDate(filteredResearch())}>{(group) => (
              <>
                <div class="agent-workspace-history-drawer-group">{group.label}</div>
                <For each={group.items}>{(item) => (
                  <button type="button" onClick={() => props.onSelectResearch(item.id)}>
                    <strong>{item.title}</strong>
                    <AgentWorkspaceIcon name="arrow" size={16} />
                  </button>
                )}</For>
              </>
            )}</For>
          </Show>
          <Show when={props.hasOlder && !query().trim()}>
            <button type="button" class="agent-workspace-history-more" disabled={props.loadingOlder} onClick={props.onLoadOlder}>
              {props.loadingOlder ? CONTENT.chat_page.workspace.loading : CONTENT.chat_page.workspace.load_older}
            </button>
          </Show>
        </div>
        <button type="button" class="agent-workspace-drawer-user" onClick={props.onAccount}>
          <span class="agent-workspace-avatar">{avatar()}</span>
          <span><strong>{props.userName}</strong><small>{CONTENT.chat_page.workspace.personal_space}</small></span>
        </button>
      </aside>
    </Show>
  );
}

export function AgentWorkspaceMobileNav(props: {
  activeMode: "overview" | "conversation";
  activeSection?: "agent" | "research" | "pushes" | "insights" | "me";
  communityUnread: boolean;
  unreadPushCount: number;
  onHome: () => void;
  onInsights: () => void;
  onAgent: () => void;
  onResearchDesk?: () => void;
  onPushesTab?: () => void;
  onAccount: () => void;
}) {
  return <nav class="agent-workspace-mobile-nav" aria-label={CONTENT.chat_page.workspace.main_nav}>
    <button type="button" classList={{ "is-active": props.activeSection === "agent" }} onClick={props.onAgent}><AgentWorkspaceIcon name="agent" /><span>投资助手</span></button>
    <Show when={props.onResearchDesk}>{(onResearchDesk) => <button type="button" {...routePrefetchHandlers("research")} classList={{ "is-active": props.activeSection === "research" }} onClick={onResearchDesk()}><AgentWorkspaceIcon name="research" /><span>{CONTENT.chat_page.workspace.research}</span></button>}</Show>
    <Show when={props.onPushesTab}>{(onPushesTab) => <button type="button" {...routePrefetchHandlers("pushes")} class="agent-workspace-mobile-has-dot" classList={{ "is-active": props.activeSection === "pushes" }} onClick={onPushesTab()}><AgentWorkspaceIcon name="bell" /><span>{CONTENT.chat_page.workspace.pushes_tab}</span><Show when={props.unreadPushCount > 0}><i /></Show></button>}</Show>
    <button type="button" onClick={props.onInsights} class="agent-workspace-mobile-has-dot" classList={{ "is-active": props.activeSection === "insights" }}><AgentWorkspaceIcon name="insight" /><span>{CONTENT.chat_page.workspace.insights}</span><Show when={props.communityUnread}><i /></Show></button>
    <button type="button" {...routePrefetchHandlers("me")} classList={{ "is-active": props.activeSection === "me" }} onClick={props.onAccount}><AgentWorkspaceIcon name="me" /><span>{CONTENT.chat_page.workspace.me}</span></button>
  </nav>;
}
