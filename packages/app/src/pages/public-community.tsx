import { Title } from "@solidjs/meta";
import { CONTENT } from "@/lib/public-content";
import { mergeCommunityTimeline } from "@/lib/public-community-timeline";
import {
  cachedCommunityFeed,
  setCachedCommunityFeed,
} from "@/lib/public-session-cache";
import {
  For,
  Match,
  Show,
  Switch,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { Portal } from "solid-js/web";

import { PublicLoginForm } from "@/components/public-login-form";
import { PublicWorkspaceShell } from "@/components/public-workspace-shell";
import { CommunityForum } from "@/components/community-forum";
import {
  getPublicCommunity,
  getPublicCommunityResourceBlob,
  isUnauthorizedApiError,
  markPublicCommunitySeen,
  publicCommunityResourceDownloadName,
  publicCommunityResourceUrl,
} from "@/lib/api";
import {
  clampFinanceCalendarPan,
  financeCalendarAnchoredTransform,
  financeCalendarPinchZoom,
  stepFinanceCalendarZoom,
} from "@/lib/finance-calendar";
import type { PublicCommunityContent, PublicCommunityResource } from "@/lib/types";

import "./public-foundation.css";
import "./public-site.css";
import "./public-polish.css";
import "./public-community.css";

type ViewState = "loading" | "ready" | "login" | "error";
type CommunityView = "official" | "forum";

const SAFE_IMAGE_TYPES = new Set([
  "image/jpeg",
  "image/jpg",
  "image/png",
  "image/webp",
  "image/gif",
  "image/avif",
]);

function normalizedContentType(resource: PublicCommunityResource) {
  return (resource.content_type ?? "").split(";", 1)[0]!.trim().toLowerCase();
}

function formatPublishedAt(item: PublicCommunityContent) {
  const raw = item.published_at_raw || item.published_at;
  if (!raw) return CONTENT.chat_page.community_page.just_now;
  return raw.replace("T", " ").replace(/\+\d\d:\d\d$/, "").slice(0, 16);
}

function resourceIsStored(resource: PublicCommunityResource) {
  return resource.access_state === "stored";
}

function resourceIsImage(resource: PublicCommunityResource) {
  const contentType = normalizedContentType(resource);
  return (
    SAFE_IMAGE_TYPES.has(contentType) ||
    (!contentType && resource.resource_kind === "image")
  );
}

function resourceCanInlinePreview(resource: PublicCommunityResource) {
  return (
    resourceIsStored(resource) &&
    (resourceIsImage(resource) || normalizedContentType(resource) === "application/pdf")
  );
}

async function downloadCommunityResource(resource: PublicCommunityResource, preparedBlob?: Blob) {
  const blob = preparedBlob ?? await getPublicCommunityResourceBlob(
    resource.resource_id,
    resource.version,
    resource.delivery_path,
  );
  const objectUrl = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = objectUrl;
  anchor.download = publicCommunityResourceDownloadName(resource);
  anchor.rel = "noopener";
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  window.setTimeout(() => URL.revokeObjectURL(objectUrl), 1_000);
}

function CommunityMediaPreview(props: {
  resource: PublicCommunityResource;
  onClose: () => void;
}) {
  const [zoom, setZoom] = createSignal(1);
  const [pan, setPan] = createSignal({ x: 0, y: 0 });
  const [fitSize, setFitSize] = createSignal({ width: 0, height: 0 });
  const [interacting, setInteracting] = createSignal(false);
  const [downloadState, setDownloadState] = createSignal<"idle" | "working" | "error">("idle");
  const [documentSource, setDocumentSource] = createSignal<string | null>(null);
  const [documentState, setDocumentState] = createSignal<
    "loading" | "ready" | "slow" | "error"
  >("loading");
  const source = () =>
    publicCommunityResourceUrl(
      props.resource.resource_id,
      props.resource.version,
      props.resource.delivery_path,
    );
  const legacySource = () =>
    publicCommunityResourceUrl(props.resource.resource_id, props.resource.version);
  const isImage = () => resourceIsImage(props.resource);
  const titleId = `community-preview-title-${props.resource.resource_id}`;
  let dialogEl: HTMLDivElement | undefined;
  let closeButtonEl: HTMLButtonElement | undefined;
  let viewportEl: HTMLDivElement | undefined;
  let imageEl: HTMLImageElement | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let removeGestures: (() => void) | undefined;
  let viewFrame = 0;
  let disposed = false;
  let documentObjectUrl: string | undefined;
  let documentBlob: Blob | undefined;
  const documentRequest = new AbortController();
  let documentSlowTimer: number | undefined;
  let pendingView: { zoom: number; x: number; y: number } | undefined;

  const boundedView = (nextZoom: number, x: number, y: number) => {
    const bounds = clampFinanceCalendarPan({
      imageWidth: imageEl?.offsetWidth ?? 0,
      imageHeight: imageEl?.offsetHeight ?? 0,
      viewportWidth: viewportEl?.clientWidth ?? 0,
      viewportHeight: viewportEl?.clientHeight ?? 0,
      zoom: nextZoom,
      x,
      y,
    });
    return { zoom: nextZoom, ...bounds };
  };

  const commitView = (nextZoom: number, x: number, y: number) => {
    pendingView = boundedView(nextZoom, x, y);
    if (viewFrame) return;
    viewFrame = requestAnimationFrame(() => {
      viewFrame = 0;
      const next = pendingView;
      pendingView = undefined;
      if (!next) return;
      setZoom(next.zoom);
      setPan({ x: next.x, y: next.y });
    });
  };

  const fitImageToViewport = () => {
    if (!viewportEl || !imageEl || !imageEl.naturalWidth || !imageEl.naturalHeight) return;
    const scale = Math.min(
      viewportEl.clientWidth / imageEl.naturalWidth,
      viewportEl.clientHeight / imageEl.naturalHeight,
    );
    setFitSize({
      width: Math.max(1, Math.floor(imageEl.naturalWidth * scale)),
      height: Math.max(1, Math.floor(imageEl.naturalHeight * scale)),
    });
    commitView(1, 0, 0);
  };

  const changeZoom = (direction: -1 | 1) => {
    setInteracting(false);
    commitView(stepFinanceCalendarZoom(zoom(), direction), pan().x, pan().y);
  };

  const fitPreview = () => {
    setInteracting(false);
    commitView(1, 0, 0);
  };

  const bindViewport = (element: HTMLDivElement) => {
    viewportEl = element;
    resizeObserver?.disconnect();
    resizeObserver = new ResizeObserver(fitImageToViewport);
    resizeObserver.observe(element);

    let pinch:
      | {
          distance: number;
          zoom: number;
          x: number;
          y: number;
          centerX: number;
          centerY: number;
        }
      | undefined;
    let drag: { startX: number; startY: number; x: number; y: number } | undefined;
    let pointerId: number | undefined;

    const touchMetrics = (event: TouchEvent) => {
      const first = event.touches.item(0);
      const second = event.touches.item(1);
      if (!first || !second) return null;
      const rect = element.getBoundingClientRect();
      return {
        distance: Math.hypot(second.clientX - first.clientX, second.clientY - first.clientY),
        centerX: (first.clientX + second.clientX) / 2 - rect.left,
        centerY: (first.clientY + second.clientY) / 2 - rect.top,
      };
    };
    const beginDrag = (clientX: number, clientY: number) => {
      drag = { startX: clientX, startY: clientY, x: pan().x, y: pan().y };
    };
    const onTouchStart = (event: TouchEvent) => {
      const metrics = touchMetrics(event);
      if (metrics) {
        pinch = { ...metrics, zoom: zoom(), x: pan().x, y: pan().y };
        drag = undefined;
        setInteracting(true);
        return;
      }
      const touch = event.touches.item(0);
      if (touch) beginDrag(touch.clientX, touch.clientY);
    };
    const onTouchMove = (event: TouchEvent) => {
      const metrics = touchMetrics(event);
      if (pinch && metrics && pinch.distance > 0) {
        event.preventDefault();
        const nextZoom = financeCalendarPinchZoom(
          pinch.zoom,
          metrics.distance,
          pinch.distance,
        );
        const next = financeCalendarAnchoredTransform({
          startZoom: pinch.zoom,
          nextZoom,
          startX: pinch.x,
          startY: pinch.y,
          startCenterX: pinch.centerX,
          startCenterY: pinch.centerY,
          nextCenterX: metrics.centerX,
          nextCenterY: metrics.centerY,
          viewportWidth: element.clientWidth,
          viewportHeight: element.clientHeight,
        });
        commitView(nextZoom, next.x, next.y);
        return;
      }
      const touch = event.touches.item(0);
      if (!drag || !touch || zoom() <= 1) return;
      event.preventDefault();
      setInteracting(true);
      commitView(
        zoom(),
        drag.x + touch.clientX - drag.startX,
        drag.y + touch.clientY - drag.startY,
      );
    };
    const onTouchEnd = (event: TouchEvent) => {
      if (event.touches.length >= 2) return;
      pinch = undefined;
      const touch = event.touches.item(0);
      if (touch) beginDrag(touch.clientX, touch.clientY);
      else {
        drag = undefined;
        setInteracting(false);
      }
    };
    const onPointerDown = (event: PointerEvent) => {
      if (event.pointerType === "touch" || zoom() <= 1) return;
      pointerId = event.pointerId;
      element.setPointerCapture(event.pointerId);
      beginDrag(event.clientX, event.clientY);
      setInteracting(true);
    };
    const onPointerMove = (event: PointerEvent) => {
      if (pointerId !== event.pointerId || !drag || zoom() <= 1) return;
      commitView(
        zoom(),
        drag.x + event.clientX - drag.startX,
        drag.y + event.clientY - drag.startY,
      );
    };
    const onPointerEnd = (event: PointerEvent) => {
      if (pointerId !== event.pointerId) return;
      pointerId = undefined;
      drag = undefined;
      setInteracting(false);
    };
    const onWheel = (event: WheelEvent) => {
      event.preventDefault();
      changeZoom(event.deltaY < 0 ? 1 : -1);
    };
    const onDoubleClick = () => {
      if (zoom() > 1) fitPreview();
      else commitView(2, 0, 0);
    };

    element.addEventListener("touchstart", onTouchStart, { passive: true });
    element.addEventListener("touchmove", onTouchMove, { passive: false });
    element.addEventListener("touchend", onTouchEnd, { passive: true });
    element.addEventListener("touchcancel", onTouchEnd, { passive: true });
    element.addEventListener("pointerdown", onPointerDown);
    element.addEventListener("pointermove", onPointerMove);
    element.addEventListener("pointerup", onPointerEnd);
    element.addEventListener("pointercancel", onPointerEnd);
    element.addEventListener("wheel", onWheel, { passive: false });
    element.addEventListener("dblclick", onDoubleClick);
    removeGestures = () => {
      element.removeEventListener("touchstart", onTouchStart);
      element.removeEventListener("touchmove", onTouchMove);
      element.removeEventListener("touchend", onTouchEnd);
      element.removeEventListener("touchcancel", onTouchEnd);
      element.removeEventListener("pointerdown", onPointerDown);
      element.removeEventListener("pointermove", onPointerMove);
      element.removeEventListener("pointerup", onPointerEnd);
      element.removeEventListener("pointercancel", onPointerEnd);
      element.removeEventListener("wheel", onWheel);
      element.removeEventListener("dblclick", onDoubleClick);
    };
  };

  const download = async () => {
    if (downloadState() === "working") return;
    setDownloadState("working");
    try {
      await downloadCommunityResource(props.resource, documentBlob);
      setDownloadState("idle");
    } catch {
      setDownloadState("error");
    }
  };

  onMount(() => {
    if (!isImage()) {
      documentSlowTimer = window.setTimeout(() => {
        if (!disposed && documentState() === "loading") setDocumentState("slow");
      }, 5_000);
      void getPublicCommunityResourceBlob(
        props.resource.resource_id,
        props.resource.version,
        props.resource.delivery_path,
        documentRequest.signal,
      )
        .then((blob) => {
          if (disposed) return;
          documentBlob = blob;
          documentObjectUrl = URL.createObjectURL(blob);
          setDocumentSource(documentObjectUrl);
        })
        .catch(() => {
          if (!disposed) setDocumentState("error");
        });
    }

    const previousFocus = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : undefined;
    const pageRoot = document.querySelector<HTMLElement>(".public-community-page");
    const rootWasInert = pageRoot?.hasAttribute("inert") ?? false;
    const previousBodyOverflow = document.body.style.overflow;
    const previousHtmlOverflow = document.documentElement.style.overflow;
    pageRoot?.setAttribute("inert", "");
    document.body.style.overflow = "hidden";
    document.documentElement.style.overflow = "hidden";

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        props.onClose();
        return;
      }
      if (event.key !== "Tab" || !dialogEl) return;
      const focusable = Array.from(
        dialogEl.querySelectorAll<HTMLElement>(
          'button:not(:disabled), a[href], iframe, [tabindex]:not([tabindex="-1"])',
        ),
      );
      if (!focusable.length) return;
      const first = focusable[0]!;
      const last = focusable[focusable.length - 1]!;
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", onKeyDown);
    queueMicrotask(() => closeButtonEl?.focus());

    onCleanup(() => {
      document.removeEventListener("keydown", onKeyDown);
      if (!rootWasInert) pageRoot?.removeAttribute("inert");
      document.body.style.overflow = previousBodyOverflow;
      document.documentElement.style.overflow = previousHtmlOverflow;
      previousFocus?.focus();
    });
  });

  onCleanup(() => {
    disposed = true;
    documentRequest.abort();
    if (documentSlowTimer !== undefined) window.clearTimeout(documentSlowTimer);
    if (documentObjectUrl) URL.revokeObjectURL(documentObjectUrl);
    removeGestures?.();
    resizeObserver?.disconnect();
    if (viewFrame) cancelAnimationFrame(viewFrame);
  });

  return (
    <Portal>
      <div
        ref={(element) => { dialogEl = element; }}
        class="public-community-lightbox"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
      >
        <header>
          <div>
            <small>{CONTENT.chat_page.community_page.official}</small>
            <strong id={titleId}>{props.resource.display_name || CONTENT.chat_page.community_page.resources}</strong>
          </div>
          <button
            ref={(element) => { closeButtonEl = element; }}
            type="button"
            onClick={props.onClose}
            aria-label={CONTENT.chat_page.community_page.close_preview}
          >
            ×
          </button>
        </header>
        <main classList={{ "is-image": isImage() }}>
          <Show
            when={isImage()}
            fallback={
              <div class="public-community-document-preview">
                <Show
                  when={documentState() !== "error"}
                  fallback={
                    <div class="public-community-document-fallback" role="alert">
                      <strong>{CONTENT.chat_page.community_page.pdf_unsupported}</strong>
                      <span>{CONTENT.chat_page.community_page.pdf_fallback}</span>
                    </div>
                  }
                >
                  <Show
                    when={documentSource()}
                    fallback={<div class="public-workspace-state" role="status">{CONTENT.chat_page.community_page.pdf_preparing}</div>}
                  >
                    <iframe
                      title={props.resource.display_name || CONTENT.chat_page.community_page.file_preview}
                      src={documentSource()!}
                      sandbox="allow-downloads allow-same-origin"
                      referrerPolicy="no-referrer"
                      onLoad={() => {
                        if (documentSlowTimer !== undefined) {
                          window.clearTimeout(documentSlowTimer);
                          documentSlowTimer = undefined;
                        }
                        setDocumentState("ready");
                      }}
                      onError={() => setDocumentState("error")}
                    />
                  </Show>
                </Show>
                <div
                  class="public-community-document-status"
                  classList={{ "is-warning": documentState() === "slow" }}
                  role="status"
                >
                  {documentState() === "slow"
                    ? CONTENT.chat_page.community_page.pdf_slow
                    : documentState() === "error"
                      ? CONTENT.chat_page.community_page.pdf_unavailable
                    : documentState() === "ready"
                      ? CONTENT.chat_page.community_page.pdf_loaded
                      : CONTENT.chat_page.community_page.pdf_verifying}
                </div>
              </div>
            }
          >
            <div
              class="public-community-lightbox-viewport"
              ref={(element) => bindViewport(element)}
            >
              <div class="public-community-lightbox-canvas">
                <img
                  ref={(element) => { imageEl = element; }}
                  classList={{ "is-interacting": interacting() }}
                  style={{
                    width: fitSize().width ? `${fitSize().width}px` : undefined,
                    height: fitSize().height ? `${fitSize().height}px` : undefined,
                    transform: `translate3d(${pan().x}px, ${pan().y}px, 0) scale(${zoom()})`,
                  }}
                  src={source()}
                  alt={props.resource.display_name || CONTENT.chat_page.community_page.image}
                  onLoad={fitImageToViewport}
                  onError={(event) => {
                    const fallback = legacySource();
                    if (event.currentTarget.src !== fallback) {
                      event.currentTarget.src = fallback;
                    }
                  }}
                />
              </div>
            </div>
          </Show>
        </main>
        <footer>
          <span>
            {isImage()
              ? CONTENT.chat_page.community_page.zoom_hint
              : documentState() === "error"
                ? CONTENT.chat_page.community_page.preview_na_hint
                : CONTENT.chat_page.community_page.sandboxed}
          </span>
          <Show when={isImage()}>
            <div class="public-community-zoom-controls" aria-label={CONTENT.chat_page.community_page.image_zoom}>
              <button type="button" aria-label={CONTENT.chat_page.community_page.zoom_out} disabled={zoom() <= 1} onClick={() => changeZoom(-1)}>−</button>
              <output aria-live="polite">{Math.round(zoom() * 100)}%</output>
              <button type="button" aria-label={CONTENT.chat_page.community_page.zoom_in} disabled={zoom() >= 3} onClick={() => changeZoom(1)}>+</button>
              <button type="button" disabled={zoom() === 1} onClick={fitPreview}>{CONTENT.chat_page.community_page.fit_screen}</button>
            </div>
          </Show>
          <button type="button" class="public-community-download" disabled={downloadState() === "working"} onClick={() => void download()}>
            {downloadState() === "working" ? CONTENT.chat_page.community_page.downloading : CONTENT.chat_page.community_page.download}
          </button>
          <Show when={downloadState() === "error"}>
            <small role="alert">{CONTENT.chat_page.community_page.download_failed}</small>
          </Show>
        </footer>
      </div>
    </Portal>
  );
}

