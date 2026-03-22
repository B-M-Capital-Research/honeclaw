import type { JSX } from "solid-js"

export function Textarea(props: JSX.TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return (
    <textarea
      {...props}
      class={[
        "w-full resize-none rounded-2xl border border-[color:var(--border)] bg-[color:var(--surface)] px-4 py-3 text-sm text-[color:var(--text-primary)] outline-none placeholder:text-[color:var(--text-muted)] focus:border-[color:var(--accent)]",
        props.class,
      ]
        .filter(Boolean)
        .join(" ")}
    />
  )
}
