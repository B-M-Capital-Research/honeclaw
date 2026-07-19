import { A, useLocation, useNavigate } from "@solidjs/router";
import {
  createResource,
  createSignal,
  For,
  Match,
  onCleanup,
  onMount,
  Show,
  Switch,
  type JSX,
} from "solid-js";

import { HoneBrand } from "@/components/hone-brand";
import {
  PUBLIC_BILIBILI_URL,
  PUBLIC_YOUTUBE_URL,
} from "@/components/public-contact-menu";
import { displayGithubStars, fetchGithubStars } from "@/lib/github-stars";
import { setLocale, useLocale } from "@/lib/i18n";
import { CONTENT } from "@/lib/public-content";
import "../pages/public-foundation.css";
import "../pages/public-site.css";
import "../pages/public-polish.css";

const NAV_LINKS = [
  { labelKey: "home", path: "/" },
  { labelKey: "community", path: "/community" },
  { labelKey: "blog", path: "/blog" },
  { labelKey: "plan", path: "/plan" },
] as const;

const MOBILE_TABS = [
  { labelKey: "home", path: "/", icon: "home" },
  { labelKey: "chat", path: "/chat", icon: "chat" },
  { labelKey: "community", path: "/community", icon: "community" },
  { labelKey: "me", path: "/me", icon: "me" },
] as const;

function MobileTabIcon(props: { icon: (typeof MOBILE_TABS)[number]["icon"] }) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <Switch>
        <Match when={props.icon === "home"}>
          <path d="m4 10 8-6 8 6v9a1 1 0 0 1-1 1h-5v-6h-4v6H5a1 1 0 0 1-1-1Z" />
        </Match>
        <Match when={props.icon === "chat"}>
          <path d="M20 15a3 3 0 0 1-3 3H9l-5 3v-6a3 3 0 0 1-1-2.2V7a3 3 0 0 1 3-3h11a3 3 0 0 1 3 3Z" />
        </Match>
        <Match when={props.icon === "community"}>
          <path d="M5 5.5h14a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H8l-5 2v-13a2 2 0 0 1 2-2Z" />
          <path d="M7.5 10h9M7.5 14h6" />
        </Match>
        <Match when={props.icon === "me"}>
          <circle cx="12" cy="8" r="3.5" />
          <path d="M5.5 20a6.5 6.5 0 0 1 13 0" />
        </Match>
      </Switch>
    </svg>
  );
}

