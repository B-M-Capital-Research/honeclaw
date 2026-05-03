// i18n.ts — Locale state and bilingual content helpers (Hone)
//
// The public site (hone-claw.com) and admin console both run as the same
// Solid SPA. Locale state lives here as a single signal: read via `useLocale()`
// inside any reactive scope, mutated via `setLocale()` / `toggleLocale()`.
// Persisted to localStorage so a per-device override survives reloads. First
// visit on the public surface falls back to navigator.language; the admin
// surface bootstraps from the backend `/api/meta` `language` field
// (see context/backend.tsx) when no localStorage override exists.
//
// Bilingual content is modeled as parallel ZH / EN object trees; consumers
// build a Proxy via `makeContentProxy(zh, en)` and read fields like normal
// object properties. The proxy reads `useLocale()` on every access so any
// reactive scope re-evaluates when the locale signal flips.

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

/** True iff the user has an explicit per-device locale override stored. */
export function hasLocaleOverride(): boolean {
  if (typeof window === "undefined") return false
  try {
    const saved = window.localStorage.getItem(STORAGE_KEY)
    return saved === "zh" || saved === "en"
  } catch {
    return false
  }
}

/**
 * Build a deep Proxy that returns a value from the ZH or EN tree based on the
 * current locale signal. Consumers read fields like `T.foo.bar`; the proxy
 * looks up the path in the active tree on every access, so JSX expressions
 * `{T.foo.bar}` and `<For each={T.items}>` re-render automatically when the
 * locale flips.
 *
 * Both trees MUST share the same shape — drift will surface as undefined
 * reads. The structure.test.ts harness in lib/admin-content/ enforces parity.
 */
export function makeContentProxy<T extends object>(zh: T, en: T): T {
  const sources: Record<Locale, T> = { zh, en }
  const resolveAt = (path: readonly (string | symbol)[]): unknown => {
    let v: unknown = sources[useLocale()]
    for (const seg of path) {
      if (v == null) return undefined
      v = (v as Record<string | symbol, unknown>)[seg as string]
    }
    return v
  }
  const build = (path: readonly (string | symbol)[]): T => {
    return new Proxy(Object.create(null), {
      get(_target, key) {
        if (typeof key === "symbol") {
          const resolved = resolveAt(path)
          return resolved == null ? undefined : (resolved as Record<symbol, unknown>)[key]
        }
        const next = resolveAt([...path, key])
        if (next === null || next === undefined) return next
        if (typeof next !== "object") return next
        if (Array.isArray(next)) return next
        return build([...path, key])
      },
      has(_target, key) {
        const v = resolveAt(path)
        return v != null && typeof v === "object" && key in (v as object)
      },
      ownKeys() {
        const v = resolveAt(path)
        if (v == null || typeof v !== "object") return []
        return Reflect.ownKeys(v as object)
      },
      getOwnPropertyDescriptor(_target, key) {
        const v = resolveAt(path)
        if (v == null || typeof v !== "object") return undefined
        if (!(key in (v as object))) return undefined
        return {
          enumerable: true,
          configurable: true,
          writable: false,
          value: (v as Record<string | symbol, unknown>)[key as string],
        }
      },
    }) as T
  }
  return build([])
}

/**
 * Substitute `{name}` placeholders in a template string with values from
 * `vars`. Missing keys render as empty string. Used at callsites like:
 *   tpl(T.holdings_count, { count: 7 })
 */
export function tpl(s: string, vars: Record<string, string | number> = {}): string {
  return s.replace(/\{(\w+)\}/g, (_, k) => {
    const v = vars[k]
    return v === undefined || v === null ? "" : String(v)
  })
}

const PLURAL_RULES_CACHE = new Map<Locale, Intl.PluralRules>()
function pluralRulesFor(loc: Locale): Intl.PluralRules {
  let rules = PLURAL_RULES_CACHE.get(loc)
  if (!rules) {
    rules = new Intl.PluralRules(loc === "zh" ? "zh-CN" : "en-US")
    PLURAL_RULES_CACHE.set(loc, rules)
  }
  return rules
}

/**
 * Pick from a `{ one, other }` tuple based on Intl.PluralRules in the current
 * locale, then run `tpl` to substitute `{count}` and any other vars. zh-CN
 * always returns "other" — pluralization only matters in English.
 */
export function plural(
  forms: { one: string; other: string },
  count: number,
  vars: Record<string, string | number> = {},
): string {
  const cat = pluralRulesFor(useLocale()).select(count)
  const tmpl = cat === "one" ? forms.one : forms.other
  return tpl(tmpl, { count, ...vars })
}

/** Format a Date in the current locale (zh-CN / en-US). */
export function formatDate(d: Date | string | number, opts?: Intl.DateTimeFormatOptions): string {
  const date = d instanceof Date ? d : new Date(d)
  if (Number.isNaN(date.getTime())) return ""
  const loc = useLocale() === "zh" ? "zh-CN" : "en-US"
  return new Intl.DateTimeFormat(loc, opts).format(date)
}

/** Format a number in the current locale. */
export function formatNumber(n: number, opts?: Intl.NumberFormatOptions): string {
  const loc = useLocale() === "zh" ? "zh-CN" : "en-US"
  return new Intl.NumberFormat(loc, opts).format(n)
}
