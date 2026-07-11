import type { JSX } from "solid-js";

export function HoneBrand(props: {
  class?: string;
  dark?: boolean;
  markOnly?: boolean;
}): JSX.Element {
  return (
    <span
      class={`hone-brand${props.dark ? " hone-brand--dark" : ""}${props.markOnly ? " hone-brand--mark-only" : ""}${props.class ? ` ${props.class}` : ""}`}
      aria-label="HONE"
    >
      <span class="hone-brand-mark" aria-hidden="true">
        <img src="/hone-mark.svg" alt="" />
      </span>
      {!props.markOnly ? <span class="hone-brand-word">HONE</span> : null}
    </span>
  );
}
