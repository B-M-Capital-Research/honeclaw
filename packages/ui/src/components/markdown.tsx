import { createResource, createSignal } from "solid-js"
import { useMarked } from "../context/marked"

export function Markdown(props: { text: string; class?: string }) {
  const marked = useMarked()
  const [host, setHost] = createSignal<HTMLDivElement>()
  const [html] = createResource(
    () => props.text,
    (value) => marked.parse(value),
  )

  return (
    <div
      ref={setHost}
      class={["hf-markdown", props.class].filter(Boolean).join(" ")}
      innerHTML={html() ?? ""}
    />
  )
}
