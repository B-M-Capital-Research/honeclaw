import { createSignal, type ParentProps } from "solid-js";
import { useNavigate } from "@solidjs/router";
import {
  AgentWorkspaceMobileHeader,
  AgentWorkspaceMobileNav,
  AgentWorkspaceSidebar,
  AgentWorkspaceTopbar,
} from "@/components/public-agent-workspace";
import "@/pages/public-foundation.css";
import "@/pages/public-site.css";
import "@/pages/public-agent-workspace.css";
import "@/pages/public-workspace.css";

export type PublicWorkspaceSection =
  | "invest"
  | "insights"
  | "tracking"
  | "me";

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
  const updateQuery = (value: string) => {
    setQuery(value);
    props.onSearch?.(value);
  };
  const goTracking = () => navigate("/portfolio");
  const goAgent = () => navigate("/chat");
  return (
    <div class="public-chat-page public-chat-page--ready public-workspace-page">
      <AgentWorkspaceSidebar
        userName={props.userName ?? "HONE 用户"}
        research={[]}
        activeMode="conversation"
        activeSection={props.active}
        communityUnread={props.communityUnread ?? false}
        onNewResearch={goAgent}
        onSelectResearch={goAgent}
        onInvest={() => navigate("/portfolio")}
        onInsights={() => navigate("/community")}
        onTracking={goTracking}
        onAccount={() => navigate("/me")}
        onLogout={() => navigate("/me")}
      />
      <div class="agent-workspace-stage public-workspace-stage">
        <AgentWorkspaceTopbar
          query={query()}
          unreadPushCount={0}
          label={props.topbarLabel ?? "长期研究，理性决策，复利为王。"}
          placeholder={props.searchPlaceholder}
          onQueryChange={updateQuery}
          onPushes={goAgent}
        />
        <AgentWorkspaceMobileHeader
          userName={props.userName ?? "HONE 用户"}
          unreadPushCount={0}
          onPushes={goAgent}
          onAccount={() => navigate("/me")}
        />
        <main class="public-workspace-content">{props.children}</main>
      </div>
      <AgentWorkspaceMobileNav
        activeMode="conversation"
        activeSection={props.active}
        communityUnread={props.communityUnread ?? false}
        onInvest={() => navigate("/portfolio")}
        onInsights={() => navigate("/community")}
        onAgent={goAgent}
        onTracking={goTracking}
        onAccount={() => navigate("/me")}
      />
    </div>
  );
}
