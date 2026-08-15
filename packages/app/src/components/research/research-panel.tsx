import { createEffect, onCleanup, type ParentProps } from "solid-js";
import { Portal } from "solid-js/web";
import "./research.css";

/**
 * Shared modal shell for research detail panels.
 *
 * Every dashboard used to build its own backdrop/dialog pair, and they drifted:
 * three of them lost Escape handling, none locked body scroll, some dropped
 * `aria-modal`. The shell owns that behaviour once; callers keep their own
 * dialog class so existing panel styling continues to apply.
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
    // The page behind a modal must not scroll; restore whatever was there so
    // nested/successive panels do not clobber each other's state.
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    dialogRef?.focus();
    onCleanup(() => {
      document.removeEventListener("keydown", onKeyDown);
      document.body.style.overflow = previousOverflow;
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
