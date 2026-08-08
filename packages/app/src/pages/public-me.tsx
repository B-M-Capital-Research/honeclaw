import { For, Show, createSignal, onCleanup, onMount } from "solid-js";
import { CONTENT } from "@/lib/public-content";
import { useLocale } from "@/lib/i18n";
import { useNavigate } from "@solidjs/router";
import { PublicChatStartup } from "@/components/public-chat-startup";
import { PublicLoginForm } from "@/components/public-login-form";
import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import { PublicHoldingsPanel } from "@/components/public-holdings-panel";
import { PublicSettingsPanel } from "@/components/public-settings-panel";
import { PublicAdminWhitelistPanel } from "@/components/public-admin-whitelist-panel";
import { PublicAdminUsagePanel } from "@/components/public-admin-usage-panel";
import {
  createStripePortal,
  getPublicAuthMe,
  getPublicBillingConfig,
  publicLogout,
} from "@/lib/api";
import { workspaceUserName } from "@/lib/public-agent-workspace";
import {
  cachedPublicUser,
  hasCachedPublicUser,
  setCachedPublicUser,
} from "@/lib/public-session-cache";
import {
  billingEntitlementGrantsAccess,
  billingEntitlementStatusLabel,
  billingProviderLabel,
  publicUserHasProductAccess,
} from "@/lib/public-membership";
import type {
  PublicAuthUserInfo,
  PublicBillingEntitlement,
} from "@/lib/types";

