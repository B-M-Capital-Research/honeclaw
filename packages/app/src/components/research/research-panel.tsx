import { createEffect, onCleanup, type JSX, type ParentProps } from "solid-js";
import { Portal } from "solid-js/web";
import "./research.css";

/**
 * Shared modal shell for research detail panels.
 *
 * Owns three things the panels kept getting wrong on their own: Escape and
 * `aria-modal` semantics, a background scroll lock that also holds on iOS
 * Safari (where `body { overflow: hidden }` alone does not), and a single
 * scroll container that contains its overscroll so a touch drag never chains
 * into the page behind the sheet.
 *
 * On phones the panel is a full-screen sheet rather than a floating card:
 * chrome shrinks to a compact header, the body owns the remaining height, and
 * the action row sits on the safe-area inset.
 */
export function ResearchPanel(
  props: ParentProps<{
    onClose: () => void;
    labelledBy: string;
    /** Extra class on the dialog section, e.g. `daily-signal-dialog is-macro`. */
    dialogClass?: string;
    /** Extra class on the backdrop, defaults to the shared one. */
    backdropClass?: string;
  }>,
) {
  let dialogRef: HTMLElement | undefined;

  createEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") props.onClose();
    };
    document.addEventListener("keydown", onKeyDown);
    // iOS Safari ignores `overflow: hidden` on <body> for touch scrolling, so
    // the page is pinned in place and its offset restored on close instead.
    const { body } = document;
    const scrollY = window.scrollY;
    const previous = {
      overflow: body.style.overflow,
      position: body.style.position,
      top: body.style.top,
      width: body.style.width,
    };
    body.style.overflow = "hidden";
    body.style.position = "fixed";
    body.style.top = `-${scrollY}px`;
    body.style.width = "100%";
    dialogRef?.focus();
    onCleanup(() => {
      document.removeEventListener("keydown", onKeyDown);
      body.style.overflow = previous.overflow;
      body.style.position = previous.position;
      body.style.top = previous.top;
      body.style.width = previous.width;
      window.scrollTo(0, scrollY);
    });
  });

  return (
    <Portal>
      <div
        class={`research-panel-backdrop${props.backdropClass ? ` ${props.backdropClass}` : ""}`}
        onClick={() => props.onClose()}
      >
        <section
          ref={dialogRef}
          class={`research-panel${props.dialogClass ? ` ${props.dialogClass}` : ""}`}
          role="dialog"
          aria-modal="true"
          aria-labelledby={props.labelledBy}
          tabindex="-1"
          onClick={(event) => event.stopPropagation()}
        >
          {props.children}
        </section>
      </div>
    </Portal>
  );
}

/**
 * The one piece of chrome every panel shows first.
 *
 * Panels used to open with a title, a kicker, a description line, a metadata
 * strip and a tab bar before any actual finding — a third of a phone screen
 * spent before the answer. This keeps the verdict (`headline` + `signal`) and
 * one sentence adjacent to the title, and pushes provenance into a collapsed
 * `meta` line that reads as secondary.
 */
export function ResearchPanelHead(props: {
  id: string;
  kicker: string;
  title: string;
  /** Headline number or verdict, e.g. `61.6` or `52 家`. */
  headline?: string;
  /** Traffic-light key (`green` | `yellow` | `orange` | `red`) when it has one. */
  signal?: string;
  signalLabel?: string;
  /** One sentence. Not a paragraph — the body owns detail. */
  summary?: string;
  /** Provenance line: report date, coverage, model version. */
  meta?: string;
  onClose: () => void;
  /** Optional trailing control, e.g. a refresh button. */
  action?: JSX.Element;
}) {
  return (
    <header class={`research-panel__head${props.signal ? ` is-${props.signal}` : ""}`}>
      <div class="research-panel__head-top">
        <p class="research-panel__kicker">{props.kicker}</p>
        <button
          type="button"
          class="research-panel__close"
          aria-label="关闭"
          onClick={() => props.onClose()}
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M18 6 6 18M6 6l12 12" />
          </svg>
        </button>
      </div>
      <div class="research-panel__headline">
        <h2 id={props.id}>{props.title}</h2>
        {props.headline ? <b>{props.headline}</b> : null}
        {props.signalLabel ? <span class="research-panel__signal">{props.signalLabel}</span> : null}
      </div>
      {props.summary ? <p class="research-panel__summary">{props.summary}</p> : null}
      {props.meta ? <p class="research-panel__meta">{props.meta}</p> : null}
      {props.action ? <div class="research-panel__head-action">{props.action}</div> : null}
    </header>
  );
}
