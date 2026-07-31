import { For, Show, createSignal, onMount } from "solid-js";
import {
  createPublicAdminInvite,
  disablePublicAdminInvite,
  getPublicAdminInvites,
} from "@/lib/api";
import type {
  PublicAdminInviteInfo,
  PublicAdminInviteList,
} from "@/lib/types";

function formatAdminDate(value?: string) {
  if (!value) return "尚未登录";
  return new Date(value).toLocaleDateString("zh-CN", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export function normalizeDomesticAdminPhone(value: string) {
  const digits = value.replace(/\D/g, "");
  return digits.startsWith("86") && digits.length === 13
    ? digits.slice(2)
    : digits;
}

export function publicAdminCanCreate(
  list: Pick<PublicAdminInviteList, "remaining_today"> | null,
  pending: boolean,
) {
  return Boolean(list && list.remaining_today > 0 && !pending);
}

function replaceInvite(
  invites: PublicAdminInviteInfo[],
  next: PublicAdminInviteInfo,
) {
  const index = invites.findIndex((invite) => invite.user_id === next.user_id);
  if (index < 0) return [next, ...invites];
  return invites.map((invite) =>
    invite.user_id === next.user_id ? next : invite,
  );
}

export function PublicAdminWhitelistPanel() {
  const [list, setList] = createSignal<PublicAdminInviteList | null>(null);
  const [phone, setPhone] = createSignal("");
  const [loading, setLoading] = createSignal(true);
  const [pending, setPending] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const load = async () => {
    setLoading(true);
    setError("");
    try {
      setList(await getPublicAdminInvites());
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "读取会员白名单失败");
    } finally {
      setLoading(false);
    }
  };

  onMount(() => void load());

  const createInvite = async (event: SubmitEvent) => {
    event.preventDefault();
    const normalized = normalizeDomesticAdminPhone(phone());
    if (!/^1\d{10}$/.test(normalized)) {
      setError("请输入正确的中国大陆手机号");
      return;
    }
    if (!publicAdminCanCreate(list(), pending())) return;
    setPending(true);
    setError("");
    setNotice("");
    try {
      const result = await createPublicAdminInvite(normalized);
      setList((current) =>
        current
          ? {
              ...current,
              invites: replaceInvite(current.invites, result.invite),
              daily_create_limit: result.daily_create_limit,
              created_today: result.created_today,
              remaining_today: result.remaining_today,
            }
          : current,
      );
      setPhone("");
      setNotice(result.message);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "新增会员白名单失败");
      await load();
    } finally {
      setPending(false);
    }
  };

  const disableInvite = async (invite: PublicAdminInviteInfo) => {
    if (
      !invite.can_disable ||
      pending() ||
      !window.confirm(`确认禁用会员 ${invite.phone_number}？该用户会立即退出登录。`)
    ) {
      return;
    }
    setPending(true);
    setError("");
    setNotice("");
    try {
      const result = await disablePublicAdminInvite(invite.user_id);
      setList((current) =>
        current
          ? {
              ...current,
              invites: replaceInvite(current.invites, result.invite),
              daily_create_limit: result.daily_create_limit,
              created_today: result.created_today,
              remaining_today: result.remaining_today,
            }
          : current,
      );
      setNotice(result.message);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "禁用会员白名单失败");
    } finally {
      setPending(false);
    }
  };

  return (
    <section class="public-workspace-panel public-admin-panel" aria-labelledby="public-admin-title">
      <header class="public-admin-head">
        <div>
          <span class="public-workspace-eyebrow">管理员</span>
          <h2 id="public-admin-title">管理</h2>
          <p>管理国内手机号会员白名单。禁用后，该用户现有登录态会立即失效。</p>
        </div>
        <Show when={list()}>
          {(current) => (
            <span class="public-admin-limit">
              今日还可新增 <strong>{current().remaining_today}</strong> / {current().daily_create_limit}
            </span>
          )}
        </Show>
      </header>

      <form class="public-admin-create" onSubmit={createInvite}>
        <label for="public-admin-phone">新增白名单手机号</label>
        <div>
          <input
            id="public-admin-phone"
            type="tel"
            inputmode="numeric"
            autocomplete="tel"
            maxlength="18"
            placeholder="请输入 11 位手机号"
            value={phone()}
            onInput={(event) => setPhone(event.currentTarget.value)}
            disabled={pending()}
          />
          <button
            type="submit"
            disabled={!publicAdminCanCreate(list(), pending())}
          >
            {pending() ? "处理中…" : "加入白名单"}
          </button>
        </div>
        <small>为防止误操作，每位管理员按北京时间每天最多成功新增 5 人。</small>
      </form>

      <Show when={error()}>
        <p class="public-admin-feedback is-error" role="alert">{error()}</p>
      </Show>
      <Show when={notice()}>
        <p class="public-admin-feedback is-success" role="status">{notice()}</p>
      </Show>

      <Show
        when={!loading()}
        fallback={<div class="public-admin-loading">正在读取会员白名单…</div>}
      >
        <Show
          when={(list()?.invites.length ?? 0) > 0}
          fallback={<div class="public-admin-empty">当前还没有国内手机号会员。</div>}
        >
          <div class="public-admin-table-wrap">
            <table class="public-admin-table">
              <thead>
                <tr>
                  <th>手机号</th>
                  <th>加入时间</th>
                  <th>最近登录</th>
                  <th>状态</th>
                  <th><span class="sr-only">操作</span></th>
                </tr>
              </thead>
              <tbody>
                <For each={list()?.invites ?? []}>
                  {(invite) => (
                    <tr>
                      <td data-label="手机号"><strong>{invite.phone_number}</strong></td>
                      <td data-label="加入时间">{formatAdminDate(invite.created_at)}</td>
                      <td data-label="最近登录">{formatAdminDate(invite.last_login_at)}</td>
                      <td data-label="状态">
                        <span classList={{
                          "public-admin-status": true,
                          "is-enabled": invite.enabled,
                        }}>
                          {invite.enabled ? "已启用" : "已禁用"}
                        </span>
                      </td>
                      <td class="public-admin-row-action">
                        <Show
                          when={invite.can_disable}
                          fallback={<span>{invite.enabled ? "当前管理员" : "—"}</span>}
                        >
                          <button
                            type="button"
                            disabled={pending()}
                            onClick={() => void disableInvite(invite)}
                          >
                            禁用
                          </button>
                        </Show>
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
      </Show>
    </section>
  );
}
