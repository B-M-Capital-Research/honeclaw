import { createEffect, createMemo, createSignal, For, Match, onCleanup, Show, Switch } from "solid-js";
import { HoneBrand } from "@/components/hone-brand";
import type {
  AgentWorkspaceEvent,
  AgentWorkspaceInsight,
} from "@/lib/public-agent-workspace";

type ResearchItem = { id: string; title: string };

type IconName =
  | "agent"
  | "arrow"
  | "bell"
  | "briefcase"
  | "calendar"
  | "compare"
  | "history"
  | "insight"
  | "invest"
  | "me"
  | "new"
  | "paper"
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
        <Match when={props.name === "insight"}><path d="M9 18h6M10 22h4M8.6 14.8A7 7 0 1 1 15.4 14.8C14.5 15.5 14 16.5 14 18h-4c0-1.5-.5-2.5-1.4-3.2Z" /></Match>
        <Match when={props.name === "invest"}><path d="M4 19V9M10 19V5M16 19v-7M22 19V3M2 19h22" /></Match>
        <Match when={props.name === "me"}><circle cx="12" cy="8" r="4" /><path d="M4 21a8 8 0 0 1 16 0" /></Match>
        <Match when={props.name === "new"}><path d="M12 5v14M5 12h14" /></Match>
        <Match when={props.name === "paper"}><path d="M6 2h9l4 4v16H6zM14 2v5h5M9 12h7M9 16h7" /></Match>
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
  activeSection?: "agent" | "invest" | "insights" | "tracking" | "me";
  communityUnread: boolean;
  onNewResearch: () => void;
  onSelectResearch: (id: string) => void;
  onInvest: () => void;
  onInsights: () => void;
  onTracking: () => void;
  onAccount: () => void;
  onLogout: () => void;
}) {
  const avatar = () =>
    props.userName === "HONE 用户" || props.userName.startsWith("用户 ")
      ? "H"
      : props.userName.slice(-1);
  const [query, setQuery] = createSignal("");
  const filteredResearch = createMemo(() => {
    const normalized = query().trim().toLowerCase();
    if (!normalized) return props.research;
    return props.research.filter((item) => item.title.toLowerCase().includes(normalized));
  });
  return (
    <aside class="agent-workspace-sidebar" aria-label="HONE 工作台">
      <button type="button" class="agent-workspace-brand" onClick={props.onNewResearch} aria-label="HONE Agent">
        <HoneBrand />
      </button>
      <div class="agent-workspace-nav-label">工作台</div>
      <nav class="agent-workspace-nav">
        <button type="button" classList={{ "is-active": props.activeSection === "invest" }} onClick={props.onInvest}><AgentWorkspaceIcon name="invest" /><span>投资</span></button>
        <button type="button" onClick={props.onInsights} class="agent-workspace-nav-with-dot" classList={{ "is-active": props.activeSection === "insights" }}><AgentWorkspaceIcon name="insight" /><span>洞察</span><Show when={props.communityUnread}><i /></Show></button>
        <button type="button" classList={{ "is-active": props.activeSection === "tracking" }} onClick={props.onTracking}><AgentWorkspaceIcon name="track" /><span>跟踪</span></button>
      </nav>
      <div class="agent-workspace-sidebar-rule" />
      <div class="agent-workspace-nav-label">AI 研究</div>
      <button type="button" class={`agent-workspace-new ${props.activeSection === "agent" && props.activeMode === "overview" ? "is-active" : ""}`} onClick={props.onNewResearch}>
        <AgentWorkspaceIcon name="new" /><span>新研究</span>
      </button>
      <label class="agent-workspace-history-search">
        <AgentWorkspaceIcon name="search" size={16} />
        <input value={query()} onInput={(event) => setQuery(event.currentTarget.value)} placeholder="搜索研究记录" />
      </label>
      <section class="agent-workspace-history">
        <div class="agent-workspace-history-label">最近</div>
        <Show when={filteredResearch().length > 0} fallback={<p>你的研究记录会出现在这里。</p>}>
          <For each={filteredResearch()}>{(item) => (
            <button type="button" onClick={() => props.onSelectResearch(item.id)}>{item.title}</button>
          )}</For>
        </Show>
      </section>
      <div class="agent-workspace-user">
        <button type="button" class="agent-workspace-user-main" classList={{ "is-active": props.activeSection === "me" }} onClick={props.onAccount}>
          <span class="agent-workspace-avatar">{avatar()}</span>
          <span><strong>{props.userName}</strong><small>个人研究空间</small></span>
        </button>
        <button type="button" class="agent-workspace-logout" onClick={props.onLogout}>退出</button>
      </div>
    </aside>
  );
}