export function PublicNav(
  props: {
    extraActions?: JSX.Element;
    mobileAction?: JSX.Element;
    chatMode?: boolean;
    mobileLabel?: string;
    communityUnread?: boolean;
  } = {},
) {
  const [scrolled, setScrolled] = createSignal(false);
  const [menuOpen, setMenuOpen] = createSignal(false);
  const [moreOpen, setMoreOpen] = createSignal(false);
  const [stars] = createResource(fetchGithubStars);
  const location = useLocation();
  const C = CONTENT.nav;
  let moreMenuEl: HTMLDivElement | undefined;

  onMount(() => {
    const onScroll = () => setScrolled(window.scrollY > 20);
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      setMenuOpen(false);
      setMoreOpen(false);
    };
    const onPointerDown = (event: PointerEvent) => {
      if (!moreMenuEl?.contains(event.target as Node)) setMoreOpen(false);
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    document.addEventListener("keydown", onKeyDown);
    document.addEventListener("pointerdown", onPointerDown);
    onCleanup(() => {
      window.removeEventListener("scroll", onScroll);
      document.removeEventListener("keydown", onKeyDown);
      document.removeEventListener("pointerdown", onPointerDown);
    });
  });

  const isActive = (path: string) => {
    if (path === "/") return location.pathname === "/";
    return (
      location.pathname === path || location.pathname.startsWith(`${path}/`)
    );
  };

  const closeAfterNavigation = (path: string) => {
    const changed = location.pathname !== path;
    if (changed) window.scrollTo({ top: 0, left: 0, behavior: "auto" });
    setMenuOpen(false);
    setMoreOpen(false);
  };

  const languageSwitch = () => (
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
            aria-pressed={useLocale() === option.code}
          >
            {C[option.labelKey]}
          </button>
        )}
      </For>
    </div>
  );

  const utilityLinks = () => (
    <div class="pub-nav-utility-links">
      <a href={C.github_url} target="_blank" rel="noopener noreferrer">
        <span>GitHub</span>
        <small>{displayGithubStars(stars())}</small>
      </a>
      <a href={PUBLIC_BILIBILI_URL} target="_blank" rel="noopener noreferrer">
        <span>{C.bilibili_label}</span>
        <small>Bilibili</small>
      </a>
      <a href={PUBLIC_YOUTUBE_URL} target="_blank" rel="noopener noreferrer">
        <span>YouTube</span>
        <small>{C.youtube_channel_name}</small>
      </a>
      <a href={`mailto:${C.contact_email}`}>
        <span>{C.contact_email_label}</span>
        <small>{C.contact_email}</small>
      </a>
      <div
        class="pub-nav-utility-item"
        title={`${C.contact_wechat_label}: ${C.contact_wechat}`}
      >
        <span>{C.contact_wechat_group}</span>
        <small>{C.contact_wechat}</small>
      </div>
    </div>
  );

  return (
    <>
      <nav
        class="pub-nav"
        classList={{
          "is-scrolled": scrolled(),
          "is-open": menuOpen(),
          "is-chat-mode": props.chatMode,
        }}
        aria-label="HONE 主导航"
      >
        <A
          href="/"
          class="pub-nav-brand"
          onClick={() => closeAfterNavigation("/")}
          aria-label="HONE"
        >
          <HoneBrand />
          <Show when={props.mobileLabel}>
            <span class="pub-nav-chat-copy">
              <strong>HONE</strong>
              <small>{props.mobileLabel}</small>
            </span>
          </Show>
        </A>

        <div class="pub-nav-links">
          <div class="pub-nav-primary">
            <For each={NAV_LINKS}>
              {(link) => (
                <A
                  href={link.path}
                  onClick={() => closeAfterNavigation(link.path)}
                  class="pub-nav-link"
                  classList={{ "is-active": isActive(link.path) }}
                  aria-current={isActive(link.path) ? "page" : undefined}
                >
                  {C[link.labelKey]}
                  <Show when={link.path === "/community" && props.communityUnread}>
                    <i class="pub-community-unread-dot" aria-hidden="true" />
                  </Show>
                </A>
              )}
            </For>
          </div>

          <div class="pub-nav-actions">
            <Show when={props.extraActions}>
              <div class="pub-nav-extra-actions">{props.extraActions}</div>
            </Show>
            <div class="pub-nav-more" ref={moreMenuEl}>
              <button
                type="button"
                class="pub-nav-more-trigger"
                classList={{
                  "is-active": isActive("/roadmap") || isActive("/me"),
                }}
                aria-expanded={moreOpen()}
                aria-controls="pub-nav-more-panel"
                onClick={() => setMoreOpen((open) => !open)}
              >
                <span>{C.more}</span>
                <i aria-hidden="true">•••</i>
              </button>
              <Show when={moreOpen()}>
                <section
                  id="pub-nav-more-panel"
                  class="pub-nav-more-panel"
                  aria-label={C.more}
                >
                  <div class="pub-nav-more-links">
                    <A href="/roadmap" onClick={() => closeAfterNavigation("/roadmap")}>
                      <span>{C.roadmap}</span><small>→</small>
                    </A>
                    <A href="/me" onClick={() => closeAfterNavigation("/me")}>
                      <span>{C.me}</span><small>→</small>
                    </A>
                  </div>
                  {utilityLinks()}
                  <footer>{languageSwitch()}</footer>
                </section>
              </Show>
            </div>
            <A
              href="/chat"
              onClick={() => closeAfterNavigation("/chat")}
              class="pub-nav-cta"
              classList={{ "is-active": isActive("/chat") }}
              aria-current={isActive("/chat") ? "page" : undefined}
            >
              <span>{C.chat}</span>
              <span aria-hidden="true">↗</span>
            </A>
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
            aria-controls="pub-mobile-menu"
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
        <section id="pub-mobile-menu" class="pub-mobile-menu" aria-label={C.menu_aria}>
          <div class="pub-mobile-menu-links">
            <For
              each={[
                { labelKey: "plan" as const, path: "/plan" },
                { labelKey: "roadmap" as const, path: "/roadmap" },
                { labelKey: "blog" as const, path: "/blog" },
              ]}
            >
              {(link) => (
                <A
                  href={link.path}
                  onClick={() => closeAfterNavigation(link.path)}
                  classList={{ "is-active": isActive(link.path) }}
                  aria-current={isActive(link.path) ? "page" : undefined}
                >
                  <span>{C[link.labelKey]}</span>
                  <small aria-hidden="true">→</small>
                </A>
              )}
            </For>
          </div>
          {utilityLinks()}
          <div class="pub-mobile-menu-meta">
            <span>{C.contact_title}</span>
            {languageSwitch()}
          </div>
        </section>
      </Show>

      <nav class="pub-mobile-tabs" aria-label="HONE">
        <For each={MOBILE_TABS}>
          {(tab) => (
            <A
              href={tab.path}
              class="pub-mobile-tab"
              classList={{ "is-active": isActive(tab.path) }}
              aria-current={isActive(tab.path) ? "page" : undefined}
              onClick={() => closeAfterNavigation(tab.path)}
            >
              <span class="pub-mobile-tab-icon">
                <MobileTabIcon icon={tab.icon} />
                <Show when={tab.path === "/community" && props.communityUnread}>
                  <i class="pub-community-unread-dot" aria-hidden="true" />
                </Show>
              </span>
              <small>{C[tab.labelKey]}</small>
            </A>
          )}
        </For>
      </nav>
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
