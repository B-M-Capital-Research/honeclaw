// Shared TOC + back-to-top for the long /terms and /privacy pages.
// Mobile-first: TOC is a <details> the user can collapse, back-to-top
// is a fixed button that only appears after the user scrolls past the hero.

import { For, Show, createSignal, onCleanup, onMount } from "solid-js"

export function sectionAnchor(index: number): string {
  return `section-${index + 1}`
}

export function LegalToc(props: { sections: { title: string }[] }) {
  return (
    <details
      style={{
        "margin-bottom": "32px",
        padding: "14px 18px",
        "border-radius": "12px",
        background: "#f8fafc",
        border: "1px solid rgba(15,23,42,0.06)",
      }}
    >
      <summary
        style={{
          "font-size": "13px",
          "font-weight": "700",
          color: "#475569",
          cursor: "pointer",
          "list-style-position": "outside",
        }}
      >
        目录 / Contents
      </summary>
      <ol
        style={{
          "margin-top": "12px",
          "padding-left": "20px",
          display: "flex",
          "flex-direction": "column",
          gap: "6px",
          "font-size": "13.5px",
          "line-height": "1.45",
        }}
      >
        <For each={props.sections}>
          {(s, i) => (
            <li>
              <a
                href={`#${sectionAnchor(i())}`}
                style={{
                  color: "#334155",
                  "text-decoration": "none",
                }}
              >
                {s.title}
              </a>
            </li>
          )}
        </For>
      </ol>
    </details>
  )
}

export function BackToTop() {
  const [visible, setVisible] = createSignal(false)

  onMount(() => {
    const onScroll = () => setVisible(window.scrollY > 600)
    window.addEventListener("scroll", onScroll, { passive: true })
    onScroll()
    onCleanup(() => window.removeEventListener("scroll", onScroll))
  })

  return (
    <Show when={visible()}>
      <button
        type="button"
        onClick={() =>
          window.scrollTo({ top: 0, left: 0, behavior: "smooth" })
        }
        aria-label="回到顶部"
        style={{
          position: "fixed",
          right: "20px",
          bottom: "calc(20px + env(safe-area-inset-bottom))",
          width: "44px",
          height: "44px",
          "border-radius": "999px",
          background: "#0f172a",
          color: "#fff",
          border: "none",
          cursor: "pointer",
          "box-shadow": "0 6px 20px rgba(15,23,42,0.25)",
          display: "inline-flex",
          "align-items": "center",
          "justify-content": "center",
          "z-index": "150",
        }}
      >
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.4"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M12 19V5M5 12l7-7 7 7" />
        </svg>
      </button>
    </Show>
  )
}
