import { Show, createSignal, onMount } from "solid-js";
import { useNavigate } from "@solidjs/router";

import { CONTENT } from "@/lib/public-content";
import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import { PublicSubscriptionManager } from "@/components/public-subscription-manager";
import { PublicLoginForm } from "@/components/public-login-form";
import { getPublicAuthMe } from "@/lib/api";
import { workspaceUserName } from "@/lib/public-agent-workspace";
import {
  cachedPublicUser,
  hasCachedPublicUser,
  setCachedPublicUser,
} from "@/lib/public-session-cache";
import type { PublicAuthUserInfo } from "@/lib/types";
import "./public-pushes.css";

/**
 * The push section.
 *
 * Scheduled pushes previously lived behind a bell icon that opened a read-only
 * modal, so people received them without any visible way to change or stop
 * them. Giving the thing its own tab is most of the fix: it becomes somewhere
 * you can go, not something you have to discover.
 */
export default function PublicPushesPage() {
  const navigate = useNavigate();
  // Same treatment as the account page: paint from what is already known and
  // revalidate behind it rather than blanking the viewport for a round-trip.
  const [user, setUser] = createSignal<PublicAuthUserInfo | null>(cachedPublicUser());
  const [loading, setLoading] = createSignal(!hasCachedPublicUser());

  const load = async () => {
    try {
      setUser(await getPublicAuthMe());
    } catch {
      setUser(null);
      setCachedPublicUser(null);
    } finally {
      setLoading(false);
    }
  };

  onMount(() => void load());

  return (
    <Show
      when={!loading() || user()}
      fallback={
        <div class="public-pushes-loading" role="status">
          {CONTENT.chat_page.subscriptions.loading}
        </div>
      }
    >
      <Show when={user()} fallback={<PublicLoginForm onLogin={() => void load()} />}>
        {(currentUser) => (
          <PublicWorkspaceShell
            active="pushes"
            userName={workspaceUserName(currentUser().user_id)}
          >
            <main class="public-pushes-main">
              <PublicSubscriptionManager />
              <button
                type="button"
                class="public-pushes-back"
                onClick={() => navigate("/chat")}
              >
                {CONTENT.chat_page.subscriptions.back_to_chat}
              </button>
            </main>
          </PublicWorkspaceShell>
        )}
      </Show>
    </Show>
  );
}
