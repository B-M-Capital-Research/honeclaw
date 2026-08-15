/**
 * Warms a lazily-loaded route's chunk before the click that needs it.
 *
 * Every route is `lazy()`, so the first navigation to one pays for a network
 * round-trip to fetch its JavaScript before anything can render. Pointer entry
 * and keyboard focus both precede activation by enough time to hide that cost,
 * and the dynamic import is idempotent — the bundler resolves an already-loaded
 * chunk from memory, so hovering repeatedly costs nothing.
 */
const loaders = {
  me: () => import("@/pages/public-me"),
  community: () => import("@/pages/public-community"),
  pushes: () => import("@/pages/public-pushes"),
  plan: () => import("@/pages/public-plan"),
  research: () => import("@/pages/public-research"),
} satisfies Record<string, () => Promise<unknown>>;

export type PrefetchableRoute = keyof typeof loaders;

const started = new Set<PrefetchableRoute>();

export function prefetchRoute(route: PrefetchableRoute): void {
  if (started.has(route)) return;
  started.add(route);
  // A prefetch is an optimisation; a failure here must never surface to the
  // user, and the real navigation will retry the import anyway.
  void loaders[route]().catch(() => started.delete(route));
}

/** Handlers to spread onto a control that navigates to `route`. */
export function routePrefetchHandlers(route: PrefetchableRoute) {
  const warm = () => prefetchRoute(route);
  return { onPointerEnter: warm, onFocus: warm, onTouchStart: warm };
}
