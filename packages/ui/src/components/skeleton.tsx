export function Skeleton(props: { class?: string }) {
  return <div class={["animate-pulse rounded-xl bg-white/6", props.class].filter(Boolean).join(" ")} />
}
