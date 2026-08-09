import {
  createSignal,
  onCleanup,
  onMount,
  type ParentProps,
} from "solid-js";
import { useNavigate } from "@solidjs/router";
import { CONTENT } from "@/lib/public-content";
import {
  AgentWorkspaceHistoryDrawer,
  AgentWorkspaceMobileHeader,
  AgentWorkspaceMobileNav,
  AgentWorkspaceSidebar,
  AgentWorkspaceTopbar,
} from "@/components/public-agent-workspace";
import { PublicPrefsButton } from "@/components/public-prefs-button";
import {
  getPublicChatBootstrap,
  getPublicPushes,
} from "@/lib/api";
import { publicWorkspaceResearchFromHistory } from "@/lib/public-workspace-research";
import {
  publicPushUnreadCount,
  setPublicPushUnreadCount,
} from "@/lib/public-push-unread";
import "@/pages/public-foundation.css";
import "@/pages/public-site.css";
import "@/pages/public-agent-workspace.css";
import "@/pages/public-workspace.css";

export type PublicWorkspaceSection = "pushes" | "insights" | "me";

export function PublicWorkspaceShell(
  props: ParentProps<{
    active: PublicWorkspaceSection;
    userName?: string;
    communityUnread?: boolean;
    topbarLabel?: string;
    searchPlaceholder?: string;
    onSearch?: (value: string) => void;
    /** The push page owns this while it performs authoritative read-through. */
    pushUnreadCount?: number;
  }>,
) {
  const navigate = useNavigate();
  const [query, setQuery] = createSignal("");
  const [research, setResearch] = createSignal<
    ReturnType<typeof publicWorkspaceResearchFromHistory>
  >([]);
  const [researchLoading, setResearchLoading] = createSignal(true);
  const [historyDrawerOpen, setHistoryDrawerOpen] = createSignal(false);
  let disposed = false;
  let bootstrapController: AbortController | undefined;

  const updateQuery = (value: string) => {
    setQuery(value);
    props.onSearch?.(value);
  };
  const goAgent = () => navigate("/chat");
  const startNewResearch = () => navigate("/chat?new=1");
  const openResearch = (id: string) => {
    setHistoryDrawerOpen(false);
    navigate(`/chat?research=${encodeURIComponent(id)}`);
  };

  const loadResearch = async () => {
    bootstrapController?.abort();
    const controller = new AbortController();
    bootstrapController = controller;
    setResearchLoading(true);
    try {
      const bootstrap = await getPublicChatBootstrap(controller.signal);
      if (disposed || controller.signal.aborted) return;
      setResearch(
        publicWorkspaceResearchFromHistory(
          bootstrap.messages ?? [],
          bootstrap.history_start,
        ),
      );
    } catch {
      if (!disposed && !controller.signal.aborted) setResearch([]);
    } finally {
      if (!disposed && !controller.signal.aborted) setResearchLoading(false);
      if (bootstrapController === controller) bootstrapController = undefined;
    }
  };

  const pushUnreadCount = () => props.pushUnreadCount ?? publicPushUnreadCount();

  const loadPushUnreadCount = async () => {
    if (props.pushUnreadCount !== undefined) return;
    try {
      const payload = await getPublicPushes(undefined, 1);
      if (disposed) return;
      setPublicPushUnreadCount(payload.unread_count);
    } catch {
      // A badge refresh must never interrupt the workspace itself.
    }
  };

  const openPushCenter = () => navigate("/pushes");

  onMount(() => {
    void loadResearch();
    void loadPushUnreadCount();
    const refresh = () => {
      if (document.visibilityState !== "visible") return;
      void loadResearch();
      void loadPushUnreadCount();
    };
    window.addEventListener("focus", refresh);
    document.addEventListener("visibilitychange", refresh);
    onCleanup(() => {
      disposed = true;
      bootstrapController?.abort();
      window.removeEventListener("focus", refresh);
      document.removeEventListener("visibilitychange", refresh);
    });
  });

  return (
    <div class="public-chat-page public-chat-page--ready public-workspace-page">
      <AgentWorkspaceSidebar
        userName={props.userName ?? `HONE ${CONTENT.chat_page.misc.shell_user}`}
        research={research()}
        researchLoading={researchLoading()}
        activeMode="conversation"
        activeSection={props.active}
        onPushes={() => navigate("/pushes")}
        unreadPushCount={pushUnreadCount()}
        communityUnread={props.communityUnread ?? false}
        onNewResearch={startNewResearch}
        onSelectResearch={openResearch}
        onHome={goAgent}
        onInsights={() => navigate("/community")}
        onAccount={() => navigate("/me")}
        onLogout={() => navigate("/me")}
      />
      <div class="agent-workspace-stage public-workspace-stage">
        <AgentWorkspaceTopbar
          query={query()}
          unreadPushCount={pushUnreadCount()}
          label={props.topbarLabel ?? CONTENT.chat_page.misc.shell_motto}
          placeholder={props.searchPlaceholder}
          showSearch={props.onSearch !== undefined}
          onQueryChange={updateQuery}
          preferences={<PublicPrefsButton />}
          onPushes={openPushCenter}
        />
        <AgentWorkspaceMobileHeader
          userName={props.userName ?? `HONE ${CONTENT.chat_page.misc.shell_user}`}
          unreadPushCount={pushUnreadCount()}
          historyCount={research().length}
          preferences={<PublicPrefsButton />}
          onMenu={() => setHistoryDrawerOpen(true)}
          onPushes={openPushCenter}
          onAccount={() => navigate("/me")}
        />
        <main class="public-workspace-content">{props.children}</main>
      </div>
      <AgentWorkspaceMobileNav
        activeMode="conversation"
        activeSection={props.active}
        communityUnread={props.communityUnread ?? false}
        unreadPushCount={pushUnreadCount()}
        onHome={goAgent}
        onInsights={() => navigate("/community")}
        onAgent={goAgent}
        onPushesTab={() => navigate("/pushes")}
        onAccount={() => navigate("/me")}
      />
      <AgentWorkspaceHistoryDrawer
        open={historyDrawerOpen()}
        userName={props.userName ?? `HONE ${CONTENT.chat_page.misc.shell_user}`}
        research={research()}
        hasOlder={false}
        loadingOlder={false}
        communityUnread={props.communityUnread ?? false}
        researchLoading={researchLoading()}
        onOpen={() => setHistoryDrawerOpen(true)}
        onClose={() => setHistoryDrawerOpen(false)}
        onSelectResearch={openResearch}
        onLoadOlder={() => undefined}
        onNewResearch={startNewResearch}
        onHome={goAgent}
        onInsights={() => navigate("/community")}
        onAccount={() => navigate("/me")}
      />
    </div>
  );
}
