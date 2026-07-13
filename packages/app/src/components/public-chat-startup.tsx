import { Show } from "solid-js";
import { HoneBrand } from "@/components/hone-brand";
import { AgentWorkspaceIcon } from "@/components/public-agent-workspace";

type PublicChatStartupProps = {
  failed?: boolean;
  title?: string;
  description?: string;
  onRetry?: () => void;
  retryLabel?: string;
};

export function PublicChatStartup(props: PublicChatStartupProps) {
  const content = (
    <div class="public-chat-startup-layout" role="status" aria-live="polite">
      <aside class="public-chat-startup-sidebar" aria-hidden="true">
        <div class="public-chat-startup-brand"><HoneBrand /></div>
        <span class="public-chat-startup-rail is-wide" />
        <span class="public-chat-startup-rail" />
        <span class="public-chat-startup-rail" />
      </aside>
      <main class="public-chat-startup-main">
        <div class="public-chat-startup-copy">
          <span class="public-chat-startup-kicker">HONE AGENT</span>
          <strong>{props.title ?? "正在准备你的对话"}</strong>
          <p>{props.description ?? "正在同步最近消息与投研上下文，请稍候。"}</p>
          <Show when={props.failed}>
            <button type="button" onClick={props.onRetry}>
              {props.retryLabel ?? "重新尝试"}
            </button>
          </Show>
        </div>
        <div class="public-chat-startup-thread" aria-hidden="true">
          <span class="public-chat-startup-line is-short" />
          <span class="public-chat-startup-line" />
          <span class="public-chat-startup-line is-medium" />
          <span class="public-chat-startup-bubble" />
        </div>
        <div class="public-chat-startup-composer" aria-hidden="true">
          <i />
          <span />
          <b />
        </div>
      </main>
    </div>
  );

  return (
    <div class="public-chat-startup-page">
      <header class="public-chat-startup-header">
        <div class="public-chat-startup-brand"><HoneBrand /></div>
        <span />
      </header>
      {content}
      <nav class="public-chat-startup-tabs" aria-hidden="true">
        <span><AgentWorkspaceIcon name="invest" /></span>
        <span><AgentWorkspaceIcon name="insight" /></span>
        <b><AgentWorkspaceIcon name="agent" /></b>
        <span><AgentWorkspaceIcon name="track" /></span>
        <span><AgentWorkspaceIcon name="me" /></span>
      </nav>
    </div>
  );
}