export function AgentWorkspaceTopbar(props: {
  query: string;
  unreadPushCount: number;
  label?: string;
  placeholder?: string;
  onQueryChange: (value: string) => void;
  onPushes: () => void;
}) {
  return (
    <header class="agent-workspace-topbar">
      <span>{props.label ?? "你的投资研究智能体"}</span>
      <div class="agent-workspace-topbar-actions">
        <label><AgentWorkspaceIcon name="search" size={17} /><input value={props.query} onInput={(event) => props.onQueryChange(event.currentTarget.value)} placeholder={props.placeholder ?? "搜索公司、主题或洞察"} /></label>
        <button type="button" onClick={props.onPushes} aria-label="打开通知">
          <AgentWorkspaceIcon name="bell" />
          <Show when={props.unreadPushCount > 0}><i /></Show>
        </button>
      </div>
    </header>
  );
}

const QUICK_STARTS: Array<{
  icon: IconName;
  title: string;
  summary: string;
  meta: string;
  prompt: string;
  action?: "tracking";
}> = [
  { icon: "invest", title: "解释组合波动", summary: "解释今天组合上涨或下跌的主要原因", meta: "组合 · 今日", prompt: "请结合我的持仓，解释今天组合波动的主要原因，并按影响大小排序。" },
  { icon: "compare", title: "比较两家公司", summary: "比较两家公司当前的推理侧机会", meta: "公司 · 对比", prompt: "我想比较两家公司，请先问我公司名称，再从业务、竞争力、估值和风险展开。" },
  { icon: "paper", title: "阅读财报材料", summary: "从财报中识别需要验证的主线", meta: "材料 · 深度", prompt: "我会上传一份财报材料，请提取关键数据、管理层表述、变化和待验证问题。" },
  { icon: "track", title: "建立跟踪计划", summary: "为持仓或关注标的建立持续跟踪", meta: "任务 · 持续", prompt: "请根据我的持仓和关注标的，帮我建立一套持续跟踪计划。", action: "tracking" },
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
  const fallbackInsights: AgentWorkspaceInsight[] = [
    { id: "portfolio", eyebrow: "持仓研究", title: "梳理今天的组合变化", summary: "从持仓、新闻与事件中找出值得关注的变量" },
    { id: "event", eyebrow: "即将发生", title: "查看近期重要事件", summary: "把宏观日程与持仓财报放进同一条时间线" },
    { id: "research", eyebrow: "新研究", title: "建立一条研究主线", summary: "从问题出发，保留来源、结论与后续跟踪" },
  ];
  const visibleInsights = createMemo(() => {
    const source = props.insights.length ? props.insights : fallbackInsights;
    const query = props.searchQuery.trim().toLowerCase();
    if (!query) return source;
    return source.filter((item) => `${item.title} ${item.summary}`.toLowerCase().includes(query));
  });
  const promptForInsight = (item: AgentWorkspaceInsight) =>
    `请基于这条研究线索继续分析：${item.title}。${item.summary}`;
  return (
    <main class="agent-workspace-overview">
      <div class="agent-workspace-title-row">
        <div><h1>Agent</h1><div class="agent-workspace-context">正在基于：<span>我的组合</span><span>今日事件</span></div></div>
        <div class="agent-workspace-modes"><span>快速问答</span><strong>深度研究</strong><span>任务</span></div>
      </div>
      <section class="agent-workspace-greeting">
        <span class="agent-workspace-agent-mark"><AgentWorkspaceIcon name="agent" size={25} /></span>
        <div><h2>{props.greeting}</h2><p>今天有 {props.insightCount} 条值得继续研究的线索。</p></div>
      </section>
      <section class="agent-workspace-section">
        <div class="agent-workspace-section-heading"><h2>快速开始</h2><span>会展开来源与推理过程</span></div>
        <div class="agent-workspace-quick-grid">
          <For each={QUICK_STARTS}>{(item) => (
            <button type="button" onClick={() => item.action === "tracking" ? props.onTracking() : props.onPrompt(item.prompt)}>
              <AgentWorkspaceIcon name={item.icon} />
              <strong>{item.title}</strong><span>{item.summary}</span><small>{item.meta}</small>
            </button>
          )}</For>
        </div>
      </section>
      <section class="agent-workspace-section agent-workspace-insights">
        <div class="agent-workspace-section-heading"><h2>今日研究线索</h2><button type="button" onClick={props.onInsights}>查看洞察 <AgentWorkspaceIcon name="arrow" size={16} /></button></div>
        <div class="agent-workspace-insight-list">
          <Show when={visibleInsights().length > 0} fallback={<div class="agent-workspace-empty">没有匹配的研究线索，换一个关键词试试。</div>}>
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
          <h2>重要事件</h2>
          <button type="button" onClick={props.onCalendar}>
            财经日历 <AgentWorkspaceIcon name="arrow" size={16} />
          </button>
        </div>
        <button type="button" onClick={props.onCalendar}>
          <span class="agent-workspace-mobile-event-icon">
            <AgentWorkspaceIcon name="calendar" />
          </span>
          <span>
            <strong>{props.events[0]?.title ?? "查看我的财经日历"}</strong>
            <small>
              {props.events[0]
                ? `${props.events[0].date} ${props.events[0].time}`.trim()
                : "宏观日程与持仓财报"}
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
      <section><div class="agent-workspace-rail-heading"><h2>即将到来的事件</h2><button type="button" onClick={props.onCalendar}>财经日历</button></div>
        <div class="agent-workspace-event-list">
          <Show when={props.events.length > 0} fallback={<button type="button" onClick={props.onCalendar} class="agent-workspace-rail-empty"><AgentWorkspaceIcon name="calendar" /><span>查看你的财经日历</span></button>}>
            <For each={props.events}>{(event) => <button type="button" onClick={props.onCalendar}><span><strong>{event.title}</strong><small>{event.date}{event.time ? ` ${event.time}` : ""}</small><em>{event.summary}</em></span><AgentWorkspaceIcon name="arrow" size={15} /></button>}</For>
          </Show>
        </div>
      </section>
      <section><div class="agent-workspace-rail-heading"><h2>最近研究</h2></div>
        <div class="agent-workspace-saved-list">
          <Show when={props.research.length > 0} fallback={<p>发起研究后会自动保存在这里。</p>}>
            <For each={props.research.slice(0, 3)}>{(item) => <button type="button" onClick={() => props.onSelectResearch(item.id)}><strong>{item.title}</strong><small>继续这项研究</small></button>}</For>
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
  onPushes: () => void;
  onHistory?: () => void;
  onAccount: () => void;
}) {
  const avatar = () =>
    props.userName === "HONE 用户" || props.userName.startsWith("用户 ")
      ? "H"
      : props.userName.slice(-1);
  return <header class="agent-workspace-mobile-header"><HoneBrand /><div><Show when={props.onHistory}>{(onHistory) => <button type="button" onClick={onHistory()} aria-label="会话历史" class="agent-workspace-mobile-history-trigger"><AgentWorkspaceIcon name="history" /><Show when={(props.historyCount ?? 0) > 0}><span>{Math.min(props.historyCount ?? 0, 99)}</span></Show></button>}</Show><button type="button" onClick={props.onPushes} aria-label="通知"><AgentWorkspaceIcon name="bell" /><Show when={props.unreadPushCount > 0}><i /></Show></button><button type="button" onClick={props.onAccount} class="agent-workspace-mobile-avatar">{avatar()}</button></div></header>;
}

