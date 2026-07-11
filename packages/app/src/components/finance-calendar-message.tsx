import { createMemo, createSignal, onCleanup, Show } from "solid-js";
import { Portal } from "solid-js/web";

import { CONTENT } from "@/lib/public-content";
import { stepFinanceCalendarZoom } from "@/lib/finance-calendar";

type ShareNavigator = Navigator & {
  canShare?: (data: ShareData) => boolean;
  share?: (data: ShareData) => Promise<void>;
};

export function FinanceCalendarMessageImage(props: {
  src: string;
  month: string;
}) {
  const [loaded, setLoaded] = createSignal(false);
  const [failed, setFailed] = createSignal(false);
  const [retry, setRetry] = createSignal(0);
  const [open, setOpen] = createSignal(false);
  const [zoom, setZoom] = createSignal(1);
  const [working, setWorking] = createSignal<"save" | "share" | null>(null);
  const [actionError, setActionError] = createSignal(false);
  const source = createMemo(() => {
    const join = props.src.includes("?") ? "&" : "?";
    return `${props.src}${join}calendar_retry=${retry()}`;
  });
  const fileName = () => `HONE-finance-calendar-${props.month}.png`;
  const zoomLabel = () =>
    zoom() === 1
      ? CONTENT.chat_page.composer.finance_calendar_zoom_fit
      : `${Math.round(zoom() * 100)}%`;
  const zoomWidth = () => `min(${zoom() * 100}%, ${Math.round(900 * zoom())}px)`;
  let cachedBlob: Blob | undefined;
  let viewportEl: HTMLDivElement | undefined;

  const settleViewport = (fit = false) => {
    requestAnimationFrame(() => {
      if (!viewportEl) return;
      viewportEl.scrollLeft = fit
        ? 0
        : Math.max(0, (viewportEl.scrollWidth - viewportEl.clientWidth) / 2);
      if (fit) viewportEl.scrollTop = 0;
    });
  };
  const changeZoom = (direction: -1 | 1) => {
    setZoom((value) => stepFinanceCalendarZoom(value, direction));
    settleViewport();
  };
  const fitPreview = () => {
    setZoom(1);
    settleViewport(true);
  };

  const close = () => {
    setOpen(false);
    setZoom(1);
  };
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key !== "Escape") return;
    document.removeEventListener("keydown", onKeyDown);
    close();
  };
  const openPreview = () => {
    if (!loaded()) return;
    setOpen(true);
    document.addEventListener("keydown", onKeyDown);
  };
  const closePreview = () => {
    document.removeEventListener("keydown", onKeyDown);
    close();
  };
  onCleanup(() => document.removeEventListener("keydown", onKeyDown));

  const retryImage = () => {
    cachedBlob = undefined;
    setFailed(false);
    setLoaded(false);
    setRetry((value) => value + 1);
  };
  const loadBlob = async () => {
    if (cachedBlob) return cachedBlob;
    const response = await fetch(source(), { credentials: "include" });
    if (!response.ok) throw new Error(`calendar image ${response.status}`);
    cachedBlob = await response.blob();
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
      <section class="public-finance-calendar-message">
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
                viewportEl = element;
              }}
            >
              <div
                class="public-finance-calendar-lightbox-canvas"
                style={{ width: zoomWidth() }}
              >
                <img src={source()} alt={CONTENT.chat_page.composer.finance_calendar_preview_aria} />
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
                  disabled={zoom() >= 2}
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
