import type { JSX } from "solid-js"

export function Input(props: JSX.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      {...props}
      class={[
        "w-full rounded-md border border-[color:var(--border)] bg-[color:var(--surface)] px-3 py-2 text-sm text-[color:var(--text-primary)] outline-none placeholder:text-[color:var(--text-muted)] focus:border-[color:var(--accent)] focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]",
        props.class,
      ]
        .filter(Boolean)
        .join(" ")}
    />
  )
}
