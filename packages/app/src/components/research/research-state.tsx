import { Show } from "solid-js";
import "./research.css";

/**
 * Shared loading / error / empty presentation for research surfaces.
 * Thirteen bespoke `X-state` / `X-empty` / `X-loading` classes collapse into
 * one component so the copy, spacing and retry affordance stay consistent.
 */
export function ResearchState(props: {
  kind: "loading" | "error" | "empty";
  message: string;
  detail?: string;
  onRetry?: () => void;
  retryLabel?: string;
}) {
  return (
    <div class={`research-state is-${props.kind}`} role={props.kind === "error" ? "alert" : "status"}>
      <Show when={props.kind === "loading"}>
        <i class="research-state__spinner" aria-hidden="true" />
      </Show>
      <strong>{props.message}</strong>
      <Show when={props.detail}>
        <p>{props.detail}</p>
      </Show>
      <Show when={props.onRetry}>
        {(onRetry) => (
          <button type="button" onClick={() => onRetry()()}>
            {props.retryLabel ?? "重试"}
          </button>
        )}
      </Show>
    </div>
  );
}
