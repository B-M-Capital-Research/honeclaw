import { For, Show, createSignal, onMount } from "solid-js";
import { CONTENT } from "@/lib/public-content";
import { useLocale } from "@/lib/i18n";
import {
  createPublicAdminInvite,
  disablePublicAdminInvite,
  getPublicAdminInvites,
  setPublicAdminInviteEmail,
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

export function normalizeAdminEmail(value: string) {
  return value.trim().toLowerCase();
}

export function adminEmailLooksValid(value: string) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
}

/**
 * Validates the create form: phone and email are each optional, but at least
 * one is required, and whatever is filled in must look right.
 */
export function validateAdminInviteInput(phone: string, email: string):
  | { ok: true; phone_number?: string; email_address?: string }
  | { ok: false; reason: "need_one" | "bad_phone" | "bad_email" } {
  const normalizedPhone = normalizeDomesticAdminPhone(phone);
  const normalizedEmail = normalizeAdminEmail(email);
  if (!normalizedPhone && !normalizedEmail) return { ok: false, reason: "need_one" };
  if (normalizedPhone && !/^1\d{10}$/.test(normalizedPhone)) {
    return { ok: false, reason: "bad_phone" };
  }
  if (normalizedEmail && !adminEmailLooksValid(normalizedEmail)) {
    return { ok: false, reason: "bad_email" };
  }
  return {
    ok: true,
    phone_number: normalizedPhone || undefined,
    email_address: normalizedEmail || undefined,
  };
}

/** Label used in confirmations: phone when present, otherwise the email. */
export function adminInviteLabel(invite: Pick<PublicAdminInviteInfo, "phone_number" | "email_address">) {
  return invite.phone_number || invite.email_address || "—";
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
  const [email, setEmail] = createSignal("");
  const [editingUserId, setEditingUserId] = createSignal<string | null>(null);
  const [editingEmail, setEditingEmail] = createSignal("");
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
    const input = validateAdminInviteInput(phone(), email());
    if (!input.ok) {
      setError(
        input.reason === "need_one"
          ? CONTENT.chat_page.admin.w_need_one
          : input.reason === "bad_phone"
            ? CONTENT.chat_page.admin.w_bad_phone
            : CONTENT.chat_page.admin.w_bad_email,
      );
      return;
    }
    if (!publicAdminCanCreate(list(), pending())) return;
    setPending(true);
    setError("");
    setNotice("");
    try {
      const result = await createPublicAdminInvite({
        phone_number: input.phone_number,
        email_address: input.email_address,
      });
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
      setEmail("");
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
        `${CONTENT.chat_page.admin.w_confirm.replace("{phone}", adminInviteLabel(invite))}${CONTENT.chat_page.admin.w_confirm_note}`,
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

  const startEmailEdit = (invite: PublicAdminInviteInfo) => {
    setEditingUserId(invite.user_id);
    setEditingEmail(invite.email_address ?? "");
    setError("");
    setNotice("");
  };

  const saveEmail = async (invite: PublicAdminInviteInfo) => {
    if (pending()) return;
    const next = normalizeAdminEmail(editingEmail());
    if (next && !adminEmailLooksValid(next)) {
      setError(CONTENT.chat_page.admin.w_bad_email);
      return;
    }
    setPending(true);
    setError("");
    setNotice("");
    try {
      const result = await setPublicAdminInviteEmail(invite.user_id, next || null);
      setList((current) =>
        current
          ? { ...current, invites: replaceInvite(current.invites, result.invite) }
          : current,
      );
      setEditingUserId(null);
      setNotice(result.message);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : CONTENT.chat_page.admin.w_email_failed);
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
          <div class="public-admin-create-fields">
            <input
              id="public-admin-phone"
              type="tel"
              inputmode="numeric"
              autocomplete="off"
              maxlength="18"
              aria-label={CONTENT.chat_page.admin.w_phone}
              placeholder={CONTENT.chat_page.admin.w_phone_ph}
              value={phone()}
              onInput={(event) => setPhone(event.currentTarget.value)}
              disabled={pending()}
            />
            <input
              id="public-admin-email"
              type="email"
              inputmode="email"
              autocomplete="off"
              maxlength="254"
              aria-label={CONTENT.chat_page.admin.w_email}
              placeholder={CONTENT.chat_page.admin.w_email_ph}
              value={email()}
              onInput={(event) => setEmail(event.currentTarget.value)}
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
                    <th>{CONTENT.chat_page.admin.w_email}</th>
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
                        <td data-label={CONTENT.chat_page.admin.w_phone}><strong>{invite.phone_number || "—"}</strong></td>
                        <td data-label={CONTENT.chat_page.admin.w_email} class="public-admin-email-cell">
                          <Show
                            when={editingUserId() === invite.user_id}
                            fallback={
                              <span class="public-admin-email-view">
                                <span classList={{ "is-empty": !invite.email_address }}>
                                  {invite.email_address || CONTENT.chat_page.admin.w_no_email}
                                </span>
                                <Show when={invite.enabled}>
                                  <button
                                    type="button"
                                    class="public-admin-link-button"
                                    disabled={pending()}
                                    onClick={() => startEmailEdit(invite)}
                                  >
                                    {CONTENT.chat_page.admin.w_edit_email}
                                  </button>
                                </Show>
                              </span>
                            }
                          >
                            <form
                              class="public-admin-email-edit"
                              onSubmit={(event) => {
                                event.preventDefault();
                                void saveEmail(invite);
                              }}
                            >
                              <input
                                type="email"
                                inputmode="email"
                                autocomplete="off"
                                maxlength="254"
                                aria-label={CONTENT.chat_page.admin.w_email}
                                placeholder={CONTENT.chat_page.admin.w_email_edit_ph}
                                value={editingEmail()}
                                onInput={(event) => setEditingEmail(event.currentTarget.value)}
                                disabled={pending()}
                              />
                              <button type="submit" disabled={pending()}>
                                {CONTENT.chat_page.admin.w_save_email}
                              </button>
                              <button
                                type="button"
                                class="public-admin-link-button"
                                disabled={pending()}
                                onClick={() => setEditingUserId(null)}
                              >
                                {CONTENT.chat_page.admin.w_cancel}
                              </button>
                            </form>
                          </Show>
                        </td>
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
