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
        <img class="hone-brand-img-legacy" src="/hone-mark.svg" alt="" />
        <svg class="hone-brand-svg-v7" viewBox="0 0 96 96" aria-hidden="true">
          <defs>
            <linearGradient id="hone-v7-plate" x1="0" y1="0" x2="96" y2="96" gradientUnits="userSpaceOnUse">
              <stop stop-color="#111a2e" />
              <stop offset="1" stop-color="#090d16" />
            </linearGradient>
            <linearGradient id="hone-v7-stone" x1="34" y1="21" x2="71" y2="57" gradientUnits="userSpaceOnUse">
              <stop stop-color="#60a5fa" />
              <stop offset="1" stop-color="#2563eb" />
            </linearGradient>
            <linearGradient id="hone-v7-blade" x1="28" y1="40" x2="71" y2="68" gradientUnits="userSpaceOnUse">
              <stop stop-color="#e2e8f0" />
              <stop offset="1" stop-color="#94a3b8" />
            </linearGradient>
          </defs>
          <rect x="0" y="0" width="96" height="96" rx="6" fill="url(#hone-v7-plate)" />
          <rect x="2" y="2" width="92" height="92" rx="4" fill="none" stroke="rgba(148, 163, 184, 0.24)" stroke-width="2" />
          <path d="m34 49 22-28 15 8-22 27Z" fill="url(#hone-v7-stone)" />
          <path d="M34 49v10l15 8V56Z" fill="#1d4ed8" />
          <path d="M49 56v11l22-28V29Z" fill="#1e3a8a" />
          <path d="m32 36 43 19-4 3-43-18Z" fill="#f8fafc" />
          <path d="m28 40 43 18v10L28 53Z" fill="url(#hone-v7-blade)" />
        </svg>
      </span>
      {!props.markOnly ? <span class="hone-brand-word">HONE</span> : null}
    </span>
  );
}
