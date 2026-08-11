import { Show } from "solid-js";
import { CONTENT } from "@/lib/public-content";
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
          <span class="public-chat-startup-kicker">HONE 投资助手</span>
          <strong>{props.title ?? CONTENT.chat_page.misc.startup_title}</strong>
          <p>{props.description ?? CONTENT.chat_page.misc.startup_detail}</p>
          <Show when={props.failed}>
            <button type="button" onClick={props.onRetry}>
              {props.retryLabel ?? CONTENT.chat_page.misc.startup_retry}
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