function formatDate(value?: string) {
  if (!value) return "—";
  return new Date(value).toLocaleDateString(useLocale() === "en" ? "en-US" : "zh-CN", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}

function AccountRow(props: { label: string; value: string }) {
  return <div class="public-account-row"><span>{props.label}</span><strong>{props.value}</strong></div>;
}

/// Read at render time; a module-level const would freeze whichever language
/// was active at import.
const vipBenefits = () => [
  CONTENT.chat_page.me_page.plan_live,
  CONTENT.chat_page.me_page.plan_group,
  CONTENT.chat_page.me_page.plan_planet,
  CONTENT.chat_page.me_page.plan_qa,
];

function EntitlementRow(props: {
  entitlement: PublicBillingEntitlement;
  onStripeManage: () => void;
  managing: boolean;
  managementAllowed: boolean;
}) {
  const active = () => billingEntitlementGrantsAccess(props.entitlement);
  const recurring = () => props.entitlement.entitlement_kind === "recurring_subscription";
  const kindLabel = () =>
    props.entitlement.entitlement_kind === "fixed_term_purchase"
      ? CONTENT.chat_page.me_page.fixed_term_label
      : recurring()
        ? CONTENT.chat_page.me_page.recurring_label
        : billingProviderLabel(props.entitlement.provider);
  return (
    <article class="public-account-card public-billing-entitlement">
      <div class="public-account-row">
        <span>{billingProviderLabel(props.entitlement.provider)} · {kindLabel()}</span>
        <strong>{billingEntitlementStatusLabel(props.entitlement)}</strong>
      </div>
      <Show when={props.entitlement.current_period_end}>
        {(periodEnd) => (
          <p>
            {recurring()
              ? CONTENT.chat_page.me_page.cycle_until
              : CONTENT.chat_page.me_page.expires_on} {formatDate(periodEnd())}
            {props.entitlement.cancel_at_period_end
              ? CONTENT.chat_page.me_page.no_renew
              : ""}
          </p>
        )}
      </Show>
      <Show when={props.managementAllowed && props.entitlement.provider === "stripe" && recurring()}>
        <button type="button" disabled={props.managing} onClick={props.onStripeManage}>
          {props.managing ? CONTENT.chat_page.me_page.opening : CONTENT.chat_page.me_page.manage_stripe}
        </button>
      </Show>
      <Show when={!active()}>
        <small>{CONTENT.chat_page.me_page.no_access_sub}</small>
      </Show>
    </article>
  );
}

function MembershipCard(props: {
  user: PublicAuthUserInfo;
  syncing: boolean;
  managementAllowed: boolean;
}) {
  const [qrOpen, setQrOpen] = createSignal(false);
  const [managing, setManaging] = createSignal(false);
  const [manageError, setManageError] = createSignal("");
  const active = () => publicUserHasProductAccess(props.user);
  const international = () => props.user.identity_kind === "international_email";

  const openStripePortal = async () => {
    setManageError("");
    setManaging(true);
    try {
      const { portal_url } = await createStripePortal();
      window.location.assign(portal_url);
    } catch (cause) {
      setManageError(cause instanceof Error ? cause.message : String(cause));
      setManaging(false);
    }
  };

  return (
    <section class="public-workspace-panel public-vip-card">
      <div class="public-vip-main">
        <div class="public-vip-head">
          <span class="public-vip-badge">
            {international() ? CONTENT.chat_page.me_page.intl_member : CONTENT.chat_page.me_page.cn_invite}
          </span>
        </div>
        <h2>{active() ? CONTENT.chat_page.me_page.entitled : CONTENT.chat_page.me_page.not_entitled}</h2>
        <Show when={props.syncing && !active()}>
          <p role="status">{CONTENT.chat_page.me_page.awaiting_payment}</p>
        </Show>
        <Show when={props.user.billing.has_duplicate_active_subscriptions}>
          <p class="public-billing-warning" role="alert">
            {CONTENT.chat_page.me_page.duplicate_subs}
          </p>
        </Show>
        <Show
          when={active()}
          fallback={<p>{CONTENT.chat_page.me_page.paused_note}</p>}
        >
          <p>{international() ? CONTENT.chat_page.me_page.any_stripe_grants : CONTENT.chat_page.me_page.cn_channel_grants}</p>
          <ul class="public-vip-list">
            {vipBenefits().map((benefit) => (
              <li>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 6 9 17l-5-5" /></svg>
                <span>{benefit}</span>
              </li>
            ))}
          </ul>
        </Show>

        <Show when={props.user.billing.entitlements.length > 0}>
          <div class="public-billing-entitlements">
            <For each={props.user.billing.entitlements}>
              {(entitlement) => (
                <EntitlementRow
                  entitlement={entitlement}
                  managing={managing()}
                  managementAllowed={props.managementAllowed}
                  onStripeManage={openStripePortal}
                />
              )}
            </For>
          </div>
        </Show>
        <Show when={manageError()}>{(message) => <p role="alert">{message()}</p>}</Show>
      </div>
      <div class="public-vip-side">
        <button type="button" class="public-vip-qr" onClick={() => setQrOpen(true)}>
          <img src="/membership_wechat.jpg" alt={CONTENT.chat_page.me_page.wecom_qr_alt} loading="lazy" />
        </button>
        <span class="public-vip-side-copy">
          <strong>{CONTENT.chat_page.me_page.support_cta}</strong>
          <small>{CONTENT.chat_page.me_page.support_hint}</small>
        </span>
      </div>
      <Show when={qrOpen()}>
        <div class="public-vip-qr-pop" onClick={() => setQrOpen(false)}>
          <figure onClick={(event) => event.stopPropagation()}>
            <figcaption>
              <strong>{CONTENT.chat_page.me_page.support_title}</strong>
              <button type="button" aria-label={CONTENT.chat_page.me_page.close} onClick={() => setQrOpen(false)}>×</button>
            </figcaption>
            <img src="/membership_wechat.jpg" alt={CONTENT.chat_page.me_page.wecom_qr_alt} />
            <small>{CONTENT.chat_page.me_page.save_qr_hint}</small>
          </figure>
        </div>
      </Show>
    </section>
  );
}

function AccountView(props: {
  user: PublicAuthUserInfo;
  syncing: boolean;
  managementAllowed: boolean;
  onLogout: () => void;
}) {
  const navigate = useNavigate();
  return (
    <PublicWorkspaceShell active="me" userName={workspaceUserName(props.user.user_id)}>
      <div class="public-workspace-inner">
        <header class="public-workspace-page-heading">
          <div>
            <span class="public-workspace-eyebrow">{CONTENT.chat_page.me_page.personal_space}</span>
            <h1>{CONTENT.chat_page.me_page.me}</h1>
            <p>{CONTENT.chat_page.me_page.me_subtitle}</p>
          </div>
        </header>
        <Show when={publicUserHasProductAccess(props.user)}>
          <PublicHoldingsPanel />
          <PublicSettingsPanel />
        </Show>
        <Show when={props.user.is_admin}>
          <PublicAdminUsagePanel />
          <PublicAdminWhitelistPanel />
        </Show>
        <MembershipCard
          user={props.user}
          syncing={props.syncing}
          managementAllowed={props.managementAllowed}
        />
        <div class="public-account-grid">
          <section class="public-workspace-panel public-account-card">
            <h2>{CONTENT.chat_page.me_page.account_info}</h2>
            <AccountRow label={CONTENT.chat_page.me_page.account} value={props.user.user_id} />
            <AccountRow
              label={CONTENT.chat_page.me_page.verify_channel}
              value={props.user.identity_kind === "international_email"
                ? CONTENT.chat_page.me_page.email_channel.replace(
                    "{value}",
                    props.user.email_hint ? ` · ${props.user.email_hint}` : "",
                  )
                : CONTENT.chat_page.me_page.cn_phone_invite}
            />
            <AccountRow label={CONTENT.chat_page.me_page.registered_at} value={formatDate(props.user.created_at)} />
            <AccountRow label={CONTENT.chat_page.me_page.last_login} value={formatDate(props.user.last_login_at)} />
            <AccountRow
              label={CONTENT.chat_page.me_page.access}
              value={publicUserHasProductAccess(props.user)
                ? props.user.daily_limit > 0
                  ? CONTENT.chat_page.me_page.enabled_quota.replace(
                      "{count}",
                      String(props.user.daily_limit),
                    )
                  : CONTENT.chat_page.me_page.enabled
                : CONTENT.chat_page.me_page.paused}
            />
          </section>
          <section>
            <div class="public-account-actions">
              <Show
                when={publicUserHasProductAccess(props.user)}
                fallback={<button type="button" class="is-primary" onClick={() => navigate("/plan")}>{CONTENT.chat_page.me_page.view_membership}</button>}
              >
                <button type="button" class="is-primary" onClick={() => navigate("/chat")}>{CONTENT.chat_page.me_page.open_agent}</button>
                <button type="button" onClick={() => navigate("/community")}>{CONTENT.chat_page.me_page.open_community}</button>
              </Show>
              <button type="button" class="is-danger" onClick={props.onLogout}>{CONTENT.chat_page.me_page.sign_out}</button>
            </div>
            <p class="public-account-note">{CONTENT.chat_page.me_page.billing_note}</p>
          </section>
        </div>
        <section class="public-workspace-panel public-account-card public-account-links">
          <h2>{CONTENT.chat_page.me_page.about_help}</h2>
          <nav aria-label={CONTENT.chat_page.me_page.about_help}>
            <a href="/">{CONTENT.chat_page.me_page.home}</a><a href="/plan">{CONTENT.chat_page.me_page.pricing}</a><a href="/blog">Blog</a>
            <a href="/roadmap">{CONTENT.chat_page.me_page.roadmap}</a><a href="/terms">{CONTENT.chat_page.me_page.tos}</a><a href="/privacy">{CONTENT.chat_page.me_page.privacy}</a>
          </nav>
          <p class="public-account-note">{CONTENT.chat_page.me_page.disclaimer}</p>
        </section>
      </div>
    </PublicWorkspaceShell>
  );
}

export default function PublicMePage() {
  const navigate = useNavigate();
  const [user, setUser] = createSignal<PublicAuthUserInfo | null>(cachedPublicUser());
  // Arriving from a chat that already knows the user must not blank the screen
  // for a round-trip. Paint from what is known, then revalidate behind it.
  const [loading, setLoading] = createSignal(!hasCachedPublicUser());
  const [managementAllowed, setManagementAllowed] = createSignal(false);
  const [syncing, setSyncing] = createSignal(
    new URLSearchParams(window.location.search).has("checkout") ||
      new URLSearchParams(window.location.search).has("billing"),
  );
  let pollTimer: ReturnType<typeof setInterval> | undefined;
  let pollCount = 0;

  const load = async (initial = false) => {
    if (initial) setLoading(true);
    try {
      const next = await getPublicAuthMe();
      setUser(next);
      if (syncing() && next.billing.access_granted) {
        setSyncing(false);
        if (pollTimer) clearInterval(pollTimer);
        window.history.replaceState({}, "", "/me");
      }
    } catch {
      setUser(null);
      setCachedPublicUser(null);
      setSyncing(false);
      if (pollTimer) clearInterval(pollTimer);
    } finally {
      if (initial) setLoading(false);
    }
  };

  onMount(() => {
    void load(true);
    void getPublicBillingConfig()
      .then((config) => setManagementAllowed(config.management_allowed_on_this_client))
      .catch(() => setManagementAllowed(false));
    if (syncing()) {
      pollTimer = setInterval(() => {
        pollCount += 1;
        if (pollCount >= 30) {
          setSyncing(false);
          if (pollTimer) clearInterval(pollTimer);
          return;
        }
        void load();
      }, 2000);
    }
  });
  onCleanup(() => {
    if (pollTimer) clearInterval(pollTimer);
  });

  const logout = async () => {
    try {
      await publicLogout();
    } finally {
      setUser(null);
      setCachedPublicUser(null);
      navigate("/chat");
    }
  };

  return (
    <Show when={!loading()} fallback={<PublicChatStartup title={CONTENT.chat_page.me_page.loading_title} description={CONTENT.chat_page.me_page.loading_detail} />}>
      <Show when={user()} fallback={<PublicLoginForm onLogin={() => void load(true)} />}>
        {(currentUser) => (
          <AccountView
            user={currentUser()}
            syncing={syncing()}
            managementAllowed={managementAllowed()}
            onLogout={logout}
          />
        )}
      </Show>
    </Show>
  );
}
