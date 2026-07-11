import { useLocation, useNavigate } from "@solidjs/router";
import {
  createResource,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
  type JSX,
} from "solid-js";

import { HoneBrand } from "@/components/hone-brand";
import {
  PublicContactCards,
  PublicContactMenu,
} from "@/components/public-contact-menu";
import { displayGithubStars, fetchGithubStars } from "@/lib/github-stars";
import { setLocale, useLocale } from "@/lib/i18n";
import { CONTENT } from "@/lib/public-content";
import "../pages/public-foundation.css";
import "../pages/public-site.css";
import "../pages/public-polish.css";

const NAV_LINKS = [
  { labelKey: "home", path: "/" },
  { labelKey: "roadmap", path: "/roadmap" },
  { labelKey: "blog", path: "/blog" },
  { labelKey: "me", path: "/me" },
] as const;

export function PublicNav(
  props: { extraActions?: JSX.Element; mobileAction?: JSX.Element } = {},
) {
  const [scrolled, setScrolled] = createSignal(false);
  const [menuOpen, setMenuOpen] = createSignal(false);
  const [stars] = createResource(fetchGithubStars);
  const navigate = useNavigate();
  const location = useLocation();
  const C = CONTENT.nav;

  onMount(() => {
    const onScroll = () => setScrolled(window.scrollY > 20);
    window.addEventListener("scroll", onScroll, { passive: true });
    onCleanup(() => window.removeEventListener("scroll", onScroll));
  });

  const isActive = (path: string) => {
    if (path === "/") return location.pathname === "/";
    return (
      location.pathname === path || location.pathname.startsWith(`${path}/`)
    );
  };

  const go = (path: string) => {
    const changed = location.pathname !== path;
    navigate(path);
    if (changed) window.scrollTo({ top: 0, left: 0, behavior: "auto" });
    setMenuOpen(false);
  };

  return (
    <>
      <nav
        class="pub-nav"
        classList={{ "is-scrolled": scrolled(), "is-open": menuOpen() }}
        aria-label="HONE"
      >
        <button
          type="button"
          class="pub-nav-brand"
          onClick={() => go("/")}
          aria-label="HONE"
        >
          <HoneBrand />
        </button>

        <div class="pub-nav-links">
          <div class="pub-nav-primary">
            <For each={NAV_LINKS}>
              {(link) => (
                <button
                  type="button"
                  onClick={() => go(link.path)}
                  class="pub-nav-link"
                  classList={{ "is-active": isActive(link.path) }}
                >
                  {C[link.labelKey]}
                </button>
              )}
            </For>
          </div>

          <div class="pub-nav-actions">
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
              <span class="pub-github-star-count">
                {displayGithubStars(stars())}
              </span>
            </a>
            <div class="pub-nav-lang" aria-label="Language">
              <For
                each={[
                  { code: "zh" as const, labelKey: "locale_zh" as const },
                  { code: "en" as const, labelKey: "locale_en" as const },
                ]}
              >
                {(option) => (
                  <button
                    type="button"
                    onClick={() => setLocale(option.code)}
                    classList={{ "is-active": useLocale() === option.code }}
                  >
                    {C[option.labelKey]}
                  </button>
                )}
              </For>
            </div>
            <button
              type="button"
              onClick={() => go("/chat")}
              class="pub-nav-cta"
              classList={{ "is-active": isActive("/chat") }}
            >
              <span>{C.chat}</span>
              <span aria-hidden="true">↗</span>
            </button>
          </div>
        </div>

        <div class="pub-nav-mobile-controls">
          <Show when={props.mobileAction}>{props.mobileAction}</Show>
          <button
            type="button"
            class="pub-nav-hamburger"
            classList={{ "is-open": menuOpen() }}
            onClick={() => setMenuOpen((open) => !open)}
            aria-expanded={menuOpen()}
            aria-label={C.menu_aria}
          >
            <span />
            <span />
          </button>
        </div>
      </nav>

      <Show when={menuOpen()}>
        <button
          type="button"
          class="pub-mobile-menu-scrim"
          onClick={() => setMenuOpen(false)}
          aria-label={C.menu_aria}
        />
        <section class="pub-mobile-menu" aria-label={C.menu_aria}>
          <div class="pub-mobile-menu-kicker">HONE</div>
          <div class="pub-mobile-menu-links">
            <For each={NAV_LINKS}>
              {(link, index) => (
                <button
                  type="button"
                  onClick={() => go(link.path)}
                  classList={{ "is-active": isActive(link.path) }}
                >
                  <span class="pub-mobile-menu-index">
                    {String(index() + 1).padStart(2, "0")}
                  </span>
                  <span>{C[link.labelKey]}</span>
                  <span aria-hidden="true">→</span>
                </button>
              )}
            </For>
          </div>
          <button
            type="button"
            onClick={() => go("/chat")}
            class="pub-mobile-menu-chat"
          >
            <span>{C.chat}</span>
            <span aria-hidden="true">↗</span>
          </button>
          <div class="pub-mobile-menu-meta">
            <a
              href={C.github_url}
              target="_blank"
              rel="noopener noreferrer"
            >
              GitHub · {displayGithubStars(stars())}
            </a>
            <div class="pub-nav-lang">
              <button
                type="button"
                onClick={() => setLocale("zh")}
                classList={{ "is-active": useLocale() === "zh" }}
              >
                中文
              </button>
              <button
                type="button"
                onClick={() => setLocale("en")}
                classList={{ "is-active": useLocale() === "en" }}
              >
                EN
              </button>
            </div>
          </div>
          <div class="pub-mobile-contact">
            <div class="pub-mobile-contact-title">{C.contact_title}</div>
            <PublicContactCards />
          </div>
        </section>
      </Show>
    </>
  );
}

export function PublicFooter() {
  const C = CONTENT.footer;
  const navigate = useNavigate();

  const go = (href: string) => {
    navigate(href);
    window.scrollTo({ top: 0, left: 0, behavior: "auto" });
  };

  return (
    <footer class="pub-footer">
      <div class="pub-footer-inner">
        <div class="pub-footer-grid">
          <div class="pub-footer-brand">
            <HoneBrand dark />
            <p>{C.tagline}</p>
          </div>
          <For each={Object.values(C.columns)}>
            {(column) => (
              <div class="pub-footer-column">
                <h4>{column.title}</h4>
                <ul>
                  <For each={column.items}>
                    {(item) => (
                      <li>
                        {item.href.startsWith("http") || item.href === "#" ? (
                          <a
                            href={item.href}
                            target={
                              item.href.startsWith("http") ? "_blank" : undefined
                            }
                            rel="noopener noreferrer"
                          >
                            {item.label}
                          </a>
                        ) : (
                          <button type="button" onClick={() => go(item.href)}>
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
        <div class="pub-footer-bottom">
          <span>{C.copyright}</span>
          <strong>{C.mantra}</strong>
        </div>
      </div>
    </footer>
  );
}
