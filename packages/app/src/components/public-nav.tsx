// public-nav.tsx — Navigation + Footer for Hone Public Site

import {
  createResource,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
  type JSX,
} from "solid-js"
import { useNavigate, useLocation } from "@solidjs/router"
import { displayGithubStars, fetchGithubStars } from "@/lib/github-stars"
import { CONTENT } from "@/lib/public-content"
import { setLocale, useLocale } from "@/lib/i18n"
import { PublicContactCards, PublicContactMenu } from "@/components/public-contact-menu"
import "../pages/public-site.css"

export function PublicNav(
  props: { extraActions?: JSX.Element; mobileAction?: JSX.Element } = {},
) {
  const [scrolled, setScrolled] = createSignal(false)
  const [menuOpen, setMenuOpen] = createSignal(false)
  const [stars] = createResource(fetchGithubStars)
  const navigate = useNavigate()
  const location = useLocation()
  const C = CONTENT.nav

  const page = () => location.pathname

  onMount(() => {
    const onScroll = () => setScrolled(window.scrollY > 40)
    window.addEventListener("scroll", onScroll)
    onCleanup(() => window.removeEventListener("scroll", onScroll))
  })

  const transparent = () => false

  const navBg = () =>
    menuOpen()
      ? "#fff"
      : transparent()
      ? "transparent"
      : "rgba(255,255,255,0.96)"
  const navBorder = () => (transparent() ? "none" : "1px solid rgba(0,0,0,0.08)")
  const logoColor = () => (transparent() ? "rgba(255,255,255,0.75)" : "#64748b")

  const isActive = (path: string) => {
    if (path === "/") return page() === "/"
    return page() === path || page().startsWith(`${path}/`)
  }

  const getLinkColor = (path: string) => {
    if (isActive(path)) return "#f59e0b"
    return transparent() ? "rgba(255,255,255,0.75)" : "#475569"
  }
  const getLinkBg = (path: string) => {
    if (!isActive(path)) return "transparent"
    return transparent() ? "rgba(255,255,255,0.08)" : "rgba(245,158,11,0.08)"
  }
  // NOTE: store `labelKey` (not pre-resolved strings) so each render re-reads
  // the CONTENT proxy inside JSX and tracks the locale signal.
  const links = [
    { labelKey: "home", path: "/" },
    { labelKey: "roadmap", path: "/roadmap" },
    { labelKey: "blog", path: "/blog" },
    { labelKey: "me", path: "/me" },
  ] as const

  const go = (path: string) => {
    const current = page()
    navigate(path)
    if (current !== path) window.scrollTo({ top: 0, left: 0, behavior: "auto" })
    setMenuOpen(false)
  }

  return (
    <>
      <nav
        class="pub-nav"
        data-transparent={transparent() ? "true" : undefined}
        style={{
          position: "fixed",
          top: "0",
          left: "0",
          right: "0",
          "z-index": "200",
          height: "56px",
          display: "flex",
          "align-items": "center",
          "justify-content": "space-between",
          padding: "0 32px",
          background: navBg(),
          "border-bottom": navBorder(),
          "backdrop-filter": scrolled() && !menuOpen() ? "blur(16px)" : "none",
          "-webkit-backdrop-filter": scrolled() && !menuOpen() ? "blur(16px)" : "none",
          transition: "background 0.35s ease, border-color 0.35s ease",
        }}
      >
        {/* Logo */}
        <div
          onClick={() => go("/")}
          style={{ display: "flex", "align-items": "center", gap: "10px", cursor: "pointer" }}
        >
          <img src="/logo.svg" style={{ height: "26px" }} alt="Hone" />
          <span
            style={{
              "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
              "font-size": "10px",
              "font-weight": "600",
              "letter-spacing": "0.28em",
              "text-transform": "uppercase",
              color: logoColor(),
              transition: "color 0.35s",
            }}
          >
            {C.logo_tagline}
          </span>
        </div>

        {/* Desktop links */}
        <div class="pub-nav-links">
          <For each={links}>
            {(l) => (
              <button
                onClick={() => go(l.path)}
                class={`pub-nav-link${isActive(l.path) ? " is-active" : ""}`}
                style={{
                  background: getLinkBg(l.path),
                  color: getLinkColor(l.path),
                }}
              >
                {C[l.labelKey]}
              </button>
            )}
          </For>

          <button
            onClick={() => go("/chat")}
            class={`pub-nav-cta${isActive("/chat") ? " is-active" : ""}`}
          >
            {C.chat}
          </button>

          <Show when={props.extraActions}>
            <div class="pub-nav-extra-actions">{props.extraActions}</div>
          </Show>

          <PublicContactMenu />

          <a
            href={C.github_url}
            target="_blank"
            rel="noopener noreferrer"
            class="pub-github-star-link"
          >
            <span>GitHub</span>
            <span class="pub-github-star-count">{displayGithubStars(stars())}</span>
          </a>

          <div
            class="pub-nav-lang"
          >
            <For each={[{ code: "zh" as const, labelKey: "locale_zh" as const }, { code: "en" as const, labelKey: "locale_en" as const }]}>
              {(opt) => {
                const active = () => useLocale() === opt.code
                return (
                  <button
                    onClick={() => setLocale(opt.code)}
                    class={active() ? "is-active" : ""}
                  >
                    {C[opt.labelKey]}
                  </button>
                )
              }}
            </For>
          </div>
        </div>

        <div class="pub-nav-mobile-controls">
          <Show when={props.mobileAction}>{props.mobileAction}</Show>
          {/* Mobile hamburger */}
          <button
            class="pub-nav-hamburger"
            onClick={() => setMenuOpen((v) => !v)}
            style={{
              display: "none",
              background: "none",
              border: "none",
              cursor: "pointer",
              padding: "6px",
              "flex-direction": "column",
              gap: "5px",
            }}
            aria-label={C.menu_aria}
          >
            <span
              style={{
                display: "block",
                width: "22px",
                height: "2px",
                background: transparent() ? "rgba(255,255,255,0.75)" : "#475569",
                transition: "all 0.2s",
                transform: menuOpen() ? "translateY(7px) rotate(45deg)" : "none",
              }}
            />
            <span
              style={{
                display: "block",
                width: "22px",
                height: "2px",
                background: transparent() ? "rgba(255,255,255,0.75)" : "#475569",
                transition: "all 0.2s",
                opacity: menuOpen() ? "0" : "1",
              }}
            />
            <span
              style={{
                display: "block",
                width: "22px",
                height: "2px",
                background: transparent() ? "rgba(255,255,255,0.75)" : "#475569",
                transition: "all 0.2s",
                transform: menuOpen() ? "translateY(-7px) rotate(-45deg)" : "none",
              }}
            />
          </button>
        </div>
      </nav>

      {/* Mobile dropdown menu */}
      <Show when={menuOpen()}>
        <div
          class="pub-mobile-menu"
          style={{
            position: "fixed",
            top: "56px",
            left: "0",
            right: "0",
            "z-index": "199",
            "max-height": "calc(100dvh - 56px)",
            overflow: "auto",
            "-webkit-overflow-scrolling": "touch",
            background: "#fff",
            "border-bottom": "1px solid rgba(0,0,0,0.08)",
            padding: "16px 24px 24px",
            "box-shadow": "0 8px 24px rgba(0,0,0,0.08)",
          }}
        >
          <div style={{ display: "flex", "flex-direction": "column", gap: "4px", "margin-bottom": "16px" }}>
            <For each={links}>
              {(l) => (
                <button
                  onClick={() => go(l.path)}
                  style={{
                    "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                    "font-size": "15px",
                    "font-weight": isActive(l.path) ? "700" : "500",
                    color: isActive(l.path) ? "#d97706" : "#0f172a",
                    background: isActive(l.path) ? "rgba(245,158,11,0.10)" : "none",
                    border: "none",
                    cursor: "pointer",
                    padding: "10px 12px",
                    "border-radius": "8px",
                    "text-align": "left",
                    width: "100%",
                  }}
                >
                  {C[l.labelKey]}
                </button>
              )}
            </For>
          </div>
          <div style={{ display: "flex", gap: "10px" }}>
            <button
              onClick={() => go("/chat")}
              style={{
                flex: "1",
                padding: "11px 20px",
                "border-radius": "8px",
                background: isActive("/chat") ? "#f59e0b" : "#0f172a",
                border: "none",
                cursor: "pointer",
                "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                "font-size": "14px",
                "font-weight": "600",
                color: "#fff",
              }}
            >
              {C.chat}
            </button>
            <a
              href={C.github_url}
              target="_blank"
              rel="noopener noreferrer"
              class="pub-github-star-link"
              style={{
                padding: "11px 16px",
                "border-radius": "8px",
                border: "1px solid rgba(0,0,0,0.10)",
                "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                "font-size": "13px",
                "font-weight": "500",
                color: "#64748b",
                "text-decoration": "none",
                display: "inline-flex",
                "align-items": "center",
                "justify-content": "center",
                gap: "7px",
              }}
            >
              <span>GitHub</span>
              <span class="pub-github-star-count">{displayGithubStars(stars())}</span>
            </a>
          </div>
          <div
            class="pub-mobile-contact"
            style={{
              "margin-top": "14px",
              padding: "12px",
              "border-radius": "12px",
              border: "1px solid rgba(15,23,42,0.08)",
              background: "#f8fafc",
              "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
            }}
          >
            <div style={{ "font-size": "12px", "font-weight": "700", color: "#64748b", "margin-bottom": "8px" }}>
              {C.contact_title}
            </div>
            <PublicContactCards />
          </div>
        </div>
      </Show>
    </>
  )
}

