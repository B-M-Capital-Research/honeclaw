// 「我的 · 设置」：投资画像风格。系统会从公司画像里蒸馏一份，用户可以
// 自己改写；改写后以用户版本为准，后台蒸馏不会覆盖。

import { createEffect, createSignal, Show } from "solid-js";
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
      setError(cause instanceof Error ? cause.message : "设置加载失败");
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
      setError(cause instanceof Error ? cause.message : "保存失败，请稍后再试");
    } finally {
      setSaving(false);
    }
  };

  return (
    <section class="public-workspace-panel public-account-card public-settings-panel" aria-label="我的设置">
      <h2>投资画像风格</h2>
      <p class="public-settings-desc">
        HONE 会参考这段描述调整推送与研究结论的侧重点——比如你更看重长期产业叙事、还是短期催化。
        留空则使用系统从你的公司画像里蒸馏出来的版本。
      </p>

      <Show when={!loading()} fallback={<div class="public-workspace-state">正在加载设置…</div>}>
        <textarea
          class="public-settings-input"
          rows="5"
          value={draft()}
          placeholder="例如：长期叙事派，重视行业结构性变化与公司壁垒，不看重短期技术形态与分析师评级。"
          onInput={(event) => setDraft(event.currentTarget.value)}
        />
        <div class="public-settings-actions">
          <button
            type="button"
            class="is-primary"
            disabled={saving() || draft() === (settings()?.style ?? "")}
            onClick={() => void save(draft())}
          >
            {saving() ? "保存中…" : "保存"}
          </button>
          <Show when={settings()?.user_edited}>
            <button type="button" disabled={saving()} onClick={() => void save("")}>
              恢复系统版本
            </button>
          </Show>
          <Show when={saved()}>
            <span class="public-settings-saved" role="status">已保存</span>
          </Show>
        </div>

        <Show when={error()}>
          <p class="public-holding-form-error" role="alert">{error()}</p>
        </Show>

        <Show when={settings()?.user_edited && settings()?.distilled_style}>
          {(distilled) => (
            <div class="public-settings-distilled">
              <strong>系统蒸馏版本</strong>
              <p>{distilled()}</p>
            </div>
          )}
        </Show>
      </Show>
    </section>
  );
}
