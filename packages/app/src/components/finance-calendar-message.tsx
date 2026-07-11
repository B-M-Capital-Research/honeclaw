import { createMemo, createSignal, onCleanup, onMount, Show } from "solid-js";
import { Portal } from "solid-js/web";

import { getPublicFinanceCalendar } from "@/lib/api";
import { renderFinanceCalendarMobilePng } from "@/lib/finance-calendar-mobile-renderer";
import { CONTENT } from "@/lib/public-content";
import {
  clampFinanceCalendarPan,
  financeCalendarAnchoredTransform,
  financeCalendarPinchZoom,
  selectFinanceCalendarImageSource,
  shouldUpgradeFinanceCalendarMobileSource,
  stepFinanceCalendarZoom,
} from "@/lib/finance-calendar";

type ShareNavigator = Navigator & {
  canShare?: (data: ShareData) => boolean;
  share?: (data: ShareData) => Promise<void>;
};

export function FinanceCalendarMessageImage(props: {
  src: string;
  mobileSrc?: string;
  month: string;
}) {
  const [loaded, setLoaded] = createSignal(false);
  const [failed, setFailed] = createSignal(false);
  const [retry, setRetry] = createSignal(0);
  const [open, setOpen] = createSignal(false);
  const [zoom, setZoom] = createSignal(1);
  const [pan, setPan] = createSignal({ x: 0, y: 0 });
  const [fitSize, setFitSize] = createSignal({ width: 0, height: 0 });
  const [interacting, setInteracting] = createSignal(false);
  const [preferMobile, setPreferMobile] = createSignal(
    typeof window !== "undefined" && window.matchMedia("(max-width: 768px)").matches,
  );
  const [legacyMobileSrc, setLegacyMobileSrc] = createSignal<string>();
  const [working, setWorking] = createSignal<"save" | "share" | null>(null);
  const [actionError, setActionError] = createSignal(false);
  const mobileSource = createMemo(() => legacyMobileSrc() ?? props.mobileSrc);
  const needsMobileDesignUpgrade = () =>
    shouldUpgradeFinanceCalendarMobileSource(props.mobileSrc);
  const selectedSource = createMemo(() =>
    selectFinanceCalendarImageSource(
      props.src,
      mobileSource(),
      preferMobile(),
    ),
  );
  const source = createMemo(() => {
    const selected = selectedSource();
    if (/^(?:blob:|data:)/.test(selected)) return selected;
    const join = selected.includes("?") ? "&" : "?";
    return `${selected}${join}calendar_retry=${retry()}`;
  });
  const fileName = () =>
    `HONE-finance-calendar-${props.month}${preferMobile() && mobileSource() ? "-mobile" : ""}.png`;
  const zoomLabel = () => `${Math.round(zoom() * 100)}%`;
  let cachedBlob: Blob | undefined;
  let cachedSource = "";
  let messageEl: HTMLElement | undefined;
  let viewportEl: HTMLDivElement | undefined;
  let imageEl: HTMLImageElement | undefined;
  let legacyBuildStarted = false;
  let legacyObjectUrl: string | undefined;
  let removeViewportGestures: (() => void) | undefined;
  let viewportResizeObserver: ResizeObserver | undefined;
  let viewFrame = 0;
  let pendingView: { zoom: number; x: number; y: number } | undefined;

  const buildLegacyMobileImage = async () => {
    if (
      legacyBuildStarted ||
      !needsMobileDesignUpgrade() ||
      !preferMobile()
    ) {
      return;
    }
    legacyBuildStarted = true;
    try {
      const payload = await getPublicFinanceCalendar(props.month);
      await (
        document as Document & { fonts?: { ready: Promise<unknown> } }
      ).fonts?.ready.catch(() => undefined);
      const blob = await renderFinanceCalendarMobilePng(payload);
      legacyObjectUrl = URL.createObjectURL(blob);
      setLoaded(false);
      setFailed(false);
      setLegacyMobileSrc(legacyObjectUrl);
    } catch {
      legacyBuildStarted = false;
    }
  };

  onMount(() => {
    const media = window.matchMedia("(max-width: 768px)");
    const sync = () => {
      setPreferMobile(media.matches);
      if (media.matches && messageEl) {
        const rect = messageEl.getBoundingClientRect();
        if (rect.bottom >= -160 && rect.top <= window.innerHeight + 160) {
          void buildLegacyMobileImage();
        }
      }
    };
    sync();
    media.addEventListener?.("change", sync);
    const observer =
      needsMobileDesignUpgrade() && "IntersectionObserver" in window && messageEl
        ? new IntersectionObserver(
            (entries) => {
              if (
                media.matches &&
                entries.some((entry) => entry.isIntersecting)
              ) {
                void buildLegacyMobileImage();
                observer?.disconnect();
              }
            },
            { rootMargin: "160px" },
          )
        : undefined;
    if (observer && messageEl) observer.observe(messageEl);
    if (!observer && media.matches) void buildLegacyMobileImage();
    onCleanup(() => {
      media.removeEventListener?.("change", sync);
      observer?.disconnect();
      if (legacyObjectUrl) URL.revokeObjectURL(legacyObjectUrl);
    });
  });

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
  const fitImageToViewport = () => {
    if (!viewportEl || !imageEl || !imageEl.naturalWidth || !imageEl.naturalHeight) {
      return;
    }
    const availableWidth = viewportEl.clientWidth;
    const availableHeight = viewportEl.clientHeight;
    const scale = Math.min(
      availableWidth / imageEl.naturalWidth,
      availableHeight / imageEl.naturalHeight,
    );
    setFitSize({
      width: Math.max(1, Math.floor(imageEl.naturalWidth * scale)),
      height: Math.max(1, Math.floor(imageEl.naturalHeight * scale)),
    });
    commitView(1, 0, 0);
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
  const changeZoom = (direction: -1 | 1) => {
    setInteracting(false);
    commitView(stepFinanceCalendarZoom(zoom(), direction), pan().x, pan().y);
  };
  const fitPreview = () => {
    setInteracting(false);
    commitView(1, 0, 0);
  };

  const bindViewport = (element: HTMLDivElement) => {
    removeViewportGestures?.();
    viewportResizeObserver?.disconnect();
    viewportEl = element;
    viewportResizeObserver = new ResizeObserver(() => fitImageToViewport());
    viewportResizeObserver.observe(element);
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
    let drag:
      | { startX: number; startY: number; x: number; y: number }
      | undefined;
    const touchMetrics = (event: TouchEvent) => {
      const first = event.touches.item(0);
      const second = event.touches.item(1);
      if (!first || !second) return null;
      const rect = element.getBoundingClientRect();
      return {
        distance: Math.hypot(
          second.clientX - first.clientX,
          second.clientY - first.clientY,
        ),
        centerX: (first.clientX + second.clientX) / 2 - rect.left,
        centerY: (first.clientY + second.clientY) / 2 - rect.top,
      };
    };
    const beginDrag = (touch: Touch) => {
      drag = {
        startX: touch.clientX,
        startY: touch.clientY,
        x: pan().x,
        y: pan().y,
      };
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
      if (touch) beginDrag(touch);
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
      const deltaX = touch.clientX - drag.startX;
      const deltaY = touch.clientY - drag.startY;
      if (Math.hypot(deltaX, deltaY) < 4) return;
      event.preventDefault();
      setInteracting(true);
      commitView(zoom(), drag.x + deltaX, drag.y + deltaY);
    };
    const onTouchEnd = (event: TouchEvent) => {
      if (event.touches.length >= 2) return;
      pinch = undefined;
      const remaining = event.touches.item(0);
      if (remaining) {
        beginDrag(remaining);
      } else {
        drag = undefined;
        setInteracting(false);
      }
    };
    element.addEventListener("touchstart", onTouchStart, { passive: true });
    element.addEventListener("touchmove", onTouchMove, { passive: false });
    element.addEventListener("touchend", onTouchEnd, { passive: true });
    element.addEventListener("touchcancel", onTouchEnd, { passive: true });
    removeViewportGestures = () => {
      viewportResizeObserver?.disconnect();
      element.removeEventListener("touchstart", onTouchStart);
      element.removeEventListener("touchmove", onTouchMove);
      element.removeEventListener("touchend", onTouchEnd);
      element.removeEventListener("touchcancel", onTouchEnd);
    };
  };

  const close = () => {
    setOpen(false);
    setZoom(1);
    setPan({ x: 0, y: 0 });
    setInteracting(false);
  };
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key !== "Escape") return;
    document.removeEventListener("keydown", onKeyDown);
    close();
  };
  const openPreview = () => {
    if (!loaded()) return;
    void buildLegacyMobileImage();
    setOpen(true);
    document.addEventListener("keydown", onKeyDown);
  };
  const closePreview = () => {
    document.removeEventListener("keydown", onKeyDown);
    close();
  };
  onCleanup(() => {
    document.removeEventListener("keydown", onKeyDown);
    removeViewportGestures?.();
    viewportResizeObserver?.disconnect();
    if (viewFrame) cancelAnimationFrame(viewFrame);
  });

  const retryImage = () => {
    cachedBlob = undefined;
    setFailed(false);
    setLoaded(false);
    setRetry((value) => value + 1);
  };
  const loadBlob = async () => {
    if (cachedBlob && cachedSource === source()) return cachedBlob;
    const response = await fetch(source(), { credentials: "include" });
    if (!response.ok) throw new Error(`calendar image ${response.status}`);
    cachedBlob = await response.blob();
    cachedSource = source();
    return cachedBlob;
  };
  const downloadBlob = (blob: Blob) => {
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = fileName();
    anchor.rel = "noopener";
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1_000);
  };
  const saveImage = async () => {
    if (working()) return;
    setWorking("save");
    setActionError(false);
    try {
      downloadBlob(await loadBlob());
    } catch {
      setActionError(true);
    } finally {
      setWorking(null);
    }
  };
  const shareImage = async () => {
    if (working()) return;
    setWorking("share");
    setActionError(false);
    try {
      const blob = await loadBlob();
      const file = new File([blob], fileName(), { type: blob.type || "image/png" });
      const shareNavigator = navigator as ShareNavigator;
      const data: ShareData = {
        title: `HONE ${props.month}`,
        text: CONTENT.chat_page.composer.finance_calendar_share_text,
        files: [file],
      };
      if (shareNavigator.share && (!shareNavigator.canShare || shareNavigator.canShare(data))) {
        await shareNavigator.share(data);
      } else {
        downloadBlob(blob);
      }
    } catch (error) {
      if ((error as DOMException)?.name !== "AbortError") setActionError(true);
    } finally {
      setWorking(null);
    }
  };

  return (
    <>
      <section
        class="public-finance-calendar-message"
        ref={(element) => {
          messageEl = element;
        }}
      >
        <button
          type="button"
          class="public-finance-calendar-preview"
          aria-label={CONTENT.chat_page.composer.finance_calendar_preview_open}
          onClick={openPreview}
          disabled={!loaded()}
        >
          <div
            class="public-finance-calendar-image-frame"
            classList={{ "is-loaded": loaded() }}
          >
            <img
              src={source()}
              alt={CONTENT.chat_page.composer.finance_calendar_preview_aria}
              onLoad={() => {
                setLoaded(true);
                setFailed(false);
              }}
              onError={() => {
                setLoaded(false);
                setFailed(true);
              }}
            />
            <Show when={!loaded() && !failed()}>
              <div class="public-finance-calendar-loading" role="status">
                <span class="public-finance-calendar-loading-mark" aria-hidden="true">
                  {props.month.slice(-2)}
                </span>
                <strong>{CONTENT.chat_page.composer.finance_calendar_image_loading}</strong>
                <small>{CONTENT.chat_page.composer.finance_calendar_image_loading_hint}</small>
                <span class="public-finance-calendar-progress" aria-hidden="true"><i /></span>
              </div>
            </Show>
            <Show when={failed()}>
              <div class="public-finance-calendar-loading is-error" role="alert">
                <strong>{CONTENT.chat_page.composer.finance_calendar_image_failed}</strong>
                <small>{CONTENT.chat_page.composer.finance_calendar_image_failed_hint}</small>
              </div>
            </Show>
          </div>
        </button>
        <Show when={failed()}>
          <button type="button" class="public-finance-calendar-retry" onClick={retryImage}>
            {CONTENT.chat_page.composer.finance_calendar_image_retry}
          </button>
        </Show>
        <Show when={loaded()}>
          <div class="public-finance-calendar-actions">
            <button type="button" onClick={openPreview}>
              <CalendarActionIcon name="expand" />
              <span>{CONTENT.chat_page.composer.finance_calendar_preview_open}</span>
            </button>
            <button type="button" onClick={() => void saveImage()} disabled={working() !== null}>
              <CalendarActionIcon name="save" />
              <span>{working() === "save" ? CONTENT.chat_page.composer.finance_calendar_image_saving : CONTENT.chat_page.composer.finance_calendar_image_save}</span>
            </button>
            <button type="button" onClick={() => void shareImage()} disabled={working() !== null}>
              <CalendarActionIcon name="share" />
              <span>{CONTENT.chat_page.composer.finance_calendar_image_share}</span>
            </button>
          </div>
        </Show>
        <Show when={actionError()}>
          <p class="public-finance-calendar-action-error">
            {CONTENT.chat_page.composer.finance_calendar_image_action_failed}
          </p>
        </Show>
      </section>

      <Portal>
        <Show when={open()}>
          <div class="public-finance-calendar-lightbox" role="dialog" aria-modal="true" aria-label={CONTENT.chat_page.composer.finance_calendar_preview_aria}>
            <header>
              <div>
                <small>HONE · {props.month}</small>
                <strong>{CONTENT.chat_page.composer.finance_calendar_title}</strong>
              </div>
              <button type="button" aria-label={CONTENT.chat_page.composer.finance_calendar_preview_close} onClick={closePreview}>×</button>
            </header>
            <div
              class="public-finance-calendar-lightbox-viewport"
              ref={(element) => {
                bindViewport(element);
              }}
            >
              <div
                class="public-finance-calendar-lightbox-canvas"
              >
                <img
                  ref={(element) => {
                    imageEl = element;
                  }}
                  classList={{ "is-interacting": interacting() }}
                  style={{
                    width: fitSize().width ? `${fitSize().width}px` : undefined,
                    height: fitSize().height ? `${fitSize().height}px` : undefined,
                    transform: `translate3d(${pan().x}px, ${pan().y}px, 0) scale(${zoom()})`,
                  }}
                  src={source()}
                  alt={CONTENT.chat_page.composer.finance_calendar_preview_aria}
                  onLoad={fitImageToViewport}
                />
              </div>
            </div>
            <footer>
              <p>{CONTENT.chat_page.composer.finance_calendar_image_save_hint}</p>
              <div class="public-finance-calendar-zoom-bar" aria-label={CONTENT.chat_page.composer.finance_calendar_image_zoom}>
                <button
                  type="button"
                  aria-label={CONTENT.chat_page.composer.finance_calendar_zoom_out}
                  disabled={zoom() <= 1}
                  onClick={() => changeZoom(-1)}
                >
                  −
                </button>
                <output>{zoomLabel()}</output>
                <button
                  type="button"
                  aria-label={CONTENT.chat_page.composer.finance_calendar_zoom_in}
                  disabled={zoom() >= 3}
                  onClick={() => changeZoom(1)}
                >
                  +
                </button>
              </div>
              <div class="public-finance-calendar-lightbox-actions">
                <button type="button" onClick={fitPreview} disabled={zoom() === 1}>
                  <CalendarActionIcon name="expand" />
                  {CONTENT.chat_page.composer.finance_calendar_zoom_fit}
                </button>
                <button type="button" onClick={() => void saveImage()} disabled={working() !== null}>
                  <CalendarActionIcon name="save" />
                  {CONTENT.chat_page.composer.finance_calendar_image_save}
                </button>
                <button type="button" onClick={() => void shareImage()} disabled={working() !== null}>
                  <CalendarActionIcon name="share" />
                  {CONTENT.chat_page.composer.finance_calendar_image_share}
                </button>
              </div>
              <Show when={actionError()}>
                <p class="public-finance-calendar-action-error">
                  {CONTENT.chat_page.composer.finance_calendar_image_action_failed}
                </p>
              </Show>
            </footer>
          </div>
        </Show>
      </Portal>
    </>
  );
}

function CalendarActionIcon(props: { name: "expand" | "save" | "share" }) {
  if (props.name === "save") {
    return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3v12m0 0 4-4m-4 4-4-4M5 20h14" /></svg>;
  }
  if (props.name === "share") {
    return <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="18" cy="5" r="2.5" /><circle cx="6" cy="12" r="2.5" /><circle cx="18" cy="19" r="2.5" /><path d="m8.2 10.8 7.5-4.4M8.2 13.2l7.5 4.4" /></svg>;
  }
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 3H3v5M16 3h5v5M8 21H3v-5m13 5h5v-5" /></svg>;
}