export function PublicFooter() {
  const C = CONTENT.footer
  const navigate = useNavigate()

  const go = (href: string) => {
    navigate(href)
    window.scrollTo({ top: 0, left: 0, behavior: "auto" })
  }

  return (
    <footer
      style={{
        background: "#0f172a",
        color: "#94a3b8",
        padding: "64px 32px 40px",
        "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
      }}
    >
      <div style={{ "max-width": "1100px", margin: "0 auto" }}>
        <div class="pub-footer-grid">
          {/* Brand column */}
          <div>
            <div style={{ display: "flex", "align-items": "center", gap: "10px", "margin-bottom": "16px" }}>
              <img
                src="/logo.svg"
                style={{
                  height: "28px",
                  padding: "4px",
                  "border-radius": "10px",
                  background: "#fff",
                }}
                alt="Hone"
              />
              <span
                style={{
                  color: "#f8fafc",
                  "font-size": "18px",
                  "font-weight": "850",
                  "letter-spacing": "-0.02em",
                }}
              >
                Hone
              </span>
            </div>
            <p style={{ "font-size": "13px", "line-height": "1.7", color: "#64748b", margin: "0 0 16px" }}>
              {C.tagline}
            </p>
          </div>

          <For each={Object.values(C.columns)}>
            {(col) => (
              <div>
                <h4
                  style={{
                    "font-size": "11px",
                    "font-weight": "600",
                    "letter-spacing": "0.20em",
                    "text-transform": "uppercase",
                    color: "#475569",
                    margin: "0 0 16px",
                  }}
                >
                  {col.title}
                </h4>
                <ul style={{ "list-style": "none", padding: "0", margin: "0", display: "flex", "flex-direction": "column", gap: "10px" }}>
                  <For each={col.items}>
                    {(item) => (
                      <li>
                        {item.href.startsWith("http") || item.href === "#" ? (
                          <a
                            href={item.href}
                            target={item.href.startsWith("http") ? "_blank" : undefined}
                            rel="noopener noreferrer"
                            style={{ color: "#64748b", "text-decoration": "none", "font-size": "13px" }}
                          >
                            {item.label}
                          </a>
                        ) : (
                          <button
                            onClick={() => go(item.href)}
                            style={{
                              background: "none",
                              border: "none",
                              padding: "0",
                              cursor: "pointer",
                              color: "#64748b",
                              "font-size": "13px",
                              "text-align": "left",
                              "font-family": "inherit",
                            }}
                          >
                            {item.label}
                          </button>
                        )}
                      </li>
                    )}
                  </For>
                </ul>
              </div>
            )}
          </For>
        </div>

        <div
          style={{
            "border-top": "1px solid #1e293b",
            "padding-top": "24px",
            display: "flex",
            "align-items": "center",
            "justify-content": "space-between",
          }}
        >
          <span style={{ "font-size": "12px", color: "#334155" }}>{C.copyright}</span>
          <span
            style={{
              "font-size": "10px",
              "letter-spacing": "0.25em",
              "text-transform": "uppercase",
              color: "#1e293b",
              "font-weight": "600",
            }}
          >
            {C.mantra}
          </span>
        </div>
      </div>
    </footer>
  )
}
