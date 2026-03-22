import type { ParentProps } from "solid-js"

export function Badge(props: ParentProps<{ tone?: "accent" | "neutral" }>) {
  const tone = props.tone ?? "neutral"
  return (
    <span
      class={[
        "inline-flex items-center rounded-full px-2.5 py-1 text-xs font-medium",
        tone === "accent"
          ? "bg-[color:var(--accent-soft)] text-[color:var(--accent)]"
          : "bg-white/6 text-[color:var(--text-secondary)]",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {props.children}
    </span>
  )
}
