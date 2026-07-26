import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { CONTENT } from "@/lib/public-content";
import {
  initPublicPrefs,
  publicFontScale,
  publicTheme,
  setPublicFontScale,
  setPublicTheme,
  type PublicTheme,
} from "@/lib/public-prefs";
import "./public-prefs-button.css";

export function PublicPrefsButton() {
  const [open, setOpen] = createSignal(false);
  const themeOptions = createMemo<{ value: PublicTheme; label: string }[]>(() => [
    { value: "auto", label: CONTENT.chat_page.prefs.theme_auto },
    { value: "light", label: CONTENT.chat_page.prefs.theme_light },
    { value: "dark", label: CONTENT.chat_page.prefs.theme_dark },
  ]);
  const close = () => setOpen(false);
  let rootRef: HTMLDivElement | undefined;

  onMount(initPublicPrefs);

  createEffect(() => {
    if (!open()) return;
    const onPointer = (event: PointerEvent) => {
      if (rootRef && !rootRef.contains(event.target as Node)) close();
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") close();
    };
    document.addEventListener("pointerdown", onPointer, true);
    document.addEventListener("keydown", onKey);
    onCleanup(() => {
      document.removeEventListener("pointerdown", onPointer, true);
      document.removeEventListener("keydown", onKey);
    });
  });

  return (
    <div class="hone-prefs" ref={rootRef}>
      <button
        type="button"
        class="hone-prefs-trigger"
        aria-label={CONTENT.chat_page.prefs.aria_label}
        aria-expanded={open()}
        aria-haspopup="dialog"
        onClick={() => setOpen((value) => !value)}
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M4 19l5.5-13 5.5 13M6.5 14h6M16 19h4M16 13h4M16 7h4" />
        </svg>
      </button>
      <Show when={open()}>
        <div class="hone-prefs-panel" role="dialog" aria-label={CONTENT.chat_page.prefs.aria_label}>
          <div class="hone-prefs-row">
            <span class="hone-prefs-label">{CONTENT.chat_page.prefs.font_size}</span>
            <div class="hone-prefs-segmented">
              <For each={["s", "m", "l", "xl"] as const}>
                {(size) => (
                  <button
                    type="button"
                    class="hone-prefs-seg"
                    classList={{ "is-active": publicFontScale() === size }}
                    data-size={size}
                    aria-pressed={publicFontScale() === size}
                    aria-label={`${CONTENT.chat_page.prefs.font_size} ${size.toUpperCase()}`}
                    onClick={() => setPublicFontScale(size)}
                  >
                    A
                  </button>
                )}
              </For>
            </div>
          </div>
          <div class="hone-prefs-row">
            <span class="hone-prefs-label">{CONTENT.chat_page.prefs.theme}</span>
            <div class="hone-prefs-segmented">
              <For each={themeOptions()}>
                {(option) => (
                  <button
                    type="button"
                    class="hone-prefs-seg hone-prefs-seg--text"
                    classList={{ "is-active": publicTheme() === option.value }}
                    aria-pressed={publicTheme() === option.value}
                    onClick={() => setPublicTheme(option.value)}
                  >
                    {option.label}
                  </button>
                )}
              </For>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
