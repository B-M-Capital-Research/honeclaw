import { For, Show, createSignal, type ParentProps } from "solid-js"
import { createSimpleContext } from "./helper"

type ToastItem = {
  id: string
  title: string
  description?: string
}

type ToastContextValue = {
  show: (title: string, description?: string) => void
}

const [ToastContextProvider, useToast] = createSimpleContext<ToastContextValue>("Toast")

export { useToast }

export function ToastProvider(props: ParentProps) {
  const [items, setItems] = createSignal<ToastItem[]>([])

  const remove = (id: string) => {
    setItems((current) => current.filter((item) => item.id !== id))
  }

  const show = (title: string, description?: string) => {
    const id = typeof crypto !== "undefined" && "randomUUID" in crypto ? crypto.randomUUID() : String(Date.now())
    setItems((current) => [...current, { id, title, description }])
    window.setTimeout(() => remove(id), 2600)
  }

  return (
    <ToastContextProvider value={{ show }}>
      {props.children}
      <div class="pointer-events-none fixed right-4 top-4 z-50 flex w-[320px] flex-col gap-3">
        <For each={items()}>
          {(item) => (
            <div class="rounded-2xl border border-white/10 bg-[color:var(--panel)] p-4 text-sm shadow-2xl backdrop-blur">
              <div class="font-semibold text-[color:var(--text-primary)]">{item.title}</div>
              <Show when={item.description}>
                <div class="mt-1 text-[color:var(--text-secondary)]">{item.description}</div>
              </Show>
            </div>
          )}
        </For>
      </div>
    </ToastContextProvider>
  )
}
