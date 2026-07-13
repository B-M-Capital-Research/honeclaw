import { Title } from "@solidjs/meta";
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
  if (!raw) return "刚刚";
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

async function downloadCommunityResource(resource: PublicCommunityResource) {
  const blob = await getPublicCommunityResourceBlob(resource.resource_id, resource.version);
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
  const source = () =>
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
      await downloadCommunityResource(props.resource);
      setDownloadState("idle");
    } catch {
      setDownloadState("error");
    }
  };

  onMount(() => {
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
            <small>HONE 官方社区</small>
            <strong id={titleId}>{props.resource.display_name || "社区资源"}</strong>
          </div>
          <button
            ref={(element) => { closeButtonEl = element; }}
            type="button"
            onClick={props.onClose}
            aria-label="关闭预览"
          >
            ×
          </button>
        </header>
        <main classList={{ "is-image": isImage() }}>
          <Show
            when={isImage()}
            fallback={
              <iframe
                title={props.resource.display_name || "社区文件预览"}
                src={source()}
                sandbox="allow-downloads"
                referrerPolicy="no-referrer"
              />
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
                  alt={props.resource.display_name || "社区图片"}
                  onLoad={fitImageToViewport}
                />
              </div>
            </div>
          </Show>
        </main>
        <footer>
          <span>
            {isImage()
              ? "双指或滚轮缩放，放大后可拖动"
              : "PDF 在安全沙箱中预览"}
          </span>
          <Show when={isImage()}>
            <div class="public-community-zoom-controls" aria-label="图片缩放">
              <button type="button" aria-label="缩小" disabled={zoom() <= 1} onClick={() => changeZoom(-1)}>−</button>
              <output aria-live="polite">{Math.round(zoom() * 100)}%</output>
              <button type="button" aria-label="放大" disabled={zoom() >= 3} onClick={() => changeZoom(1)}>+</button>
              <button type="button" disabled={zoom() === 1} onClick={fitPreview}>适应屏幕</button>
            </div>
          </Show>
          <button type="button" class="public-community-download" disabled={downloadState() === "working"} onClick={() => void download()}>
            {downloadState() === "working" ? "正在下载…" : "下载资源"}
          </button>
          <Show when={downloadState() === "error"}>
            <small role="alert">下载失败，请重试</small>
          </Show>
        </footer>
      </div>
    </Portal>
  );
}

export default function PublicCommunityPage() {
  const [state, setState] = createSignal<ViewState>("loading");
  const [items, setItems] = createSignal<PublicCommunityContent[]>([]);
  const [nextBefore, setNextBefore] = createSignal<number | null>(null);
  const [loadingMore, setLoadingMore] = createSignal(false);
  const [error, setError] = createSignal("");
  const [loadMoreError, setLoadMoreError] = createSignal("");
  const [preview, setPreview] = createSignal<PublicCommunityResource | null>(null);
  const [downloadingResourceId, setDownloadingResourceId] = createSignal<number | null>(null);
  const [downloadError, setDownloadError] = createSignal("");
  const [query, setQuery] = createSignal("");
  const filteredItems = createMemo(() => {
    const normalized = query().trim().toLowerCase();
    if (!normalized) return items();
    return items().filter((item) =>
      `${item.author_name} ${item.body_text}`.toLowerCase().includes(normalized),
    );
  });

  const load = async (more = false) => {
    if (more) {
      setLoadingMore(true);
      setLoadMoreError("");
    } else {
      setState("loading");
      setError("");
    }
    try {
      const page = await getPublicCommunity({
        before: more ? nextBefore() ?? undefined : undefined,
      });
      setItems((current) => (more ? [...current, ...page.items] : page.items));
      setNextBefore(page.next_before ?? null);
      setState("ready");
      if (!more && page.items[0]) {
        void markPublicCommunitySeen(page.items[0].content_id);
      }
    } catch (cause) {
      if (isUnauthorizedApiError(cause)) {
        setState("login");
      } else if (more) {
        setLoadMoreError(cause instanceof Error ? cause.message : "更早动态加载失败");
      } else {
        setError(cause instanceof Error ? cause.message : "社区内容暂时无法加载");
        setState("error");
      }
    } finally {
      setLoadingMore(false);
    }
  };

  const download = async (resource: PublicCommunityResource) => {
    if (downloadingResourceId() !== null) return;
    setDownloadingResourceId(resource.resource_id);
    setDownloadError("");
    try {
      await downloadCommunityResource(resource);
    } catch (cause) {
      setDownloadError(cause instanceof Error ? cause.message : "资源下载失败");
    } finally {
      setDownloadingResourceId(null);
    }
  };

  onMount(() => void load());

  return (
    <div class="hone-landing-v4 public-community-page">
      <Title>HONE 官方社区</Title>
      <Show
        when={state() !== "login"}
        fallback={
          <PublicLoginForm
            title="登录后查看 HONE 社区"
            subtitle="社区当前为只读，内容仅向已登录用户展示。"
            onLogin={() => void load()}
          />
        }
      >
        <PublicWorkspaceShell
          active="insights"
          communityUnread={false}
          searchPlaceholder="搜索洞察、公司或主题"
          onSearch={setQuery}
        >
          <div class="public-workspace-inner">
            <header class="public-workspace-page-heading">
              <div>
                <span class="public-workspace-eyebrow">社区研究 · 只读</span>
                <h1>洞察</h1>
                <p>来自 HONE 社区的研究判断、市场观察与关键资料，按发生时间连续沉淀。</p>
              </div>
              <div class="public-insights-filter" aria-label="洞察筛选">
                <button type="button" class="is-active">最新</button>
                <button type="button">持仓相关</button>
                <button type="button">已保存</button>
              </div>
            </header>
            <main class="public-community-shell">

          <Switch>
            <Match when={state() === "loading"}>
              <div class="public-workspace-state" role="status">正在加载洞察…</div>
            </Match>
            <Match when={state() === "error"}>
              <div class="public-workspace-state is-error" role="alert">
                <p>{error()}</p>
                <button type="button" onClick={() => void load()}>重新加载</button>
              </div>
            </Match>
            <Match when={state() === "ready"}>
              <section class="public-community-timeline" aria-label="HONE 官方社区动态">
                <Show when={filteredItems().length > 0} fallback={<div class="public-workspace-state">没有匹配的洞察。</div>}>
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
                            <em>只读</em>
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
                                    aria-label={`预览${resource.display_name || "社区图片"}`}
                                    onClick={() => setPreview(resource)}
                                  >
                                    <Show
                                      when={resourceCanInlinePreview(resource)}
                                      fallback={<span>图片受来源保护</span>}
                                    >
                                      <img
                                        src={publicCommunityResourceUrl(
                                          resource.resource_id,
                                          resource.version,
                                        )}
                                        alt={resource.display_name || "社区图片"}
                                        loading="lazy"
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
                                        <strong>{resource.display_name || "社区文件"}</strong>
                                        <small>
                                          {!stored
                                            ? "受来源保护，仅保留元数据"
                                            : working()
                                              ? "正在下载…"
                                              : previewable
                                                ? "点击安全预览"
                                                : "点击下载"}
                                        </small>
                                      </span>
                                    </button>
                                  );
                                }}
                              </For>
                            </div>
                          </Show>
                          <Show when={item.crawl_status === "partial"}>
                            <small class="public-community-note">该长文在来源页展示为折叠摘要。</small>
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
                      disabled={loadingMore()}
                      onClick={() => void load(true)}
                    >
                      {loadingMore() ? "正在加载…" : loadMoreError() ? "重试加载更早动态" : "加载更早动态"}
                    </button>
                  </Show>
                </Show>
              </section>
            </Match>
          </Switch>
            </main>
          </div>
        </PublicWorkspaceShell>
      </Show>
      <Show when={preview()}>
        <CommunityMediaPreview resource={preview()!} onClose={() => setPreview(null)} />
      </Show>
    </div>
  );
}
