import type { JSX } from "solid-js"

type Props = JSX.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "ghost" | "subtle" | "outline"
}

export function Button(props: Props) {
  const variant = props.variant ?? "primary"
  const className = () =>
    [
      "inline-flex items-center justify-center gap-2 rounded-md px-4 py-2 text-sm font-medium transition disabled:cursor-not-allowed disabled:opacity-50",
      variant === "primary" && "bg-[color:var(--accent)] text-white hover:bg-[color:var(--accent-strong)]",
      variant === "outline" && "border border-[color:var(--border)] bg-transparent text-[color:var(--text-primary)] hover:border-[color:var(--accent)] hover:text-[color:var(--accent)]",
      variant === "ghost" &&
      "border border-transparent bg-transparent text-[color:var(--text-primary)] hover:bg-black/5",
      variant === "subtle" &&
      "bg-black/5 text-[color:var(--text-primary)] hover:bg-black/10",
      props.class,
    ]
      .filter(Boolean)
      .join(" ")

  return (
    <button {...props} class={className()}>
      {props.children}
    </button>
  )
}
