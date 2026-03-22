import { Show, type ParentProps } from "solid-js"

export function EmptyState(props: ParentProps<{ title: string; description?: string; action?: unknown }>) {
  return (
    <div class="flex h-full min-h-0 flex-col items-center justify-center rounded-[28px] border border-dashed border-[color:var(--border)] bg-[linear-gradient(180deg,rgba(255,255,255,0.03),rgba(255,255,255,0.01))] px-8 py-8 text-center">
      <div class="text-xl font-semibold text-[color:var(--text-primary)]">{props.title}</div>
      <Show when={props.description}>
        <div class="mt-3 max-w-md text-sm leading-6 text-[color:var(--text-secondary)]">{props.description}</div>
      </Show>
      <Show when={props.action}>
        <div class="mt-6">{props.action as any}</div>
      </Show>
    </div>
  )
}
