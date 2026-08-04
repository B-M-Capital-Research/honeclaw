// 「我的 · 设置」：投资画像风格。系统会从公司画像里蒸馏一份，用户可以
// 自己改写；改写后以用户版本为准，后台蒸馏不会覆盖。

import { createEffect, createSignal, Show } from "solid-js";
import { CONTENT } from "@/lib/public-content";
import { getPublicSettings, putPublicInvestorStyle, type PublicSettings } from "@/lib/api";

export function PublicSettingsPanel() {
  const [settings, setSettings] = createSignal<PublicSettings>();
  const [draft, setDraft] = createSignal("");
  const [loading, setLoading] = createSignal(true);
  const [saving, setSaving] = createSignal(false);
  const [error, setError] = createSignal<string>();
  const [saved, setSaved] = createSignal(false);

  const load = async () => {
    setLoading(true);
    setError(undefined);
    try {
      const payload = await getPublicSettings();
      setSettings(payload);
      setDraft(payload.style ?? "");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : CONTENT.chat_page.settings_panel.load_failed);
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    void load();
  });

  const save = async (value: string) => {
    setSaving(true);
    setError(undefined);
    setSaved(false);
    try {
      const payload = await putPublicInvestorStyle(value);
      setSettings(payload);
      setDraft(payload.style ?? "");
      setSaved(true);
      window.setTimeout(() => setSaved(false), 2400);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : CONTENT.chat_page.settings_panel.save_failed);
    } finally {
      setSaving(false);
    }
  };

  return (
    <section class="public-workspace-panel public-account-card public-settings-panel" aria-label={CONTENT.chat_page.settings_panel.title}>
      <h2>{CONTENT.chat_page.settings_panel.style_label}</h2>
      <p class="public-settings-desc">
        {CONTENT.chat_page.settings_panel.style_hint}
        {CONTENT.chat_page.settings_panel.style_blank}
      </p>

      <Show when={!loading()} fallback={<div class="public-workspace-state">{CONTENT.chat_page.settings_panel.loading}</div>}>
        <textarea
          class="public-settings-input"
          rows="5"
          value={draft()}
          placeholder={CONTENT.chat_page.settings_panel.style_eg}
          onInput={(event) => setDraft(event.currentTarget.value)}
        />
        <div class="public-settings-actions">
          <button
            type="button"
            class="is-primary"
            disabled={saving() || draft() === (settings()?.style ?? "")}
            onClick={() => void save(draft())}
          >
            {saving() ? CONTENT.chat_page.settings_panel.saving : CONTENT.chat_page.settings_panel.save}
          </button>
          <Show when={settings()?.user_edited}>
            <button type="button" disabled={saving()} onClick={() => void save("")}>
              {CONTENT.chat_page.settings_panel.restore}
            </button>
          </Show>
          <Show when={saved()}>
            <span class="public-settings-saved" role="status">{CONTENT.chat_page.settings_panel.saved}</span>
          </Show>
        </div>

        <Show when={error()}>
          <p class="public-holding-form-error" role="alert">{error()}</p>
        </Show>

        <Show when={settings()?.user_edited && settings()?.distilled_style}>
          {(distilled) => (
            <div class="public-settings-distilled">
              <strong>{CONTENT.chat_page.settings_panel.system_version}</strong>
              <p>{distilled()}</p>
            </div>
          )}
        </Show>
      </Show>
    </section>
  );
}
