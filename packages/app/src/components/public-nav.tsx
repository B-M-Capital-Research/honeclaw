// public-nav.tsx — Navigation + Footer for Hone Public Site

import { createSignal, For, onCleanup, onMount, Show } from "solid-js"
import { useNavigate, useLocation } from "@solidjs/router"
import { CONTENT } from "@/lib/public-content"
import { setLocale, useLocale } from "@/lib/i18n"
import "../pages/public-site.css"

export function PublicNav() {
  const [scrolled, setScrolled] = createSignal(false)
  const [menuOpen, setMenuOpen] = createSignal(false)
  const navigate = useNavigate()
  const location = useLocation()
  const C = CONTENT.nav

  const page = () => location.pathname

  onMount(() => {
    const onScroll = () => setScrolled(window.scrollY > 40)
    window.addEventListener("scroll", onScroll)
    onCleanup(() => window.removeEventListener("scroll", onScroll))
  })

  const isHome = () => page() === "/"
  const transparent = () => isHome() && !scrolled() && !menuOpen()

  const navBg = () =>
    menuOpen()
      ? "#fff"
      : transparent()
      ? "transparent"
      : "rgba(255,255,255,0.96)"
  const navBorder = () => (transparent() ? "none" : "1px solid rgba(0,0,0,0.08)")
  const logoColor = () => (transparent() ? "rgba(255,255,255,0.75)" : "#64748b")

  const getLinkColor = (path: string) => {
    if (page() === path) return "#f59e0b"
    return transparent() ? "rgba(255,255,255,0.75)" : "#475569"
  }
  const getLinkBg = (path: string) => {
    if (page() !== path) return "transparent"
    return transparent() ? "rgba(255,255,255,0.08)" : "rgba(245,158,11,0.08)"
  }
  const contactHref = () => `mailto:${C.contact_email}`

  // NOTE: store `labelKey` (not pre-resolved strings) so each render re-reads
  // the CONTENT proxy inside JSX and tracks the locale signal.
  const links = [
    { labelKey: "home", path: "/" },
    { labelKey: "roadmap", path: "/roadmap" },
    { labelKey: "me", path: "/me" },
  ] as const

  const go = (path: string) => {
    navigate(path)
    window.scrollTo(0, 0)
    setMenuOpen(false)
  }

  return (
    <>
      <nav
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
        <div class="pub-nav-links" style={{ "align-items": "center", gap: "2px" }}>
          <For each={links}>
            {(l) => (
              <button
                onClick={() => go(l.path)}
                style={{
                  "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                  "font-size": "14px",
                  "font-weight": "500",
                  background: getLinkBg(l.path),
                  border: "none",
                  cursor: "pointer",
                  padding: "6px 12px",
                  "border-radius": "6px",
                  transition: "color 0.2s, background 0.2s",
                  "letter-spacing": "0.01em",
                  color: getLinkColor(l.path),
                }}
              >
                {C[l.labelKey]}
              </button>
            )}
          </For>

          <button
            onClick={() => go("/chat")}
            style={{
              "margin-left": "8px",
              padding: "7px 18px",
              "border-radius": "6px",
              background: "#f59e0b",
              border: "none",
              cursor: "pointer",
              "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
              "font-size": "13px",
              "font-weight": "600",
              color: "#fff",
              "letter-spacing": "0.02em",
              "box-shadow": "0 2px 8px rgba(245,158,11,0.30)",
              transition: "background 0.2s",
            }}
            onMouseEnter={(e) => { e.currentTarget.style.background = "#d97706" }}
            onMouseLeave={(e) => { e.currentTarget.style.background = "#f59e0b" }}
          >
            {C.chat}
          </button>

          <a
            href={contactHref()}
            class="pub-contact-link"
            title={`${C.contact_wechat_label}: ${C.contact_wechat}`}
            aria-label={`${C.contact_email_label}: ${C.contact_email}`}
            style={{
              "margin-left": "10px",
              "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
              "font-size": "12px",
              "font-weight": "600",
              color: transparent() ? "rgba(255,255,255,0.78)" : "#334155",
              "text-decoration": "none",
              padding: "6px 10px",
              "border-radius": "6px",
              border: transparent() ? "1px solid rgba(255,255,255,0.18)" : "1px solid rgba(0,0,0,0.10)",
              background: transparent() ? "rgba(255,255,255,0.08)" : "rgba(255,255,255,0.84)",
              transition: "all 0.2s",
              "letter-spacing": "0",
              "white-space": "nowrap",
            }}
          >
            <span class="pub-contact-label">{C.contact_label}</span>
            <span class="pub-contact-email">{C.contact_email}</span>
          </a>

          <a
            href={C.github_url}
            target="_blank"
            rel="noopener noreferrer"
            style={{
              "margin-left": "10px",
              "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
              "font-size": "12px",
              "font-weight": "500",
              color: transparent() ? "rgba(255,255,255,0.5)" : "#94a3b8",
              "text-decoration": "none",
              padding: "6px 10px",
              "border-radius": "6px",
              border: transparent() ? "1px solid rgba(255,255,255,0.15)" : "1px solid rgba(0,0,0,0.10)",
              transition: "all 0.2s",
              "letter-spacing": "0.02em",
            }}
          >
            GitHub ↗
          </a>

          <div
            style={{
              "margin-left": "8px",
              display: "inline-flex",
              "align-items": "center",
              gap: "2px",
              padding: "2px",
              "border-radius": "6px",
              border: transparent() ? "1px solid rgba(255,255,255,0.15)" : "1px solid rgba(0,0,0,0.10)",
            }}
          >
            <For each={[{ code: "zh" as const, labelKey: "locale_zh" as const }, { code: "en" as const, labelKey: "locale_en" as const }]}>
              {(opt) => {
                const active = () => useLocale() === opt.code
                return (
                  <button
                    onClick={() => setLocale(opt.code)}
                    style={{
                      "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                      "font-size": "11px",
                      "font-weight": active() ? "600" : "500",
                      "letter-spacing": "0.05em",
                      padding: "4px 8px",
                      "border-radius": "4px",
                      border: "none",
                      cursor: "pointer",
                      background: active()
                        ? transparent()
                          ? "rgba(255,255,255,0.14)"
                          : "rgba(245,158,11,0.10)"
                        : "transparent",
                      color: active()
                        ? transparent()
                          ? "#fff"
                          : "#f59e0b"
                        : transparent()
                        ? "rgba(255,255,255,0.55)"
                        : "#64748b",
                      transition: "color 0.2s, background 0.2s",
                    }}
                  >
                    {C[opt.labelKey]}
                  </button>
                )
              }}
            </For>
          </div>
        </div>

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
      </nav>

      {/* Mobile dropdown menu */}
      <Show when={menuOpen()}>
        <div
          style={{
            position: "fixed",
            top: "56px",
            left: "0",
            right: "0",
            "z-index": "199",
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
                    "font-weight": page() === l.path ? "600" : "500",
                    color: page() === l.path ? "#f59e0b" : "#0f172a",
                    background: "none",
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
                background: "#f59e0b",
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
              style={{
                padding: "11px 16px",
                "border-radius": "8px",
                border: "1px solid rgba(0,0,0,0.10)",
                "font-family": "var(--font-sans, 'Plus Jakarta Sans', sans-serif)",
                "font-size": "13px",
                "font-weight": "500",
                color: "#64748b",
                "text-decoration": "none",
              }}
            >
              GitHub ↗
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
              {C.contact_label}
            </div>
            <div style={{ display: "grid", gap: "6px", "font-size": "13px", color: "#0f172a" }}>
              <div>
                <span style={{ color: "#64748b" }}>{C.contact_wechat_label}: </span>
                <span style={{ "font-weight": "700" }}>{C.contact_wechat}</span>
              </div>
              <a href={contactHref()} style={{ color: "#0f172a", "font-weight": "700", "text-decoration": "none", "word-break": "break-word" }}>
                <span style={{ color: "#64748b", "font-weight": "500" }}>{C.contact_email_label}: </span>
                {C.contact_email}
              </a>
            </div>
          </div>
        </div>
      </Show>
    </>
  )
}

export function PublicFooter() {
  const C = CONTENT.footer
  const Cnav = CONTENT.nav
  const navigate = useNavigate()

  const go = (href: string) => {
    navigate(href)
    window.scrollTo(0, 0)
  }

  const chipStyle = (active: boolean) => ({
    padding: "3px 10px",
    "border-radius": "999px",
    border: active ? "1px solid #f59e0b" : "1px solid #1e293b",
    cursor: "pointer",
    background: active ? "rgba(245,158,11,0.10)" : "transparent",
    color: active ? "#f59e0b" : "#475569",
    "font-size": "11px",
    "letter-spacing": "0.05em",
    "font-family": "inherit",
    transition: "color 0.2s, background 0.2s, border-color 0.2s",
  })

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
              <img src="/logo.svg" style={{ height: "24px", filter: "brightness(0.9)" }} alt="Hone" />
            </div>
            <p style={{ "font-size": "13px", "line-height": "1.7", color: "#64748b", margin: "0 0 16px" }}>
              {C.tagline}
            </p>
            <div style={{ display: "flex", gap: "6px" }}>
              <button
                onClick={() => setLocale("zh")}
                style={chipStyle(useLocale() === "zh")}
              >
                {Cnav.locale_zh}
              </button>
              <button
                onClick={() => setLocale("en")}
                style={chipStyle(useLocale() === "en")}
              >
                {Cnav.locale_en}
              </button>
            </div>
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