export function AgentWorkspaceHistoryDrawer(props: {
  open: boolean;
  research: ResearchItem[];
  hasOlder: boolean;
  loadingOlder: boolean;
  onClose: () => void;
  onSelectResearch: (id: string) => void;
  onLoadOlder: () => void;
}) {
  createEffect(() => {
    if (!props.open) return;
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") props.onClose();
    };
    document.addEventListener("keydown", closeOnEscape);
    onCleanup(() => document.removeEventListener("keydown", closeOnEscape));
  });

  return (
    <Show when={props.open}>
      <div class="agent-workspace-history-backdrop" onClick={props.onClose} />
      <aside class="agent-workspace-history-drawer" aria-label="会话历史" aria-modal="true" role="dialog">
        <header>
          <div><strong>会话历史</strong><span>最近的研究与提问</span></div>
          <button type="button" onClick={props.onClose} aria-label="关闭会话历史">×</button>
        </header>
        <div class="agent-workspace-history-drawer-list">
          <Show when={props.research.length > 0} fallback={<p>开始对话后，历史记录会出现在这里。</p>}>
            <For each={props.research}>{(item, index) => (
              <button type="button" onClick={() => props.onSelectResearch(item.id)}>
                <span>{String(index() + 1).padStart(2, "0")}</span>
                <strong>{item.title}</strong>
                <AgentWorkspaceIcon name="arrow" size={16} />
              </button>
            )}</For>
          </Show>
        </div>
        <Show when={props.hasOlder}>
          <button type="button" class="agent-workspace-history-more" disabled={props.loadingOlder} onClick={props.onLoadOlder}>
            {props.loadingOlder ? "正在加载…" : "加载更早记录"}
          </button>
        </Show>
      </aside>
    </Show>
  );
}

