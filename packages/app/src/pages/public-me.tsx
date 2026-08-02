import { createSignal, onMount, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { PublicChatStartup } from "@/components/public-chat-startup";
import { PublicLoginForm } from "@/components/public-login-form";
import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import { PublicHoldingsPanel } from "@/components/public-holdings-panel";
import { PublicSettingsPanel } from "@/components/public-settings-panel";
import { PublicAdminWhitelistPanel } from "@/components/public-admin-whitelist-panel";
import { PublicAdminUsagePanel } from "@/components/public-admin-usage-panel";
import { getPublicAuthMe, publicLogout } from "@/lib/api";
import { workspaceUserName } from "@/lib/public-agent-workspace";
import {
  publicUserHasProductAccess,
  whopMembershipStatusLabel,
} from "@/lib/public-membership";
import type { PublicAuthUserInfo } from "@/lib/types";

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
  "VIP 群与 500+ 高手畅聊，禁言群持续分享深度投研资料与实时动态",
  "知识星球 & 社区：完整的公司研报、估值和投资策略分享",
  "HONE 畅享：任何问题在社区都能得到及时反馈",
];

function MembershipCard(props: { user: PublicAuthUserInfo }) {
  const [qrOpen, setQrOpen] = createSignal(false);
  const membership = () => props.user.whop_membership;
  const isWhop = () => props.user.registration_policy === "whop_international";
  const active = () => publicUserHasProductAccess(props.user);
  const badge = () => {
    const current = membership();
    if (!isWhop()) return "HONE 账号 · 已启用";
    return `Whop 会员 · ${current ? whopMembershipStatusLabel(current) : "等待同步"}`;
  };
  const heading = () => {
    if (!isWhop()) return "你的 HONE 账号已启用";
    return active() ? "Whop 会员已连接 HONE" : "Whop 会员当前不可用";
  };
  return (
    <section class="public-workspace-panel public-vip-card">
      <div class="public-vip-main">
        <div class="public-vip-head">
          <span class="public-vip-badge">{badge()}</span>
        </div>
        <h2>{heading()}</h2>
        <Show
          when={isWhop()}
          fallback={
            <p>
              该账号通过国内手机号渠道验证。会员、付费和续费状态以国内购买渠道及客服记录为准。
            </p>
          }
        >
          <Show
            when={active()}
            fallback={
              <p>
                账号资料仍会保留，但 HONE 付费功能已暂停。你可以前往 Whop
                查看账单、恢复订阅，状态同步后会自动恢复访问。
              </p>
            }
          >
            <p>购买邮箱已验证，当前 Whop 会员权益已同步到 HONE：</p>
            <ul class="public-vip-list">
              {VIP_BENEFITS.map((benefit) => (
                <li>
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 6 9 17l-5-5" /></svg>
                  <span>{benefit}</span>
                </li>
              ))}
            </ul>
          </Show>
          <Show when={membership()?.renewal_period_end}>
            {(periodEnd) => (
              <p>
                当前周期至 {formatDate(periodEnd())}
                {membership()?.cancel_at_period_end
                  ? "，到期后不再自动续费。"
                  : "。"}
              </p>
            )}
          </Show>
          <Show when={membership()?.manage_url}>
            {(manageUrl) => (
              <p>
                <a
                  href={manageUrl()}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  在 Whop 管理订阅 →
                </a>
              </p>
            )}
          </Show>
        </Show>
      </div>
      <div class="public-vip-side">
        <button type="button" class="public-vip-qr" onClick={() => setQrOpen(true)}>
          <img src="/membership_wechat.jpg" alt="企业微信客服二维码" loading="lazy" />
        </button>
        <span class="public-vip-side-copy">
          <strong>有疑问？加客服微信</strong>
          <small>扫码添加企业微信客服，会员与使用问题随时咨询。</small>
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
            <p>自选与持仓、投资画像风格、账户信息都在这里管理。</p>
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
        <MembershipCard user={props.user} />
        <div class="public-account-grid">
          <section class="public-workspace-panel public-account-card">
            <h2>账户信息</h2>
            <AccountRow label="账户" value={props.user.user_id} />
            <AccountRow
              label="验证渠道"
              value={
                props.user.registration_policy === "whop_international"
                  ? `Whop 购买邮箱${props.user.email_hint ? ` · ${props.user.email_hint}` : ""}`
                  : "国内手机号"
              }
            />
            <AccountRow label="注册时间" value={formatDate(props.user.created_at)} />
            <AccountRow label="最近登录" value={formatDate(props.user.last_login_at)} />
            <AccountRow
              label="访问权限"
              value={
                publicUserHasProductAccess(props.user)
                  ? props.user.daily_limit > 0
                    ? `已启用 · 每日 ${props.user.daily_limit} 次`
                    : "已启用"
                  : "已暂停"
              }
            />
          </section>
          <section>
            <div class="public-account-actions">
              <Show
                when={publicUserHasProductAccess(props.user)}
                fallback={
                  <button type="button" class="is-primary" onClick={() => navigate("/plan")}>
                    查看会员与续费
                  </button>
                }
              >
                <button type="button" class="is-primary" onClick={() => navigate("/chat")}>进入 Agent</button>
                <button type="button" onClick={() => navigate("/community")}>去社区看看</button>
              </Show>
              <button type="button" class="is-danger" onClick={props.onLogout}>退出登录</button>
            </div>
            <p class="public-account-note">账户页不展示内部已读状态、运行配置或系统权限。需要修改持仓、提醒和研究偏好时，直接在 Agent 对话中说明即可。</p>
          </section>
        </div>
        {/* 登录后工作台内不再直达营销站，官网/协议等入口收敛在这里。 */}
        <section class="public-workspace-panel public-account-card public-account-links">
          <h2>关于与帮助</h2>
          <nav aria-label="关于与帮助">
            <a href="/">官网首页</a>
            <a href="/plan">会员与定价</a>
            <a href="/blog">Blog</a>
            <a href="/roadmap">路线图与文档</a>
            <a href="/terms">用户协议</a>
            <a href="/privacy">隐私政策</a>
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

  const load = async () => {
    setLoading(true);
    try {
      setUser(await getPublicAuthMe());
    } catch {
      setUser(null);
    } finally {
      setLoading(false);
    }
  };

  onMount(() => void load());

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
      <Show when={user()} fallback={<PublicLoginForm onLogin={() => void load()} />}>
        {(currentUser) => <AccountView user={currentUser()} onLogout={logout} />}
      </Show>
    </Show>
  );
}