export default function PublicCommunityPage() {
  // Reopening the section repaints the previous page immediately and
  // revalidates behind it, rather than showing a loading line over content
  // that was already good enough to read.
  const restored = cachedCommunityFeed() as PublicCommunityContent[] | null;
  const [state, setState] = createSignal<ViewState>(
    restored && restored.length > 0 ? "ready" : "loading",
  );
  const [items, setItems] = createSignal<PublicCommunityContent[]>(restored ?? []);
  const [nextBefore, setNextBefore] = createSignal<number | null>(null);
  const [loadingMore, setLoadingMore] = createSignal(false);
  const [refreshing, setRefreshing] = createSignal(false);
  const [error, setError] = createSignal("");
  const [loadMoreError, setLoadMoreError] = createSignal("");
  const [preview, setPreview] = createSignal<PublicCommunityResource | null>(null);
  const [downloadingResourceId, setDownloadingResourceId] = createSignal<number | null>(null);
  const [downloadError, setDownloadError] = createSignal("");
  const [query, setQuery] = createSignal("");
  const [communityView, setCommunityView] = createSignal<CommunityView>("official");
  let requestController: AbortController | undefined;
  let hasLoaded = false;
  let lastRefreshAt = 0;
  let lastSeenId: number | undefined;
  const filteredItems = createMemo(() => {
    const normalized = query().trim().toLowerCase();
    if (!normalized) return items();
    return items().filter((item) =>
      `${item.author_name} ${item.body_text}`.toLowerCase().includes(normalized),
    );
  });

  const load = async (more = false) => {
    if (requestController || (more && !nextBefore())) return;
    const controller = new AbortController();
    requestController = controller;
    if (more) {
      setLoadingMore(true);
      setLoadMoreError("");
    } else {
      setRefreshing(true);
      // Only blank the list when there is nothing worth showing yet.
      if (items().length === 0) setState("loading");
      setError("");
    }
    try {
      const page = await getPublicCommunity({
        before: more ? nextBefore() ?? undefined : undefined,
        signal: controller.signal,
      });
      if (controller.signal.aborted) return;
      const merged = mergeCommunityTimeline(
        hasLoaded ? items() : [], nextBefore(), page, more,
      );
      setItems(merged.items);
      setNextBefore(merged.nextBefore);
      hasLoaded = true;
      if (!more) {
        setCachedCommunityFeed(page.items);
        lastRefreshAt = Date.now();
      }
      setState("ready");
      if (!more && page.items[0] && page.items[0].content_id !== lastSeenId) {
        const latestId = page.items[0].content_id;
        void markPublicCommunitySeen(latestId)
          .then(() => { lastSeenId = latestId; })
          .catch(() => { /* A read receipt must not break the timeline. */ });
      }
    } catch (cause) {
      if (controller.signal.aborted) return;
      if (isUnauthorizedApiError(cause)) {
        // A signed-out visitor must not keep reading a cached feed.
        setCachedCommunityFeed(null);
        setItems([]);
        setState("login");
      } else if (more) {
        setLoadMoreError(cause instanceof Error ? cause.message : CONTENT.chat_page.community_page.older_failed);
      } else {
        setError(cause instanceof Error ? cause.message : CONTENT.chat_page.community_page.load_failed);
        // A failed refresh over a list already on screen is not worth
        // replacing that list with an error page.
        if (items().length === 0) setState("error");
      }
    } finally {
      if (requestController === controller) requestController = undefined;
      setLoadingMore(false);
      setRefreshing(false);
    }
  };

  const download = async (resource: PublicCommunityResource) => {
    if (downloadingResourceId() !== null) return;
    setDownloadingResourceId(resource.resource_id);
    setDownloadError("");
    try {
      await downloadCommunityResource(resource);
    } catch (cause) {
      setDownloadError(cause instanceof Error ? cause.message : CONTENT.chat_page.community_page.resource_failed);
    } finally {
      setDownloadingResourceId(null);
    }
  };

  onMount(() => {
    void load();
    const refreshIfVisible = () => {
      if (
        document.visibilityState === "hidden" ||
        communityView() !== "official" ||
        state() === "login" ||
        preview() ||
        Date.now() - lastRefreshAt < 30_000
      ) return;
      void load();
    };
    const timer = window.setInterval(refreshIfVisible, 60_000);
    document.addEventListener("visibilitychange", refreshIfVisible);
    window.addEventListener("focus", refreshIfVisible);
    window.addEventListener("online", refreshIfVisible);
    onCleanup(() => {
      window.clearInterval(timer);
      document.removeEventListener("visibilitychange", refreshIfVisible);
      window.removeEventListener("focus", refreshIfVisible);
      window.removeEventListener("online", refreshIfVisible);
      requestController?.abort();
    });
  });

  return (
    <div class="hone-landing-v4 public-community-page">
      <Title>{CONTENT.chat_page.community_page.official}</Title>
      <Show
        when={state() !== "login"}
        fallback={
          <PublicLoginForm
            title={CONTENT.chat_page.community_page.login_title}
            subtitle={CONTENT.chat_page.community_page.login_hint}
            onLogin={() => void load()}
          />
        }
      >
        <PublicWorkspaceShell
          active="insights"
          communityUnread={false}
          searchPlaceholder={CONTENT.chat_page.community_page.search}
          onSearch={setQuery}
        >
          <div class="public-workspace-inner">
            <header class="public-workspace-page-heading">
              <div>
                <span class="public-workspace-eyebrow">{CONTENT.chat_page.community_page.eyebrow}</span>
                <h1>{CONTENT.chat_page.community_page.title}</h1>
                <p>{CONTENT.chat_page.community_page.subtitle}</p>
              </div>
            </header>
            <main class="public-community-shell">

          <nav class="public-community-view-tabs" aria-label="社区内容分类">
            <button type="button" classList={{ active: communityView() === "official" }} onClick={() => setCommunityView("official")}><strong>官方动态</strong><span>经过 HONE 发布的只读资料</span></button>
            <button type="button" classList={{ active: communityView() === "forum" }} onClick={() => setCommunityView("forum")}><strong>讨论区</strong><span>用户观点 · 未经官方核验</span></button>
          </nav>

          <Show when={communityView() === "official"}>
          <div class="public-community-refresh">
            <Show when={error() && items().length > 0}>
              <span role="alert">{CONTENT.chat_page.community_page.refresh_failed}</span>
            </Show>
            <button
              type="button"
              disabled={refreshing() || loadingMore()}
              onClick={() => void load()}
            >
              {refreshing() ? CONTENT.chat_page.community_page.refreshing : CONTENT.chat_page.community_page.refresh}
            </button>
          </div>
          <Switch>
            <Match when={state() === "loading"}>
              <div class="public-workspace-state" role="status">{CONTENT.chat_page.community_page.loading}</div>
            </Match>
            <Match when={state() === "error"}>
              <div class="public-workspace-state is-error" role="alert">
                <p>{error()}</p>
                <button type="button" onClick={() => void load()}>{CONTENT.chat_page.community_page.reload}</button>
              </div>
            </Match>
            <Match when={state() === "ready"}>
              <section class="public-community-timeline" aria-label={CONTENT.chat_page.community_page.feed_title}>
                <Show when={filteredItems().length > 0} fallback={<div class="public-workspace-state">{CONTENT.chat_page.community_page.no_match}</div>}>
                  <For each={filteredItems()}>
                    {(item) => {
                      const images = item.resources.filter(resourceIsImage);
                      const files = item.resources.filter((resource) => !resourceIsImage(resource));
                      return (
                        <article class="public-community-card">
                          <header>
                            <div class="public-community-avatar" aria-hidden="true">H</div>
                            <div>
                              <strong>{item.author_name}</strong>
                              <time dateTime={item.published_at ?? undefined}>{formatPublishedAt(item)}</time>
                            </div>
                            <em>{CONTENT.chat_page.community_page.read_only}</em>
                          </header>
                          <Show when={item.body_text.trim()}>
                            <p class="public-community-body">{item.body_text}</p>
                          </Show>
                          <Show when={images.length > 0}>
                            <div
                              class="public-community-image-grid"
                              classList={{ "is-single": images.length === 1 }}
                            >
                              <For each={images}>
                                {(resource) => (
                                  <button
                                    type="button"
                                    class="public-community-image"
                                    disabled={!resourceCanInlinePreview(resource)}
                                    aria-label={CONTENT.chat_page.community_page.preview_label.replace(
                                      "{name}",
                                      resource.display_name ||
                                        CONTENT.chat_page.community_page.image,
                                    )}
                                    onClick={() => setPreview(resource)}
                                  >
                                    <Show
                                      when={resourceCanInlinePreview(resource)}
                                      fallback={<span>{CONTENT.chat_page.community_page.image_protected}</span>}
                                    >
                                      <img
                                        src={publicCommunityResourceUrl(
                                          resource.resource_id,
                                          resource.version,
                                          resource.delivery_path,
                                        )}
                                        alt={resource.display_name || CONTENT.chat_page.community_page.image}
                                        loading="lazy"
                                        onError={(event) => {
                                          const fallback = publicCommunityResourceUrl(
                                            resource.resource_id,
                                            resource.version,
                                          );
                                          if (event.currentTarget.src !== fallback) {
                                            event.currentTarget.src = fallback;
                                          }
                                        }}
                                      />
                                    </Show>
                                  </button>
                                )}
                              </For>
                            </div>
                          </Show>
                          <Show when={files.length > 0}>
                            <div class="public-community-files">
                              <For each={files}>
                                {(resource) => {
                                  const stored = resourceIsStored(resource);
                                  const previewable = resourceCanInlinePreview(resource);
                                  const working = () => downloadingResourceId() === resource.resource_id;
                                  return (
                                    <button
                                      type="button"
                                      class="public-community-file"
                                      classList={{ "is-protected": !stored }}
                                      disabled={!stored || working()}
                                      onClick={() => previewable ? setPreview(resource) : void download(resource)}
                                    >
                                      <span aria-hidden="true">{stored ? "▧" : "⌁"}</span>
                                      <span>
                                        <strong>{resource.display_name || CONTENT.chat_page.community_page.community_file}</strong>
                                        <small>
                                          {!stored
                                            ? CONTENT.chat_page.community_page.meta_only
                                            : working()
                                              ? CONTENT.chat_page.community_page.downloading
                                              : previewable
                                                ? CONTENT.chat_page.community_page.click_preview
                                                : CONTENT.chat_page.community_page.click_download}
                                        </small>
                                      </span>
                                    </button>
                                  );
                                }}
                              </For>
                            </div>
                          </Show>
                          <Show when={item.crawl_status === "partial"}>
                            <small class="public-community-note">{CONTENT.chat_page.community_page.collapsed_note}</small>
                          </Show>
                        </article>
                      );
                    }}
                  </For>
                  <Show when={downloadError()}>
                    <p class="public-community-inline-error" role="alert">{downloadError()}</p>
                  </Show>
                  <Show when={loadMoreError()}>
                    <p class="public-community-inline-error" role="alert">{loadMoreError()}</p>
                  </Show>
                  <Show when={nextBefore()}>
                    <button
                      type="button"
                      class="public-community-more"
                      disabled={loadingMore() || refreshing()}
                      onClick={() => void load(true)}
                    >
                      {loadingMore() ? CONTENT.chat_page.community_page.loading_short : loadMoreError() ? CONTENT.chat_page.community_page.retry_older : CONTENT.chat_page.community_page.load_older}
                    </button>
                  </Show>
                </Show>
              </section>
            </Match>
          </Switch>
          </Show>
          <Show when={communityView() === "forum"}>
            <CommunityForum query={query()} />
          </Show>
            </main>
            <p class="public-workspace-disclaimer">{CONTENT.chat_page.community_page.disclaimer}</p>
          </div>
        </PublicWorkspaceShell>
      </Show>
      <Show when={preview()}>
        <CommunityMediaPreview resource={preview()!} onClose={() => setPreview(null)} />
      </Show>
    </div>
  );
}