export function AgentWorkspaceMobileNav(props: {
  activeMode: "overview" | "conversation";
  activeSection?: "agent" | "invest" | "insights" | "tracking" | "me";
  communityUnread: boolean;
  onInvest: () => void;
  onInsights: () => void;
  onAgent: () => void;
  onTracking: () => void;
  onAccount: () => void;
}) {
  return <nav class="agent-workspace-mobile-nav" aria-label="主要导航">
    <button type="button" classList={{ "is-active": props.activeSection === "invest" }} onClick={props.onInvest}><AgentWorkspaceIcon name="invest" /><span>投资</span></button>
    <button type="button" onClick={props.onInsights} class="agent-workspace-mobile-has-dot" classList={{ "is-active": props.activeSection === "insights" }}><AgentWorkspaceIcon name="insight" /><span>洞察</span><Show when={props.communityUnread}><i /></Show></button>
    <button type="button" class="is-agent" classList={{ "is-active": props.activeSection === "agent" }} onClick={props.onAgent}><b><AgentWorkspaceIcon name="agent" size={27} /></b><span>Agent</span></button>
    <button type="button" classList={{ "is-active": props.activeSection === "tracking" }} onClick={props.onTracking}><AgentWorkspaceIcon name="track" /><span>跟踪</span></button>
    <button type="button" classList={{ "is-active": props.activeSection === "me" }} onClick={props.onAccount}><AgentWorkspaceIcon name="me" /><span>我的</span></button>
  </nav>;
}
