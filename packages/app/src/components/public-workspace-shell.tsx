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
  PublicPushCenter,
  PublicPushDetailDialog,
} from "@/components/public-push-center";
import {
  getPublicChatBootstrap,
  getPublicPushes,
  openPublicPush,
} from "@/lib/api";
import {
  latestUnreadPushId,
  mergePublicPushItems,
} from "@/lib/public-chat";
import { publicWorkspaceResearchFromHistory } from "@/lib/public-workspace-research";
import type {
  PublicPushDetail,
  PublicPushListItem,
} from "@/lib/types";
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
  }>,
) {
  const navigate = useNavigate();
  const [query, setQuery] = createSignal("");
  const [research, setResearch] = createSignal<
    ReturnType<typeof publicWorkspaceResearchFromHistory>
  >([]);
  const [researchLoading, setResearchLoading] = createSignal(true);
  const [historyDrawerOpen, setHistoryDrawerOpen] = createSignal(false);
  const [pushCenterOpen, setPushCenterOpen] = createSignal(false);
  const [pushItems, setPushItems] = createSignal<PublicPushListItem[]>([]);
  const [pushUnreadCount, setPushUnreadCount] = createSignal(0);
  const [pushNextBefore, setPushNextBefore] = createSignal<string>();
  const [pushLoading, setPushLoading] = createSignal(false);
  const [pushLoadingMore, setPushLoadingMore] = createSignal(false);
  const [pushError, setPushError] = createSignal<string>();
  const [pushDetailOpen, setPushDetailOpen] = createSignal(false);
  const [pushDetailLoading, setPushDetailLoading] = createSignal(false);
  const [pushDetailError, setPushDetailError] = createSignal<string>();
  const [pushDetail, setPushDetail] = createSignal<PublicPushDetail>();
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

  const loadPushes = async (mode: "reset" | "more" = "reset") => {
    if (mode === "more") {
      if (!pushNextBefore() || pushLoadingMore()) return;
      setPushLoadingMore(true);
    } else {
      if (pushLoading()) return;
      setPushLoading(true);
      setPushError(undefined);
    }
    try {
      const payload = await getPublicPushes(
        mode === "more" ? pushNextBefore() : undefined,
      );
      if (disposed) return;
      setPushItems((current) =>
        mode === "more"
          ? mergePublicPushItems(current, payload.items)
          : payload.items,
      );
      setPushUnreadCount(pushCenterOpen() ? 0 : payload.unread_count);
      setPushNextBefore(payload.next_before ?? undefined);
    } catch (error) {
      if (!disposed) {
        setPushError(error instanceof Error ? error.message : String(error));
      }
    } finally {
      if (!disposed) {
        setPushLoading(false);
        setPushLoadingMore(false);
      }
    }
  };

  const acknowledgePushCenter = async (unreadBeforeOpen: number) => {
    let items = pushItems();
    let unread = unreadBeforeOpen;
    try {
      if (items.length === 0) {
        const payload = await getPublicPushes();
        if (disposed) return;
        items = payload.items;
        unread = payload.unread_count;
        setPushItems(payload.items);
        setPushNextBefore(payload.next_before ?? undefined);
      }
      const latestPushId = latestUnreadPushId(items, unread);
      if (!latestPushId) return;
      const payload = await openPublicPush(latestPushId);
      if (!disposed) setPushUnreadCount(payload.unread_count);
    } catch (error) {
      if (!disposed) {
        setPushUnreadCount(unreadBeforeOpen || unread);
        setPushError(error instanceof Error ? error.message : String(error));
      }
    }
  };

  const openPushCenter = () => {
    const unreadBeforeOpen = pushUnreadCount();
    setPushCenterOpen(true);
    setPushUnreadCount(0);
    void acknowledgePushCenter(unreadBeforeOpen);
  };

  const openPushListItem = async (item: PublicPushListItem) => {
    setPushDetailOpen(true);
    setPushDetail(undefined);
    setPushDetailError(undefined);
    setPushDetailLoading(true);
    try {
      const payload = await openPublicPush(item.push_id);
      if (disposed) return;
      setPushDetail(payload.push);
      setPushUnreadCount(payload.unread_count);
    } catch (error) {
      if (!disposed) {
        setPushDetailError(error instanceof Error ? error.message : String(error));
      }
    } finally {
      if (!disposed) setPushDetailLoading(false);
    }
  };

  onMount(() => {
    void loadResearch();
    void loadPushes();
    const refresh = () => {
      if (document.visibilityState !== "visible") return;
      void loadResearch();
      void loadPushes();
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
      <PublicPushCenter
        open={pushCenterOpen()}
        items={pushItems()}
        loading={pushLoading()}
        loadingMore={pushLoadingMore()}
        error={pushError()}
        nextBefore={pushNextBefore()}
        onClose={() => setPushCenterOpen(false)}
        onOpenPush={(item) => void openPushListItem(item)}
        onLoadMore={() => void loadPushes("more")}
      />
      <PublicPushDetailDialog
        open={pushDetailOpen()}
        detail={pushDetail()}
        loading={pushDetailLoading()}
        error={pushDetailError()}
        onClose={() => setPushDetailOpen(false)}
      />
    </div>
  );
}
