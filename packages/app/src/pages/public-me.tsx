import { createSignal, onMount, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { PublicChatStartup } from "@/components/public-chat-startup";
import { PublicLoginForm } from "@/components/public-login-form";
import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import { getPublicAuthMe, publicLogout } from "@/lib/api";
import { workspaceUserName } from "@/lib/public-agent-workspace";
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
            <p>管理你的 HONE 账户与研究入口。持仓、洞察和会话数据仍由各自的安全存储维护。</p>
          </div>
        </header>
        <div class="public-account-grid">
          <section class="public-workspace-panel public-account-card">
            <h2>账户信息</h2>
            <AccountRow label="账户" value={props.user.user_id} />
            <AccountRow label="注册时间" value={formatDate(props.user.created_at)} />
            <AccountRow label="最近登录" value={formatDate(props.user.last_login_at)} />
            <AccountRow label="访问权限" value={props.user.daily_limit > 0 ? `每日 ${props.user.daily_limit} 次` : "已启用"} />
          </section>
          <section>
            <div class="public-account-actions">
              <button type="button" class="is-primary" onClick={() => navigate("/chat")}>进入 Agent</button>
              <button type="button" onClick={() => navigate("/portfolio")}>查看跟踪与财经日历</button>
              <button type="button" onClick={() => navigate("/community")}>查看洞察</button>
              <button type="button" class="is-danger" onClick={props.onLogout}>退出登录</button>
            </div>
            <p class="public-account-note">账户页不展示内部已读状态、运行配置或系统权限。需要修改持仓、提醒和研究偏好时，直接在 Agent 对话中说明即可。</p>
          </section>
        </div>
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
