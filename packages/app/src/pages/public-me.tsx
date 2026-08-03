import { For, Show, createSignal, onCleanup, onMount } from "solid-js";
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
  return new Date(value).toLocaleDateString("zh-CN", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}

function AccountRow(props: { label: string; value: string }) {
  return <div class="public-account-row"><span>{props.label}</span><strong>{props.value}</strong></div>;
}

const VIP_BENEFITS = [
  "每周四主理人深度公司讲解，在线直播可任意提问",
  "VIP 群与 500+ 高手畅聊，持续分享深度投研资料与实时动态",
  "知识星球与社区：完整的公司研报、估值和投资策略分享",
  "HONE 畅享：任何问题在社区都能得到及时反馈",
];

function EntitlementRow(props: {
  entitlement: PublicBillingEntitlement;
  onStripeManage: () => void;
  managing: boolean;
  managementAllowed: boolean;
}) {
  const active = () => billingEntitlementGrantsAccess(props.entitlement);
  return (
    <article class="public-account-card public-billing-entitlement">
      <div class="public-account-row">
        <span>{billingProviderLabel(props.entitlement.provider)}</span>
        <strong>{billingEntitlementStatusLabel(props.entitlement)}</strong>
      </div>
      <Show when={props.entitlement.current_period_end}>
        {(periodEnd) => (
          <p>
            当前周期至 {formatDate(periodEnd())}
            {props.entitlement.cancel_at_period_end ? "，到期后不再自动续费。" : "。"}
          </p>
        )}
      </Show>
      <Show when={props.managementAllowed && props.entitlement.provider === "stripe"}>
        <button type="button" disabled={props.managing} onClick={props.onStripeManage}>
          {props.managing ? "正在打开…" : "在 Stripe 管理订阅"}
        </button>
      </Show>
      <Show when={props.managementAllowed && props.entitlement.provider === "whop" && props.entitlement.manage_url}>
        {(manageUrl) => (
          <a href={manageUrl()} target="_blank" rel="noopener noreferrer">在 Whop 管理订阅 →</a>
        )}
      </Show>
      <Show when={!active()}>
        <small>这条订阅当前不授予 HONE 访问权限。</small>
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
            {international() ? "国际会员 · 统一权益" : "HONE 账号 · 国内邀请"}
          </span>
        </div>
        <h2>{active() ? "你的 HONE 权益已启用" : "会员权益当前不可用"}</h2>
        <Show when={props.syncing && !active()}>
          <p role="status">正在等待付款平台确认。页面会自动刷新；成功跳转不会直接开通权益。</p>
        </Show>
        <Show when={props.user.billing.has_duplicate_active_subscriptions}>
          <p class="public-billing-warning" role="alert">
            检测到多条有效订阅。HONE 访问不会中断，但你可能被重复扣费，请分别检查 Stripe 与 Whop 并取消不需要的一条。
          </p>
        </Show>
        <Show
          when={active()}
          fallback={<p>账号资料仍会保留，付费功能已暂停。你可以恢复付款或重新订阅，服务端确认后会自动恢复访问。</p>}
        >
          <p>{international() ? "任意一条有效 Stripe 或 Whop 权益都可授予访问：" : "该账号由国内邀请渠道授予访问："}</p>
          <ul class="public-vip-list">
            {VIP_BENEFITS.map((benefit) => (
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
          <img src="/membership_wechat.jpg" alt="企业微信客服二维码" loading="lazy" />
        </button>
        <span class="public-vip-side-copy">
          <strong>有疑问？加客服微信</strong>
          <small>扫码添加企业微信客服，会员与账单问题随时咨询。</small>
        </span>
      </div>
      <Show when={qrOpen()}>
        <div class="public-vip-qr-pop" onClick={() => setQrOpen(false)}>
          <figure onClick={(event) => event.stopPropagation()}>
            <figcaption>
              <strong>企业微信客服</strong>
              <button type="button" aria-label="关闭" onClick={() => setQrOpen(false)}>×</button>
            </figcaption>
            <img src="/membership_wechat.jpg" alt="企业微信客服二维码" />
            <small>长按或右键保存图片，扫码添加客服。</small>
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
            <span class="public-workspace-eyebrow">个人研究空间</span>
            <h1>我的</h1>
            <p>自选与持仓、投资画像风格、账户信息和订阅都在这里管理。</p>
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
            <h2>账户信息</h2>
            <AccountRow label="账户" value={props.user.user_id} />
            <AccountRow
              label="验证渠道"
              value={props.user.identity_kind === "international_email"
                ? `邮箱${props.user.email_hint ? ` · ${props.user.email_hint}` : ""}`
                : "国内手机号邀请"}
            />
            <AccountRow label="注册时间" value={formatDate(props.user.created_at)} />
            <AccountRow label="最近登录" value={formatDate(props.user.last_login_at)} />
            <AccountRow
              label="访问权限"
              value={publicUserHasProductAccess(props.user)
                ? props.user.daily_limit > 0
                  ? `已启用 · 每日 ${props.user.daily_limit} 次`
                  : "已启用"
                : "已暂停"}
            />
          </section>
          <section>
            <div class="public-account-actions">
              <Show
                when={publicUserHasProductAccess(props.user)}
                fallback={<button type="button" class="is-primary" onClick={() => navigate("/plan")}>查看会员与续费</button>}
              >
                <button type="button" class="is-primary" onClick={() => navigate("/chat")}>进入 Agent</button>
                <button type="button" onClick={() => navigate("/community")}>去社区看看</button>
              </Show>
              <button type="button" class="is-danger" onClick={props.onLogout}>退出登录</button>
            </div>
            <p class="public-account-note">Stripe 与 Whop 只负责账单处理；HONE 以服务端统一权益记录决定访问权限。</p>
          </section>
        </div>
        <section class="public-workspace-panel public-account-card public-account-links">
          <h2>关于与帮助</h2>
          <nav aria-label="关于与帮助">
            <a href="/">官网首页</a><a href="/plan">会员与定价</a><a href="/blog">Blog</a>
            <a href="/roadmap">路线图与文档</a><a href="/terms">用户协议</a><a href="/privacy">隐私政策</a>
          </nav>
          <p class="public-account-note">内容仅供研究参考，不构成投资建议。市场有风险，决策需独立判断。</p>
        </section>
      </div>
    </PublicWorkspaceShell>
  );
}

export default function PublicMePage() {
  const navigate = useNavigate();
  const [user, setUser] = createSignal<PublicAuthUserInfo | null>(null);
  const [loading, setLoading] = createSignal(true);
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
      navigate("/chat");
    }
  };

  return (
    <Show when={!loading()} fallback={<PublicChatStartup title="正在加载个人空间" description="正在确认账户与研究权限。" />}>
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
