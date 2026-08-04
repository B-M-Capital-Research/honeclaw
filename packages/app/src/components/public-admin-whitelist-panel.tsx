import { For, Show, createSignal, onMount } from "solid-js";
import { CONTENT } from "@/lib/public-content";
import { useLocale } from "@/lib/i18n";
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
  if (!value) return CONTENT.chat_page.admin.w_not_signed_in;
  return new Date(value).toLocaleDateString(useLocale() === "en" ? "en-US" : "zh-CN", {
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
      setError(cause instanceof Error ? cause.message : CONTENT.chat_page.admin.w_read_failed);
    } finally {
      setLoading(false);
    }
  };

  onMount(() => void load());

  const createInvite = async (event: SubmitEvent) => {
    event.preventDefault();
    const normalized = normalizeDomesticAdminPhone(phone());
    if (!/^1\d{10}$/.test(normalized)) {
      setError(CONTENT.chat_page.admin.w_bad_phone);
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
      setError(cause instanceof Error ? cause.message : CONTENT.chat_page.admin.w_add_failed);
      await load();
    } finally {
      setPending(false);
    }
  };

  const disableInvite = async (invite: PublicAdminInviteInfo) => {
    if (
      !invite.can_disable ||
      pending() ||
      !window.confirm(
        `${CONTENT.chat_page.admin.w_confirm.replace("{phone}", invite.phone_number)}${CONTENT.chat_page.admin.w_confirm_note}`,
      )
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
      setError(cause instanceof Error ? cause.message : CONTENT.chat_page.admin.w_disable_failed);
    } finally {
      setPending(false);
    }
  };

  return (
    <details class="public-workspace-panel public-admin-panel" aria-labelledby="public-admin-title">
      <summary class="public-admin-section-summary">
        <span class="public-admin-section-copy">
          <span class="public-workspace-eyebrow">{CONTENT.chat_page.admin.w_admin}</span>
          <h2 id="public-admin-title">{CONTENT.chat_page.admin.w_title}</h2>
          <p>{CONTENT.chat_page.admin.w_subtitle}</p>
        </span>
        <Show when={list()}>
          {(current) => (
            <span class="public-admin-limit">
              {CONTENT.chat_page.admin.w_remaining} <strong>{current().remaining_today}</strong> / {current().daily_create_limit}
            </span>
          )}
        </Show>
        <span class="public-admin-section-toggle-label" aria-hidden="true">
          <span class="when-open">{CONTENT.chat_page.admin.u_collapse}</span>
          <span class="when-closed">{CONTENT.chat_page.admin.u_expand}</span>
          <span class="public-admin-section-chevron" />
        </span>
      </summary>

      <div class="public-admin-section-body">
        <form class="public-admin-create" onSubmit={createInvite}>
          <label for="public-admin-phone">{CONTENT.chat_page.admin.w_add_label}</label>
          <div>
            <input
              id="public-admin-phone"
              type="tel"
              inputmode="numeric"
              autocomplete="tel"
              maxlength="18"
              placeholder={CONTENT.chat_page.admin.w_phone_ph}
              value={phone()}
              onInput={(event) => setPhone(event.currentTarget.value)}
              disabled={pending()}
            />
            <button
              type="submit"
              disabled={!publicAdminCanCreate(list(), pending())}
            >
              {pending() ? CONTENT.chat_page.admin.w_processing : CONTENT.chat_page.admin.w_add}
            </button>
          </div>
          <small>{CONTENT.chat_page.admin.w_quota_note}</small>
        </form>

        <Show when={error()}>
          <p class="public-admin-feedback is-error" role="alert">{error()}</p>
        </Show>
        <Show when={notice()}>
          <p class="public-admin-feedback is-success" role="status">{notice()}</p>
        </Show>

        <Show
          when={!loading()}
          fallback={<div class="public-admin-loading">{CONTENT.chat_page.admin.w_loading}</div>}
        >
          <Show
            when={(list()?.invites.length ?? 0) > 0}
            fallback={<div class="public-admin-empty">{CONTENT.chat_page.admin.w_empty}</div>}
          >
            <div class="public-admin-table-wrap">
              <table class="public-admin-table">
                <thead>
                  <tr>
                    <th>{CONTENT.chat_page.admin.w_phone}</th>
                    <th>{CONTENT.chat_page.admin.w_joined}</th>
                    <th>{CONTENT.chat_page.admin.w_last_login}</th>
                    <th>{CONTENT.chat_page.admin.w_status}</th>
                    <th><span class="sr-only">{CONTENT.chat_page.admin.w_actions}</span></th>
                  </tr>
                </thead>
                <tbody>
                  <For each={list()?.invites ?? []}>
                    {(invite) => (
                      <tr>
                        <td data-label={CONTENT.chat_page.admin.w_phone}><strong>{invite.phone_number}</strong></td>
                        <td data-label={CONTENT.chat_page.admin.w_joined}>{formatAdminDate(invite.created_at)}</td>
                        <td data-label={CONTENT.chat_page.admin.w_last_login}>{formatAdminDate(invite.last_login_at)}</td>
                        <td data-label={CONTENT.chat_page.admin.w_status}>
                          <span classList={{
                            "public-admin-status": true,
                            "is-enabled": invite.enabled,
                          }}>
                            {invite.enabled ? CONTENT.chat_page.admin.w_enabled : CONTENT.chat_page.admin.w_disabled}
                          </span>
                        </td>
                        <td class="public-admin-row-action">
                          <Show
                            when={invite.can_disable}
                            fallback={<span>{invite.enabled ? CONTENT.chat_page.admin.w_current_admin : "—"}</span>}
                          >
                            <button
                              type="button"
                              disabled={pending()}
                              onClick={() => void disableInvite(invite)}
                            >
                              {CONTENT.chat_page.admin.w_disable}
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
      </div>
    </details>
  );
}
