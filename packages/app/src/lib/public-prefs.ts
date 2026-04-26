// User preferences for the public surface — chat font scale + theme.
// Persisted in localStorage and applied as data-* attributes on <html> so
// CSS can target them with cascading rules.

import { createSignal } from "solid-js";

export type PublicTheme = "auto" | "light" | "dark";
export type PublicFontScale = "s" | "m" | "l" | "xl";

const THEME_KEY = "hone.public.theme";
const FS_KEY = "hone.public.fs";

function readTheme(): PublicTheme {
  try {
    const v = localStorage.getItem(THEME_KEY);
    if (v === "light" || v === "dark" || v === "auto") return v;
  } catch {}
  return "auto";
}

function readFontScale(): PublicFontScale {
  try {
    const v = localStorage.getItem(FS_KEY);
    if (v === "s" || v === "m" || v === "l" || v === "xl") return v;
  } catch {}
  return "m";
}

const [themeSig, setThemeSig] = createSignal<PublicTheme>(readTheme());
const [fontScaleSig, setFontScaleSig] = createSignal<PublicFontScale>(readFontScale());

export function publicTheme() { return themeSig(); }
export function publicFontScale() { return fontScaleSig(); }

function resolveTheme(pref: PublicTheme): "light" | "dark" {
  if (pref !== "auto") return pref;
  try {
    if (window.matchMedia?.("(prefers-color-scheme: dark)").matches) return "dark";
  } catch {}
  return "light";
}

function applyToDom() {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  const pref = themeSig();
  root.setAttribute("data-theme-pref", pref);
  root.setAttribute("data-theme", resolveTheme(pref));
  root.setAttribute("data-chat-fs", fontScaleSig());
}

export function setPublicTheme(value: PublicTheme) {
  setThemeSig(value);
  try { localStorage.setItem(THEME_KEY, value); } catch {}
  applyToDom();
}

export function setPublicFontScale(value: PublicFontScale) {
  setFontScaleSig(value);
  try { localStorage.setItem(FS_KEY, value); } catch {}
  applyToDom();
}

let mqlInited = false;
function initSystemThemeListener() {
  if (mqlInited || typeof window === "undefined") return;
  mqlInited = true;
  try {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => { if (themeSig() === "auto") applyToDom(); };
    if (mql.addEventListener) mql.addEventListener("change", handler);
    else if ((mql as any).addListener) (mql as any).addListener(handler);
  } catch {}
}

export function initPublicPrefs() {
  initSystemThemeListener();
  applyToDom();
}
