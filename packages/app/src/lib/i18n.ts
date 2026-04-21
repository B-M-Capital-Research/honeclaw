// i18n.ts — Locale state for the Hone Public Site
//
// The public site surface (hone-claw.com) supports Simplified Chinese ("zh")
// and English ("en"). Locale is persisted to localStorage; the first visit
// auto-detects from navigator.language.

import { createSignal } from "solid-js"

export type Locale = "zh" | "en"

const STORAGE_KEY = "hone-public-locale"

function detectInitialLocale(): Locale {
  if (typeof window === "undefined") return "zh"
  try {
    const saved = window.localStorage.getItem(STORAGE_KEY)
    if (saved === "zh" || saved === "en") return saved
  } catch {
    // ignore storage errors (private mode, quota, etc.)
  }
  const nav = window.navigator?.language ?? ""
  return nav.toLowerCase().startsWith("zh") ? "zh" : "en"
}

const [locale, setLocaleSignal] = createSignal<Locale>(detectInitialLocale())

export function useLocale(): Locale {
  return locale()
}

export function setLocale(next: Locale): void {
  setLocaleSignal(next)
  if (typeof window !== "undefined") {
    try {
      window.localStorage.setItem(STORAGE_KEY, next)
    } catch {
      // ignore
    }
    try {
      document.documentElement.setAttribute("lang", next === "zh" ? "zh-CN" : "en")
    } catch {
      // ignore
    }
  }
}

export function toggleLocale(): void {
  setLocale(locale() === "zh" ? "en" : "zh")
}
